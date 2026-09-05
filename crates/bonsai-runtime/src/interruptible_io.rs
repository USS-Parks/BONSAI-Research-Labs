//! Cancellable pipe polling; Linux pipes are made nonblocking before use.

#[cfg(target_os = "linux")]
use std::io::Write;
use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::Instant;

pub(crate) const POLL_INTERVAL: Duration = Duration::from_millis(2);

pub(crate) struct InterruptibleReader<R> {
    inner: R,
    stopped: Arc<AtomicBool>,
}

impl<R> InterruptibleReader<R> {
    pub(crate) const fn new(inner: R, stopped: Arc<AtomicBool>) -> Self {
        Self { inner, stopped }
    }
}

impl<R: Read> Read for InterruptibleReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.stopped.load(Ordering::Acquire) {
                return Err(io::ErrorKind::ConnectionAborted.into());
            }
            match self.inner.read(buffer) {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(POLL_INTERVAL);
                }
                result => return result,
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) struct DeadlineWriter<'a, W> {
    pub(crate) inner: &'a mut W,
    pub(crate) stopped: &'a AtomicBool,
    pub(crate) started: Instant,
    pub(crate) timeout: Duration,
}

#[cfg(target_os = "linux")]
fn retry<T>(
    stopped: &AtomicBool,
    started: Instant,
    timeout: Duration,
    mut operation: impl FnMut() -> io::Result<T>,
) -> io::Result<T> {
    loop {
        if stopped.load(Ordering::Acquire) {
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        if started.elapsed() >= timeout {
            return Err(io::ErrorKind::TimedOut.into());
        }
        match operation() {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(POLL_INTERVAL.min(timeout.saturating_sub(started.elapsed())));
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            result => return result,
        }
    }
}

#[cfg(target_os = "linux")]
impl<W: Write> Write for DeadlineWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        retry(self.stopped, self.started, self.timeout, || {
            self.inner.write(bytes)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        retry(self.stopped, self.started, self.timeout, || {
            self.inner.flush()
        })
    }
}
