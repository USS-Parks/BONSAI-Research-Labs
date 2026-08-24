use bonsai_metrics::continual::{
    ContinualMetricError, ContinualPhase, ContinualPoint, derive_continual_metrics,
};
use serde_json::{Value, json};

fn point(
    step: u64,
    task_id: &str,
    phase: ContinualPhase,
    performance: i64,
    agent_age: u64,
) -> ContinualPoint {
    ContinualPoint {
        step,
        task_id: task_id.to_owned(),
        phase,
        performance,
        agent_age,
    }
}

fn metric_map(table: &bonsai_metrics::continual::ContinualMetricTable) -> Value {
    let metrics = table
        .metrics
        .iter()
        .map(|metric| {
            json!({
                "id": metric.key.id,
                "numerator": metric.value.as_ref().map(|value| value.numerator),
                "denominator": metric.value.as_ref().map(|value| value.denominator),
                "detail_code": metric.detail_code,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": table.schema,
        "metrics": metrics,
        "age_curve": table.age_curve,
    })
}

#[test]
fn synthetic_forgetting_and_plasticity_fixtures_are_distinct() {
    let forgetting_heavy = [
        point(0, "A", ContinualPhase::Train, 10, 0),
        point(1, "B", ContinualPhase::Adapt, 10, 1),
        point(2, "A", ContinualPhase::RetainProbe, 1, 2),
        point(3, "A", ContinualPhase::Relearn, 9, 3),
    ];
    let plasticity_loss_heavy = [
        point(0, "A", ContinualPhase::Train, 10, 0),
        point(1, "B", ContinualPhase::Adapt, 1, 1),
        point(2, "A", ContinualPhase::RetainProbe, 10, 2),
        point(3, "A", ContinualPhase::Relearn, 10, 3),
    ];
    let forgetting = derive_continual_metrics(&forgetting_heavy).expect("forgetting");
    let plasticity = derive_continual_metrics(&plasticity_loss_heavy).expect("plasticity");
    let actual = json!({
        "schema": "bonsai.continual-metric-outcomes/v1",
        "forgetting_heavy": metric_map(&forgetting),
        "plasticity_loss_heavy": metric_map(&plasticity),
    });
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/continual-metrics/v1/expected-outcomes.json"
    ))
    .expect("expected");
    assert_eq!(actual, expected);
    let f_forget = forgetting
        .metrics
        .iter()
        .find(|metric| metric.key.id == "forgetting")
        .expect("f");
    let p_forget = plasticity
        .metrics
        .iter()
        .find(|metric| metric.key.id == "forgetting")
        .expect("p");
    let f_plast = forgetting
        .metrics
        .iter()
        .find(|metric| metric.key.id == "plasticity_loss")
        .expect("fp");
    let p_plast = plasticity
        .metrics
        .iter()
        .find(|metric| metric.key.id == "plasticity_loss")
        .expect("pp");
    assert!(
        f_forget.value.as_ref().expect("v").numerator
            > p_forget.value.as_ref().expect("v").numerator
    );
    assert!(
        p_plast.value.as_ref().expect("v").numerator > f_plast.value.as_ref().expect("v").numerator
    );
}

#[test]
fn malformed_traces_fail_closed() {
    assert!(matches!(
        derive_continual_metrics(&[]),
        Err(ContinualMetricError::Trace)
    ));
    let bad = [ContinualPoint {
        step: 1,
        task_id: "A".to_owned(),
        phase: ContinualPhase::Train,
        performance: 1,
        agent_age: 0,
    }];
    assert!(matches!(
        derive_continual_metrics(&bad),
        Err(ContinualMetricError::Trace)
    ));
}
