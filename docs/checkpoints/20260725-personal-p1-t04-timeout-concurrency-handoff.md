# 20260725 Personal P1-T04 Timeout/Concurrency Handoff

## 1. Task Snapshot

- Task: `P1-T04` — 有界 Personal daemon 与本地认证（timeout/concurrency 原子子批）
- Date: 2026-07-25
- Branch: `lane/personal-p1-t04-timeout-concurrency`
- Base commit: `de66370` (`main` docs note that acceptance gaps remain)
- Lane: Personal composition root `apps/kernel-server` (no Lane-RUN ownership of
  `cognitive-runtime` / `cognitive-management`)
- Status: **done** for P1-T04 acceptance (auth/size/timeout/concurrency/restart).
  Implementation + unit tests landed; CI Ubuntu/Windows-MSVC SUCCESS on PR #96
  (runs 30162481713 and 30162477963). Local Windows GNU remains non-supported
  (linker exit 121). Not a G0/B01-B12/Profile claim. P1-T05 is now unblocked by
  dependency only (still requires implementation).

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
| CI PR run | executed | [30162481713](https://github.com/agentkernel/cognitive-os/actions/runs/30162481713) SUCCESS (Ubuntu + Windows-MSVC) |
| CI push run | executed | [30162477963](https://github.com/agentkernel/cognitive-os/actions/runs/30162477963) SUCCESS (Ubuntu + Windows-MSVC) |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- Clients remain non-authority; timeout/limit rejections create no authority
  side effects.
- No secrets/tokens in timeout or concurrency error payloads.
- Existing auth/size/host/cookie/restart process tests from PR #95 remain.
- Fail-closed default: slow or excess traffic is refused, not buffered forever.

## 6. Next entry

1. Merge PR #96 when ready; rebase next work on `main`.
2. Start **P1-T05** readiness/status/doctor application service (depends on
   P1-T03 + P1-T04).
3. **P0-T03** still needs owner license/platform/distribution GO/NO-GO
   (blocks P0-T06 / G0 / installer path).
4. Suggested prompt: implement P1-T05 blocked/degraded/ready fact separation
   behind the authenticated Personal front door without claiming G0/Profile.

## 7. Snapshot

- PROGRESS updated: yes (P1-T04 done; no Profile claim)
- Formal Personal ledger updated: yes (`done`)
- PR: [#96](https://github.com/agentkernel/cognitive-os/pull/96)
- CI: 30162481713 + 30162477963 SUCCESS
- Commits: `4d84745` (implementation) + docs-done follow-up on same branch
