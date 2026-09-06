//! Live delegated-cgroup probe for the governed adapter launch seam.
#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bonsai_platform::linux_authority::{LinuxAuthority, LinuxLimits};
    use bonsai_runtime::{ChildTransport, ProcessCommand, TransportLimits};
    use std::path::Path;
    use std::time::Duration;

    let membership = std::fs::read_to_string("/proc/self/cgroup")?;
    let relative = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .ok_or("missing unified cgroup")?;
    let current = Path::new("/sys/fs/cgroup").join(relative.trim_start_matches('/'));
    let parent = current.parent().ok_or("missing delegated parent")?;
    let mut authority = LinuxAuthority::create(
        parent,
        &format!("bonsai-transport-{}", std::process::id()),
        LinuxLimits {
            cpu_quota_usec: 100_000,
            cpu_period_usec: 100_000,
            memory_max_bytes: 128 * 1024 * 1024,
            pids_max: 8,
        },
    )?;
    let script = r"import json,os,struct,sys
payload=json.dumps({'cwd':os.getcwd(),'environment':dict(os.environ),'cgroup':open('/proc/self/cgroup').read(),'pid':os.getpid()}).encode()
sys.stdout.buffer.write(struct.pack('<I',len(payload))+payload)
sys.stdout.buffer.flush()
sys.stdin.buffer.read()
";
    let command = ProcessCommand::new("/usr/bin/python3")
        .argument("-I")
        .argument("-B")
        .argument("-c")
        .argument(script)
        .current_directory("/tmp")
        .clear_environment()
        .environment("BONSAI_LAUNCH_PROBE", "literal spaces; $not_expanded");
    let mut transport =
        ChildTransport::spawn_governed(&command, TransportLimits::default(), &authority)?;
    let payload = transport
        .receive(Duration::from_secs(5))?
        .ok_or("adapter EOF")?;
    let value: serde_json::Value = serde_json::from_slice(&payload)?;
    assert_eq!(value["cwd"], "/tmp");
    assert_eq!(
        value["environment"]["BONSAI_LAUNCH_PROBE"],
        "literal spaces; $not_expanded"
    );
    assert!(value["environment"].get("PATH").is_none());
    assert_eq!(
        value["pid"].as_u64(),
        Some(u64::from(transport.process_id()))
    );
    let controls = authority.controls()?;
    let child_cgroup = value["cgroup"].as_str().ok_or("missing child membership")?;
    assert!(child_cgroup.contains(controls.cgroup_path.trim_start_matches("/sys/fs/cgroup")));
    let before = authority.sample()?;
    assert!(before.member_pids.contains(&transport.process_id()));
    let shutdown = transport.shutdown(Duration::from_secs(1))?;
    let after = authority.cleanup(Duration::from_secs(1))?;
    assert!(!after.populated && after.pids_current == 0 && after.member_pids.is_empty());
    println!(
        "{}",
        serde_json::json!({"probe":"governed-transport", "child":value,
        "controls":controls, "before":before, "after":after, "shutdown":format!("{shutdown:?}")})
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("governed transport probe requires delegated Linux cgroup v2");
    std::process::exit(2);
}
