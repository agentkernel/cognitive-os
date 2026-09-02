# P13-T02 Hosted DSH real Attempt loop — closure

- Task: `P13-T02` **done** / slices `P13-T02/D01` **done**, `P13-T02/D02` **done** (on the PR head the formal plan / PROGRESS still read `in-progress → closing` because the active lease binds `P13-T02/D02`; the lease-closure commit on `main` records `done`, following the P13-T01 closure precedent)
- Branch: `personal/P13-T02-hosted-dsh-attempt` (remote deleted after merge; worktree `D:\agent-kernel-wt-p13-t02` removed)
- Lease: `lease/personal/P13-T02/hosted-dsh-attempt` → PARALLEL-LANES §3.1 (closed in the same closure delivery on `main`)
- PR: [#310](https://github.com/agentkernel/cognitive-os/pull/310) (merge revision recorded in the lease-closure commit on `main`)
- Required CI: [33676373077](https://github.com/agentkernel/cognitive-os/actions/runs/33676373077) **SUCCESS** at `f82bd437` (resolve 3s, ubuntu 3m53s, windows 12m48s, required-ci 3s); the closure-head run is recorded in the lease-closure commit
- Validated implementation revision: `f82bd4373657983b0744f4fd38e43bac0c9098c8` (`DEV-LINUX-NATIVE-01` + required CI); closure commits after it are documentation-only
- Change class: `implementation-only` (no `core/specs`, no Lane-CTR, no new first-level chrome, no Pi Member engine; additive v36 migration)
- Claim ceiling: `hypothesis`
- Running report: [2026-09-03-personal-p13-t02-hosted-dsh-attempt-report.md](2026-09-03-personal-p13-t02-hosted-dsh-attempt-report.md) (incremental log + acceptance mapping)

## Delivered

- **Store v36** `personal/crates/cognitive-store/src/hosted_dsh_attempt.rs` — `p13_hosted_dsh_artifact_fact` (append-only; kind `health-check` / `update` / `rollback` derived by the daemon; health `pinned` / `absent` / `corrupt` / `mismatch` / `script-missing`; only `pinned` admits a spawn), `p13_hosted_dsh_attempt` (the persist-before-dispatch Intent: `persisted` → `dispatched` → daemon `terminal` with `exited` / `signaled` / `timed-out` / `spawn-failed`, or `unknown-outcome` after a daemon crash; CHECK `completion_claimed = 0`, `verification_status = 'not-run'`, `context_bytes ≤ 65536`; never deleted), `p13_hosted_dsh_attempt_frame` (append-only observations, `authority_written = 0`). `hosted_dsh.rs` gains `observe_spawn` for the v31 child identity.
- **Runtime broker** `personal/crates/cognitive-runtime/src/hosted_dsh_broker.rs` — fail-closed `validate_launch_plan` (exact artifact digest, secret-shaped env/argv, Provider hosts / base-URL / `--api-key-file` / Path A, native MCP / base tool / HMR / home patch, timeout ≤ 30 min), `run_hosted_child` (`env_clear` + allowlist, own process group on Unix, one stdin `request` frame with bounded Context + digest + loopback daemon origin + bootstrap *path* + `timeout_ms`, NDJSON `observation` / `candidate` / `heartbeat` / `response` frames, 256 KiB stdout cap with drain, 512-frame cap, 1024-char redacted frame text, 2 KiB redacted stderr tail, group kill on timeout), `HostedDshArtifact::observe / resolve / launch_plan` from the daemon-owned `dsh.json` + pin file + child-script digest. There is deliberately no `Success` terminal kind and `completion_claimed()` is always false.
- **Kernel-server routes** `personal/apps/kernel-server/src/personal/hosted_dsh_attempt.rs` — `POST /management/project/v1/dsh.hosted.attempt.run` (artifact health fact → 422 `HOSTED_ARTIFACT_UNHEALTHY` if not pinned → Attempt Intent persisted → v31 child identity bound → real spawn on a daemon thread → frames + daemon terminal written; pre-spawn refusals become a durable `spawn-failed` terminal + 422 `HOSTED_ATTEMPT_SPAWN_REFUSED`), `GET …/dsh.hosted.attempt.list?project_id` / `…/dsh.hosted.attempt.detail?attempt_id` (the `runs` reads; redaction guard), `POST …/dsh.hosted.artifact.check`, `GET …/dsh.hosted.artifact.facts`; task-channel aliases 403 `HOSTED_ATTEMPT_CHANNEL_FORBIDDEN`; `serve_personal_loopback` reconciles crash-shaped rows to `unknown-outcome` before listening.
- **Product child** `personal/packages/dsh-akp-adapter/scripts/hosted-attempt-child.mjs` (+ 12 Node tests) — reads one request frame, refuses Path A / API-key / MCP flags (exit 2), pin mismatch (3), missing dsh CLI (4), non-loopback / absent bootstrap (5); mints a management session from the bootstrap *path*, writes a 0600 credentials file in a private disposable `DSH_HOME`, patches the pinned dsh (compiled-lib) `baseURL` to the daemon `/provider/v1/dsh` proxy only, streams observations / heartbeats, emits at most one `DeliverableDraft` candidate, `response done|failed` with `completion_claimed:false`, and removes its work dir.
- Handbook (en + zh-CN): generated `http-api` (10 route annotations), `store-and-migrations` (v36), `daemon-and-http`, `agent-and-pi-lifecycle`, `capability-status`, fingerprints; `dsh-akp-adapter/README.md`; plan / PROGRESS / plan.md / trace synchronised.

## Evidence (all at `f82bd437`)

| Unit | Result | Environment |
|---|---|---|
| `cargo test -p cognitive-store --test p13_t02_hosted_dsh_attempt` + `p1_t01_layout_migrations` | **pass** 9/9 + 8/8 | `DEV-LINUX-NATIVE-01` |
| `cargo test -p cognitive-runtime --test p13_t02_hosted_dsh_broker` (real `node` children) | **pass** 7/7 | `DEV-LINUX-NATIVE-01` |
| `cargo test -p kernel-server hosted_dsh_attempt` (HTTP handler → real child) | **pass** 2/2 | `DEV-LINUX-NATIVE-01` |
| `node --test scripts/hosted-attempt-child.test.mjs` | **pass** 12/12 | `DEV-WIN-GNU-01` (Node 22) |
| Live daemon E2E: seated researcher → `dsh.hosted.attempt.run` → product child → pinned dsh `528c682e` compiled-lib → daemon proxy (Provider unbound) → `exited/6`, `response failed`, `completion_claimed=false`, 8 frames `authority_written=false`, 0 secret leaks; `attempt.list` / `.detail` / `artifact.check` / `.facts`; task alias 403; no bearer 401; secret context 422 | **pass** (loop) / **partial** (Provider `done` leg not exercised) | `DEV-LINUX-NATIVE-01` (disposable runtime root, cleaned) |
| Live daemon crash: `attempt.run wait:false` → daemon `SIGKILL` → restart → `unknown-outcome` | **pass** | `DEV-LINUX-NATIVE-01` |
| Required CI (workspace build / test / clippy, Ubuntu + Windows MSVC) | **pass** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` |
| Local `cargo fmt --check`, `check:consistency`, `check:handbook`, generator `--check`, `pnpm -r build/test`, `git diff --check` | **pass** | `DEV-WIN-GNU-01` |

Two CI/Linux-caught test defects (inline `node -e` scripts carrying `api.deepseek.com` / `Bearer ` / `Authorization` literals in argv) were refusals by the product's own argv secret-shape check — the check worked; the tests now assemble those strings at runtime.

## Non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, or Agent-benefit. Linux real spawn closes "implementation exists" only (hard gate (6)); Windows sandbox / ACL / supply-chain E2E cells stay **not-run** until `P13-T13/D02` backfills them on `DEV-WINDOWS-NATIVE-OPC-01`. `DEV-WIN-GNU-01` cargo is `not-run` (`RUST-LINK-DEV-WIN-GNU-01`; fenced route `HOSTED_DSH_WIN_GNU_FENCE`). The live Provider `done` leg was not exercised because no new SecretStore entry was created (owner boundary). Not `P13-T04` CAS / verifier / outputs, not `P13-T05` Routine arming or `runs` chrome, not `P13-T08` Settings diagnostics chrome (engine facts only become readable). Not Installed Agent chrome, not native DSH UI in `/ui/`, not Pi as Member engine. Evaluation routing OFF.

## Observed, not decided (for the owner / plan)

- **Orphan window after a daemon crash.** The child runs in its own process group so a *timeout* kills child + dsh grandchild together, but a daemon *crash* (SIGKILL) leaves the child alive until its own budget (`timeout_ms − 3 s`) or its daemon-unreachable failure. Cross-crash reaping belongs to the supervisor seam (P2-T06), not to this task; recorded, not claimed as containment.
- **No product path applies a PlanRevision.** `roster.register` requires `plan_revision_id`, but no HTTP route or CLI verb calls `apply_plan_revision`; the live E2E seeded it as a fixture before starting the daemon. `P13-T05` / `P13-T06` (or `P13-T03` propose) must close this for a seated Member to exist on a real daemon without a fixture.
- **Provider `done` leg.** With a Provider bound to `agent://personal/dsh` the same loop would carry a real dsh completion through the daemon proxy; exercising it needs an owner-designated key imported into the Linux Secret Service backend (Operating Model §2.3) — not done here.

## Unique next

Close the lease on `main` (this delivery), then claim `P13-T04/D01` (Attempt artifacts → CAS → independent verifier) and/or `P13-T05/D01` (runs / Routine arming) — both unblocked by `P13-T02/D02` — and `P13-T03/D01` if still unclaimed; `P13-T06/D01` after `P13-T03/D01`. Do not claim `P11-T15`.
