#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P5-T05/D03: recover/orphan and identity-separation negatives.
//!
//! Cleared process attempts stay orphaned (no reattach). Recover allocates a
//! new fenced attempt. AgentInstance, SidecarSession, process-attempt, and
//! Task identities stay separated. Install/register never grants permission.

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, OfficialPiAgentActivationRequest,
    OfficialPiAgentLifecycleRequest, OfficialPiAgentRegistrationRequest, PackageInstallRequest,
    PiInstallationRootActivationRequest, acquire_official_pi_durable,
    activate_official_pi_agent_durable, activate_official_pi_root_durable,
    observe_official_pi_agent_health_durable, package_artifact_digest, package_sha256_digest,
    package_sri_sha512, recover_official_pi_agent_durable, register_official_pi_agent_durable,
    stop_official_pi_agent_durable,
};

fn official_request() -> OfficialPiAcquisitionRequest {
    let artifact = b"staged-official-pi-package".to_vec();
    let dependency_lock = b"locked-dependencies".to_vec();
    let artifact_digest = package_artifact_digest(&artifact).unwrap();
    let lock_digest = package_artifact_digest(&dependency_lock).unwrap();
    OfficialPiAcquisitionRequest {
        install: PackageInstallRequest {
            package_id: format!("pkg://{OFFICIAL_PI_PACKAGE}@{OFFICIAL_PI_VERSION}"),
            publisher: OFFICIAL_PI_PACKAGE.to_owned(),
            package_version: OFFICIAL_PI_VERSION.to_owned(),
            artifact: artifact.clone(),
            declared_artifact_digest: artifact_digest,
            signature_ref: "official-lock".to_owned(),
            provenance_ref: OFFICIAL_NPM_ORIGIN.to_owned(),
            adapter_digest: "sha256:adapter".to_owned(),
            sandbox_digest: "sha256:sandbox".to_owned(),
            compatibility_digest: "sha256:compatibility".to_owned(),
            lockfile_digest: lock_digest.clone(),
            expected_adapter_digest: "sha256:adapter".to_owned(),
            expected_sandbox_digest: "sha256:sandbox".to_owned(),
            expected_compatibility_digest: "sha256:compatibility".to_owned(),
        },
        registry_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        resolved_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        sri_sha512: package_sri_sha512(&artifact),
        declared_package_sha256: package_sha256_digest(&artifact),
        dependency_lock,
        declared_dependency_lock_digest: lock_digest,
        node_version: "22.19.0".to_owned(),
        signed_acquisition_lock_ref: "attestation://pi/lock-01".to_owned(),
    }
}

fn activate_official_root(
    manager: &cognitive_runtime::DurableInstallationManager<'_>,
) -> cognitive_store::InstallationRootBinding {
    let request = official_request();
    acquire_official_pi_durable(
        manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();
    activate_official_pi_root_durable(
        manager,
        &PiInstallationRootActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            package_ref: request.install.package_id.clone(),
            expected_activation_version: None,
            compatibility_accepted: true,
            health_accepted: true,
        },
    )
    .unwrap()
}

fn registration_request(expected_activation_version: u64) -> OfficialPiAgentRegistrationRequest {
    OfficialPiAgentRegistrationRequest {
        installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
        expected_activation_version,
        expected_adapter_digest: "sha256:adapter".to_owned(),
        protocol_digest: "sha256:sidecar-protocol".to_owned(),
        policy_digest: "sha256:personal-policy".to_owned(),
    }
}

fn activate_registered(
    manager: &cognitive_runtime::DurableInstallationManager<'_>,
) -> (
    cognitive_store::AgentRegistrationRecord,
    cognitive_store::SidecarSessionRecord,
) {
    let binding = activate_official_root(manager);
    let registered = register_official_pi_agent_durable(
        manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();
    activate_official_pi_agent_durable(
        manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: registered.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap()
}

#[test]
fn install_and_register_do_not_grant_permission_or_process_binding() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let binding = activate_official_root(&manager);
    assert_eq!(authority.capability_grants(), 0);

    let registered = register_official_pi_agent_durable(
        &manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();
    assert_eq!(registered.lifecycle_state(), "registered");
    assert_eq!(authority.capability_grants(), 0);
    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(!health.process_bound());
    assert!(!health.current_sidecar_session());
    assert!(
        manager
            .current_sidecar_session(registered.instance_id())
            .unwrap()
            .is_none()
    );
}

#[test]
fn identities_stay_separated_across_activate_stop_recover() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, session) = activate_registered(&manager);
    let first_attempt = session.process_attempt_id().unwrap().to_owned();

    assert_ne!(activated.instance_id(), session.session_id());
    assert_ne!(activated.instance_id(), first_attempt.as_str());
    assert_ne!(session.session_id(), first_attempt.as_str());
    // No Task / AgentExecution / PiSession identity is minted by this path.
    assert!(!activated.instance_id().starts_with("task://"));
    assert!(!session.session_id().starts_with("task://"));
    assert!(!first_attempt.starts_with("task://"));

    let stopped = stop_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    let stopped_health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(!stopped_health.process_bound());
    assert!(
        manager
            .current_sidecar_session(stopped.instance_id())
            .unwrap()
            .is_none()
    );

    let (recovered, recovered_session) = recover_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: stopped.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    let recovered_attempt = recovered_session.process_attempt_id().unwrap();
    assert_eq!(recovered.instance_id(), activated.instance_id());
    assert_ne!(recovered_session.session_id(), session.session_id());
    assert_ne!(recovered_attempt, first_attempt.as_str());
    assert_ne!(recovered.instance_id(), recovered_session.session_id());
    assert_ne!(recovered.instance_id(), recovered_attempt);
    assert_ne!(recovered_session.session_id(), recovered_attempt);
    assert!(
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .process_bound()
    );
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn orphan_cleared_attempt_is_not_reattached_by_stale_recover() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, session) = activate_registered(&manager);
    let orphan_attempt = session.process_attempt_id().unwrap().to_owned();

    let stopped = stop_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();

    let stale = recover_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch() - 1,
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap_err();
    assert_eq!(stale.code, "STATE_CONFLICT");

    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(!health.process_bound());
    assert_eq!(health.lifecycle_state(), "stopped");
    assert!(
        manager
            .current_sidecar_session(stopped.instance_id())
            .unwrap()
            .is_none()
    );

    let (recovered, recovered_session) = recover_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: stopped.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert_ne!(
        recovered_session.process_attempt_id().unwrap(),
        orphan_attempt.as_str()
    );
    assert_eq!(recovered.fencing_epoch(), stopped.fencing_epoch() + 1);
    assert_eq!(authority.capability_grants(), 0);
}
