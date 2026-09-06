//! Scoped cgroup v2 authority. Construction writes and reads back real controls.
//! The supervisor must already run inside an explicitly delegated subtree.

use crate::capability::{BackendError, CapabilityMatrix, HostClass, Support, control};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const MOUNT: &str = "/sys/fs/cgroup";
const POLL: Duration = Duration::from_millis(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct LinuxLimits {
    /// Fair-scheduler CPU bandwidth, not a cumulative CPU-time budget.
    pub cpu_quota_usec: u64,
    pub cpu_period_usec: u64,
    pub memory_max_bytes: u64,
    /// Kernel tasks, including threads, rather than only process leaders.
    pub pids_max: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinuxControlEvidence {
    pub cgroup_path: String,
    pub limits: LinuxLimits,
    pub cpu_max: String,
    pub memory_max: u64,
    pub memory_swap_max: u64,
    pub memory_oom_group: u64,
    pub pids_max: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinuxUsage {
    pub cpu_usage_usec: u64,
    pub cpu_throttled_periods: u64,
    pub cpu_throttled_usec: u64,
    pub memory_current_bytes: u64,
    pub memory_peak_bytes: u64,
    pub memory_max_events: u64,
    pub memory_oom_kills: u64,
    pub pids_current: u64,
    pub pids_max_events: u64,
    pub populated: bool,
    pub member_pids: Vec<u32>,
}

/// Owns one newly created leaf cgroup. It never adopts or deletes existing groups.
/// Explicit cleanup reports errors; Drop is only a bounded best-effort fallback.
#[derive(Debug)]
pub struct LinuxAuthority {
    path: PathBuf,
    limits: LinuxLimits,
    removed: bool,
}

impl LinuxAuthority {
    /// Create an owned leaf under an already delegated, empty parent.
    /// Only the current process's cgroup or its immediate parent is accepted;
    /// the global cgroup root is always refused. Existing names are not adopted.
    ///
    /// # Errors
    /// Fails on invalid limits/scope, missing controllers, denied writes, or
    /// mismatching controller readbacks. No measurement-only fallback exists.
    pub fn create(root: &Path, name: &str, limits: LinuxLimits) -> Result<Self, BackendError> {
        validate_limits(limits)?;
        if !name.starts_with("bonsai-")
            || name.len() > 80
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(BackendError::Identity);
        }
        let root = validate_root(root)?;
        // Opening checks actual delegated write permission without changing a value.
        OpenOptions::new()
            .write(true)
            .open(root.join("cgroup.subtree_control"))
            .map_err(|error| io_error(&error))?;
        if !read(&root, "cgroup.procs")?.is_empty() {
            return Err(BackendError::Identity);
        }
        let controllers = read(&root, "cgroup.controllers")?;
        if ["cpu", "memory", "pids"]
            .iter()
            .any(|wanted| !controllers.split_whitespace().any(|value| value == *wanted))
        {
            return Err(BackendError::Unsupported);
        }
        write(&root, "cgroup.subtree_control", "+cpu +memory +pids")?;
        let path = root.join(name);
        fs::create_dir(&path).map_err(|error| io_error(&error))?;
        let mut authority = Self {
            path,
            limits,
            removed: false,
        };
        if let Err(error) = authority.apply() {
            // No workload has been launched; never recursively remove anything.
            authority.removed = fs::remove_dir(&authority.path).is_ok();
            return Err(error);
        }
        Ok(authority)
    }

    fn apply(&self) -> Result<(), BackendError> {
        write(
            &self.path,
            "cpu.max",
            &format!(
                "{} {}",
                self.limits.cpu_quota_usec, self.limits.cpu_period_usec
            ),
        )?;
        write(
            &self.path,
            "memory.max",
            &self.limits.memory_max_bytes.to_string(),
        )?;
        write(&self.path, "memory.swap.max", "0")?;
        write(&self.path, "memory.oom.group", "1")?;
        write(&self.path, "pids.max", &self.limits.pids_max.to_string())?;
        // Whole-tree kill support is mandatory for this authority path.
        OpenOptions::new()
            .write(true)
            .open(self.path.join("cgroup.kill"))
            .map_err(|error| io_error(&error))?;
        self.controls().map(|_| ())
    }

    /// Re-read applied settings; changed controls invalidate the authority.
    ///
    /// # Errors
    /// Fails when readbacks differ or the group has been removed.
    pub fn controls(&self) -> Result<LinuxControlEvidence, BackendError> {
        let result = LinuxControlEvidence {
            cgroup_path: self.path.display().to_string(),
            limits: self.limits,
            cpu_max: read(&self.path, "cpu.max")?,
            memory_max: number(&self.path, "memory.max")?,
            memory_swap_max: number(&self.path, "memory.swap.max")?,
            memory_oom_group: number(&self.path, "memory.oom.group")?,
            pids_max: number(&self.path, "pids.max")?,
        };
        if result.cpu_max
            != format!(
                "{} {}",
                self.limits.cpu_quota_usec, self.limits.cpu_period_usec
            )
            || result.memory_max != self.limits.memory_max_bytes
            || result.memory_swap_max != 0
            || result.memory_oom_group != 1
            || result.pids_max != self.limits.pids_max
        {
            return Err(BackendError::Identity);
        }
        Ok(result)
    }

    /// Produce the existing governor capability input from verified readbacks.
    /// This does not certify physical-host acceptance or a whole experiment.
    ///
    /// # Errors
    /// Fails if a control was removed or altered after creation.
    pub fn capabilities(&self) -> Result<CapabilityMatrix, BackendError> {
        self.controls()?;
        self.sample()?;
        Ok(CapabilityMatrix::assembled(
            "linux-cgroup-v2",
            "linux",
            HostClass::Unknown,
            [
                "cgroup.cpu.stat",
                "cgroup.memory.current",
                "cgroup.pids.current",
            ]
            .into_iter()
            .map(|id| control(id, Support::Supported, "CGROUP_READBACK_VERIFIED"))
            .collect(),
            ["cgroup.cpu.max", "cgroup.memory.max", "cgroup.pids.max"]
                .into_iter()
                .map(|id| control(id, Support::Supported, "CGROUP_READBACK_VERIFIED"))
                .chain([control(
                    "cgroup.io.max",
                    Support::Unsupported,
                    "CGROUP_IO_LIMIT_UNIMPLEMENTED",
                )])
                .collect(),
            Support::Supported,
            "Scoped delegated cgroup; fair-scheduler bandwidth; swap disabled; host class requires separate evidence",
        ))
    }

    /// Launch through a minimal shell gate, attach before exec, verify membership,
    /// then release the executable. Descendants inherit membership from birth.
    /// The returned child's pipes and reaping are owned by the caller.
    ///
    /// # Errors
    /// Fails closed on launch, attachment, readback, or gate release failure.
    pub fn spawn(&self, program: &str, args: &[&str]) -> Result<Child, BackendError> {
        self.controls()?;
        let mut child = Command::new("/bin/sh")
            .args([
                "-c",
                "IFS= read -r gate && [ \"$gate\" = BONSAI_START ] || exit 78; exec \"$@\"",
                "bonsai-cgroup-launch",
                program,
            ])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| io_error(&error))?;
        let result = (|| {
            self.attach_gated_child(&child)?;
            child
                .stdin
                .as_mut()
                .ok_or(BackendError::Io)?
                .write_all(b"BONSAI_START\n")
                .map_err(|error| io_error(&error))
        })();
        if let Err(error) = result {
            let _ = child.kill();
            let started = Instant::now();
            while started.elapsed() < Duration::from_secs(1) {
                if child.try_wait().ok().flatten().is_some() {
                    break;
                }
                thread::sleep(POLL);
            }
            return Err(error);
        }
        Ok(child)
    }

    /// Attach an owned child that is still waiting at its launch gate.
    /// The caller must not release the executable before this succeeds.
    ///
    /// # Errors
    /// Fails if controls changed, attachment fails, or membership is not observed.
    pub fn attach_gated_child(&self, child: &Child) -> Result<(), BackendError> {
        self.controls()?;
        write(&self.path, "cgroup.procs", &child.id().to_string())?;
        if !self.sample()?.member_pids.contains(&child.id()) {
            return Err(BackendError::Identity);
        }
        Ok(())
    }

    /// Read hierarchical counters from this owned leaf and its process members.
    ///
    /// # Errors
    /// Missing or malformed required counters fail instead of becoming zero.
    pub fn sample(&self) -> Result<LinuxUsage, BackendError> {
        Ok(LinuxUsage {
            cpu_usage_usec: field(&self.path, "cpu.stat", "usage_usec")?,
            cpu_throttled_periods: field(&self.path, "cpu.stat", "nr_throttled")?,
            cpu_throttled_usec: field(&self.path, "cpu.stat", "throttled_usec")?,
            memory_current_bytes: number(&self.path, "memory.current")?,
            memory_peak_bytes: number(&self.path, "memory.peak")?,
            memory_max_events: field(&self.path, "memory.events", "max")?,
            memory_oom_kills: field(&self.path, "memory.events", "oom_kill")?,
            pids_current: number(&self.path, "pids.current")?,
            pids_max_events: field(&self.path, "pids.events", "max")?,
            populated: field(&self.path, "cgroup.events", "populated")? != 0,
            member_pids: read(&self.path, "cgroup.procs")?
                .split_whitespace()
                .map(|value| value.parse().map_err(|_| BackendError::Io))
                .collect::<Result<_, _>>()?,
        })
    }

    /// Signal the owned cgroup tree before the caller reaps its Child handles.
    ///
    /// # Errors
    /// Fails if whole-tree termination is denied or the group no longer exists.
    pub fn terminate(&self) -> Result<(), BackendError> {
        if self.removed {
            return Err(BackendError::Identity);
        }
        write(&self.path, "cgroup.kill", "1")
    }

    /// Kill only this owned cgroup tree, require empty kernel membership, and
    /// remove the owned leaf. Call terminate and reap Child handles first: kernel
    /// task accounting may include zombies until they are reaped.
    ///
    /// # Errors
    /// Denial, surviving members, new subgroups, or expired deadline fail cleanup.
    pub fn cleanup(&mut self, timeout: Duration) -> Result<LinuxUsage, BackendError> {
        if self.removed {
            return Err(BackendError::Identity);
        }
        let started = Instant::now();
        write(&self.path, "cgroup.kill", "1")?;
        loop {
            let usage = self.sample()?;
            if !usage.populated && usage.pids_current == 0 && usage.member_pids.is_empty() {
                fs::remove_dir(&self.path).map_err(|error| io_error(&error))?;
                self.removed = true;
                return Ok(usage);
            }
            if started.elapsed() >= timeout {
                return Err(BackendError::Io);
            }
            thread::sleep(POLL);
        }
    }
}

impl Drop for LinuxAuthority {
    fn drop(&mut self) {
        if !self.removed {
            let _ = self.cleanup(Duration::from_secs(1));
        }
    }
}

fn validate_limits(limits: LinuxLimits) -> Result<(), BackendError> {
    if limits.cpu_quota_usec < 1_000
        || !(1_000..=1_000_000).contains(&limits.cpu_period_usec)
        || limits.memory_max_bytes < 1_048_576
        || limits.pids_max == 0
    {
        return Err(BackendError::Identity);
    }
    Ok(())
}

fn validate_root(root: &Path) -> Result<PathBuf, BackendError> {
    let root = fs::canonicalize(root).map_err(|error| io_error(&error))?;
    let text = fs::read_to_string("/proc/self/cgroup").map_err(|error| io_error(&error))?;
    let relative = text
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .ok_or(BackendError::Unsupported)?;
    let current = Path::new(MOUNT).join(relative.trim_start_matches('/'));
    if root == Path::new(MOUNT)
        || !root.starts_with(MOUNT)
        || (root != current && Some(root.as_path()) != current.parent())
    {
        return Err(BackendError::Identity);
    }
    Ok(root)
}

fn io_error(error: &std::io::Error) -> BackendError {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => BackendError::Privilege,
        std::io::ErrorKind::NotFound | std::io::ErrorKind::Unsupported => BackendError::Unsupported,
        _ => BackendError::Io,
    }
}

fn read(root: &Path, file: &str) -> Result<String, BackendError> {
    fs::read_to_string(root.join(file))
        .map(|s| s.trim().to_owned())
        .map_err(|error| io_error(&error))
}

fn write(root: &Path, file: &str, value: &str) -> Result<(), BackendError> {
    fs::write(root.join(file), value).map_err(|error| io_error(&error))
}

fn number(root: &Path, file: &str) -> Result<u64, BackendError> {
    read(root, file)?.parse().map_err(|_| BackendError::Io)
}

fn field(root: &Path, file: &str, key: &str) -> Result<u64, BackendError> {
    read(root, file)?
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(' ')?;
            (name == key).then(|| value.parse().ok()).flatten()
        })
        .ok_or(BackendError::Unsupported)
}
