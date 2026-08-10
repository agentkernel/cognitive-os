//! Daemon-private Agent registration from an active official Pi installation root.
//!
//! P5-T02/D01: install ≠ register ≠ activate. Registration binds an inactive
//! `AgentInstance` identity to exact package/adapter/protocol/policy digests and
//! grants zero capabilities. It creates no SidecarSession, process, Effect, or
//! Task completion fact.

use crate::installer::{
    DurableInstallationManager, InstallerError, OFFICIAL_PI_INSTALLATION_ROOT, map_store_error,
    verification_failure,
};
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_kernel::ports::IdGenerator;
use cognitive_store::{AgentRegistrationCommit, AgentRegistrationRecord, UuidV7Generator};

/// Fixed official Pi sidecar protocol identity for Linux 1.0 foundation work.
pub const OFFICIAL_PI_SIDECAR_PROTOCOL: &str = "cognitiveos.private-sidecar/1";

/// Request to register a managed Agent from an active official installation root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfficialPiAgentRegistrationRequest {
    pub installation_root: String,
    pub expected_activation_version: u64,
    pub expected_adapter_digest: String,
    pub protocol_digest: String,
    pub policy_digest: String,
}

/// Register one official Pi Agent from the exact active installation root.
///
/// The durable outcome is an inactive `registered` instance. Callers must not
/// treat this as SidecarSession creation, process supervision, permission, or
/// Task completion.
pub fn register_official_pi_agent_durable(
    manager: &DurableInstallationManager<'_>,
    request: &OfficialPiAgentRegistrationRequest,
) -> Result<AgentRegistrationRecord, InstallerError> {
    if request.installation_root != OFFICIAL_PI_INSTALLATION_ROOT {
        return Err(verification_failure(
            "agent registration currently admits only the official Pi installation root",
        ));
    }
    if !is_digest(&request.expected_adapter_digest)
        || !is_digest(&request.protocol_digest)
        || !is_digest(&request.policy_digest)
    {
        return Err(verification_failure(
            "agent registration requires sha256 digests for adapter, protocol, and policy",
        ));
    }

    let binding = manager
        .active_installation_root(&request.installation_root)?
        .ok_or_else(|| {
            verification_failure("official Pi installation root is not active; install ≠ register")
        })?;
    if binding.activation_version() != request.expected_activation_version {
        return Err(InstallerError::new(
            RegisteredErrorCode::StateConflict,
            format!(
                "installation root expected version {}, found {}",
                request.expected_activation_version,
                binding.activation_version()
            ),
        ));
    }

    let committed = manager
        .committed_installation(binding.package_ref())?
        .ok_or_else(|| verification_failure("active root package acquisition lock is missing"))?;
    let evidence = committed.evidence().ok_or_else(|| {
        verification_failure("active root package lacks official Pi acquisition evidence")
    })?;
    if evidence.source_mode() != "official_pi"
        || evidence.verification_result() != "official_acquisition_lock_verified"
    {
        return Err(verification_failure(
            "active root is not an official Pi acquisition lock",
        ));
    }
    let acquisition_lock = evidence.acquisition_lock().ok_or_else(|| {
        verification_failure("active root official Pi evidence has no acquisition lock")
    })?;
    if acquisition_lock != binding.acquisition_lock() {
        return Err(verification_failure(
            "active root acquisition lock no longer matches committed evidence",
        ));
    }
    if committed.adapter_digest() != request.expected_adapter_digest {
        return Err(InstallerError::new(
            RegisteredErrorCode::DigestMismatch,
            "adapter digest does not match the committed official Pi installation",
        ));
    }

    if manager
        .current_agent_registration(&request.installation_root)?
        .is_some()
    {
        return Err(InstallerError::new(
            RegisteredErrorCode::StateConflict,
            "installation root already has a current agent registration",
        ));
    }

    let ids = UuidV7Generator;
    let registration_id = ids.next_uuid_v7().map_err(|err| {
        InstallerError::new(
            RegisteredErrorCode::StateStoreUnavailable,
            format!("allocate registration id: {err}"),
        )
    })?;
    let instance_id = ids.next_uuid_v7().map_err(|err| {
        InstallerError::new(
            RegisteredErrorCode::StateStoreUnavailable,
            format!("allocate instance id: {err}"),
        )
    })?;

    let record = manager
        .register_agent_from_active_root(&AgentRegistrationCommit {
            registration_id,
            instance_id,
            installation_root: binding.installation_root().to_owned(),
            expected_activation_version: binding.activation_version(),
            package_ref: binding.package_ref().to_owned(),
            acquisition_lock: binding.acquisition_lock().to_owned(),
            adapter_digest: committed.adapter_digest().to_owned(),
            protocol_digest: request.protocol_digest.clone(),
            policy_digest: request.policy_digest.clone(),
        })
        .map_err(map_store_error)?;

    if record.lifecycle_state() != "registered" || record.fencing_epoch() != 1 {
        return Err(InstallerError::new(
            RegisteredErrorCode::StateStoreUnavailable,
            "registration persisted an unexpected instance lifecycle state",
        ));
    }
    Ok(record)
}

fn is_digest(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("sha256:") else {
        return false;
    };
    !rest.is_empty()
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::is_digest;

    #[test]
    fn digest_helper_rejects_malformed_values() {
        assert!(is_digest("sha256:adapter"));
        assert!(is_digest(&format!("sha256:{}", "ab".repeat(32))));
        assert!(!is_digest("sha256:"));
        assert!(!is_digest("md5:deadbeef"));
    }
}
