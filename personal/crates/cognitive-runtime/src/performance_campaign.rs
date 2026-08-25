//! Secret-free measurement envelope for P9-T04 campaign evidence.
//!
//! The envelope records correlation, monotonic timing, and Provider usage
//! availability without receiving credentials, prompts, responses, headers, or
//! authority-store contents. It is intentionally not an authority writer.

use serde::Serialize;

const CAMPAIGN_CORRELATION_ID_PREFIX: &str = "campaign-";
const CAMPAIGN_CORRELATION_ID_HEX_LENGTH: usize = 32;

/// A campaign correlation identifier supplied by the daemon-owned execution
/// path. It is opaque metadata, never a bearer, SecretRef, prompt, or response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CampaignCorrelationId(String);

impl CampaignCorrelationId {
    /// Accept the fixed-width opaque identifier emitted for one campaign task.
    pub fn parse(value: impl Into<String>) -> Result<Self, PerformanceCampaignError> {
        let value = value.into();
        let Some(hexadecimal_suffix) = value.strip_prefix(CAMPAIGN_CORRELATION_ID_PREFIX) else {
            return Err(PerformanceCampaignError::InvalidCorrelationId);
        };
        if hexadecimal_suffix.len() != CAMPAIGN_CORRELATION_ID_HEX_LENGTH
            || !hexadecimal_suffix
                .bytes()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(PerformanceCampaignError::InvalidCorrelationId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A measured stage from the campaign execution plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStage {
    SessionMint,
    IntentRecord,
    Interpret,
    Preview,
    Admit,
    SchedulerWait,
    Context,
    Cache,
    PiLaunch,
    ProviderPreflight,
    ProviderNetwork,
    CandidateParse,
    IntentPersist,
    Dispatch,
    Reconcile,
    Verification,
    Acceptance,
}

/// One elapsed monotonic interval. The runner receives elapsed duration only,
/// so neither wall-clock timestamps nor user data enter campaign evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CampaignStageTiming {
    pub stage: CampaignStage,
    pub elapsed_nanos: u128,
}

/// Provider usage is measured only when the Provider explicitly returns all
/// finite counters needed for the report. A missing value is not a zero value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub enum ProviderUsage {
    NotAvailable,
    Measured {
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
    },
}

impl ProviderUsage {
    pub fn from_reported_counts(
        prompt_tokens: Option<u64>,
        completion_tokens: Option<u64>,
        total_tokens: Option<u64>,
    ) -> Result<Self, PerformanceCampaignError> {
        match (prompt_tokens, completion_tokens, total_tokens) {
            (None, None, None) => Ok(Self::NotAvailable),
            (Some(prompt_tokens), Some(completion_tokens), Some(total_tokens))
                if prompt_tokens
                    .checked_add(completion_tokens)
                    .is_some_and(|calculated_total| calculated_total == total_tokens) =>
            {
                Ok(Self::Measured {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                })
            }
            _ => Err(PerformanceCampaignError::InconsistentProviderUsage),
        }
    }
}

/// A redacted measurement record. It deliberately contains no request or
/// response content and cannot be used to advance authority state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CampaignMeasurementEnvelope {
    pub campaign_id: String,
    pub correlation_id: CampaignCorrelationId,
    pub source_revision: String,
    pub timings: Vec<CampaignStageTiming>,
    pub provider_usage: ProviderUsage,
}

impl CampaignMeasurementEnvelope {
    pub fn new(
        campaign_id: impl Into<String>,
        correlation_id: CampaignCorrelationId,
        source_revision: impl Into<String>,
        timings: Vec<CampaignStageTiming>,
        provider_usage: ProviderUsage,
    ) -> Result<Self, PerformanceCampaignError> {
        let campaign_id = campaign_id.into();
        let source_revision = source_revision.into();
        if campaign_id.trim().is_empty() {
            return Err(PerformanceCampaignError::MissingCampaignId);
        }
        if !is_full_hex_revision(&source_revision) {
            return Err(PerformanceCampaignError::InvalidSourceRevision);
        }
        if timings.is_empty() || timings.iter().any(|timing| timing.elapsed_nanos == 0) {
            return Err(PerformanceCampaignError::InvalidTiming);
        }
        Ok(Self {
            campaign_id,
            correlation_id,
            source_revision,
            timings,
            provider_usage,
        })
    }
}

fn is_full_hex_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|character| character.is_ascii_hexdigit())
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PerformanceCampaignError {
    #[error(
        "campaign correlation ID must be an opaque campaign- prefixed 32-character hexadecimal value"
    )]
    InvalidCorrelationId,
    #[error("campaign ID must not be empty")]
    MissingCampaignId,
    #[error("source revision must be a full 40-character hexadecimal Git revision")]
    InvalidSourceRevision,
    #[error("campaign timing records must be non-empty with positive elapsed durations")]
    InvalidTiming,
    #[error("Provider usage must be fully available and internally consistent, or not_available")]
    InconsistentProviderUsage,
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    const VALID_CORRELATION_ID: &str = "campaign-0123456789abcdef0123456789abcdef";
    const VALID_REVISION: &str = "9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c";

    #[test]
    fn missing_provider_counts_remain_not_available() {
        assert_eq!(
            ProviderUsage::from_reported_counts(None, None, None).unwrap(),
            ProviderUsage::NotAvailable
        );
    }

    #[test]
    fn partial_or_inconsistent_provider_counts_fail_closed() {
        assert_eq!(
            ProviderUsage::from_reported_counts(Some(2), None, Some(2)).unwrap_err(),
            PerformanceCampaignError::InconsistentProviderUsage
        );
        assert_eq!(
            ProviderUsage::from_reported_counts(Some(2), Some(3), Some(4)).unwrap_err(),
            PerformanceCampaignError::InconsistentProviderUsage
        );
    }

    #[test]
    fn envelope_rejects_secret_shaped_correlation_input() {
        let error = CampaignCorrelationId::parse("provider-key-value").unwrap_err();
        assert_eq!(error, PerformanceCampaignError::InvalidCorrelationId);
    }

    #[test]
    fn envelope_rejects_zero_duration_to_prevent_fabricated_stage_evidence() {
        let error = CampaignMeasurementEnvelope::new(
            "P9-T04-comprehensive-performance-001",
            CampaignCorrelationId::parse(VALID_CORRELATION_ID).unwrap(),
            VALID_REVISION,
            vec![CampaignStageTiming {
                stage: CampaignStage::ProviderNetwork,
                elapsed_nanos: 0,
            }],
            ProviderUsage::NotAvailable,
        )
        .unwrap_err();
        assert_eq!(error, PerformanceCampaignError::InvalidTiming);
    }
}
