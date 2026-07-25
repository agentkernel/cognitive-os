# 20260725 Personal P1-T05 Readiness/Status/Doctor Handoff

## 1. Task Snapshot

- Task: `P1-T05` — Readiness、status 与 doctor 应用服务
- Date: 2026-07-25
- Branch: `lane/personal-p1-t05-readiness-doctor`
- Base commit: `7cfa9ac` (`origin/main` after P1-T04 merge)
- Lane: Personal composition root `apps/kernel-server` (does **not** take
  Lane-RUN ownership of `cognitive-management` / `cognitive-runtime`)
- Status: **in-progress** (implementation landed; Rust behavior evidence
  awaits CI Ubuntu/Windows-MSVC). Not a G0/B01-B12/Profile claim.

## 2. Completed in this atomic batch

- `apps/kernel-server/src/personal/readiness.rs`
  - `evaluate_personal_readiness` for system/database/secret/provider/daemon/pi
  - overall aggregation: blocked > degraded > ready
  - `first_conversation_ready` requires Pi ready (still `not_configured`)
  - status/doctor JSON projections with non-claims and redaction
- Authenticated routes (management bearer only):
  - `GET /personal/status`
  - `GET /personal/readiness`
  - `GET /personal/doctor`
- Unit tests: blocked / degraded / ready+non-Pi / secret locked / SecretRef redaction
- Process test: `tests/p1_t05_personal_readiness.rs` (auth + wrong channel + blocked projection)
- ADR-0023 documents ownership, aggregation, and non-claims
- Formal ledger + `plan.md` task card aligned; PROGRESS note pending commit

## 3. Not completed / out of scope

- CI Ubuntu/Windows-MSVC execution evidence (pending push)
- CLI product entry `cognitive status/doctor` (P1-T06)
- Pi package checks (P1-T07)
- Live Secret Service / live Provider network
- G0 / B01-B12 / Profile claims
- Registry / schema / vector / transition changes

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| Local Windows GNU `cargo test -p kernel-server` | not-supported host | linker exit 121 (P0-T01) |
| Local MSVC `cargo check` | not-supported host | `link.exe` not found |
| `pnpm run check:consistency` | executed | OK (273 REQ / 55 codes / 63 schemas / 85 vectors) |
| `git diff --check` | executed | clean (CRLF→LF warning only on server.rs) |
| CI PR run | pending | — |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- Clients remain non-authority; readiness is a projection only.
- No authority side effects from status/doctor evaluation.
- Secrets, bootstrap material, and opaque SecretRef values are not serialized.
- Static analysis / CI green is never rewritten as runtime ready.
- Pi deferred honestly as `not_configured` rather than synthetic ready.

## 6. Next entry

1. Push branch + open PR; wait for CI Ubuntu/Windows-MSVC.
2. On CI green, mark P1-T05 `done` with run IDs and merge.
3. Next dependency-satisfied Personal task: **P1-T06** CLI product entry
   (depends P1-T02 + P1-T05). **P0-T03** still needs owner license/platform
   decisions (blocks P0-T06 / installer / G0).
4. Suggested prompt: after P1-T05 CI green, implement P1-T06
   `cognitive init/doctor/status/daemon` calling the shared readiness
   projections without direct SQLite authority writes.

## 7. Snapshot

- PROGRESS updated: yes (P1-T05 in-progress; no Profile claim)
- Formal Personal ledger updated: yes (`in-progress`)
- PR: pending
- CI: pending
- Commit: pending
