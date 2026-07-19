# 20260720 M0 Milestone Review

## 1. 范围回顾

M0 = 工程基线与开发体系（`docs/plan/DEVELOPMENT-PLAN.md` M0 节；引导提示词 `docs/prompts/00-bootstrap-dev-system.md` 任务 A–F）。本评审 = M0 出口评审，逐条对照引导提示词 §9 的 M0 验收清单。

## 2. 验收判据逐条对照

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 既有规范资产与 `apps/cognitiveos-console/` 未被移动、重命名或改写；`History/` 未被读取或引用 | **通过（一处声明的最小修正除外）** | 本会话仅对 `specs/schemas/state-transition-table.schema.json` 做一处最小修正（D-005：`version` const→enum，先登记后闭合，提交说明注明）；console 目录本会话零改动；全部工具与文档排除 `History/`（`tools/src/lib.mjs` EXCLUDED_DIR_NAMES + 检查器拒绝指向 History 的链接/owner_spec）。会话期间工作区出现**并行会话的未提交改动**（36 份 schema 单轨迁移、`apps/cognitiveos-console/` 产品文档改写与新增 docs/），已全程保护、未混入本会话提交（见 handoff §4） |
| 2 | 空实现下 `cargo build && cargo test`、`pnpm -r build && pnpm -r test` 通过 | **通过** | 本地 Rust 1.97.1（gnu）与 Node 22/pnpm 10.33.2 全绿；Rust 15 个单测/集成测试、TS 12 个测试（含 golden 双向验证） |
| 3 | CI 在 Windows + Linux 矩阵可运行，含静态一致性检查与跨语言 golden digest 对比 | **通过（本地等价验证；远端待推送）** | `.github/workflows/ci.yml`（windows-latest + ubuntu-latest 全步骤矩阵）；本地逐步执行同一命令序列全绿；仓库尚无 remote（待决项 P-3） |
| 4 | `.cursor/rules/` 9 份规则、`AGENTS.md`、`docs/README.md` 就绪且 frontmatter 正确 | **通过** | `00/01/02`（alwaysApply: true）+ `10~15`（globs）共 9 份，每份 ≤50 行三段式；既有 `use-skills.mdc` 未动 |
| 5 | 8 份标准 + 4 份 ADR 补齐，格式与既有一致 | **通过** | `error-contract / authn-authz-capability / context-resolution-and-cache / intent-effect-idempotency / event-audit-watch / task-loop-verification / akp-envelope-and-http-profile / conformance-evidence`（H1+元数据列表+编号节，v0.1 Draft，只引用既有 REQ）；ADR-0001/0002/0003/0006（Context/Decision/Alternatives/Consequences/Compliance checks，注明参考实现决策） |
| 6 | 计划/进度/车道/matrix/台账/checkpoints 模板/prompts 全套就绪 | **通过** | `DEVELOPMENT-PLAN.md`（M0~M11+Console 依赖台账+映射表+风险）、`PROGRESS.md`、`PARALLEL-LANES.md`、`matrix.yaml`（273 条派生骨架 + gen-matrix 工具）、`findings-ledger.md`（F-001~030 + IMP-01~18 逐条 + 漂移 D-001~D-006）、`checkpoints/TEMPLATE.md`、`prompts/`（common-prefix + 7 车道 + M1~M6 = 14 份） |
| 7 | registry↔schema↔vector 双向无孤儿（或漂移已列明并最小修正闭合） | **通过** | `node tools/src/check-consistency.mjs` 绿：273 REQ 全有测试映射、74 测试 ID↔74 向量双向闭合、向量 REQ/错误码全在 registry、56 schema 全可达且 `$ref` 全解析；漂移 D-001~D-006 登记（D-002/003/005 闭合，D-001/004/006 排 M1） |
| 8 | runner 骨架对全部向量报告 not-run；样例 profile manifest 全部标 planned | **通过** | `cargo run -p cognitive-conformance --bin conformance-runner`：74 向量全 not-run（15 层 + contract-traceability 跨切片）；样例 manifest 13 Profile 全 planned，过 `tools/src/validate-manifest.mjs` schema 校验 |
| 9 | 故意注入的孤儿引用/断链能被 CI 捕获（验证后已回滚） | **通过** | 注入演练记录见 §4 |
| 10 | 全部工作分批提交，提交信息引用对应任务/文档条目 | **通过** | 提交列表见 §6 |

## 3. 安全负例清单（M0 阶段）

M0 不实现业务逻辑，安全负例为编码层：golden 负例夹具 8 条（重复键、BOM、非法 UTF-8、不安全整数、NaN、孤立代理项转义、未转义控制符、尾随内容）在 Rust 与 TS 双侧执行并要求同类别拒绝；域分离负例（空/保留/大写域名拒绝）双侧单测覆盖。行为级安全负例自 M2 起按里程碑验收执行。

## 4. 注入演练记录（docs-sync-contract §5 破坏性验证）

临时分支 `drill/docs-sync-injection` 注入两处缺陷：① 在 `docs/plan/PROGRESS.md` 写入一个 registry 中不存在的 REQ 编号（CTX 域 999 号，孤儿引用）；② 在 `docs/README.md` 加一条指向不存在文件 `standards/no-such-standard.md` 的相对链接（断链）。`node tools/src/check-consistency.mjs` 以退出码 1 失败并逐条指出文件与原因（输出摘录见同日 handoff §3）。验证后分支已删除，主工作区未受影响。

## 5. 漂移与规范变更

见 `docs/traceability/findings-ledger.md` §三：D-001~D-006 全部登记；M0 内闭合 D-002（术语收口）、D-003（实测计数口径）、D-005（迁移表版本枚举，唯一一处对既有资产的最小修正）；D-001/D-004/D-006 排入 M1（Lane-CTR/CFR）。

## 6. 指标快照与提交

- REQ：273 specified / 0 实现 / 0 已测试；向量：74 全 not-run；Profile：13 全 planned。
- 开放 P0：F-003（阻断 M1 出口）+ F-001（证据性质）；开放 P1：F-011/F-014/F-023/F-017（+F-015 持续）。
- 提交列表：见 git log（本次会话按任务 A–F 分批提交，哈希在 handoff §6）。

## 7. 结论

**GO → M1**。入口条件已满足（M0 出口通过）；M1 使用 `docs/prompts/milestone-m1.md`（或拆分 `lane-ctr.md` + `lane-cfr.md` 两个 Multitask 会话）。遗留条件：待决项 P-1（许可证）~P-4（pnpm 版本）不阻塞，见 DEVELOPMENT-PLAN §6。
