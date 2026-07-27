#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P1-T02 focused tests: Provider config binding, rotation, restart, redaction.
//!
//! Synthetic non-production bytes only. Never writes real Provider API keys,
//! never publishes secrets to env/argv/logs/evidence, and never claims G0,
//! B01-B12, or Profile conformance.

use cognitive_secret::{
    EphemeralSecretStore, LinuxSecretToolStore, ProviderConfig, ProviderConfigError,
    ProviderConfigRepository, ProviderKeyService, ProviderKeyServiceError, SecretError,
    SecretMaterial, SecretServiceSimulation, SecretStore, SecretStoreAvailability,
    SecretStoreClass, SelectedModel, SimulatedSecretServiceStore, read_secret_material_from_reader,
    select_production_secret_store, select_production_secret_store_with,
};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn hermetic_config_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cognitiveos-p1-t02-{label}-{nanos}"));
    fs::create_dir_all(&directory).expect("temp config dir");
    directory.join("provider.json")
}

fn sample_material(marker: &str) -> SecretMaterial {
    SecretMaterial::from_bytes(format!("poc-provider-material-{marker}").into_bytes())
        .expect("material")
}

#[test]
fn configure_rotate_and_delete_provider_key() {
    let config_path = hermetic_config_path("lifecycle");
    let service = ProviderKeyService::new(
        EphemeralSecretStore::default(),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    service
        .selected_model_repository()
        .store(
            &SelectedModel::new("stale-before-configure", "fnv1a64:test", true)
                .expect("selected model"),
        )
        .expect("selected model store");

    let configured = service
        .configure_provider(
            "deepseek",
            "https://api.deepseek.com",
            sample_material("v1"),
            None,
        )
        .expect("configure");
    assert_eq!(configured.provider_id(), "deepseek");
    assert_eq!(configured.base_url(), "https://api.deepseek.com");
    assert!(
        service
            .selected_model_repository()
            .load()
            .expect("selected model state")
            .is_none(),
        "provider configuration must invalidate a previous selected model"
    );

    let on_disk = fs::read_to_string(&config_path).expect("read config");
    assert!(on_disk.contains("secret_ref"));
    assert!(on_disk.contains(configured.secret_ref().as_str()));
    assert!(!on_disk.contains("poc-provider-material-v1"));
    assert!(!on_disk.contains("poc-provider-material"));

    let resolved = service
        .resolve_provider_material()
        .expect("resolve after configure");
    assert_eq!(resolved, sample_material("v1"));

    service
        .selected_model_repository()
        .store(
            &SelectedModel::new("stale-after-configure", "fnv1a64:test", true)
                .expect("selected model"),
        )
        .expect("selected model store");

    let rotated = service
        .rotate_provider_key(sample_material("v2"))
        .expect("rotate");
    assert_eq!(rotated.secret_ref(), configured.secret_ref());
    let resolved_rotated = service
        .resolve_provider_material()
        .expect("resolve after rotate");
    assert_eq!(resolved_rotated, sample_material("v2"));
    assert!(
        service
            .selected_model_repository()
            .load()
            .expect("selected model state")
            .is_none(),
        "key rotation must invalidate a previous selected model"
    );

    service
        .selected_model_repository()
        .store(
            &SelectedModel::new("stale-after-rotate", "fnv1a64:test", true)
                .expect("selected model"),
        )
        .expect("selected model store");

    service.delete_provider_key().expect("delete");
    assert!(service.load_config().expect("load after delete").is_none());
    assert!(!config_path.exists());
    assert!(
        service
            .selected_model_repository()
            .load()
            .expect("selected model state")
            .is_none(),
        "provider deletion must invalidate a previous selected model"
    );
}

#[test]
fn restart_resolves_secret_via_persisted_ref_on_shared_store() {
    let config_path = hermetic_config_path("restart");
    let shared_store = Arc::new(SimulatedSecretServiceStore::default());
    let first = ProviderKeyService::new(
        Arc::clone(&shared_store),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    first
        .configure_provider(
            "deepseek",
            "https://api.deepseek.com",
            sample_material("persist"),
            Some("deadbeef".into()),
        )
        .expect("configure");

    // Second process/image: new service, same durable config + secret backend.
    let second = ProviderKeyService::new(
        Arc::clone(&shared_store),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    let config = second.load_config().expect("load").expect("present");
    assert_eq!(config.selected_snapshot_digest(), Some("deadbeef"));
    let material = second.resolve_provider_material().expect("resolve");
    assert_eq!(material, sample_material("persist"));
}

#[test]
fn deleted_secret_fails_closed_after_config_reload() {
    let config_path = hermetic_config_path("deleted");
    let store = Arc::new(SimulatedSecretServiceStore::default());
    let service = ProviderKeyService::new(
        Arc::clone(&store),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    let config = service
        .configure_provider(
            "deepseek",
            "https://api.deepseek.com",
            sample_material("gone"),
            None,
        )
        .expect("configure");
    store
        .delete(config.secret_ref())
        .expect("direct delete of secret item");

    let reloaded = ProviderKeyService::new(
        Arc::clone(&store),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    assert!(matches!(
        reloaded.resolve_provider_material().expect_err("missing"),
        ProviderKeyServiceError::SecretMissing
    ));
    // Config still present; secret material is gone.
    assert!(reloaded.load_config().expect("load").is_some());
}

#[test]
fn locked_secret_store_fails_closed_without_config_write() {
    let config_path = hermetic_config_path("locked");
    let store = SimulatedSecretServiceStore::new(SecretServiceSimulation::CollectionLocked);
    let service = ProviderKeyService::new(
        store,
        ProviderConfigRepository::from_file_path(&config_path),
    );
    assert_eq!(
        service.probe_secret_store().expect("probe"),
        SecretStoreAvailability::Locked
    );
    let err = service
        .configure_provider(
            "deepseek",
            "https://api.deepseek.com",
            sample_material("locked"),
            None,
        )
        .expect_err("locked put");
    assert!(matches!(
        err,
        ProviderKeyServiceError::Secret(SecretError::Locked)
    ));
    assert!(!config_path.exists());
}

#[test]
fn provider_config_rejects_http_and_embedded_credentials() {
    let secret_ref = cognitive_secret::SecretRef::from_opaque("ssv1:ephemeral/1").expect("ref");
    let http_err = ProviderConfig::new(
        "deepseek",
        "http://api.deepseek.com",
        secret_ref.clone(),
        None,
    )
    .expect_err("http");
    assert!(matches!(http_err, ProviderConfigError::Invalid { .. }));

    let cred_err = ProviderConfig::new(
        "deepseek",
        "https://user:pass@api.deepseek.com",
        secret_ref,
        None,
    )
    .expect_err("credentials");
    assert!(matches!(cred_err, ProviderConfigError::Invalid { .. }));
}

#[test]
fn config_debug_and_service_debug_never_include_secret_bytes() {
    let config_path = hermetic_config_path("redact");
    let marker = "poc-provider-material-redact-marker";
    let service = ProviderKeyService::new(
        EphemeralSecretStore::default(),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    let config = service
        .configure_provider(
            "deepseek",
            "https://api.deepseek.com",
            sample_material("redact-marker"),
            None,
        )
        .expect("configure");
    let config_debug = format!("{config:?}");
    let service_debug = format!("{service:?}");
    let on_disk = fs::read_to_string(&config_path).expect("disk");
    assert!(!config_debug.contains(marker));
    assert!(!service_debug.contains(marker));
    assert!(!on_disk.contains(marker));
    assert!(service_debug.contains("redacted-backend"));
}

#[test]
fn production_selection_never_returns_ephemeral_double() {
    let non_linux = select_production_secret_store_with(false);
    assert_ne!(non_linux.class(), SecretStoreClass::EphemeralTestDouble);
    assert_eq!(non_linux.class(), SecretStoreClass::Unavailable);

    let host = select_production_secret_store();
    assert_ne!(host.class(), SecretStoreClass::EphemeralTestDouble);
    assert!(matches!(
        host.class(),
        SecretStoreClass::Native | SecretStoreClass::Unavailable
    ));
}

#[test]
fn linux_secret_tool_store_is_native_and_fail_closed_without_session() {
    let store = LinuxSecretToolStore::new();
    assert_eq!(store.class(), SecretStoreClass::Native);
    let availability = store.probe().expect("probe");
    if !cfg!(target_os = "linux") {
        assert_eq!(availability, SecretStoreAvailability::Unavailable);
    }
    // Mutating without a usable session must fail closed (no plaintext path).
    if availability != SecretStoreAvailability::Available {
        let label = cognitive_secret::SecretLabel::new("cognitiveos-personal-provider-api-key")
            .expect("label");
        let attributes = cognitive_secret::SecretAttributes::from_pairs(vec![
            ("application".into(), "cognitiveos-personal".into()),
            ("provider".into(), "deepseek".into()),
            ("purpose".into(), "provider-api-key".into()),
        ])
        .expect("attrs");
        let err = store
            .put(&label, &attributes, sample_material("native-unavailable"))
            .expect_err("unavailable put");
        assert!(matches!(err, SecretError::Unavailable { .. }));
    }
}

#[test]
fn hidden_input_reader_strips_newline_and_never_exports_env() {
    let marker = "hidden-input-marker-bytes";
    let mut cursor = Cursor::new(format!("{marker}\n").into_bytes());
    let material = read_secret_material_from_reader(&mut cursor).expect("read");
    assert_eq!(material, sample_material_raw(marker));
    for (key, value) in std::env::vars() {
        assert!(
            !value.contains(marker),
            "environment variable {key} unexpectedly contains secret marker"
        );
    }
}

fn sample_material_raw(marker: &str) -> SecretMaterial {
    SecretMaterial::from_bytes(marker.as_bytes().to_vec()).expect("material")
}

#[test]
fn provider_config_round_trip_json_without_secret_bytes() {
    let secret_ref = cognitive_secret::SecretRef::from_opaque("ssv1:ephemeral/42").expect("ref");
    let config = ProviderConfig::new("deepseek", "https://api.deepseek.com", secret_ref, None)
        .expect("config");
    let document = config.to_json_document();
    assert!(!document.contains("poc-provider"));
    let parsed = ProviderConfig::from_json_document(&document).expect("parse");
    assert_eq!(parsed, config);
}
