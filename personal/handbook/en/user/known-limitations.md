---
doc_id: user.limitations
locale: en
kind: reference
audience: [user]
status: implemented
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
  - path: personal/apps/kernel-server/src/personal/auth.rs
  - path: personal/crates/cognitive-store/src/personal_backup.rs
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
  - path: docs/adr/0053-personal-web-ui-stack.md
  - path: docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
tests:
  - personal/apps/kernel-server/tests/p2_t18_local_token_csprng.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
fingerprint: "sha256:3044376c1a9ce03ea6e36107c9764bb0a2d80141bbdc30249add9687ab09b4ef"
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
- **Backup/restore excludes secrets and authority SQLite**; `cognitive backup` /
  `restore` and the management HTTP routes write a digest-bound archive and
  overlay live files after preflight. Provider keys stay in the Secret Store
  and must be re-entered after a machine move. Managed Pi recover is not yet
  wired on this path.
- **Control Plane Web UI is not in the Linux RC claim**:
  [ADR-0053](../../../../docs/adr/0053-personal-web-ui-stack.md) accepted React +
  TypeScript + Vite and same-origin daemon `GET /ui` serving. After
  [ADR-0054](../../../../docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
  the SPA lives in this repository at `clients/pc/web/` and is copied into
  `data_dir()/ui` for product serving. The daemon enforces the loopback
  Origin/Referer allowlist and returns `503` `not_available` when
  `data_dir()/ui/index.html` is absent. HTTP cancel and class-C Agent lifecycle
  remain `not-run`. There is no Windows/macOS installation product and no
  multi-agent orchestration. The Pi shell has no resource/task browsing UX yet.
  The adopted Personal 2.0 desktop-first redesign has not been applied to this
  current SPA.
  Operator steps: [Provider Control Plane](provider-control-plane.md). Linux RC
  claim set: [Linux RC operator map](rc-and-support.md).
- **Personal 2.0 target capabilities are not current APIs**: the current
  resource model and Resource Manager remain six-family; the seventh MCP family
  and federated resources are `Requires-backend`. Account Hub has no
  browser-profile, Agent credential-file, subscription, or OAuth import
  mechanism. Goal/Plan revision APIs, vendor-specific Agent conversation
  adapters, and multi-Agent supervision do not exist. Pi remains the only
  qualified Agent.
- Budget alerts are observe/query only; they do not block or reroute Provider calls.
- Custom endpoints are OpenAI-compatible only; third-party Anthropic-compatible
  URLs are refused. `cognitive usage query` and `cognitive audit query` take no
  filters; the usage JSON is `event_id` / `account_id` / `cost_micros` /
  `cost_status` only.
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
- `cognitive doctor` `first_conversation_ready` is conversation-shell readiness,
  not C1/C2 Task lifecycle; an admitted Task can stay `DRAFT` until the scheduler
  acquires a lease. CLI `cognitive daemon start` retains kernel-server stdio in
  `state/cognitiveos/daemon.log` rather than `/dev/null`.
- Readiness can say `ready` while your Provider key is stale (no live probe).
- Restarting/replacing the daemon invalidates dsh Path B's process-local
  management session. The new daemon projects dsh as `INACTIVE`, so
  `cognitive dsh apply` is rejected and cannot recover the stale bearer.
  Do not extract the bearer for a direct probe; restart `cognitive dsh web`,
  then check `cognitive dsh status`. Reserve `apply` for supported
  binding/model overlay synchronization when the daemon has not restarted and
  the runtime is already `ACTIVE`. Persisted account `active` is not a live
  SecretStore-resolution result; discovery/proxy use performs live resolution,
  so a locked or changed store remains a separate possible cause. This is an
  [open tracked defect](../../../../docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)
  with an operational recovery, not a product-code fix.
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
