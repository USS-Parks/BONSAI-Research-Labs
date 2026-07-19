use bonsai_runtime::{ChildTransport, ProcessCommand, TransportError, TransportLimits};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn python() -> OsString {
    let root = workspace_root();
    let local = if cfg!(windows) {
        root.join(".venv/Scripts/python.exe")
    } else {
        root.join(".venv/bin/python")
    };
    let candidates = std::env::var_os("PYTHON")
        .into_iter()
        .chain(local.exists().then(|| local.into_os_string()))
        .chain([OsString::from("python3"), OsString::from("python")]);
    for candidate in candidates {
        if Command::new(&candidate)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return candidate;
        }
    }
    panic!("BR-02 requires Python on every supported CI host");
}

fn child(mode: &str, limits: TransportLimits) -> ChildTransport {
    let fixture =
        workspace_root().join("python/bonsai-reference/tests/fixtures/transport_adapter.py");
    ChildTransport::spawn(
        &ProcessCommand::new(python())
            .argument(fixture.into_os_string())
            .argument(mode),
        limits,
    )
    .expect("spawn fixture adapter")
}

#[test]
fn rust_python_transport_separates_protocol_and_stderr_and_shuts_down_cleanly() {
    let mut transport = child("echo", TransportLimits::default());
    transport.send(b"rust-to-python").expect("send frame");
    assert_eq!(
        transport.receive(Duration::from_secs(2)),
        Ok(Some(b"rust-to-python".to_vec()))
    );
    let outcome = transport
        .shutdown(Duration::from_secs(2))
        .expect("clean shutdown");
    assert_eq!(outcome.exit_code, Some(0));
    assert!(outcome.failures.is_empty());
    assert_eq!(outcome.stderr.retained, b"fixture-stderr\n");
    assert!(!outcome.stderr.truncated);
}

#[test]
fn malformed_partial_and_stalled_senders_are_contained_and_recorded() {
    let mut partial = child("partial", TransportLimits::default());
    assert_eq!(
        partial.receive(Duration::from_secs(2)),
        Err(TransportError::PayloadPartial)
    );
    assert_eq!(partial.failures()[0].code, "TRANSPORT_PAYLOAD_PARTIAL");

    let mut oversized = child(
        "oversized",
        TransportLimits {
            maximum_frame_bytes: 1024,
            ..TransportLimits::default()
        },
    );
    assert_eq!(
        oversized.receive(Duration::from_secs(2)),
        Err(TransportError::FrameTooLarge {
            declared: 1025,
            maximum: 1024,
        })
    );
    assert_eq!(oversized.failures()[0].code, "TRANSPORT_FRAME_TOO_LARGE");

    let mut stalled = child("stalled", TransportLimits::default());
    assert_eq!(
        stalled.receive(Duration::from_millis(50)),
        Err(TransportError::ReadTimeout)
    );
    assert_eq!(stalled.failures()[0].code, "TRANSPORT_READ_TIMEOUT");
}

#[test]
fn flood_sender_cannot_exceed_the_pending_queue_bound() {
    let limits = TransportLimits {
        maximum_frame_bytes: 1024,
        pending_frame_capacity: 1,
        retained_stderr_bytes: 1024,
    };
    let mut flood = child("flood", limits);
    let started = std::time::Instant::now();
    while !flood
        .stderr_snapshot()
        .retained
        .windows(b"flood-ready".len())
        .any(|window| window == b"flood-ready")
    {
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "flood fixture start"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(Duration::from_millis(100));
    let mut observed = None;
    for _ in 0..4 {
        match flood.receive(Duration::from_millis(200)) {
            Err(error) => {
                observed = Some(error);
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) => break,
        }
    }
    assert_eq!(observed, Some(TransportError::BackpressureExceeded));
    assert_eq!(flood.failures()[0].code, "TRANSPORT_BACKPRESSURE_EXCEEDED");
}
