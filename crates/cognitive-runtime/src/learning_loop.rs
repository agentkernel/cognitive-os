//! Cross-episode learning-loop planner — private MVP (P8-T06/D01).
//!
//! Reflexion-family failure experience becomes a digest-bound Memory candidate
//! proposal only. The planner never self-authorizes, never writes authority
//! SQLite, and never promotes lessons into Policy/Model/Verifier controls.
//! Durable admission remains a later slice through existing Memory admission.

use sha2::{Digest, Sha256};
use thiserror::Error;

const TARGET_MEMORY_CANDIDATE: &str = "memory_candidate";
const LINEAGE_REFLEXION: &str = "reflexion_failure_lesson";

/// One episode failure lesson eligible for Memory-candidate planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureLessonFact {
    pub episode_id: String,
    pub lesson_digest: String,
    pub source_digest: String,
    pub purpose: String,
    /// Declared promotion target; only `memory_candidate` is accepted in D01.
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

/// Fail-closed learning-loop planner errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LearningLoopError {
    #[error("learning lesson is missing required identity or digest material")]
    MissingIdentity,
    #[error("learning proposal cannot self-authorize durable admission")]
    SelfAuthorizationForbidden,
    #[error("learning lesson may only promote to a Memory candidate")]
    DirectPromotionForbidden,
}

/// Plan a Memory candidate from a Reflexion-family failure lesson.
///
/// Output is a proposal shape for later daemon admission. It does not admit,
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
}
