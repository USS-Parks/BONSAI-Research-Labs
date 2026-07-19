//! Agent-neutral C0-C5 claim ladder with ternary evidence verdicts.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ClaimLevel {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
}

const CLAIM_LEVELS: [ClaimLevel; 6] = [
    ClaimLevel::C0,
    ClaimLevel::C1,
    ClaimLevel::C2,
    ClaimLevel::C3,
    ClaimLevel::C4,
    ClaimLevel::C5,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    LocalStructural,
    HostedSemantic,
    PhysicalHost,
    Confirmatory,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Passed,
    Failed,
    Missing,
    Contradictory,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimVerdict {
    Pass,
    Fail,
    Indeterminate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Criterion {
    pub criterion_id: String,
    pub evidence_tier: EvidenceTier,
    pub fact_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRule {
    pub level: ClaimLevel,
    pub prerequisites: Vec<ClaimLevel>,
    pub criteria: Vec<Criterion>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleSet {
    pub rule_version: String,
    pub rules: Vec<ClaimRule>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReasonNode {
    pub node_id: String,
    pub verdict: ClaimVerdict,
    pub reason_code: String,
    pub children: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimResult {
    pub level: ClaimLevel,
    pub rule_version: String,
    pub verdict: ClaimVerdict,
    pub root_reason_id: String,
    pub reason_graph: Vec<ReasonNode>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimEvaluation {
    pub schema: String,
    pub rule_version: String,
    pub results: Vec<ClaimResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimError {
    Version,
    Coverage,
    Identity,
    Prerequisite,
    Duplicate,
}

impl fmt::Display for ClaimError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Version => "CLAIM_RULE_VERSION_INVALID",
            Self::Coverage => "CLAIM_RULE_COVERAGE_INVALID",
            Self::Identity => "CLAIM_RULE_IDENTITY_INVALID",
            Self::Prerequisite => "CLAIM_RULE_PREREQUISITE_INVALID",
            Self::Duplicate => "CLAIM_RULE_DUPLICATE",
        })
    }
}

impl Error for ClaimError {}

impl RuleSet {
    /// Validate and evaluate every claim level in dependency order.
    ///
    /// # Errors
    ///
    /// Rejects malformed versions, incomplete C0-C5 coverage, duplicate facts,
    /// and non-lower-level prerequisites.
    pub fn evaluate(
        &self,
        facts: &BTreeMap<String, EvidenceState>,
    ) -> Result<ClaimEvaluation, ClaimError> {
        self.validate()?;
        let rules = self
            .rules
            .iter()
            .map(|rule| (rule.level, rule))
            .collect::<BTreeMap<_, _>>();
        let mut results = Vec::new();
        let mut prior = BTreeMap::new();
        for level in CLAIM_LEVELS {
            let rule = rules[&level];
            let result = evaluate_rule(rule, &self.rule_version, facts, &prior);
            prior.insert(level, result.verdict);
            results.push(result);
        }
        Ok(ClaimEvaluation {
            schema: "bonsai.claim-evaluation/v1".to_owned(),
            rule_version: self.rule_version.clone(),
            results,
        })
    }

    fn validate(&self) -> Result<(), ClaimError> {
        if self.rule_version.is_empty()
            || !self
                .rule_version
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'.')
        {
            return Err(ClaimError::Version);
        }
        let levels = self
            .rules
            .iter()
            .map(|rule| rule.level)
            .collect::<BTreeSet<_>>();
        if self.rules.len() != CLAIM_LEVELS.len() || levels != BTreeSet::from(CLAIM_LEVELS) {
            return Err(ClaimError::Coverage);
        }
        let mut criterion_ids = BTreeSet::new();
        let mut fact_ids = BTreeSet::new();
        for rule in &self.rules {
            if rule.criteria.is_empty()
                || rule.prerequisites.iter().any(|level| *level >= rule.level)
                || rule.prerequisites.iter().collect::<BTreeSet<_>>().len()
                    != rule.prerequisites.len()
            {
                return Err(ClaimError::Prerequisite);
            }
            for criterion in &rule.criteria {
                if criterion.criterion_id.is_empty()
                    || criterion.fact_ids.is_empty()
                    || !criterion_ids.insert(criterion.criterion_id.as_str())
                {
                    return Err(ClaimError::Identity);
                }
                for fact_id in &criterion.fact_ids {
                    if fact_id.is_empty() || !fact_ids.insert(fact_id.as_str()) {
                        return Err(ClaimError::Duplicate);
                    }
                }
            }
        }
        Ok(())
    }
}

fn evaluate_rule(
    rule: &ClaimRule,
    version: &str,
    facts: &BTreeMap<String, EvidenceState>,
    prior: &BTreeMap<ClaimLevel, ClaimVerdict>,
) -> ClaimResult {
    let mut graph = Vec::new();
    let mut children = Vec::new();
    let mut verdicts = Vec::new();
    for prerequisite in &rule.prerequisites {
        let verdict = prior[prerequisite];
        let node_id = format!("{:?}.prerequisite.{prerequisite:?}", rule.level);
        children.push(node_id.clone());
        verdicts.push(verdict);
        graph.push(ReasonNode {
            node_id,
            verdict,
            reason_code: "CLAIM_PREREQUISITE".to_owned(),
            children: Vec::new(),
        });
    }
    for criterion in &rule.criteria {
        let fact_states = criterion
            .fact_ids
            .iter()
            .map(|fact_id| {
                facts
                    .get(fact_id)
                    .copied()
                    .unwrap_or(EvidenceState::Missing)
            })
            .collect::<Vec<_>>();
        let verdict = fold_evidence(&fact_states);
        children.push(criterion.criterion_id.clone());
        verdicts.push(verdict);
        graph.push(ReasonNode {
            node_id: criterion.criterion_id.clone(),
            verdict,
            reason_code: criterion_reason(&fact_states).to_owned(),
            children: criterion.fact_ids.clone(),
        });
        for (fact_id, state) in criterion.fact_ids.iter().zip(fact_states) {
            graph.push(ReasonNode {
                node_id: fact_id.clone(),
                verdict: fold_evidence(&[state]),
                reason_code: format!("EVIDENCE_{state:?}").to_ascii_uppercase(),
                children: Vec::new(),
            });
        }
    }
    let verdict = fold_verdicts(&verdicts);
    let root_reason_id = format!("{:?}.root", rule.level);
    graph.push(ReasonNode {
        node_id: root_reason_id.clone(),
        verdict,
        reason_code: "CLAIM_RULE_EVALUATED".to_owned(),
        children,
    });
    graph.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    ClaimResult {
        level: rule.level,
        rule_version: version.to_owned(),
        verdict,
        root_reason_id,
        reason_graph: graph,
    }
}

fn fold_evidence(states: &[EvidenceState]) -> ClaimVerdict {
    if states.contains(&EvidenceState::Failed) {
        ClaimVerdict::Fail
    } else if states
        .iter()
        .any(|state| matches!(state, EvidenceState::Missing | EvidenceState::Contradictory))
    {
        ClaimVerdict::Indeterminate
    } else {
        ClaimVerdict::Pass
    }
}

fn fold_verdicts(verdicts: &[ClaimVerdict]) -> ClaimVerdict {
    if verdicts.contains(&ClaimVerdict::Fail) {
        ClaimVerdict::Fail
    } else if verdicts.contains(&ClaimVerdict::Indeterminate) {
        ClaimVerdict::Indeterminate
    } else {
        ClaimVerdict::Pass
    }
}

fn criterion_reason(states: &[EvidenceState]) -> &'static str {
    match fold_evidence(states) {
        ClaimVerdict::Pass => "CRITERION_PASSED",
        ClaimVerdict::Fail => "CRITERION_FAILED",
        ClaimVerdict::Indeterminate => "CRITERION_INDETERMINATE",
    }
}

/// Return the versioned agent-neutral skeleton expanded by later BV prompts.
#[must_use]
pub fn default_rule_set() -> RuleSet {
    let levels = [
        (
            ClaimLevel::C0,
            vec![],
            "valid_run_evidence",
            EvidenceTier::LocalStructural,
        ),
        (
            ClaimLevel::C1,
            vec![ClaimLevel::C0],
            "budget_compliance",
            EvidenceTier::HostedSemantic,
        ),
        (
            ClaimLevel::C2,
            vec![ClaimLevel::C1],
            "continual_adaptation",
            EvidenceTier::Confirmatory,
        ),
        (
            ClaimLevel::C3,
            vec![ClaimLevel::C2],
            "abstraction_utility",
            EvidenceTier::Confirmatory,
        ),
        (
            ClaimLevel::C4,
            vec![ClaimLevel::C3],
            "construction_credit_cycle",
            EvidenceTier::Confirmatory,
        ),
        (
            ClaimLevel::C5,
            vec![ClaimLevel::C4],
            "multigenerational_gain",
            EvidenceTier::PhysicalHost,
        ),
    ];
    RuleSet {
        rule_version: "1.0.0".to_owned(),
        rules: levels
            .into_iter()
            .map(|(level, prerequisites, fact, evidence_tier)| ClaimRule {
                level,
                prerequisites,
                criteria: vec![Criterion {
                    criterion_id: format!("{level:?}.criterion"),
                    evidence_tier,
                    fact_ids: vec![fact.to_owned()],
                }],
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{ClaimLevel, ClaimVerdict, EvidenceState, default_rule_set};
    use std::collections::BTreeMap;

    fn all_pass() -> BTreeMap<String, EvidenceState> {
        [
            "valid_run_evidence",
            "budget_compliance",
            "continual_adaptation",
            "abstraction_utility",
            "construction_credit_cycle",
            "multigenerational_gain",
        ]
        .into_iter()
        .map(|fact| (fact.to_owned(), EvidenceState::Passed))
        .collect()
    }

    #[test]
    fn version_is_stored_and_complete_evidence_passes() {
        let evaluation = default_rule_set()
            .evaluate(&all_pass())
            .expect("evaluation");
        assert_eq!(evaluation.rule_version, "1.0.0");
        assert!(evaluation.results.iter().all(|result| {
            result.rule_version == "1.0.0" && result.verdict == ClaimVerdict::Pass
        }));
    }

    #[test]
    fn missing_failed_and_contradictory_evidence_never_pass() {
        for state in [
            EvidenceState::Missing,
            EvidenceState::Failed,
            EvidenceState::Contradictory,
        ] {
            let mut facts = all_pass();
            facts.insert("valid_run_evidence".to_owned(), state);
            let evaluation = default_rule_set().evaluate(&facts).expect("evaluation");
            assert!(
                evaluation
                    .results
                    .iter()
                    .all(|result| result.verdict != ClaimVerdict::Pass)
            );
        }
    }

    #[test]
    fn prerequisite_verdicts_block_higher_claims_with_reason_graphs() {
        let mut facts = all_pass();
        facts.insert("budget_compliance".to_owned(), EvidenceState::Failed);
        let evaluation = default_rule_set().evaluate(&facts).expect("evaluation");
        assert_eq!(evaluation.results[0].verdict, ClaimVerdict::Pass);
        assert_eq!(evaluation.results[1].verdict, ClaimVerdict::Fail);
        assert!(
            evaluation.results[2..]
                .iter()
                .all(|result| result.verdict == ClaimVerdict::Fail)
        );
        let c2 = evaluation
            .results
            .iter()
            .find(|result| result.level == ClaimLevel::C2)
            .expect("C2");
        assert!(
            c2.reason_graph
                .iter()
                .any(|node| node.reason_code == "CLAIM_PREREQUISITE")
        );
    }
}
