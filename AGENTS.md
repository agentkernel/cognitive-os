# AGENTS.md — CognitiveOS 参考实现开发代理入口

新会话三步接入：**① 读本文件 → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近一份 `docs/checkpoints/*-handoff.md`**，确认车道与任务边界（`docs/plan/PARALLEL-LANES.md`）后领取任务。交接文档是跨会话唯一记忆载体，禁止依赖对话历史承载工程状态。

## 命令速查

| 目的 | Windows PowerShell（本地） | CI（bash） |
|---|---|---|
| Rust 构建 | `cargo build --workspace` | 同左 |
| Rust 测试 | `cargo test --workspace` | 同左 |
| Rust lint | `cargo clippy --workspace --all-targets` | 同左 |
| TS 安装 | `pnpm install` | `pnpm install --frozen-lockfile` |
| TS 构建/测试 | `pnpm -r build ; pnpm -r test` | `pnpm -r build && pnpm -r test` |
| 静态一致性检查 | `pnpm run check:consistency` | 同左 |
| 符合性 runner（枚举） | `cargo run -p cognitive-conformance --bin conformance-runner` | 同左 |
| 跨语言 golden 对比 | 见 `.github/workflows/ci.yml` golden job | CI 自动 |

本机若 `cargo` 不在 PATH：`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`。工具链钉在 `rust-toolchain.toml`（1.97.1）。

## 目录地图

```text
specs/            规范资产（registry 273 REQ / 55 错误码；5 迁移表；61 schema）——机器合同真相
conformance/      84 份声明式向量 + 15 测试层（数据包，非 runner）
crates/           Rust：contracts → domain → kernel/store → runtime/management/akp → conformance
apps/             kernel-server、admin-cli（Rust）；agent-shell（TS 客户端）；cognitiveos-console（兼容 stub，正文迁 clients/）
packages/         contracts-ts、sdk-ts
clients/          客户端项目根（PC/mobile/shared/Agent Hub 文档；实现 gate 阻断；ADR-0007）
tests/            golden（跨语言夹具）/ e2e / faults / security
tools/            静态一致性检查（Node）
docs/             standards / adr / plan / traceability / checkpoints / prompts / evaluation
artifacts/        运行证据（gitignore）
History/          冻结归档：禁止读取、引用、参与构建
```

## 硬纪律摘要（全文见 `.cursor/rules/`）

1. **确定性边界**：概率组件只产 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing 与最终提交必须由确定性代码执行。
2. **规范优先级**：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议；冲突取不扩大权限/范围/风险/预算/完成声明的解释。
3. **四类状态用语**：规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合，严格区分；`implemented` 仅指全部适用 MUST 有通过证据。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；完成证明只来自 authority 状态、Effect、Verification 与 Event。
5. **规范表面冻结**：v0.1 前不新增对象族、Profile、REQ 域；只允许修正型规范变更（IMP-01）。
6. **P0 门禁**：`docs/traceability/findings-ledger.md` 中开放的 P0 未闭合前，对应子系统不得进入实现里程碑。
7. **可追溯提交**：每个提交/PR 关联 REQ-ID、F/IMP 条目或文档条目；确无关联时写明原因。

## 四类状态用语

| 用语 | 含义 | 不代表 |
|---|---|---|
| 规范已登记 specified | REQ/schema/vector 在 registry 存在 | 实现存在 |
| 实现已提供 | 代码存在且构建通过 | 行为被证明 |
| 测试已执行 | runner 真实执行并保留证据 | Profile 符合 |
| Profile 已符合 implemented | 全部适用 MUST 有通过证据或有据 not-applicable | ——（安全负例不可豁免） |

## Definition of Done（每个任务/PR）

1. CI 全绿（Windows + Linux 矩阵：Rust 构建测试、TS 构建测试、静态一致性检查、跨语言 golden digest 对比）。
2. 相关向量 `pass`，或 `not-applicable` 有据（M0 阶段：保持 `not-run`，不得虚报）。
3. 文档联动完成（`docs/standards/docs-sync-contract.md` 三分类义务）。
4. `docs/plan/PROGRESS.md` 已更新。
5. （会话结束时）handoff 已写入 `docs/checkpoints/`。

## 会话协议（详见 `.cursor/rules/02-workflow-docs-sync.mdc`）

- 开始：AGENTS.md → PROGRESS → 最近 handoff → 领取任务。
- 结束：更新 PROGRESS → 写 handoff（已完成/未完成 REQ、提交哈希、测试与证据状态、未决风险与漂移、下一步入口与建议提示词路径）→ 分批提交。
- 上下文接近极限：提前执行结束协议，剩余工作写入接续提示词。

## 红线

- 禁止读取/引用 `History/`；禁止虚构规范资产；禁止改写向量迎合实现。
- 既有未提交改动（他人工作区状态)不覆盖、不回退、不混入自己的提交（逐路径 `git add`，禁 `git add -A`）。
- Console 车道未过后端 gate 前只维护依赖台账（`docs/plan/DEVELOPMENT-PLAN.md` Console 节），不启动实现。
