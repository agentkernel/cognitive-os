# Lane-RUN admin CLI Custom installation handoff

- Date: 2026-07-24
- Base: `codex/custom-agent-install@d64f08e`
- Scope: Batch A management CLI entry only; no Console, normative asset, conformance vector, or KRN store change.

## 1. Completed

- `admin-cli install` provides a Custom User-Provided entry point for
  `REQ-AGENT-INSTALL-001`: absence of `--confirm-custom-source yes` prints the
  full fixed `CUSTOM_USER_PROVIDED_RISK_NOTICE` and opens no installation store.
- The command derives the acknowledgement operator solely from the parsed,
  currently authorized management session (`principal://`); it does not accept
  an operator flag. It checks the management `agent.install` scope before
  touching the project or store.
- It canonicalizes a local project to `file://`, requires `package-lock.json`,
  rejects floating dependency declarations and symlinks, creates a deterministic
  in-memory bundle, and invokes only `npm ci --ignore-scripts --offline` with
  common model/package credentials removed. A changed project after preparation
  is rejected.
- Bundle, lockfile, adapter, sandbox and compatibility digests are returned in
  canonical JSON. The durable manager verifies the exact bundle acknowledgement
  before SQLite stage/commit; the result reports zero capability grants, Effects
  and Task completions. `--mode official` is deliberately blocked until a real
  trusted publisher attestation verifier exists.

## 2. Test-first and verification

- `apps/admin-cli/tests/agent_installation.rs` was added first. Its initial run
  failed 2/2 because `install` was an unknown verb. After implementation it
  passes 2/2: unconfirmed Custom is refused with the full notice and confirmed
  Custom commits with the session-derived operator and zero authority side
  effects. The fake package-manager fixture asserts the first command arguments
  are `ci --ignore-scripts`; no project lifecycle script executes in test.
- Local checks pass using the existing isolated LLVM-linker environment:
  `cargo test -p admin-cli -j 1` (13 tests),
  `cargo test -p cognitive-runtime -j 1` (45 tests),
  `cargo clippy -p admin-cli --all-targets -- -D warnings`,
  `cargo fmt --check`, and `git diff --check`.
- The first PR CI run exposed a test-fixture portability defect: the fixture
  created `npm.cmd`, while Linux resolves `npm`. `efc55d7` supplies an executable
  POSIX `npm` fixture on non-Windows and preserves the Windows `.cmd` fixture.
  Its first rerun still failed because the fixture incorrectly passed the entire
  inherited POSIX `PATH` as one `join_paths` segment. The pending follow-up uses
  the platform path separator to prepend the fixture directory. The focused
  Windows test/lint checks pass again, but no Linux CI pass is claimed until the
  follow-up run completes; both earlier Ubuntu failures remain evidence of the
  fixture defect, not implementation passes.
- Conformance runner and vectors were not changed or rerun. Existing reported
  pins remain 60 pass / 25 not-run / 0 fail; this local CLI test is not a
  behavior-vector pass or Profile claim.

## 3. Remaining blockers and non-claims

- The existing `InstallationStore` persists package/bundle/adapter/sandbox/
  compatibility digests but has no KRN-owned carrier for source mode,
  acknowledgement or lockfile digest. The CLI output is therefore not durable
  query evidence. Do not claim Batch A complete or release-ready until KRN
  supplies an atomic durable carrier and manager-only query surface.
- The install action uses the existing generic management-session gate, but no
  new management action schema/REQ was registered. It grants no capability and
  does not start an Agent, create an Effect, or complete a Task.
- Official provenance, Linux-native OS containment, six-family Pi lifecycle/I/O
  mappings, OOB/recovery execution evidence, full conformance execution and the
  REQ-PERF-004 campaign remain not completed. Windows-native sandbox remains
  unsupported; no WSL2 result may change that.

## 4. Next entry

- Suggested prompt: `docs/prompts/lane-run.md`.
- Branch: `codex/custom-agent-install`.
- First action: request/consume a Lane-KRN atomic installation-evidence carrier
  for the source acknowledgement and lockfile digest, then add the manager-only
  query command and crash-boundary tests before expanding Pi execution.

## 5. Snapshot

- PROGRESS updated: yes.
- Code commit: `0d67cab9c2468604671fb59e3643a897df963abf`
  (`feat(run): add confirmed custom install CLI`, `REQ-AGENT-INSTALL-001`).
- CI-fixture repair: `efc55d734ce6acab17f0862730e4b172c29bff12`
  (`test(run): make custom install fixture portable`).
- Documentation commit: this handoff commit (`docs(run): record custom install CLI limits`).
