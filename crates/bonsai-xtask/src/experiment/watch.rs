use super::{Inputs, number};
use bonsai_platform::linux_authority::LinuxAuthority;
use serde_json::Value;
use std::sync::mpsc;
use std::time::{Duration, Instant};

// Only these two run-owned authorities are visible to the watcher. Termination
// repeats until the operation returns, covering a deadline racing child launch.
pub(super) fn operation(
    inputs: &Inputs,
    began: Instant,
    agent: &LinuxAuthority,
    environment: &LinuxAuthority,
    run: impl FnOnce() -> Result<Value, String>,
) -> Result<Value, String> {
    let limit = Duration::from_nanos(number(
        &inputs.manifest["resource_profile"],
        "wall_time_limit_ns",
    )?);
    if let Some(reason) = interrupted(inputs, began, limit) {
        return Err(reason.into());
    }
    std::thread::scope(|scope| {
        let (stop, stopped) = mpsc::sync_channel(1);
        let watcher = scope.spawn(move || {
            let mut reason = None;
            let mut failure = None;
            loop {
                reason = reason.or_else(|| interrupted(inputs, began, limit));
                if reason.is_some() {
                    for authority in [agent, environment] {
                        if let Err(error) = authority.terminate() {
                            failure.get_or_insert_with(|| error.to_string());
                        }
                    }
                }
                let wait = limit
                    .saturating_sub(began.elapsed())
                    .min(Duration::from_millis(10));
                let wait = if reason.is_some() {
                    Duration::from_millis(10)
                } else {
                    wait
                };
                match stopped.recv_timeout(wait) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
            (reason, failure)
        });
        let result = run();
        let _ = stop.send(());
        let (reason, failure) = watcher.join().map_err(|_| "RUN_WATCHER_FAILED")?;
        if let Some(failure) = failure {
            return Err(format!("RUN_TERMINATION_FAILED:{failure}"));
        }
        if let Some(reason) = reason.or_else(|| interrupted(inputs, began, limit)) {
            return Err(reason.into());
        }
        result
    })
}

fn interrupted(inputs: &Inputs, began: Instant, limit: Duration) -> Option<&'static str> {
    if began.elapsed() >= limit {
        Some("RUN_WALL_LIMIT_EXCEEDED")
    } else if inputs.cancel.as_ref().is_some_and(|path| path.exists()) {
        Some("RUN_CANCELLED")
    } else {
        None
    }
}
