use bonsai_metrics::feature::{
    FeatureClass, FeatureEvent, FeatureEventKind, FeatureMetricError, derive_feature_metrics,
};
use serde_json::{Value, json};

#[allow(clippy::too_many_arguments)]
fn event(
    step: u64,
    feature_id: &str,
    kind: FeatureEventKind,
    representation_id: &str,
    bytes: u64,
    work_cost: u64,
    consumer_delta: i64,
    marginal_utility: i64,
) -> FeatureEvent {
    FeatureEvent {
        step,
        feature_id: feature_id.to_owned(),
        kind,
        representation_id: representation_id.to_owned(),
        bytes,
        work_cost,
        consumer_delta,
        marginal_utility,
    }
}

fn summarize(table: &bonsai_metrics::feature::FeatureMetricTable) -> Value {
    json!({
        "schema": table.schema,
        "metrics": table.metrics.iter().map(|metric| json!({
            "id": metric.key.id,
            "numerator": metric.value.as_ref().map(|value| value.numerator),
            "denominator": metric.value.as_ref().map(|value| value.denominator),
            "detail_code": metric.detail_code,
        })).collect::<Vec<_>>(),
        "classifications": table.classifications.iter().map(|row| json!({
            "feature_id": row.feature_id,
            "class": row.class,
            "age": row.age,
            "activations": row.activations,
            "consumers": row.consumers,
            "marginal_utility": row.marginal_utility,
        })).collect::<Vec<_>>(),
    })
}

#[test]
#[allow(clippy::too_many_lines)]
fn controlled_lineage_fixture_identifies_useful_redundant_and_dormant() {
    let events = [
        event(
            0,
            "useful",
            FeatureEventKind::Birth,
            "rep-useful",
            10,
            4,
            0,
            0,
        ),
        event(
            1,
            "redundant",
            FeatureEventKind::Birth,
            "rep-useful",
            10,
            4,
            0,
            0,
        ),
        event(
            2,
            "dormant",
            FeatureEventKind::Birth,
            "rep-dormant",
            6,
            2,
            0,
            0,
        ),
        event(
            3,
            "obsolete",
            FeatureEventKind::Birth,
            "rep-obsolete",
            5,
            3,
            0,
            0,
        ),
        event(
            4,
            "useful",
            FeatureEventKind::Activate,
            "rep-useful",
            0,
            0,
            0,
            0,
        ),
        event(
            5,
            "useful",
            FeatureEventKind::ConsumerUpdate,
            "rep-useful",
            0,
            0,
            2,
            8,
        ),
        event(
            6,
            "obsolete",
            FeatureEventKind::Activate,
            "rep-obsolete",
            0,
            0,
            0,
            0,
        ),
        event(
            7,
            "obsolete",
            FeatureEventKind::ConsumerUpdate,
            "rep-obsolete",
            0,
            0,
            1,
            1,
        ),
        event(
            8,
            "obsolete",
            FeatureEventKind::Retire,
            "rep-obsolete",
            0,
            0,
            0,
            0,
        ),
        event(
            9,
            "redundant",
            FeatureEventKind::UtilityUpdate,
            "rep-useful",
            0,
            0,
            0,
            -2,
        ),
    ];
    let table = derive_feature_metrics(&events).expect("table");
    let actual = json!({
        "schema": "bonsai.feature-metric-outcomes/v1",
        "table": summarize(&table),
    });
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/feature-metrics/v1/expected-outcomes.json"
    ))
    .expect("expected");
    assert_eq!(actual, expected);
    let class = |id: &str| {
        table
            .classifications
            .iter()
            .find(|row| row.feature_id == id)
            .expect("row")
            .class
    };
    assert_eq!(class("useful"), FeatureClass::Useful);
    assert_eq!(class("redundant"), FeatureClass::Redundant);
    assert_eq!(class("dormant"), FeatureClass::Dormant);
    assert_eq!(class("obsolete"), FeatureClass::ObsoleteProtected);
}

#[test]
fn malformed_feature_traces_fail_closed() {
    assert!(matches!(
        derive_feature_metrics(&[]),
        Err(FeatureMetricError::Trace)
    ));
    let activate_before_birth = [event(0, "x", FeatureEventKind::Activate, "r", 0, 0, 0, 0)];
    assert!(matches!(
        derive_feature_metrics(&activate_before_birth),
        Err(FeatureMetricError::Lifecycle)
    ));
}
