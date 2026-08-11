//! Context compaction planner — private MVP (P8-T05/D01).
//!
//! Daemon-owned compaction turns digest-bound session/Context source material
//! into a digest-bound compact artifact with explicit loss records. Compact
//! output is a Context source candidate only: it cannot complete Tasks, grant
//! capability, or self-authorize inclusion. Adaptive budgets and UCR-01
//! benefit observation remain later slices.

use sha2::{Digest, Sha256};
use thiserror::Error;

/// One immutable source fact eligible for compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionSourceFact {
    pub source_id: String,
    pub content_digest: String,
}

/// Explicit loss recorded when a source is omitted from the compact artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionLossRecord {
    pub source_id: String,
    pub content_digest: String,
    pub reason: &'static str,
}

/// Digest-bound compact artifact (Context source candidate, non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactContextArtifact {
    pub artifact_digest: String,
    pub retained_source_digests: Vec<String>,
    pub losses: Vec<CompactionLossRecord>,
    pub summary_digest: String,
}

/// Fail-closed compaction errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContextCompactionError {
    #[error("compaction source or summary is missing required identity or digest material")]
    MissingIdentity,
    #[error("model-written summary cannot self-authorize inclusion")]
    SelfAuthorizationForbidden,
    #[error("compaction requires at least one retained source digest")]
    EmptyRetention,
}

/// Plan a daemon-owned compact artifact over exact source digests.
///
/// `retain_source_ids` selects which sources remain represented in the compact
/// view; omitted sources become explicit loss records. `summary_claims_authority`
/// must be false — model summaries never self-authorize.
pub fn plan_context_compaction(
    sources: &[CompactionSourceFact],
    retain_source_ids: &[&str],
    summary_digest: &str,
    summary_claims_authority: bool,
) -> Result<CompactContextArtifact, ContextCompactionError> {
    if summary_digest.trim().is_empty() {
        return Err(ContextCompactionError::MissingIdentity);
    }
    if summary_claims_authority {
        return Err(ContextCompactionError::SelfAuthorizationForbidden);
    }
    if sources.is_empty() || retain_source_ids.is_empty() {
        return Err(ContextCompactionError::EmptyRetention);
    }

    for source in sources {
        if source.source_id.trim().is_empty() || source.content_digest.trim().is_empty() {
            return Err(ContextCompactionError::MissingIdentity);
        }
    }

    let mut retained_source_digests = Vec::new();
    let mut losses = Vec::new();
    for source in sources {
        if retain_source_ids
            .iter()
            .any(|retain_id| *retain_id == source.source_id)
        {
            retained_source_digests.push(source.content_digest.clone());
        } else {
            losses.push(CompactionLossRecord {
                source_id: source.source_id.clone(),
                content_digest: source.content_digest.clone(),
                reason: "omitted_by_compaction",
            });
        }
    }

    if retained_source_digests.is_empty() {
        return Err(ContextCompactionError::EmptyRetention);
    }

    let artifact_digest =
        bind_compact_artifact_digest(&retained_source_digests, &losses, summary_digest);
    Ok(CompactContextArtifact {
        artifact_digest,
        retained_source_digests,
        losses,
        summary_digest: summary_digest.to_owned(),
    })
}

fn bind_compact_artifact_digest(
    retained_source_digests: &[String],
    losses: &[CompactionLossRecord],
    summary_digest: &str,
) -> String {
    let mut hasher = Sha256::new();
    for digest in retained_source_digests {
        hasher.update(digest.as_bytes());
        hasher.update(b"\0");
    }
    for loss in losses {
        hasher.update(loss.source_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(loss.content_digest.as_bytes());
        hasher.update(b"\0");
        hasher.update(loss.reason.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(summary_digest.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn source(id: &str, digest_seed: char) -> CompactionSourceFact {
        CompactionSourceFact {
            source_id: id.to_owned(),
            content_digest: format!("sha256:{}", digest_seed.to_string().repeat(64)),
        }
    }

    #[test]
    fn plans_digest_bound_compact_artifact_with_explicit_loss() {
        let sources = [source("src.keep", 'a'), source("src.drop", 'b')];
        let artifact = plan_context_compaction(
            &sources,
            &["src.keep"],
            &format!("sha256:{}", "c".repeat(64)),
            false,
        )
        .expect("plan");
        assert_eq!(artifact.retained_source_digests.len(), 1);
        assert_eq!(artifact.losses.len(), 1);
        assert_eq!(artifact.losses[0].source_id, "src.drop");
        assert_eq!(artifact.losses[0].reason, "omitted_by_compaction");
        assert_eq!(artifact.artifact_digest.len(), 64);
    }

    #[test]
    fn rejects_self_authorization_and_missing_identity() {
        let sources = [source("src.keep", 'a')];
        assert_eq!(
            plan_context_compaction(
                &sources,
                &["src.keep"],
                &format!("sha256:{}", "c".repeat(64)),
                true,
            )
            .unwrap_err(),
            ContextCompactionError::SelfAuthorizationForbidden
        );
        assert_eq!(
            plan_context_compaction(&sources, &["src.keep"], "  ", false).unwrap_err(),
            ContextCompactionError::MissingIdentity
        );
        assert_eq!(
            plan_context_compaction(
                &sources,
                &["src.missing"],
                &format!("sha256:{}", "c".repeat(64)),
                false,
            )
            .unwrap_err(),
            ContextCompactionError::EmptyRetention
        );
    }
}
