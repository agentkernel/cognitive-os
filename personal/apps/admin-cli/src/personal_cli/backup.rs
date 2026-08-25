//! Public `cognitive backup` / `restore` client (P2-T27/D01).
//!
//! Default verbs talk to the Personal daemon over the management channel so
//! the daemon remains the archive writer (A1). `--output` and `--archive`
//! filesystem paths are the offline path used when the daemon is stopped.
//! This module never copies raw authority SQLite, secrets, bearer material,
//! or provider-config.

use std::path::PathBuf;

use cognitive_store::{
    preflight_personal_backup_archive, restore_personal_backup_archive,
    write_personal_backup_archive, write_personal_backup_archive_to,
};
use serde_json::{Value, json};

use super::client::PersonalDaemonClient;
use super::daemon;
use super::layout::{self, LayoutRoots};

/// `cognitive backup [--output <dir>] [--endpoint <host:port>]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupOptions {
    pub layout_roots: LayoutRoots,
    pub endpoint_override: Option<String>,
    pub output: Option<PathBuf>,
}

/// `cognitive restore --archive-id <id> | --archive <dir> [--preflight]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOptions {
    pub layout_roots: LayoutRoots,
    pub endpoint_override: Option<String>,
    pub archive: Option<PathBuf>,
    pub archive_id: Option<String>,
    pub preflight_only: bool,
}

pub fn run_backup(options: &BackupOptions) -> Result<Value, String> {
    let layout = layout::build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    if let Some(output) = &options.output {
        let receipt = write_personal_backup_archive_to(&layout, Some(output))
            .map_err(|error| error.to_string())?;
        return Ok(backup_receipt_json(&receipt));
    }
    if let Some(client) = try_connect(options.endpoint_override.as_deref(), &layout) {
        let body = client.post_backup().map_err(|error| error.to_string())?;
        return decorate_daemon_body("backup", &body);
    }
    let receipt = write_personal_backup_archive(&layout).map_err(|error| error.to_string())?;
    Ok(backup_receipt_json(&receipt))
}

pub fn run_restore(options: &RestoreOptions) -> Result<Value, String> {
    let layout = layout::build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    if let Some(archive_id) = options.archive_id.as_deref() {
        let client = connect_required(options.endpoint_override.as_deref(), &layout)?;
        let body = if options.preflight_only {
            client
                .post_backup_preflight(archive_id)
                .map_err(|error| error.to_string())?
        } else {
            client
                .post_restore(archive_id)
                .map_err(|error| error.to_string())?
        };
        return decorate_daemon_body("restore", &body);
    }
    let archive = options
        .archive
        .as_ref()
        .ok_or_else(|| "restore requires --archive <dir> or --archive-id <id>".to_owned())?;
    if options.preflight_only {
        let preflight = preflight_personal_backup_archive(&layout, archive)
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "status": "ok",
            "surface": "cognitive-cli",
            "verb": "restore",
            "preflight_only": true,
            "export_plan_digest": preflight.export_plan_digest,
            "backup_schema_version": preflight.backup_schema_version,
            "profile_claim": "not-claimed",
            "gate_claim": "not-claimed"
        }));
    }
    let receipt =
        restore_personal_backup_archive(&layout, archive).map_err(|error| error.to_string())?;
    Ok(json!({
        "status": "ok",
        "surface": "cognitive-cli",
        "verb": "restore",
        "schema": receipt.schema,
        "archive_id": receipt.archive_id,
        "restored_path": receipt.restored_path,
        "manifest_digest": receipt.manifest_digest,
        "live_applied": receipt.live_applied,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed"
    }))
}

fn backup_receipt_json(receipt: &cognitive_store::BackupArchiveReceipt) -> Value {
    json!({
        "status": "ok",
        "surface": "cognitive-cli",
        "verb": "backup",
        "schema": receipt.schema,
        "archive_id": receipt.archive_id,
        "archive_path": receipt.archive_path,
        "manifest_digest": receipt.manifest_digest,
        "export_plan_digest": receipt.export_plan_digest,
        "sqlite_copied": receipt.sqlite_copied,
        "excluded_secret_count": receipt.excluded_secret_count,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed"
    })
}

fn decorate_daemon_body(verb: &str, body: &str) -> Result<Value, String> {
    let mut value: Value = serde_json::from_str(body).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        object.insert("status".to_owned(), json!("ok"));
        object.insert("surface".to_owned(), json!("cognitive-cli"));
        object.insert("verb".to_owned(), json!(verb));
        object.insert("profile_claim".to_owned(), json!("not-claimed"));
        object.insert("gate_claim".to_owned(), json!("not-claimed"));
    }
    Ok(value)
}

fn try_connect(
    endpoint_override: Option<&str>,
    layout: &cognitive_store::PersonalDataLayout,
) -> Option<PersonalDaemonClient> {
    let endpoint = match endpoint_override {
        Some(value) => value.to_owned(),
        None => daemon::load_endpoint(layout).ok()?,
    };
    PersonalDaemonClient::connect(&endpoint, layout).ok()
}

fn connect_required(
    endpoint_override: Option<&str>,
    layout: &cognitive_store::PersonalDataLayout,
) -> Result<PersonalDaemonClient, String> {
    try_connect(endpoint_override, layout).ok_or_else(|| {
        "restore --archive-id requires a running Personal daemon; start it or use --archive <dir>"
            .to_owned()
    })
}
