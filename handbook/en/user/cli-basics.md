---
doc_id: user.cli-basics
locale: en
kind: guide
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["parse_cognitive_args", "COGNITIVE_USAGE"]
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: apps/admin-cli/src/personal_cli/backup.rs
tests:
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - apps/admin-cli/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/kernel-server/tests/p2_t27_backup_restore.rs
fingerprint: "sha256:e1d7c616abd8ee04bade7f6f4ce8cce89fdbf7d259d663eca4af960c36d15894"
non_claims:
  - The CLI is a non-authority client; nothing it prints implies Task completion or Gate results.
---

# CLI basics

The `cognitive` binary is the deterministic product entry. It never writes authority
state directly — it prepares configuration, spawns/stops the daemon process, and
reads authenticated projections. Exit codes: `0` success, `1` operational error,
`2` usage error; success output is JSON.

| Verb | What it actually does |
|---|---|
| `cognitive init` | prepare XDG layout + databases (with backups), store the Provider key into the Secret Service or bind an already-stored item with `--reuse-existing-secret-binding`, probe the Provider, persist `provider.json` and `selected-model.json` |
| `cognitive status` | authenticated component projection (system, database, secret, provider, daemon, pi) |
| `cognitive doctor` | the same projection plus redacted diagnostic sections |
| `cognitive daemon start` | spawn `kernel-server --personal` bound to `127.0.0.1:48181` (override with `--bind`, `--kernel-server`, or `COGNITIVE_KERNEL_SERVER`); append stdout/stderr to `state/cognitiveos/daemon.log` (mode `0600`) |
| `cognitive daemon status` | report daemon lock/endpoint liveness |
| `cognitive daemon stop` | signal the recorded PID; remove lock/endpoint only after confirmed exit |
| `cognitive pi configure` | write non-secret `pi.json` (absolute executable + extension paths) |
| `cognitive pi launch [--task-ref <task://URI>]` | fail-closed Pi launch after full doctor readiness and exact version check; task-bound launches expose only daemon-governed WorkspaceRead/Search/Write/Patch and submit untrusted candidates to the task channel |
| `cognitive resource get/watch --family <memory\|skill\|tool\|context\|task\|runtime>` | read the private six-family projection (management channel) |
| `cognitive task watch [--resume-from N]` | follow the bounded Task watch stream (task channel) |
| `cognitive task evidence --task-ref <URI>` | read bounded redacted terminal evidence reconstructed from durable authority and Artifact CAS (task channel) |
| `cognitive backup [--output <dir>]` | write a secret-excluding digest-bound archive (no authority SQLite / provider-config / bearer) |
| `cognitive restore --archive <dir> [--preflight]` | preflight then overlay live files from a verified archive; `--preflight` mutates nothing |

Two honest quirks (also flagged in the generated
[CLI reference](../reference/cli-cognitive.md)): the built-in usage text does not yet
list `resource`/`task`, and `--runtime-root <dir>` (accepted by every verb) is the
hermetic-test escape hatch that relocates the whole layout.

The separate `admin-cli` binary is the management fallback (inspect / stop / revoke
/ reconcile plus agent lifecycle verbs) and requires a privileged session document;
see the [admin-cli reference](../reference/cli-admin.md) and the
[management plane](../developer/management-plane.md) page.
