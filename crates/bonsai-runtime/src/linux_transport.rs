//! Safe Linux pipe and owned process-group operations (BX-02).

use crate::TransportError;
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};
use std::io;
use std::os::fd::AsFd;
use std::process::Child;

pub(crate) fn nonblocking(pipe: &impl AsFd) -> Result<(), TransportError> {
    let result = fcntl_getfl(pipe).and_then(|flags| fcntl_setfl(pipe, flags | OFlags::NONBLOCK));
    result.map_err(|error| TransportError::Io(io::Error::from(error).kind()))
}

pub(crate) fn kill_group(child: &Child) -> Result<(), TransportError> {
    let pid = Pid::from_child(child);
    // Never permit the special PID 1 semantics of kill(-1, signal).
    if pid.is_init() {
        return Err(TransportError::Io(io::ErrorKind::InvalidInput));
    }
    match kill_process_group(pid, Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(error) => Err(TransportError::Io(io::Error::from(error).kind())),
    }
}

/// Confirm the killed owned group no longer exists, including descendant zombies.
pub(crate) fn wait_group_exit(
    child: &Child,
    timeout: std::time::Duration,
) -> Result<(), TransportError> {
    let pid = Pid::from_child(child);
    if pid.is_init() {
        return Err(TransportError::Io(io::ErrorKind::InvalidInput));
    }
    let started = std::time::Instant::now();
    loop {
        match test_kill_process_group(pid) {
            Err(rustix::io::Errno::SRCH) => return Ok(()),
            Err(error) => return Err(TransportError::Io(io::Error::from(error).kind())),
            Ok(()) => {}
        }
        if started.elapsed() >= timeout {
            return Err(TransportError::CleanupTimeout);
        }
        std::thread::sleep(crate::interruptible_io::POLL_INTERVAL);
    }
}
