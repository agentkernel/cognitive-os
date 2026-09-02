# P13-T03 Hidden Pi Assistant real inference — running report

- Task: `P13-T03` / slice `P13-T03/D01`
- Change class: `implementation-only` (daemon/store/runtime/adapter/HTTP candidate path + `clients/pc/web` create-page assistant chat; no `core/specs`, no Lane-CTR, no new SQLite migration)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T03/pi-inference`
- Branch: `personal/P13-T03-pi-inference` (worktree `D:\agent-kernel-wt-p13-t03`, cut from `origin/main@a0465653`)
- Sibling: `P13-T02` (hosted DSH real Attempt loop) runs concurrently on its own lease/worktree; shared registration files are edited additively
- PR: Draft (this delivery)
- Claim ceiling: `hypothesis` (A7: local/CI/Linux-native evidence is not Gate/release/Profile; Pi Linux qualification does not transfer to Windows)
- Evaluation routing: **OFF**

## Identifier

Hidden engine pin: `cognitiveos.personal.hidden-pi-assistant/0.1` (P11-T06, unchanged).
New daemon-side inference protocol: `cognitiveos.personal.assistant-inference/0.1`
(request/response frame between the daemon and the `pi-agent-adapter assistant-turn`
verb; not a Core contract, not a second candidate schema — the candidate still lands
through v26 `register_candidate` / `request_preview`).

Reused, not rebuilt: exact Pi `0.81.1` (`PiCompatibilityPin`), the daemon-supervised
private completion socket + Provider proxy (`pi_runtime.rs`, P8-T13 `agent://personal/pi`
binding), `HttpFetchReadOnly` pre-validator + Rustls read-only transport (P2-T05/P2-T10),
T10 `CONTEXT_INJECT_ORDER`, T05 conversation archive index (read-only), T11 closed
candidate schema + typed provenance validators.

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-03 | worktree + lease claim | recorded | docs-only | uncommitted | `git worktree add -b personal/P13-T03-pi-inference D:\agent-kernel-wt-p13-t03 origin/main` at `a0465653`; lease row added to PARALLEL-LANES §3; PROGRESS Active task lease / `P13-T03/D01` in-progress; plan Phase 13 counts 1 in-progress |
| 2026-09-03 | worker resume after abort | recorded | `DEV-WIN-GNU-01` | uncommitted | Previous worker aborted mid-flight with uncommitted store/runtime/adapter edits and new test files. Reviewed `git diff`: store `run_turn` now requires a daemon-observed `AssistantInferenceRecord`; runtime `pi_inference.rs` (frame + bounded Context + prompt + chain parser); adapter `assistant-turn` verb + `extract_assistant_text_from_pi_events`; `pi_runtime.rs` exposes the one-shot completion socket + `accepted()` fact. Kept all of it; the kernel-server `assistant_inference.rs` module referenced by `mod.rs` did not exist yet — written in this session. |
| 2026-09-03 | failure-first negatives authored (store / runtime / adapter / HTTP) | authored (Rust execution routed) | `DEV-WIN-GNU-01` (`RUST-LINK-DEV-WIN-GNU-01`: cargo test forbidden locally) | uncommitted | Store `p13_t03_assistant_inference.rs`: echo (zero Provider round trips) refused; field without provenance refused; fabricated `sources[]` refused; chain outside closed kinds/order/schema refused; ambient tool refused before inference; inferred turn registers the chain not the echo; Provider-unbound guidance is a Settings pointer; assistant direct writes to authority/Secret/archive/Memory refused. Runtime `pi_inference.rs` unit tests: prompt names closed schema + forbids tools; request frame refuses unknown protocol/layer/oversize/bearer field; bounded Context follows inject order and drops from the tail; chain parser accepts fenced JSON only; response frame protocol-bound; research targets refused unless HTTPS + pinned. Adapter `p13_t03_assistant_turn.rs`: exactly one final text; any `tool_execution_*` (Workspace* included) refused; multiple/missing/errored finals refused. HTTP `assistant_inference.rs` tests with a scripted runtime: unbound → 409 Settings pointer + nothing registered; ambient tool / missing draft / closed schema / unlabeled provenance refused before Pi spawns; prose echo, fabricated source, zero round trips → 422 nothing registered; adapter failure 502, Pi missing 503; inferred turn registers chain with bounded context and no Approve; unpinned research targets never fetched. Because local cargo is forbidden, the "fails before" half of failure-first is observed on `DEV-LINUX-NATIVE-01` as an A/B: at pre-task `main@84188aac` the existing `assistant_turn_registers_candidate_and_omits_approve` unit passes by **echoing** the client payload as a 200 candidate with no Provider bound (the P13-T01 gap record); at this task's revision that same request is refused 409 and the new negatives pass (unit below). |
| 2026-09-03 | `clients/pc/web` `pnpm test` (vitest) | **pass** | `DEV-WIN-GNU-01` | uncommitted | 56 files / 433 tests pass, including new `createAssistantChat.test.tsx` (Settings pointer instead of chat box when unbound; honest `pi_unavailable`; no chat box when status unavailable; explain turn on lazily created research draft renders typed chain; research targets posted with `HttpFetchReadOnly` only for research turns; empty / secret-shaped text refused before POST; daemon 409 flips to Settings pointer; refused candidate renders no reply), `railWrite.test.tsx` additions (unbound → Settings pointer, nothing applied; inferred reply + chain kinds rendered), `assistant.test.ts` additions (chain projection drops unprovenanced fields; status mapping opens input only on explicit daemon `ready`; unbound detection). |
| 2026-09-03 | `clients/pc/web` `pnpm build` (`tsc --noEmit` + `vite build`) | **pass** | `DEV-WIN-GNU-01` | uncommitted | 164 modules; pre-existing >500 kB chunk warning unchanged. |
| 2026-09-03 | `cargo fmt --all -- --check` | **pass** (after `cargo fmt --all`) | `DEV-WIN-GNU-01` | uncommitted | Formatting only; not build/test evidence. |
| 2026-09-03 | `pnpm run check:consistency` / `pnpm run check:rules` | **pass** | `DEV-WIN-GNU-01` | uncommitted | First run failed on Layer 1 counts (167/132/1/1/17/35 vs plan 2 in-progress) and on writable-path overlap with the still-active `lease/personal/P13-T12/visual-spec` row; resolved by fast-forwarding onto `origin/main@84188aac` (T12 lease closed there) and updating counts to 167/132/2/1/16/35. Rules: 4 rules, 88 path references, 5 known local-only warnings. |
| 2026-09-03 | handbook sync (`generate-handbook`, `fill-handbook-fingerprints`, `check-handbook`, `generate-handbook --check`) | **pass** | `DEV-WIN-GNU-01` | uncommitted | New route annotations `assistant.status` (management + forbidden task alias) and rewritten `assistant.turn` annotation; generated `ref.http-api` regenerated; hand-written bilingual updates: `dev.daemon-and-http`, `dev.store-migrations`, `dev.agent-pi-lifecycle`, `user.pi-shell`, `user.known-limitations`, `ai.code-map`; fingerprint-only refresh where the checker required it. 58 × 2 documents OK; 18 generated pages byte-identical. |
| 2026-09-03 | `git diff --check` | **pass** | `DEV-WIN-GNU-01` | uncommitted | no whitespace errors |
| 2026-09-03 | `DEV-WIN-GNU-01` cargo build / test / clippy | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | Environment boundary, not a product failure; routed to required CI + `DEV-LINUX-NATIVE-01`. |
| 2026-09-03 | Windows Pi route (native Pi inference on Windows) | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` (not provisioned) | — | Backfilled by `P13-T13`; Linux Pi qualification does not transfer. |

## Implementation shape (this checkpoint)

- **Store** (`cognitive-store/src/assistant.rs`): `AssistantTurnSpec.inference: &AssistantInferenceRecord` is mandatory; `validate_inference_record` (protocol, bound `model_id`, `provider_round_trips ≥ 1`, bounded reply); `validate_inferred_object_chain` is the single chain validator (closed kinds in chain order, one per kind, `{value, provenance}` per field, typed provenance via the P11-T06 validator, `sources[]` uris only from fetched/owner-supplied, closed schema); ops carry `chain` / `owner_payload` (labelled) / `reply_digest` / `model_id` / `provider_round_trips` / `allowed_source_uris` / `inject_order_ref`; `admit_turn_request` for pre-spawn refusal; `provider_unbound_guidance()`; `candidate_count` read-only helper.
- **Runtime** (`cognitive-runtime/src/pi_inference.rs`): request/response frames (`deny_unknown_fields`), `assemble_bounded_context` (T10 `CONTEXT_INJECT_ORDER`, 16 KiB, tail drop), `render_assistant_prompt`, `parse_assistant_object_chain`, `validate_research_target` (registered `validate_read_only_http_fetch`).
- **Adapter** (`pi-agent-adapter`): `assistant-turn` verb (exact pin check, `--no-builtin-tools --no-extensions --no-skills --no-context-files --no-session --no-approve --mode rpc`, only the private completion provider extension, prompt over RPC stdin, one final text, tool events refused).
- **Daemon HTTP** (`kernel-server/src/personal/assistant_inference.rs`): `AssistantRuntime` trait; `DaemonAssistantRuntime` (binding from P8-T13 `agent://personal/pi` else legacy carrier; `pi.json` adapter/extension presence; pinned origins for `task://personal/assistant-research`; Rustls read-only GET; unix-only adapter spawn with allowlisted env + one-shot `PrivateCompletionSocket`, `accepted()` → `provider_round_trips`); `handle_turn` (admit → draft → binding → Context → research → infer → parse → `run_turn`); `handle_status`; `project_aggregate::handle_with_assistant` wired from `server.rs` with `layout.config_dir()/data_dir()`; store-only `handle` uses `UnconfiguredAssistantRuntime` (unbound).
- **Web** (`clients/pc/web`): `assistant.ts` (status/turn/chain projections, unbound detection, turn text gate), `CreateAssistantChat.tsx` embedded in `CreateWizardPage` (status-gated: Settings pointer / Pi-unavailable note / chat; lazily creates a research draft; renders reply + typed chain; research targets only for research turns), `RailCanvasWrite.tsx` (Settings pointer on 409; reply + chain kinds), `normalize.ts` route.

## Unique next action

Checkpoint commit + push + Draft PR so required CI (`CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`) runs
the Rust tests; then exact-revision `DEV-LINUX-NATIVE-01` validation with the pinned Pi.

## Non-claims

Not P13-T02 (hosted DSH), not P13-T06 group chat, not P13-T10 Skill/MCP acquisition.
Not Installed Agent, second scheduler, engine store, chat Approve, or mixing draft-apply
with authority-approve. No Gate/release/Profile/B01/Agent-benefit claim. Linux Pi
inference is not a Windows OPC qualification (P13-T13 backfills; Windows Pi route `not-run`).
