use bonsai_metrics::repro::reproduce_metrics;
use bonsai_metrics::{
    MetricDirection, MetricFormula, MetricKey, MetricRegistry, MetricSpec, MetricWindow,
    RationalValue,
};
use std::collections::BTreeMap;

fn key(id: &str) -> MetricKey {
    MetricKey {
        id: id.to_owned(),
        version: "1.0".to_owned(),
    }
}

fn spec(id: &str, formula: MetricFormula, inputs: Vec<MetricKey>) -> MetricSpec {
    MetricSpec {
        key: key(id),
        formula,
        unit: "count".to_owned(),
        window: MetricWindow::Lifetime,
        direction: MetricDirection::Neutral,
        inputs,
        availability_rule: "all_inputs_required".to_owned(),
        claim_uses: vec!["C0".to_owned()],
    }
}

#[test]
fn deterministic_derivation_stays_inside_streaming_envelope() {
    let registry = MetricRegistry::new(vec![
        spec("reward_total", MetricFormula::Input, vec![]),
        spec("steps", MetricFormula::Input, vec![]),
        spec(
            "reward_rate",
            MetricFormula::Ratio,
            vec![key("reward_total"), key("steps")],
        ),
    ])
    .expect("registry");
    let inputs = BTreeMap::from([
        (
            key("reward_total"),
            Some(RationalValue {
                numerator: 12,
                denominator: 1,
            }),
        ),
        (
            key("steps"),
            Some(RationalValue {
                numerator: 3,
                denominator: 1,
            }),
        ),
    ]);
    let expected = registry.compute(&inputs).expect("expected");
    let report = reproduce_metrics(&registry, &inputs, &expected, 8).expect("repro");
    assert!(report.identical);
    assert!(report.within_envelope);
    assert!(!report.agent_access);
    assert!(reproduce_metrics(&registry, &inputs, &expected, 1).is_err());
}
