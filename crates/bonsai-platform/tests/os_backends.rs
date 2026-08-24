use bonsai_platform::capability::{Support, record_enforcement};
use bonsai_platform::energy::{
    EnergyEvidence, EnergyTier, adjudicate_energy, qualify_present_backends,
};
use bonsai_platform::equivalence::{Comparability, resource_comparability_matrix};
use bonsai_platform::linux::{
    CgroupSnapshot, CgroupWorkload, collect_current_cgroup, detect_linux_backend,
    linux_enforcement, reconcile_cgroup, try_apply_probe_limit,
};
use bonsai_platform::macos::{
    MacosObservation, MacosWorkload, detect_macos_backend, macos_enforcement, reconcile_macos,
};
use bonsai_platform::nvidia::{NvidiaPresence, detect_nvidia, nvidia_capability_matrix};
use bonsai_platform::windows::{
    ControlledWorkload, JobObjectAccounting, detect_windows_backend, reconcile_job_object,
    windows_enforcement,
};

#[test]
fn windows_capability_and_job_object_reconciliation() {
    let matrix = detect_windows_backend();
    assert_eq!(matrix.backend_id, "windows-job-object");
    assert!(!matrix.physical_acceptance);
    let job_cpu = matrix.control("job.cpu_time").expect("job cpu");
    assert_eq!(job_cpu.support, Support::Unsupported);
    assert!(
        job_cpu.detail_code == "NOT_WINDOWS_HOST"
            || job_cpu.detail_code == "NO_SAFE_JOB_OBJECT_BINDING"
    );

    let workload = ControlledWorkload {
        workload_id: "escape-child".to_owned(),
        process_ids: vec![10, 11],
        cpu_time_ns: 1_000,
        committed_memory_bytes: 64,
        io_read_bytes: 8,
        io_write_bytes: 4,
    };
    let job = JobObjectAccounting {
        job_id: "job-1".to_owned(),
        process_ids: vec![10],
        cpu_time_ns: 1_000,
        committed_memory_bytes: 64,
        io_read_bytes: 8,
        io_write_bytes: 4,
        process_count: 1,
        kill_on_close: true,
    };
    let escape = reconcile_job_object(&workload, &job).expect("escape");
    assert!(escape.child_escape);
    assert!(!escape.claim_ready);

    let covered = JobObjectAccounting {
        process_ids: vec![10, 11],
        process_count: 2,
        ..job
    };
    let ok = reconcile_job_object(&workload, &covered).expect("covered");
    assert!(!ok.child_escape);
    assert!(ok.claim_ready);

    let crossed =
        windows_enforcement("job.memory", true, Some(100), Some(140), true).expect("cross");
    assert_eq!(crossed.overshoot, Some(40));
    assert!(crossed.terminated);
    let unsupported = windows_enforcement("job.io", false, None, None, false).expect("decl");
    assert_eq!(unsupported.support, Support::Unsupported);
}

#[test]
fn macos_capability_and_monitor_terminate_overshoot() {
    let matrix = detect_macos_backend();
    assert_eq!(matrix.backend_id, "macos-process");
    let thermal = matrix.control("thermal.pressure").expect("thermal");
    assert_ne!(thermal.support, Support::Supported);
    let cpu_limit = matrix
        .limits
        .iter()
        .find(|limit| limit.control_id == "cpu_time")
        .expect("cpu limit");
    assert!(matches!(
        cpu_limit.support,
        Support::Unsupported | Support::MonitorTerminateOnly
    ));

    let report = reconcile_macos(
        &MacosWorkload {
            workload_id: "apple-cpu".to_owned(),
            cpu_time_ns: 2_000,
            resident_memory_bytes: 32,
            process_count: 2,
        },
        &MacosObservation {
            cpu_time_ns: 2_000,
            resident_memory_bytes: 32,
            process_count: 2,
            thermal_available: false,
        },
    )
    .expect("macos");
    assert!(report.claim_ready);
    assert_eq!(report.thermal_status, Support::NoPermission);

    let overshoot = macos_enforcement("memory", 100, 125, true).expect("overshoot");
    assert_eq!(overshoot.support, Support::MonitorTerminateOnly);
    assert_eq!(overshoot.overshoot, Some(25));
    assert!(overshoot.terminated);
}

#[test]
fn linux_cgroup_live_read_and_fail_closed_enforcement() {
    let matrix = detect_linux_backend();
    assert_eq!(matrix.backend_id, "linux-cgroup-v2");
    if cfg!(target_os = "linux") {
        let snapshot = collect_current_cgroup().expect("live cgroup");
        assert!(!snapshot.path.is_empty());
        assert!(snapshot.cpu_usage_usec.is_some());
        assert!(snapshot.memory_current_bytes.is_some());
        assert!(
            matrix
                .control("cgroup.cpu.stat")
                .is_some_and(|control| { control.support == Support::Supported })
        );
        let enforce = linux_enforcement("cgroup.memory.max", 1, None, try_apply_probe_limit)
            .expect("enforce");
        assert_eq!(enforce.support, Support::NoPermission);
        assert!(!enforce.descendant_escape);
        assert!(enforce.cleaned_up);
    }

    let reconciled = reconcile_cgroup(
        &CgroupWorkload {
            workload_id: "nested".to_owned(),
            cpu_usage_usec: 50,
            memory_current_bytes: 128,
            pids_current: 3,
            nested_child_pids: 2,
        },
        &CgroupSnapshot {
            path: "/sys/fs/cgroup/fixture".to_owned(),
            cpu_usage_usec: Some(50),
            memory_current_bytes: Some(128),
            io_read_bytes: None,
            io_write_bytes: None,
            pids_current: Some(3),
            controllers: vec!["cpu".to_owned(), "memory".to_owned(), "pids".to_owned()],
        },
    )
    .expect("reconcile");
    assert!(reconciled.nested_covered);
    assert!(reconciled.claim_ready);
}

#[test]
fn nvidia_absence_does_not_break_cpu_only() {
    let detection = detect_nvidia();
    assert!(!detection.board_energy_process_exclusive);
    assert!(matches!(
        detection.presence,
        NvidiaPresence::NotSupported | NvidiaPresence::NoPermission | NvidiaPresence::Supported
    ));
    let matrix = nvidia_capability_matrix();
    assert_eq!(matrix.backend_id, "nvidia-nvml");
    assert!(!matrix.physical_acceptance || detection.presence == NvidiaPresence::Supported);
}

#[test]
fn energy_tiers_cannot_overclaim_or_invent_zero() {
    let rows = adjudicate_energy(&[
        EnergyEvidence {
            case_id: "missing".to_owned(),
            collector_id: "nvml".to_owned(),
            counter_wrap: false,
            missing_sample: true,
            clock_skew: false,
            privilege_failure: false,
            shared_device: false,
            calibrated: true,
            process_exclusive: true,
            laboratory_probe: false,
            estimated_zero: false,
        },
        EnergyEvidence {
            case_id: "shared".to_owned(),
            collector_id: "nvml".to_owned(),
            counter_wrap: false,
            missing_sample: false,
            clock_skew: false,
            privilege_failure: false,
            shared_device: true,
            calibrated: true,
            process_exclusive: false,
            laboratory_probe: false,
            estimated_zero: false,
        },
        EnergyEvidence {
            case_id: "wrap".to_owned(),
            collector_id: "powercap".to_owned(),
            counter_wrap: true,
            missing_sample: false,
            clock_skew: false,
            privilege_failure: false,
            shared_device: false,
            calibrated: true,
            process_exclusive: true,
            laboratory_probe: false,
            estimated_zero: false,
        },
    ])
    .expect("tiers");
    assert!(rows.iter().all(|row| row.tier == EnergyTier::E0));
    assert!(
        adjudicate_energy(&[EnergyEvidence {
            case_id: "zero".to_owned(),
            collector_id: "fake".to_owned(),
            counter_wrap: false,
            missing_sample: true,
            clock_skew: false,
            privilege_failure: false,
            shared_device: false,
            calibrated: false,
            process_exclusive: false,
            laboratory_probe: false,
            estimated_zero: true,
        }])
        .is_err()
    );
    assert!(
        qualify_present_backends()
            .iter()
            .all(|record| record.tier <= EnergyTier::E1)
    );
}

#[test]
fn comparability_matrix_has_no_unsupported_numeric_claim() {
    let matrix = resource_comparability_matrix();
    assert!(!matrix.unsupported_cross_platform_claim);
    assert_eq!(matrix.energy_floor, EnergyTier::E0);
    let cpu = matrix
        .cells
        .iter()
        .find(|cell| cell.counter_id == "process.cpu_time")
        .expect("cpu");
    assert_eq!(cpu.comparability, Comparability::Semantic);
    let energy = matrix
        .cells
        .iter()
        .find(|cell| cell.counter_id == "energy")
        .expect("energy");
    assert_eq!(energy.comparability, Comparability::Unavailable);
    assert!(
        !record_enforcement(
            "linux-cgroup-v2",
            "cgroup.pids.max",
            Support::NoPermission,
            None,
            None,
            false,
            true,
            false,
            "CGROUP_CONTROLLER_NOT_DELEGATED",
        )
        .expect("record")
        .descendant_escape
    );
}
