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

## Remaining

WorkspaceRead extension registration and adapter extraction are implemented.
Retry the normal exact-revision push for `6e7e4197`, then run the public
production-path validation on `DEV-LINUX-NATIVE-01`. No B01 guest, Provider
sample, paired runner, or evaluation campaign is used by this task.
