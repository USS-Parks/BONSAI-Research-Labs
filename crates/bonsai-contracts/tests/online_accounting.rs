use bonsai_contracts::accounting::OnlineAccounting;
use bonsai_contracts::bonsai::adapter::v1::PrimitiveAccounting;
use serde_json::json;

fn declaration() -> serde_json::Value {
    json!({"schema":"bonsai.online-accounting/v1","parameter_touches_per_update":3,
        "parameter_update_schema":"bonsai.parameter-update/v1"})
}

fn declaration_v2() -> serde_json::Value {
    json!({
        "schema":"bonsai.online-accounting/v2",
        "feedback_signal_type":"bonsai.agent.causal-transition/v1",
        "parameter_update_schema":"bonsai.online-update/v2",
        "work_per_step":[
            {"work_class":"acting","amount":2},
            {"work_class":"learning","amount":1},
            {"work_class":"feature_generation","amount":4},
            {"work_class":"option_learning","amount":16}
        ],
        "maximum_parameter_touches_per_update":9,
        "retained_state_limit_bytes":1024,
        "serialized_state_limit_bytes":512
    })
}

fn measured_v2() -> PrimitiveAccounting {
    let completed = 2_u64;
    PrimitiveAccounting {
        environment_steps: completed,
        updates: completed,
        parameter_touches: 18,
        work_items: 46,
        replay_items_retained: 0,
        parameter_update: serde_json::to_vec(&json!({
            "schema":"bonsai.online-update/v2", "update":completed, "action":2, "reward":1,
            "work_by_class":{"acting":4,"learning":2,"feature_generation":8,"option_learning":32},
            "parameter_touches":18,
            "state_before":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "state_after":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "allocated_bytes":1024,
            "serialized_bytes":512,
            "details":{}
        }))
        .expect("json"),
    }
}
fn measured() -> PrimitiveAccounting {
    PrimitiveAccounting {
        environment_steps: 1,
        updates: 1,
        parameter_touches: 3,
        work_items: 4,
        replay_items_retained: 0,
        parameter_update: serde_json::to_vec(&json!({
            "schema":"bonsai.parameter-update/v1","update":1,"action":2,"reward":1,
            "before":[0,0,0],"after":[0.1,0.2,0.3]
        }))
        .expect("json"),
    }
}
#[test]
fn explicit_accounting_binds_update_shape_and_causal_feedback() {
    let contract =
        OnlineAccounting::from_declaration_or_legacy(Some(&declaration())).expect("contract");
    contract.validate(&measured(), 1, 2, 1).expect("valid");
    assert_eq!(
        contract.validate(&measured(), 1, 1, 1),
        Err("PARAMETER_UPDATE_LINK_MISMATCH")
    );
    assert_eq!(
        contract.validate(&measured(), 1, 2, 0),
        Err("PARAMETER_UPDATE_LINK_MISMATCH")
    );
    let mut changed = measured();
    changed.parameter_touches = 2;
    assert_eq!(
        contract.validate(&changed, 1, 2, 1),
        Err("ACCOUNTING_INCONSISTENT")
    );
    changed = measured();
    changed.parameter_update.clear();
    assert_eq!(
        contract.validate(&changed, 1, 2, 1),
        Err("PARAMETER_UPDATE_INVALID")
    );
    changed = measured();
    changed.parameter_update = vec![b' '; 32 * 1024 + 1];
    assert_eq!(
        contract.validate(&changed, 1, 2, 1),
        Err("PARAMETER_UPDATE_TOO_LARGE")
    );
    changed = measured();
    changed.parameter_update = serde_json::to_vec(&json!({
        "schema":"bonsai.parameter-update/v1","update":1,"action":2,"reward":1,
        "before":[0,0],"after":[0,0,0]}))
    .expect("json");
    assert_eq!(
        contract.validate(&changed, 1, 2, 1),
        Err("PARAMETER_UPDATE_SHAPE_MISMATCH")
    );
}
#[test]
fn missing_declaration_preserves_exact_historical_accounting_without_new_evidence() {
    let legacy = OnlineAccounting::from_declaration_or_legacy(None).expect("legacy");
    let mut old = measured();
    old.parameter_touches = 2;
    old.parameter_update.clear();
    legacy.validate(&old, 1, 2, 1).expect("historical");
    assert_eq!(
        legacy.validate(&old, 2, 2, 1),
        Err("ACCOUNTING_INCONSISTENT")
    );
}
#[test]
fn unknown_contracts_or_fields_never_fall_back_to_legacy() {
    for change in [
        json!({"schema":"unknown"}),
        json!({"schema":"bonsai.online-accounting/v1","parameter_touches_per_update":0,
            "parameter_update_schema":"bonsai.parameter-update/v1"}),
        json!({"schema":"bonsai.online-accounting/v1","parameter_touches_per_update":3,
            "parameter_update_schema":"bonsai.parameter-update/v1","extra":true}),
    ] {
        assert!(OnlineAccounting::from_declaration_or_legacy(Some(&change)).is_err());
    }
}

#[test]
fn v2_binds_transition_feedback_cumulative_work_and_state_caps() {
    let contract =
        OnlineAccounting::from_declaration_or_legacy(Some(&declaration_v2())).expect("v2 contract");
    assert!(contract.is_transition_feedback());
    assert_eq!(contract.maximum_parameter_touches_per_update(), Some(9));
    assert_eq!(contract.retained_state_limit_bytes(), Some(1024));
    assert_eq!(contract.serialized_state_limit_bytes(), Some(512));
    assert_eq!(
        serde_json::to_value(contract.work_per_step(99)).expect("tariffs"),
        json!([
            ["acting", 2],
            ["learning", 1],
            ["feature_generation", 4],
            ["option_learning", 16]
        ])
    );
    assert_eq!(
        contract.admission_payload(7),
        Some(json!({
            "schema":"bonsai.online-admission/v2", "step":7,
            "work_per_step":[
                {"work_class":"acting","amount":2}, {"work_class":"learning","amount":1},
                {"work_class":"feature_generation","amount":4}, {"work_class":"option_learning","amount":16}
            ],
            "retained_state_limit_bytes":1024, "serialized_state_limit_bytes":512
        }))
    );
    contract
        .validate(&measured_v2(), 2, 2, 1)
        .expect("valid v2");

    let mut changed = measured_v2();
    changed.work_items = 45;
    assert_eq!(
        contract.validate(&changed, 2, 2, 1),
        Err("ACCOUNTING_INCONSISTENT")
    );
    changed = measured_v2();
    let mut update: serde_json::Value =
        serde_json::from_slice(&changed.parameter_update).expect("json");
    update["work_by_class"]["acting"] = json!(3);
    changed.parameter_update = serde_json::to_vec(&update).expect("json");
    assert_eq!(
        contract.validate(&changed, 2, 2, 1),
        Err("PARAMETER_UPDATE_SHAPE_MISMATCH")
    );
    changed = measured_v2();
    let mut update: serde_json::Value =
        serde_json::from_slice(&changed.parameter_update).expect("json");
    update["allocated_bytes"] = json!(1025);
    changed.parameter_update = serde_json::to_vec(&update).expect("json");
    assert_eq!(
        contract.validate(&changed, 2, 2, 1),
        Err("PARAMETER_UPDATE_LINK_MISMATCH")
    );
}

#[test]
fn v2_rejects_malformed_versions_duplicate_classes_and_extra_update_fields() {
    for change in [
        json!({"schema":"bonsai.online-accounting/v2"}),
        json!({
            "schema":"bonsai.online-accounting/v2", "feedback_signal_type":"bonsai.agent.reward/v1",
            "parameter_update_schema":"bonsai.online-update/v2",
            "work_per_step":[{"work_class":"acting","amount":1},{"work_class":"learning","amount":1}],
            "maximum_parameter_touches_per_update":1, "retained_state_limit_bytes":1, "serialized_state_limit_bytes":1
        }),
        json!({
            "schema":"bonsai.online-accounting/v2", "feedback_signal_type":"bonsai.agent.causal-transition/v1",
            "parameter_update_schema":"bonsai.online-update/v2",
            "work_per_step":[{"work_class":"acting","amount":1},{"work_class":"acting","amount":1},{"work_class":"learning","amount":1}],
            "maximum_parameter_touches_per_update":1, "retained_state_limit_bytes":1, "serialized_state_limit_bytes":1
        }),
    ] {
        assert!(OnlineAccounting::from_declaration_or_legacy(Some(&change)).is_err());
    }
    let contract =
        OnlineAccounting::from_declaration_or_legacy(Some(&declaration_v2())).expect("v2 contract");
    let mut changed = measured_v2();
    let mut update: serde_json::Value =
        serde_json::from_slice(&changed.parameter_update).expect("json");
    update["extra"] = json!(true);
    changed.parameter_update = serde_json::to_vec(&update).expect("json");
    assert_eq!(
        contract.validate(&changed, 2, 2, 1),
        Err("PARAMETER_UPDATE_INVALID")
    );
}

#[test]
fn v2_accepts_actual_touches_below_maximum_and_rejects_overrun_or_mismatch() {
    let contract =
        OnlineAccounting::from_declaration_or_legacy(Some(&declaration_v2())).expect("v2");
    for touches in [0, 1, 9, 17, 18] {
        let mut measured = measured_v2();
        measured.parameter_touches = touches;
        let mut update: serde_json::Value =
            serde_json::from_slice(&measured.parameter_update).expect("json");
        update["parameter_touches"] = json!(touches);
        measured.parameter_update = serde_json::to_vec(&update).expect("json");
        contract
            .validate(&measured, 2, 2, 1)
            .expect("within declared maximum");
    }
    let mut overrun = measured_v2();
    overrun.parameter_touches = 19;
    assert_eq!(
        contract.validate(&overrun, 2, 2, 1),
        Err("ACCOUNTING_INCONSISTENT")
    );
    let mut mismatch = measured_v2();
    mismatch.parameter_touches = 17;
    assert_eq!(
        contract.validate(&mismatch, 2, 2, 1),
        Err("PARAMETER_UPDATE_LINK_MISMATCH")
    );
}
