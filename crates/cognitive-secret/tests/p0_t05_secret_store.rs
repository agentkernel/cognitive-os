#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P0-T05 focused tests: put/get/rotate/delete, fail-closed modes, redaction.
//!
//! These tests never write Provider keys, SQLite rows, config files, evidence
//! digests, or environment variables containing secret material.

use cognitive_secret::{
    EphemeralSecretStore, LinuxSecretServiceProbe, SecretAttributes, SecretError, SecretLabel,
    SecretMaterial, SecretServiceSimulation, SecretStore, SecretStoreAvailability,
    SecretStoreClass, SimulatedSecretServiceStore, UnavailableSecretStore,
};

fn sample_label() -> SecretLabel {
    SecretLabel::new("cognitiveos-personal-poc").expect("label")
}

fn sample_attributes() -> SecretAttributes {
    SecretAttributes::from_pairs(vec![
        ("application".into(), "cognitiveos-personal".into()),
        ("provider".into(), "deepseek".into()),
        ("purpose".into(), "provider-api-key".into()),
    ])
    .expect("attributes")
}

fn sample_material(marker: &str) -> SecretMaterial {
    // Synthetic non-production bytes only. Never a real Provider key.
    SecretMaterial::from_bytes(format!("poc-test-material-{marker}").into_bytes())
        .expect("material")
}

#[test]
fn put_get_rotate_delete_on_ephemeral_store() {
    let store = EphemeralSecretStore::default();
    assert_eq!(store.class(), SecretStoreClass::EphemeralTestDouble);
    assert_eq!(
        store.probe().expect("probe"),
        SecretStoreAvailability::Available
    );

    let first = sample_material("v1");
    let secret_ref = store
        .put(&sample_label(), &sample_attributes(), first)
        .expect("put");
    let loaded = store.get(&secret_ref).expect("get");
    assert_eq!(loaded, sample_material("v1"));

    // Rotate = put with the same attributes replaces material.
    let rotated_ref = store
        .put(&sample_label(), &sample_attributes(), sample_material("v2"))
        .expect("rotate");
    assert_eq!(rotated_ref, secret_ref);
    let rotated = store.get(&secret_ref).expect("get after rotate");
    assert_eq!(rotated, sample_material("v2"));

    store.delete(&secret_ref).expect("delete");
    let missing = store.get(&secret_ref).expect_err("deleted");
    assert_eq!(missing, SecretError::NotFound);
}

#[test]
fn unavailable_store_fails_closed_without_fallback() {
    let store = UnavailableSecretStore;
    assert_eq!(store.class(), SecretStoreClass::Unavailable);
    assert_eq!(
        store.probe().expect("probe"),
        SecretStoreAvailability::Unavailable
    );
    let err = store
        .put(
            &sample_label(),
            &sample_attributes(),
            sample_material("blocked"),
        )
        .expect_err("put must fail closed");
    assert!(matches!(err, SecretError::Unavailable { .. }));
}

#[test]
fn service_absent_locked_and_prompt_modes_fail_closed() {
    let store = SimulatedSecretServiceStore::new(SecretServiceSimulation::ServiceAbsent);
    assert_eq!(
        store.probe().expect("probe"),
        SecretStoreAvailability::Unavailable
    );
    assert!(matches!(
        store
            .put(
                &sample_label(),
                &sample_attributes(),
                sample_material("absent")
            )
            .expect_err("absent"),
        SecretError::Unavailable { .. }
    ));

    store
        .set_mode(SecretServiceSimulation::CollectionLocked)
        .expect("mode");
    assert_eq!(
        store.probe().expect("probe"),
        SecretStoreAvailability::Locked
    );
    assert_eq!(
        store.get(&secret_ref_for_test()).expect_err("locked"),
        SecretError::Locked
    );

    store
        .set_mode(SecretServiceSimulation::PromptUnavailable)
        .expect("mode");
    assert_eq!(
        store.probe().expect("probe"),
        SecretStoreAvailability::PromptUnavailable
    );
    assert_eq!(
        store.delete(&secret_ref_for_test()).expect_err("prompt"),
        SecretError::PromptUnavailable
    );
}

fn secret_ref_for_test() -> cognitive_secret::SecretRef {
    cognitive_secret::SecretRef::from_opaque("ssv1:ephemeral/missing").expect("ref")
}

#[test]
fn secret_material_debug_display_and_errors_never_leak_bytes() {
    let marker = "super-secret-leak-marker-9f3c2a";
    let material = SecretMaterial::from_bytes(marker.as_bytes().to_vec()).expect("material");
    let debug_text = format!("{material:?}");
    let display_text = format!("{material}");
    assert!(!debug_text.contains(marker));
    assert!(!display_text.contains(marker));
    assert!(debug_text.contains("redacted"));
    assert!(display_text.contains("redacted"));

    let err = SecretError::Unavailable {
        reason: "org.freedesktop.secrets is absent from the session bus",
    };
    let err_text = err.to_string();
    assert!(!err_text.contains(marker));
}

#[test]
fn secret_material_not_exported_to_process_environment() {
    let marker = "env-leak-marker-should-not-exist";
    let material = SecretMaterial::from_bytes(marker.as_bytes().to_vec()).expect("material");
    // Holding material in-process must not publish it into the environment.
    for (key, value) in std::env::vars() {
        assert!(
            !value.contains(marker),
            "environment variable {key} unexpectedly contains secret marker"
        );
    }
    // Keep material alive across the scan so the test is meaningful.
    assert_eq!(material.len(), marker.len());
}

#[test]
fn linux_probe_is_native_class_and_never_plaintext_fallback() {
    let probe = LinuxSecretServiceProbe::new();
    assert_eq!(probe.class(), SecretStoreClass::Native);
    let availability = probe.probe().expect("probe");
    if cfg!(target_os = "linux") {
        assert!(matches!(
            availability,
            SecretStoreAvailability::Available | SecretStoreAvailability::Unavailable
        ));
    } else {
        assert_eq!(availability, SecretStoreAvailability::Unavailable);
    }
    // Mutating native I/O is intentionally deferred; no plaintext path exists.
    let put_err = probe
        .put(
            &sample_label(),
            &sample_attributes(),
            sample_material("native-deferred"),
        )
        .expect_err("native put deferred");
    assert!(matches!(put_err, SecretError::Unavailable { .. }));
}

#[test]
fn delete_missing_ref_is_not_found() {
    let store = EphemeralSecretStore::default();
    let missing = cognitive_secret::SecretRef::from_opaque("ssv1:ephemeral/404").expect("ref");
    assert_eq!(
        store.delete(&missing).expect_err("missing"),
        SecretError::NotFound
    );
}
