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
  - path: apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["configure", "launch", "status"]
  - path: apps/admin-cli/src/personal_cli/provider.rs
tests:
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - apps/admin-cli/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/kernel-server/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/src/personal_cli/dsh.rs
fingerprint: "sha256:1724d6e2e5c511ae3e62506a5268331971ec134ac76ca661456c34c51df69762"
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
| `cognitive pi launch [--task-ref <task://URI>] [--append-system-prompt <absolute-path>]` | fail-closed Pi launch after full doctor readiness and exact version check; task-bound launches expose only daemon-governed WorkspaceRead/Search/Write/Patch and submit untrusted candidates to the task channel; `--append-system-prompt` forwards an existing absolute UTF-8 file to Pi and is not a Provider credential |
| `cognitive dsh configure --dsh-root <absolute-path> --adapter-root <absolute-path> --revision <git-object>` | write non-secret `dsh.json` (pinned dsh checkout, AKP adapter root, candidate-only adapter digest); revision must match the product pin |
| `cognitive dsh launch [--print] [--path b] [--task <prompt>]` | fail-closed dsh launch after daemon-owned system/database/secret/daemon ready (Pi and Pi `provider.json` may stay blocked); Path B uses the Cos Provider control plane + SecretStore, loads the pinned AKP plugin, and never treats a dsh response as Task completion; `--path a` is rejected here and is measurement-only via `paired-path.mjs` |
| `cognitive dsh web [--host 127.0.0.1] [--port 3080] [--no-open]` | start the **native** dsh control panel (`dsh --profile web`), not Personal `/ui/`. Default `http://127.0.0.1:3080`. Loopback only; `--host 0.0.0.0` is refused. Requires `apps/web/dist` from `pnpm run build` in the pinned dsh root. Path B still uses the daemon Provider proxy / SecretStore and overlays the Cos-assigned dsh model plus that account catalog. Do not put a second key in dsh `.env`. A panel session is never Task completion |
| `cognitive dsh apply` | publish the Cos dsh binding as Path B selected-model (`POST /personal/dsh/runtime` `op=apply`) and sync native Models to that bound account catalog (Cos web reloads; leftover grok is dropped when dsh is unbound). Chat uses the bound account (never DeepSeek when Cos assigned grok) |
| `cognitive dsh status` | authenticated observation of dsh sessions/fencing and optional bound pid liveness (`GET /personal/dsh/runtime`); not Task completion |
| `cognitive resource get/watch --family <memory\|skill\|tool\|context\|task\|runtime>` | read the private six-family projection (management channel) |
| `cognitive resource list/inspect --family <…> [--id <id>]` | common Resource Manager read envelope (management channel) |
| `cognitive resource bind\|unbind\|enable\|disable\|revoke --family <…> --id <id> --expected-version <n> --idempotency-key <key>` | common Resource Manager mutation onto existing Skill/Tool sinks; not generic create/execute/complete |
| `cognitive provider account create\|list\|show\|update\|delete` | management Provider Control Plane accounts; `--api-key-file` only; custom HTTP/private endpoints need `--allow-insecure-http` / `--allow-private-network`. Details: [Provider Control Plane](./provider-control-plane.md) |
| `cognitive provider key set\|rotate\|remove` | Secret Store key ops via the daemon; never SQLite |
| `cognitive provider models refresh\|list\|add\|set-price` | foreground discovery, manual models, prices |
| `cognitive agent binding set\|show\|list\|remove` | fixed pi/dsh account+provider+model binding; no fallback |
| `cognitive usage query` / `cognitive budget set\|list\|remove` / `cognitive alerts list\|acknowledge` / `cognitive audit query` | usage/cost/audit and observe-only budgets; `usage`/`audit` take no filters in this phase |
| `cognitive task watch [--resume-from N]` | follow the bounded Task watch stream (task channel) |
| `cognitive task evidence --task-ref <URI>` | read bounded redacted terminal evidence reconstructed from durable authority and Artifact CAS (task channel) |
| `cognitive backup [--output <dir>]` | write a secret-excluding digest-bound archive (no authority SQLite / provider-config / bearer) |
| `cognitive restore --archive <dir> [--preflight]` | preflight then overlay live files from a verified archive; `--preflight` mutates nothing |

Two honest quirks (also flagged in the generated
[CLI reference](../reference/cli-cognitive.md)): `--runtime-root <dir>` (accepted
by every verb) is the hermetic-test escape hatch that relocates the whole layout.

The separate `admin-cli` binary is the management fallback (inspect / stop / revoke
/ reconcile plus agent lifecycle verbs) and requires a privileged session document;
see the [admin-cli reference](../reference/cli-admin.md) and the
[management plane](../developer/management-plane.md) page.
