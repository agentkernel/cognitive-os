//! Denominator, safety, cleanup and claim policy for the P9-T04 campaign.
//!
//! ADR-0051 fixes what a comprehensive performance campaign may say. This
//! module is the executable form of that policy: it refuses to publish a
//! report that silently drops a started sample, that survives a safety
//! failure, that promotes itself past independent verification, or that turns
//! an incomplete `L5` into an Agent-benefit claim.

use serde::Serialize;

/// The six execution layers from the campaign execution plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignLayer {
    L0Eligibility,
    L1ModuleBenchmark,
    L2GovernedAndTransport,
    L3ProviderRoute,
    L4GovernedTaskScenarios,
    L5BenefitCampaign,
}

/// Every layer of the campaign must be accounted for, including the ones that
/// were never executed.
pub const REQUIRED_CAMPAIGN_LAYERS: [CampaignLayer; 6] = [
    CampaignLayer::L0Eligibility,
    CampaignLayer::L1ModuleBenchmark,
    CampaignLayer::L2GovernedAndTransport,
    CampaignLayer::L3ProviderRoute,
    CampaignLayer::L4GovernedTaskScenarios,
    CampaignLayer::L5BenefitCampaign,
];

/// Non-claims the report must always carry, so a reader cannot mistake a
/// performance observation for a product decision.
pub const REQUIRED_CAMPAIGN_NON_CLAIMS: [&str; 5] = [
    "no-product-gate",
    "no-release",
    "no-profile",
    "no-generalized-agent-benefit",
    "no-b01-promotion",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerDisposition {
    Completed,
    NotRun,
    Blocked,
    Failed,
}

/// What the report is allowed to assert. The ceiling is `TestedLocal`, and
/// only after independent verification of a complete, safe, cleaned campaign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignClaimLevel {
    NonClaim,
    Hypothesis,
    TestedLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierDisposition {
    NotReviewed,
    Affirmative,
    Negative,
}

/// Per-layer accounting. Warmups are excluded before a cell begins; every
/// started sample stays in the denominator whatever its outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerOutcome {
    pub layer: CampaignLayer,
    pub disposition: LayerDisposition,
    pub started_samples: u64,
    pub retained_samples: u64,
    pub excluded_warmups: u64,
    pub evidence_digest: Option<String>,
}

/// The hard safety conditions. Any non-zero value stops claim promotion; a
/// better latency number can never offset one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CampaignSafetyAccounting {
    pub unauthorized_or_stale_context_exposures: u64,
    pub provider_secret_exposures: u64,
    pub duplicate_external_effects: u64,
    pub false_completions: u64,
    pub stale_epoch_commits: u64,
    pub unreconciled_effects: u64,
    pub completions_without_independent_acceptance: u64,
    /// An `L4` scenario that exceeded the mutation budget its oracle allows,
    /// for example a read-only analysis that wrote a file.
    pub scenario_boundary_violations: u64,
}

impl CampaignSafetyAccounting {
    pub fn total_failures(&self) -> u64 {
        self.unauthorized_or_stale_context_exposures
            .saturating_add(self.provider_secret_exposures)
            .saturating_add(self.duplicate_external_effects)
            .saturating_add(self.false_completions)
            .saturating_add(self.stale_epoch_commits)
            .saturating_add(self.unreconciled_effects)
            .saturating_add(self.completions_without_independent_acceptance)
            .saturating_add(self.scenario_boundary_violations)
    }
}

/// Cleanup facts. The owner's own Provider source file is never touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CampaignCleanupOutcome {
    pub campaign_processes_stopped: bool,
    pub campaign_state_removed: bool,
    pub campaign_secret_entry_removed: bool,
    pub owner_source_file_untouched: bool,
    pub orphan_processes: u64,
    pub orphan_sockets: u64,
    pub stale_locks: u64,
}

impl CampaignCleanupOutcome {
    pub fn is_complete(&self) -> bool {
        self.campaign_processes_stopped
            && self.campaign_state_removed
            && self.campaign_secret_entry_removed
            && self.owner_source_file_untouched
            && self.orphan_processes == 0
            && self.orphan_sockets == 0
            && self.stale_locks == 0
    }
}

/// The single redacted campaign report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CampaignEvidenceReport {
    pub report_kind: &'static str,
    pub campaign_id: String,
    pub source_revision: String,
    pub environment_id: String,
    pub claim_level: CampaignClaimLevel,
    pub benefit_claimed: bool,
    pub layers: Vec<LayerOutcome>,
    pub safety: CampaignSafetyAccounting,
    pub cleanup: CampaignCleanupOutcome,
    pub verifier_disposition: VerifierDisposition,
    pub non_claims: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CampaignReportError {
    #[error("every campaign layer must be accounted for exactly once")]
    IncompleteLayerCoverage,
    #[error("every started sample must stay in the denominator")]
    DroppedStartedSample,
    #[error("a completed layer must have retained at least one started sample")]
    EmptyCompletedLayer,
    #[error("a layer that did not complete must not publish an evidence digest")]
    EvidenceWithoutExecution,
    #[error("a safety failure forces the report to a non-claim")]
    SafetyFailureBlocksClaim,
    #[error("a tested-local claim requires a complete, cleaned, independently verified campaign")]
    UnverifiedClaimPromotion,
    #[error("an incomplete or unverified campaign must not claim Agent benefit")]
    UnsupportedBenefitClaim,
    #[error("campaign reports must carry their registered non-claims")]
    MissingNonClaim,
    #[error("campaign identity and source revision must be recorded")]
    MissingCampaignIdentity,
}

pub const CAMPAIGN_REPORT_KIND: &str = "p9-t04-campaign-evidence/0.1";

/// Assemble the report and refuse anything the campaign policy forbids.
#[allow(clippy::too_many_arguments)] // Every field is a separately preregistered fact.
pub fn build_campaign_evidence_report(
    campaign_id: impl Into<String>,
    source_revision: impl Into<String>,
    environment_id: impl Into<String>,
    claim_level: CampaignClaimLevel,
    benefit_claimed: bool,
    layers: Vec<LayerOutcome>,
    safety: CampaignSafetyAccounting,
    cleanup: CampaignCleanupOutcome,
    verifier_disposition: VerifierDisposition,
) -> Result<CampaignEvidenceReport, CampaignReportError> {
    let report = CampaignEvidenceReport {
        report_kind: CAMPAIGN_REPORT_KIND,
        campaign_id: campaign_id.into(),
        source_revision: source_revision.into(),
        environment_id: environment_id.into(),
        claim_level,
        benefit_claimed,
        layers,
        safety,
        cleanup,
        verifier_disposition,
        non_claims: REQUIRED_CAMPAIGN_NON_CLAIMS.to_vec(),
    };
    validate_campaign_evidence_report(&report)?;
    Ok(report)
}

/// Validate one assembled report against the ADR-0051 campaign policy.
pub fn validate_campaign_evidence_report(
    report: &CampaignEvidenceReport,
) -> Result<(), CampaignReportError> {
    if report.campaign_id.trim().is_empty()
        || !is_full_hex_revision(&report.source_revision)
        || report.environment_id.trim().is_empty()
    {
        return Err(CampaignReportError::MissingCampaignIdentity);
    }
    for non_claim in REQUIRED_CAMPAIGN_NON_CLAIMS {
        if !report.non_claims.contains(&non_claim) {
            return Err(CampaignReportError::MissingNonClaim);
        }
    }
    for required_layer in REQUIRED_CAMPAIGN_LAYERS {
        if report
            .layers
            .iter()
            .filter(|outcome| outcome.layer == required_layer)
            .count()
            != 1
        {
            return Err(CampaignReportError::IncompleteLayerCoverage);
        }
    }
    for outcome in &report.layers {
        if outcome.retained_samples != outcome.started_samples {
            return Err(CampaignReportError::DroppedStartedSample);
        }
        let completed = outcome.disposition == LayerDisposition::Completed;
        if completed && outcome.retained_samples == 0 {
            return Err(CampaignReportError::EmptyCompletedLayer);
        }
        if !completed && outcome.evidence_digest.is_some() {
            return Err(CampaignReportError::EvidenceWithoutExecution);
        }
    }

    let safety_failed = report.safety.total_failures() > 0;
    if safety_failed && report.claim_level != CampaignClaimLevel::NonClaim {
        return Err(CampaignReportError::SafetyFailureBlocksClaim);
    }
    let fully_executed = report
        .layers
        .iter()
        .all(|outcome| outcome.disposition == LayerDisposition::Completed);
    let independently_verified = report.verifier_disposition == VerifierDisposition::Affirmative;
    let promotable =
        fully_executed && independently_verified && !safety_failed && report.cleanup.is_complete();
    if report.claim_level == CampaignClaimLevel::TestedLocal && !promotable {
        return Err(CampaignReportError::UnverifiedClaimPromotion);
    }
    if report.benefit_claimed
        && (!promotable || report.claim_level != CampaignClaimLevel::TestedLocal)
    {
        return Err(CampaignReportError::UnsupportedBenefitClaim);
    }
    Ok(())
}

/// The disposition a campaign must record when a layer could not run. It is a
/// complete report, not an absence of one.
pub fn not_run_layer(layer: CampaignLayer) -> LayerOutcome {
    LayerOutcome {
        layer,
        disposition: LayerDisposition::NotRun,
        started_samples: 0,
        retained_samples: 0,
        excluded_warmups: 0,
        evidence_digest: None,
    }
}

fn is_full_hex_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    const CAMPAIGN_ID: &str = "P9-T04-comprehensive-performance-001";
    const REVISION: &str = "ad3c01751647af2e870c0559cb914a935392aed2";
    const ENVIRONMENT: &str = "DEV-LINUX-NATIVE-01";

    fn completed_layer(layer: CampaignLayer) -> LayerOutcome {
        LayerOutcome {
            layer,
            disposition: LayerDisposition::Completed,
            started_samples: 30,
            retained_samples: 30,
            excluded_warmups: 3,
            evidence_digest: Some(format!("sha256:{}", "ab".repeat(32))),
        }
    }

    fn complete_cleanup() -> CampaignCleanupOutcome {
        CampaignCleanupOutcome {
            campaign_processes_stopped: true,
            campaign_state_removed: true,
            campaign_secret_entry_removed: true,
            owner_source_file_untouched: true,
            orphan_processes: 0,
            orphan_sockets: 0,
            stale_locks: 0,
        }
    }

    fn all_completed_layers() -> Vec<LayerOutcome> {
        REQUIRED_CAMPAIGN_LAYERS.map(completed_layer).to_vec()
    }

    fn build(
        claim_level: CampaignClaimLevel,
        benefit_claimed: bool,
        layers: Vec<LayerOutcome>,
        safety: CampaignSafetyAccounting,
        cleanup: CampaignCleanupOutcome,
        verifier: VerifierDisposition,
    ) -> Result<CampaignEvidenceReport, CampaignReportError> {
        build_campaign_evidence_report(
            CAMPAIGN_ID,
            REVISION,
            ENVIRONMENT,
            claim_level,
            benefit_claimed,
            layers,
            safety,
            cleanup,
            verifier,
        )
    }

    #[test]
    fn complete_verified_safe_campaign_may_reach_tested_local() {
        let report = build(
            CampaignClaimLevel::TestedLocal,
            true,
            all_completed_layers(),
            CampaignSafetyAccounting::default(),
            complete_cleanup(),
            VerifierDisposition::Affirmative,
        )
        .expect("publishable report");
        assert_eq!(report.report_kind, CAMPAIGN_REPORT_KIND);
        assert_eq!(report.non_claims.len(), REQUIRED_CAMPAIGN_NON_CLAIMS.len());
    }

    #[test]
    fn partially_executed_campaign_still_produces_a_hypothesis_report() {
        let mut layers = all_completed_layers();
        layers[5] = not_run_layer(CampaignLayer::L5BenefitCampaign);
        let report = build(
            CampaignClaimLevel::Hypothesis,
            false,
            layers,
            CampaignSafetyAccounting::default(),
            complete_cleanup(),
            VerifierDisposition::NotReviewed,
        )
        .expect("incomplete campaigns still report");
        assert_eq!(report.claim_level, CampaignClaimLevel::Hypothesis);
        assert!(!report.benefit_claimed);
    }

    #[test]
    fn a_dropped_started_sample_is_not_reportable() {
        let mut layers = all_completed_layers();
        layers[3].retained_samples = 28;
        assert_eq!(
            build(
                CampaignClaimLevel::Hypothesis,
                false,
                layers,
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::DroppedStartedSample
        );
    }

    #[test]
    fn a_safety_failure_forces_a_non_claim() {
        let safety = CampaignSafetyAccounting {
            duplicate_external_effects: 1,
            ..CampaignSafetyAccounting::default()
        };
        assert_eq!(
            build(
                CampaignClaimLevel::Hypothesis,
                false,
                all_completed_layers(),
                safety,
                complete_cleanup(),
                VerifierDisposition::Affirmative,
            )
            .unwrap_err(),
            CampaignReportError::SafetyFailureBlocksClaim
        );
        let report = build(
            CampaignClaimLevel::NonClaim,
            false,
            all_completed_layers(),
            safety,
            complete_cleanup(),
            VerifierDisposition::Affirmative,
        )
        .expect("a safety failure is still reported in full");
        assert_eq!(report.safety.total_failures(), 1);
    }

    #[test]
    fn an_unverified_or_uncleaned_campaign_cannot_reach_tested_local() {
        assert_eq!(
            build(
                CampaignClaimLevel::TestedLocal,
                false,
                all_completed_layers(),
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::UnverifiedClaimPromotion
        );
        let leaked_cleanup = CampaignCleanupOutcome {
            orphan_processes: 1,
            ..complete_cleanup()
        };
        assert_eq!(
            build(
                CampaignClaimLevel::TestedLocal,
                false,
                all_completed_layers(),
                CampaignSafetyAccounting::default(),
                leaked_cleanup,
                VerifierDisposition::Affirmative,
            )
            .unwrap_err(),
            CampaignReportError::UnverifiedClaimPromotion
        );
        let owner_file_touched = CampaignCleanupOutcome {
            owner_source_file_untouched: false,
            ..complete_cleanup()
        };
        assert_eq!(
            build(
                CampaignClaimLevel::TestedLocal,
                false,
                all_completed_layers(),
                CampaignSafetyAccounting::default(),
                owner_file_touched,
                VerifierDisposition::Affirmative,
            )
            .unwrap_err(),
            CampaignReportError::UnverifiedClaimPromotion
        );
    }

    #[test]
    fn an_unrun_l5_cannot_produce_an_agent_benefit_claim() {
        let mut layers = all_completed_layers();
        layers[5] = not_run_layer(CampaignLayer::L5BenefitCampaign);
        assert_eq!(
            build(
                CampaignClaimLevel::Hypothesis,
                true,
                layers,
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::Affirmative,
            )
            .unwrap_err(),
            CampaignReportError::UnsupportedBenefitClaim
        );
    }

    #[test]
    fn a_layer_that_did_not_run_cannot_present_evidence() {
        let mut layers = all_completed_layers();
        layers[4].disposition = LayerDisposition::Failed;
        assert_eq!(
            build(
                CampaignClaimLevel::Hypothesis,
                false,
                layers,
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::EvidenceWithoutExecution
        );
    }

    #[test]
    fn a_completed_layer_with_no_samples_is_not_reportable() {
        let mut layers = all_completed_layers();
        layers[1].started_samples = 0;
        layers[1].retained_samples = 0;
        assert_eq!(
            build(
                CampaignClaimLevel::Hypothesis,
                false,
                layers,
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::EmptyCompletedLayer
        );
    }

    #[test]
    fn missing_layers_and_non_claims_fail_closed() {
        let mut layers = all_completed_layers();
        layers.remove(2);
        assert_eq!(
            build(
                CampaignClaimLevel::Hypothesis,
                false,
                layers,
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::IncompleteLayerCoverage
        );
        let mut report = build(
            CampaignClaimLevel::Hypothesis,
            false,
            all_completed_layers(),
            CampaignSafetyAccounting::default(),
            complete_cleanup(),
            VerifierDisposition::NotReviewed,
        )
        .expect("publishable report");
        report.non_claims.retain(|claim| *claim != "no-profile");
        assert_eq!(
            validate_campaign_evidence_report(&report).unwrap_err(),
            CampaignReportError::MissingNonClaim
        );
    }

    #[test]
    fn abbreviated_revision_or_missing_environment_fails_closed() {
        assert_eq!(
            build_campaign_evidence_report(
                CAMPAIGN_ID,
                "ad3c017",
                ENVIRONMENT,
                CampaignClaimLevel::Hypothesis,
                false,
                all_completed_layers(),
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::MissingCampaignIdentity
        );
        assert_eq!(
            build_campaign_evidence_report(
                CAMPAIGN_ID,
                REVISION,
                "  ",
                CampaignClaimLevel::Hypothesis,
                false,
                all_completed_layers(),
                CampaignSafetyAccounting::default(),
                complete_cleanup(),
                VerifierDisposition::NotReviewed,
            )
            .unwrap_err(),
            CampaignReportError::MissingCampaignIdentity
        );
    }
}
