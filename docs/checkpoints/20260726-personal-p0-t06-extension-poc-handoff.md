# 20260726 Personal P0-T06 Extension PoC Handoff

## 1. Task snapshot

- Task: `P0-T06` - Pi version, Extension, and RPC compatibility PoC.
- Date: 2026-07-26.
- Lane / branch: Lane-RUN / `lane/run-personal-p0-t06-extension-poc`.
- Base: `main@e2bead5`.
- Status: **blocked**. This batch supplies a second independently testable
  compatibility fixture, but the task cannot safely advance to a real Pi
  session/RPC load check until the Provider-key boundary is resolved.

## 2. Completed atomic part

- Added the pinned Pi `0.81.1` TypeScript extension fixture at
  `apps/pi-agent-adapter/fixtures/p0_t06_extension.ts`.
- The fixture returns `{ trusted: "no" }` for `project_trust`, blocks Pi
  built-in `write`, `edit`, and `bash` at `tool_call`, and sets only
  session-local UI status on `session_start`.
- Added Rust guards to assert those surfaces and to reject provider
  credential or durable-state access in the fixture source.
- The fixture does not access a database, secret, network, filesystem, or
  CognitiveOS authority. It creates no Effect, capability, Task transition,
  or completion evidence.

## 3. Verification and evidence boundary

| Check | Status | Result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/cognitiveos-p0-t06-extension-target cargo test -p pi-agent-adapter --test p0_t06_compatibility --offline` (WSL) | executed | 7 passed / 0 failed |
| `npx --yes --package="@earendil-works/pi-coding-agent@0.81.1" --call "pi -e apps/pi-agent-adapter/fixtures/p0_t06_extension.ts --version"` | executed | printed `0.81.1`; this is pinned CLI resolution only, not Extension runtime-load evidence |
| credential-free Pi RPC clean-run | not-run to completion | process did not naturally exit; no Provider key was supplied, and no runtime-load claim is made |
| native Windows GNU focused Rust test | blocked environment | linker exit 121, the documented unsupported GNU baseline |

No conformance report, Profile manifest, Gate, B01-B12, C0/C1, or release
evidence was created or claimed.

## 4. Security blocker requiring owner decision

`apps/pi-agent-adapter/src/main.rs` currently reads `DEEPSEEK_API_KEY` and
injects it into the Pi child-process environment. This conflicts with the
Personal plan and session instruction that a Provider API key must remain in
the native Secret Store and must not enter Pi or an environment variable.

Do not run a real Pi session, RPC execution, or Provider smoke while this
boundary is unresolved. The formal Personal ledger and PROGRESS both mark
`P0-T06` as `blocked`.

## 5. Remaining work and next entry

1. Obtain owner decision on the Provider-auth boundary before changing the
   candidate launcher or attempting a real Extension session/RPC load test.
2. After a decision, perform an isolated real Extension load test without a
   Provider key and preserve only redacted evidence.
3. Archive integrity/source provenance verification remains Pi P2 work and
   must not be inferred from the compatibility pin.

Suggested prompt: "Resolve the P0-T06 Provider-key boundary recorded in
`20260726-personal-p0-t06-extension-poc-handoff.md`; do not put a Provider
key in Pi or an environment variable. After the owner decision, continue the
isolated Pi Extension session/RPC load PoC."

## 6. Commit and CI status

- Commit: pending for this blocked atomic batch.
- PR / push / CI: pending; no claim before the owner decision response.
- `personal-blog/` was not read, modified, staged, or included.
