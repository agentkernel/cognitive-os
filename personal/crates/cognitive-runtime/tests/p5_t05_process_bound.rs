#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P5-T05/D01: process-bound SidecarSession failure-first coverage.
//!
//! Activate registers a durable fenced process-attempt identity. Health reports
//! `process_bound=true` only while that binding is current. Pause/stop clear the
//! binding without PID attach, capability grant, Effect, or Task completion.

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, OfficialPiAgentActivationRequest,
    OfficialPiAgentLifecycleRequest, OfficialPiAgentRegistrationRequest, PackageInstallRequest,
    PiInstallationRootActivationRequest, acquire_official_pi_durable,
    activate_official_pi_agent_durable, activate_official_pi_root_durable,
    observe_official_pi_agent_health_durable, package_artifact_digest, package_sha256_digest,
    package_sri_sha512, pause_official_pi_agent_durable, register_official_pi_agent_durable,
    resume_official_pi_agent_durable, stop_official_pi_agent_durable,
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
fn activate_registers_fenced_process_attempt_and_health_is_bound() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, session) = activate_registered(&manager);

    assert!(session.process_bound());
    let attempt = session.process_attempt_id().expect("process attempt");
    assert!(!attempt.is_empty());
    assert_ne!(attempt, session.session_id());
    assert_ne!(attempt, activated.instance_id());

    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(health.process_bound());
    assert!(health.current_sidecar_session());
    assert_eq!(health.lifecycle_state(), "active");
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn unbound_registered_health_reports_process_bound_false() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let binding = activate_official_root(&manager);
    let registered = register_official_pi_agent_durable(
        &manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();

    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert_eq!(health.instance_id(), registered.instance_id());
    assert_eq!(health.lifecycle_state(), "registered");
    assert!(!health.current_sidecar_session());
    assert!(!health.process_bound());
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn pause_and_stop_clear_process_binding_without_capability() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, session) = activate_registered(&manager);
    let attempt = session.process_attempt_id().unwrap().to_owned();

    let paused = pause_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    let paused_health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert_eq!(paused.lifecycle_state(), "paused");
    assert!(!paused_health.process_bound());
    assert!(!paused_health.current_sidecar_session());
    assert!(
        manager
            .current_sidecar_session(paused.instance_id())
            .unwrap()
            .is_none()
    );

    let (resumed, resumed_session) = resume_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: paused.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert!(resumed_session.process_bound());
    assert_ne!(
        resumed_session.process_attempt_id().unwrap(),
        attempt.as_str()
    );
    let resumed_health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(resumed_health.process_bound());

    let stopped = stop_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: resumed.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    let stopped_health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert_eq!(stopped.lifecycle_state(), "stopped");
    assert!(!stopped_health.process_bound());
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn stale_epoch_pause_preserves_process_binding() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, session) = activate_registered(&manager);

    let error = pause_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch() - 1,
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "STATE_CONFLICT");

    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(health.process_bound());
    let current = manager
        .current_sidecar_session(activated.instance_id())
        .unwrap()
        .expect("current session remains");
    assert_eq!(current.process_attempt_id(), session.process_attempt_id());
    assert_eq!(authority.capability_grants(), 0);
}
