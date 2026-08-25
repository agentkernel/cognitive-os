# 28 — Real API Contract Map (implementation-verified)

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Rule enforced: **a route exists only because implementation confirms it.** Every entry cites the handler. Documentation claims without handlers are listed as absent. Audited source: `apps/kernel-server/src/personal/` @ working tree (~`main` `aeb9c3a9`).

---

## 1. Front door (all routes inherit)

Hand-rolled HTTP/1.1 over `std::net::TcpListener`, loopback-only (`server.rs:443-453`), single-instance lock, `Connection: close`. Per-request: cookie rejection (403 `LOCAL_COOKIE_AUTH_FORBIDDEN`), loopback Host check (400 `LOCAL_HOST_HEADER_REJECTED`), Origin/Referer loopback allowlist (403 `LOCAL_ORIGIN_HEADER_REJECTED`), body ≤1 MiB, ≤32 conns / ≤16 in-flight (429), read timeouts (408). Router: prefix cascade `dispatch_http_route` (`server.rs:645-953`); unmatched → 404 `PERSONAL_ROUTE_NOT_FOUND` **except** unmatched `POST /management/*` and `POST|GET /task/*` which return **200 stubs** (see §8 risk R-1).

**Error envelopes (three shapes — normalization required in any client):**
1. Front door: `{"status":"error","error":{"code","message","category":"protocol","retryable":false,"stage":"personal-front-door"}}` (`server.rs:3146-3164`)
2. Task/resource subsystems: `{"status":"error","code","message"}` (`task_api.rs:2168-2174`, `resource_manager.rs:931-936`)
3. Backup: `{"error":{"code","detail"}}` (`user_backup.rs:184-189`)

## 2. Session domain

| Route | Auth | Request → Response | Errors | Evidence |
|---|---|---|---|---|
| `POST /local/session` | bootstrap secret in body | `{channel:"task"\|"management", principal_id, bootstrap_secret}` → `{status:"ok", token, channel, session_id, absolute_expiry_secs, idle_expiry_secs}` | 401 `LOCAL_BOOTSTRAP_MISMATCH` | `server.rs:1020-1059` |
| `GET /personal/health` | none | → `{schema_version:1, surface:"personal-health", status:"ok", authority_side_effects:false, readiness_claim:"not-claimed", profile_claim:"not-claimed"}` | — | `server.rs:930-941` |

Session facts: in-process `HashMap` only (restart clears all), 12 h absolute / 30 min idle expiry (`bounds.rs:33-34`), channel-scoped, cross-channel → 403 `SHELL_CHANNEL_BINDING_MISMATCH` (`auth.rs:340-342`). **No logout/revoke route.**

## 3. Readiness / doctor domain

| Route | Auth | Response | Evidence |
|---|---|---|---|
| `GET /personal/status` | mgmt | compact status projection (components: system/database/secret/provider/daemon/pi; overall `blocked\|degraded\|ready`; `first_conversation_ready`) | `server.rs:2635-2683`, `readiness.rs:201-214` |
| `GET /personal/readiness` | mgmt | **identical projection to /status** (alias) | `server.rs:922-925` |
| `GET /personal/doctor` | mgmt | detailed projection (+`source`, `observed_at_unix_ms`, `facts[]`, `guidance[]`); sub-sections six_resource/headless_vault/operability are **static placeholders** (`*_NOT_PROBED`) | `readiness.rs:217-234, 236-313` |

## 4. Provider Control Plane domain (all mgmt; `/task/*` twins → 403 `PROVIDER_CONTROL_CHANNEL_FORBIDDEN`)

| Route | Mutates | Shape (key fields) | Evidence |
|---|---|---|---|
| `GET /management/providers/accounts` | no | `{accounts:[{id, display_name, provider_kind, endpoint, secret_ref, status, catalog_revision, last_discovery_error, allow_private_network, allow_insecure_http, network_scope}]}` — **`secret_ref` is serialized** | `provider_control_plane.rs:202, 1270-1284` |
| `GET /management/providers/accounts/inspect?id=` | no | `{account}`; 404 `PROVIDER_ACCOUNT_NOT_FOUND` | `:212` |
| `POST /management/providers/accounts` | yes | `{display_name, provider_kind, endpoint?, allow_*?, api_key?}`; `headers`/`authorization` keys rejected | `:223, 1177-1186` |
| `POST /management/providers/accounts/update` | yes | `{id, endpoint?, allow_*?, reconfirm?}`; trust change without `reconfirm:true` → 409 | `:301, 348-356` |
| `POST /management/providers/accounts/delete` | yes | `{id}`; active bindings block | `:383` |
| `POST /management/providers/accounts/key` | yes (secret-bearing) | `{id, op:"set"\|"rotate"\|"remove", api_key?}` → SecretStore only | `:408, 449-553` |
| `POST /management/providers/models/refresh` | yes (catalog) | `{id}`; live upstream `GET /models`; failure preserves catalog, marks `degraded` | `:555-723` |
| `GET /management/providers/models?account_id=` | no | `{models:[{account_id, model_id, source, pricing_version, price_*_per_million}]}` | `:725, 1286-1297` |
| `POST /management/providers/models/add` | yes | `{account_id, model_id, pricing_version?, price_*?}`; endpoint-servability enforced | `:746` |
| `POST /management/providers/models/set-price` | yes | `{account_id, model_id, …}` | `:806` |
| `GET /management/agent-bindings` | no | `{bindings:[{agent, account_id, model_id, revision, status}]}` | `:850, 1299-1307` |
| `POST /management/agent-bindings` | yes | `{agent:"pi"\|"dsh", account_id, model_id, expected_revision?}`; 409 `PROVIDER_BINDING_REVISION_STALE` | `:860` |
| `POST /management/agent-bindings/remove` | yes | `{agent}` | `:942` |
| `GET /management/usage` | no | `{events:[{event_id, account_id, cost_micros, cost_status}]}` (no filters) | `:957` |
| `GET /management/budgets` · `POST /management/budgets` · `POST /management/budgets/remove` | list no; set/remove yes | `{budget_id, scope_kind, scope_id, token_limit, amount_micros_limit}` — **observe-only, no enforcement hook** | `:973-1029` |
| `GET /management/alerts` · `POST /management/alerts/acknowledge` | list no; ack yes | `{alert_id, budget_id, threshold_kind, issued_at_ms, acknowledged_at_ms}` | `:1047-1064` |
| `GET /management/audit` | no | `{events:[{audit_id, action, outcome, detail}]}` (provider-plane only) | `:1082` |

## 5. Provider proxy + selected model (mgmt)

| Route | Notes | Evidence |
|---|---|---|
| `POST /provider/v1/chat/completions` | OpenAI body; only `model`+`stream` validated; `stream:true` → SSE passthrough; bound-binding path with legacy fallback; header `X-CognitiveOS-Provider-Network-Nanos` always | `server.rs:1745-1846, 2064-2153`; `provider_proxy.rs:477-490` |
| `POST /provider/v1/dsh/chat/completions` | agent=dsh; model overridden to bound model; streaming refused for http/anthropic plans | `server.rs:660, 1882-1890` |
| `GET /provider/v1/selected-model` · `GET /provider/v1/dsh/selected-model` | `{selected_model, selected_snapshot_digest, chat_capable, …}`; 503 when unavailable | `server.rs:2555-2633` |

## 6. dsh runtime domain (mgmt)

| Route | Shape | Evidence |
|---|---|---|
| `GET /personal/dsh/runtime` | `{state:"ACTIVE"\|"INACTIVE"\|"CRASHED", session_count, sessions:[{session_id,state,fencing_epoch,last_sequence,task_ref}], process_id, process_alive, last_heartbeat_unix_ms, candidate_only:true, authority_side_effects:false, dsh_response_is_not_task_completion:true, …}` | `server.rs:2175-2295`, `task_api.rs:766-803` |
| `POST /personal/dsh/runtime` | `{op:"bind"\|"heartbeat"\|"clear"\|"apply", process_id?, expected_revision?}`; `apply` waits ≤4 s for dsh-web ack; 409 `DSH_RUNTIME_INACTIVE` / `PROVIDER_BINDING_REVISION_STALE` | `server.rs:2423-2553` |

## 7. Task channel domain (task bearer)

| Route | Request → Response | Evidence |
|---|---|---|
| `POST /task/intent.record` | `TaskIntentRecordRequest{schema_version, conversation_or_scope_ref, input_refs?, raw_expression}` → `{user_intent_record_id, intent_digest, recorded_at}`; 409 `TASK_INTENT_RECORD_REJECTED` | `task_api.rs:360-429` |
| `POST /task/intent.interpret` | `{user_intent_record_id, candidate:{objectives, constraints, forbidden, assumptions, ambiguities[], information_gaps, …}}` → `{interpretation_id, interpretation_digest, material_ambiguity_count, status:"candidate"\|"clarification_required"}` | `task_api.rs:431-530` |
| `POST /task/preview` | `TaskPreviewRequest{task_contract_draft{task_ref, objective, scope, conditions[], budget, budget_id, deadline, loop_object_id, max_iterations, max_retries, allowed_state_domains, allowed_tools}}` → `{task_ref, preview_digest, objective, condition_count, budget}`; 400 `TASK_PREVIEW_REJECTED` | `task_api.rs:532-565`; `task_preview_request.rs:55-68` |
| `POST /task/admit` | `{task_contract_draft, preview_digest, expected_current_epoch, acceptance:{accepted_by, accepted_digest, interpretation_id}}` → `{task_ref, task_contract_ref, contract_digest, contract_epoch}`; 403 principal mismatch; 409 admission rejected | `task_api.rs:567-665` |
| `POST /task/candidate` | `PublicPiCandidateRequest{task_ref, tool_ref, action, target, parameters?, parameters_digest, expected_state_version, operation_descriptor_id}` → `{authorization_id, admitted:true, authority_side_effects:true}`; 404/409 classes | `task_api.rs:667-673, 923-1027` |
| `POST /task/akp/dsh` | dsh bridge ops `activate\|stop\|event` → `{accepted, sequence, candidate_only:true, result?, error?}` | `task_api.rs:675-698`; `deepseek_harness.rs:139-170` |
| `GET /task/evidence?task_ref=` | → `TerminalTaskEvidence{contract_epoch, lifecycle{current_state,current_version,transitions[],transitions_truncated}, intent_refs, effect_refs, reconcile_class, latest_verification?, latest_acceptance?, durable_cursor}`; 404 `TASK_EVIDENCE_NOT_FOUND` | `task_api.rs:81-137, 1068-1093` |
| `GET /task/effects?task_ref=` | → `BoundedEffectHistory{effects:[{effect_ref, original_key_digest, stage, outcome_class, reconcile_class, mutation_count?, fixed_post_state_ref?, report_ref?}], effects_truncated}`; non-`task_ref` keys → 400 | `task_api.rs:1104-1129, 2029-2068` |
| `GET /task/observation?family=&task_ref=` | O2/O3/O4/O5/O13 bounded families; forbidden query keys rejected 400; stale cursor/digest/gap → 409 | `observation.rs:35-44, 547-949` |
| `GET /task/watch?resume_from=` | SSE `snapshot` (tasks **always empty**) + `delta` frames; 128-event process-local ring; stale resume → 409 `TASK_WATCH_RESUME_STALE` | `task_api.rs:1029-1066` |
| any other `/task/*` | **200 stub** `{note:"no Task API operation matched"}` | `task_api.rs:346-356` |

**Absent (confirmed by absence of handlers):** `/task/cancel`, `/task/complete` (forbidden by inventory), task pause/resume/retry, task list/search, task detail GET.

## 8. Resource domains (mgmt unless noted)

| Route | Status | Evidence |
|---|---|---|
| `GET /resource/v1/projection?family=` (+`/task/resource/v1/projection?task_ref=`) | implemented; memory/skill/context families self-declare `not-backed`; tool/task/runtime `available` | `resource_api.rs:1335-1381, 1567-1646` |
| `GET /resource/v1/watch?family=` (+task twin) | SSE; **inert after startup** (only `projection.initialized` ever published) | `resource_api.rs:1383-1433, 60-70` |
| `GET /management/resource/v1/memory/object?id=` | `memory.explain` shape | `resource_api.rs:527-550` |
| `POST /management/resource/v1/memory/remember` | 201; sealed + unsealed paths; retention cap 31 536 000 s | `resource_api.rs:736-765, 1006-1010` |
| `POST /management/resource/v1/memory/forget` | 201 tombstone | `resource_api.rs:1198-1268` |
| `POST /management/resource/v1/skill/import` (+supersede) | 201 | `resource_api.rs:1036-1147` |
| `POST /management/resource/v1/skill/bind` | 201 | `resource_api.rs:1149-1196` |
| `POST /management/resource/v1/skill/binding/revoke` | 201 | `resource_api.rs:1270-1333` |
| `GET /management/resource/v1/skill/binding/explain[?kind=revision]` | explain shapes | `resource_api.rs:551-614` |
| `GET /management/resource/v1/list?family=` | tool/memory/skill/task backed (limit 64); context/runtime empty `projection-only` | `resource_manager.rs:143-161, 485-721` |
| `GET /management/resource/v1/inspect?family=&id=` | envelope; 404 `RESOURCE_MANAGER_NOT_FOUND` | `resource_manager.rs` |
| `POST /management/resource/v1/{bind,unbind,revoke}` (skill) · `{enable,disable,revoke}` (tool) | CAS envelope mutations; other combos → 400 `RESOURCE_MANAGER_OPERATION_UNSUPPORTED` | `resource_manager.rs:287-436` |
| `POST /management/resource/v1/{create,install,execute,complete}` | **deliberately refused** 400 `RESOURCE_MANAGER_OPERATION_FORBIDDEN` | `resource_manager.rs:101-106` |
| `GET /{mgmt,task}/resource/v1/tool[/discover]` · `…/tool/exposure?task_ref=` | catalog + exposure | `tool_lifecycle.rs:156-200, 404-429` |
| `POST /management/resource/v1/tool/{enable,disable,quarantine,revoke}` | lifecycle mutations; illegal transitions 409 | `tool_lifecycle.rs:202-278` |
| `POST /task/resource/v1/tool/selection` | digest-gated selection receipt | `tool_lifecycle.rs:280-390` |
| `POST|GET /task/resource/v1/consumption` | task-scoped memory/skill pins (task channel) | `resource_api.rs:90-503` |
| `POST /management/resource/v1/backup[/preflight]` · `POST …/restore` | secret-excluding archive; restore live-apply with 409 classes | `user_backup.rs:24-50, 152-175` |
| `POST /management/context-authorization/{facts,revocations}` | 201 admitted | `server.rs:983-1018` |
| `GET|POST /management/resource/v1/fault-profile`, `…/http-origin` | **campaign-gated** (`PERSONAL-PERF-EVAL-*`/pinned tasks only) — not product surface | `fault_profile.rs:353-357`, `pinned_https.rs:366` |
| unmatched `POST /management/*` | **200 stub** "business routes deferred" | `server.rs:1086-1095` |

## 9. Static UI domain

| Route | Notes | Evidence |
|---|---|---|
| `GET /ui`, `GET /ui/*` | unauthenticated; bundle root `<data_dir>/ui`; index fallback only for `/ui`/`/ui/` (no SPA fallback); segment allowlist; 1 MiB/asset; 503 `LOCAL_UI_BUNDLE_UNAVAILABLE` when absent; CSP `default-src 'self'`; `Cache-Control: no-store` | `server.rs:2943-3007` |

## 10. Risk register for any client (R-1..R-5)

- **R-1 200-stub fallthrough** (`server.rs:1086-1095`, `task_api.rs:346-356`): unknown management/task routes return 200 — clients must whitelist known routes and treat stub notes as not-run.
- **R-2 Three error envelopes** (§1): one normalization layer required.
- **R-3 Inert resource watch** (`resource_api.rs:60-70`): no deltas after startup — do not design live resource updates on it.
- **R-4 Empty task-watch snapshot** (`task_api.rs:1047-1050`): `tasks:[]` always — do not design "watch shows current tasks" on it.
- **R-5 `secret_ref` serialized** in account responses (`provider_control_plane.rs:1276`): display policy presence/absence only.

---

*This map is the contract baseline for the traceability matrix (`35`) and the backend dependency matrix (`37`).*
