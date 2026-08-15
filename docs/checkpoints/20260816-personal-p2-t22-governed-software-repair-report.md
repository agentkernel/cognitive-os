# P2-T22 Governed software-repair journey — running validation report

- Task: `P2-T22`
- Branch: `personal/P2-T22-governed-software-repair`
- Lease: `lease/personal/P2-T22/governed-software-repair`
- Base: `origin/main` after P2-T21 merge `5d1f5c2643d82807eb96f2c615d460dbdca749c3` / PR #220
- Change class: `implementation-only` (catalog freeze + failure-first tests; no
  public contract). Mapped `execution-chain-status` pages updated bilingually
  for the frozen TypeScript/Rust catalog and the write-alone D02 gap; fingerprints
  refreshed in the same change set.
- Claim ceiling: implementation evidence only; hypothesis/non-claim. No Gate,
  release, Profile, B01, EVAL, or Agent-benefit promotion.

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录；已发布
结果只通过追加的 superseding entry 更正。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt --check`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy（`RUST-LINK-DEV-WIN-GNU-01`
  已登记 exit 121 linker failure）。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu required CI（`verify (ubuntu-latest)`
  workspace test + Clippy + handbook）。Windows 是
  `not-run by owner-directed Linux-only route`。
- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2` / `hal9000`）：exact pushed-revision
  worktree；只做 native build/test/clippy/fmt 验证；不触碰 `B01-Desktop-Linux-002`
  guest / EVAL-004 campaign roots。
- `B01-Desktop-Linux-002` guest 属于 owner-directed evaluation campaign，与本 task
  验证无关，本任务不使用。

## D01 — freeze TypeScript/Rust corpora + failure-first tests

### D01-DOC-01 — lease, plan, and BR-02 registration

- Instrument: `docs/plan/PARALLEL-LANES.md` active table,
  `docs/plan/PROGRESS.md` Current snapshot,
  `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`,
  `docs/evaluation/personal-performance-benchmark-readiness-closure-plan.md`
- Outcome: `pass` (authored). Lease
  `lease/personal/P2-T22/governed-software-repair` claimed with
  `P2-T22/D01`. P2-T21 flipped to `done` at merge
  `5d1f5c2643d82807eb96f2c615d460dbdca749c3` / PR #220. Layer 1
  `88 | 74 | 1 | 1 | 12 | 14`. BR-01 `done`, BR-02 `in-progress`.
- Disposition: opens D01; does not execute Rust tests.

### D01-IMPL-01 — catalog freeze

- Instrument: `apps/kernel-server/src/personal/registered_check/mod.rs`
  plus on-disk fixtures under `tests/fixtures/p2_t16_registered_check/`
- Outcome: authored. `c2a.repair.typescript` descriptor_version 2 pins
  hidden `tests/hidden.repair.test.ts` (`add(4,1)!==5`). New
  `c2a.repair.rust` pins repaired `src/repair.rs` plus public/hidden
  tests. Broken starting sources are not in `expected_file_digests`.
  Shared `frozen_registered_check_descriptor` helper owns argv/env/
  timeout/network=deny policy. Corpus helpers:
  `RepairCorpusFamily`, `reset_broken_repair_corpus`,
  `write_repaired_oracle_files`, `corpus_snapshot_digest`,
  `repaired_source_bytes`.
- Disposition: failure-first tests below must prove freeze + D02 gap.

### D01-LOCAL-01 — cargo fmt (Windows GNU allowlist)

- Instrument: `cargo fmt --all -- --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass`
- Note: Rust build/test/Clippy are `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`).

### D01-LOCAL-02 — check:consistency

- Instrument: `pnpm run check:consistency`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass` (275 requirements, 55 error codes, 74 schemas, 89 vectors,
  Personal plan/Gates, leases verified)

### D01-LOCAL-03 — handbook + docs-sync-gate

- Instrument: bilingual `execution-chain-status` update +
  `node tools/src/fill-handbook-fingerprints.mjs` +
  `node tools/src/docs-sync-gate.mjs --staged`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass` — `check-handbook` OK (54×2 locales); generator `--check` OK
  (18 pages byte-identical); docs-sync-gate OK without `DOCS_IMPACT_NONE`
