//! First non-Pi Agent fixture qualification — private MVP (P8-T03/D01).
//!
//! Selects OpenAI Codex as the first non-Pi CLI agent from the documented
//! deferred-agent set and binds a fixture package identity that is independent
//! of managed Pi evidence. Registration reuses the AKP adapter contract and
//! never transfers Pi/B09 claims.

use crate::agent_adapter_manifest::{
    AdapterCapabilityDeclaration, AdapterLifecycleHandle, AdapterTransportProfile,
    AgentAdapterError, RegisteredAgentAdapter, activate_adapter_lifecycle,
    open_registered_adapter_lifecycle, pause_adapter_lifecycle, register_agent_adapter,
    stop_adapter_lifecycle,
};
use crate::channel_binding::AuthorityChannel;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Documented first non-Pi agent selection for P8-T03 MVP.
pub const FIRST_NON_PI_AGENT_ID: &str = "openai.codex.cli";
const PI_AGENT_ID: &str = "earendil.pi.coding-agent";

/// Fixture package identity for Codex qualification (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPiAgentPackageIdentity {
    pub agent_id: String,
    pub package_digest: String,
    pub protocol: AdapterTransportProfile,
    pub independent_of_pi: bool,
}

/// Fail-closed non-Pi qualification errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NonPiAgentError {
    #[error("non-Pi package identity is missing required material")]
    MissingIdentity,
    #[error("non-Pi qualification cannot reuse the managed Pi agent identity")]
    PiIdentityForbidden,
    #[error("non-Pi qualification cannot inherit Pi or B09 evidence")]
    PiEvidenceTransferForbidden,
    #[error("non-Pi qualification rejects Gate/authority-shaped claims")]
    AuthorityShapedClaimForbidden,
    #[error("non-Pi adapter registration failed: {0}")]
    Adapter(#[from] AgentAdapterError),
}

/// Bind the selected Codex fixture package identity (independent of Pi).
pub fn bind_codex_fixture_package_identity(
    package_digest: &str,
) -> Result<NonPiAgentPackageIdentity, NonPiAgentError> {
    if package_digest.trim().is_empty() {
        return Err(NonPiAgentError::MissingIdentity);
    }
    Ok(NonPiAgentPackageIdentity {
        agent_id: FIRST_NON_PI_AGENT_ID.to_owned(),
        package_digest: package_digest.to_owned(),
        protocol: AdapterTransportProfile::AkpHttpJsonSse,
        independent_of_pi: true,
    })
}

/// Register the Codex fixture as an AKP adapter without inheriting Pi evidence.
pub fn register_codex_fixture_adapter(
    package: &NonPiAgentPackageIdentity,
    inherits_pi_evidence: bool,
    declares_public_listener: bool,
    declares_authority_writer: bool,
) -> Result<RegisteredAgentAdapter, NonPiAgentError> {
    if package.agent_id == PI_AGENT_ID {
        return Err(NonPiAgentError::PiIdentityForbidden);
    }
    if package.agent_id != FIRST_NON_PI_AGENT_ID {
        return Err(NonPiAgentError::MissingIdentity);
    }
    if inherits_pi_evidence || !package.independent_of_pi {
        return Err(NonPiAgentError::PiEvidenceTransferForbidden);
    }

    let declaration = AdapterCapabilityDeclaration {
        adapter_id: package.agent_id.clone(),
        protocol: package.protocol,
        candidate_only: true,
        public_listener: declares_public_listener,
        authority_writer: declares_authority_writer,
        discovery_card_digest: bind_discovery_card_digest(package),
    };
    Ok(register_agent_adapter(&declaration)?)
}

/// Open Codex fixture lifecycle (Registered, epoch 0).
pub fn open_codex_fixture_lifecycle(
    registered: &RegisteredAgentAdapter,
) -> Result<AdapterLifecycleHandle, NonPiAgentError> {
    if registered.adapter_id != FIRST_NON_PI_AGENT_ID {
        return Err(NonPiAgentError::MissingIdentity);
    }
    Ok(open_registered_adapter_lifecycle(registered)?)
}

/// Activate/pause/stop Codex fixture lifecycle on the management channel only.
pub fn activate_codex_fixture_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, NonPiAgentError> {
    Ok(activate_adapter_lifecycle(
        handle,
        expected_declaration_digest,
        expected_epoch,
        channel,
    )?)
}

pub fn pause_codex_fixture_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, NonPiAgentError> {
    Ok(pause_adapter_lifecycle(
        handle,
        expected_declaration_digest,
        expected_epoch,
        channel,
    )?)
}

pub fn stop_codex_fixture_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, NonPiAgentError> {
    Ok(stop_adapter_lifecycle(
        handle,
        expected_declaration_digest,
        expected_epoch,
        channel,
    )?)
}

fn bind_discovery_card_digest(package: &NonPiAgentPackageIdentity) -> String {
    let mut hasher = Sha256::new();
    hasher.update(package.agent_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(package.package_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(b"independent_of_pi");
    format!("{:x}", hasher.finalize())
}

const NON_CLAIM: &str = "non-claim";
const REQUIRED_OBSERVATIONS: &[&str] = &[
    "package_identity_bound",
    "akp_registration_independent_of_pi",
    "lifecycle_management_channel_only",
    "no_pi_evidence_transfer",
];

/// Fixed-denominator non-Pi qualification observation (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPiQualificationObservation {
    pub agent_id: String,
    pub package_digest: String,
    pub declaration_digest: String,
    pub claim_scope: &'static str,
    pub observations: Vec<&'static str>,
    pub report_digest: String,
}

/// Build a B09-mode non-claim qualification report for the Codex fixture.
pub fn build_codex_qualification_report(
    package: &NonPiAgentPackageIdentity,
    registered: &RegisteredAgentAdapter,
    observations: &[&str],
    authority_claim_labels: &[&str],
) -> Result<NonPiQualificationObservation, NonPiAgentError> {
    if package.agent_id != FIRST_NON_PI_AGENT_ID
        || registered.adapter_id != FIRST_NON_PI_AGENT_ID
        || package.package_digest.trim().is_empty()
        || registered.declaration_digest.trim().is_empty()
    {
        return Err(NonPiAgentError::MissingIdentity);
    }
    if !package.independent_of_pi {
        return Err(NonPiAgentError::PiEvidenceTransferForbidden);
    }
    for label in authority_claim_labels {
        let normalized = label.trim().to_ascii_lowercase();
        if matches!(
            normalized.as_str(),
            "gate" | "release" | "profile" | "b09" | "pass" | "passed"
        ) {
            return Err(NonPiAgentError::AuthorityShapedClaimForbidden);
        }
    }
    let mut sorted_required: Vec<&str> = REQUIRED_OBSERVATIONS.to_vec();
    sorted_required.sort_unstable();
    let mut sorted_actual: Vec<&str> = observations.to_vec();
    sorted_actual.sort_unstable();
    if sorted_actual != sorted_required {
        return Err(NonPiAgentError::MissingIdentity);
    }

    let mut hasher = Sha256::new();
    hasher.update(package.agent_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(package.package_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(registered.declaration_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(NON_CLAIM.as_bytes());
    for observation in sorted_required.iter() {
        hasher.update(observation.as_bytes());
        hasher.update(b"\0");
    }
    Ok(NonPiQualificationObservation {
        agent_id: package.agent_id.clone(),
        package_digest: package.package_digest.clone(),
        declaration_digest: registered.declaration_digest.clone(),
        claim_scope: NON_CLAIM,
        observations: REQUIRED_OBSERVATIONS.to_vec(),
        report_digest: format!("{:x}", hasher.finalize()),
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::agent_adapter_manifest::AdapterLifecycleState;

    #[test]
    fn binds_codex_fixture_independent_of_pi() {
        let package = bind_codex_fixture_package_identity(&format!("sha256:{}", "c".repeat(64)))
            .expect("bind");
        assert_eq!(package.agent_id, FIRST_NON_PI_AGENT_ID);
        assert!(package.independent_of_pi);
        let registered =
            register_codex_fixture_adapter(&package, false, false, false).expect("register");
        assert_eq!(registered.adapter_id, FIRST_NON_PI_AGENT_ID);
        assert!(registered.candidate_only);
    }

    #[test]
    fn rejects_pi_evidence_transfer_and_authority_surface() {
        let package = bind_codex_fixture_package_identity(&format!("sha256:{}", "c".repeat(64)))
            .expect("bind");
        assert_eq!(
            register_codex_fixture_adapter(&package, true, false, false).unwrap_err(),
            NonPiAgentError::PiEvidenceTransferForbidden
        );
        assert!(matches!(
            register_codex_fixture_adapter(&package, false, true, false).unwrap_err(),
            NonPiAgentError::Adapter(AgentAdapterError::PublicListenerForbidden)
        ));
        assert!(matches!(
            register_codex_fixture_adapter(&package, false, false, true).unwrap_err(),
            NonPiAgentError::Adapter(AgentAdapterError::AuthorityWriterForbidden)
        ));
        assert_eq!(
            bind_codex_fixture_package_identity("  ").unwrap_err(),
            NonPiAgentError::MissingIdentity
        );
    }

    #[test]
    fn activates_pauses_and_stops_on_management_channel_only() {
        let package = bind_codex_fixture_package_identity(&format!("sha256:{}", "c".repeat(64)))
            .expect("bind");
        let registered =
            register_codex_fixture_adapter(&package, false, false, false).expect("register");
        let opened = open_codex_fixture_lifecycle(&registered).expect("open");
        assert_eq!(opened.state, AdapterLifecycleState::Registered);

        let active = activate_codex_fixture_lifecycle(
            &opened,
            &opened.declaration_digest,
            opened.fencing_epoch,
            AuthorityChannel::Management,
        )
        .expect("activate");
        assert_eq!(active.state, AdapterLifecycleState::Active);

        assert!(matches!(
            activate_codex_fixture_lifecycle(
                &opened,
                &opened.declaration_digest,
                opened.fencing_epoch,
                AuthorityChannel::Task,
            )
            .unwrap_err(),
            NonPiAgentError::Adapter(AgentAdapterError::ChannelIsolationViolation)
        ));

        let paused = pause_codex_fixture_lifecycle(
            &active,
            &active.declaration_digest,
            active.fencing_epoch,
            AuthorityChannel::Management,
        )
        .expect("pause");
        assert_eq!(paused.state, AdapterLifecycleState::Paused);

        let stopped = stop_codex_fixture_lifecycle(
            &paused,
            &paused.declaration_digest,
            paused.fencing_epoch,
            AuthorityChannel::Management,
        )
        .expect("stop");
        assert_eq!(stopped.state, AdapterLifecycleState::Stopped);
    }

    #[test]
    fn builds_non_claim_qualification_report_and_rejects_authority_claims() {
        let package = bind_codex_fixture_package_identity(&format!("sha256:{}", "c".repeat(64)))
            .expect("bind");
        let registered =
            register_codex_fixture_adapter(&package, false, false, false).expect("register");
        let report =
            build_codex_qualification_report(&package, &registered, REQUIRED_OBSERVATIONS, &[])
                .expect("report");
        assert_eq!(report.claim_scope, NON_CLAIM);
        assert_eq!(report.observations.len(), 4);
        assert_eq!(report.report_digest.len(), 64);

        assert_eq!(
            build_codex_qualification_report(
                &package,
                &registered,
                REQUIRED_OBSERVATIONS,
                &["B09"],
            )
            .unwrap_err(),
            NonPiAgentError::AuthorityShapedClaimForbidden
        );
        assert_eq!(
            build_codex_qualification_report(&package, &registered, &["incomplete"], &[])
                .unwrap_err(),
            NonPiAgentError::MissingIdentity
        );
    }
}
