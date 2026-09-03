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
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
tests:
  - personal/apps/kernel-server/tests/p2_t18_local_token_csprng.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
fingerprint: "sha256:b543703d966b52925d88696782667191f62a5367b6d9d38fca5ff2c170884ae9"
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
- **Personal 2.0 OPC capabilities are not current**: Windows host/tray/
  background, Project/Role/Employee/Routine/Attempt authority, Personal
  Conversation archive/Vault/retrieval, Pi-backed Assistant, managed DSH
  artifact/child/sandbox, contextual attention (Inbox is not first-level
  navigation), binding/budget enforcement, OPC UI and X
  connector are `Requires-backend`/`Requires-environment`.
- **Installed Agent target is narrow**: DSH is the only 2.0 runtime
  qualification target. Existing dsh Path B does not prove the Windows managed
  artifact; Pi remains a hidden Assistant target, and Hermes/Codex/Cursor are
  future candidates. No native DSH UI/conversation synchronization is planned.
- **Fixed acceptance is not user or release evidence**: unparked Phase 11 T15 uses
  N=15 Windows OPC scenarios and is not the prototype-completeness mutex. None has run. Canvas/ordinary CI cannot prove
  human desirability, usability, adoption, willingness to pay, support,
  release/Gate readiness, or Agent benefit.
- **Frozen-prototype `/ui/` completeness is a separate plan phase**: default
  walkable scenes on daemon `/ui/` are Phase 12 cards. They are not a canvas
  pixel replica, not 2.1, and not T15. Dual Track: no authority yields empty or
  Requires-backend; zero fake Create/Activate/Approve.
- **Phase 12 closing does not mean a Member really works**: as of 2026-09-02
  hosted DSH is a start skeleton, `runs`/`outputs` show only the process axis,
  Settings still defers connections to the legacy `/providers` page, Memory
  correct/forget has no OPC surface, no visual specification exists, and the
  Windows native environment is not provisioned. Formal-plan **Phase 13**
  (`P13-T01`–`T13`) owns these gaps card by card; Phase 13 done is still not
  release / signing / B01-W. `P13-T03` closed the hidden Pi assistant gap: its
  four turns now really run the exact pinned Pi through the daemon Provider
  proxy (Linux evidence only; the Windows Pi route stays `not-run` until
  `P13-T13`), and an unbound Provider yields a Settings pointer, not a chat box.
- **Project group chat never Approves**: inside a Project the right rail layers
  the Owner / manager / Members conversation over the Personal Assistant;
  `@manager` / `@member` mint digest-bound previews; Confirm stays on the
  Projects canvas. Secret-shaped chat is refused and pointed at Settings.
  Windows host E2E is `not-run` until `P13-T13`.
- Budget alerts are observe/query only; they do not block or reroute Provider calls.
- Custom endpoints are OpenAI-compatible only; third-party Anthropic-compatible
  URLs are refused. `cognitive usage query` and `cognitive audit query` take no
  filters; usage JSON includes labelled events (`cost` / `cost_label`
  actual|estimated|unknown, never `0`), `binding_explanation` layers, and
  separated `account` vs `quota` objects.
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
- `pnpm run verify:local` (developer entry point) is usable only inside a
  local MSVC-override directory; its pins now match `ci.yml` (re-pinned
  2026-09-03) but its output is local development evidence, never a Gate,
  release or Profile result.

## Platform

- Product platform: Linux x86_64 with user systemd; desktop needs a Secret Service
  keyring. WSL2 is an engineering environment, not a product target.
- Personal 2.0 is Windows-first, but the qualified native Windows development
  environment and B01-W do not exist. Linux, WSL, ordinary CI, Canvas, and
  Windows GNU evidence do not transfer.
- Native mobile/device pairing/E2E relay remote is deferred to Personal 2.1 and
  remains host-online only; no secret downlink is planned.
- Headless encrypted-vault operation is designed but not selectable today.
- Windows: daemon/CLI compile in CI and a Credential Manager backend plus
  installer/scheduled-task templates exist, but the B01-W install campaign has
  not run — no installable Windows product, and no ACL hardening on local files.
  Local bootstrap/session tokens do use the OS CSPRNG; that correction does not
  strengthen Windows file ACLs.
- A runtime left with the pre-CSPRNG bootstrap shape intentionally fails daemon
  startup after upgrade. Stop the daemon and remove only `local-bootstrap.secret`
  from its private runtime directory so the next start can mint a replacement.
