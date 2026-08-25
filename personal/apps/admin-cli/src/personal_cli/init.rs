//! `cognitive init` — layout, migrations, Provider binding, self-check.

use cognitive_provider_transport::RustlsProviderTransport;
use cognitive_secret::{
    EphemeralSecretStore, ModelSelection, ProviderConfig, ProviderConfigRepository,
    ProviderDiscoveryService, ProviderKeyService, ProviderProbeOptions, ProviderTransport,
    SecretMaterial, SecretRef, SecretStore, provider_secret_attributes,
    select_production_secret_store,
};
use cognitive_store::prepare_personal_databases;
use serde_json::{Value, json};

use super::InitOptions;
use super::layout::build_layout;
use super::secret_input::read_api_key_material;
use super::url::normalize_provider_base_url;

/// Run init and return a redacted JSON report (never includes secret bytes).
pub fn run_init(options: &InitOptions) -> Result<Value, String> {
    let layout = build_layout(&options.layout_roots).map_err(|error| {
        format!(
            "layout resolution failed: {error}. Set HOME/USERPROFILE and XDG_RUNTIME_DIR, \
             or pass --runtime-root for a hermetic tree."
        )
    })?;

    layout
        .ensure_directories()
        .map_err(|error| format!("unable to create Personal directories: {error}"))?;

    let prepare_report = prepare_personal_databases(&layout).map_err(|error| {
        format!(
            "database preparation failed: {error}. Existing data was not deleted; \
             inspect backups under {} and retry.",
            layout.backups_dir().display()
        )
    })?;

    let config_repository = ProviderConfigRepository::under_config_dir(layout.config_dir());
    let already_configured = match config_repository.load() {
        Ok(_) => true,
        Err(cognitive_secret::ProviderConfigError::NotFound) => false,
        Err(error) => return Err(format!("unable to load provider config: {error}")),
    };

    let provider_action =
        configure_provider_if_requested(options, &config_repository, already_configured)?;
    let self_check = run_self_check(
        &layout,
        &config_repository,
        options.allow_ephemeral_secret_backend,
    )?;

    Ok(json!({
        "status": "ok",
        "surface": "cognitive-init",
        "schema_version": 1,
        "layout": {
            "config_dir": layout.config_dir().display().to_string(),
            "data_dir": layout.data_dir().display().to_string(),
            "state_dir": layout.state_dir().display().to_string(),
            "cache_dir": layout.cache_dir().display().to_string(),
            "runtime_dir": layout.runtime_dir().display().to_string(),
            "authority_database": layout.authority_database_path().display().to_string(),
            "installation_database": layout.installation_database_path().display().to_string()
        },
        "databases": {
            "authority_applied_versions": prepare_report.authority().applied_versions(),
            "installation_applied_versions": prepare_report.installation().applied_versions(),
            "authority_backup": prepare_report.authority_backup_path().map(|path| path.display().to_string()),
            "installation_backup": prepare_report.installation_backup_path().map(|path| path.display().to_string())
        },
        "provider": provider_action,
        "self_check": self_check,
        "next_steps": [
            "cognitive daemon start",
            "cognitive doctor",
            "Pi package integration remains P1-T07 (not configured)"
        ],
        "idempotent_reinit": true,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false
    }))
}

fn configure_provider_if_requested(
    options: &InitOptions,
    config_repository: &ProviderConfigRepository,
    already_configured: bool,
) -> Result<Value, String> {
    let wants_provider = options.provider_id.is_some()
        || options.base_url.is_some()
        || options.api_key_file.is_some()
        || options.model_id.is_some()
        || options.rotate_key
        || options.reuse_existing_secret_binding;

    if options.reuse_existing_secret_binding
        && (options.api_key_file.is_some() || options.rotate_key)
    {
        return Err(
            "--reuse-existing-secret-binding cannot be combined with --api-key-file or --rotate-key"
                .to_owned(),
        );
    }
    if options.reuse_existing_secret_binding && options.allow_ephemeral_secret_backend {
        return Err(
            "--reuse-existing-secret-binding requires the production Linux Secret Service backend"
                .to_owned(),
        );
    }

    if !wants_provider {
        return Ok(json!({
            "action": if already_configured { "unchanged" } else { "skipped" },
            "configured": already_configured,
            "detail": if already_configured {
                "existing provider config preserved (re-init without provider flags is idempotent)"
            } else {
                "no provider flags supplied; layout and databases only"
            }
        }));
    }

    let provider_id = options.provider_id.clone().ok_or_else(|| {
        "provider configuration requires --provider <id> (example: --provider deepseek)".to_owned()
    })?;
    let base_url_raw = options.base_url.clone().ok_or_else(|| {
        "provider configuration requires --base-url <https-url> (example: --base-url https://api.deepseek.com/v1)"
            .to_owned()
    })?;
    let base_url = normalize_provider_base_url(&base_url_raw)?;

    if options.allow_ephemeral_secret_backend {
        let secret_store = EphemeralSecretStore::default();
        return configure_and_discover_with_store(
            &secret_store,
            &RustlsProviderTransport::default(),
            options,
            config_repository,
            already_configured,
            &provider_id,
            &base_url,
            "ephemeral-test-double",
        );
    }

    match select_production_secret_store() {
        cognitive_secret::ProductionSecretBackend::LinuxSecretTool(store) => {
            configure_and_discover_with_store(
                &store,
                &RustlsProviderTransport::default(),
                options,
                config_repository,
                already_configured,
                &provider_id,
                &base_url,
                "linux-secret-tool",
            )
        }
        cognitive_secret::ProductionSecretBackend::WindowsCredentialManager(store) => {
            if options.reuse_existing_secret_binding {
                return Err(
                    "--reuse-existing-secret-binding currently supports only Linux Secret Service"
                        .to_owned(),
                );
            }
            configure_and_discover_with_store(
                &store,
                &RustlsProviderTransport::default(),
                options,
                config_repository,
                already_configured,
                &provider_id,
                &base_url,
                "windows-credential-manager",
            )
        }
        cognitive_secret::ProductionSecretBackend::Unavailable(_) => Err(
            "no production SecretStore is available on this host. On Linux install \
             FreeDesktop Secret Service and secret-tool, then retry. On Windows ensure the \
             system Windows PowerShell and Credential Manager are usable, then retry. For \
             hermetic tests only, pass --allow-ephemeral-secret-backend (never for real keys)."
                .to_owned(),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn configure_and_discover_with_store<S: SecretStore, T: ProviderTransport>(
    secret_store: &S,
    transport: &T,
    options: &InitOptions,
    config_repository: &ProviderConfigRepository,
    already_configured: bool,
    provider_id: &str,
    base_url: &str,
    backend_class: &str,
) -> Result<Value, String> {
    let provider_key_service = ProviderKeyService::new(secret_store, config_repository.clone());
    let secret_material_written = !options.reuse_existing_secret_binding
        && (!already_configured || options.rotate_key || options.api_key_file.is_some());
    let action = if options.reuse_existing_secret_binding && !already_configured {
        let secret_ref = existing_linux_provider_secret_ref(provider_id)?;
        let bound = ProviderConfig::new(
            provider_id.to_owned(),
            base_url.to_owned(),
            secret_ref,
            None,
        )
        .map_err(|error| format!("invalid reused provider config: {error}"))?;
        config_repository
            .store(&bound)
            .map_err(|error| format!("unable to store reused provider config: {error}"))?;
        "bound_existing_secret_ref"
    } else if secret_material_written {
        let material = read_api_key_material(options.api_key_file.as_deref())?;
        let request = PutProviderKeyRequest {
            config_repository,
            provider_id,
            base_url,
            material,
            rotate: options.rotate_key && already_configured,
        };
        put_with_store(&provider_key_service, request)?
    } else {
        let existing = provider_key_service
            .load_config()
            .map_err(|error| format!("provider config reload failed: {error}"))?
            .ok_or_else(|| "provider configuration disappeared during refresh".to_owned())?;
        let refreshed = ProviderConfig::new(
            provider_id.to_owned(),
            base_url.to_owned(),
            existing.secret_ref().clone(),
            None,
        )
        .map_err(|error| format!("invalid provider config refresh: {error}"))?;
        config_repository
            .store(&refreshed)
            .map_err(|error| format!("unable to refresh provider config: {error}"))?;
        "refreshed_non_secret"
    };

    let model_selection = match &options.model_id {
        Some(model_id) => ModelSelection::ExactCatalog {
            model_id: model_id.clone(),
        },
        None => ModelSelection::FirstDiscovered,
    };
    let discovery = ProviderDiscoveryService::new(&provider_key_service, transport);
    let readiness = discovery
        .discover_probe_and_persist(&ProviderProbeOptions {
            selection: model_selection,
            ..ProviderProbeOptions::default()
        })
        .map_err(|error| {
            format!(
                "provider discovery and capability probe did not select a usable model: {error}"
            )
        })?;
    if !readiness.snapshot.is_minimally_ready() {
        return Err(
            "provider discovery completed but the selected model is not chat-capable; selected-model state was cleared"
                .to_owned(),
        );
    }

    Ok(json!({
        "action": action,
        "configured": true,
        "provider_id": readiness.snapshot.provider_id(),
        "base_url": readiness.snapshot.base_url(),
        "selected_model": readiness.snapshot.selected_model(),
        "snapshot_digest": readiness.snapshot_digest,
        "secret_backend": backend_class,
        "secret_material_written": secret_material_written,
        "secret_ref_redacted": true
    }))
}

struct PutProviderKeyRequest<'a> {
    config_repository: &'a ProviderConfigRepository,
    provider_id: &'a str,
    base_url: &'a str,
    material: SecretMaterial,
    rotate: bool,
}

fn put_with_store<S: SecretStore>(
    service: &ProviderKeyService<S>,
    request: PutProviderKeyRequest<'_>,
) -> Result<&'static str, String> {
    let config = if request.rotate {
        service
            .rotate_provider_key(request.material)
            .map_err(|error| format!("provider key rotate failed: {error}"))?;
        let existing = service
            .load_config()
            .map_err(|error| format!("provider config reload failed: {error}"))?
            .ok_or_else(|| "provider config missing after rotate".to_owned())?;
        let refreshed = ProviderConfig::new(
            request.provider_id,
            request.base_url,
            existing.secret_ref().clone(),
            None,
        )
        .map_err(|error| format!("invalid provider config after rotate: {error}"))?;
        request
            .config_repository
            .store(&refreshed)
            .map_err(|error| format!("unable to store provider config after rotate: {error}"))?;
        refreshed
    } else {
        service
            .configure_provider(
                request.provider_id,
                request.base_url,
                request.material,
                None,
            )
            .map_err(|error| format!("provider key configure failed: {error}"))?
    };

    let _ = config;
    Ok(if request.rotate {
        "rotated"
    } else {
        "configured"
    })
}

fn existing_linux_provider_secret_ref(provider_id: &str) -> Result<SecretRef, String> {
    let attributes = provider_secret_attributes(provider_id)
        .map_err(|error| format!("unable to build provider secret attributes: {error}"))?;
    let mut segments = vec!["ssv1:fdss".to_owned()];
    for (key, value) in attributes.pairs() {
        segments.push(key.clone());
        segments.push(value.clone());
    }
    SecretRef::from_opaque(segments.join("/"))
        .map_err(|error| format!("unable to encode existing provider SecretRef: {error}"))
}

fn run_self_check(
    layout: &cognitive_store::PersonalDataLayout,
    config_repository: &ProviderConfigRepository,
    allow_ephemeral: bool,
) -> Result<Value, String> {
    let authority_exists = layout.authority_database_path().exists();
    let installation_exists = layout.installation_database_path().exists();
    let provider_config = match config_repository.load() {
        Ok(config) => Some(json!({
            "present": true,
            "provider_id": config.provider_id(),
            "base_url": config.base_url(),
            "model_selection": config.selected_snapshot_digest(),
            "secret_ref_present": true
        })),
        Err(cognitive_secret::ProviderConfigError::NotFound) => Some(json!({ "present": false })),
        Err(error) => return Err(format!("self-check provider config failed: {error}")),
    };

    let secret_backend = if allow_ephemeral {
        "ephemeral-test-double-allowed"
    } else {
        match select_production_secret_store() {
            cognitive_secret::ProductionSecretBackend::LinuxSecretTool(_) => "linux-secret-tool",
            cognitive_secret::ProductionSecretBackend::WindowsCredentialManager(_) => {
                "windows-credential-manager"
            }
            cognitive_secret::ProductionSecretBackend::Unavailable(_) => "unavailable",
        }
    };

    Ok(json!({
        "authority_database_exists": authority_exists,
        "installation_database_exists": installation_exists,
        "provider_config": provider_config,
        "secret_backend": secret_backend,
        "pi_package": "not_configured",
        "static_check_is_not_runtime_ready": true
    }))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::fs;

    use super::{InitOptions, configure_and_discover_with_store};
    use crate::personal_cli::layout::LayoutRoots;
    use cognitive_secret::{
        EphemeralSecretStore, ProviderConfigRepository, ProviderHttpMethod, ProviderHttpRequest,
        ProviderHttpResponse, ProviderKeyService, ProviderTransport, ProviderTransportError,
        SecretMaterial, SelectedModel,
    };

    #[derive(Debug, Default)]
    struct DeterministicProviderTransport;

    impl ProviderTransport for DeterministicProviderTransport {
        fn exchange(
            &self,
            request: &ProviderHttpRequest,
        ) -> Result<ProviderHttpResponse, ProviderTransportError> {
            if request.method == ProviderHttpMethod::Get && request.url.ends_with("/models") {
                return Ok(ProviderHttpResponse {
                    status: 200,
                    body: br#"{"object":"list","data":[{"id":"catalog-model","object":"model"}]}"#
                        .to_vec(),
                });
            }

            let request_body = request.body.as_deref().unwrap_or_default();
            let request_text = std::str::from_utf8(request_body).unwrap_or_default();
            if request_text.contains("\"tools\"") {
                return Ok(ProviderHttpResponse {
                    status: 200,
                    body: br#"{"choices":[{"message":{"role":"assistant","tool_calls":[{"id":"call-1","type":"function","function":{"name":"cognitiveos_probe_noop","arguments":"{}"}}]}}]}"#.to_vec(),
                });
            }
            if request_text.contains("\"stream\":true") {
                return Ok(ProviderHttpResponse {
                    status: 200,
                    body: b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n".to_vec(),
                });
            }
            Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#.to_vec(),
            })
        }
    }

    fn init_options(api_key_file: Option<std::path::PathBuf>, model_id: &str) -> InitOptions {
        InitOptions {
            layout_roots: LayoutRoots { runtime_root: None },
            provider_id: Some("test-provider".to_owned()),
            base_url: Some("https://provider.example/v1".to_owned()),
            model_id: Some(model_id.to_owned()),
            api_key_file,
            allow_ephemeral_secret_backend: true,
            rotate_key: false,
            reuse_existing_secret_binding: false,
        }
    }

    #[test]
    fn existing_linux_provider_secret_ref_matches_documented_fdss_encoding() {
        let encoded = super::existing_linux_provider_secret_ref("deepseek").expect("ref");
        assert_eq!(
            encoded.as_str(),
            "ssv1:fdss/application/cognitiveos-personal/provider/deepseek/purpose/provider-api-key"
        );
    }

    #[test]
    fn reuse_existing_binding_rejects_key_file_and_ephemeral() {
        let temporary_directory = tempfile::tempdir().expect("temporary directory");
        let config_repository = ProviderConfigRepository::from_file_path(
            temporary_directory.path().join("provider.json"),
        );
        let mut with_key = init_options(
            Some(temporary_directory.path().join("provider-key.txt")),
            "catalog-model",
        );
        with_key.reuse_existing_secret_binding = true;
        let combined = super::configure_provider_if_requested(&with_key, &config_repository, false)
            .expect_err("reuse cannot combine with a key file");
        assert!(combined.contains("cannot be combined"), "{combined}");

        let mut ephemeral = init_options(None, "catalog-model");
        ephemeral.reuse_existing_secret_binding = true;
        let backend = super::configure_provider_if_requested(&ephemeral, &config_repository, false)
            .expect_err("reuse cannot use the ephemeral backend");
        assert!(backend.contains("Linux Secret Service"), "{backend}");
    }

    #[test]
    fn exact_catalog_discovery_persists_selection_without_secret_output() {
        let temporary_directory = tempfile::tempdir().expect("temporary directory");
        let api_key_path = temporary_directory.path().join("provider-key.txt");
        let synthetic_key = "synthetic-provider-secret";
        fs::write(&api_key_path, format!("{synthetic_key}\n")).expect("write synthetic key");
        let config_repository = ProviderConfigRepository::from_file_path(
            temporary_directory.path().join("provider.json"),
        );
        let secret_store = EphemeralSecretStore::default();

        let result = configure_and_discover_with_store(
            &secret_store,
            &DeterministicProviderTransport,
            &init_options(Some(api_key_path), "catalog-model"),
            &config_repository,
            false,
            "test-provider",
            "https://provider.example/v1",
            "ephemeral-test-double",
        )
        .expect("exact catalog discovery must succeed");

        assert_eq!(result["selected_model"], "catalog-model");
        assert!(result["snapshot_digest"].as_str().is_some());
        assert!(!result.to_string().contains(synthetic_key));
        let selected_model = ProviderKeyService::new(&secret_store, config_repository)
            .selected_model_repository()
            .load()
            .expect("load selected model")
            .expect("discovery must persist selected model");
        assert_eq!(selected_model.model_id(), "catalog-model");
    }

    #[test]
    fn missing_catalog_model_clears_stale_selection() {
        let temporary_directory = tempfile::tempdir().expect("temporary directory");
        let config_repository = ProviderConfigRepository::from_file_path(
            temporary_directory.path().join("provider.json"),
        );
        let secret_store = EphemeralSecretStore::default();
        let provider_key_service =
            ProviderKeyService::new(&secret_store, config_repository.clone());
        provider_key_service
            .configure_provider(
                "test-provider",
                "https://provider.example/v1",
                SecretMaterial::from_bytes(b"synthetic-provider-secret".to_vec())
                    .expect("synthetic material"),
                None,
            )
            .expect("configure provider");
        provider_key_service
            .selected_model_repository()
            .store(
                &SelectedModel::new("stale-model", "fnv1a64:stale", true).expect("stale selection"),
            )
            .expect("persist stale selection");

        let error = configure_and_discover_with_store(
            &secret_store,
            &DeterministicProviderTransport,
            &init_options(None, "missing-model"),
            &config_repository,
            true,
            "test-provider",
            "https://provider.example/v1",
            "ephemeral-test-double",
        )
        .expect_err("missing exact catalog model must fail");

        assert!(error.contains("capability probe"));
        assert!(!error.contains("synthetic-provider-secret"));
        assert!(
            provider_key_service
                .selected_model_repository()
                .load()
                .expect("load selected model")
                .is_none()
        );
    }
}
