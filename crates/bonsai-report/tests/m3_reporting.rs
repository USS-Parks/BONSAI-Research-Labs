use bonsai_report::compare::{ComparisonRow, CurvePoint, comparison_html, comparison_table};
use bonsai_report::equivalence::{PlatformRecord, semantic_equivalence};
use bonsai_report::overhead::{OverheadPair, accept_overhead};

#[test]
fn comparison_table_requires_uncertainty_and_refuses_hidden_failures() {
    let row = ComparisonRow {
        series_id: "dense".to_owned(),
        curve: vec![CurvePoint {
            step: 1,
            value: 4,
            lower: 3,
            upper: 5,
        }],
        worst_window: 2,
        resource_front: 10,
        suppressed_work: 0,
        uncertainty_label: "95% interval".to_owned(),
    };
    let table =
        comparison_table("lifelong", vec![row.clone()], false, false, false).expect("table");
    let html = comparison_html(&table);
    assert!(html.contains("Uncertainty-labeled comparison"));
    assert!(html.contains("dense"));
    assert!(html.contains("95% interval"));
    assert!(comparison_table("lifelong", vec![row], true, false, false).is_err());
}

#[test]
fn semantic_equivalence_separates_performance() {
    let records = [
        PlatformRecord {
            os_family: "windows".to_owned(),
            schema_ok: true,
            manifest_ok: true,
            track: "track_a".to_owned(),
            ordering_ok: true,
            metric_ok: true,
            bundle_ok: true,
            claim_ok: true,
            wall_time_ns: 100,
        },
        PlatformRecord {
            os_family: "macos".to_owned(),
            schema_ok: true,
            manifest_ok: true,
            track: "track_a".to_owned(),
            ordering_ok: true,
            metric_ok: true,
            bundle_ok: true,
            claim_ok: true,
            wall_time_ns: 140,
        },
        PlatformRecord {
            os_family: "linux".to_owned(),
            schema_ok: true,
            manifest_ok: true,
            track: "track_a".to_owned(),
            ordering_ok: true,
            metric_ok: true,
            bundle_ok: true,
            claim_ok: true,
            wall_time_ns: 90,
        },
    ];
    let matrix = semantic_equivalence(&records).expect("eq");
    assert!(matrix.semantic_equivalent);
    assert_eq!(matrix.performance_delta_ns, 50);
    assert!(semantic_equivalence(&records[..2]).is_err());
}

#[test]
fn overhead_acceptance_uses_raw_pairs_and_d11_ceilings() {
    let pairs = [
        OverheadPair {
            series: "throughput".to_owned(),
            none: 100,
            minimal: 101,
            full: 103,
        },
        OverheadPair {
            series: "p95_latency".to_owned(),
            none: 100,
            minimal: 104,
            full: 108,
        },
        OverheadPair {
            series: "cpu".to_owned(),
            none: 100,
            minimal: 102,
            full: 105,
        },
        OverheadPair {
            series: "memory".to_owned(),
            none: 100,
            minimal: 101,
            full: 102,
        },
        OverheadPair {
            series: "storage".to_owned(),
            none: 100,
            minimal: 100,
            full: 101,
        },
    ];
    let report = accept_overhead(&pairs, false).expect("overhead");
    assert!(report.within_d11);
    assert_eq!(report.throughput_ppm, 30_000);
    assert_eq!(report.latency_ppm, 80_000);
    assert!(accept_overhead(&pairs, true).is_err());
}
