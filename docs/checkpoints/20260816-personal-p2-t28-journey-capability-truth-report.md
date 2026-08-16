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
