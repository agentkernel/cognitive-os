//! P2-T27 public backup/restore over the management channel.
//!
//! The daemon writes secret-excluding archives. Task callers cannot mutate
//! backup state. Raw authority SQLite files are never copied.

use cognitive_store::{
    BackupRestoreOptions, PersonalDataLayout, preflight_personal_backup_archive,
    restore_personal_backup_archive_with_options, write_personal_backup_archive,
};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UserBackupChannel {
    Management,
    Task,
}

#[derive(Debug)]
pub(crate) struct UserBackupResponse {
    pub status: u16,
    pub body: String,
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    channel: UserBackupChannel,
) -> UserBackupResponse {
    if channel == UserBackupChannel::Task {
        return error(
            403,
            "RESOURCE_BACKUP_CHANNEL_FORBIDDEN",
            "backup and restore are management-channel only",
        );
    }
    let path = method_path.split_whitespace().nth(1).unwrap_or_default();
    if method_path.starts_with("POST ")
        && path.starts_with("/management/resource/v1/backup/preflight")
    {
        return preflight(layout, body);
    }
    if method_path.starts_with("POST ") && path.starts_with("/management/resource/v1/backup") {
        return backup(layout);
    }
    if method_path.starts_with("POST ") && path.starts_with("/management/resource/v1/restore") {
        return restore(layout, body);
    }
    error(404, "RESOURCE_BACKUP_NOT_FOUND", "unknown backup route")
}

fn backup(layout: &PersonalDataLayout) -> UserBackupResponse {
    match write_personal_backup_archive(layout) {
        Ok(receipt) => ok(json!({
            "schema": receipt.schema,
            "archive_id": receipt.archive_id,
            "archive_path": receipt.archive_path,
            "manifest_digest": receipt.manifest_digest,
            "export_plan_digest": receipt.export_plan_digest,
            "sqlite_copied": receipt.sqlite_copied,
            "excluded_secret_count": receipt.excluded_secret_count
        })),
        Err(error) => map_store_error(error),
    }
}

fn preflight(layout: &PersonalDataLayout, body: &[u8]) -> UserBackupResponse {
    let archive_id = match parse_archive_id(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let archive_path = layout.backups_dir().join("archives").join(&archive_id);
    match preflight_personal_backup_archive(layout, &archive_path) {
        Ok(receipt) => ok(json!({
            "preflight_only": true,
            "archive_id": archive_id,
            "export_plan_digest": receipt.export_plan_digest,
            "backup_schema_version": receipt.backup_schema_version
        })),
        Err(error) => map_store_error(error),
    }
}

fn restore(layout: &PersonalDataLayout, body: &[u8]) -> UserBackupResponse {
    let archive_id = match parse_archive_id(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let archive_path = layout.backups_dir().join("archives").join(&archive_id);
    match restore_personal_backup_archive_with_options(
        layout,
        &archive_path,
        BackupRestoreOptions {
            apply_live: true,
            inject_fault_before_apply: false,
            refuse_if_daemon_lock: false,
        },
    ) {
        Ok(receipt) => ok(json!({
            "schema": receipt.schema,
            "archive_id": archive_id,
            "restored_path": receipt.restored_path,
            "manifest_digest": receipt.manifest_digest,
            "live_applied": receipt.live_applied
        })),
        Err(error) => map_store_error(error),
    }
}

fn parse_archive_id(body: &[u8]) -> Result<String, UserBackupResponse> {
    let parsed: serde_json::Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(_) => {
            return Err(error(
                400,
                "RESOURCE_BACKUP_QUERY_FORBIDDEN",
                "restore requires JSON {\"archive_id\":\"...\"}",
            ));
        }
    };
    if parsed.get("prompt").is_some()
        || parsed.get("secret").is_some()
        || parsed.get("api_key").is_some()
    {
        return Err(error(
            400,
            "RESOURCE_BACKUP_QUERY_FORBIDDEN",
            "restore body cannot restate secrets or prompts",
        ));
    }
    let Some(archive_id) = parsed.get("archive_id").and_then(|value| value.as_str()) else {
        return Err(error(
            400,
            "RESOURCE_BACKUP_QUERY_FORBIDDEN",
            "restore requires archive_id",
        ));
    };
    if archive_id.trim().is_empty()
        || archive_id.contains("..")
        || archive_id.contains('/')
        || archive_id.contains('\\')
    {
        return Err(error(
            400,
            "RESOURCE_BACKUP_QUERY_FORBIDDEN",
            "archive_id must be a single archive identifier",
        ));
    }
    Ok(archive_id.to_owned())
}

fn map_store_error(store_error: cognitive_store::PersonalBackupError) -> UserBackupResponse {
    let (status, code) = match store_error {
        cognitive_store::PersonalBackupError::ArchiveTampered
        | cognitive_store::PersonalBackupError::ArchiveSecretIncluded
        | cognitive_store::PersonalBackupError::RawSqliteCopyForbidden => {
            (409, "RESOURCE_BACKUP_TAMPERED")
        }
        cognitive_store::PersonalBackupError::SchemaIncompatible => {
            (409, "RESOURCE_BACKUP_SCHEMA_INCOMPATIBLE")
        }
        cognitive_store::PersonalBackupError::IncompleteBackup
        | cognitive_store::PersonalBackupError::ArchiveManifestInvalid => {
            (409, "RESOURCE_BACKUP_INCOMPLETE")
        }
        cognitive_store::PersonalBackupError::RestorePartialRefused => {
            (409, "RESOURCE_BACKUP_PARTIAL_REFUSED")
        }
        cognitive_store::PersonalBackupError::DaemonLockHeld => {
            (409, "RESOURCE_BACKUP_DAEMON_LOCK")
        }
        _ => (400, "RESOURCE_BACKUP_REFUSED"),
    };
    error(status, code, &store_error.to_string())
}

fn ok(value: serde_json::Value) -> UserBackupResponse {
    UserBackupResponse {
        status: 200,
        body: value.to_string(),
    }
}

fn error(status: u16, code: &str, detail: &str) -> UserBackupResponse {
    UserBackupResponse {
        status,
        body: json!({ "error": { "code": code, "detail": detail } }).to_string(),
    }
}
