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
  - path: docs/adr/0053-personal-web-ui-stack.md
tests:
  - apps/kernel-server/tests/p2_t18_local_token_csprng.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
fingerprint: "sha256:217edc29193ef48d56f395d250239f33a1c5bb31b18765f133c6fd4508277e22"
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
- **No Web UI SPA in this repository**: [ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md)
  accepted React + TypeScript + Vite and same-origin daemon `GET /ui` serving.
  The daemon now enforces the loopback Origin/Referer allowlist and returns
  `503` `not_available` when `data_dir()/ui/index.html` is absent. The SPA
  lives in the official `cognitiveos-clients` checkout at
  `D:\cognitiveos-clients\pc\web\` and is copied into `data_dir()/ui` for
  product serving. This kernel tree still must not contain `clients/**`.
  There is no Windows/macOS installation product and no multi-agent
  orchestration. The Pi shell has no resource/task browsing UX yet. Provider
  Control Plane in this phase is daemon API + CLI only — see
  [Provider Control Plane](./provider-control-plane.md).
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
