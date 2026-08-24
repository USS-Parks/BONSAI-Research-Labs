//! D-20 utility estimator hierarchy with tier labels and C3 eligibility.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Estimator tier. Proxy utility cannot establish C3.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityTier {
    ExactLeaveOneOut,
    MatchedAblation,
    ConsumerCredit,
    InfluenceProxy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityAvailability {
    Available,
    Indeterminate,
}

/// One artifact with exact, matched, consumer, and proxy evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityRecord {
    pub artifact_id: String,
    pub present_return: i64,
    pub omitted_return: Option<i64>,
    pub matched_ablation_return: Option<i64>,
    pub consumer_credit: Option<i64>,
    pub influence_proxy: Option<i64>,
    pub proxy_confidence: Option<i64>,
    pub cost: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityRow {
    pub artifact_id: String,
    pub exact_utility: Option<i64>,
    pub matched_utility: Option<i64>,
    pub consumer_credit: Option<i64>,
    pub proxy_utility: Option<i64>,
    pub declared_utility: Option<i64>,
    pub utility_per_cost: Option<RationalValue>,
    pub tier: UtilityTier,
    pub availability: UtilityAvailability,
    pub detail_code: Option<String>,
    pub c3_eligible: bool,
    pub cost: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityTable {
    pub schema: String,
    pub confidence_threshold: i64,
    pub rows: Vec<UtilityRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UtilityError {
    Identity,
    Trace,
}

impl fmt::Display for UtilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "UTILITY_IDENTITY_INVALID",
            Self::Trace => "UTILITY_TRACE_INVALID",
        })
    }
}

impl Error for UtilityError {}

/// Derive tiered utility and refuse proxy-only C3 eligibility.
///
/// Exact leave-one-out and matched ablation may establish C3 when available.
/// Consumer credit is a lower labeled tier. Proxy sign errors or confidence
/// failures force indeterminate utility. Proxy-only rows stay ineligible for C3.
///
/// # Errors
///
/// Rejects empty identities or a non-positive confidence threshold.
pub fn derive_utility(
    records: &[UtilityRecord],
    confidence_threshold: i64,
) -> Result<UtilityTable, UtilityError> {
    if records.is_empty() || confidence_threshold <= 0 {
        return Err(UtilityError::Trace);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for record in records {
        if record.artifact_id.is_empty() || !ids.insert(record.artifact_id.as_str()) {
            return Err(UtilityError::Identity);
        }
        rows.push(classify(record, confidence_threshold));
    }
    rows.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    Ok(UtilityTable {
        schema: "bonsai.utility-table/v1".to_owned(),
        confidence_threshold,
        rows,
    })
}

fn classify(record: &UtilityRecord, threshold: i64) -> UtilityRow {
    let exact = record
        .omitted_return
        .map(|omitted| record.present_return.saturating_sub(omitted));
    let matched = record
        .matched_ablation_return
        .map(|ablated| record.present_return.saturating_sub(ablated));
    let (tier, declared, availability, detail, c3_eligible) = decide(
        exact,
        matched,
        record.consumer_credit,
        record.influence_proxy,
        record.proxy_confidence,
        threshold,
    );
    UtilityRow {
        artifact_id: record.artifact_id.clone(),
        exact_utility: exact,
        matched_utility: matched,
        consumer_credit: record.consumer_credit,
        proxy_utility: record.influence_proxy,
        declared_utility: declared,
        utility_per_cost: declared.filter(|_| record.cost > 0).map(|utility| {
            crate::normalize(RationalValue {
                numerator: utility,
                denominator: record.cost,
            })
        }),
        tier,
        availability,
        detail_code: detail,
        c3_eligible,
        cost: record.cost,
    }
}

fn decide(
    exact: Option<i64>,
    matched: Option<i64>,
    consumer: Option<i64>,
    proxy: Option<i64>,
    confidence: Option<i64>,
    threshold: i64,
) -> (
    UtilityTier,
    Option<i64>,
    UtilityAvailability,
    Option<String>,
    bool,
) {
    if let Some(exact) = exact {
        if let Some(proxy) = proxy {
            if sign(proxy) != sign(exact) {
                return (
                    UtilityTier::InfluenceProxy,
                    None,
                    UtilityAvailability::Indeterminate,
                    Some("UTILITY_PROXY_SIGN_ERROR".to_owned()),
                    false,
                );
            }
            if confidence.unwrap_or(0) < threshold {
                return (
                    UtilityTier::InfluenceProxy,
                    None,
                    UtilityAvailability::Indeterminate,
                    Some("UTILITY_PROXY_CONFIDENCE_FAILURE".to_owned()),
                    false,
                );
            }
        }
        return (
            UtilityTier::ExactLeaveOneOut,
            Some(exact),
            UtilityAvailability::Available,
            None,
            true,
        );
    }
    if let Some(matched) = matched {
        return (
            UtilityTier::MatchedAblation,
            Some(matched),
            UtilityAvailability::Available,
            None,
            true,
        );
    }
    if let Some(consumer) = consumer {
        return (
            UtilityTier::ConsumerCredit,
            Some(consumer),
            UtilityAvailability::Available,
            Some("UTILITY_CONSUMER_CREDIT".to_owned()),
            false,
        );
    }
    if let Some(proxy) = proxy {
        if confidence.unwrap_or(0) < threshold {
            return (
                UtilityTier::InfluenceProxy,
                None,
                UtilityAvailability::Indeterminate,
                Some("UTILITY_PROXY_CONFIDENCE_FAILURE".to_owned()),
                false,
            );
        }
        return (
            UtilityTier::InfluenceProxy,
            Some(proxy),
            UtilityAvailability::Available,
            Some("UTILITY_PROXY_ONLY".to_owned()),
            false,
        );
    }
    (
        UtilityTier::InfluenceProxy,
        None,
        UtilityAvailability::Indeterminate,
        Some("UTILITY_EVIDENCE_UNAVAILABLE".to_owned()),
        false,
    )
}

fn sign(value: i64) -> i64 {
    value.signum()
}

#[cfg(test)]
mod tests {
    use super::{UtilityAvailability, UtilityRecord, UtilityTier, derive_utility};

    fn record(
        id: &str,
        omitted: Option<i64>,
        matched: Option<i64>,
        consumer: Option<i64>,
        proxy: Option<i64>,
        confidence: Option<i64>,
    ) -> UtilityRecord {
        UtilityRecord {
            artifact_id: id.to_owned(),
            present_return: 10,
            omitted_return: omitted,
            matched_ablation_return: matched,
            consumer_credit: consumer,
            influence_proxy: proxy,
            proxy_confidence: confidence,
            cost: 2,
        }
    }

    fn diagnostic() -> Vec<UtilityRecord> {
        vec![
            record("exact_positive", Some(5), None, None, Some(4), Some(90)),
            record("proxy_sign_error", Some(5), None, None, Some(-3), Some(90)),
            record(
                "proxy_low_confidence",
                Some(5),
                None,
                None,
                Some(5),
                Some(10),
            ),
            record("proxy_only", None, None, None, Some(7), Some(90)),
        ]
    }

    #[test]
    fn proxy_only_cannot_establish_c3() {
        let table = derive_utility(&diagnostic(), 80).expect("utility");
        let row = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.artifact_id == id)
                .expect("row")
        };
        assert_eq!(row("exact_positive").tier, UtilityTier::ExactLeaveOneOut);
        assert!(row("exact_positive").c3_eligible);
        assert!(!row("proxy_only").c3_eligible);
        assert_eq!(
            row("proxy_only").detail_code.as_deref(),
            Some("UTILITY_PROXY_ONLY")
        );
    }

    #[test]
    fn sign_and_confidence_failures_are_indeterminate() {
        let table = derive_utility(&diagnostic(), 80).expect("utility");
        let row = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.artifact_id == id)
                .expect("row")
        };
        assert_eq!(
            row("proxy_sign_error").availability,
            UtilityAvailability::Indeterminate
        );
        assert_eq!(
            row("proxy_sign_error").detail_code.as_deref(),
            Some("UTILITY_PROXY_SIGN_ERROR")
        );
        assert_eq!(
            row("proxy_low_confidence").detail_code.as_deref(),
            Some("UTILITY_PROXY_CONFIDENCE_FAILURE")
        );
        assert!(!row("proxy_sign_error").c3_eligible);
        assert!(!row("proxy_low_confidence").c3_eligible);
    }

    #[test]
    fn committed_utility_tiers_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/utility-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_utility(&diagnostic(), 80).expect("utility");
        let rows = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.artifact_id.clone(),
                    serde_json::json!({
                        "tier": row.tier,
                        "availability": row.availability,
                        "c3_eligible": row.c3_eligible,
                        "detail_code": row.detail_code,
                    }),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.utility-metric-outcomes/v1","rows":rows}),
            expected
        );
    }
}
