//! Cross-episode learning-loop planner — private MVP (P8-T06).
//!
//! Reflexion-family failure experience becomes a digest-bound Memory candidate
//! proposal, then may be admitted only through `decide_memory_admission`.
//! The planner never self-authorizes, never writes authority SQLite, and never
//! promotes lessons into Policy/Model/Verifier controls. Forget remains an
//! explainable, non-resurrecting tombstone plan.

use cognitive_kernel::memory_admission::{
    CurrentMemorySourceFacts, MemoryAdmissionDecision, MemoryAdmissionOutcome,
    MemoryAdmissionPolicy, MemoryProposal, decide_memory_admission,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

const TARGET_MEMORY_CANDIDATE: &str = "memory_candidate";
const TARGET_SKILL_CANDIDATE: &str = "skill_candidate";
const LINEAGE_REFLEXION: &str = "reflexion_failure_lesson";
const FORGET_REASON: &str = "learning_memory_forgotten";
const REVOKE_REASON: &str = "learning_skill_binding_revoked";

/// One episode failure lesson eligible for Memory-candidate planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureLessonFact {
    pub episode_id: String,
    pub lesson_digest: String,
    pub source_digest: String,
    pub purpose: String,
    /// Declared promotion target; only `memory_candidate` is accepted.
    pub promotion_target: String,
    pub claims_authority: bool,
}

/// Non-authoritative Memory candidate plan derived from a failure lesson.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningMemoryCandidatePlan {
    pub plan_digest: String,
    pub candidate_id: String,
    pub source_digest: String,
    pub purpose: String,
    pub lesson_digest: String,
    pub lineage: &'static str,
}

/// Explainable forget/tombstone plan for an admitted learning Memory object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningMemoryForgetPlan {
    pub forget_digest: String,
    pub memory_id: String,
    pub lesson_digest: String,
    pub reason: &'static str,
    pub resurrectable: bool,
}

/// Non-authoritative Skill candidate plan derived from a failure lesson.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningSkillCandidatePlan {
    pub plan_digest: String,
    pub package_key: String,
    pub revision_digest: String,
    pub lesson_digest: String,
    pub lineage: &'static str,
    pub grants_capability: bool,
}

/// Explainable Skill binding revocation plan (non-resurrecting).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningSkillRevokePlan {
    pub revoke_digest: String,
    pub binding_id: String,
    pub lesson_digest: String,
    pub reason: &'static str,
    pub resurrectable: bool,
}

/// Fail-closed learning-loop planner errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LearningLoopError {
    #[error("learning lesson is missing required identity or digest material")]
    MissingIdentity,
    #[error("learning proposal cannot self-authorize durable admission")]
    SelfAuthorizationForbidden,
    #[error("learning lesson may only promote to a Memory or Skill candidate")]
    DirectPromotionForbidden,
    #[error("learning admission rejects a producer-forged outcome")]
    ProducerForgedAdmit,
    #[error("learning admission source facts do not match the candidate plan")]
    SourceMismatch,
    #[error("learning Skill candidate cannot grant capability")]
    CapabilityGrantForbidden,
}

/// Plan a Memory candidate from a Reflexion-family failure lesson.
///
/// Output is a proposal shape for daemon admission. It does not admit,
/// persist, or grant capability.
pub fn plan_failure_lesson_memory_candidate(
    lesson: &FailureLessonFact,
) -> Result<LearningMemoryCandidatePlan, LearningLoopError> {
    if lesson.claims_authority {
        return Err(LearningLoopError::SelfAuthorizationForbidden);
    }
    if lesson.episode_id.trim().is_empty()
        || lesson.lesson_digest.trim().is_empty()
        || lesson.source_digest.trim().is_empty()
        || lesson.purpose.trim().is_empty()
    {
        return Err(LearningLoopError::MissingIdentity);
    }
    if lesson.promotion_target != TARGET_MEMORY_CANDIDATE {
        return Err(LearningLoopError::DirectPromotionForbidden);
    }

    let candidate_id = format!("learn.memory.{}", lesson.episode_id.trim());
    let plan_digest = bind_plan_digest(
        &candidate_id,
        &lesson.lesson_digest,
        &lesson.source_digest,
        &lesson.purpose,
    );
    Ok(LearningMemoryCandidatePlan {
        plan_digest,
        candidate_id,
        source_digest: lesson.source_digest.clone(),
        purpose: lesson.purpose.clone(),
        lesson_digest: lesson.lesson_digest.clone(),
        lineage: LINEAGE_REFLEXION,
    })
}

/// Build a daemon Memory proposal from a learning plan and current source facts.
pub fn memory_proposal_from_learning_plan(
    plan: &LearningMemoryCandidatePlan,
    current_source: &CurrentMemorySourceFacts,
    retention_expires_at_unix_seconds: i64,
) -> Result<MemoryProposal, LearningLoopError> {
    if plan.source_digest != current_source.source_digest {
        return Err(LearningLoopError::SourceMismatch);
    }
    Ok(MemoryProposal {
        candidate_id: plan.candidate_id.clone(),
        source_id: current_source.source_id.clone(),
        source_digest: current_source.source_digest.clone(),
        source_provenance_ref: current_source.provenance_ref.clone(),
        governance_scope: current_source.governance_scope.clone(),
        target_scope: current_source.governance_scope.clone(),
        purpose: plan.purpose.clone(),
        retention_expires_at_unix_seconds,
        observed_at_unix_seconds: current_source.observed_at_unix_seconds,
    })
}

/// Admit a learning Memory candidate only through daemon policy derivation.
///
/// `requested_outcome` must match `decide_memory_admission`; forged Admit fails
/// closed. This does not write SQLite — it gates the durable append path.
pub fn decide_learning_memory_admission(
    plan: &LearningMemoryCandidatePlan,
    current_source: &CurrentMemorySourceFacts,
    policy: &MemoryAdmissionPolicy,
    retention_expires_at_unix_seconds: i64,
    requested_outcome: MemoryAdmissionOutcome,
) -> Result<MemoryAdmissionDecision, LearningLoopError> {
    let proposal = memory_proposal_from_learning_plan(
        plan,
        current_source,
        retention_expires_at_unix_seconds,
    )?;
    let derived = decide_memory_admission(&proposal, current_source, policy);
    if requested_outcome != derived.outcome {
        return Err(LearningLoopError::ProducerForgedAdmit);
    }
    Ok(derived)
}

/// Plan an explainable, non-resurrecting forget for a learning Memory object.
pub fn plan_learning_memory_forget(
    memory_id: &str,
    lesson_digest: &str,
) -> Result<LearningMemoryForgetPlan, LearningLoopError> {
    if memory_id.trim().is_empty() || lesson_digest.trim().is_empty() {
        return Err(LearningLoopError::MissingIdentity);
    }
    let mut hasher = Sha256::new();
    hasher.update(FORGET_REASON.as_bytes());
    hasher.update(b"\0");
    hasher.update(memory_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(lesson_digest.as_bytes());
    Ok(LearningMemoryForgetPlan {
        forget_digest: format!("{:x}", hasher.finalize()),
        memory_id: memory_id.to_owned(),
        lesson_digest: lesson_digest.to_owned(),
        reason: FORGET_REASON,
        resurrectable: false,
    })
}

/// Plan a Skill package candidate from a Reflexion-family failure lesson.
///
/// Output is provenance-only for later `import_local_skill_package`. It never
/// grants capability or executes package content.
pub fn plan_failure_lesson_skill_candidate(
    lesson: &FailureLessonFact,
    package_manifest_digest: &str,
    claims_capability: bool,
) -> Result<LearningSkillCandidatePlan, LearningLoopError> {
    if lesson.claims_authority {
        return Err(LearningLoopError::SelfAuthorizationForbidden);
    }
    if claims_capability {
        return Err(LearningLoopError::CapabilityGrantForbidden);
    }
    if lesson.episode_id.trim().is_empty()
        || lesson.lesson_digest.trim().is_empty()
        || package_manifest_digest.trim().is_empty()
    {
        return Err(LearningLoopError::MissingIdentity);
    }
    if lesson.promotion_target != TARGET_SKILL_CANDIDATE {
        return Err(LearningLoopError::DirectPromotionForbidden);
    }

    let package_key = format!("learn.skill.{}", lesson.episode_id.trim());
    let mut hasher = Sha256::new();
    hasher.update(LINEAGE_REFLEXION.as_bytes());
    hasher.update(b"\0");
    hasher.update(package_key.as_bytes());
    hasher.update(b"\0");
    hasher.update(lesson.lesson_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(package_manifest_digest.as_bytes());
    Ok(LearningSkillCandidatePlan {
        plan_digest: format!("{:x}", hasher.finalize()),
        package_key,
        revision_digest: package_manifest_digest.to_owned(),
        lesson_digest: lesson.lesson_digest.clone(),
        lineage: LINEAGE_REFLEXION,
        grants_capability: false,
    })
}

/// Plan an explainable, non-resurrecting Skill binding revocation.
pub fn plan_learning_skill_binding_revoke(
    binding_id: &str,
    lesson_digest: &str,
) -> Result<LearningSkillRevokePlan, LearningLoopError> {
    if binding_id.trim().is_empty() || lesson_digest.trim().is_empty() {
        return Err(LearningLoopError::MissingIdentity);
    }
    let mut hasher = Sha256::new();
    hasher.update(REVOKE_REASON.as_bytes());
    hasher.update(b"\0");
    hasher.update(binding_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(lesson_digest.as_bytes());
    Ok(LearningSkillRevokePlan {
        revoke_digest: format!("{:x}", hasher.finalize()),
        binding_id: binding_id.to_owned(),
        lesson_digest: lesson_digest.to_owned(),
        reason: REVOKE_REASON,
        resurrectable: false,
    })
}

fn bind_plan_digest(
    candidate_id: &str,
    lesson_digest: &str,
    source_digest: &str,
    purpose: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(LINEAGE_REFLEXION.as_bytes());
    hasher.update(b"\0");
    hasher.update(candidate_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(lesson_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(source_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(purpose.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn valid_lesson() -> FailureLessonFact {
        FailureLessonFact {
            episode_id: "ep-42".to_owned(),
            lesson_digest: format!("sha256:{}", "a".repeat(64)),
            source_digest: format!("sha256:{}", "b".repeat(64)),
            purpose: "retain explainable failure lesson".to_owned(),
            promotion_target: TARGET_MEMORY_CANDIDATE.to_owned(),
            claims_authority: false,
        }
    }

    fn current_source_for(plan: &LearningMemoryCandidatePlan) -> CurrentMemorySourceFacts {
        CurrentMemorySourceFacts {
            source_id: "context-source-1".to_owned(),
            source_digest: plan.source_digest.clone(),
            provenance_ref: "workspace://source/1".to_owned(),
            governance_scope: "workspace://personal".to_owned(),
            observed_at_unix_seconds: 100,
        }
    }

    fn policy() -> MemoryAdmissionPolicy {
        MemoryAdmissionPolicy {
            policy_version: 1,
            now_unix_seconds: 100,
            maximum_retention_seconds: 3600,
        }
    }

    #[test]
    fn plans_digest_bound_memory_candidate_from_failure_lesson() {
        let plan = plan_failure_lesson_memory_candidate(&valid_lesson()).expect("plan");
        assert_eq!(plan.candidate_id, "learn.memory.ep-42");
        assert_eq!(plan.lineage, LINEAGE_REFLEXION);
        assert_eq!(plan.plan_digest.len(), 64);
        assert_eq!(plan.lesson_digest, valid_lesson().lesson_digest);
    }

    #[test]
    fn rejects_self_authorization_direct_promotion_and_missing_identity() {
        let mut self_auth = valid_lesson();
        self_auth.claims_authority = true;
        assert_eq!(
            plan_failure_lesson_memory_candidate(&self_auth).unwrap_err(),
            LearningLoopError::SelfAuthorizationForbidden
        );

        let mut direct = valid_lesson();
        direct.promotion_target = "policy".to_owned();
        assert_eq!(
            plan_failure_lesson_memory_candidate(&direct).unwrap_err(),
            LearningLoopError::DirectPromotionForbidden
        );
        direct.promotion_target = "verifier".to_owned();
        assert_eq!(
            plan_failure_lesson_memory_candidate(&direct).unwrap_err(),
            LearningLoopError::DirectPromotionForbidden
        );

        let mut missing = valid_lesson();
        missing.purpose = "  ".to_owned();
        assert_eq!(
            plan_failure_lesson_memory_candidate(&missing).unwrap_err(),
            LearningLoopError::MissingIdentity
        );
    }

    #[test]
    fn admits_only_through_daemon_policy_and_rejects_forged_outcome() {
        let plan = plan_failure_lesson_memory_candidate(&valid_lesson()).expect("plan");
        let source = current_source_for(&plan);
        let decision = decide_learning_memory_admission(
            &plan,
            &source,
            &policy(),
            3700,
            MemoryAdmissionOutcome::Admit,
        )
        .expect("admit");
        assert_eq!(decision.outcome, MemoryAdmissionOutcome::Admit);

        assert_eq!(
            decide_learning_memory_admission(
                &plan,
                &source,
                &policy(),
                3700,
                MemoryAdmissionOutcome::Reject,
            )
            .unwrap_err(),
            LearningLoopError::ProducerForgedAdmit
        );

        let mut mismatched = source.clone();
        mismatched.source_digest = format!("sha256:{}", "c".repeat(64));
        assert_eq!(
            decide_learning_memory_admission(
                &plan,
                &mismatched,
                &policy(),
                3700,
                MemoryAdmissionOutcome::Admit,
            )
            .unwrap_err(),
            LearningLoopError::SourceMismatch
        );
    }

    #[test]
    fn plans_explainable_non_resurrecting_forget() {
        let forget =
            plan_learning_memory_forget("mem-learn-1", &format!("sha256:{}", "a".repeat(64)))
                .expect("forget");
        assert!(!forget.resurrectable);
        assert_eq!(forget.reason, FORGET_REASON);
        assert_eq!(forget.forget_digest.len(), 64);
        assert_eq!(
            plan_learning_memory_forget("  ", "sha256:x").unwrap_err(),
            LearningLoopError::MissingIdentity
        );
    }

    #[test]
    fn plans_skill_candidate_and_rejects_capability_grant() {
        let mut lesson = valid_lesson();
        lesson.promotion_target = TARGET_SKILL_CANDIDATE.to_owned();
        let plan = plan_failure_lesson_skill_candidate(
            &lesson,
            &format!("sha256:{}", "d".repeat(64)),
            false,
        )
        .expect("skill plan");
        assert_eq!(plan.package_key, "learn.skill.ep-42");
        assert!(!plan.grants_capability);
        assert_eq!(plan.plan_digest.len(), 64);

        assert_eq!(
            plan_failure_lesson_skill_candidate(
                &lesson,
                &format!("sha256:{}", "d".repeat(64)),
                true,
            )
            .unwrap_err(),
            LearningLoopError::CapabilityGrantForbidden
        );

        let mut memory_target = lesson.clone();
        memory_target.promotion_target = TARGET_MEMORY_CANDIDATE.to_owned();
        assert_eq!(
            plan_failure_lesson_skill_candidate(
                &memory_target,
                &format!("sha256:{}", "d".repeat(64)),
                false,
            )
            .unwrap_err(),
            LearningLoopError::DirectPromotionForbidden
        );

        let revoke =
            plan_learning_skill_binding_revoke("bind-1", &format!("sha256:{}", "a".repeat(64)))
                .expect("revoke");
        assert!(!revoke.resurrectable);
        assert_eq!(revoke.reason, REVOKE_REASON);
    }
}
