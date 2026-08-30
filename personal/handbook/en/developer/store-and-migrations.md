---
doc_id: dev.store-migrations
locale: en
kind: reference
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-store/src/personal_backup.rs
    symbols: ["write_personal_backup_archive", "restore_personal_backup_archive"]
  - path: personal/crates/cognitive-store/src/personal_db.rs
    symbols: ["authority_migration_plan", "prepare_personal_databases"]
  - path: personal/crates/cognitive-store/src/project_aggregate.rs
    symbols: ["PROJECT_AGGREGATE_SCHEMA_V26", "APPROVAL_PREVIEW_NARROW_SCHEMA_V29", "STANDING_APPROVAL_POLICY_SCHEMA_V30", "ProjectAggregateStore"]
  - path: personal/crates/cognitive-store/src/employee.rs
    symbols: ["EMPLOYEE_SCHEMA_V27", "EmployeeStore", "HandoffSpec"]
  - path: personal/crates/cognitive-store/src/conversation.rs
    symbols: ["CONVERSATION_ARCHIVE_SCHEMA_V28", "ConversationStore", "CONVERSATION_ARCHIVE_PROJECTION_ID", "ArchiveReadSpec", "ArchiveAppendSpec"]
  - path: personal/crates/cognitive-store/src/assistant.rs
    symbols: ["AssistantPlane", "AssistantTurnSpec", "ASSISTANT_ENGINE_ID", "ASSISTANT_PI_PIN"]
  - path: personal/crates/cognitive-store/src/hosted_dsh.rs
    symbols: ["HOSTED_DSH_SCHEMA_V31", "HostedDshPlane", "HostedDshStartSpec", "HOSTED_DSH_ENGINE_ID"]
  - path: personal/crates/cognitive-store/src/vault.rs
    symbols: ["VAULT_SCHEMA_V32", "VaultStore", "VaultImportSpec", "CONTEXT_INJECT_ORDER", "VAULT_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/migration.rs
    symbols: ["execute_sqlite_migration_plan"]
  - path: personal/crates/cognitive-store/src/provider_control_plane.rs
    symbols: ["honest_usage_read_model", "labelled_cost_source", "honest_unknown_cost", "replace_binding"]
  - path: personal/crates/cognitive-store/src/sqlite/store.rs
    symbols: ["SqliteAuthorityStore"]
  - path: personal/crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: personal/crates/cognitive-store/src/scheduler.rs
    symbols: ["SchedulerRepository", "acquire_eligible_lease"]
tests:
  - personal/crates/cognitive-store/tests/p1_t01_layout_migrations.rs
  - personal/crates/cognitive-store/tests/p11_t03_project_aggregate.rs
  - personal/crates/cognitive-store/tests/p11_t04_employee.rs
  - personal/crates/cognitive-store/tests/p11_t05_conversation.rs
  - personal/crates/cognitive-store/tests/p11_t06_assistant.rs
  - personal/crates/cognitive-store/tests/p11_t07_hosted_dsh.rs
  - personal/crates/cognitive-store/tests/p11_t10_vault.rs
  - personal/crates/cognitive-store/tests/p11_t09_hitl_canvas.rs
  - personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/m2_acceptance.rs
  - personal/crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:3ef20288e60487d73b9b965336666f59da76b112df98dbd29bcba77783b295b3"
non_claims:
  - Cross-database atomicity between authority and installation SQLite files is explicitly not claimed.
---

# Store and migrations

`cognitive-store` is the single-writer SQLite WAL adapter behind the kernel ports.
`SqliteAuthorityStore` is cloneable: clones share one connection mutex so the
Personal daemon can hand the same writer to HTTP Task admission and the periodic
scheduler tick. Two databases under XDG state: **authority** (migrations v1–v33) and
**installation** (v1–v4). No cross-database atomicity is claimed; preparation
orders authority first and names the backup path on a second-phase failure.

## Authority migration map (v1–v33)

| Versions | Adds |
|---|---|
| v1 | governed objects (CAS rows), append-only events/records, budgets, outbox, intents (idempotency-unique), fencing singleton, checkpoints, user intents, interpretations, task contracts, loop progress facts |
| v2–v3 | scheduler entries; v3 rebuilds to PK `(task_ref, contract_epoch)` preserving leases |
| v4–v9 | operation candidate proposals, daemon operation descriptors + authorization snapshots, worker iteration authorizations (WIA) with one-time consumption and scheduler-lease bindings |
| v10–v11 | fixed post-states, verification requests/reports, continuation authorizations + lease-bound consumption |
| v12–v15 | context requests/views, workspace context sources (role/trust CHECKs), authorization/revocation fact sets, scheduler execution policies |
| v16–v20 | Memory candidates/decisions/objects, FTS5 derived index, tombstones (forget → +expire → +supersede), version lineage |
| v21–v23 | Skill packages/revisions/bindings, binding revocations, revision lineage |
| v24 | append-only Memory/Skill consumption records keyed by Task/epoch/request/session |
| v25 | Provider Control Plane accounts, models, bindings, usage events/aggregates, budgets, alerts, audit |
| v26 | Personal-private Project aggregate (`p11_draft`, `p11_candidate`, `p11_charter_revision`, `p11_project`, `p11_plan_revision`, `p11_stage`, `p11_gap`, `p11_stage_test_fact`, `p11_acceptance_fact`, `p11_approval_preview`). New tables, not `family=task`. |
| v27 | Role Blueprint / Assignment / Employee / Grant (`p11_role_blueprint`, `p11_role_blueprint_revision`, `p11_employee`, `p11_employee_revision`, `p11_assignment`, `p11_install_fact`, `p11_grant`, `p11_speech_audit`, `p11_handoff`). No Provider binding on Blueprint. Employee is the authority id; runtime_binding_ref is replaceable. Handoff rows keep `authority_stays=1`; writers take `HandoffSpec` so chat cannot transfer authority. |
| v28 | Personal-private conversation archive (`p11_conversation_archive`) under new identifier `cognitiveos.personal.conversation-archive/0.1`. Delivered whitelist speech lands a row; owner `append` accepts `note`/`deliverable`/`handoff`/`blocked`/`decision-request`. Chatter stays audit-only. Index requires `limit` 1..=32 and returns refs (record_id + digest), not bodies. ADR-0058 `conversation-projection/0.1` is not coerced. Archive rows are observation-only; a record_id cannot satisfy stage-test completion. |
| v29 | ApprovalPreview `superseded_by` (P11-T09 HITL). Narrow mints a **new** pending preview and freezes the old row as `superseded`. Reject leaves a `receipt_ref`. Stale is mechanical `base_state_digest` mismatch only — not wall-clock freshness. Chat/task cannot confirm, reject, or narrow. |
| v30 | `grant-expansion` subject_kind plus StandingApprovalPolicy time-box (`p11_standing_approval_policy`). `expires_at` is required and ≤7 days. Settings list/revoke is management HTTP. Chat cannot mint. Rebuilds `p11_approval_preview` CHECK. |
| v31 | Hidden hosted DSH managed child (`p11_hosted_dsh_child`). `runtime_binding_ref` binds to `hosted-dsh:<artifact>:<child_id>` (pid/digest/artifact). Process exit clears pid and marks `exited`; it does not delete Employee, conversation archive, or Memory. Isolated spawn fail-closes on Windows GNU. Windows OPC E2E is `not-run`. |
| v32 | Markdown Vault (`p11_vault_document`, rebuildable `p11_vault_index_entry`, `p11_vault_conflict`) under `cognitiveos.personal.markdown-vault/0.1`. Import requires rights/provenance. Files are not Project authority (`is_authority=0`). Index is not Memory FTS. Last-write-wins without a conflict row is rejected. Host filesystem E2E is `not-run`. |
| v33 | Routine revision / Trigger occurrence ledger (`p11_routine`, `p11_routine_revision`, `p11_routine_occurrence`) under `cognitiveos.personal.routine/0.1`. Overlap policy is `no-overlap-queue-latest`. Missed/coalesced rows are visible. Active occurrences reuse `scheduler_entries` (`task://personal/routine/{occurrence_id}`). Checkpoint is not completion. No Temporal / second scheduler table. Clock/sleep/restart E2E is `not-run`. |

P11-T06 Hidden Pi Assistant adds **no new migration**. It reuses v26 `p11_candidate` / `p11_approval_preview` and T05 read-only archive context. Assistant register requires typed provenance (`sources[]` | `owner-stated` | `assistant-assumption`); a non-null blob is rejected. Closed candidate JSON forbids `grant` / `secret` / `trigger-arm`. `draft.apply` targeting a Project/Employee/Grant/confirmed charter is rejected. The assistant plane cannot write archive, SecretStore, Memory, or confirm/apply authority. Default-deny tools; research may name existing `HttpFetchReadOnly` only. Exact Pi `0.81.1` and `cognitiveos.private-candidate/1` are identity pins, not a second scheduler or Installed Agent.

P11-T09 HITL canvas reuses v26 `request_preview` / `confirm_preview` / `p11_approval_preview` plus v29 `superseded_by` and v30 grant-expansion / StandingApprovalPolicy. Management HTTP `preview.reject` / `preview.narrow` / `confirm` / `standing-policy.*` are the durable caller; T05 announce+deep-link only; T06 `draft.apply` is not authority-approve. Host UI E2E is `not-run`. Settings chrome is T13. No second scheduler, no chat Approve, no Inbox L1.

P11-T07 hidden hosted DSH adds v31 `p11_hosted_dsh_child`. The Attempt-runner `start` caller is management HTTP `dsh.hosted.start`; task-channel aliases are 403. Digest/protocol mismatch, env/argv secrets, Pi-as-member-engine, Installed Agent chrome, and unknown child output (`success`/`ok`/`agent_end`) fail closed. Daemon Provider proxy `POST /provider/v1/dsh/chat/completions` remains the only secret-bearing path. Linux Path B is not Windows hosted qualification.

P11-T12 honest usage adds **no new migration**. It is a labelled read of v25 `llm_usage_events` / `agent_provider_bindings` / `provider_accounts`: `cost_label` is `actual` (`provider_reported`+`priced`), `estimated` (`locally_estimated`+`priced`, only when that source was recorded), or `unknown` (never JSON `0`). `GET /management/usage` also returns a four-layer binding explanation; Project/employee/Task layers are explicit `unbound` today. Account identity and quota are separate objects. Silent account/model rebind is rejected. Member-level budget hard-stop is 2.1 / Deferred.

P11-T08 Routine/Trigger adds v33. Management HTTP `routine.revision` / `routine.trigger` / `routine.ledger` / `routine.checkpoint` / `routine.resume` is the real caller. Task-channel aliases are 403. Overlap is rejected or queued as latest; host-unavailable records a visible missed row; stale revision fail-closes; completing from a checkpoint is rejected; consequential auto-resume fail-closes. HITL remains T09 canvas (not Inbox L1). Clock/sleep/restart host E2E is `not-run`.

P11-T10 Markdown Vault adds v32. Management HTTP `vault.import` / `vault.index.rebuild` / `vault.index` / `vault.conflicts` is the real caller. Context inject order is a documented store helper (current Task contract → fixed decisions → sourced excerpts → summaries → older narrative; over-limit drops older narrative first). Vault files cannot confirm/apply Project authority. Memory admission cannot swallow Vault files. Conversation archive and Artifact CAS blobs are not Vault files. Obsidian is not bundled. Host filesystem E2E is `not-run` until `DEV-WINDOWS-NATIVE-OPC-01`.

Nearly every durable table carries BEFORE UPDATE/DELETE triggers that abort with
"append-only"; derived tables are `memory_search_fts` and `p11_vault_index_entry`
(rebuildable; Vault searches do not use Memory FTS).

**Load-bearing nuance**: `SqliteAuthorityStore::open` bootstraps schema constants
v1–v17 only; v18–v33 tables exist only after `prepare_personal_databases` runs the
versioned plan (production paths and P4 tests always do).

## Migration engine

Plans are validated (strictly increasing versions, digest self-consistency) before
any side effect. `DryRun` executes against a `VACUUM INTO` scratch copy; `Apply`
writes a timestamped backup, then runs all pending migrations inside **one**
immediate transaction with recorded-row digest verification, replay-skip safety,
and a `PRAGMA quick_check` gate before commit. Preparation holds an exclusive
`migration.lock` (stale lock after a crash requires manual removal).

## Concurrency model

One `Mutex<Connection>` per store instance; WAL + `synchronous=FULL` asserted at
open; read-only opens model degraded volumes (writes fail closed as
`STATE_STORE_UNAVAILABLE`, reads and replay stay alive). Scheduler leases are
transactional CAS: eligibility requires `runnable` past `next_eligible` or an
expired lease reclaimed at a strictly higher epoch; release demands the exact
`(owner, epoch)`; consumption of WIA/continuation authority is bound to the exact
active leased row in the same transaction.

Task admission reuses those existing v1–v3 tables; it adds no migration or
parallel scheduler. `insert_task_contract_with_execution_bootstrap` repeats the
writer-fence and contract-epoch CAS inside one immediate authority transaction,
then inserts the TaskContract event, registered `START` Loop admission/event,
its governed Task projection at registered `DRAFT` without a second
`(object_id, INITIAL)` event, hard Budget, and
`(task_ref, contract_epoch)` runnable scheduler row. A conflict in any late
member rolls the earlier inserts back; a crash after a successful commit
reopens all five prerequisites. Startup recovery can idempotently repair an
older current contract missing only Task, Loop, Budget, or scheduler work in one
fenced transaction. Existing rows are validated and never replaced or reset;
stale contract epochs cannot be repaired.

Verification start reuses the existing fixed-post-state/request tables and adds
no migration. One immediate transaction verifies the writer, current contract,
closed Effect version, shared row bindings, and Loop CAS, then inserts both
append-only rows and commits `ACT -> VERIFY`; any late conflict rolls everything
back.

Verified Task completion adds no migration or dedicated acceptance table.
Canonical decision bytes live in Artifact CAS. Two immediate transactions reuse
the existing governed-object/event/transition-record tables and recheck current
contract, exact fixed state, newest report, complete closed Effect set, and
fencing before the candidate and final acceptance CAS updates.

Resource Manager list/inspect helpers (`list_non_tombstoned_memory_objects`,
`load_non_tombstoned_memory_object`, `list_skill_bindings`) are inherent store
reads over those same v16–v23 rows. They add no migration and invent no seventh
family table.

## User backup archive

`write_personal_backup_archive` copies config/data/state/artifact files into a
digest-bound directory archive and writes a Memory/Skill export sidecar. It
skips `authority.sqlite`, secret-named paths, and `provider-config.json`.
Restore preflight checks schema, completeness, and part digests, then overlays
live files from a staging tree with snapshot rollback. Focused tests record
byte-equal restored files and a finite restore wall time as hypothesis-only
facts. This is not a SQLite dump and does not claim Gate/RTO/RPO results.
