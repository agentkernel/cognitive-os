# P2-T36 running validation report

- Task: `P2-T36` - C1 WorkspaceRead/Search public production path
- Branch: `personal/P2-T36-c1-public-production-path`
- Lease: `lease/personal/P2-T36/c1-public-production-path`
- Classification: product-semantic + implementation; no normative contract change
- Claim ceiling: hypothesis/non-claim; no EVAL, Gate, release, Profile, B01,
  or Agent-benefit promotion

## Running results

Results are appended immediately after each completed validation unit.

| Unit | Environment | Result | Evidence |
|---|---|---|---|
| D01 failure-first Pi extension registration | `DEV-WIN-GNU-01` Node/TypeScript allowlist | fail (expected) | After adding the C1 expectation, `pnpm --filter @cognitiveos/pi-cognitiveos build` passed, then `node --test --test-name-pattern "registration queues" "packages/pi-cognitiveos/dist/extension.test.js"` failed: the actual registered tools are `WorkspacePatch`, `WorkspaceSearch`, and `WorkspaceWrite`; `WorkspaceRead` is absent. This proves the C1 Read Pi-visible tool gap before implementation. No Rust build/test was run on Windows GNU. |
| D01 Pi extension regression suite | `DEV-WIN-GNU-01` Node/TypeScript allowlist | pass | After adding the I/O-free daemon-governed WorkspaceRead tool and preserving the empty Pi-native allowlist, `pnpm --filter @cognitiveos/pi-cognitiveos build` and `pnpm --filter @cognitiveos/pi-cognitiveos test` passed. This is implementation evidence only; it does not prove a Rust scheduler, verifier, acceptance, real Pi, Provider, C1, B01, or paired benchmark path. |
| D01 Rust formatting | `DEV-WIN-GNU-01` Rust formatting allowlist | pass | `cargo fmt --all` completed after the adapter protocol update. Rust compilation and tests remain routed to exact pushed `DEV-LINUX-NATIVE-01`. |
| D01 plan and handbook checks | `DEV-WIN-GNU-01` static/documentation allowlist | pass | After registering P2-T36/D01-D03 and synchronizing both handbook locales, `pnpm run check:consistency`, `node tools/src/check-handbook.mjs`, and `node tools/src/generate-handbook.mjs --check` passed. These checks do not execute the C1 runtime path. |
| D01 staged documentation gate | `DEV-WIN-GNU-01` static/documentation allowlist | pass | `node tools/src/docs-sync-gate.mjs --staged` passed. The mapped `pi-shell` pages and fingerprints are synchronized; this gate does not execute the C1 runtime path. |
| D01 exact-revision push | local Git transport | partial | Checkpoint `6e7e4197` committed, but `git push -u origin HEAD` could not connect to GitHub through the configured loopback proxy. No supported Linux runtime validation has started because it must consume a pushed exact revision. Retry the normal push path before invoking `DEV-LINUX-NATIVE-01`; no force push or alternate source copy is permitted. |
| D01 transport recovery retry | local Git transport | partial | A second normal `git push -u origin HEAD` and a one-command no-config-override retry (`git -c http.proxy= -c https.proxy= push -u origin HEAD`) both failed with the same connection refusal via `127.0.0.1:443`. The current session has no `*PROXY` environment variables and no repository-level `http.proxy`/`https.proxy` setting. The fault remains external to task-owned code. |
| D01 exact-revision delivery recovery | GitHub HTTPS | pass | The normal `git push -u origin HEAD` succeeded after connectivity was restored. `origin/personal/P2-T36-c1-public-production-path` now contains product checkpoint `ae6fd828`; the next supported validation must check out that exact product commit from Git rather than copy local files. |
| D01 exact-worktree provisioning | `DEV-LINUX-NATIVE-01` | pass | Direct full and HTTP/1.1 clone attempts were transiently unavailable or below Git's transfer floor. A clean shallow Git clone of the pushed branch then completed at `/home/wuz/p2-t36-c1`; `git rev-parse HEAD` was `15557d18d5efa5a38e5c2948545742e07d53db81`, matching the pushed task head. The pre-existing `/home/wuz/agent-kernel` source directory has no `.git` metadata and was not used. |
| D01 adapter WorkspaceRead protocol | `DEV-LINUX-NATIVE-01` exact `15557d18` | pass | `cargo test -p pi-agent-adapter --test daemon_candidate_protocol --locked` completed **21/21**. This includes `pi_print_events_accept_one_daemon_governed_workspace_read` and the existing duplicate/mixed candidate, Pi-built-in, unknown digest, missing digest, authority-field, and oversized-context negatives. It proves adapter protocol parsing only, not public daemon admission, scheduler lease, dispatch, verification, acceptance, real Provider/Pi execution, C1, B01, or a paired benchmark. |
| D01 supporting CI | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` exact `15557d18` | partial | Ubuntu verification passed. Windows failed only in existing unrelated `personal::p2_t17_a7_failure_first::post_dispatch_fault_points_reconcile_without_redispatch_or_task_acceptance` (`Indeterminate` observed where `ReconciledExecuted` was asserted); required-ci therefore failed. The failed job was rerun before any task-owned code change. This is not evidence for the pending real Pi public path. |
| D02 real-adapter prerequisites | `DEV-LINUX-NATIVE-01` exact `15557d18` | partial | `pnpm install --frozen-lockfile`, `pnpm --filter @cognitiveos/pi-cognitiveos build`, and `cargo build -p pi-agent-adapter --locked` passed in the exact Git worktree. The final non-secret `command -v pi` probe found no Pi executable on the host PATH, so no `cognitive pi configure`, daemon start, Provider readiness, or real Pi candidate attempt began. Locate or install only the pinned Pi `0.81.1` through an approved non-secret package path before configuring the disposable product runtime. |
| D02 pinned Pi environment recovery | `DEV-LINUX-NATIVE-01` | partial | `pnpm setup` registered the user-level PNPM home in `/home/wuz/.bashrc`; no repository file or secret path changed. The first global install was rejected because that bin directory was absent from the noninteractive PATH. A corrected `PNPM_HOME`/PATH install of `@earendil-works/pi-coding-agent@0.81.1` downloaded its pinned dependency graph but made no progress after link completion for more than 60 seconds, so the stalled SSH command was stopped. No Pi executable/version confirmation, `pi configure`, daemon, Provider, or candidate execution occurred. Retry from the installed package state with a bounded noninteractive verification before any public product attempt. |
| D02 Pi runtime configuration | `DEV-LINUX-NATIVE-01` exact `15557d18` | pass | A bounded retry confirmed `/home/wuz/.local/share/pnpm/bin/pi` reports pinned version `0.81.1`. On the same exact Git worktree, `cargo build -p admin-cli --locked` passed and public `cognitive pi configure` created a cleanable non-B01 runtime's non-secret `pi.json`, referencing the pinned Pi binary, built Extension, and built candidate adapter. This configures no Provider material and does not launch Pi or the daemon. |

## Remaining

WorkspaceRead extension registration and adapter extraction are implemented,
the exact Git worktree and cleanable non-B01 Pi configuration are prepared, and
Draft PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) exists.
Continue D02 by starting the public daemon and using the configured real Pi
path only after daemon-owned readiness admits it. Record candidate validation,
scheduler lease, dispatch, verification, and acceptance separately. No B01
guest, Provider sample, paired runner, or evaluation campaign is used by this
task.
