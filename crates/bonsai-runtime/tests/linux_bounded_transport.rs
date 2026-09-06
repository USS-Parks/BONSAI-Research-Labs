#![cfg(target_os = "linux")]

use bonsai_runtime::{ChildTransport, ProcessCommand, TransportError, TransportLimits};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const OPERATION: Duration = Duration::from_millis(80);
// One second declared cleanup plus a measured scheduling allowance, not a longer I/O timeout.
const OBSERVED_BOUND: Duration = Duration::from_millis(1300);

fn fixture(mode: &str) -> (ChildTransport, Vec<u32>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../python/bonsai-reference/tests/fixtures/bounded_transport_adapter.py");
    let mut transport = ChildTransport::spawn(
        &ProcessCommand::new("python3")
            .argument(path.into_os_string())
            .argument(mode),
        TransportLimits {
            maximum_frame_bytes: 16 * 1024 * 1024,
            pending_frame_capacity: 2,
            retained_stderr_bytes: 2048,
        },
    )
    .expect("spawn bounded fixture");
    let ready = transport
        .receive(Duration::from_secs(2))
        .expect("ready frame")
        .expect("ready bytes");
    let pids: Vec<u32> = serde_json::from_slice(&ready).expect("fixture process identities");
    assert_eq!(pids[0], transport.process_id());
    (transport, pids)
}

fn absent(pids: &[u32]) {
    let started = Instant::now();
    while pids
        .iter()
        .any(|pid| PathBuf::from(format!("/proc/{pid}")).exists())
    {
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "surviving fixture processes: {pids:?}"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

fn bound(started: Instant, label: &str, pids: &[u32]) {
    let elapsed = started.elapsed();
    eprintln!(
        "BX-02 {label}: pids={pids:?} elapsed_ms={}",
        elapsed.as_millis()
    );
    assert!(
        elapsed < OBSERVED_BOUND,
        "{label} exceeded its deadline + cleanup allowance: {elapsed:?}"
    );
    absent(pids);
}

fn count(path: &str) -> usize {
    fs::read_dir(path).expect("OS process inventory").count()
}

fn external_cancellation_cases() {
    for writing in [false, true] {
        let (mut pending, pids) = fixture("no_read");
        let cancellation = pending.cancellation_handle();
        let trigger = thread::spawn(move || {
            thread::sleep(Duration::from_millis(40));
            cancellation.cancel();
        });
        let started = Instant::now();
        let result = if writing {
            pending.send_with_timeout(&vec![1; 8 * 1024 * 1024], Duration::from_secs(10))
        } else {
            pending.receive(Duration::from_secs(10)).map(|_| ())
        };
        assert_eq!(result, Err(TransportError::Cancelled));
        trigger.join().expect("cancellation sender");
        drop(pending);
        bound(started, "external cancellation", &pids);
    }

    let (pending_shutdown, pids) = fixture("no_read");
    let cancellation = pending_shutdown.cancellation_handle();
    let trigger = thread::spawn(move || {
        thread::sleep(Duration::from_millis(40));
        cancellation.cancel();
    });
    let started = Instant::now();
    assert_eq!(
        pending_shutdown.shutdown(Duration::from_secs(10)),
        Err(TransportError::Cancelled)
    );
    trigger.join().expect("shutdown cancellation sender");
    bound(started, "shutdown cancellation", &pids);
}

#[test]
fn live_pipe_deadlines_descendants_cancellation_and_repeated_cleanup() {
    // Bound the test itself even if a regression reintroduces an unbounded join/write.
    // Fixture processes also self-expire; this watchdog cannot leave an infinite helper.
    let (finished, receiver) = mpsc::channel();
    let watchdog = thread::spawn(move || {
        if receiver.recv_timeout(Duration::from_secs(30)) == Err(mpsc::RecvTimeoutError::Timeout) {
            eprintln!("BX-02 outer watchdog expired");
            std::process::exit(124);
        }
    });
    let threads_before = count("/proc/self/task");
    let handles_before = count("/proc/self/fd");

    let (mut blocked, pids) = fixture("no_read");
    let started = Instant::now();
    assert_eq!(
        blocked.send_with_timeout(&vec![1; 8 * 1024 * 1024], OPERATION),
        Err(TransportError::WriteTimeout)
    );
    assert_eq!(blocked.failures()[0].code, "TRANSPORT_WRITE_TIMEOUT");
    let outcome = blocked
        .shutdown(Duration::from_secs(1))
        .expect("already contained child finalizes");
    assert_eq!(outcome.failures[0].code, "TRANSPORT_WRITE_TIMEOUT");
    bound(started, "blocked stdin", &pids);

    let (mut partial, pids) = fixture("partial_hang");
    let started = Instant::now();
    assert_eq!(partial.receive(OPERATION), Err(TransportError::ReadTimeout));
    let outcome = partial
        .shutdown(Duration::from_secs(1))
        .expect("read timeout containment finalizes");
    assert_eq!(outcome.failures[0].code, "TRANSPORT_READ_TIMEOUT");
    bound(started, "partial frame", &pids);

    let (mut tree, pids) = fixture("grandchild");
    let started = Instant::now();
    tree.cancel().expect("cancel entire group");
    assert!(
        tree.failures()
            .iter()
            .all(|failure| failure.code == "TRANSPORT_CANCELLED")
    );
    drop(tree);
    bound(started, "grandchild cancellation", &pids);

    let (exited, pids) = fixture("exited_parent");
    let started = Instant::now();
    assert_eq!(
        exited
            .shutdown(Duration::from_secs(1))
            .expect("parent exit with retained pipes")
            .exit_code,
        Some(0)
    );
    bound(started, "exited parent inherited pipes", &pids);

    let (mut flood, pids) = fixture("stderr_flood");
    let ready = Instant::now();
    while flood.stderr_snapshot().total_bytes < 65_536 {
        assert!(
            ready.elapsed() < Duration::from_secs(2),
            "stderr flood started"
        );
        thread::sleep(Duration::from_millis(2));
    }
    let capture = flood.stderr_snapshot();
    assert_eq!(capture.retained.len(), 2048);
    assert!(capture.truncated);
    let started = Instant::now();
    flood.cancel().expect("cancel stderr flood");
    drop(flood);
    bound(started, "stderr flood", &pids);

    external_cancellation_cases();

    for _ in 0..20 {
        let (normal, pids) = fixture("normal");
        assert_eq!(
            normal
                .shutdown(Duration::from_secs(1))
                .expect("normal exit")
                .exit_code,
            Some(0)
        );
        absent(&pids);
    }
    let threads_after = count("/proc/self/task");
    let handles_after = count("/proc/self/fd");
    eprintln!(
        "BX-02 repeated cleanup: threads={threads_before}->{threads_after}, fds={handles_before}->{handles_after}"
    );
    assert!(threads_after <= threads_before, "worker thread leak");
    assert!(handles_after <= handles_before, "pipe handle leak");
    finished.send(()).expect("watchdog completion");
    watchdog.join().expect("watchdog joined");
}
