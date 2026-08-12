---
doc_id: user.limitations
locale: en
kind: reference
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
  - path: crates/cognitive-store/src/personal_backup.rs
  - path: apps/admin-cli/src/personal_cli/mod.rs
fingerprint: "sha256:fe7fad3b7cd74316ad51726448b08b5a037c59639ef428f6372e6c2bdd26a014"
non_claims:
  - This list reflects the recorded reading baseline; the live limitation set may shrink or grow with later merges — the fingerprint check flags staleness.
---

# Known limitations

An honest, verified list. "Implemented" here means the limitation itself is a
current fact of the code.

## Functional

- **Autonomous execution is not wired end-to-end**: Task admission does not enqueue
  scheduler work; the daemon runs one scheduler pass at startup only; tool
  executors and the independent verifier have test callers only.
- **No backup/restore command**; planning APIs only (secrets always excluded).
- **No Web UI, no Windows/macOS installation, no multi-agent orchestration**; the
  Pi shell has no resource/task browsing UX yet.
- Pi conversations are one-shot per exchange (no streaming, text only, fixed
  8192/1024 window constants at the client).
- `TaskApplicationService` implements `control`/`query_intent`, but no HTTP route
  exposes them yet.

## Operational quirks

- Unknown `/task/*` paths return HTTP 200 with a note instead of 404.
- The `cognitive` usage text omits the implemented `resource`/`task` verbs;
  `admin-cli install --mode official` usage omits the required `--package-id`.
- `kernel-server --personal` alone defaults to an ephemeral port (`127.0.0.1:0`);
  the canonical `48181` comes from `cognitive daemon start`.
- Readiness can say `ready` while your Provider key is stale (no live probe).
- Migration-backup files under `state/backups/` accumulate without pruning.
- A crashed migration can leave a stale `migration.lock` requiring manual removal.
- `pnpm run verify:local` pins outdated conformance counts (stale developer
  entry point).

## Platform

- Product platform: Linux x86_64 with user systemd; desktop needs a Secret Service
  keyring. WSL2 is an engineering environment, not a product target.
- Headless encrypted-vault operation is designed but not selectable today.
- Windows: daemon/CLI compile in CI, but no installer/service/credential backend
  exists (no ACL hardening on local files).
