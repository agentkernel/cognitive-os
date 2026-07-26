# 20260726 Personal P0-T06 Pi Compatibility Handoff

## 1. Task snapshot

- Task: `P0-T06` - Pi version, Extension, and RPC compatibility PoC.
- Date: 2026-07-26.
- Lane / branch: Lane-RUN / `lane/run-personal-p0-t06-pi-compat`.
- Base: `main@f9c16a19b71c6bd49429a019f0ef299426389ce5`.
- Status: **in-progress**. This commit completes only the first independently
  testable atomic part; it does not satisfy the full P0-T06 acceptance gate.

## 2. Completed atomic part

- Added `PiCompatibilityPin` for the reviewed
  `@earendil-works/pi-coding-agent@0.81.1` package metadata: npm SRI, source
  commit, repository path, and Node.js engine.
- Candidate launch now runs `pi --version` and rejects a version mismatch
  before reading `DEEPSEEK_API_KEY` or creating the candidate process.
- Added a strict-LF JSONL parser and focused fixtures covering CRLF input
  normalization, U+2028 preservation, malformed records, non-object records,
  and bare-CR rejection.
- No REQ-ID, schema, registry, transition, vector, authority state, Effect,
  Task completion, or Provider secret behavior changed.

## 3. Evidence and verification

| Check | Status | Result |
|---|---|---|
| `npm view @earendil-works/pi-coding-agent@0.81.1 version dist.integrity gitHead repository engines --json` | executed | version `0.81.1`; SRI `sha512-r6ov...N/P8A==`; gitHead `20be4b18d4c57487f8993d2762bace129f0cf7c6`; Node `>=22.19.0` |
| WSL `CARGO_TARGET_DIR=/tmp/cognitiveos-p0-t06-target cargo test -p pi-agent-adapter --test p0_t06_compatibility --offline` | executed | 5 passed / 0 failed |
| Native Windows GNU test | blocked environment | linker exit 121, the documented unsupported GNU baseline; no Windows test-pass claim |
| Full workspace build/test/clippy | not-run | not proportionate to this focused atomic part; CI remains pending |
| Pi binary / Extension execution | not-run | no real Provider key, no external Pi session, and no Extension loaded |
| G0, B01-B12, C0/C1, Profile | not-run | no claim |

The SRI and source commit are recorded compatibility pins only. They are not
archive verification or trusted provenance evidence; Pi supply-chain verifier
work remains P2 in `PI-AGENT-INTEGRATION-PLAN.md`.

## 4. Remaining P0-T06 acceptance work

1. Build a pinned-package Extension PoC that covers `project_trust`, tool
   replacement/interception, and a session event without granting the
   Extension database, secret, or authority access.
2. Run its actual compile/load check against the pinned Pi package in an
   isolated temporary environment; do not vendor Pi/Node into a release
   artifact.
3. Extend the focused test suite and update the formal ledger only if all
   P0-T06 acceptance conditions pass. Until then, retain `in-progress` and do
   not close G0.

## 5. Safety and status boundaries

- Pi remains a non-authority candidate source; the adapter does not create an
  Effect, grant a capability, write SQLite, or complete a Task.
- The version check intentionally occurs before the scoped Provider key is
  read. The key is neither recorded in this handoff nor used by any test.
- No finding or machine-contract drift was found. No entry was added to the
  findings ledger.
- `personal-blog/` was not read, modified, staged, or included.

## 6. Snapshot and next entry

- Formal Personal ledger updated: yes (`P0-T06` is `in-progress`).
- PROGRESS updated: yes (atomic scope and non-claim recorded).
- Commit / push / PR / CI: pending at handoff creation; fill after the atomic
  commit and CI result.
- Suggested prompt: continue `P0-T06` by creating an isolated, pinned Pi
  Extension compatibility compile/load PoC for project trust, tool
  interception, and session events; keep Pi non-authority and do not use a
  real Provider key.
