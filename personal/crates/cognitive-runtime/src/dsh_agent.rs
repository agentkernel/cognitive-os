//! DeepSeek Harness (`dsh`) adapter registration — private MVP (P8-T09).
//!
//! Binds a candidate-only AKP adapter identity independent of managed Pi
//! evidence. Registration reuses the Universal Agent Adapter Contract and
//! never creates a public listener or authority writer.

use crate::agent_adapter_manifest::{
    AdapterCapabilityDeclaration, AdapterLifecycleHandle, AdapterTransportProfile,
    AgentAdapterError, RegisteredAgentAdapter, activate_adapter_lifecycle,
    open_registered_adapter_lifecycle, pause_adapter_lifecycle, register_agent_adapter,
    stop_adapter_lifecycle,
};
use crate::channel_binding::AuthorityChannel;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Daemon-private adapter identity for the dsh AKP bridge.
pub const DSH_ADAPTER_ID: &str = "deepseek.dsh.akp";
const PI_AGENT_ID: &str = "earendil.pi.coding-agent";
/// Exact DeepSeek Harness source pin (git object).
pub const DSH_PACKAGE_REVISION: &str = "528c682e061696f5a160f363f236ecbf53cbd006";

/// Fixture package identity for dsh registration (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshAgentPackageIdentity {
    pub agent_id: String,
    pub package_digest: String,
    pub protocol: AdapterTransportProfile,
    pub independent_of_pi: bool,
}

/// Fail-closed dsh adapter errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DshAgentError {
    #[error("dsh package identity is missing required material")]
    MissingIdentity,
    #[error("dsh qualification cannot reuse the managed Pi agent identity")]
    PiIdentityForbidden,
    #[error("dsh qualification cannot inherit Pi or B09 evidence")]
    PiEvidenceTransferForbidden,
    #[error("dsh qualification rejects Gate/authority-shaped claims")]
    AuthorityShapedClaimForbidden,
    #[error("dsh adapter registration failed: {0}")]
    Adapter(#[from] AgentAdapterError),
}

/// Bind the dsh package identity (independent of Pi).
pub fn bind_dsh_package_identity(
    package_digest: &str,
) -> Result<DshAgentPackageIdentity, DshAgentError> {
    if package_digest.trim().is_empty() {
        return Err(DshAgentError::MissingIdentity);
    }
    Ok(DshAgentPackageIdentity {
        agent_id: DSH_ADAPTER_ID.to_owned(),
        package_digest: package_digest.to_owned(),
        protocol: AdapterTransportProfile::AkpHttpJsonSse,
        independent_of_pi: true,
    })
}

/// Register the dsh adapter without inheriting Pi evidence.
pub fn register_dsh_adapter(
    package: &DshAgentPackageIdentity,
    inherits_pi_evidence: bool,
    declares_public_listener: bool,
    declares_authority_writer: bool,
) -> Result<RegisteredAgentAdapter, DshAgentError> {
    if package.agent_id == PI_AGENT_ID {
        return Err(DshAgentError::PiIdentityForbidden);
    }
    if package.agent_id != DSH_ADAPTER_ID {
        return Err(DshAgentError::MissingIdentity);
    }
    if inherits_pi_evidence || !package.independent_of_pi {
        return Err(DshAgentError::PiEvidenceTransferForbidden);
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

pub fn open_dsh_lifecycle(
    registered: &RegisteredAgentAdapter,
) -> Result<AdapterLifecycleHandle, DshAgentError> {
    if registered.adapter_id != DSH_ADAPTER_ID {
        return Err(DshAgentError::MissingIdentity);
    }
    Ok(open_registered_adapter_lifecycle(registered)?)
}

pub fn activate_dsh_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, DshAgentError> {
    Ok(activate_adapter_lifecycle(
        handle,
        expected_declaration_digest,
        expected_epoch,
        channel,
    )?)
}

pub fn pause_dsh_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, DshAgentError> {
    Ok(pause_adapter_lifecycle(
        handle,
        expected_declaration_digest,
        expected_epoch,
        channel,
    )?)
}

pub fn stop_dsh_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, DshAgentError> {
    Ok(stop_adapter_lifecycle(
        handle,
        expected_declaration_digest,
        expected_epoch,
        channel,
    )?)
}

fn bind_discovery_card_digest(package: &DshAgentPackageIdentity) -> String {
    let mut hasher = Sha256::new();
    hasher.update(package.agent_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(package.package_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(b"independent_of_pi");
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::agent_adapter_manifest::AdapterLifecycleState;

    #[test]
    fn binds_dsh_adapter_independent_of_pi() {
        let package = bind_dsh_package_identity(DSH_PACKAGE_REVISION).expect("bind");
        assert_eq!(package.agent_id, DSH_ADAPTER_ID);
        assert!(package.independent_of_pi);
        let registered = register_dsh_adapter(&package, false, false, false).expect("register");
        assert_eq!(registered.adapter_id, DSH_ADAPTER_ID);
        assert!(registered.candidate_only);
    }

    #[test]
    fn rejects_pi_evidence_transfer_and_authority_surface() {
        let package = bind_dsh_package_identity(DSH_PACKAGE_REVISION).expect("bind");
        assert_eq!(
            register_dsh_adapter(&package, true, false, false).unwrap_err(),
            DshAgentError::PiEvidenceTransferForbidden
        );
        assert!(matches!(
            register_dsh_adapter(&package, false, true, false).unwrap_err(),
            DshAgentError::Adapter(AgentAdapterError::PublicListenerForbidden)
        ));
        assert!(matches!(
            register_dsh_adapter(&package, false, false, true).unwrap_err(),
            DshAgentError::Adapter(AgentAdapterError::AuthorityWriterForbidden)
        ));
        assert_eq!(
            bind_dsh_package_identity("  ").unwrap_err(),
            DshAgentError::MissingIdentity
        );
    }

    #[test]
    fn activates_on_management_channel_only() {
        let package = bind_dsh_package_identity(DSH_PACKAGE_REVISION).expect("bind");
        let registered = register_dsh_adapter(&package, false, false, false).expect("register");
        let opened = open_dsh_lifecycle(&registered).expect("open");
        assert_eq!(opened.state, AdapterLifecycleState::Registered);

        let active = activate_dsh_lifecycle(
            &opened,
            &opened.declaration_digest,
            opened.fencing_epoch,
            AuthorityChannel::Management,
        )
        .expect("activate");
        assert_eq!(active.state, AdapterLifecycleState::Active);

        assert!(matches!(
            activate_dsh_lifecycle(
                &opened,
                &opened.declaration_digest,
                opened.fencing_epoch,
                AuthorityChannel::Task,
            )
            .unwrap_err(),
            DshAgentError::Adapter(AgentAdapterError::ChannelIsolationViolation)
        ));

        let stopped = stop_dsh_lifecycle(
            &active,
            &active.declaration_digest,
            active.fencing_epoch,
            AuthorityChannel::Management,
        )
        .expect("stop");
        assert_eq!(stopped.state, AdapterLifecycleState::Stopped);
    }
}
