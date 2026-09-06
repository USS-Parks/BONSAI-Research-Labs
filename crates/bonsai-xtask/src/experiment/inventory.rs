use super::identity::file_hash;
use bonsai_platform::clock::{ClockCalibrationPolicy, calibrate_clock, capture_system_probes};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn collect() -> Result<Value, String> {
    let cpu = fs::read_to_string("/proc/cpuinfo").map_err(|e| e.to_string())?;
    let field = |name: &str| {
        cpu.lines()
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;
                (key.trim() == name).then(|| value.trim().to_owned())
            })
            .unwrap_or_else(|| "unavailable".into())
    };
    let cores = cpu
        .lines()
        .filter(|line| {
            line.split_once(':')
                .is_some_and(|(key, _)| key.trim() == "processor")
        })
        .count();
    if cores == 0 {
        return Err("CPU_COUNT_UNAVAILABLE".into());
    }
    let memory = fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?;
    let total = memory
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .ok_or("MEMORY_TOTAL_UNAVAILABLE")?;
    let mut memory_fields = total.split_whitespace();
    let kib = memory_fields
        .next()
        .ok_or("MEMORY_TOTAL_INVALID")?
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    if memory_fields.next() != Some("kB") {
        return Err("MEMORY_TOTAL_UNIT_INVALID".into());
    }
    let bytes = kib.checked_mul(1024).ok_or("MEMORY_TOTAL_OVERFLOW")?;
    let pages = Command::new("/usr/bin/getconf")
        .arg("PAGESIZE")
        .output()
        .map_err(|e| e.to_string())?;
    if !pages.status.success() {
        return Err("PAGE_SIZE_UNAVAILABLE".into());
    }
    let page_size = String::from_utf8(pages.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let calibration = calibrate_clock(
        &capture_system_probes(256).map_err(|e| e.to_string())?,
        ClockCalibrationPolicy::default(),
    )
    .map_err(|e| e.to_string())?;
    let kernel = fs::read_to_string("/proc/sys/kernel/osrelease").map_err(|e| e.to_string())?;
    let version = fs::read_to_string("/proc/version").map_err(|e| e.to_string())?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let inventory_id = uuid()?;
    let machine_id = uuid()?;
    Ok(
        json!({"schema_version":"1.0","inventory_id":inventory_id,"machine_identity_id":machine_id,
            "os":{"family":"linux","version":kernel.trim(),"build":version.trim(),"kernel":kernel.trim(),"architecture":std::env::consts::ARCH},
            "cpu":{"vendor":field("vendor_id"),"model":field("model name"),"architecture":std::env::consts::ARCH,"logical_cores":cores},
            "accelerators":[],"memory":{"physical_total_bytes":bytes,"page_size_bytes":page_size},
            "clocks":[{"clock_id":"rust_instant","kind":"monotonic","resolution_ns":calibration.effective_resolution_ns,"monotonic":true}],
            "drivers":[],"runtimes":[{"component_id":"bonsai-runtime","version":env!("CARGO_PKG_VERSION")}],
            "compilers":[{"component_id":"rustc","version":env!("BONSAI_BUILD_COMPILER")}],
            "dependency_locks":[{"ecosystem":"cargo","lockfile_name":"Cargo.lock","sha256":file_hash(&root.join("Cargo.lock"))?},
                {"ecosystem":"uv","lockfile_name":"uv.lock","sha256":file_hash(&root.join("uv.lock"))?}],
            "privilege":{"process_level":"unknown","elevation_available":false},
            "collectors":[{"collector_id":"linux_proc_inventory","version":"1.0","status":"available","privilege_requirement":"none",
                "capabilities":["cpu_topology","guest_memory","monotonic_clock"]},
                {"collector_id":"energy_accelerators_drivers","version":"1.0","status":"unavailable","privilege_requirement":"none","capabilities":[]}],
            "thermal_power":{"thermal_state":"unavailable","power_source":"unknown"}
        }),
    )
}

fn uuid() -> Result<String, String> {
    fs::read_to_string("/proc/sys/kernel/random/uuid")
        .map(|s| s.trim().to_owned())
        .map_err(|e| e.to_string())
}
