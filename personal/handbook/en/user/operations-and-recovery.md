---
doc_id: user.operations-recovery
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/readiness.rs
    symbols: ["evaluate_personal_readiness"]
  - path: personal/apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/pinned_https.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/observation.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/six_resource_doctor.rs
  - path: personal/apps/admin-cli/src/personal_cli/daemon.rs
  - path: personal/apps/kernel-server/src/personal/user_backup.rs
    symbols: ["handle"]
  - path: personal/apps/admin-cli/src/personal_cli/backup.rs
  - path: personal/apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["launch"]
  - path: personal/apps/admin-cli/src/personal_cli/provider.rs
  - path: personal/crates/cognitive-store/src/personal_backup.rs
    symbols: ["write_personal_backup_archive", "restore_personal_backup_archive", "plan_personal_lifecycle"]
  - path: personal/crates/cognitive-store/src/personal_db.rs
    symbols: ["prepare_personal_databases"]
  - path: personal/crates/cognitive-store/src/windows_host.rs
    symbols: ["WindowsHostStore", "WAKE_RECOVERY_STEPS"]
  - path: personal/crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
tests:
  - personal/apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - personal/apps/kernel-server/tests/p2_t27_backup_restore.rs
  - personal/apps/admin-cli/tests/p2_t27_backup_restore.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - personal/crates/cognitive-store/tests/p1_t01_layout_migrations.rs
  - personal/crates/cognitive-store/tests/p11_t02_windows_host.rs
fingerprint: "sha256:f87bd697910ef94b16069d339ea9fbc66e91d05118ea3186e2c54675a3243750"
non_claims:
  - "`ready` is a configuration/liveness projection, not a live Provider or end-to-end guarantee. Backup/restore excludes secrets and does not copy authority SQLite."
---

# Operations and recovery

A missed Routine occurrence is a visible ledger row (`GET /management/project/v1/routine.ledger`), not a silent drop. Sleep/shutdown/daemon-stop/network-loss/locked SecretStore/Provider outage also become explicit Windows host offline/missed facts (`POST /management/host/v1/offline.record`). Clock/sleep/restart host E2E remains `not-run`.

## Daily checks — `implemented`

- `cognitive status` / `cognitive doctor`: six components (system, database,
  secret, provider, daemon, pi) with `blocked | degraded | ready` levels plus
  `first_conversation_ready`. The `provider` component resolves the configured
  `secret_ref`, so a Provider whose stored key was removed reports
  `provider_secret_unresolvable` and blocks rather than claiming ready; re-run
  `cognitive init` to store the key again. One evaluation uses one loaded config
  snapshot for provider, model/digest and secret resolution, so an atomic config
  replacement cannot mix facts from two versions. One status/doctor evaluation
  also binds one SecretStore for the secret probe and the provider resolve —
  no secret material is retained, and a later request is evaluated again
  rather than served from a stale-ready TTL. Doctor adds redacted six-resource,
  headless-vault, and operability sections (currently static
  `not_run`/`not_configured` reports — redaction validators more than live probes).
- `GET /personal/health` (no auth) is liveness only — the installer and service
  controller use it; don't read readiness into it.
- `GET /ui` (no auth) serves the same-origin static Web UI from `data_dir()/ui`
  with CSP `default-src 'self'`. A missing bundle is `503` `not_available`
  (`LOCAL_UI_BUNDLE_UNAVAILABLE`) and is not readiness. A browser `Origin` or
  `Referer`, when present, must be this daemon's loopback HTTP origin.
- Service logs: `journalctl --user -u cognitiveos-personal.service`. CLI
  `cognitive daemon start` also appends kernel-server stdout/stderr to
  `state/cognitiveos/daemon.log` (mode `0600`); scheduler skip lines are not a
  public HTTP fact. A private-candidate adapter rejection is retained there as a
  redacted diagnostic (`sk-` / `api_key=` / `token=` spans stripped).
- `cognitive pi launch --print` is the bounded non-interactive Pi path: it keeps
  the public CLI attached while Pi reads its prompt from stdin and exits. It still
  requires the daemon's full ready projection, disables Pi-native tools, passes no
  Provider credential to Pi, and must not be substituted with direct
  daemon/private-candidate calls. `--append-system-prompt <absolute-path>` forwards
  an existing non-empty UTF-8 file to Pi; it is not a Provider credential and the
  file bytes are not printed.
- `cognitive dsh launch --print` is the bounded non-interactive dsh Path B:
  it requires daemon-owned system/database/secret/daemon ready (Pi and Pi
  `provider.json` may stay blocked), loads the pinned AKP plugin, and never
  treats a dsh response as Task completion.
  Direct Flash (`--path a`) is refused; use `personal/packages/dsh-akp-adapter/scripts/paired-path.mjs`
  for same-host Path A vs Path B measurement only.
- `cognitive dsh web` starts the native dsh control panel (`dsh --profile web --no-open`)
  at `http://127.0.0.1:3080` by default. This is not Personal `/ui/`. Bind is
  loopback only (`--host 0.0.0.0` is refused). The pinned dsh root must contain
  `apps/web/dist` (`pnpm run build`). Path B still uses the daemon Provider
  proxy and SecretStore; the Models page should not ask for a second DeepSeek
  key. Do not copy SecretStore material into dsh `.env`. A panel session is
  never Task completion. On SSH guests pass `--no-open` (already the product default).
- `cognitive dsh apply` publishes the Cos dsh Agent binding as Path B
  selected-model (`POST /personal/dsh/runtime` `op=apply`) and writes the native
  Models overlay from that bound account catalog. Cos-installed web reloads so
  conversation and Models match the control plane; unbinding dsh drops those
  models (including grok). Chat uses the bound account (never DeepSeek when Cos
  assigned grok). Apply fails closed when web is INACTIVE or the model is not in
  that account catalog. A leftover grok-on-DeepSeek binding fail-closes Path B
  with `PERSONAL_PROVIDER_BINDING_MISMATCH` instead of posting to DeepSeek.
- `cognitive dsh status` reads `GET /personal/dsh/runtime`: INACTIVE / ACTIVE /
  CRASHED from process-local sessions plus an optional bound pid. Linux liveness
  is `/proc/{pid}` existence only (never cmdline/environ). It is not an
  authority writer. UI-up is not Task completion.

## Stop, restart, stale state — `implemented`

`cognitive daemon stop` signals the recorded PID and removes `daemon.lock` plus the
endpoint document only after the process is confirmed gone; a live-looking lock is
never deleted. On every start the daemon re-runs migrations idempotently, recovers
consumed worker handoffs, repairs only missing Loop/Budget/scheduler
prerequisites for current admitted contracts without resetting existing rows,
and republishes the endpoint atomically. Only then does one periodic scheduler
worker start; orderly exit cancels, wakes, and joins it before daemon state is
released.

Tool overlay and pinned HTTPS origin files live under the Personal data
directory (`personal-tool-lifecycle.json`, `personal-pinned-https.json`). A
restart reloads them; they are not Artifact CAS objects. Production
HttpFetchReadOnly stays fail-closed until a management caller with an
authorized campaign pins exact HTTPS origins. Bounded O2/O3/O4/O5/O13
observation samples live in `personal-observation-plane.json` (O2–O4) and the
authority event log (O5/O13) and survive restart; empty windows return
`observed_zero` with a named negative control.

## Database safety — `implemented`

Databases live under XDG state (`authority.sqlite`, `installation.sqlite`, WAL
mode, 0600). Every migration apply first writes a timestamped backup under
`state/backups/` (never pruned automatically). Derived data (Memory FTS index) is
rebuildable from authority rows; forgetting a Memory can never be undone by an
index rebuild. Authority migrations now include v24 append-only Memory/Skill
consumption records; a later session may reuse exact pins, but forget, revoke,
digest drift, or a competing record fail closed instead of resurrecting
forgotten facts. Management Memory/Skill lifecycle rows and Skill revision
lineage remain inspectable after daemon restart, including through
`cognitive resource list|inspect`. Public Memory remember may send
unsealed owner fields; the daemon composes sealed headers from its persisted
governance root.

## Crash and unknown-outcome recovery — `implemented` at the engine level

Recovery follows a fixed eight-step order (fence old writers → replay history →
reconcile every in-flight Effect with its **original** idempotency key → reauthorize
→ rebuild context → resume or quarantine). The deterministic management fallback
(`admin-cli reconcile`) drives the same sequence without any model dependency —
with no executor configured, still-unknown outcomes quarantine (fail-safe) rather
than resolve. Native HTTP attempts persist before egress and remain indeterminate
after restart until a terminal receipt exists. Workspace mutations use durable
original-key receipts; matching file bytes alone are not execution proof, and
orphan staging is cleaned conservatively on restart. Production HTTP fetch
staging also consults the campaign-authorized pinned origin registry; without
a pin the allowlist stays empty and the request fails closed.

A successful Task admission is also crash-atomic inside the authority database:
the contract, `DRAFT` governed Task, `START` Loop, hard Budget, and runnable
scheduler row appear together. Daemon-owned Context authorization facts and the
tenant `personal` revocation epoch are persisted immediately before that CAS as
idempotent owner-local policy, not a client capability channel. A failure before commit leaves none of those
admission members, while a crash after the success response reopens the complete
publication. Startup can repair a missing legacy Task projection without
resetting any existing lifecycle state.

When verification starts, the closed Effect pin, verification request, and Loop
`ACT -> VERIFY` publication are one crash-atomic authority transaction. A stale
writer or Loop leaves none of those new members.

Task completion is a later authority boundary. The daemon rechecks the latest
independent passed report and every referenced CAS artifact, then SQLite
rechecks the fixed Effect version, complete closed Effect set, current epoch and
Task CAS in each candidate/acceptance transaction. A crash between the two
transitions leaves `CANDIDATE_COMPLETE` for the same evidence-bound acceptance
retry; duplicate acceptance cannot write a second completion.

Registered native Tools start enabled. A management session can disable,
quarantine, or revoke one by `operation_id`; Agent exposure follows that overlay
immediately and never rewrites the immutable descriptor. Ordinary Task callers
can read the current least exposure set and record a selection receipt, but they
cannot enable, disable, quarantine, or revoke Tools.

## Provider Control Plane — `partial`

`cognitive provider …`, `cognitive agent binding …`, `cognitive usage query`,
`cognitive budget …`, `cognitive alerts …`, and `cognitive audit query` call the
daemon management surface. The localhost Web UI uses the same routes as a daemon
client (`GET /ui/`); keys enter SecretStore only through the management key POST
or `--api-key-file`, never SQLite, argv, or browser storage. There is no desktop
control panel. Budget alerts are observe-only and do not block or
reroute calls. Custom HTTP or private-network endpoints require durable
`--allow-insecure-http` / `--allow-private-network` grants. Binding updates may
send optional `expected_revision`; a mismatch is HTTP 409
`PROVIDER_BINDING_REVISION_STALE`. Operator steps, worked commands, and common
failures:
[Provider Control Plane](provider-control-plane.md).

## Update, rollback, and uninstall — `partial`

There is no public `cognitive update` or `cognitive uninstall`. The Linux
product path is:

- **Update:** re-run the inspected signed `install.sh` from a newer bundle. The
  previous version stays on disk until the new activation succeeds. Failure
  compensates (previous version, unit, and `active-version` pointer restored; no
  success receipt).
- **Stop / disable the user unit:** `cognitive daemon stop`, then
  `systemctl --user disable --now cognitiveos-personal.service`. That does not
  delete Secret Store items, `authority.sqlite`, or XDG data.
- **Authority-path plan:** `plan_personal_lifecycle` records Update/Rollback/
  Uninstall intent. Uninstall refuses a Secret target and refuses Data unless
  deletion is explicitly confirmed. Committing the plan still does not delete
  host files.
- **Managed Pi uninstall** remains the `admin-cli` lifecycle listed under
  backup/restore below.

The step index is [Linux RC operator map](rc-and-support.md).

## Reflection and Member Runtime — `partial`

Daemon `/ui/` Member configuration has a Reflection tab. Generate from facts
calls `POST /management/project/v1/reflection.generate`. A model self-report is
not an improvement. Request runtime preview mints a `member-runtime-revision`
canvas; Apply is Owner canvas Confirm only (no Admit). Rollback appends a new
Employee revision. Role Template preview does not copy the Member into another
Project. Task-channel aliases return 403.

## Backup and restore — `partial`

`cognitive backup [--output <dir>]` writes a digest-bound directory archive of
config/data/state/artifacts plus a Memory/Skill export sidecar. It never copies
`authority.sqlite`, `provider-config.json`, bootstrap secrets, or bearer files.
`cognitive restore --archive <dir>` (or `--archive-id`) runs schema/digest
preflight, then overlays live files from a staging tree; a failure rolls the
snapshot back. `--preflight` verifies without mutation. The same operations are
available on the management channel as `POST /management/resource/v1/backup` and
`.../restore`. After restore, re-enter the Provider key with `cognitive init`
if the Secret Store item is missing. Public `admin-cli` callers cover managed Pi
install→activate-root→register→activate→pause/resume→upgrade/rollback→stop→
recover→uninstall. Focused tests record byte-equal restore and a finite restore
wall time as hypothesis-only facts; this page does not claim RTO/RPO or Gate
results.
