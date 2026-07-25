# 20260725 Personal P1-T04 Timeout/Concurrency Handoff

## 1. Task Snapshot

- Task: `P1-T04` — 有界 Personal daemon 与本地认证（timeout/concurrency 原子子批）
- Date: 2026-07-25
- Branch: `lane/personal-p1-t04-timeout-concurrency`
- Base commit: `de66370` (`main` docs note that acceptance gaps remain)
- Lane: Personal composition root `apps/kernel-server` (no Lane-RUN ownership of
  `cognitive-runtime` / `cognitive-management`)
- Status: **in-progress**. Implementation and unit tests for timeout +
  concurrency are in this branch. Local Windows GNU cannot link (exit 121).
  P1-T04 remains not-done until CI Ubuntu/Windows-MSVC executes the new tests
  successfully. Do not unlock P1-T05 yet.

## 2. Completed in this atomic batch

- `PersonalResourceBounds`: `read_header_timeout_secs` (10s) and
  `request_body_read_timeout_secs` (30s) baseline fields (ADR-0019 table).
- `serve_personal_loopback` non-`once` path: each accepted connection runs on a
  worker thread so shared connection/in-flight counters are meaningful.
- `handle_connection`:
  - socket header read timeout before parse
  - body read timeout after headers/`Content-Length` validation
  - `PERSONAL_REQUEST_READ_TIMEOUT` → HTTP 408
  - `CONNECTION_LIMIT_EXCEEDED` / `IN_FLIGHT_LIMIT_EXCEEDED` → HTTP 429
  - counters released on both success and fail-closed paths
- Unit tests in `apps/kernel-server/src/personal/server.rs`:
  - `slow_header_read_times_out_with_stable_protocol_code`
  - `slow_body_read_times_out_with_stable_protocol_code`
  - `concurrent_connection_limit_rejects_excess_connection`
  - `in_flight_request_limit_rejects_excess_request`
- ADR-0022 bounds section updated for timeout/concurrency codes.

## 3. Not completed / out of scope

- CI execution of the new unit tests (pending this PR)
- Marking P1-T04 `done` / unlocking P1-T05
- Per-channel concurrent connection accounting (`max_concurrent_connections_per_channel`)
- Idle connection timeout (ADR-0019 60s) as a separate timer
- UDS product default listener
- Readiness/doctor (P1-T05), CLI product entry (P1-T06)
- Task/Memory/MCP; registry/schema/vector; G0/B01-B12/Profile claims

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| Local Windows GNU `cargo test -p kernel-server` | not-supported host | linker exit 121 (P0-T01) |
| `pnpm run check:consistency` | executed | OK (273 REQ / 55 codes / 63 schemas / 85 vectors) |
| `git diff --check` | executed | clean |
| CI `cargo test --workspace --locked` | pending | required before P1-T04 done |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- Clients remain non-authority; timeout/limit rejections create no authority
  side effects.
- No secrets/tokens in timeout or concurrency error payloads.
- Existing auth/size/host/cookie/restart process tests from PR #95 remain.
- Fail-closed default: slow or excess traffic is refused, not buffered forever.

## 6. Next entry

1. Open/merge PR for this branch; watch Ubuntu + Windows/MSVC CI.
2. On CI SUCCESS for the new timeout/concurrency tests, mark P1-T04 `done` in
   `PERSONAL-DEVELOPMENT-PLAN.md` + PROGRESS, then start P1-T05.
3. **P0-T03** still needs owner license/platform/distribution GO/NO-GO.
4. Suggested prompt: after CI green, close P1-T04 done evidence and begin
   P1-T05 readiness/status/doctor without claiming G0/Profile.

## 7. Snapshot

- PROGRESS updated: yes (still in-progress; no Profile claim)
- Formal Personal ledger updated: yes (`in-progress`, CI pending)
- PR: pending
- CI: pending
- Commit: pending (this handoff lands with the code batch)
