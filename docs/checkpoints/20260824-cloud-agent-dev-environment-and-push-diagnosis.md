# Cloud Agent development environment and push/merge diagnosis

- Date: 2026-08-24
- Baseline: `main@463977648c5d0ab2ebb9801712139b8de08e9a4f`
- Branch: `cursor/cognitive-os-dev-env-setup-18be`
- Change class: `implementation-only` (developer environment, environments
  registry, handbook routing) plus an append-only operational record
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit claim. No
  formal `P*-T*` task is claimed, started, or closed. Container measurements
  below are wall-clock observations on shared cloud hardware, not performance
  baselines. The `agentkernel/cognitiveos-clients` publication remains
  **blocked**; nothing here makes it succeed.

## 1. Why this record exists

Two problems were reported together: pushes/merges "not working", and the
Cloud Agent development environment for `agentkernel/cognitive-os` not being
set up correctly. They are separate faults with separate owners, and one of
them had been described only from the `cognitiveos-clients` side. This record
separates them with directly executed probes.

## 2. Push and merge capability — measured, not assumed

All probes ran from the Cursor Cloud Agent run
`bc-0c6f2f39-7484-5007-ad1f-f96a2cfd18be` on environment
`9a1980df-9f6c-11f1-a7d1-d6b4613131ce`.

| Probe | Result |
|---|---|
| `git fetch origin main` | **pass** — `origin/main` resolves to `46397764`, identical to the recorded merge point of PR [#266](https://github.com/agentkernel/cognitive-os/pull/266) |
| Push a transient branch `cursor/probe-push-diagnostic-18be` to the kernel repository | **pass** — new branch created |
| Delete that remote branch | **pass** — probe left no residue |
| Push the real work branch `cursor/cognitive-os-dev-env-setup-18be` | **pass** |
| `GET /installation/repositories` with this run's token | `total_count: 1` → **only** `agentkernel/cognitive-os` |
| `git ls-remote` on `agentkernel/cognitiveos-clients` | **pass** (repository is public, so read needs no grant) |
| `git push --dry-run` of a new branch to `agentkernel/cognitiveos-clients` | **fail** — `Permission to agentkernel/cognitiveos-clients.git denied to cursor[bot]`, HTTP 403 |
| PR states #264 / #265 / #266 / #267 | `OPEN` draft (dirty) / `MERGED` / `MERGED` / `MERGED` — matches the recorded campaign facts |
| `GET /repos/agentkernel/cognitive-os/rulesets` | `[]` — no repository ruleset blocks the branch |
| `GET /repos/agentkernel/cognitive-os/branches/main/protection` | `403 Resource not accessible by integration` — the App token lacks `administration:read`; this is a **visibility** limit on reading the setting, not a push failure. Merged PRs #265–#267 already demonstrate the protected-branch path works end to end |

**Conclusion.** There is no kernel-repository push or merge fault. Fetch,
branch push, branch delete, and the Draft → ready → merge path are all healthy
for `agentkernel/cognitive-os`. The only failure is repository-scoped: the
Cursor GitHub App installation token minted for this run covers exactly one
repository, so any write to `agentkernel/cognitiveos-clients` fails closed with
HTTP 403 no matter what that repository's visibility is. This reproduces and
confirms — from an independent run and environment — the root cause already
recorded in
[the clients write-access remediation](20260824-personal-p7-t05-clients-write-access-remediation.md)
§1 and §5.

Additional fact for the owner: this environment's repository list
(`environment-info` → `repos`) is `["github.com/agentkernel/cognitive-os"]`.
The installation grant in that remediation §2 is necessary but **not
sufficient** on its own — a run only receives a token for the repositories in
its own environment. Both of the following must hold before any Cloud Agent
can push to the clients repository:

1. `agentkernel/cognitiveos-clients` is added to the Cursor GitHub App
   installation (owner-only, remediation §2); and
2. the publishing run is started from an environment that lists that
   repository — either an agent launched on `agentkernel/cognitiveos-clients`,
   or this environment extended with it as a second repo.

The alternative that needs neither is for the owner to import the verified
recovery bundle with an identity that already has write access.

The recovery bundle SHA-256
`02a0216fd4611b88d904a1c481dc96b1f9ec06f62b99979511c45258250b641e` is **not
present** in this run's agent store (`/cursor/stores/self/artifacts/` is
empty); agent stores are per-run. Publication from this run is therefore
impossible on two independent counts, and no client push, client branch, or
client Draft PR is claimed here.

## 3. Development environment — what was actually wrong

`environment-info` for this run reported an environment whose saved
configuration contained **no recognized fields**, and
`build.resolution: no_healthy_builds`. The three most recent SYSTEM/RECURRING
builds (`bld-20260824-22bf90eb…`, `bld-20260824-e34c43a8…`,
`bld-20260824-2d87dabb…`) all reached `FAILED` / `TERMINAL_FAILURE` within
~15 ms of creation with `environmentVersionId: null` and no captured logs —
they never booted a pod.

The practical consequence in a fresh pod:

- no `node_modules` anywhere in the workspace, so every `pnpm`-based checker
  and the whole TypeScript build/test path failed until installed by hand;
- the pinned Rust toolchain was not materialized, so the first `cargo`
  invocation spent ~11 s downloading it;
- the mandatory docs-sync hooks were not registered, so the
  `docs/standards/docs-sync-contract.md` §2 pre-commit gate did not run.

### Fix

[`.cursor/environment.json`](../../.cursor/environment.json) now exists and
runs [`scripts/setup-dev-env.sh`](../../scripts/setup-dev-env.sh) as the
`install` step. A repository `.cursor/environment.json` takes precedence over
a personal or team saved environment, so this is durable in the repository and
needs no dashboard edit. The script is idempotent and does three things:
install the pnpm workspace from the frozen lockfile, materialize the pinned
Rust toolchain and pre-fetch crates, and register the docs-sync hooks.

Hook registration deserves a note. `pnpm run hooks:install` sets
`core.hooksPath` to `.githooks`, which in a Cloud Agent pod would **replace**
the agent-managed hooks directory (the one that carries the secret scanner).
The script therefore composes instead of replacing: when a foreign
`core.hooksPath` is configured it writes forwarders into `<git-dir>/hooks` —
the path such dispatchers chain to — and never touches the managed directory.
When no `core.hooksPath` is set, it falls back to the documented
`git config core.hooksPath .githooks`.

Known residual limitation: under an agent-managed hooks directory only
`pre-commit` is chained, so the repository's `pre-push` docs-sync gate does not
fire automatically. Run `node tools/src/docs-sync-gate.mjs --push` explicitly
before pushing from such a pod. The CI handbook step remains the unconditional
merge gate either way.

### Verification of the fix

A draft environment build was triggered against this branch:
[`bld-20260824-06aed49d-2035-4d37-b3e4-394ab21ae759`](https://cursor.com/dashboard/cloud-agents/builds/bld-20260824-06aed49d-2035-4d37-b3e4-394ab21ae759),
status **SUCCEEDED** in ~71 s (previous recurring builds of this environment
failed terminally in ~15 ms). Its install log shows the whole script running in
a clean pod: `installing pnpm workspace dependencies` → `Done in 824ms`,
`resolving pinned Rust toolchain` → `cargo 1.97.1`, crate download, then
`registering .githooks as core.hooksPath` → `done`. That pod had no managed
hooks directory, so it took the fallback branch; the forwarder branch was
exercised in this run's pod. Both paths are covered.

The build is a draft created from a non-default ref, so it is intentionally
not promotable; it exists as evidence that the configuration installs cleanly.
Promotable builds come from the default branch after this change merges.

## 4. Environment capability recorded on this baseline

`CLOUD-AGENT-LINUX-01` is now registered in
[`PERSONAL-TEST-ENVIRONMENTS.md`](../plan/PERSONAL-TEST-ENVIRONMENTS.md) §15
with its claim ceiling. Executed on `main@46397764` in this pod:

| Check | Result |
|---|---|
| `pnpm install --frozen-lockfile` | **pass** |
| `pnpm -r build` | **pass** (4 projects) |
| `pnpm -r test` | **pass** before a Rust debug build exists; see §4.1 |
| `pnpm run check:consistency` | **pass** |
| `pnpm run check:handbook` | **pass** |
| `node tools/src/generate-handbook.mjs --check` | **pass** (18 pages byte-identical) |
| `cargo fmt --all -- --check` | **pass** |
| `cargo build --workspace` | **pass**, 48.9 s |
| `cargo clippy --workspace --all-targets` | **pass**, 19.6 s |
| `cargo test --workspace` | **pass**, 1210 passed / 0 failed |
| `cargo run -p cognitive-conformance --bin conformance-runner` | **pass** (completes; writes to ignored `artifacts/`) |

### 4.1 Pre-existing condition surfaced by running both halves in one place

`packages/sdk-ts/src/http_live.test.ts` activates only when
`target/debug/kernel-server` exists (`skip: !LIVE`). On a host that runs both
the TypeScript and the Rust halves, the second `pnpm -r test` after
`cargo build --workspace` therefore behaves differently from the first:

- before any Rust build: sdk-ts 74 tests, 74 pass (the live tests are skipped);
- after `cargo build --workspace`: sdk-ts 73 pass / **1 fail**, reproducible
  3/3 — `live: task watch stream yields snapshot then delta frames` collects
  zero frames, so `assert.ok(frames.some(f => f.includes('"kind":"snapshot"')))`
  fails. Its sibling `live: shell.detach` against the same `--once` server
  passes, so the spawned binary does serve requests and the failure is specific
  to `watch.open` on the task channel.

This is **not** caused by anything in this change — no file under `packages/`,
`crates/`, or `apps/` was touched. CI never observes it because
[`ci.yml`](../../.github/workflows/ci.yml) runs `pnpm -r test` before the Rust
steps, so `target/debug/kernel-server` does not yet exist at that point.

Deliberately not fixed here: repairing it means changing product or test
semantics in `packages/sdk-ts` or the daemon watch path, which belongs to a
formal task with its own acceptance, not to environment setup. Recorded so the
next developer on a full-capability host is not surprised, and so the choice
between fixing the live test and making CI exercise it is made explicitly.

This matters beyond convenience. `RUST-LINK-DEV-WIN-GNU-01` records that the
owner's local Windows GNU host cannot link Rust at all, which is why Rust
validation has had to wait for CI or for `DEV-LINUX-NATIVE-01`. A Cloud Agent
pod is a native GNU/Linux link host and runs the full Rust workspace in about
a minute, so it is a usable pre-CI iteration surface. `COMMAND-SHELL-PS51`
also does not apply there — the shell is bash.

The claim ceiling is deliberately conservative and unchanged in substance:
container-class implementation evidence, equivalent in standing to
`DEV-WSL2-01`. It is not native user-systemd, not Secret Service, not
Pi-qualified, carries no timing baseline, and never substitutes for required
CI or for exact-revision native Linux evidence.

## 5. Scope boundaries observed

- No formal `P*-T*` task was claimed, started, or closed, and no active lease
  was opened. The active lease table stays empty: the consistency checker
  requires every active non-evaluation lease to name a formal `P<n>-T<n>/D<nn>`
  slice, and inventing one for infrastructure work would be a false record.
- Evaluation routing is OFF (`PERSONAL-PERF-EVAL-015` closed), so no
  owner-directed campaign row was touched.
- PR [#264](https://github.com/agentkernel/cognitive-os/pull/264) and its
  `D10` slice-ID collision were left exactly as recorded; the collision is not
  in this scope.
- Nothing in `specs/`, `conformance/`, `crates/`, `apps/`, or `packages/` was
  modified. The transient push probe branch was deleted from the remote.
- One recoverable mistake was made and repaired inside this session: an early
  version of the setup script wrote a hook forwarder through a symlink and
  overwrote the agent-managed hook dispatcher. It was restored byte-identically
  (verified against a sibling symlink) and the script was changed to target
  `<git-dir>/hooks` explicitly, so the managed directory is never written.

## 6. Unique next actions

- **Owner (clients publication, unchanged):** complete remediation §2, then
  run the publication from a run whose environment includes
  `agentkernel/cognitiveos-clients` — or import a verified bundle with a
  write-capable identity. Rebuilding the D10 client work is required again in
  that run, because the bundle does not survive across agent stores.
- **Owner (optional, kernel environment):** if Cloud Agents should also be
  able to work on the clients repository from this environment, add it to
  `environment.repos` on the environment dashboard
  ([`9a1980df-9f6c-11f1-a7d1-d6b4613131ce`](https://cursor.com/dashboard/cloud-agents/environments/e/9a1980df-9f6c-11f1-a7d1-d6b4613131ce)).
  That is a dashboard action; it cannot be done from inside a pod.
- **Repository:** merge this branch so new pods bootstrap automatically.
- **Follow-up candidate (not claimed as a task):** decide the disposition of
  the §4.1 `sdk-ts` live watch failure — either fix the `watch.open` task
  channel path it exercises, or make CI run `pnpm -r test` after the Rust build
  so the condition stops being invisible. Either choice is a formal task.

## 7. Required CI on this branch (appended after the run finished)

Required CI run
[`32698197416`](https://github.com/agentkernel/cognitive-os/actions/runs/32698197416)
at `62b2de7c255286b18f328795efecea7d419216f9`:

| Job | Result |
|---|---|
| `resolve validation route` | **pass** |
| `verify (ubuntu-latest)` | **pass** |
| `verify (windows-latest)` | **fail** |
| `required-ci` | **fail** (gated on the Windows job) |

The Windows job failed one assertion in
`apps/kernel-server/src/personal/p2_t17_a7_failure_first.rs:331`
(`post_dispatch_fault_points_reconcile_without_redispatch_or_task_acceptance`
observed `Indeterminate` where `ReconciledExecuted` is asserted); 349 of 350
binary tests passed and Ubuntu ran the identical suite green.

Attribution, stated as evidence rather than convenience: this branch changes no
file under `apps/`, `crates/`, `packages/`, `specs/`, `conformance/`, or
`tests/`, and its merge base is exactly `main@46397764`, whose own required run
[`32696407284`](https://github.com/agentkernel/cognitive-os/actions/runs/32696407284)
passed Ubuntu **and** Windows. The same test file failed once before on an
unrelated branch — run
[`32691510523`](https://github.com/agentkernel/cognitive-os/actions/runs/32691510523)
at `830ca0b2`, Windows only, line 96 — and the next run on that branch passed
without any change to that code. The signature is an intermittent
Windows-only failure in this restart-reconciliation suite, not a regression
introduced here. Diagnosing it belongs to a formal task with its own
acceptance; this record does not claim a root cause.

`gh run rerun` is not available to an installation token
(`Resource not accessible by integration`), so re-running the job required
pushing this appended record to retrigger the workflow.

## 8. Independent re-verification from a second Cloud Agent run

Run `bc-8d704ebc-13be-5dc9-b13f-8b6c082216c9`, same environment
`9a1980df-9f6c-11f1-a7d1-d6b4613131ce`, re-executed the load-bearing probes
rather than trusting §2–§4:

| Probe | Result |
|---|---|
| `GET /installation/repositories` | `total_count: 1` → only `agentkernel/cognitive-os`; the clients grant is still missing |
| `git ls-remote` on `agentkernel/cognitiveos-clients` | **pass** — `main` at `db563744`, plus `personal/P7-T05-dsh-binding-cas` and `personal/P7-T05-web-ui-sidebar-fix` |
| `git push --dry-run` to `agentkernel/cognitiveos-clients` | **fail** — `Permission to agentkernel/cognitiveos-clients.git denied to cursor[bot]`, HTTP 403 |
| `environment.repos` for this run | `["github.com/agentkernel/cognitive-os"]` — confirms a run token cannot reach a repository its environment does not list |
| Environment builds | draft `bld-20260824-06aed49d…` **SUCCEEDED** (~71 s); the three preceding recurring system builds are `TERMINAL_FAILURE` — the fault this change repairs |
| `bash scripts/setup-dev-env.sh` in a pod whose `core.hooksPath` is agent-managed | **pass** — took the forwarder branch, wrote `pre-commit`/`pre-push` into `<git-dir>/hooks`, and left the managed directory untouched |
| `check-consistency` / `check-handbook` / `generate-handbook --check` / `docs-sync-gate --staged` | **pass** (275 requirements; 55 docs × 2 locales; 18 generated pages byte-identical; no changes in scope) |

This run claimed no formal task and opened no lease; the active lease table is
still empty.
