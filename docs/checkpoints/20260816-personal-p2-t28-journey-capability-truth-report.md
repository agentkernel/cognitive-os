# P2-T28 End-to-end journey and capability truth — running validation report

- Task: `P2-T28` (BR-08)
- Branch: `personal/P2-T28-journey-capability-truth`
- Lease: `lease/personal/P2-T28/journey-capability-truth`
- Change class: `implementation-only`
- Claim ceiling: hypothesis/non-claim. No Gate, release, Profile, B01, EVAL, or
  Agent-benefit promotion.

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt --check`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy（`RUST-LINK-DEV-WIN-GNU-01`）。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu supporting CI
  (`verify (ubuntu-latest)` workspace test + Clippy `-D warnings` + handbook)。
  Windows 是 `not-run by owner-directed Linux-only route`。
- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2` / `hal9000`）是 D03 所需的 exact
  pushed-revision 验收环境。
- `B01-Desktop-Linux-002` 属于 owner-directed EVAL-004-only guest；本任务不使用。

## D01 — freeze UJ1..UJ6 capability-truth matrix

### D01-IMPL-01 — frozen matrix and daemon register

- Instrument: `tools/fixtures/p2_t28_uj_matrix.json`,
  `tools/src/p2_t28_uj_matrix.mjs`,
  `apps/kernel-server/src/personal/capability_truth.rs`.
- Outcome: authored. Twelve required UJ1–UJ6 rows name an existing public
  caller file, mechanical oracle file, cleanup, and evidence schema. Web UI
  and Multi-Agent stay explicit `excluded` / `ScopeExcluded` and cannot be
  marked required. Claim ceiling `hypothesis`. This freeze does not execute
  journeys or claim EVAL-004/Gate/release/Profile results.

### D01-TEST-01 — missing caller / oracle / path / cleanup / exclusion

- Instrument: `tools/test/p2_t28_capability_truth.test.mjs` and
  `capability_truth` unit tests.
- Oracle: empty caller/oracle/cleanup fail closed; a named caller path that
  does not exist fails closed; Web UI cannot be required; Multi-Agent must
  remain an explicit excluded row; duplicate ids and missing UJ families fail
  closed.
- Windows GNU: Node tests are the local route. Rust unit tests
  `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).

### D01-LOCAL-01 — Node matrix validator

- Instrument: `node --test tools/test/p2_t28_capability_truth.test.mjs`.
- Outcome: **pass** 6/6 on `DEV-WIN-GNU-01` (frozen register, missing caller,
  missing cleanup, missing oracle, missing caller path, Web UI/Multi-Agent
  exclusion).

### D01-LOCAL-02 — Windows GNU Rust tests

- Instrument: local `cargo test -p kernel-server capability_truth`.
- Outcome: `not-run by owner-directed Linux-only route` /
  `RUST-LINK-DEV-WIN-GNU-01`.

### D01-LOCAL-03 — cargo fmt (Windows GNU allowlist)

- Instrument: `cargo fmt --all -- --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: **pass**
- Note: Rust build/test/Clippy are `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`).

### D01-LOCAL-04 — check:consistency

- Instrument: `pnpm run check:consistency`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: **pass** (275 requirements, 55 error codes, 74 schemas, 89 vectors,
  Personal plan/Gates, leases verified)

### D01-LOCAL-05 — handbook + generator check

- Instrument: bilingual conformance-and-testing / validation-commands /
  capability-status / architecture-overview updates +
  `node tools/src/fill-handbook-fingerprints.mjs` +
  `node tools/src/check-handbook.mjs` +
  `node tools/src/generate-handbook.mjs --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: **pass** (54 documents × 2 locales; 18 generated pages
  byte-identical)

### D01-CI-01 — Ubuntu supporting CI

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31941480101`](https://github.com/agentkernel/cognitive-os/actions/runs/31941480101)
  on Draft PR [#227](https://github.com/agentkernel/cognitive-os/pull/227) at
  `aafe45a5`.
- Outcome: **fail**. Clippy `-D warnings` rejected `expect` / `unwrap_err` in
  `capability_truth` unit tests (`clippy::expect_used`, `clippy::unwrap_used`).
  Fix follows on the same branch; Node freeze 6/6 is unchanged.

### D01-CI-02 — Ubuntu supporting CI after Clippy fix

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31941999146`](https://github.com/agentkernel/cognitive-os/actions/runs/31941999146)
  on Draft PR [#227](https://github.com/agentkernel/cognitive-os/pull/227) at
  `c7d4c7f7`.
- Outcome: **pass** (`verify (ubuntu-latest)` + `required-ci` green). Windows
  `not-run by owner-directed Linux-only route`.

### D01-LINUX-01 — capability-truth units at `c7d4c7f7`

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`, Rust 1.97.1)
  at `c7d4c7f7`.
- Outcome: **pass**. `capability_truth` **7/7**; `cargo fmt --all -- --check`
  **pass**; `cargo clippy --workspace --all-targets --locked -- -D warnings`
  **pass**.

## D02 — required public journeys

### D02-IMPL-01 — hermetic public-caller smoke

- Instrument: `apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`.
- Outcome: authored. One hermetic daemon exercises UJ1/UJ3 status+doctor,
  UJ3 observation named zero, UJ4 missing-evidence fail closed, UJ5 Effect
  history unknown-task/restatement, UJ6 consumption restatement denial, and
  UJ6 backup `sqlite_copied=false` with secret exclusion. Runtime root is
  removed. Nested Pi timing and managed Pi install→recover stay on their
  named D01 oracles for the linux-002 aggregate. Web UI/Multi-Agent are not
  called.

### D02-LOCAL-01 — Windows GNU Rust tests

- Instrument: local `cargo test -p kernel-server --test p2_t28_end_to_end_journey`.
- Outcome: `not-run by owner-directed Linux-only route` /
  `RUST-LINK-DEV-WIN-GNU-01`.

### D02-LINUX-01 — public-caller smoke at `c7d4c7f7`

- Instrument: `cargo test -p kernel-server --test p2_t28_end_to_end_journey`
  on `DEV-LINUX-NATIVE-01` at `c7d4c7f7`.
- Outcome: **pass** 1/1. Residue `/tmp/cos-p2t28-*` count **0**. Web UI and
  Multi-Agent were not called.

## D03 — exact-revision linux-002 aggregate

### D03-LINUX-01 — named UJ oracles and workspace (`c7d4c7f7`)

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`, Rust 1.97.1)
  at `c7d4c7f7`.
- Outcome: **pass**. Named oracles: `p1_t05_personal_readiness` **1/1**,
  `p9_t07_route_observation` **2/2**, `p2_t26_observation_plane` **1/1**,
  `p2_t24_effect_fault` **1/1**, `p4_t05_resource_api` **5/5**,
  `p2_t16_registered_check` **3/3**, `p2_t28_end_to_end_journey` **1/1**,
  `p2_t17_a7_failure_first` **15/15**, `admin-cli` `p2_t27_backup_restore`
  **2/2**, `admin-cli` `p2_t27_pi_lifecycle` **1/1**. kernel-server `--bins`
  **336/336**; workspace tests **0 failed**; `cargo fmt --all -- --check`
  **pass**; `cargo clippy --workspace --all-targets --locked -- -D warnings`
  **pass**. Residue `/tmp/cos-p2t28-*` count **0**. Windows `not-run by
  owner-directed Linux-only route`. `B01-Desktop-Linux-002` untouched
  (EVAL-004-only). Claim ceiling `hypothesis`.

### D03-CI-01 — Ubuntu supporting CI at implementation revision

- Instrument: GitHub Actions run
  [`31941999146`](https://github.com/agentkernel/cognitive-os/actions/runs/31941999146)
  at `c7d4c7f7`.
- Outcome: **pass**. Docs-only follow-up CI is recorded when that head finishes.

### D03-ACCEPT-01 — formal acceptance mapping

| Acceptance | Evidence |
|---|---|
| UJ1 install/init/first response public caller + oracle | D01 freeze `UJ1-install-init-first-response`; D02 status/doctor smoke; D03 `p1_t05` 1/1 |
| UJ2 cold/warm nested timing | D01 freeze `UJ2-cold-warm-nested-timing`; D03 `p9_t07_route_observation` 2/2 |
| UJ3 status/doctor/observation/restart replay | D01 freeze; D02 named-zero observation; D03 `p2_t26` 1/1 |
| UJ4 Task admission/execution/terminal query | D01 freeze; D02 missing-evidence 400; bins include scheduler_authority |
| UJ5 fault/restart/deadline/cleanup | D01 freeze; D02 Effect-history 404/400; D03 `p2_t24` 1/1 and A7 15/15 |
| UJ6 Memory/Skill | D01 freeze; D02 consumption restatement denial; D03 `p4_t05` 5/5 |
| UJ6 read/search/write/patch/check | D01 freeze; D03 `p2_t16` 3/3 plus bins production sinks |
| UJ6 Pi lifecycle | D01 freeze; D03 `p2_t27_pi_lifecycle` 1/1 |
| UJ6 backup/restore | D01 freeze; D02 `sqlite_copied=false`; D03 CLI backup 2/2 |
| UJ6 verified completion | D01 freeze; scheduler_authority in bins 336/336 |
| Web UI / Multi-Agent | explicit `excluded`; not called; do not block BR-08 |
| Exact-revision linux-002 | this D03 matrix at `c7d4c7f7` |
| Ubuntu supporting CI | run `31941999146` pass at `c7d4c7f7` |
| Windows | `not-run by owner-directed Linux-only route` |

Layer 2 `P2-T28/D03` stays `in-progress` until ready/merge so the active lease
slice cannot mismatch `CURRENT_SNAPSHOT_LEASE_MISMATCH`.

### D03-LOCAL-01 — Windows GNU allowlist after D03 docs

- Instrument: `cargo fmt --all -- --check`, `pnpm run check:consistency`,
  `node tools/src/check-handbook.mjs`, `node tools/src/generate-handbook.mjs --check`,
  `git diff --check`.
- Environment: local `DEV-WIN-GNU-01`
- Outcome: **pass** (fmt; consistency 275/55/74/89; handbook 54×2 locales;
  18 generated pages byte-identical; diff check clean).
- Note: Rust build/test/Clippy remain `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`).

### D03-CI-02 — Ubuntu supporting CI on docs head

- Instrument: GitHub Actions run
  [`31943289974`](https://github.com/agentkernel/cognitive-os/actions/runs/31943289974)
  at `d45a64be`.
- Outcome: **pass** (`verify (ubuntu-latest)` + `required-ci` green).

### D03-MERGE-01 — PR #227 merged; lease closed

- Instrument: GitHub PR [#227](https://github.com/agentkernel/cognitive-os/pull/227)
  merge to `main`.
- Outcome: **merged** at `main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd`.
  Task lease `lease/personal/P2-T28/journey-capability-truth` closed 2026-08-16.
  Remote task branch deleted. Claim ceiling `hypothesis`; no
  Gate/release/Profile/B01/EVAL promotion. EVAL-004 re-freeze is a separate
  measurement campaign under `lease/personal/EVAL-20260816/full-os-only-refreeze`.

