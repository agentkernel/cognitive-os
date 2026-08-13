---
doc_id: user.operations-recovery
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/readiness.rs
    symbols: ["evaluate_personal_readiness"]
  - path: apps/kernel-server/src/personal/six_resource_doctor.rs
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: crates/cognitive-store/src/personal_backup.rs
    symbols: ["plan_personal_backup_inventory"]
  - path: crates/cognitive-store/src/personal_db.rs
    symbols: ["prepare_personal_databases"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
tests:
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - crates/cognitive-store/tests/p1_t01_layout_migrations.rs
fingerprint: "sha256:3c066a965e9e8e7c3c0ddcf80f29fbd138dee3a4498b820557bfcec4f6ff0fc5"
non_claims:
  - "`ready` is a configuration/liveness projection, not a live Provider or end-to-end guarantee. Backup/restore has no runnable command today."
---

# Operations and recovery

## Daily checks — `implemented`

- `cognitive status` / `cognitive doctor`: six components (system, database,
  secret, provider, daemon, pi) with `blocked | degraded | ready` levels plus
  `first_conversation_ready`. The `provider` component resolves the configured
  `secret_ref`, so a Provider whose stored key was removed reports
  `provider_secret_unresolvable` and blocks rather than claiming ready; re-run
  `cognitive init` to store the key again. One evaluation uses one loaded config
  snapshot for provider, model/digest and secret resolution, so an atomic config
  replacement cannot mix facts from two versions. Doctor adds redacted six-resource,
  headless-vault, and operability sections (currently static
  `not_run`/`not_configured` reports — redaction validators more than live probes).
- `GET /personal/health` (no auth) is liveness only — the installer and service
  controller use it; don't read readiness into it.
- Service logs: `journalctl --user -u cognitiveos-personal.service`.

## Stop, restart, stale state — `implemented`

`cognitive daemon stop` signals the recorded PID and removes `daemon.lock` plus the
endpoint document only after the process is confirmed gone; a live-looking lock is
never deleted. On every start the daemon re-runs migrations idempotently, recovers
consumed worker handoffs, and republishes the endpoint atomically.

## Database safety — `implemented`

Databases live under XDG state (`authority.sqlite`, `installation.sqlite`, WAL
mode, 0600). Every migration apply first writes a timestamped backup under
`state/backups/` (never pruned automatically). Derived data (Memory FTS index) is
rebuildable from authority rows; forgetting a Memory can never be undone by an
index rebuild.

## Crash and unknown-outcome recovery — `implemented` at the engine level

Recovery follows a fixed eight-step order (fence old writers → replay history →
reconcile every in-flight Effect with its **original** idempotency key → reauthorize
→ rebuild context → resume or quarantine). The deterministic management fallback
(`admin-cli reconcile`) drives the same sequence without any model dependency —
with no executor configured, still-unknown outcomes quarantine (fail-safe) rather
than resolve. Native HTTP attempts persist before egress and remain indeterminate
after restart until a terminal receipt exists. Workspace mutations use durable
original-key receipts; matching file bytes alone are not execution proof, and
orphan staging is cleaned conservatively on restart.

A successful Task admission is also crash-atomic inside the authority database:
the contract, `START` Loop, hard Budget, and runnable scheduler row appear
together. A failure before commit leaves none of those admission members, while
a crash after the success response reopens the complete publication. This does
not by itself mean the daemon's periodic worker loop or independent verifier is
wired.

## Backup and restore — `unavailable` as a user feature

Inventory/export/restore-preflight planning code exists (secret paths are always
excluded by design), but no `cognitive backup`/`restore` command or archive I/O is
wired yet. Today's honest fallback: stop the daemon, copy the XDG state/config
directories yourself, and remember that Provider keys are **not** in those files —
after restoring on a new machine, re-run `cognitive init` to re-enter the key.
