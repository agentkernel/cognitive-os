//! Post-1.0 dynamic Tool ecosystem — private MVP (P5-T04 / B10).
//!
//! Dynamic discovery produces disabled candidates that require explicit
//! re-qualification before enable. Exposure is TaskContract-scoped and
//! health-gated. Quarantine blocks enable/exposure. Composite tools retain
//! child Intent/Effect evidence slots. Cache admits only pure-read,
//! version-bound operations. Nothing here grants CognitiveOS capability,
//! Intent/Effect authority, or Task completion.

use sha2::{Digest, Sha256};
use thiserror::Error;

/// Fixture dynamic Tool package id for P5-T04 / B10 qualification.
pub const DYNAMIC_TOOL_FIXTURE_PACKAGE_ID: &str = "fixture.dynamic.tool.catalog";

/// Fixture package schema pin (non-authoritative research ledger).
pub const DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN: &str = "dynamic-tool-package/0.1";

/// Digest-bound dynamic Tool package identity (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicToolPackageManifest {
    pub package_id: String,
    pub schema_version: String,
    pub package_digest: String,
    pub manifest_digest: String,
    pub authority_writer: bool,
}

/// Lifecycle state for a discovered dynamic Tool candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicToolLifecycleState {
    Discovered,
    Enabled,
    Disabled,
    Quarantined,
}

impl DynamicToolLifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Quarantined => "quarantined",
        }
    }
}

/// Discovered dynamic Tool candidate. Discovery never auto-enables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicToolCandidate {
    pub package_id: String,
    pub tool_id: String,
    pub descriptor_digest: String,
    pub state: DynamicToolLifecycleState,
    pub healthy: bool,
    pub requires_requalification: bool,
    pub enabled: bool,
}

/// TaskContract-scoped exposure plan for one scheduler round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicToolExposurePlan {
    pub task_ref: String,
    pub allowed_tool_ids: Vec<String>,
    pub exposed_tool_ids: Vec<String>,
    pub exposure_digest: String,
}

/// Child Intent/Effect evidence slot retained by a composite Tool plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeChildEvidenceSlot {
    pub child_operation_id: String,
    pub intent_digest: String,
    pub effect_digest: String,
}

/// Composite Tool plan that cannot hide unknown outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeToolPlan {
    pub composite_tool_id: String,
    pub children: Vec<CompositeChildEvidenceSlot>,
    pub plan_digest: String,
    pub hides_unknown_outcome: bool,
}

/// Pure-read, version-bound Tool cache entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicToolCacheEntry {
    pub tool_id: String,
    pub version_digest: String,
    pub result_digest: String,
    pub pure_read: bool,
}

/// Bounded Tool cache telemetry (schema token cost / utilization / hit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicToolCacheTelemetry {
    pub schema_token_cost: u64,
    pub result_utilization: u64,
    pub cache_hit: bool,
    pub telemetry_digest: String,
}

/// Unknown-outcome reconcile receipt bound to the original idempotency key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicToolReconcileReceipt {
    pub original_key: String,
    pub outcome_digest: String,
    pub blind_retry: bool,
}

/// Fixed-denominator B10 non-claim observation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct B10QualificationObservation {
    pub package_id: String,
    pub manifest_digest: String,
    pub claim_scope: &'static str,
    pub observations: Vec<&'static str>,
    pub report_digest: String,
}

/// Fail-closed dynamic Tool ecosystem errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DynamicToolEcosystemError {
    #[error("dynamic Tool package/manifest identity is missing required material")]
    MissingIdentity,
    #[error("dynamic Tool package schema does not match the pinned fixture version")]
    SchemaVersionMismatch,
    #[error("dynamic Tool package/manifest digest drifted from the admitted fixture")]
    ManifestDigestDrift,
    #[error("dynamic Tool discovery must not auto-enable candidates")]
    AutoEnableForbidden,
    #[error("dynamic Tool must not declare CognitiveOS authority-writer capability")]
    AuthorityWriterForbidden,
    #[error("dynamic Tool enable requires an explicit re-qualification digest")]
    RequalificationRequired,
    #[error("quarantined dynamic Tool cannot be enabled or exposed")]
    QuarantinedForbidden,
    #[error("TaskContract does not allow the requested dynamic Tool")]
    TaskContractDenied,
    #[error("unhealthy dynamic Tool cannot enter the exposure set")]
    UnhealthyForbidden,
    #[error("composite Tool must retain child Intent/Effect evidence slots")]
    CompositeEvidenceRequired,
    #[error("composite Tool must not hide unknown outcomes")]
    UnknownOutcomeHidden,
    #[error("Tool cache admits only pure-read, version-bound operations")]
    CacheMutationForbidden,
    #[error("unknown-outcome reconcile rejects blind retry without the original key")]
    BlindRetryForbidden,
    #[error("direct sandbox bypass of dynamic Tool mediation is forbidden")]
    DirectBypassForbidden,
    #[error("B10 qualification rejects Gate/authority-shaped claims")]
    AuthorityShapedClaimForbidden,
}

const NON_CLAIM: &str = "non-claim";

/// Fixed B10 MVP observation names (ADR-0050 denominator).
pub const B10_REQUIRED_OBSERVATIONS: &[&str] = &[
    "dynamic_package_identity_bound",
    "discovery_disabled_no_auto_enable",
    "task_contract_scoped_exposure",
    "enable_requires_requalification",
    "disable_removes_exposure",
    "quarantine_blocks_enable",
    "package_manifest_drift_fail_closed",
    "reconcile_unknown_outcome_original_key",
    "composite_retains_child_intent_effect",
    "pure_read_cache_only",
    "sandbox_bypass_rejected",
];

/// Bind a fixture dynamic Tool package/manifest identity.
pub fn bind_dynamic_tool_package(
    package_id: &str,
    schema_version: &str,
    package_digest: &str,
) -> Result<DynamicToolPackageManifest, DynamicToolEcosystemError> {
    if package_id.trim().is_empty() || package_digest.trim().is_empty() {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    if schema_version != DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN {
        return Err(DynamicToolEcosystemError::SchemaVersionMismatch);
    }
    if package_id != DYNAMIC_TOOL_FIXTURE_PACKAGE_ID {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }

    let mut hasher = Sha256::new();
    hasher.update(package_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(schema_version.as_bytes());
    hasher.update(b"\0");
    hasher.update(package_digest.as_bytes());
    Ok(DynamicToolPackageManifest {
        package_id: package_id.to_owned(),
        schema_version: schema_version.to_owned(),
        package_digest: package_digest.to_owned(),
        manifest_digest: format!("{:x}", hasher.finalize()),
        authority_writer: false,
    })
}

/// Verify an observed package digest against the admitted manifest.
pub fn verify_dynamic_tool_package_current(
    manifest: &DynamicToolPackageManifest,
    observed_schema_version: &str,
    observed_manifest_digest: &str,
) -> Result<(), DynamicToolEcosystemError> {
    if manifest.package_id != DYNAMIC_TOOL_FIXTURE_PACKAGE_ID
        || manifest.manifest_digest.trim().is_empty()
    {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    if observed_schema_version != DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN
        || observed_schema_version != manifest.schema_version
    {
        return Err(DynamicToolEcosystemError::SchemaVersionMismatch);
    }
    if observed_manifest_digest != manifest.manifest_digest {
        return Err(DynamicToolEcosystemError::ManifestDigestDrift);
    }
    Ok(())
}

/// Discover a dynamic Tool candidate. Discovery never auto-enables.
pub fn discover_dynamic_tool_candidate(
    manifest: &DynamicToolPackageManifest,
    tool_id: &str,
    descriptor_digest: &str,
    auto_enable: bool,
    declares_authority_writer: bool,
) -> Result<DynamicToolCandidate, DynamicToolEcosystemError> {
    if manifest.package_id != DYNAMIC_TOOL_FIXTURE_PACKAGE_ID
        || manifest.authority_writer
        || tool_id.trim().is_empty()
        || descriptor_digest.trim().is_empty()
    {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    if declares_authority_writer {
        return Err(DynamicToolEcosystemError::AuthorityWriterForbidden);
    }
    if auto_enable {
        return Err(DynamicToolEcosystemError::AutoEnableForbidden);
    }
    Ok(DynamicToolCandidate {
        package_id: manifest.package_id.clone(),
        tool_id: tool_id.to_owned(),
        descriptor_digest: descriptor_digest.to_owned(),
        state: DynamicToolLifecycleState::Discovered,
        healthy: true,
        requires_requalification: true,
        enabled: false,
    })
}

/// Explicitly enable a discovered/disabled candidate after re-qualification.
pub fn enable_dynamic_tool(
    candidate: &DynamicToolCandidate,
    requalification_digest: &str,
) -> Result<DynamicToolCandidate, DynamicToolEcosystemError> {
    if candidate.state == DynamicToolLifecycleState::Quarantined {
        return Err(DynamicToolEcosystemError::QuarantinedForbidden);
    }
    if requalification_digest.trim().is_empty() {
        return Err(DynamicToolEcosystemError::RequalificationRequired);
    }
    if !candidate.requires_requalification && candidate.enabled {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    Ok(DynamicToolCandidate {
        state: DynamicToolLifecycleState::Enabled,
        requires_requalification: false,
        enabled: true,
        ..candidate.clone()
    })
}

/// Disable an enabled candidate; it leaves the exposure set.
pub fn disable_dynamic_tool(
    candidate: &DynamicToolCandidate,
) -> Result<DynamicToolCandidate, DynamicToolEcosystemError> {
    if candidate.state == DynamicToolLifecycleState::Quarantined {
        return Err(DynamicToolEcosystemError::QuarantinedForbidden);
    }
    Ok(DynamicToolCandidate {
        state: DynamicToolLifecycleState::Disabled,
        requires_requalification: true,
        enabled: false,
        ..candidate.clone()
    })
}

/// Quarantine a candidate; enable and exposure remain blocked.
pub fn quarantine_dynamic_tool(
    candidate: &DynamicToolCandidate,
) -> Result<DynamicToolCandidate, DynamicToolEcosystemError> {
    Ok(DynamicToolCandidate {
        state: DynamicToolLifecycleState::Quarantined,
        requires_requalification: true,
        enabled: false,
        healthy: false,
        ..candidate.clone()
    })
}

/// Expose only TaskContract-allowed, enabled, healthy tools for one round.
pub fn plan_task_contract_exposure(
    task_ref: &str,
    allowed_tool_ids: &[&str],
    candidates: &[DynamicToolCandidate],
) -> Result<DynamicToolExposurePlan, DynamicToolEcosystemError> {
    if task_ref.trim().is_empty() {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    let allowed: Vec<String> = allowed_tool_ids.iter().map(|id| (*id).to_owned()).collect();
    let mut exposed = Vec::new();
    for candidate in candidates {
        if !candidate.enabled || !candidate.healthy {
            continue;
        }
        if candidate.state == DynamicToolLifecycleState::Quarantined {
            return Err(DynamicToolEcosystemError::QuarantinedForbidden);
        }
        if !allowed.iter().any(|id| id == &candidate.tool_id) {
            return Err(DynamicToolEcosystemError::TaskContractDenied);
        }
        if !candidate.healthy {
            return Err(DynamicToolEcosystemError::UnhealthyForbidden);
        }
        exposed.push(candidate.tool_id.clone());
    }

    let mut hasher = Sha256::new();
    hasher.update(task_ref.as_bytes());
    hasher.update(b"\0");
    for id in &allowed {
        hasher.update(id.as_bytes());
        hasher.update(b"\0");
    }
    for id in &exposed {
        hasher.update(id.as_bytes());
        hasher.update(b"\0");
    }
    Ok(DynamicToolExposurePlan {
        task_ref: task_ref.to_owned(),
        allowed_tool_ids: allowed,
        exposed_tool_ids: exposed,
        exposure_digest: format!("{:x}", hasher.finalize()),
    })
}

/// Build a composite Tool plan that retains child Intent/Effect evidence.
pub fn plan_composite_tool(
    composite_tool_id: &str,
    children: &[CompositeChildEvidenceSlot],
    hide_unknown_outcome: bool,
) -> Result<CompositeToolPlan, DynamicToolEcosystemError> {
    if composite_tool_id.trim().is_empty() || children.is_empty() {
        return Err(DynamicToolEcosystemError::CompositeEvidenceRequired);
    }
    if hide_unknown_outcome {
        return Err(DynamicToolEcosystemError::UnknownOutcomeHidden);
    }
    for child in children {
        if child.child_operation_id.trim().is_empty()
            || child.intent_digest.trim().is_empty()
            || child.effect_digest.trim().is_empty()
        {
            return Err(DynamicToolEcosystemError::CompositeEvidenceRequired);
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(composite_tool_id.as_bytes());
    hasher.update(b"\0");
    for child in children {
        hasher.update(child.child_operation_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(child.intent_digest.as_bytes());
        hasher.update(b"\0");
        hasher.update(child.effect_digest.as_bytes());
        hasher.update(b"\0");
    }
    Ok(CompositeToolPlan {
        composite_tool_id: composite_tool_id.to_owned(),
        children: children.to_vec(),
        plan_digest: format!("{:x}", hasher.finalize()),
        hides_unknown_outcome: false,
    })
}

/// Admit a pure-read, version-bound cache lookup and emit telemetry.
pub fn lookup_pure_read_cache(
    entry: &DynamicToolCacheEntry,
    schema_token_cost: u64,
    result_utilization: u64,
    cache_hit: bool,
) -> Result<DynamicToolCacheTelemetry, DynamicToolEcosystemError> {
    if entry.tool_id.trim().is_empty()
        || entry.version_digest.trim().is_empty()
        || entry.result_digest.trim().is_empty()
    {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    if !entry.pure_read {
        return Err(DynamicToolEcosystemError::CacheMutationForbidden);
    }

    let mut hasher = Sha256::new();
    hasher.update(entry.tool_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(entry.version_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(entry.result_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(schema_token_cost.to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(result_utilization.to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(if cache_hit {
        "hit".as_bytes()
    } else {
        "miss".as_bytes()
    });
    Ok(DynamicToolCacheTelemetry {
        schema_token_cost,
        result_utilization,
        cache_hit,
        telemetry_digest: format!("{:x}", hasher.finalize()),
    })
}

/// Reconcile an unknown outcome by the original idempotency key only.
pub fn reconcile_dynamic_tool_unknown_outcome(
    original_key: &str,
    observed_outcome_digest: &str,
    blind_retry: bool,
) -> Result<DynamicToolReconcileReceipt, DynamicToolEcosystemError> {
    if original_key.trim().is_empty() || observed_outcome_digest.trim().is_empty() {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    if blind_retry {
        return Err(DynamicToolEcosystemError::BlindRetryForbidden);
    }
    Ok(DynamicToolReconcileReceipt {
        original_key: original_key.to_owned(),
        outcome_digest: observed_outcome_digest.to_owned(),
        blind_retry: false,
    })
}

/// Mediated dynamic Tool access; unmediated paths fail closed.
pub fn mediate_dynamic_tool_access(
    package_id: &str,
    mediated: bool,
) -> Result<(), DynamicToolEcosystemError> {
    if package_id != DYNAMIC_TOOL_FIXTURE_PACKAGE_ID {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    if !mediated {
        return Err(DynamicToolEcosystemError::DirectBypassForbidden);
    }
    Ok(())
}

/// Build a fixed-denominator non-claim B10 qualification report.
pub fn build_b10_qualification_report(
    manifest: &DynamicToolPackageManifest,
    observations: &[&str],
    authority_claim_labels: &[&str],
) -> Result<B10QualificationObservation, DynamicToolEcosystemError> {
    if manifest.package_id != DYNAMIC_TOOL_FIXTURE_PACKAGE_ID
        || manifest.manifest_digest.trim().is_empty()
        || manifest.authority_writer
    {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }
    for label in authority_claim_labels {
        let normalized = label.trim().to_ascii_lowercase();
        if matches!(
            normalized.as_str(),
            "gate" | "release" | "profile" | "b10" | "pass" | "passed" | "gmvp-linux"
        ) {
            return Err(DynamicToolEcosystemError::AuthorityShapedClaimForbidden);
        }
    }
    let mut sorted_required: Vec<&str> = B10_REQUIRED_OBSERVATIONS.to_vec();
    sorted_required.sort_unstable();
    let mut sorted_actual: Vec<&str> = observations.to_vec();
    sorted_actual.sort_unstable();
    if sorted_actual != sorted_required {
        return Err(DynamicToolEcosystemError::MissingIdentity);
    }

    let mut hasher = Sha256::new();
    hasher.update(manifest.package_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(manifest.manifest_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(NON_CLAIM.as_bytes());
    for observation in sorted_required.iter() {
        hasher.update(observation.as_bytes());
        hasher.update(b"\0");
    }
    Ok(B10QualificationObservation {
        package_id: manifest.package_id.clone(),
        manifest_digest: manifest.manifest_digest.clone(),
        claim_scope: NON_CLAIM,
        observations: B10_REQUIRED_OBSERVATIONS.to_vec(),
        report_digest: format!("{:x}", hasher.finalize()),
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn fixture_digest() -> String {
        format!("sha256:{}", "b".repeat(64))
    }

    fn bound_manifest() -> DynamicToolPackageManifest {
        bind_dynamic_tool_package(
            DYNAMIC_TOOL_FIXTURE_PACKAGE_ID,
            DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN,
            &fixture_digest(),
        )
        .expect("bind")
    }

    fn discovered(tool_id: &str) -> DynamicToolCandidate {
        discover_dynamic_tool_candidate(
            &bound_manifest(),
            tool_id,
            "sha256:descriptor",
            false,
            false,
        )
        .expect("discover")
    }

    #[test]
    fn binds_package_and_discovers_disabled_candidate() {
        let manifest = bound_manifest();
        assert_eq!(manifest.package_id, DYNAMIC_TOOL_FIXTURE_PACKAGE_ID);
        assert!(!manifest.authority_writer);
        assert_eq!(manifest.manifest_digest.len(), 64);

        let candidate = discovered("tool.read");
        assert!(!candidate.enabled);
        assert!(candidate.requires_requalification);
        assert_eq!(candidate.state, DynamicToolLifecycleState::Discovered);
    }

    #[test]
    fn rejects_identity_schema_auto_enable_and_authority_writer() {
        assert_eq!(
            bind_dynamic_tool_package(
                "other.pkg",
                DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN,
                &fixture_digest()
            )
            .unwrap_err(),
            DynamicToolEcosystemError::MissingIdentity
        );
        assert_eq!(
            bind_dynamic_tool_package(
                DYNAMIC_TOOL_FIXTURE_PACKAGE_ID,
                "other/0.1",
                &fixture_digest()
            )
            .unwrap_err(),
            DynamicToolEcosystemError::SchemaVersionMismatch
        );
        let manifest = bound_manifest();
        assert_eq!(
            discover_dynamic_tool_candidate(&manifest, "tool.read", "sha256:d", true, false)
                .unwrap_err(),
            DynamicToolEcosystemError::AutoEnableForbidden
        );
        assert_eq!(
            discover_dynamic_tool_candidate(&manifest, "tool.read", "sha256:d", false, true)
                .unwrap_err(),
            DynamicToolEcosystemError::AuthorityWriterForbidden
        );
    }

    #[test]
    fn enable_disable_quarantine_and_task_contract_exposure() {
        let discovered = discovered("tool.read");
        assert_eq!(
            enable_dynamic_tool(&discovered, "  ").unwrap_err(),
            DynamicToolEcosystemError::RequalificationRequired
        );
        let enabled = enable_dynamic_tool(&discovered, "sha256:requal").expect("enable");
        assert!(enabled.enabled);
        assert_eq!(enabled.state, DynamicToolLifecycleState::Enabled);

        let plan = plan_task_contract_exposure("task-1", &["tool.read"], std::slice::from_ref(&enabled))
            .expect("expose");
        assert_eq!(plan.exposed_tool_ids, vec!["tool.read".to_owned()]);
        assert_eq!(plan.exposure_digest.len(), 64);

        assert_eq!(
            plan_task_contract_exposure("task-1", &["tool.other"], std::slice::from_ref(&enabled))
                .unwrap_err(),
            DynamicToolEcosystemError::TaskContractDenied
        );

        let disabled = disable_dynamic_tool(&enabled).expect("disable");
        assert!(!disabled.enabled);
        let empty =
            plan_task_contract_exposure("task-1", &["tool.read"], std::slice::from_ref(&disabled))
                .expect("no expose");
        assert!(empty.exposed_tool_ids.is_empty());

        let quarantined = quarantine_dynamic_tool(&disabled).expect("quarantine");
        assert_eq!(
            enable_dynamic_tool(&quarantined, "sha256:requal").unwrap_err(),
            DynamicToolEcosystemError::QuarantinedForbidden
        );
    }

    #[test]
    fn reject_manifest_drift_composite_cache_reconcile_and_bypass() {
        let manifest = bound_manifest();
        verify_dynamic_tool_package_current(
            &manifest,
            DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN,
            &manifest.manifest_digest,
        )
        .expect("current");
        assert_eq!(
            verify_dynamic_tool_package_current(
                &manifest,
                DYNAMIC_TOOL_PACKAGE_SCHEMA_PIN,
                "sha256:drifted",
            )
            .unwrap_err(),
            DynamicToolEcosystemError::ManifestDigestDrift
        );

        let child = CompositeChildEvidenceSlot {
            child_operation_id: "child.1".to_owned(),
            intent_digest: "sha256:intent".to_owned(),
            effect_digest: "sha256:effect".to_owned(),
        };
        let composite = plan_composite_tool("composite.1", &[child], false).expect("composite");
        assert!(!composite.hides_unknown_outcome);
        assert_eq!(
            plan_composite_tool("composite.1", &[], false).unwrap_err(),
            DynamicToolEcosystemError::CompositeEvidenceRequired
        );
        assert_eq!(
            plan_composite_tool(
                "composite.1",
                &[CompositeChildEvidenceSlot {
                    child_operation_id: "child.1".to_owned(),
                    intent_digest: "sha256:intent".to_owned(),
                    effect_digest: "sha256:effect".to_owned(),
                }],
                true,
            )
            .unwrap_err(),
            DynamicToolEcosystemError::UnknownOutcomeHidden
        );

        let pure = DynamicToolCacheEntry {
            tool_id: "tool.read".to_owned(),
            version_digest: "sha256:v1".to_owned(),
            result_digest: "sha256:r1".to_owned(),
            pure_read: true,
        };
        let telemetry = lookup_pure_read_cache(&pure, 12, 3, true).expect("cache");
        assert!(telemetry.cache_hit);
        assert_eq!(telemetry.schema_token_cost, 12);
        let mutating = DynamicToolCacheEntry {
            pure_read: false,
            ..pure
        };
        assert_eq!(
            lookup_pure_read_cache(&mutating, 1, 1, false).unwrap_err(),
            DynamicToolEcosystemError::CacheMutationForbidden
        );

        let receipt = reconcile_dynamic_tool_unknown_outcome("key-1", "sha256:out", false)
            .expect("reconcile");
        assert!(!receipt.blind_retry);
        assert_eq!(
            reconcile_dynamic_tool_unknown_outcome("key-1", "sha256:out", true).unwrap_err(),
            DynamicToolEcosystemError::BlindRetryForbidden
        );

        mediate_dynamic_tool_access(DYNAMIC_TOOL_FIXTURE_PACKAGE_ID, true).expect("mediated");
        assert_eq!(
            mediate_dynamic_tool_access(DYNAMIC_TOOL_FIXTURE_PACKAGE_ID, false).unwrap_err(),
            DynamicToolEcosystemError::DirectBypassForbidden
        );

        let report = build_b10_qualification_report(&manifest, B10_REQUIRED_OBSERVATIONS, &[])
            .expect("report");
        assert_eq!(report.claim_scope, NON_CLAIM);
        assert_eq!(report.observations.len(), B10_REQUIRED_OBSERVATIONS.len());
        assert_eq!(
            build_b10_qualification_report(&manifest, B10_REQUIRED_OBSERVATIONS, &["B10"])
                .unwrap_err(),
            DynamicToolEcosystemError::AuthorityShapedClaimForbidden
        );
    }
}
