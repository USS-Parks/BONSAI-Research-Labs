//! Explicit live gate: never silently skips when delegation is absent.

#[cfg(not(target_os = "linux"))]
fn main() {
    panic!("BX-03 live authority gate requires Linux");
}

#[cfg(target_os = "linux")]
fn main() {
    live::run();
}

#[cfg(target_os = "linux")]
mod live {
    use bonsai_governor::enforcement::{HardControl, preflight_linux_authority};
    use bonsai_platform::capability::BackendError;
    use bonsai_platform::linux_authority::{LinuxAuthority, LinuxLimits};
    use serde_json::{Value, json};
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::path::{Path, PathBuf};
    use std::process::Child;
    use std::thread;
    use std::time::{Duration, Instant};

    pub fn run() {
        thread::spawn(|| {
            thread::sleep(Duration::from_secs(40));
            eprintln!("BX03_WATCHDOG_EXPIRED");
            std::process::exit(90);
        });
        let membership = fs::read_to_string("/proc/self/cgroup").expect("self cgroup");
        let relative = membership
            .lines()
            .find_map(|line| line.strip_prefix("0::"))
            .expect("v2");
        let current = Path::new("/sys/fs/cgroup").join(relative.trim_start_matches('/'));
        if std::env::args().any(|arg| arg == "denied") {
            let error = LinuxAuthority::create(&current, "bonsai-denied-probe", limits(4))
                .expect_err("nondelegated scope must reject");
            assert_eq!(error, BackendError::Privilege);
            println!("{}", json!({"denied": error.to_string(), "path": current}));
            return;
        }
        assert_eq!(current.file_name().expect("supervisor"), "supervisor");
        let root = current.parent().expect("delegated parent");
        assert!(
            LinuxAuthority::create(Path::new("/sys/fs/cgroup"), "bonsai-global", limits(4))
                .is_err()
        );
        assert!(LinuxAuthority::create(root, "../escape", limits(4)).is_err());
        tree_and_cpu(root);
        memory_crossing(root);
        pid_crossing(root);
        println!("BX03_LIVE_GATE_PASS");
    }

    fn limits(pids_max: u64) -> LinuxLimits {
        LinuxLimits {
            cpu_quota_usec: 10_000,
            cpu_period_usec: 100_000,
            memory_max_bytes: 64 * 1024 * 1024,
            pids_max,
        }
    }

    fn create(root: &Path, mode: &str, pids: u64) -> LinuxAuthority {
        let authority = LinuxAuthority::create(root, &format!("bonsai-{mode}"), limits(pids))
            .expect("real controller writes");
        let controls =
            ["cgroup.cpu.max", "cgroup.memory.max", "cgroup.pids.max"].map(|id| HardControl {
                control_id: id.to_owned(),
                required: true,
            });
        assert!(
            preflight_linux_authority(&authority, &controls)
                .expect("live preflight")
                .admitted
        );
        assert!(
            !preflight_linux_authority(
                &authority,
                &[HardControl {
                    control_id: "cgroup.io.max".to_owned(),
                    required: true,
                }]
            )
            .expect("unsupported preflight")
            .admitted
        );
        assert!(
            !authority
                .capabilities()
                .expect("matrix")
                .physical_acceptance
        );
        println!(
            "{}",
            json!({"mode": mode, "controls": authority.controls().expect("readbacks"),
            "capabilities": authority.capabilities().expect("capabilities")})
        );
        authority
    }

    fn launch(authority: &LinuxAuthority, mode: &str) -> (Child, Vec<u32>, Value) {
        let mut child = authority
            .spawn(
                "python3",
                &["evidence/verification/bx-03/workload.py", mode],
            )
            .expect("attached launch");
        let mut line = String::new();
        BufReader::new(child.stdout.as_mut().expect("stdout"))
            .read_line(&mut line)
            .expect("fixture ready");
        let ready: Value = serde_json::from_str(&line).expect("ready JSON");
        let pids: Vec<u32> = ready["pids"]
            .as_array()
            .expect("pids")
            .iter()
            .map(|value| u32::try_from(value.as_u64().expect("PID")).expect("PID range"))
            .collect();
        assert_eq!(pids[0], child.id());
        let sample = authority.sample().expect("membership");
        let path = authority.controls().expect("controls").cgroup_path;
        let relative = path.strip_prefix("/sys/fs/cgroup").expect("cgroup path");
        for pid in &pids {
            assert!(sample.member_pids.contains(pid));
            assert_eq!(
                fs::read_to_string(format!("/proc/{pid}/cgroup"))
                    .expect("member cgroup")
                    .trim(),
                format!("0::{relative}")
            );
        }
        println!(
            "{}",
            json!({"mode": mode, "ready": ready, "membership": sample})
        );
        (child, pids, ready)
    }

    fn finish(authority: &mut LinuxAuthority, child: &mut Child, pids: &[u32]) {
        let path = PathBuf::from(authority.controls().expect("controls").cgroup_path);
        let start = Instant::now();
        authority.terminate().expect("terminate owned tree");
        while child.try_wait().expect("reap").is_none() {
            assert!(start.elapsed() < Duration::from_secs(3), "leader survived");
            thread::sleep(Duration::from_millis(5));
        }
        let final_usage = authority
            .cleanup(Duration::from_secs(2))
            .expect("empty cgroup cleanup");
        while pids
            .iter()
            .any(|pid| Path::new(&format!("/proc/{pid}")).exists())
        {
            assert!(
                start.elapsed() < Duration::from_secs(3),
                "descendant survived or unreaped"
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!path.exists());
        assert!(authority.capabilities().is_err());
        println!(
            "{}",
            json!({"cleanup": final_usage, "elapsed_ms": start.elapsed().as_millis(), "pids_absent": pids})
        );
    }

    fn tree_and_cpu(root: &Path) {
        let mut authority = create(root, "tree", 8);
        let (mut child, pids, _) = launch(&authority, "tree");
        assert_eq!(pids.len(), 3);
        let before = authority.sample().expect("before");
        let start = Instant::now();
        thread::sleep(Duration::from_secs(2));
        let after = authority.sample().expect("after");
        let wall_usec = u64::try_from(start.elapsed().as_micros()).expect("wall");
        let cpu_usec = after.cpu_usage_usec - before.cpu_usage_usec;
        assert!(cpu_usec > 10_000);
        assert!(
            cpu_usec <= wall_usec / 10 + 70_000,
            "quota exceeded: {cpu_usec}/{wall_usec}"
        );
        assert!(after.cpu_throttled_periods > before.cpu_throttled_periods);
        assert!(after.cpu_throttled_usec > before.cpu_throttled_usec);
        assert_eq!(after.pids_current, 3);
        println!(
            "{}",
            json!({"cpu_before": before, "cpu_after": after, "wall_usec": wall_usec, "cpu_delta_usec": cpu_usec})
        );
        // External controller alteration must invalidate governor admission.
        let path = authority.controls().expect("controls").cgroup_path;
        fs::write(Path::new(&path).join("pids.max"), "7").expect("alter owned fixture control");
        assert!(preflight_linux_authority(&authority, &[]).is_err());
        fs::write(Path::new(&path).join("pids.max"), "8").expect("restore fixture control");
        finish(&mut authority, &mut child, &pids);
    }

    fn memory_crossing(root: &Path) {
        let mut authority = create(root, "memory", 4);
        let (mut child, pids, _) = launch(&authority, "memory");
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(b"allocate\n")
            .expect("cross memory limit");
        let start = Instant::now();
        while child.try_wait().expect("OOM exit").is_none() {
            assert!(
                start.elapsed() < Duration::from_secs(8),
                "memory limit did not terminate"
            );
            thread::sleep(Duration::from_millis(10));
        }
        // Observe the controller's complete termination before explicit cleanup;
        // otherwise cleanup could hide a partially surviving OOM workload.
        let sample = loop {
            let sample = authority.sample().expect("OOM counters");
            if !sample.populated && sample.pids_current == 0 {
                break sample;
            }
            assert!(
                start.elapsed() < Duration::from_secs(8),
                "OOM descendants survived"
            );
            thread::sleep(Duration::from_millis(10));
        };
        assert!(sample.memory_max_events > 0);
        assert!(sample.memory_oom_kills >= 2);
        assert!(sample.memory_peak_bytes >= 63 * 1024 * 1024);
        println!("{}", json!({"memory_crossing": sample}));
        finish(&mut authority, &mut child, &pids);
    }

    fn pid_crossing(root: &Path) {
        let mut authority = create(root, "pids", 4);
        let (mut child, pids, ready) = launch(&authority, "pids");
        assert_eq!(ready["fork_denied"], true);
        assert_eq!(pids.len(), 4);
        let sample = authority.sample().expect("pids events");
        assert_eq!(sample.pids_current, 4);
        assert!(sample.pids_max_events > 0);
        println!("{}", json!({"pid_crossing": sample}));
        finish(&mut authority, &mut child, &pids);
    }
}
