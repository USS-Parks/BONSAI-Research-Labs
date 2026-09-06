use bonsai_contracts::accounting::OnlineAccounting;
use bonsai_contracts::bonsai::adapter::v1::PrimitiveAccounting;
use serde_json::json;

fn declaration() -> serde_json::Value {
    json!({"schema":"bonsai.online-accounting/v1","parameter_touches_per_update":3,
        "parameter_update_schema":"bonsai.parameter-update/v1"})
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
