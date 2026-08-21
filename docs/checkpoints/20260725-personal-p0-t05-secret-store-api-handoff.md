# 20260725 Personal P0-T05 SecretStore API Handoff

## 1. Task Snapshot

- Task: `P0-T05` — Linux Secret Service PoC
- Date: 2026-07-25
- Branch: `lane/personal-p0-t05-secret-store-api`
- Base commit: `cdd177806e0e0e7908459ab4f3f1303527310bf4` (`main` @ PR #89)
- Lane: Personal / isolated crate (does not take Lane-RUN `cognitive-runtime` ownership)
- Status: **done** (CI executed workspace tests on Ubuntu + Windows/MSVC)

## 2. Completed in this atomic batch

- Added workspace crate `crates/cognitive-secret` (zero external dependencies).
- Froze daemon-facing API:
  - `SecretStore::{probe, put, get, delete}`
  - opaque `SecretRef`
  - attribute-keyed `put` as rotate
- Implemented backends:
  - `SimulatedSecretServiceStore` / `EphemeralSecretStore` (test double only)
  - `UnavailableSecretStore` (fail-closed)
  - `LinuxSecretServiceProbe` (native class; probe-only; mutating D-Bus deferred to P1-T02)
- Focused tests in `crates/cognitive-secret/tests/p0_t05_secret_store.rs`:
  - put/get/rotate/delete
  - service-absent / locked / prompt-unavailable fail-closed
  - Debug/Display redaction
  - environment non-leak
  - native probe never provides plaintext fallback
- ADR-0018 documents product secret boundary and rejected alternatives.
- Aligned `docs/plan/plan.md` P0-T05 task card; updated formal Personal ledger and PROGRESS.

## 3. Not completed / out of scope

- Live `org.freedesktop.secrets` D-Bus put/get/delete adapter (P1-T02).
- Provider configuration, CLI hidden input, daemon init integration (P1-T02+).
- Real Provider API keys, evidence digests containing secrets, SQLite secret tables.
- G0 / B01-B12 / Profile claims.
- Prior unmerged docs-only branch `lane/personal-p0-t05-secret-service-poc` (WSL `secret-tool` environment notes) was **not** merged; this batch supersedes it with a frozen Rust port.

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| `cargo check -p cognitive-secret` | pass (local Windows) | Typecheck succeeded. |
| `cargo test -p cognitive-secret` | not-supported locally | Windows GNU linker exit 121; MSVC toolchain present but `link.exe` missing. P0-T01 defines CI Linux + Windows/MSVC as supported. |
| CI `cargo test --workspace --locked` (includes `p0_t05_secret_store`) | pass | CI run [30153311857](https://github.com/agentkernel/cognitive-os/actions/runs/30153311857) Ubuntu + Windows/MSVC success. |
| `pnpm run check:consistency` | to-run before commit | Docs/static only; no registry change. |
| `git diff --check` | to-run before commit | — |
| Personal Gates / B01-B12 / Profile | not-run | No claim. |

No secret material was written to repository evidence. Test markers are synthetic non-production bytes only.

## 5. Design and safety boundaries

- `cognitive-secret` is not an authority writer and does not touch SQLite.
- Ephemeral simulation is never a product fallback (`SecretStoreClass::EphemeralTestDouble`).
- Unavailable / locked / prompt-unavailable fail closed with no plaintext path.
- `SecretMaterial` Debug/Display always redacted; Drop best-effort zeroizes bytes.
- No change to registry, schema, transitions, vectors, or generated bindings.

## 6. Next entry

1. Open/merge PR for this branch; wait for Ubuntu + Windows/MSVC CI green on workspace tests.
2. P0-T05 is marked `done` after CI green; remaining Phase 0 items are P0-T03 (owner decision), P0-T06 (depends P0-T03), P0-T07.
3. Next legal Personal tasks: P0-T03 (owner license/platform/distribution decision), P0-T07 (transport/threat model, docs/ADR), P1-T01 (XDG + migrations operationalization). Do not start P1-T02 until P0-T05 is `done` and P1-T01 is ready.
4. Suggested prompt: `Continue Personal plan. Read AGENTS.md, PROGRESS.md, 20260725-personal-p0-t05-secret-store-api-handoff.md, PARALLEL-LANES.md, PERSONAL-DEVELOPMENT-PLAN.md. If P0-T05 CI is green, mark done; otherwise pick next dependency-satisfied P0 task without inventing owner decisions.`

## 7. Snapshot

- PROGRESS updated: yes (P0-T05 in-progress; no Profile claim).
- Formal Personal ledger updated: yes (`done`).
- Commits: `eb45e42`, `8ef7579`, plus done-status docs commit.
- PR: [#90](https://github.com/agentkernel/cognitive-os/pull/90).
- CI: [30153311857](https://github.com/agentkernel/cognitive-os/actions/runs/30153311857) success.
