//! Memory/Skill 受治理消费记录的版本化迁移。

use crate::migration::MigrationPlanEntry;

/// 迁移 v24：按 Task/epoch/request/session 绑定的只追加消费记录。
pub const MEMORY_SKILL_CONSUMPTION_SCHEMA_V24: &str = "
CREATE TABLE memory_skill_consumption_records (
  consumption_id TEXT PRIMARY KEY,
  task_ref TEXT NOT NULL CHECK (length(trim(task_ref)) > 0),
  contract_epoch INTEGER NOT NULL CHECK (contract_epoch >= 1),
  context_request_id TEXT NOT NULL CHECK (length(trim(context_request_id)) > 0),
  context_request_digest TEXT NOT NULL CHECK (length(trim(context_request_digest)) > 0),
  session_ref TEXT NOT NULL CHECK (length(trim(session_ref)) > 0),
  reuse_of TEXT REFERENCES memory_skill_consumption_records(consumption_id),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE UNIQUE INDEX memory_skill_consumption_session_binding
  ON memory_skill_consumption_records(task_ref, contract_epoch, context_request_id, session_ref);
CREATE TRIGGER memory_skill_consumption_records_append_only_update
BEFORE UPDATE ON memory_skill_consumption_records
BEGIN SELECT RAISE(ABORT, 'append-only: Memory/Skill consumption record is immutable'); END;
CREATE TRIGGER memory_skill_consumption_records_append_only_delete
BEFORE DELETE ON memory_skill_consumption_records
BEGIN SELECT RAISE(ABORT, 'append-only: Memory/Skill consumption record is immutable'); END;
";

/// 版本 24 Memory/Skill 消费记录迁移项。
pub fn memory_skill_consumption_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(24, MEMORY_SKILL_CONSUMPTION_SCHEMA_V24)
}
