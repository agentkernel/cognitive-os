# 20260726 Personal P0-T06 Extension PoC Handoff

## 1. Task snapshot

- Task: `P0-T06` - Pi version, Extension, and RPC compatibility PoC.
- Date: 2026-07-26.
- Lane / branch: Lane-RUN / `lane/run-personal-p0-t06-extension-poc`.
- Base: `main@e2bead5`.
- Status: **in-progress**. This batch supplies a second independently testable
  compatibility fixture and an owner-approved, default-deny local-development
  Provider-key exception. It does not yet supply real Pi session/RPC load
  evidence.

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
- Replaced the adapter's ambient `DEEPSEEK_API_KEY` read with a default-deny
  `--allow-local-native-provider-secret-development` switch and an explicit
  `--provider-config-dir`. Only an available Linux native Secret Service,
  configured `deepseek` Provider, and `ProviderKeyService` resolution can
  supply the initial Pi child environment. Windows, CI, absent native backends,
  missing config, and missing secrets fail closed.
- ADR-0018 and ADR-0020 record the owner-approved scope: local only, no
  command-line/config/persistence/log/evidence secret, no containment claim,
  and automatic expiry at the P2 boundary.

## 3. Verification and evidence boundary

| Check | Status | Result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/cognitiveos-p0-t06-exception-target-two cargo test -p pi-agent-adapter --offline` (WSL) | executed | 16 passed / 0 failed |
| `npx --yes --package="@earendil-works/pi-coding-agent@0.81.1" --call "pi -e apps/pi-agent-adapter/fixtures/p0_t06_extension.ts --version"` | executed | printed `0.81.1`; this is pinned CLI resolution only, not Extension runtime-load evidence |
| credential-free Pi RPC clean-run | not-run to completion | process did not naturally exit; no Provider key was supplied, and no runtime-load claim is made |
| native Windows GNU focused Rust test | blocked environment | linker exit 121, the documented unsupported GNU baseline |

No conformance report, Profile manifest, Gate, B01-B12, C0/C1, or release
evidence was created or claimed.

## 4. Owner-approved development exception

The owner approved the narrow P0-T06 exception after the initial blocker was
recorded:

- local host only;
- exact explicit CLI opt-in only;
- material only from configured native Secret Service, never an ambient parent
  environment variable, CLI argument, config value, file, or prompt;
- initial Pi child environment only; Pi can pass its environment to
  descendants, so this is explicitly not containment;
- no Windows, CI, release, Gate, Profile, or production claim;
- expires at P2 exit unless removed, replaced by a local proxy, or explicitly
  re-approved.

The adapter implementation and ADR-0018/0020 now enforce/document that scope.

## 5. Remaining work and next entry

1. Perform an isolated real Extension session/RPC load test using the approved
   local-development route only when an actual Linux native Secret Service
   configuration is available; preserve only redacted evidence.
2. Archive integrity/source provenance verification remains Pi P2 work and
   must not be inferred from the compatibility pin.

Suggested prompt: "Continue P0-T06 from
`20260726-personal-p0-t06-extension-poc-handoff.md`: use only the ADR-0018
local Linux development exception to obtain redacted, isolated Pi Extension
session/RPC load evidence; do not make a containment, Gate, or Profile claim."

## 6. Commit and CI status

- Commit: `44509b498f5d886640375a425bce0ead25e3ebfd` recorded the initial
  fixture and blocker; this follow-up owner-decision implementation is pending
  a new commit.
- PR / push / CI: PR #101 has initial fixture CI green; follow-up CI pending.
- `personal-blog/` was not read, modified, staged, or included.
