# 20260725 Personal P1-T03 Provider Discovery Handoff

## 1. Task Snapshot

- Task: `P1-T03` — Provider、模型发现与能力快照
- Date: 2026-07-25
- Branch: `lane/personal-p1-t03-provider-discovery-probe`
- Base commit: `d1fb6f3` (`main` after P1-T02)
- Lane: Personal / isolated crate `cognitive-secret` (does not take Lane-RUN
  `cognitive-runtime` / `cognitive-management` ownership)
- Status: **done pending CI linked-test evidence** (local typecheck/clippy green;
  Windows GNU host cannot link tests — P0-T01 non-supported baseline)

## 2. Completed in this atomic batch

- Extended `crates/cognitive-secret`:
  - `ProviderTransport` injectable HTTPS I/O port with Authorization/body redaction
  - `ProviderDiscoveryService::{list_models, discover_probe_and_persist}`
  - Active probes: chat / stream / tool_call (candidate shape only) / cancel
  - `ProviderCapabilitySnapshot` + product-local `fnv1a64:` identity digest
  - `ProviderKeyService::persist_selected_snapshot_digest`
  - `ProviderConfig::with_selected_snapshot_digest`
- Focused tests `tests/p1_t03_provider_discovery.rs` (mock transport):
  - happy-path discovery + persist digest
  - 401/403/404/429/5xx classification
  - alias drift
  - manual model fallback on empty catalog
  - HTTP 200 without tool_calls → capability_missing
  - chat timeout classification + secret non-leak in Debug
  - Authorization header redaction helper
- ADR-0021 documents discovery/probe/snapshot decisions.
- Aligned formal Personal ledger, `plan.md` task card, and `PROGRESS.md`.

## 3. Not completed / out of scope

- Live DeepSeek / real Provider network probe (requires user key; not in CI).
- Production HTTPS client wiring into daemon (P1-T04 / P1-T06 composition).
- Readiness/doctor projection service (P1-T05).
- Registry / schema / vector / transition changes (none).
- G0 / B01-B12 / Profile claims.
- Lane-RUN `cognitive-runtime` ownership transfer of provider module.

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| `cargo check -p cognitive-secret --tests --locked` | pass (local Windows) | Typecheck succeeded |
| `cargo clippy -p cognitive-secret --all-targets --locked` | pass (local) | Clean after parse_models fix |
| `cargo test -p cognitive-secret --locked --test p1_t03_provider_discovery` | not-supported host | Windows GNU linker failure (P0-T01 non-supported baseline) |
| `pnpm run check:consistency` | pass (local) | 273 REQ / 55 codes / 63 schemas / 85 vectors |
| CI `cargo test --workspace --locked` | pending | Must confirm `p1_t03_provider_discovery` on Ubuntu + Windows/MSVC |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

No secret material was written to repository evidence. Test markers are synthetic non-production bytes only.

## 5. Design and safety boundaries

- Secret material is attached only as a short-lived `Authorization: Bearer` header.
- Config stores only opaque `SecretRef` + non-secret snapshot digest.
- Tool-call probe success means candidate-shaped `tool_calls` only; not Effect
  dispatch and not Task completion.
- Clients/Pi/CLI remain non-authority; this crate is not an authority writer.
- No change to registry, schema, transitions, vectors, or generated bindings.

## 6. Next entry

1. Open/merge PR for this branch; wait for CI Ubuntu/Windows-MSVC green including
   `p1_t03_provider_discovery`.
2. Dependency-satisfied next Personal tasks:
   - **P1-T04** bounded daemon (depends P0-T07 + P1-T01; independent of P1-T03)
   - **P0-T03** still needs owner license/platform/distribution GO/NO-GO
   - **P1-T05** after both P1-T03 and P1-T04
3. Suggested prompt: `Continue Personal plan. Read AGENTS.md, PROGRESS,
   20260725-personal-p1-t03-provider-discovery-handoff.md, PARALLEL-LANES,
   PERSONAL-DEVELOPMENT-PLAN. Prefer next dependency-satisfied task (P1-T04)
   without claiming G0/Profile. If selecting P0-T03, stop and ask owner for
   license/platform/distribution.`

## 7. Snapshot

- PROGRESS updated: yes (P1-T03 recorded; no Profile claim)
- Formal Personal ledger updated: yes (`done`, CI test evidence pending)
- Commits: pending this session
- PR: pending
- CI: pending
