//! `cognitive init` — layout, migrations, Provider binding, self-check.

use cognitive_secret::{
    EphemeralSecretStore, ProviderConfig, ProviderConfigRepository, ProviderKeyService,
    SecretMaterial, SecretStore, select_production_secret_store,
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
        || options.rotate_key;

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

    if already_configured && !options.rotate_key && options.api_key_file.is_none() {
        if let Ok(existing) = config_repository.load() {
            let selected = options
                .model_id
                .as_ref()
                .map(|model_id| format!("manual-model:{model_id}"));
            let updated = ProviderConfig::new(
                provider_id.clone(),
                base_url.clone(),
                existing.secret_ref().clone(),
                selected.or_else(|| existing.selected_snapshot_digest().map(str::to_owned)),
            )
            .map_err(|error| format!("invalid provider config refresh: {error}"))?;
            config_repository
                .store(&updated)
                .map_err(|error| format!("unable to refresh provider config: {error}"))?;
            return Ok(json!({
                "action": "refreshed_non_secret",
                "configured": true,
                "provider_id": provider_id,
                "base_url": base_url,
                "model_selection": updated.selected_snapshot_digest(),
                "secret_material_written": false
            }));
        }
    }

    let material = read_api_key_material(options.api_key_file.as_deref())?;
    let selected_snapshot_digest = options
        .model_id
        .as_ref()
        .map(|model_id| format!("manual-model:{model_id}"));

    if options.allow_ephemeral_secret_backend {
        return put_with_store(
            EphemeralSecretStore::default(),
            config_repository,
            &provider_id,
            &base_url,
            material,
            selected_snapshot_digest,
            "ephemeral-test-double",
            options.rotate_key && already_configured,
        );
    }

    match select_production_secret_store() {
        cognitive_secret::ProductionSecretBackend::LinuxSecretTool(store) => put_with_store(
            store,
            config_repository,
            &provider_id,
            &base_url,
            material,
            selected_snapshot_digest,
            "linux-secret-tool",
            options.rotate_key && already_configured,
        ),
        cognitive_secret::ProductionSecretBackend::Unavailable(_) => Err(
            "no production SecretStore is available on this host. On Linux install \
             FreeDesktop Secret Service and secret-tool, then retry. For hermetic tests only, \
             pass --allow-ephemeral-secret-backend (never for real keys)."
                .to_owned(),
        ),
    }
}

fn put_with_store<S: SecretStore>(
    store: S,
    config_repository: &ProviderConfigRepository,
    provider_id: &str,
    base_url: &str,
    material: SecretMaterial,
    selected_snapshot_digest: Option<String>,
    backend_class: &str,
    rotate: bool,
) -> Result<Value, String> {
    let service = ProviderKeyService::new(store, config_repository.clone());
    let config = if rotate {
        service
            .rotate_provider_key(material)
            .map_err(|error| format!("provider key rotate failed: {error}"))?;
        let existing = service
            .load_config()
            .map_err(|error| format!("provider config reload failed: {error}"))?
            .ok_or_else(|| "provider config missing after rotate".to_owned())?;
        let refreshed = ProviderConfig::new(
            provider_id,
            base_url,
            existing.secret_ref().clone(),
            selected_snapshot_digest
                .clone()
                .or_else(|| existing.selected_snapshot_digest().map(str::to_owned)),
        )
        .map_err(|error| format!("invalid provider config after rotate: {error}"))?;
        config_repository
            .store(&refreshed)
            .map_err(|error| format!("unable to store provider config after rotate: {error}"))?;
        refreshed
    } else {
        service
            .configure_provider(
                provider_id,
                base_url,
                material,
                selected_snapshot_digest.clone(),
            )
            .map_err(|error| format!("provider key configure failed: {error}"))?
    };

    Ok(json!({
        "action": if rotate { "rotated" } else { "configured" },
        "configured": true,
        "provider_id": config.provider_id(),
        "base_url": config.base_url(),
        "model_selection": config.selected_snapshot_digest(),
        "secret_backend": backend_class,
        "secret_material_written": true,
        "secret_ref_redacted": true
    }))
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
