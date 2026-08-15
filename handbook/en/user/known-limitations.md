---
doc_id: user.limitations
locale: en
kind: reference
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
  - path: apps/kernel-server/src/personal/auth.rs
  - path: crates/cognitive-store/src/personal_backup.rs
  - path: apps/admin-cli/src/personal_cli/mod.rs
tests:
  - apps/kernel-server/tests/p2_t18_local_token_csprng.rs
fingerprint: "sha256:910cfb312c693ecf4d69653803f5c7e15b2bc4a14f470b665b50d7fc9f75a32d"
non_claims:
  - This list reflects the recorded reading baseline; the live limitation set may shrink or grow with later merges — the fingerprint check flags staleness.
---

# Known limitations

An honest, verified list. "Implemented" here means the limitation itself is a
current fact of the code.

## Functional

- **Autonomous execution is not wired end-to-end**: Task admission does enqueue
  its complete scheduler bootstrap and a post-bind periodic worker reaches
  candidate admission. Parameter-free WorkspaceRead has a durable production
  Effect caller, independent verifier, and an evidence-bound Task acceptance
  caller. Exact native `22c3f502` reached public C1 `COMPLETED`. Open-Effect,
  superseded-report and missing-CAS negatives are written; a stale fixed
  post-state negative is still open. The other Tool request carriers remain
  unwired.
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
- Windows: daemon/CLI compile in CI and a Credential Manager backend plus
  installer/scheduled-task templates exist, but the B01-W install campaign has
  not run — no installable Windows product, and no ACL hardening on local files.
  Local bootstrap/session tokens do use the OS CSPRNG; that correction does not
  strengthen Windows file ACLs.
- A runtime left with the pre-CSPRNG bootstrap shape intentionally fails daemon
  startup after upgrade. Stop the daemon and remove only `local-bootstrap.secret`
  from its private runtime directory so the next start can mint a replacement.
