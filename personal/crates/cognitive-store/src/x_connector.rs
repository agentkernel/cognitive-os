//! Personal-private X/Twitter connector walking skeleton (P11-T14,
//! authority migration v35).
//!
//! SecretStore-only account bind, rights-safe original content, digest-bound
//! preview, HITL confirm, persist-before-dispatch publish ledger, and honest
//! unknown readback. Live X/Twitter API E2E remains `Requires-environment` /
//! `not-run`. Not P0 hero chrome, not a business-result promise, not a second
//! credential plane.

use crate::employee::EmployeeStore;
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private X connector envelope (P11-T14). Hidden capability, not chrome.
pub const X_CONNECTOR_PROJECTION_ID: &str = "cognitiveos.personal.x-connector/0.1";

/// Authority migration v35: X connector account / preview / publish ledger.
pub const X_CONNECTOR_SCHEMA_V35: &str = "
CREATE TABLE p11_x_connector_account (
  account_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL UNIQUE REFERENCES p11_project(project_id),
  handle TEXT NOT NULL,
  secret_ref TEXT NOT NULL,
  consent TEXT NOT NULL CHECK (consent = 'owner-per-source'),
  is_p0_hero INTEGER NOT NULL CHECK (is_p0_hero = 0),
  platform_qualified INTEGER NOT NULL CHECK (platform_qualified = 0),
  created_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_x_connector_preview (
  preview_id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL REFERENCES p11_x_connector_account(account_id),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
  rights_attestation TEXT NOT NULL CHECK (rights_attestation = 'original-owner-rights'),
  content_kind TEXT NOT NULL CHECK (content_kind = 'original'),
  confirmed INTEGER NOT NULL CHECK (confirmed IN (0,1)),
  created_at INTEGER NOT NULL,
  confirmed_at INTEGER
) STRICT;
CREATE INDEX p11_x_connector_preview_account
  ON p11_x_connector_preview(account_id, created_at);
CREATE TABLE p11_x_connector_publish (
  publish_id TEXT PRIMARY KEY,
  preview_id TEXT NOT NULL UNIQUE REFERENCES p11_x_connector_preview(preview_id),
  account_id TEXT NOT NULL REFERENCES p11_x_connector_account(account_id),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  intent_persisted INTEGER NOT NULL CHECK (intent_persisted = 1),
  dispatched INTEGER NOT NULL CHECK (dispatched IN (0,1)),
  readback_status TEXT NOT NULL CHECK (readback_status IN ('unknown','observed')),
  impressions TEXT NOT NULL CHECK (impressions = 'unknown'),
  completion_claimed INTEGER NOT NULL CHECK (completion_claimed = 0),
  created_at INTEGER NOT NULL
) STRICT;
";

/// v35 migration entry.
pub fn x_connector_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(35, X_CONNECTOR_SCHEMA_V35)
}

/// Account bind input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XConnectorBindSpec<'a> {
    pub project_id: &'a str,
    pub handle: &'a str,
    pub secret_ref: &'a str,
    pub consent: &'a str,
    pub argv: &'a [&'a str],
    pub env_pairs: &'a [(&'a str, &'a str)],
    pub hero_claim: bool,
    pub default_demo: bool,
    pub p0_success_path: bool,
    pub platform_qualified_claim: bool,
    pub evasion: bool,
    pub now_ms: i64,
}

/// Preview request input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XConnectorPreviewSpec<'a> {
    pub account_id: &'a str,
    pub project_id: &'a str,
    pub content: &'a str,
    pub content_kind: &'a str,
    pub rights_attestation: &'a str,
    pub evasion: bool,
    pub chat_approve: bool,
    pub now_ms: i64,
}

/// HITL confirm input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XConnectorConfirmSpec<'a> {
    pub preview_id: &'a str,
    pub expected_digest: &'a str,
    pub chat_approve: bool,
    pub now_ms: i64,
}

/// Persist-before-dispatch publish input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XConnectorDispatchSpec<'a> {
    pub preview_id: &'a str,
    pub claim_complete: bool,
    pub impressions: Option<&'a str>,
    pub now_ms: i64,
}

/// Bound account (never includes the SecretStore handle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XConnectorAccount {
    pub account_id: String,
    pub project_id: String,
    pub handle: String,
}

/// Digest-bound preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XConnectorPreview {
    pub preview_id: String,
    pub account_id: String,
    pub project_id: String,
    pub content_digest: String,
    pub confirmed: bool,
}

/// Publish ledger row. Receipt is not completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XConnectorPublish {
    pub publish_id: String,
    pub preview_id: String,
    pub intent_persisted: bool,
    pub dispatched: bool,
    pub readback_status: String,
    pub impressions: String,
    pub receipt_is_not_completion: bool,
}

/// Redacted connector status. Never contains secret material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XConnectorStatus {
    pub account_id: String,
    pub project_id: String,
    pub handle: String,
    pub is_p0_hero: bool,
    pub platform_qualified: bool,
    pub preview_id: Option<String>,
    pub confirmed: bool,
    pub dispatched: bool,
    pub readback_status: String,
    pub impressions: String,
    pub receipt_is_not_completion: bool,
}

/// Personal-private X connector store on the authority writer.
#[derive(Clone)]
pub struct XConnectorStore {
    conn: Arc<Mutex<Connection>>,
}

impl XConnectorStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// Bind one owner-consented account per Project. SecretStore ref only.
    pub fn bind_account(
        &self,
        caller: ConfirmCaller,
        spec: &XConnectorBindSpec<'_>,
    ) -> Result<XConnectorAccount, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        reject_secret_material(spec.argv, spec.env_pairs)?;
        reject_evasion(spec.evasion, spec.argv, spec.env_pairs, spec.handle)?;
        reject_hero_path(spec.hero_claim, spec.default_demo, spec.p0_success_path)?;
        if spec.platform_qualified_claim {
            return Err(ProjectAggregateError::Rejected {
                detail: "Linux/CI must not claim X platform qualification",
            });
        }
        if spec.consent != "owner-per-source" {
            return Err(ProjectAggregateError::Rejected {
                detail: "account bind requires owner-per-source consent",
            });
        }
        require_secret_ref(spec.secret_ref)?;
        require_handle(spec.handle)?;

        let conn = self.lock()?;
        let project_found: Option<String> = conn
            .query_row(
                "SELECT project_id FROM p11_project WHERE project_id = ?1",
                params![spec.project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("lookup project"))?;
        if project_found.is_none() {
            return Err(ProjectAggregateError::NotFound {
                detail: "project not found",
            });
        }
        let existing: Option<String> = conn
            .query_row(
                "SELECT account_id FROM p11_x_connector_account WHERE project_id = ?1",
                params![spec.project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("lookup account"))?;
        if existing.is_some() {
            return Err(ProjectAggregateError::Conflict {
                detail: "duplicate X connector account",
            });
        }
        let account_id = next_id("xacct")?;
        conn.execute(
            "INSERT INTO p11_x_connector_account (
               account_id, project_id, handle, secret_ref, consent,
               is_p0_hero, platform_qualified, created_at
             ) VALUES (?1, ?2, ?3, ?4, 'owner-per-source', 0, 0, ?5)",
            params![
                account_id,
                spec.project_id,
                spec.handle,
                spec.secret_ref,
                spec.now_ms
            ],
        )
        .map_err(unavailable("insert account"))?;
        Ok(XConnectorAccount {
            account_id,
            project_id: spec.project_id.to_owned(),
            handle: spec.handle.to_owned(),
        })
    }

    /// Digest-bound preview of original rights-safe content.
    pub fn request_preview(
        &self,
        caller: ConfirmCaller,
        spec: &XConnectorPreviewSpec<'_>,
    ) -> Result<XConnectorPreview, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if spec.chat_approve {
            return Err(ProjectAggregateError::Rejected {
                detail: "chat Approve is forbidden",
            });
        }
        if spec.evasion {
            return Err(ProjectAggregateError::Rejected {
                detail: "fingerprint/CAPTCHA/anti-abuse evasion is forbidden",
            });
        }
        reject_evasion_text(spec.content)?;
        if secret_shaped_value(spec.content) {
            return Err(ProjectAggregateError::Rejected {
                detail: "secret-shaped material is rejected at registration",
            });
        }
        if spec.content.trim().is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "preview content is required",
            });
        }
        if spec.content_kind != "original" {
            return Err(ProjectAggregateError::Rejected {
                detail: "scraped or stolen content is rejected",
            });
        }
        if spec.rights_attestation != "original-owner-rights" {
            return Err(ProjectAggregateError::Rejected {
                detail: "scraped or stolen content is rejected",
            });
        }
        let content_digest = digest_hex(spec.content.as_bytes());
        let conn = self.lock()?;
        let account: Option<(String, String)> = conn
            .query_row(
                "SELECT account_id, project_id FROM p11_x_connector_account
                  WHERE account_id = ?1",
                params![spec.account_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("lookup account"))?;
        let Some((account_id, project_id)) = account else {
            return Err(ProjectAggregateError::NotFound {
                detail: "X connector account not found",
            });
        };
        if project_id != spec.project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project X connector write is rejected",
            });
        }
        let preview_id = next_id("xprev")?;
        conn.execute(
            "INSERT INTO p11_x_connector_preview (
               preview_id, account_id, project_id, content_digest,
               rights_attestation, content_kind, confirmed, created_at, confirmed_at
             ) VALUES (?1, ?2, ?3, ?4, 'original-owner-rights', 'original', 0, ?5, NULL)",
            params![
                preview_id,
                account_id,
                project_id,
                content_digest,
                spec.now_ms
            ],
        )
        .map_err(unavailable("insert preview"))?;
        Ok(XConnectorPreview {
            preview_id,
            account_id,
            project_id,
            content_digest,
            confirmed: false,
        })
    }

    /// Owner-management HITL confirm. Chat Approve is rejected.
    pub fn confirm_preview(
        &self,
        caller: ConfirmCaller,
        spec: &XConnectorConfirmSpec<'_>,
    ) -> Result<XConnectorPreview, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if spec.chat_approve {
            return Err(ProjectAggregateError::Rejected {
                detail: "chat Approve is forbidden",
            });
        }
        let conn = self.lock()?;
        let row: Option<(String, String, String, String, i64)> = conn
            .query_row(
                "SELECT preview_id, account_id, project_id, content_digest, confirmed
                   FROM p11_x_connector_preview WHERE preview_id = ?1",
                params![spec.preview_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(unavailable("lookup preview"))?;
        let Some((preview_id, account_id, project_id, content_digest, confirmed)) = row else {
            return Err(ProjectAggregateError::NotFound {
                detail: "X connector preview not found",
            });
        };
        if content_digest != spec.expected_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "preview digest mismatch",
            });
        }
        if confirmed == 0 {
            conn.execute(
                "UPDATE p11_x_connector_preview
                    SET confirmed = 1, confirmed_at = ?1
                  WHERE preview_id = ?2",
                params![spec.now_ms, preview_id],
            )
            .map_err(unavailable("confirm preview"))?;
        }
        Ok(XConnectorPreview {
            preview_id,
            account_id,
            project_id,
            content_digest,
            confirmed: true,
        })
    }

    /// Persist the publish Intent, then mark dispatched. Receipt is not completion.
    pub fn dispatch_publish(
        &self,
        caller: ConfirmCaller,
        spec: &XConnectorDispatchSpec<'_>,
    ) -> Result<XConnectorPublish, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if spec.claim_complete {
            return Err(ProjectAggregateError::Rejected {
                detail: "receipt is not completion",
            });
        }
        if let Some(impressions) = spec.impressions {
            if impressions == "0" || impressions == "0.0" || impressions.is_empty() {
                return Err(ProjectAggregateError::Rejected {
                    detail: "unknown metrics must not serialize as 0",
                });
            }
            if impressions != "unknown" {
                return Err(ProjectAggregateError::Rejected {
                    detail: "unknown metrics must not serialize as 0",
                });
            }
        }
        let conn = self.lock()?;
        let preview: Option<(String, String, String, i64)> = conn
            .query_row(
                "SELECT preview_id, account_id, project_id, confirmed
                   FROM p11_x_connector_preview WHERE preview_id = ?1",
                params![spec.preview_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("lookup preview"))?;
        let Some((preview_id, account_id, project_id, confirmed)) = preview else {
            return Err(ProjectAggregateError::NotFound {
                detail: "X connector preview not found",
            });
        };
        if confirmed != 1 {
            return Err(ProjectAggregateError::Unconfirmed {
                detail: "publish without HITL confirm is rejected",
            });
        }
        let existing: Option<String> = conn
            .query_row(
                "SELECT publish_id FROM p11_x_connector_publish WHERE preview_id = ?1",
                params![preview_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("lookup publish"))?;
        if existing.is_some() {
            return Err(ProjectAggregateError::Conflict {
                detail: "duplicate X connector publish",
            });
        }
        let publish_id = next_id("xpub")?;
        conn.execute(
            "INSERT INTO p11_x_connector_publish (
               publish_id, preview_id, account_id, project_id,
               intent_persisted, dispatched, readback_status, impressions,
               completion_claimed, created_at
             ) VALUES (?1, ?2, ?3, ?4, 1, 0, 'unknown', 'unknown', 0, ?5)",
            params![publish_id, preview_id, account_id, project_id, spec.now_ms],
        )
        .map_err(unavailable("persist publish intent"))?;
        conn.execute(
            "UPDATE p11_x_connector_publish
                SET dispatched = 1
              WHERE publish_id = ?1 AND intent_persisted = 1",
            params![publish_id],
        )
        .map_err(unavailable("mark dispatched"))?;
        Ok(XConnectorPublish {
            publish_id,
            preview_id,
            intent_persisted: true,
            dispatched: true,
            readback_status: "unknown".to_owned(),
            impressions: "unknown".to_owned(),
            receipt_is_not_completion: true,
        })
    }

    /// Redacted status. Secret refs never leave this projection.
    pub fn status(
        &self,
        caller: ConfirmCaller,
        account_id: &str,
    ) -> Result<XConnectorStatus, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        let account: Option<(String, String, String)> = conn
            .query_row(
                "SELECT account_id, project_id, handle
                   FROM p11_x_connector_account WHERE account_id = ?1",
                params![account_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("lookup account"))?;
        let Some((account_id, project_id, handle)) = account else {
            return Err(ProjectAggregateError::NotFound {
                detail: "X connector account not found",
            });
        };
        let preview: Option<(String, i64)> = conn
            .query_row(
                "SELECT preview_id, confirmed FROM p11_x_connector_preview
                  WHERE account_id = ?1
                  ORDER BY created_at DESC LIMIT 1",
                params![account_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("lookup preview"))?;
        let (preview_id, confirmed) = match preview {
            Some((id, flag)) => (Some(id), flag == 1),
            None => (None, false),
        };
        let publish: Option<(i64, String, String)> = match &preview_id {
            Some(id) => conn
                .query_row(
                    "SELECT dispatched, readback_status, impressions
                       FROM p11_x_connector_publish WHERE preview_id = ?1",
                    params![id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(unavailable("lookup publish"))?,
            None => None,
        };
        let (dispatched, readback_status, impressions) = match publish {
            Some((flag, readback, impressions)) => (flag == 1, readback, impressions),
            None => (false, "unknown".to_owned(), "unknown".to_owned()),
        };
        Ok(XConnectorStatus {
            account_id,
            project_id,
            handle,
            is_p0_hero: false,
            platform_qualified: false,
            preview_id,
            confirmed,
            dispatched,
            readback_status,
            impressions,
            receipt_is_not_completion: true,
        })
    }
}

fn require_secret_ref(secret_ref: &str) -> Result<(), ProjectAggregateError> {
    if !secret_ref.starts_with("secretref:") || secret_ref.len() <= "secretref:".len() {
        return Err(ProjectAggregateError::Rejected {
            detail: "raw secret must not enter env, argv, or bind body",
        });
    }
    let handle = &secret_ref["secretref:".len()..];
    if handle.chars().any(char::is_whitespace) || secret_shaped_value(secret_ref) {
        return Err(ProjectAggregateError::Rejected {
            detail: "raw secret must not enter env, argv, or bind body",
        });
    }
    Ok(())
}

fn require_handle(handle: &str) -> Result<(), ProjectAggregateError> {
    if handle.is_empty() || secret_shaped_value(handle) || evasion_shaped(handle) {
        return Err(ProjectAggregateError::Invalid {
            detail: "X handle is invalid",
        });
    }
    Ok(())
}

fn reject_hero_path(
    hero_claim: bool,
    default_demo: bool,
    p0_success_path: bool,
) -> Result<(), ProjectAggregateError> {
    if hero_claim || default_demo || p0_success_path {
        return Err(ProjectAggregateError::Rejected {
            detail: "X is not a P0 hero or default demo Project",
        });
    }
    Ok(())
}

fn reject_evasion(
    evasion: bool,
    argv: &[&str],
    env_pairs: &[(&str, &str)],
    extra: &str,
) -> Result<(), ProjectAggregateError> {
    if evasion {
        return Err(ProjectAggregateError::Rejected {
            detail: "fingerprint/CAPTCHA/anti-abuse evasion is forbidden",
        });
    }
    for arg in argv {
        reject_evasion_text(arg)?;
    }
    for (key, value) in env_pairs {
        reject_evasion_text(key)?;
        reject_evasion_text(value)?;
    }
    reject_evasion_text(extra)
}

fn reject_evasion_text(value: &str) -> Result<(), ProjectAggregateError> {
    if evasion_shaped(value) {
        return Err(ProjectAggregateError::Rejected {
            detail: "fingerprint/CAPTCHA/anti-abuse evasion is forbidden",
        });
    }
    Ok(())
}

fn evasion_shaped(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("captcha")
        || lowered.contains("fingerprint")
        || lowered.contains("anti-abuse")
        || lowered.contains("anti_abuse")
        || lowered.contains("stealth")
        || lowered.contains("puppeteer-extra")
        || lowered.contains("turnstile")
}

fn reject_secret_material(
    argv: &[&str],
    env_pairs: &[(&str, &str)],
) -> Result<(), ProjectAggregateError> {
    for (key, value) in env_pairs {
        if secret_shaped_key(key) || secret_shaped_value(value) {
            return Err(ProjectAggregateError::Rejected {
                detail: "raw secret must not enter env, argv, or bind body",
            });
        }
    }
    for arg in argv {
        if secret_shaped_key(arg) || secret_shaped_value(arg) {
            return Err(ProjectAggregateError::Rejected {
                detail: "raw secret must not enter env, argv, or bind body",
            });
        }
    }
    Ok(())
}

fn secret_shaped_key(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("password")
        || lowered.contains("authorization")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("api-key")
        || lowered.contains("oauth")
        || lowered.contains("bearer")
}

fn secret_shaped_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
        || lowered.contains("oauth_token")
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn next_id(prefix: &str) -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("{prefix}-{generated}"))
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
