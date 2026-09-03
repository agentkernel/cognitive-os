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
    symbols: ["EMPLOYEE_SCHEMA_V27", "EmployeeStore", "HandoffSpec", "SecurityReview", "InstallFactRow"]
  - path: personal/crates/cognitive-store/src/conversation.rs
    symbols: ["CONVERSATION_ARCHIVE_SCHEMA_V28", "ConversationStore", "CONVERSATION_ARCHIVE_PROJECTION_ID", "ArchiveReadSpec", "ArchiveAppendSpec"]
  - path: personal/crates/cognitive-store/src/assistant.rs
    symbols: ["AssistantPlane", "AssistantTurnSpec", "ASSISTANT_ENGINE_ID", "ASSISTANT_PI_PIN"]
  - path: personal/crates/cognitive-store/src/hosted_dsh.rs
    symbols: ["HOSTED_DSH_SCHEMA_V31", "HostedDshPlane", "HostedDshStartSpec", "HOSTED_DSH_ENGINE_ID"]
  - path: personal/crates/cognitive-store/src/hosted_dsh_attempt.rs
    symbols: ["HOSTED_DSH_ATTEMPT_SCHEMA_V36", "HostedDshAttemptStore", "HOSTED_ATTEMPT_PROJECTION_ID", "HostedAttemptIntentSpec", "HostedAttemptTerminalSpec", "HostedArtifactObservation"]
  - path: personal/crates/cognitive-store/src/vault.rs
    symbols: ["VAULT_SCHEMA_V32", "VaultStore", "VaultImportSpec", "CONTEXT_INJECT_ORDER", "VAULT_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/routine.rs
    symbols: ["ROUTINE_SCHEMA_V33", "RoutineStore", "ROUTINE_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/windows_host.rs
    symbols: ["WINDOWS_HOST_SCHEMA_V34", "WindowsHostStore", "WINDOWS_HOST_PROJECTION_ID", "WAKE_RECOVERY_STEPS"]
  - path: personal/crates/cognitive-store/src/x_connector.rs
    symbols: ["X_CONNECTOR_SCHEMA_V35", "XConnectorStore", "X_CONNECTOR_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/attempt_artifacts.rs
    symbols: ["ATTEMPT_ARTIFACT_SCHEMA_V37", "AttemptArtifactStore"]
  - path: personal/crates/cognitive-store/src/routine_arming.rs
    symbols: ["ROUTINE_ARMING_SCHEMA_V38", "RoutineArmingStore"]
  - path: personal/crates/cognitive-store/src/project_chat.rs
    symbols: ["PROJECT_CHAT_SCHEMA_V39", "ProjectChatStore"]
  - path: personal/crates/cognitive-store/src/reflection.rs
    symbols: ["REFLECTION_SCHEMA_V40", "ReflectionStore"]
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
  - personal/crates/cognitive-store/tests/p13_t10_capability_acquisition.rs
  - personal/crates/cognitive-store/tests/p11_t05_conversation.rs
  - personal/crates/cognitive-store/tests/p11_t06_assistant.rs
  - personal/crates/cognitive-store/tests/p11_t07_hosted_dsh.rs
  - personal/crates/cognitive-store/tests/p13_t02_hosted_dsh_attempt.rs
  - personal/crates/cognitive-store/tests/p13_t11_reflection.rs
  - personal/crates/cognitive-store/tests/p11_t10_vault.rs
  - personal/crates/cognitive-store/tests/p11_t08_routine.rs
  - personal/crates/cognitive-store/tests/p11_t02_windows_host.rs
  - personal/crates/cognitive-store/tests/p11_t14_x_connector.rs
  - personal/crates/cognitive-store/tests/p11_t09_hitl_canvas.rs
  - personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/m2_acceptance.rs
  - personal/crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:0b2676be425888062901d901a2bb169e8583e98403b17be5518d6661b47a0be5"
non_claims:
  - Cross-database atomicity between authority and installation SQLite files is explicitly not claimed.
---

# Store and migrations

`cognitive-store` is the single-writer SQLite WAL adapter behind the kernel ports.
`SqliteAuthorityStore` is cloneable: clones share one connection mutex so the
Personal daemon can hand the same writer to HTTP Task admission and the periodic
scheduler tick. Two databases under XDG state: **authority** (migrations v1–v40) and
**installation** (v1–v4). No cross-database atomicity is claimed; preparation
orders authority first and names the backup path on a second-phase failure.

## Authority migration map (v1–v40)

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
| v34 | Windows host Personal Home / lifecycle / missed / ordered recovery (`p11_windows_host_home`, `p11_windows_host_daemon`, `p11_windows_host_dsh_child`, `p11_windows_host_offline_segment`, `p11_windows_host_recovery`, `p11_windows_host_restore_point`) under `cognitiveos.personal.windows-host/0.1`. Layout is `Personal Home/app/` + `Personal Home/data/`; upgrade replaces app and preserves data. Tray observes and requests; it does not write authority. Close background-or-pause is rejected unless the daemon can honor it. Same-disk versions are local restore points, not backups. Native tray/ACL/sleep/SecretStore E2E is `not-run`. |
| v35 | X/Twitter connector account / preview / publish ledger (`p11_x_connector_account`, `p11_x_connector_preview`, `p11_x_connector_publish`) under `cognitiveos.personal.x-connector/0.1`. SecretStore `secret_ref` only. `is_p0_hero` and `platform_qualified` CHECK=0. Impressions stay the literal `unknown`. Receipt is not completion. Live X API E2E is `not-run`. |
| v36 | Hosted DSH real Attempt loop (`p13_hosted_dsh_artifact_fact`, `p13_hosted_dsh_attempt`, `p13_hosted_dsh_attempt_frame`) under `cognitiveos.personal.hosted-dsh-attempt/0.1` (P13-T02). Artifact facts are append-only with derived kind `health-check` / `update` / `rollback` and health `pinned` / `absent` / `corrupt` / `mismatch` / `script-missing`; only `pinned` admits a spawn. The Attempt row is the persist-before-dispatch Intent (`intent_persisted` CHECK=1): `persisted` → `dispatched` (pid, Effect marker) → `terminal` (`exited` / `signaled` / `timed-out` / `spawn-failed` — there is no `success`), or `unknown-outcome` after a daemon crash. `completion_claimed` CHECK=0, `verification_status` CHECK=`not-run`, `context_bytes` ≤ 65536. Frames are append-only observations (`authority_written` CHECK=0). Attempts are never deleted. Windows sandbox / ACL / supply-chain E2E is `not-run`. |
| v37 | Attempt artifacts → CAS → independent verifier → last-ring acceptance → external send (`p13_attempt_artifact`, `p13_artifact_evidence`, `p13_run_acceptance`, `p13_external_send`) under `cognitiveos.personal.attempt-artifact/0.1` (P13-T04). An artifact row is a `sha256:` reference into the single P3-T03 `ArtifactStore` CAS (`<data_dir>/artifacts`) plus format, source frame (`hosted-dsh-child:candidate:DeliverableDraft`, frame seq, payload digest) and produced-at; freshness (`current` / `superseded`) is derived per Project + task + Member, so another Member's deliverable on the same task ref never supersedes it. Evidence is append-only and pinned by CHECK to verifier `verifier://personal/attempt-artifact` and principal `principal://personal/independent-verifier`; its report bytes live in the same CAS. `p13_run_acceptance` pins the last ring by CHECK (`stage_position = stage_count - 1`) and binds one StageTestPassed fact, artifact and evidence. `p13_external_send` is a persist-before-dispatch Intent whose `published` CHECK=0 and `connector` CHECK=`none-qualified` (planned ≠ published). Rebuilds `p11_approval_preview` so `subject_kind` also admits `run-acceptance` and `external-send` (v30 precedent). Host file-open E2E is `not-run`. |
| v38 | Routine arming after G2 plus occurrence dispatch / outcome columns (`p13_routine_arming`; `p11_routine_occurrence` rebuilt) under `cognitiveos.personal.routine-arming/0.1` (P13-T05). An arming binds one current Routine revision to one plan stage and its seated responsible Member (`armed_after` CHECK=`G2`; state `armed` / `paused` / `superseded`; `apply_mode` `arm` / `continue` / `pause` / `restart` / `resume`); the ③ declaration (`cadence_kind` `manual` / `interval`, `interval_ms` ≥ 1000, `bounded_context` ≤ 65536, `attempt_timeout_ms` ≤ 30 min) is copied from the revision body with its digest. The occurrence table gains `arming_id`, `attempt_id`, `lease_epoch`, `started_at`, `attempt_outcome` (`done` / `failed` / `blocked` / `unknown` / `timed-out` / `signaled` / `spawn-failed` / `unknown-outcome` — there is no `success`), `outcome_detail`, `elapsed_ms`, `terminal_at`, `completion_claimed` CHECK=0, and the disposition `attempted` (CHECK: `attempted` ⇔ an outcome is present). Clock / sleep / restart host E2E is `not-run`. |
| v39 | Project group chat Owner turns (`p13_project_chat_turn`) plus chat-routed ApprovalPreview kinds `plan-revision` / `task-revision` (P13-T06). Owner-authored only (`author` CHECK=`owner`); mention / routing CHECKs; `approve_attempted` CHECK=0 so the schema cannot record a chat Approve. `@manager` mints a PlanRevision candidate and a digest-bound preview; `@member` mints a task-revision candidate bounded to that Member's responsible stage. Canvas Confirm is the only writer (`apply_plan_revision_locked` inside the preview transaction). Rebuilds `p11_approval_preview` keeping every v30 / v37 kind. Secret-shaped bodies are refused before any row exists. |
| v40 | Daemon-generated reflection candidates (`p13_reflection_candidate`) plus versioned Member Runtime improvement (`p13_runtime_improvement`) and cross-Project Role Template proposals (`p13_role_template_proposal`) (P13-T11). Kinds `key-result` / `daily` / `cycle` / `incident`; sources `attempt-terminal` / `verification-evidence` / `occurrence-ledger`. `completion_claimed` CHECK=0, `model_self_report` CHECK=0, `implicit_blueprint` CHECK=0, `silent_reuse` CHECK=0. A new `p11_employee_revision` is inserted only after Owner preview confirm; rollback appends another revision. Rebuilds `p11_approval_preview` for `member-runtime-revision` / `role-template-proposal` (v39 kinds kept). CHECK SQL concatenates those kind tokens so `sqlite_master` omits `sk-`. |

P11-T06 Hidden Pi Assistant adds **no new migration**. It reuses v26 `p11_candidate` / `p11_approval_preview` and T05 read-only archive context. Assistant register requires typed provenance (`sources[]` | `owner-stated` | `assistant-assumption`); a non-null blob is rejected. Closed candidate JSON forbids `grant` / `secret` / `trigger-arm`. `draft.apply` targeting a Project/Employee/Grant/confirmed charter is rejected. The assistant plane cannot write archive, SecretStore, Memory, or confirm/apply authority. Default-deny tools; research may name existing `HttpFetchReadOnly` only. Exact Pi `0.81.1` and `cognitiveos.private-candidate/1` are identity pins, not a second scheduler or Installed Agent.

P13-T03 real inference also adds **no new migration**. `AssistantPlane::run_turn` now requires a daemon-observed `AssistantInferenceRecord` (protocol `cognitiveos.personal.assistant-inference/0.1`, bound `model_id`, `provider_round_trips ≥ 1`, bounded reply, the inferred object chain, and the daemon-derived citable URIs); the registered v26 candidate ops carry the inferred chain, the owner payload labelled as owner input, the reply digest, and the inject-order reference — never the echoed payload as the candidate. `validate_inferred_object_chain` is the single object-chain validator (closed kinds in chain order, one object per kind, every field `{value, provenance}` with typed provenance, `sources[]` uris only from fetched or owner-supplied URIs, closed schema); runtime parsing and HTTP call into it. `admit_turn_request` refuses ambient tools before any Pi process spawns. `provider_unbound_guidance()` is the fixed Settings pointer (`chat_input: false`, `silent_bind: false`, `candidate_registered: false`). `candidate_count` is a read-only accounting helper. The registration secret-shape guard treats `sk-` as a key prefix only at a token start (`risk-based`, `task-contract`, `desk-side` are prose); `bearer `, `api_key`, `x-api-key`, `ssv1:` and a token-start `sk-…` stay refused.

P11-T09 HITL canvas reuses v26 `request_preview` / `confirm_preview` / `p11_approval_preview` plus v29 `superseded_by` and v30 grant-expansion / StandingApprovalPolicy. Management HTTP `preview.reject` / `preview.narrow` / `confirm` / `standing-policy.*` are the durable caller; T05 announce+deep-link only; T06 `draft.apply` is not authority-approve. Host UI E2E is `not-run`. Settings chrome is T13. No second scheduler, no chat Approve, no Inbox L1.

P13-T10 Skill/MCP acquire adds **no new migration**. It reuses v27 `p11_install_fact` / `p11_grant` and v30 `grant-expansion` previews. `SecurityReview` + `record_reviewed_install_fact` require a passed structured review before an InstallFact; the catalog stays empty until Owner confirms a grant-phase preview. Install-phase confirm writes only the InstallFact and consumes the preview (`granted: false`). Unreviewed install, hidden-instruction / prompt-injection, ambient/marketplace sources, chat/task callers, and ambient grant are refused. `compat_test` / `review_update` / `rollback_install` never silent-grant. Supply-chain host E2E is `not-run` until P13-T13. No second grant table, no engine store.

P11-T07 hidden hosted DSH adds v31 `p11_hosted_dsh_child`. The Attempt-runner `start` caller is management HTTP `dsh.hosted.start`; task-channel aliases are 403. Digest/protocol mismatch, env/argv secrets, Pi-as-member-engine, Installed Agent chrome, and unknown child output (`success`/`ok`/`agent_end`) fail closed. Daemon Provider proxy `POST /provider/v1/dsh/chat/completions` remains the only secret-bearing path. Linux Path B is not Windows hosted qualification.

P11-T12 honest usage adds **no new migration**. It is a labelled read of v25 `llm_usage_events` / `agent_provider_bindings` / `provider_accounts`: `cost_label` is `actual` (`provider_reported`+`priced`), `estimated` (`locally_estimated`+`priced`, only when that source was recorded), or `unknown` (never JSON `0`). `GET /management/usage` also returns a four-layer binding explanation; Project/employee/Task layers are explicit `unbound` today. Account identity and quota are separate objects. Silent account/model rebind is rejected. Member-level budget hard-stop is 2.1 / Deferred.

P11-T08 Routine/Trigger adds v33. Management HTTP `routine.revision` / `routine.trigger` / `routine.ledger` / `routine.checkpoint` / `routine.resume` is the real caller. Task-channel aliases are 403. Overlap is rejected or queued as latest; host-unavailable records a visible missed row; stale revision fail-closes; completing from a checkpoint is rejected; consequential auto-resume fail-closes. HITL remains T09 canvas (not Inbox L1). Clock/sleep/restart host E2E is `not-run`.

P11-T10 Markdown Vault adds v32. Management HTTP `vault.import` / `vault.index.rebuild` / `vault.index` / `vault.conflicts` is the real caller. Context inject order is a documented store helper (current Task contract → fixed decisions → sourced excerpts → summaries → older narrative; over-limit drops older narrative first). Vault files cannot confirm/apply Project authority. Memory admission cannot swallow Vault files. Conversation archive and Artifact CAS blobs are not Vault files. Obsidian is not bundled. Host filesystem E2E is `not-run` until `DEV-WINDOWS-NATIVE-OPC-01`.

P11-T02 Windows host / tray / background adds v34. Management HTTP `host.home.admit` / `host.daemon.bind` / `host.close.request` / `host.offline.record` / `host.dsh.bind` / `host.recovery.run` / `host.recovery.advance` / `host.restore-point.record` / `GET host.status` is the real caller. Task-channel aliases are 403. Wrong install root, ACL escape, raw secret env/argv, duplicate daemon, orphan DSH, fake background, restore-as-backup, and skip-step recovery fail closed. Wake/restart runs seven ordered steps and resumes only eligible work. Not a second credential plane. Not DSH web as host shell. Native Windows install/tray/sleep/SecretStore E2E is `not-run` until `DEV-WINDOWS-NATIVE-OPC-01`.

P11-T14 X/Twitter connector adds v35. Management HTTP `connector/x/v1/{account.bind,preview.request,preview.confirm,publish.dispatch}` and `GET connector/x/v1/status` is the real caller. Task-channel aliases are 403. Raw secret env/argv/body, evasion, scraped content, chat Approve, publish-without-HITL, receipt-as-completion, unknown metrics as `0`, and X-as-P0-hero fail closed. Persist Intent then mark dispatched. Status omits `secret_ref`. Live X/CAPTCHA/platform qualification is `not-run`. Not chrome. Not a business result.

P13-T02 hosted DSH real Attempt loop adds v36. Management HTTP `dsh.hosted.attempt.run` is the real caller: it records an artifact fact, calls `HostedDshAttemptStore::persist_intent` (seated Member, exact revision, `pinned` artifact, bounded secret-free Context) **before** the `cognitive-runtime` broker spawns the exact-artifact child, marks `dispatched` when the OS pid exists, appends every frame as an observation, and writes the terminal row itself. `reconcile_unknown_outcomes` runs at daemon startup so crash-shaped `persisted`/`dispatched` rows become `unknown-outcome`, never success. `dsh.hosted.attempt.list` / `detail` are the `runs` reads; `dsh.hosted.artifact.check` / `facts` expose health / update / rollback. Task-channel aliases are 403. Heartbeats, `response: done`, exit 0, and `agent_end`-shaped text change nothing but the observation ledger; completion belongs to the independent verifier (P13-T04). Linux real spawn is implementation evidence only.

P13-T04 adds v37. The broker thread that writes the Attempt terminal hands the run to `AttemptArtifactStore::ingest_candidate`: only a `DeliverableDraft` candidate frame of a `terminal` Attempt whose canonical payload hashes to the digest already recorded on that frame becomes an artifact; the payload and the deliverable text are both put into the CAS (files and `file://` references are never accepted — `resolve_openable_ref` admits `sha256:` only). `verify_artifact` is the independent verifier: deterministic re-reads (CAS digest, source-frame binding, terminal Attempt, UTF-8 / non-empty / no secret shape) produce `passed` / `failed` / `indeterminate` evidence; the child's `response done`, exit code and prose are recorded as `not-used`. `derive_stage_test` builds the P11-T03 `StageTestOracle` from durable facts only (real seating, Member holds the stage slot, `current` freshness, passed evidence whose checked digest equals the artifact digest, CAS re-read, terminal Attempt) and calls `derive_stage_test_passed`; no caller boolean exists. `request_run_acceptance` mints a `run-acceptance` ApprovalPreview only for the last ring with a current StageTestPassed backed by passed evidence; `confirm_preview` writes the append-only `p13_run_acceptance` fact. `publication_packet` is a read-only AUTONOMY packet (`planned: true`, `published: false`, `chat_can_confirm: false`); `request_external_send` mints an `external-send` ApprovalPreview and a `previewed` Intent that confirm moves to `planned` — never `published`. Management HTTP `outputs` / `outputs.detail` / `outputs.open` / `outputs.export` / `attempt.artifact.verify` / `attempt.artifact.stage-test` / `run.acceptance.request` / `run.acceptance` / `publication.packet` / `publication.external-send.request` / `publication.sends` is the real caller; task-channel aliases are 403.
P13-T05 Routine arming adds v38. Management HTTP `routine.arm` / `routine.instruction` / `routine.arming.resume` / `routine.armings` / `routine.runs` / `today.overview` is the HTTP caller; the periodic daemon scheduler tick is the **only** dispatcher of `task://personal/routine/*` rows (the generic scheduler pass skips them; there is no second scheduler). Each pass first writes observed Attempt terminals back as occurrence outcomes (`RoutineArmingStore::record_attempt_terminal`, then queue-latest `promote_queued`), then fires due interval armings through the P11-T08 `admit_trigger` path (a paused / offline P11-T02 host makes the firing a visible `missed` row with `host-unavailable:<reason>`), then leases each undispatched `active` occurrence with `SchedulerRepository::acquire_eligible_lease` (owner `personal-daemon-scheduler`, epoch fenced) and launches one hosted Attempt through the P13-T02 persist-before-dispatch path; `bind_attempt` refuses a lease that does not match. A manual trigger on an un-armed Routine is marked `missed` (`not-armed`), never dropped. A new instruction supersedes the arming and applies at a safe point (`continue` / `pause` / `restart`); the running occurrence keeps its revision and its Attempt's context digest. Arming before G2 is refused (`ROUTINE_ARM_BEFORE_G2`); the PlanRevision / stage-test / G2 product HTTP path gap stays with P13-T04 / P13-T06.

P13-T06 Project group chat adds v39. Management HTTP `chat.post` / `chat.thread` is the HTTP caller; task-channel aliases are 403. `@manager` with a plan proposal becomes a `plan-revision` ApprovalPreview; `@member` becomes a `task-revision` candidate bounded to that Member's own responsible stage. Chat never applies a PlanRevision (`confirm_chat_candidate_locked` runs only from canvas Confirm). Approve-shaped bodies are 403 before any write; secret-shaped bodies are 422 with a Settings pointer. Cross-Project reads fail closed. Manager and Member speech keep landing through the P11-T05 speech router so the speech rules are daemon record kinds, not a client filter. `chat.thread` merges Owner turns and delivered speech oldest-first; when an Owner turn and the manager announce share a millisecond, the owner-message stays ahead of speech.

P13-T11 reflection / Member Runtime adds v40. Candidates are generated from Attempt / verification / evidence / occurrence facts (`ReflectionStore::generate_from_facts`); a model self-report is never an improvement. Member Runtime change is a new Employee revision minted only after Owner confirm of a `member-runtime-revision` preview; rollback appends a copy of the pre-confirm recipe. A Role Template proposal needs Owner confirm and does not copy the Employee into another Project. HTTP/UI for this plane wait on sibling file release (T08 `server.rs` / `mod.rs`, T10 MemberConfig). Running Attempt prompt/context rewrite is refused. `response done` / exit 0 without evidence is not a `key-result`.

Nearly every durable table carries BEFORE UPDATE/DELETE triggers that abort with
"append-only"; derived tables are `memory_search_fts` and `p11_vault_index_entry`
(rebuildable; Vault searches do not use Memory FTS).

**Load-bearing nuance**: `SqliteAuthorityStore::open` bootstraps schema constants
v1–v17 only; v18–v40 tables exist only after `prepare_personal_databases` runs the
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
