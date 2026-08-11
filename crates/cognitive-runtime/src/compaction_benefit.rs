//! Compaction benefit observation — private MVP (P8-T05/D03).
//!
//! UCR-01-compatible non-claim observation over digest-bound compaction and
//! adaptive-budget facts. Observations never set Gate, release, Profile, or
//! Task-completion authority.

use sha2::{Digest, Sha256};
use thiserror::Error;

const CLAIM_SCOPE_NON_CLAIM: &str = "non-claim";
const SCENARIO_UCR_01: &str = "UCR-01";
const PROHIBITED_CLAIM_LABELS: &[&str] = &[
    "gate",
    "release",
    "profile",
    "completion",
    "passed",
    "pass",
];

/// Durable compaction/budget facts used for a non-claim benefit observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionBenefitFacts {
    pub compact_artifact_digest: String,
    pub summary_digest: String,
    pub retained_source_count: u64,
    pub loss_count: u64,
    pub adapted_fragment_budget: u64,
    pub claim_scope: String,
    pub scenario_id: String,
}

/// Digest-bound non-claim observation (UCR-01 compatible; non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionBenefitObservation {
    pub observation_digest: String,
    pub claim_scope: &'static str,
    pub scenario_id: &'static str,
    pub compact_artifact_digest: String,
    pub summary_digest: String,
    pub retained_source_count: u64,
    pub loss_count: u64,
    pub adapted_fragment_budget: u64,
}

/// Fail-closed compaction benefit observation errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompactionBenefitError {
    #[error("compaction benefit observation requires non-empty digests")]
    MissingDigest,
    #[error("compaction benefit observation claim_scope must be non-claim")]
    ClaimScopeMustBeNonClaim,
    #[error("compaction benefit observation scenario_id must be UCR-01")]
    ScenarioMustBeUcr01,
    #[error("compaction benefit observation rejects Gate/authority-shaped claims")]
    AuthorityShapedClaimForbidden,
}

/// Build a UCR-01-compatible non-claim observation over compaction digests.
///
/// `authority_claim_labels` must be empty of Gate/release/Profile/completion/
/// pass keys — benefit observations cannot assert authority outcomes.
pub fn observe_compaction_benefit(
    facts: &CompactionBenefitFacts,
    authority_claim_labels: &[&str],
) -> Result<CompactionBenefitObservation, CompactionBenefitError> {
    if facts.compact_artifact_digest.trim().is_empty() || facts.summary_digest.trim().is_empty() {
        return Err(CompactionBenefitError::MissingDigest);
    }
    if facts.claim_scope != CLAIM_SCOPE_NON_CLAIM {
        return Err(CompactionBenefitError::ClaimScopeMustBeNonClaim);
    }
    if facts.scenario_id != SCENARIO_UCR_01 {
        return Err(CompactionBenefitError::ScenarioMustBeUcr01);
    }
    for label in authority_claim_labels {
        let normalized = label.trim().to_ascii_lowercase();
        if PROHIBITED_CLAIM_LABELS
            .iter()
            .any(|prohibited| *prohibited == normalized)
        {
            return Err(CompactionBenefitError::AuthorityShapedClaimForbidden);
        }
    }

    let observation_digest = bind_observation_digest(facts);
    Ok(CompactionBenefitObservation {
        observation_digest,
        claim_scope: CLAIM_SCOPE_NON_CLAIM,
        scenario_id: SCENARIO_UCR_01,
        compact_artifact_digest: facts.compact_artifact_digest.clone(),
        summary_digest: facts.summary_digest.clone(),
        retained_source_count: facts.retained_source_count,
        loss_count: facts.loss_count,
        adapted_fragment_budget: facts.adapted_fragment_budget,
    })
}

fn bind_observation_digest(facts: &CompactionBenefitFacts) -> String {
    let mut hasher = Sha256::new();
    hasher.update(SCENARIO_UCR_01.as_bytes());
    hasher.update(b"\0");
    hasher.update(CLAIM_SCOPE_NON_CLAIM.as_bytes());
    hasher.update(b"\0");
    hasher.update(facts.compact_artifact_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(facts.summary_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(facts.retained_source_count.to_le_bytes());
    hasher.update(facts.loss_count.to_le_bytes());
    hasher.update(facts.adapted_fragment_budget.to_le_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn valid_facts() -> CompactionBenefitFacts {
        CompactionBenefitFacts {
            compact_artifact_digest: format!("sha256:{}", "a".repeat(64)),
            summary_digest: format!("sha256:{}", "b".repeat(64)),
            retained_source_count: 2,
            loss_count: 1,
            adapted_fragment_budget: 11,
            claim_scope: CLAIM_SCOPE_NON_CLAIM.to_owned(),
            scenario_id: SCENARIO_UCR_01.to_owned(),
        }
    }

    #[test]
    fn observes_digest_bound_non_claim_benefit() {
        let observation =
            observe_compaction_benefit(&valid_facts(), &[]).expect("observe");
        assert_eq!(observation.claim_scope, CLAIM_SCOPE_NON_CLAIM);
        assert_eq!(observation.scenario_id, SCENARIO_UCR_01);
        assert_eq!(observation.retained_source_count, 2);
        assert_eq!(observation.loss_count, 1);
        assert_eq!(observation.adapted_fragment_budget, 11);
        assert_eq!(observation.observation_digest.len(), 64);
    }

    #[test]
    fn rejects_gate_authority_shaped_claims() {
        assert_eq!(
            observe_compaction_benefit(&valid_facts(), &["Gate"]).unwrap_err(),
            CompactionBenefitError::AuthorityShapedClaimForbidden
        );
        assert_eq!(
            observe_compaction_benefit(&valid_facts(), &["release"]).unwrap_err(),
            CompactionBenefitError::AuthorityShapedClaimForbidden
        );
        assert_eq!(
            observe_compaction_benefit(&valid_facts(), &["profile"]).unwrap_err(),
            CompactionBenefitError::AuthorityShapedClaimForbidden
        );
        assert_eq!(
            observe_compaction_benefit(&valid_facts(), &["completion"]).unwrap_err(),
            CompactionBenefitError::AuthorityShapedClaimForbidden
        );
        assert_eq!(
            observe_compaction_benefit(&valid_facts(), &["passed"]).unwrap_err(),
            CompactionBenefitError::AuthorityShapedClaimForbidden
        );

        let mut bad_scope = valid_facts();
        bad_scope.claim_scope = "gate-pass".to_owned();
        assert_eq!(
            observe_compaction_benefit(&bad_scope, &[]).unwrap_err(),
            CompactionBenefitError::ClaimScopeMustBeNonClaim
        );

        let mut bad_scenario = valid_facts();
        bad_scenario.scenario_id = "B06".to_owned();
        assert_eq!(
            observe_compaction_benefit(&bad_scenario, &[]).unwrap_err(),
            CompactionBenefitError::ScenarioMustBeUcr01
        );

        let mut missing = valid_facts();
        missing.compact_artifact_digest = "  ".to_owned();
        assert_eq!(
            observe_compaction_benefit(&missing, &[]).unwrap_err(),
            CompactionBenefitError::MissingDigest
        );
    }
}
