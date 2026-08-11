//! First non-Pi Agent fixture qualification — private MVP (P8-T03/D01).
//!
//! Selects OpenAI Codex as the first non-Pi CLI agent from the documented
//! deferred-agent set and binds a fixture package identity that is independent
//! of managed Pi evidence. Registration reuses the AKP adapter contract and
//! never transfers Pi/B09 claims.

use crate::agent_adapter_manifest::{
    AdapterCapabilityDeclaration, AdapterTransportProfile, AgentAdapterError,
    RegisteredAgentAdapter, register_agent_adapter,
};
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

fn bind_discovery_card_digest(package: &NonPiAgentPackageIdentity) -> String {
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
}
