#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P7-T07 focused tests: Windows Credential Manager backend fail-closed
//! boundary, production selection, and (on Windows) the real native
//! round-trip against Credential Manager.
//!
//! Synthetic non-production bytes only. Never writes real Provider API keys,
//! never publishes secrets to env/argv/logs/evidence, and never claims B01-W,
//! Windows install parity, any Gate, release, or Profile conformance.
//! `CI-WINDOWS-MSVC-01` executes the Windows-native module; non-Windows hosts
//! execute the fail-closed negatives.

use cognitive_secret::{
    ProductionSecretBackend, SecretAttributes, SecretError, SecretLabel, SecretMaterial, SecretRef,
    SecretStore, SecretStoreAvailability, SecretStoreClass, WindowsCredentialManagerStore,
    select_production_secret_store, select_production_secret_store_with,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_label() -> SecretLabel {
    SecretLabel::new("cognitiveos-personal-provider-api-key (p7-t07 test)").expect("label")
}

fn unique_attributes(purpose_prefix: &str) -> SecretAttributes {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let process = std::process::id();
    SecretAttributes::from_pairs(vec![
        ("application".to_owned(), "cognitiveos-personal".to_owned()),
        (
            "purpose".to_owned(),
            format!("{purpose_prefix}-{process}-{nanos}"),
        ),
    ])
    .expect("attributes")
}

#[test]
fn windows_store_class_is_native() {
    let store = WindowsCredentialManagerStore::new();
    assert_eq!(store.class(), SecretStoreClass::Native);
}

#[test]
fn non_windows_probe_and_mutations_fail_closed() {
    if cfg!(target_os = "windows") {
        return;
    }
    let store = WindowsCredentialManagerStore::new();
    assert_eq!(
        store.probe().expect("probe"),
        SecretStoreAvailability::Unavailable
    );

    let label = test_label();
    let attributes = unique_attributes("p7t07-nonwindows");
    let material =
        SecretMaterial::from_bytes(b"synthetic-nonproduction-bytes".to_vec()).expect("material");
    assert!(matches!(
        store.put(&label, &attributes, material),
        Err(SecretError::Unavailable { .. })
    ));

    let well_formed_ref =
        SecretRef::from_opaque("ssv1:wincred/application/cognitiveos-personal/purpose/p7t07")
            .expect("ref");
    assert!(matches!(
        store.get(&well_formed_ref),
        Err(SecretError::Unavailable { .. })
    ));
    assert!(matches!(
        store.delete(&well_formed_ref),
        Err(SecretError::Unavailable { .. })
    ));
}

#[test]
fn production_selection_contract_holds_on_this_host() {
    // The frozen P1-T02 contract: the Linux-signal override with `false`
    // stays fail-closed Unavailable on every host and never selects the
    // ephemeral double.
    let overridden = select_production_secret_store_with(false);
    assert_eq!(overridden.class(), SecretStoreClass::Unavailable);

    let host = select_production_secret_store();
    assert_ne!(host.class(), SecretStoreClass::EphemeralTestDouble);
    if cfg!(target_os = "windows") {
        // On the Windows validation host the native Credential Manager
        // backend must be selected; silently degrading to Unavailable would
        // hide a broken helper pipeline.
        assert!(matches!(
            host,
            ProductionSecretBackend::WindowsCredentialManager(_)
        ));
        assert_eq!(host.class(), SecretStoreClass::Native);
    }
}

#[cfg(target_os = "windows")]
mod windows_native {
    use super::*;

    fn expected_ref_for(attributes: &SecretAttributes) -> SecretRef {
        let mut segments = vec!["ssv1:wincred".to_owned()];
        for (key, value) in attributes.pairs() {
            segments.push(key.clone());
            segments.push(value.clone());
        }
        SecretRef::from_opaque(segments.join("/")).expect("expected ref")
    }

    #[test]
    fn real_credential_manager_roundtrip_rotate_and_delete() {
        let store = WindowsCredentialManagerStore::new();
        // CI-WINDOWS-MSVC-01 is the declared validation environment for this
        // module; an unavailable backend there must fail the slice loudly
        // instead of skipping.
        assert_eq!(
            store.probe().expect("probe"),
            SecretStoreAvailability::Available
        );

        let label = test_label();
        let attributes = unique_attributes("p7t07-roundtrip");
        let first = SecretMaterial::from_bytes(b"synthetic-first-material-0001".to_vec())
            .expect("material");
        let stored_ref = store.put(&label, &attributes, first).expect("first put");

        let read_back = store.get(&stored_ref).expect("first get");
        assert_eq!(read_back.expose_bytes(), b"synthetic-first-material-0001");

        // Rotation: same attributes replace the material under the same ref.
        let second = SecretMaterial::from_bytes(b"synthetic-second-material-0002".to_vec())
            .expect("material");
        let rotated_ref = store.put(&label, &attributes, second).expect("second put");
        assert_eq!(rotated_ref.as_str(), stored_ref.as_str());
        let rotated = store.get(&stored_ref).expect("rotated get");
        assert_eq!(rotated.expose_bytes(), b"synthetic-second-material-0002");

        store.delete(&stored_ref).expect("delete");
        assert!(matches!(store.get(&stored_ref), Err(SecretError::NotFound)));
        assert!(matches!(
            store.delete(&stored_ref),
            Err(SecretError::NotFound)
        ));
    }

    #[test]
    fn oversized_material_fails_closed_and_stores_nothing() {
        let store = WindowsCredentialManagerStore::new();
        assert_eq!(
            store.probe().expect("probe"),
            SecretStoreAvailability::Available
        );

        let label = test_label();
        let attributes = unique_attributes("p7t07-oversized");
        let oversized =
            SecretMaterial::from_bytes(vec![0x5a_u8; 2561]).expect("oversized material");
        assert!(matches!(
            store.put(&label, &attributes, oversized),
            Err(SecretError::InvalidAttributes { .. })
        ));

        // Nothing may have been persisted for the rejected write.
        let expected_ref = expected_ref_for(&attributes);
        assert!(matches!(
            store.get(&expected_ref),
            Err(SecretError::NotFound)
        ));
    }

    #[test]
    fn foreign_and_absent_refs_read_not_found() {
        let store = WindowsCredentialManagerStore::new();
        assert_eq!(
            store.probe().expect("probe"),
            SecretStoreAvailability::Available
        );

        // A Linux Secret Service ref must not resolve through this backend.
        let linux_ref =
            SecretRef::from_opaque("ssv1:fdss/application/cognitiveos-personal").expect("ref");
        assert!(matches!(store.get(&linux_ref), Err(SecretError::NotFound)));

        // A well-formed wincred ref whose target was never written reads
        // NotFound from the real Credential Manager.
        let absent_ref = expected_ref_for(&unique_attributes("p7t07-absent"));
        assert!(matches!(store.get(&absent_ref), Err(SecretError::NotFound)));
    }

    #[test]
    fn secret_material_never_reaches_debug_output_or_error_text() {
        let marker = "p7t07-leak-marker-4242";
        let store = WindowsCredentialManagerStore::new();
        assert_eq!(
            store.probe().expect("probe"),
            SecretStoreAvailability::Available
        );

        let label = test_label();
        let attributes = unique_attributes("p7t07-redaction");
        let material = SecretMaterial::from_bytes(marker.as_bytes().to_vec()).expect("material");
        let material_debug = format!("{material:?}");
        assert!(!material_debug.contains(marker));

        let stored_ref = store.put(&label, &attributes, material).expect("put");
        let store_debug = format!("{store:?}");
        assert!(!store_debug.contains(marker));

        let not_found = store
            .get(&expected_ref_for(&unique_attributes(
                "p7t07-redaction-absent",
            )))
            .expect_err("absent ref must fail");
        let error_text = format!("{not_found:?} {not_found}");
        assert!(!error_text.contains(marker));

        store.delete(&stored_ref).expect("cleanup delete");
    }
}
