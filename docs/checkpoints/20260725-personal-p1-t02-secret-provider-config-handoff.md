# 20260725 Personal P1-T02 Secret Provider Config Handoff

## 1. Task Snapshot

- Task: `P1-T02` — SecretStore 正式后端与 Provider 配置
- Date: 2026-07-25
- Branch: `lane/personal-p1-t02-secret-provider-config`
- Base commit: `ee840c4` (`main` after P1-T01)
- Lane: Personal / isolated crate `cognitive-secret` (does not take Lane-RUN
  `cognitive-runtime` / `cognitive-management` ownership)
- Status: **done** (CI Ubuntu + Windows/MSVC green)

## 2. Completed in this atomic batch

- Extended `crates/cognitive-secret`:
  - `ProviderConfig` + `ProviderConfigRepository` (XDG `provider.json`; HTTPS-only
    base URL; only opaque `SecretRef`; no secret bytes)
  - `ProviderKeyService` configure / rotate / delete / resolve / restart reload
  - `LinuxSecretToolStore` native mutating adapter (`secret-tool` + session bus;
    secret material only on stdin)
  - `select_production_secret_store` never returns ephemeral test double
  - `read_secret_material_from_reader` hidden-input helper (CLI echo-off remains P1-T06)
  - `SecretStore` impls for `&T` and `Arc<T>` for shared-backend restart tests
- Focused tests `tests/p1_t02_provider_secret.rs`:
  - configure/rotate/delete
  - restart resolve via persisted ref
  - deleted secret fail-closed
  - locked store fail-closed without config write
  - HTTP/credential URL rejection
  - redaction of config/service Debug and on-disk document
  - production selection never ephemeral
  - native store fail-closed without session
  - hidden-input reader newline strip + env non-leak
  - JSON round-trip without secret bytes
- ADR-0020 documents Provider config binding; ADR-0018 updated for P1-T02 delivery.
- Aligned formal Personal ledger, `plan.md` task card, and `PROGRESS.md`.

## 3. Not completed / out of scope

- Product CLI `cognitive init` hidden input / terminal echo-off (P1-T06).
- Real DeepSeek Provider probe and model snapshot (P1-T03).
- Wiring into `kernel-server` / `admin-cli` / management readiness (P1-T04/T05/T06).
- Live gnome-keyring CI job (native path is optional; simulated backend covers CI).
- G0 / B01-B12 / Profile claims.
- Registry / schema / vector / transition changes (none).

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| `cargo check -p cognitive-secret --tests` | pass (local Windows) | Typecheck succeeded |
| `cargo clippy -p cognitive-secret --all-targets --locked` | pass (local) | Clean after is_multiple_of / is_none fixes |
| `cargo test -p cognitive-secret --locked` | not-supported host | Windows GNU linker exit 121 (P0-T01 non-supported baseline) |
| `pnpm run check:consistency` | pass (local) | 273 REQ / 55 codes / 63 schemas / 85 vectors |
| CI `cargo test --workspace --locked` | pass | run [30156079691](https://github.com/agentkernel/cognitive-os/actions/runs/30156079691) Ubuntu + Windows/MSVC; includes `p1_t02_provider_secret` |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

No secret material was written to repository evidence. Test markers are synthetic non-production bytes only.

## 5. Design and safety boundaries

- Config may store only opaque `SecretRef`; never API keys.
- Production selection never uses `EphemeralTestDouble`.
- Clients/Pi/CLI remain non-authority; this crate is not an authority writer.
- No change to registry, schema, transitions, vectors, or generated bindings.


## 6. Next entry

1. PR #93 CI green (run 30156079691); merge when ready.
2. Dependency-satisfied next Personal tasks:
   - **P1-T03** Provider/model discovery (depends P1-T02)
   - **P1-T04** bounded daemon (depends P0-T07 + P1-T01; independent of P1-T03)
   - **P0-T03** still needs owner license/platform/distribution GO/NO-GO
3. Suggested prompt: `Continue Personal plan. Read AGENTS.md, PROGRESS,
   20260725-personal-p1-t02-secret-provider-config-handoff.md, PARALLEL-LANES,
   PERSONAL-DEVELOPMENT-PLAN. Prefer next dependency-satisfied task (P1-T03 or
   P1-T04) without claiming G0/Profile. If selecting P0-T03, stop and ask owner
   for license/platform/distribution.`

## 7. Snapshot

- PROGRESS updated: yes (P1-T02 done; no Profile claim)
- Formal Personal ledger updated: yes (`done`)
- Commits: `9bc308f` (feat) + docs evidence commit
- PR: [#93](https://github.com/agentkernel/cognitive-os/pull/93)
- CI: [30156079691](https://github.com/agentkernel/cognitive-os/actions/runs/30156079691) success