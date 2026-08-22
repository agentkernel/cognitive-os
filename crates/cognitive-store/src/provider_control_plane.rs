//! Provider Control Plane durable records (P8-T13, authority migration v25).
//!
//! Named accounts, model catalog, agent bindings, usage events/aggregates,
//! budgets, alerts, and redacted audit facts. Secret material is never a
//! column. This is a private Personal projection, not a public contract.

use crate::migration::MigrationPlanEntry;
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Authority migration v25: Provider Control Plane tables.
pub const PROVIDER_CONTROL_PLANE_SCHEMA_V25: &str = "
CREATE TABLE provider_accounts (
  account_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0 AND length(display_name) <= 64),
  provider_kind TEXT NOT NULL CHECK (provider_kind IN ('openai_official','anthropic_official','openai_compatible')),
  endpoint TEXT NOT NULL CHECK (length(trim(endpoint)) > 0 AND length(endpoint) <= 512),
  secret_ref TEXT CHECK (secret_ref IS NULL OR (length(secret_ref) > 0 AND length(secret_ref) <= 256)),
  allow_private_network INTEGER NOT NULL CHECK (allow_private_network IN (0,1)),
  allow_insecure_http INTEGER NOT NULL CHECK (allow_insecure_http IN (0,1)),
  network_scope TEXT NOT NULL CHECK (network_scope IN ('loopback','private','public')),
  status TEXT NOT NULL CHECK (status IN ('active','degraded','revoked')),
  catalog_revision INTEGER NOT NULL DEFAULT 0 CHECK (catalog_revision >= 0),
  last_discovery_error TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;
CREATE UNIQUE INDEX provider_accounts_display_name
  ON provider_accounts(display_name);
CREATE TABLE provider_models (
  account_id TEXT NOT NULL REFERENCES provider_accounts(account_id),
  model_id TEXT NOT NULL CHECK (length(trim(model_id)) > 0 AND length(model_id) <= 256),
  source TEXT NOT NULL CHECK (source IN ('provider_discovered','manually_configured')),
  pricing_version TEXT,
  price_input_per_million TEXT,
  price_output_per_million TEXT,
  price_cache_read_per_million TEXT,
  price_cache_write_per_million TEXT,
  catalog_revision INTEGER NOT NULL CHECK (catalog_revision >= 0),
  PRIMARY KEY (account_id, model_id)
) STRICT;
CREATE TABLE agent_provider_bindings (
  agent_instance_id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL REFERENCES provider_accounts(account_id),
  model_id TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1),
  status TEXT NOT NULL CHECK (status IN ('active','revoked')),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;
CREATE TABLE llm_usage_events (
  event_id TEXT PRIMARY KEY,
  idempotency_key TEXT NOT NULL,
  recorded_at_ms INTEGER NOT NULL,
  account_id TEXT NOT NULL,
  provider_kind TEXT NOT NULL,
  model_id TEXT NOT NULL,
  agent_instance_id TEXT NOT NULL,
  input_tokens INTEGER,
  output_tokens INTEGER,
  cache_read_tokens INTEGER,
  cache_write_tokens INTEGER,
  duration_ms INTEGER,
  outcome TEXT NOT NULL,
  metering_source TEXT NOT NULL CHECK (metering_source IN ('provider_reported','locally_estimated','unavailable')),
  estimation_method TEXT,
  pricing_version TEXT,
  cost_micros INTEGER,
  cost_status TEXT NOT NULL CHECK (cost_status IN ('priced','cost_unavailable')),
  cache_hit_rate_unknown INTEGER NOT NULL CHECK (cache_hit_rate_unknown IN (0,1))
) STRICT;
CREATE UNIQUE INDEX llm_usage_events_idempotency
  ON llm_usage_events(idempotency_key);
CREATE TABLE llm_usage_aggregates (
  period_start_ms INTEGER NOT NULL,
  period_kind TEXT NOT NULL,
  account_id TEXT NOT NULL,
  agent_instance_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  input_tokens INTEGER,
  output_tokens INTEGER,
  cache_read_tokens INTEGER,
  cache_write_tokens INTEGER,
  cost_micros INTEGER,
  priced_call_count INTEGER NOT NULL,
  unavailable_cost_call_count INTEGER NOT NULL,
  PRIMARY KEY (period_start_ms, period_kind, account_id, agent_instance_id, model_id)
) STRICT;
CREATE TABLE llm_budgets (
  budget_id TEXT PRIMARY KEY,
  scope_kind TEXT NOT NULL CHECK (scope_kind IN ('account','agent')),
  scope_id TEXT NOT NULL,
  period_kind TEXT NOT NULL CHECK (period_kind IN ('calendar_month')),
  token_limit INTEGER,
  amount_micros_limit INTEGER,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(scope_kind, scope_id, period_kind)
) STRICT;
CREATE TABLE llm_alerts (
  alert_id TEXT PRIMARY KEY,
  budget_id TEXT NOT NULL,
  threshold_kind TEXT NOT NULL CHECK (threshold_kind IN ('warning_80','exceeded_100')),
  period_start_ms INTEGER NOT NULL,
  issued_at_ms INTEGER NOT NULL,
  acknowledged_at_ms INTEGER,
  UNIQUE(budget_id, threshold_kind, period_start_ms)
) STRICT;
CREATE TABLE llm_audit_events (
  audit_id TEXT PRIMARY KEY,
  recorded_at_ms INTEGER NOT NULL,
  action TEXT NOT NULL,
  account_id TEXT,
  agent_instance_id TEXT,
  outcome TEXT NOT NULL,
  detail TEXT NOT NULL
) STRICT;
";

/// Thirty days in milliseconds.
pub const USAGE_EVENT_RETENTION_MS: i64 = 30 * 24 * 60 * 60 * 1000;
/// Ninety days in milliseconds.
pub const USAGE_AGGREGATE_RETENTION_MS: i64 = 90 * 24 * 60 * 60 * 1000;

/// Versioned built-in prices: USD per million tokens, as decimal text.
pub const BUILTIN_PRICE_TABLE_VERSION: &str = "builtin-2026-08";

/// v25 migration entry.
pub fn provider_control_plane_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(25, PROVIDER_CONTROL_PLANE_SCHEMA_V25)
}

/// Failures from the Provider Control Plane store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderControlPlaneError {
    Unavailable { detail: String },
    Conflict { detail: &'static str },
    NotFound { detail: &'static str },
    Invalid { detail: &'static str },
}

impl std::fmt::Display for ProviderControlPlaneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable { detail } => {
                write!(
                    formatter,
                    "provider control plane store unavailable: {detail}"
                )
            }
            Self::Conflict { detail } => {
                write!(formatter, "provider control plane conflict: {detail}")
            }
            Self::NotFound { detail } => {
                write!(formatter, "provider control plane not found: {detail}")
            }
            Self::Invalid { detail } => {
                write!(formatter, "provider control plane invalid: {detail}")
            }
        }
    }
}

impl std::error::Error for ProviderControlPlaneError {}

/// Durable Provider Control Plane records on the authority SQLite writer.
#[derive(Clone)]
pub struct ProviderControlPlaneStore {
    conn: Arc<Mutex<Connection>>,
}

/// Account row (never contains API key material).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAccountRecord {
    pub account_id: String,
    pub display_name: String,
    pub provider_kind: String,
    pub endpoint: String,
    pub secret_ref: Option<String>,
    pub allow_private_network: bool,
    pub allow_insecure_http: bool,
    pub network_scope: String,
    pub status: String,
    pub catalog_revision: i64,
    pub last_discovery_error: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Catalog row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelRecord {
    pub account_id: String,
    pub model_id: String,
    pub source: String,
    pub pricing_version: Option<String>,
    pub price_input_per_million: Option<String>,
    pub price_output_per_million: Option<String>,
    pub price_cache_read_per_million: Option<String>,
    pub price_cache_write_per_million: Option<String>,
    pub catalog_revision: i64,
}

/// Fixed agent binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingRecord {
    pub agent_instance_id: String,
    pub account_id: String,
    pub model_id: String,
    pub revision: i64,
    pub status: String,
}

/// Four-category usage sample. `None` means unknown, never zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageSample {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
}

/// Cost outcome. Unknown is never disguised as zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostOutcome {
    pub cost_micros: Option<i64>,
    pub cost_status: &'static str,
    pub pricing_version: Option<String>,
    pub cache_hit_rate_unknown: bool,
}

/// New usage event to persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUsageEvent {
    pub event_id: String,
    pub idempotency_key: String,
    pub recorded_at_ms: i64,
    pub account_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub agent_instance_id: String,
    pub sample: UsageSample,
    pub duration_ms: Option<i64>,
    pub outcome: String,
    pub metering_source: String,
    pub estimation_method: Option<String>,
    pub cost: CostOutcome,
}

impl ProviderControlPlaneStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProviderControlPlaneError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("pragma"))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProviderControlPlaneError> {
        self.conn
            .lock()
            .map_err(|_| ProviderControlPlaneError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// Insert a new account. `secret_ref` is opaque or absent.
    pub fn insert_account(
        &self,
        record: &ProviderAccountRecord,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO provider_accounts (
                account_id, display_name, provider_kind, endpoint, secret_ref,
                allow_private_network, allow_insecure_http, network_scope, status,
                catalog_revision, last_discovery_error, created_at_ms, updated_at_ms
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                record.account_id,
                record.display_name,
                record.provider_kind,
                record.endpoint,
                record.secret_ref,
                i64::from(record.allow_private_network),
                i64::from(record.allow_insecure_http),
                record.network_scope,
                record.status,
                record.catalog_revision,
                record.last_discovery_error,
                record.created_at_ms,
                record.updated_at_ms
            ],
        )
        .map_err(|error| map_insert_conflict(error, "account name already exists"))?;
        Ok(())
    }

    pub fn get_account(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccountRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT account_id, display_name, provider_kind, endpoint, secret_ref,
                    allow_private_network, allow_insecure_http, network_scope, status,
                    catalog_revision, last_discovery_error, created_at_ms, updated_at_ms
               FROM provider_accounts WHERE account_id = ?1",
            [account_id],
            map_account_row,
        )
        .optional()
        .map_err(unavailable("get account"))
    }

    pub fn get_account_by_name(
        &self,
        display_name: &str,
    ) -> Result<Option<ProviderAccountRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT account_id, display_name, provider_kind, endpoint, secret_ref,
                    allow_private_network, allow_insecure_http, network_scope, status,
                    catalog_revision, last_discovery_error, created_at_ms, updated_at_ms
               FROM provider_accounts WHERE display_name = ?1",
            [display_name],
            map_account_row,
        )
        .optional()
        .map_err(unavailable("get account by name"))
    }

    pub fn list_accounts(&self) -> Result<Vec<ProviderAccountRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT account_id, display_name, provider_kind, endpoint, secret_ref,
                        allow_private_network, allow_insecure_http, network_scope, status,
                        catalog_revision, last_discovery_error, created_at_ms, updated_at_ms
                   FROM provider_accounts ORDER BY display_name",
            )
            .map_err(unavailable("list accounts"))?;
        let rows = statement
            .query_map([], map_account_row)
            .map_err(unavailable("list accounts query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list accounts rows"))
    }

    pub fn update_account_endpoint_trust(
        &self,
        account_id: &str,
        endpoint: &str,
        allow_private_network: bool,
        allow_insecure_http: bool,
        network_scope: &str,
        updated_at_ms: i64,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE provider_accounts
                    SET endpoint = ?1, allow_private_network = ?2, allow_insecure_http = ?3,
                        network_scope = ?4, updated_at_ms = ?5
                  WHERE account_id = ?6",
                params![
                    endpoint,
                    i64::from(allow_private_network),
                    i64::from(allow_insecure_http),
                    network_scope,
                    updated_at_ms,
                    account_id
                ],
            )
            .map_err(unavailable("update account endpoint"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "account not found",
            });
        }
        Ok(())
    }

    pub fn update_account_secret_and_status(
        &self,
        account_id: &str,
        secret_ref: Option<&str>,
        status: &str,
        updated_at_ms: i64,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE provider_accounts SET secret_ref = ?1, status = ?2, updated_at_ms = ?3
                  WHERE account_id = ?4",
                params![secret_ref, status, updated_at_ms, account_id],
            )
            .map_err(unavailable("update account secret"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "account not found",
            });
        }
        Ok(())
    }

    pub fn mark_discovery_outcome(
        &self,
        account_id: &str,
        status: &str,
        catalog_revision: i64,
        last_discovery_error: Option<&str>,
        updated_at_ms: i64,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE provider_accounts
                SET status = ?1, catalog_revision = ?2, last_discovery_error = ?3, updated_at_ms = ?4
              WHERE account_id = ?5",
            params![
                status,
                catalog_revision,
                last_discovery_error,
                updated_at_ms,
                account_id
            ],
        )
        .map_err(unavailable("mark discovery"))?;
        Ok(())
    }

    pub fn delete_account(&self, account_id: &str) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let active_bindings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_provider_bindings
                  WHERE account_id = ?1 AND status = 'active'",
                [account_id],
                |row| row.get(0),
            )
            .map_err(unavailable("count bindings"))?;
        if active_bindings > 0 {
            return Err(ProviderControlPlaneError::Conflict {
                detail: "account has an active agent binding",
            });
        }
        conn.execute(
            "DELETE FROM provider_models WHERE account_id = ?1",
            [account_id],
        )
        .map_err(unavailable("delete models"))?;
        let changed = conn
            .execute(
                "DELETE FROM provider_accounts WHERE account_id = ?1",
                [account_id],
            )
            .map_err(unavailable("delete account"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "account not found",
            });
        }
        Ok(())
    }

    /// Replace discovered models for a revision. Manual models are preserved.
    pub fn replace_discovered_models(
        &self,
        account_id: &str,
        catalog_revision: i64,
        models: &[ProviderModelRecord],
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let tx = conn
            .transaction()
            .map_err(unavailable("begin catalog tx"))?;
        tx.execute(
            "DELETE FROM provider_models
              WHERE account_id = ?1 AND source = 'provider_discovered'",
            [account_id],
        )
        .map_err(unavailable("clear discovered"))?;
        for model in models {
            tx.execute(
                "INSERT INTO provider_models (
                    account_id, model_id, source, pricing_version,
                    price_input_per_million, price_output_per_million,
                    price_cache_read_per_million, price_cache_write_per_million,
                    catalog_revision
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(account_id, model_id) DO UPDATE SET
                    source = excluded.source,
                    pricing_version = excluded.pricing_version,
                    catalog_revision = excluded.catalog_revision
                 WHERE provider_models.source = 'provider_discovered'",
                params![
                    account_id,
                    model.model_id,
                    model.source,
                    model.pricing_version,
                    model.price_input_per_million,
                    model.price_output_per_million,
                    model.price_cache_read_per_million,
                    model.price_cache_write_per_million,
                    catalog_revision
                ],
            )
            .map_err(unavailable("insert discovered model"))?;
        }
        tx.commit().map_err(unavailable("commit catalog"))?;
        Ok(())
    }

    pub fn upsert_manual_model(
        &self,
        model: &ProviderModelRecord,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO provider_models (
                account_id, model_id, source, pricing_version,
                price_input_per_million, price_output_per_million,
                price_cache_read_per_million, price_cache_write_per_million,
                catalog_revision
             ) VALUES (?1,?2,'manually_configured',?3,?4,?5,?6,?7,?8)
             ON CONFLICT(account_id, model_id) DO UPDATE SET
                source = 'manually_configured',
                pricing_version = excluded.pricing_version,
                price_input_per_million = excluded.price_input_per_million,
                price_output_per_million = excluded.price_output_per_million,
                price_cache_read_per_million = excluded.price_cache_read_per_million,
                price_cache_write_per_million = excluded.price_cache_write_per_million,
                catalog_revision = excluded.catalog_revision",
            params![
                model.account_id,
                model.model_id,
                model.pricing_version,
                model.price_input_per_million,
                model.price_output_per_million,
                model.price_cache_read_per_million,
                model.price_cache_write_per_million,
                model.catalog_revision
            ],
        )
        .map_err(unavailable("upsert manual model"))?;
        Ok(())
    }

    pub fn set_model_prices(
        &self,
        account_id: &str,
        model_id: &str,
        pricing_version: &str,
        input: Option<&str>,
        output: Option<&str>,
        cache_read: Option<&str>,
        cache_write: Option<&str>,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE provider_models SET
                    pricing_version = ?1,
                    price_input_per_million = ?2,
                    price_output_per_million = ?3,
                    price_cache_read_per_million = ?4,
                    price_cache_write_per_million = ?5
                  WHERE account_id = ?6 AND model_id = ?7",
                params![
                    pricing_version,
                    input,
                    output,
                    cache_read,
                    cache_write,
                    account_id,
                    model_id
                ],
            )
            .map_err(unavailable("set prices"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "model not found",
            });
        }
        Ok(())
    }

    pub fn list_models(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderModelRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT account_id, model_id, source, pricing_version,
                        price_input_per_million, price_output_per_million,
                        price_cache_read_per_million, price_cache_write_per_million,
                        catalog_revision
                   FROM provider_models WHERE account_id = ?1 ORDER BY model_id",
            )
            .map_err(unavailable("list models"))?;
        let rows = statement
            .query_map([account_id], map_model_row)
            .map_err(unavailable("list models query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list models rows"))
    }

    pub fn get_model(
        &self,
        account_id: &str,
        model_id: &str,
    ) -> Result<Option<ProviderModelRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT account_id, model_id, source, pricing_version,
                    price_input_per_million, price_output_per_million,
                    price_cache_read_per_million, price_cache_write_per_million,
                    catalog_revision
               FROM provider_models WHERE account_id = ?1 AND model_id = ?2",
            params![account_id, model_id],
            map_model_row,
        )
        .optional()
        .map_err(unavailable("get model"))
    }

    pub fn set_binding(
        &self,
        binding: &AgentProviderBindingRecord,
        now_ms: i64,
    ) -> Result<AgentProviderBindingRecord, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let existing_revision: Option<i64> = conn
            .query_row(
                "SELECT revision FROM agent_provider_bindings WHERE agent_instance_id = ?1",
                [&binding.agent_instance_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("read binding"))?;
        let revision = existing_revision.unwrap_or(0) + 1;
        conn.execute(
            "INSERT INTO agent_provider_bindings (
                agent_instance_id, account_id, model_id, revision, status, created_at_ms, updated_at_ms
             ) VALUES (?1,?2,?3,?4,'active',?5,?5)
             ON CONFLICT(agent_instance_id) DO UPDATE SET
                account_id = excluded.account_id,
                model_id = excluded.model_id,
                revision = excluded.revision,
                status = 'active',
                updated_at_ms = excluded.updated_at_ms",
            params![
                binding.agent_instance_id,
                binding.account_id,
                binding.model_id,
                revision,
                now_ms
            ],
        )
        .map_err(unavailable("set binding"))?;
        Ok(AgentProviderBindingRecord {
            agent_instance_id: binding.agent_instance_id.clone(),
            account_id: binding.account_id.clone(),
            model_id: binding.model_id.clone(),
            revision,
            status: "active".to_owned(),
        })
    }

    pub fn get_active_binding(
        &self,
        agent_instance_id: &str,
    ) -> Result<Option<AgentProviderBindingRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT agent_instance_id, account_id, model_id, revision, status
               FROM agent_provider_bindings
              WHERE agent_instance_id = ?1 AND status = 'active'",
            [agent_instance_id],
            map_binding_row,
        )
        .optional()
        .map_err(unavailable("get binding"))
    }

    pub fn list_bindings(
        &self,
    ) -> Result<Vec<AgentProviderBindingRecord>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT agent_instance_id, account_id, model_id, revision, status
                   FROM agent_provider_bindings ORDER BY agent_instance_id",
            )
            .map_err(unavailable("list bindings"))?;
        let rows = statement
            .query_map([], map_binding_row)
            .map_err(unavailable("list bindings query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list bindings rows"))
    }

    pub fn remove_binding(
        &self,
        agent_instance_id: &str,
        now_ms: i64,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE agent_provider_bindings SET status = 'revoked', updated_at_ms = ?1
                  WHERE agent_instance_id = ?2 AND status = 'active'",
                params![now_ms, agent_instance_id],
            )
            .map_err(unavailable("remove binding"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "active binding not found",
            });
        }
        Ok(())
    }

    /// Record a usage event. Duplicate idempotency keys return the stored row
    /// without double-counting aggregates.
    pub fn record_usage(&self, event: &NewUsageEvent) -> Result<bool, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO llm_usage_events (
                event_id, idempotency_key, recorded_at_ms, account_id, provider_kind,
                model_id, agent_instance_id, input_tokens, output_tokens,
                cache_read_tokens, cache_write_tokens, duration_ms, outcome,
                metering_source, estimation_method, pricing_version, cost_micros,
                cost_status, cache_hit_rate_unknown
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
                params![
                    event.event_id,
                    event.idempotency_key,
                    event.recorded_at_ms,
                    event.account_id,
                    event.provider_kind,
                    event.model_id,
                    event.agent_instance_id,
                    event.sample.input_tokens,
                    event.sample.output_tokens,
                    event.sample.cache_read_tokens,
                    event.sample.cache_write_tokens,
                    event.duration_ms,
                    event.outcome,
                    event.metering_source,
                    event.estimation_method,
                    event.cost.pricing_version,
                    event.cost.cost_micros,
                    event.cost.cost_status,
                    i64::from(event.cost.cache_hit_rate_unknown)
                ],
            )
            .map_err(unavailable("insert usage"))?;
        if inserted == 0 {
            return Ok(false);
        }
        let period_start = calendar_month_start_ms(event.recorded_at_ms);
        let priced = i64::from(event.cost.cost_status == "priced");
        let unavailable = i64::from(event.cost.cost_status != "priced");
        conn.execute(
            "INSERT INTO llm_usage_aggregates (
                period_start_ms, period_kind, account_id, agent_instance_id, model_id,
                input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                cost_micros, priced_call_count, unavailable_cost_call_count
             ) VALUES (?1,'calendar_month',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
             ON CONFLICT(period_start_ms, period_kind, account_id, agent_instance_id, model_id)
             DO UPDATE SET
                input_tokens = CASE
                    WHEN excluded.input_tokens IS NULL THEN llm_usage_aggregates.input_tokens
                    ELSE COALESCE(llm_usage_aggregates.input_tokens, 0) + excluded.input_tokens
                  END,
                output_tokens = CASE
                    WHEN excluded.output_tokens IS NULL THEN llm_usage_aggregates.output_tokens
                    ELSE COALESCE(llm_usage_aggregates.output_tokens, 0) + excluded.output_tokens
                  END,
                cache_read_tokens = CASE
                    WHEN excluded.cache_read_tokens IS NULL THEN llm_usage_aggregates.cache_read_tokens
                    ELSE COALESCE(llm_usage_aggregates.cache_read_tokens, 0) + excluded.cache_read_tokens
                  END,
                cache_write_tokens = CASE
                    WHEN excluded.cache_write_tokens IS NULL THEN llm_usage_aggregates.cache_write_tokens
                    ELSE COALESCE(llm_usage_aggregates.cache_write_tokens, 0) + excluded.cache_write_tokens
                  END,
                cost_micros = CASE
                    WHEN excluded.cost_micros IS NULL THEN llm_usage_aggregates.cost_micros
                    ELSE COALESCE(llm_usage_aggregates.cost_micros, 0) + excluded.cost_micros
                  END,
                priced_call_count = llm_usage_aggregates.priced_call_count + excluded.priced_call_count,
                unavailable_cost_call_count = llm_usage_aggregates.unavailable_cost_call_count
                    + excluded.unavailable_cost_call_count",
            params![
                period_start,
                event.account_id,
                event.agent_instance_id,
                event.model_id,
                event.sample.input_tokens,
                event.sample.output_tokens,
                event.sample.cache_read_tokens,
                event.sample.cache_write_tokens,
                event.cost.cost_micros,
                priced,
                unavailable
            ],
        )
        .map_err(unavailable("upsert aggregate"))?;
        Ok(true)
    }

    pub fn list_usage_events(
        &self,
        since_ms: i64,
    ) -> Result<Vec<(String, String, Option<i64>, String)>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT event_id, account_id, cost_micros, cost_status
                   FROM llm_usage_events WHERE recorded_at_ms >= ?1 ORDER BY recorded_at_ms",
            )
            .map_err(unavailable("list usage"))?;
        let rows = statement
            .query_map([since_ms], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(unavailable("list usage query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list usage rows"))
    }

    pub fn apply_retention(
        &self,
        now_ms: i64,
    ) -> Result<(usize, usize), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let event_cutoff = now_ms.saturating_sub(USAGE_EVENT_RETENTION_MS);
        let aggregate_cutoff = now_ms.saturating_sub(USAGE_AGGREGATE_RETENTION_MS);
        let events = conn
            .execute(
                "DELETE FROM llm_usage_events WHERE recorded_at_ms < ?1",
                [event_cutoff],
            )
            .map_err(unavailable("retain events"))?;
        let aggregates = conn
            .execute(
                "DELETE FROM llm_usage_aggregates WHERE period_start_ms < ?1",
                [aggregate_cutoff],
            )
            .map_err(unavailable("retain aggregates"))?;
        conn.execute(
            "DELETE FROM llm_audit_events WHERE recorded_at_ms < ?1",
            [event_cutoff],
        )
        .map_err(unavailable("retain audit"))?;
        Ok((events, aggregates))
    }

    pub fn upsert_budget(
        &self,
        budget_id: &str,
        scope_kind: &str,
        scope_id: &str,
        token_limit: Option<i64>,
        amount_micros_limit: Option<i64>,
        now_ms: i64,
    ) -> Result<(), ProviderControlPlaneError> {
        if token_limit.is_none() && amount_micros_limit.is_none() {
            return Err(ProviderControlPlaneError::Invalid {
                detail: "budget requires a token or amount limit",
            });
        }
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO llm_budgets (
                budget_id, scope_kind, scope_id, period_kind, token_limit, amount_micros_limit, created_at_ms
             ) VALUES (?1,?2,?3,'calendar_month',?4,?5,?6)
             ON CONFLICT(scope_kind, scope_id, period_kind) DO UPDATE SET
                token_limit = excluded.token_limit,
                amount_micros_limit = excluded.amount_micros_limit",
            params![
                budget_id,
                scope_kind,
                scope_id,
                token_limit,
                amount_micros_limit,
                now_ms
            ],
        )
        .map_err(unavailable("upsert budget"))?;
        Ok(())
    }

    pub fn list_budgets(
        &self,
    ) -> Result<Vec<(String, String, String, Option<i64>, Option<i64>)>, ProviderControlPlaneError>
    {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT budget_id, scope_kind, scope_id, token_limit, amount_micros_limit
                   FROM llm_budgets ORDER BY scope_kind, scope_id",
            )
            .map_err(unavailable("list budgets"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(unavailable("list budgets query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list budgets rows"))
    }

    pub fn remove_budget(&self, budget_id: &str) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let changed = conn
            .execute("DELETE FROM llm_budgets WHERE budget_id = ?1", [budget_id])
            .map_err(unavailable("remove budget"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "budget not found",
            });
        }
        Ok(())
    }

    /// Observe-only 80%/100% alerts. Duplicate (budget, threshold, period) is ignored.
    pub fn maybe_issue_budget_alerts(
        &self,
        now_ms: i64,
    ) -> Result<Vec<(String, String)>, ProviderControlPlaneError> {
        let budgets = self.list_budgets()?;
        let period_start = calendar_month_start_ms(now_ms);
        let mut issued = Vec::new();
        let conn = self.lock()?;
        for (budget_id, scope_kind, scope_id, token_limit, amount_limit) in budgets {
            let (tokens, cost_micros, unavailable_cost_calls): (i64, Option<i64>, i64) =
                if scope_kind == "account" {
                    conn.query_row(
                    "SELECT COALESCE(SUM(COALESCE(input_tokens,0)+COALESCE(output_tokens,0)
                        +COALESCE(cache_read_tokens,0)+COALESCE(cache_write_tokens,0)),0),
                            SUM(cost_micros),
                            COALESCE(SUM(unavailable_cost_call_count),0)
                       FROM llm_usage_aggregates
                      WHERE period_start_ms = ?1 AND period_kind = 'calendar_month' AND account_id = ?2",
                    params![period_start, scope_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(unavailable("sum account usage"))?
                } else {
                    conn.query_row(
                    "SELECT COALESCE(SUM(COALESCE(input_tokens,0)+COALESCE(output_tokens,0)
                        +COALESCE(cache_read_tokens,0)+COALESCE(cache_write_tokens,0)),0),
                            SUM(cost_micros),
                            COALESCE(SUM(unavailable_cost_call_count),0)
                       FROM llm_usage_aggregates
                      WHERE period_start_ms = ?1 AND period_kind = 'calendar_month' AND agent_instance_id = ?2",
                    params![period_start, scope_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(unavailable("sum agent usage"))?
                };
            let token_ratio_bps = token_limit
                .filter(|limit| *limit > 0)
                .and_then(|limit| tokens.checked_mul(10_000).map(|scaled| scaled / limit));
            let amount_ratio_bps = match (amount_limit, cost_micros, unavailable_cost_calls) {
                (Some(limit), Some(cost), 0) if limit > 0 => {
                    cost.checked_mul(10_000).map(|scaled| scaled / limit)
                }
                _ => None,
            };
            let ratio_bps = match (token_ratio_bps, amount_ratio_bps) {
                (Some(left), Some(right)) => Some(left.max(right)),
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                (None, None) => None,
            };
            let Some(ratio_bps) = ratio_bps else {
                continue;
            };
            if ratio_bps >= 10_000 {
                if insert_alert(&conn, &budget_id, "exceeded_100", period_start, now_ms)? {
                    issued.push((budget_id.clone(), "exceeded_100".to_owned()));
                }
            } else if ratio_bps >= 8_000
                && insert_alert(&conn, &budget_id, "warning_80", period_start, now_ms)?
            {
                issued.push((budget_id, "warning_80".to_owned()));
            }
        }
        Ok(issued)
    }

    pub fn list_alerts(
        &self,
    ) -> Result<Vec<(String, String, String, i64, Option<i64>)>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT alert_id, budget_id, threshold_kind, issued_at_ms, acknowledged_at_ms
                   FROM llm_alerts ORDER BY issued_at_ms",
            )
            .map_err(unavailable("list alerts"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(unavailable("list alerts query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list alerts rows"))
    }

    pub fn acknowledge_alert(
        &self,
        alert_id: &str,
        now_ms: i64,
    ) -> Result<(), ProviderControlPlaneError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE llm_alerts SET acknowledged_at_ms = ?1
                  WHERE alert_id = ?2 AND acknowledged_at_ms IS NULL",
                params![now_ms, alert_id],
            )
            .map_err(unavailable("ack alert"))?;
        if changed == 0 {
            return Err(ProviderControlPlaneError::NotFound {
                detail: "alert not found or already acknowledged",
            });
        }
        Ok(())
    }

    pub fn append_audit(
        &self,
        audit_id: &str,
        now_ms: i64,
        action: &str,
        account_id: Option<&str>,
        agent_instance_id: Option<&str>,
        outcome: &str,
        detail: &str,
    ) -> Result<(), ProviderControlPlaneError> {
        if looks_like_secret(detail) {
            return Err(ProviderControlPlaneError::Invalid {
                detail: "audit detail must not contain secret-shaped material",
            });
        }
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO llm_audit_events (
                audit_id, recorded_at_ms, action, account_id, agent_instance_id, outcome, detail
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                audit_id,
                now_ms,
                action,
                account_id,
                agent_instance_id,
                outcome,
                detail
            ],
        )
        .map_err(unavailable("append audit"))?;
        Ok(())
    }

    pub fn list_audit(
        &self,
        since_ms: i64,
    ) -> Result<Vec<(String, String, String, String)>, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT audit_id, action, outcome, detail
                   FROM llm_audit_events WHERE recorded_at_ms >= ?1 ORDER BY recorded_at_ms",
            )
            .map_err(unavailable("list audit"))?;
        let rows = statement
            .query_map([since_ms], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(unavailable("list audit query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list audit rows"))
    }

    /// Scan every text column in control-plane tables for a forbidden token.
    pub fn leak_scan_contains(&self, needle: &str) -> Result<bool, ProviderControlPlaneError> {
        let conn = self.lock()?;
        let tables = [
            "SELECT secret_ref, last_discovery_error, display_name, endpoint FROM provider_accounts",
            "SELECT model_id, pricing_version FROM provider_models",
            "SELECT agent_instance_id, model_id FROM agent_provider_bindings",
            "SELECT event_id, idempotency_key, outcome, estimation_method FROM llm_usage_events",
            "SELECT detail, action, outcome FROM llm_audit_events",
        ];
        for sql in tables {
            let mut statement = conn.prepare(sql).map_err(unavailable("leak scan"))?;
            let column_count = statement.column_count();
            let mut rows = statement
                .query([])
                .map_err(unavailable("leak scan query"))?;
            while let Some(row) = rows.next().map_err(unavailable("leak scan row"))? {
                for index in 0..column_count {
                    let value: Option<String> = match row.get(index) {
                        Ok(value) => value,
                        Err(_) => None,
                    };
                    if value.is_some_and(|text| text.contains(needle)) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
}

/// Compute cost. Unknown tokens are never treated as zero.
pub fn compute_cost(sample: &UsageSample, model: Option<&ProviderModelRecord>) -> CostOutcome {
    let cache_hit_rate_unknown = match (sample.input_tokens, sample.cache_read_tokens) {
        (Some(input), Some(cache_read)) if input >= cache_read && input > 0 => false,
        _ => true,
    };
    let Some(model) = model else {
        return CostOutcome {
            cost_micros: None,
            cost_status: "cost_unavailable",
            pricing_version: None,
            cache_hit_rate_unknown,
        };
    };
    let Some((input, output)) = sample.input_tokens.zip(sample.output_tokens) else {
        return CostOutcome {
            cost_micros: None,
            cost_status: "cost_unavailable",
            pricing_version: model.pricing_version.clone(),
            cache_hit_rate_unknown,
        };
    };
    let Some(input_price) = model
        .price_input_per_million
        .as_deref()
        .and_then(parse_price)
    else {
        return CostOutcome {
            cost_micros: None,
            cost_status: "cost_unavailable",
            pricing_version: model.pricing_version.clone(),
            cache_hit_rate_unknown,
        };
    };
    let Some(output_price) = model
        .price_output_per_million
        .as_deref()
        .and_then(parse_price)
    else {
        return CostOutcome {
            cost_micros: None,
            cost_status: "cost_unavailable",
            pricing_version: model.pricing_version.clone(),
            cache_hit_rate_unknown,
        };
    };
    let mut micros = price_to_micros(input, input_price) + price_to_micros(output, output_price);
    if let Some(cache_read) = sample.cache_read_tokens {
        match model
            .price_cache_read_per_million
            .as_deref()
            .and_then(parse_price)
        {
            Some(price) => micros += price_to_micros(cache_read, price),
            None => {
                return CostOutcome {
                    cost_micros: None,
                    cost_status: "cost_unavailable",
                    pricing_version: model.pricing_version.clone(),
                    cache_hit_rate_unknown,
                };
            }
        }
    }
    if let Some(cache_write) = sample.cache_write_tokens {
        match model
            .price_cache_write_per_million
            .as_deref()
            .and_then(parse_price)
        {
            Some(price) => micros += price_to_micros(cache_write, price),
            None => {
                return CostOutcome {
                    cost_micros: None,
                    cost_status: "cost_unavailable",
                    pricing_version: model.pricing_version.clone(),
                    cache_hit_rate_unknown,
                };
            }
        }
    }
    CostOutcome {
        cost_micros: Some(micros),
        cost_status: "priced",
        pricing_version: model.pricing_version.clone(),
        cache_hit_rate_unknown,
    }
}

/// Apply the built-in official price table when the model has no manual prices.
pub fn apply_builtin_prices(kind: &str, model: &mut ProviderModelRecord) {
    if model.price_input_per_million.is_some() {
        return;
    }
    let Some((input, output, cache_read, cache_write)) = builtin_prices(kind, &model.model_id)
    else {
        return;
    };
    model.pricing_version = Some(BUILTIN_PRICE_TABLE_VERSION.to_owned());
    model.price_input_per_million = Some(input.to_owned());
    model.price_output_per_million = Some(output.to_owned());
    model.price_cache_read_per_million = Some(cache_read.to_owned());
    model.price_cache_write_per_million = Some(cache_write.to_owned());
}

/// Normalize OpenAI-style usage JSON. Missing fields stay unknown.
pub fn usage_from_openai_json(value: &serde_json::Value) -> UsageSample {
    let usage = value.get("usage").unwrap_or(value);
    UsageSample {
        input_tokens: integer_field(usage, "prompt_tokens"),
        output_tokens: integer_field(usage, "completion_tokens"),
        cache_read_tokens: usage
            .get("prompt_tokens_details")
            .and_then(|details| integer_field(details, "cached_tokens")),
        cache_write_tokens: None,
    }
}

/// Normalize Anthropic-style usage JSON.
pub fn usage_from_anthropic_json(value: &serde_json::Value) -> UsageSample {
    let usage = value.get("usage").unwrap_or(value);
    UsageSample {
        input_tokens: integer_field(usage, "input_tokens"),
        output_tokens: integer_field(usage, "output_tokens"),
        cache_read_tokens: integer_field(usage, "cache_read_input_tokens"),
        cache_write_tokens: integer_field(usage, "cache_creation_input_tokens"),
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn builtin_prices(
    kind: &str,
    model_id: &str,
) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    match (kind, model_id) {
        ("openai_official", "gpt-4o") => Some(("2.50", "10.00", "1.25", "2.50")),
        ("openai_official", "gpt-4o-mini") => Some(("0.15", "0.60", "0.075", "0.15")),
        ("openai_official", "gpt-4.1") => Some(("2.00", "8.00", "0.50", "2.00")),
        ("anthropic_official", "claude-sonnet-4-20250514")
        | ("anthropic_official", "claude-3-5-sonnet-20241022") => {
            Some(("3.00", "15.00", "0.30", "3.75"))
        }
        ("anthropic_official", "claude-3-5-haiku-20241022") => {
            Some(("0.80", "4.00", "0.08", "1.00"))
        }
        _ => None,
    }
}

fn parse_price(text: &str) -> Option<i64> {
    // USD per million tokens as integer micro-USD per million, no floats.
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.starts_with('-') {
        return None;
    }
    let (whole, frac) = match trimmed.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (trimmed, ""),
    };
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if frac.bytes().any(|byte| !byte.is_ascii_digit()) {
        return None;
    }
    let whole_value: i64 = whole.parse().ok()?;
    let mut frac_value = 0_i64;
    let mut place = 100_000_i64;
    for digit in frac.as_bytes().iter().copied().take(6) {
        frac_value += i64::from(digit - b'0') * place;
        place /= 10;
    }
    whole_value.checked_mul(1_000_000)?.checked_add(frac_value)
}

fn price_to_micros(tokens: i64, micro_usd_per_million: i64) -> i64 {
    tokens.saturating_mul(micro_usd_per_million) / 1_000_000
}

fn integer_field(value: &serde_json::Value, name: &str) -> Option<i64> {
    match value.get(name)? {
        serde_json::Value::Number(number) => number.as_i64(),
        serde_json::Value::Null => None,
        _ => None,
    }
}

fn calendar_month_start_ms(unix_ms: i64) -> i64 {
    let seconds = unix_ms.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    // Civil date from Unix days (Howard Hinnant algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    civil_ymd_ms(year, month, 1)
}

fn civil_ymd_ms(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days * 86_400 * 1000
}

fn insert_alert(
    conn: &Connection,
    budget_id: &str,
    threshold_kind: &str,
    period_start_ms: i64,
    now_ms: i64,
) -> Result<bool, ProviderControlPlaneError> {
    let alert_id = format!("{budget_id}:{threshold_kind}:{period_start_ms}");
    let changed = conn
        .execute(
            "INSERT OR IGNORE INTO llm_alerts (
                alert_id, budget_id, threshold_kind, period_start_ms, issued_at_ms, acknowledged_at_ms
             ) VALUES (?1,?2,?3,?4,?5,NULL)",
            params![alert_id, budget_id, threshold_kind, period_start_ms, now_ms],
        )
        .map_err(unavailable("insert alert"))?;
    Ok(changed > 0)
}

fn looks_like_secret(detail: &str) -> bool {
    let lowered = detail.to_ascii_lowercase();
    lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
}

fn map_account_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderAccountRecord> {
    Ok(ProviderAccountRecord {
        account_id: row.get(0)?,
        display_name: row.get(1)?,
        provider_kind: row.get(2)?,
        endpoint: row.get(3)?,
        secret_ref: row.get(4)?,
        allow_private_network: row.get::<_, i64>(5)? != 0,
        allow_insecure_http: row.get::<_, i64>(6)? != 0,
        network_scope: row.get(7)?,
        status: row.get(8)?,
        catalog_revision: row.get(9)?,
        last_discovery_error: row.get(10)?,
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}

fn map_model_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderModelRecord> {
    Ok(ProviderModelRecord {
        account_id: row.get(0)?,
        model_id: row.get(1)?,
        source: row.get(2)?,
        pricing_version: row.get(3)?,
        price_input_per_million: row.get(4)?,
        price_output_per_million: row.get(5)?,
        price_cache_read_per_million: row.get(6)?,
        price_cache_write_per_million: row.get(7)?,
        catalog_revision: row.get(8)?,
    })
}

fn map_binding_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentProviderBindingRecord> {
    Ok(AgentProviderBindingRecord {
        agent_instance_id: row.get(0)?,
        account_id: row.get(1)?,
        model_id: row.get(2)?,
        revision: row.get(3)?,
        status: row.get(4)?,
    })
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProviderControlPlaneError {
    move |error| ProviderControlPlaneError::Unavailable {
        detail: format!("{operation}: {error}"),
    }
}

fn map_insert_conflict(
    error: rusqlite::Error,
    conflict: &'static str,
) -> ProviderControlPlaneError {
    match error {
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            ProviderControlPlaneError::Conflict { detail: conflict }
        }
        other => unavailable("insert account")(other),
    }
}
