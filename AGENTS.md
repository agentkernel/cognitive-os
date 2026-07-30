# AGENTS.md — CognitiveOS 参考实现开发代理入口

新会话接入：**① 读本文件与 [Development Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md) → ② 读正式任务计划 → ③ 读 `PROGRESS.md` 当前快照 → ④ 读与所选任务/车道匹配的最新 handoff**，确认活动 ownership lease 后领取任务。handoff 承载操作连续性，但不得覆盖正式任务或 Gate 状态；禁止依赖对话历史承载工程状态。

## 命令速查

| 目的 | Windows PowerShell（本地） | CI（bash） |
|---|---|---|
| Rust 构建 | `cargo build --workspace` | 同左 |
| Rust 测试 | `cargo test --workspace` | 同左 |
| Rust lint | `cargo clippy --workspace --all-targets` | 同左 |
| TS 安装 | `pnpm install` | `pnpm install --frozen-lockfile` |
| TS 构建/测试 | `pnpm -r build ; pnpm -r test` | `pnpm -r build && pnpm -r test` |
| 静态一致性检查 | `pnpm run check:consistency` | 同左 |
| 本地一键 Boot→Verify→Perf（non-claim） | `pnpm run verify:local` | 同左（见 `docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md`） |
| 符合性 runner（枚举） | `cargo run -p cognitive-conformance --bin conformance-runner` | 同左 |
| 跨语言 golden 对比 | 见 `.github/workflows/ci.yml` golden job | CI 自动 |

本机若 `cargo` 不在 PATH：`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`。工具链钉在 `rust-toolchain.toml`（1.97.1）。

## 目录地图

```text
specs/            规范资产（registry 273 REQ / 55 错误码；5 迁移表；61 schema）——机器合同真相
conformance/      84 份声明式向量 + 15 测试层（数据包，非 runner）
crates/           Rust：contracts → domain → kernel/store → runtime/management/akp → conformance
apps/             kernel-server、admin-cli（Rust）；agent-shell（TS 客户端）；cognitiveos-console（兼容 stub，正文迁独立仓 cognitiveos-clients）
packages/         contracts-ts、sdk-ts
（clients/ 已拆出）客户端项目根迁至独立仓库 https://github.com/agentkernel/cognitiveos-clients（ADR-0007；2026-07-26 拆分）
tests/            golden（跨语言夹具）/ e2e / faults / security
tools/            静态一致性检查（Node）
docs/             standards / adr / plan / traceability / checkpoints / prompts / evaluation
artifacts/        运行证据（gitignore）
History/          冻结归档：禁止读取、引用、参与构建
personal-blog/    嵌套独立仓 CognitiveOS Research（根 .gitignore；远程 github.com/agentkernel/blog；不入 Cos origin/main）
docs/_local/      本机草稿（gitignore；非 Cos 交付物）
```

## 硬纪律摘要（工具无关正文见 `docs/governance/`；`.cursor/rules/` 仅可作为本机编辑器适配层）

1. **确定性边界**：概率组件只产 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing 与最终提交必须由确定性代码执行。
2. **规范优先级**：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议；冲突取不扩大权限/范围/风险/预算/完成声明的解释。
3. **状态正交**：任务状态、实现证据、产品 Gate 与 release/Profile 声明分列；首个任务专属实现/测试批即把任务标为 `in-progress`，但 local/WSL/fixture/CI 不得升级正式 Gate。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；完成证明只来自 authority 状态、Effect、Verification 与 Event。
5. **规范表面冻结**：v0.1 前不新增对象族、Profile、REQ 域；只允许修正型规范变更（IMP-01）。
6. **P0 门禁**：开放 P0 必须列出 `blocked_paths` / `blocked_task_ids` / `blocked_gate_ids`；只阻断这些范围的验收/推广，不阻断调查、测试和修复工作。
7. **可追溯提交**：每个提交/PR 关联 REQ-ID、F/IMP 条目或文档条目；确无关联时写明原因。

## 四类状态用语

| 用语 | 含义 | 不代表 |
|---|---|---|
| 规范已登记 specified | REQ/schema/vector 在 registry 存在 | 实现存在 |
| 实现已提供 | 代码存在且构建通过 | 行为被证明 |
| 测试已执行 | runner 真实执行并保留证据 | Profile 符合 |
| Profile 已符合 implemented | 全部适用 MUST 有通过证据或有据 not-applicable | ——（安全负例不可豁免） |

## 分阶段验证与 Definition of Done

1. **提交前：** failure-first 证据（行为变更）、受影响 package 测试/lint/format、相关负例和 diff/secret 检查通过；不得存在已知受影响失败。
2. **push 前：** 在受支持且可用的本地工具链执行相关广域回归/一致性检查，未执行项准确记为 `not-run` 并说明原因；核对 staged 与完整 push 面。
3. **merge/任务 done 前：** 所需 Windows/Linux protected CI 全绿；相关向量真实 `pass` 或有据 `not-applicable`；状态、当前快照和 handoff 已对齐。
4. commit 可以在 remote CI pending 时存在；required red check 禁止 merge、禁止任务完成声明。非支持本地环境不是 pass，也不是隔离 commit 的自动阻断。
5. 文档联动按 `docs-sync-contract.md` 四分类执行；实现未改 normative surface 时走 `implementation-only`，不得为凑联动修改 registry/schema/vector。

## 会话协议

- 开始：AGENTS/Operating Model → 正式任务计划 → PROGRESS 当前快照 → 当前任务/车道 handoff → 活动 ownership lease。
- 结束或移交：更新正式状态与 PROGRESS 当前快照 → 写 handoff（已完成/未完成、实现 commit、测试/证据、non-claims、风险、下一入口、remote visibility）→ 完成 closure docs commit。
- 实现与 closure docs 必须属于同一 atomic delivery/PR，不要求同一 commit；允许 handoff commit 引用前一实现 commit 的 immutable hash。
- 上下文接近极限：提前执行结束协议，剩余工作写入接续提示词。

## 自动提交与自动 push（所有者已授权）

通过提交前验证的原子批由代理自动提交并 push，无需逐次请示（ADR-0008）。硬条件：禁止提交已知受影响失败；逐路径 `git add`；push 前必查 `git log --name-only origin/main..HEAD`；禁止 force-push；禁止推送 `personal-blog/**`；docs-only 低风险批可直推 main，代码批走 lane 分支 + PR，并仅在 required CI 全绿后合并。

## 红线

- 禁止读取/引用 `History/`；禁止虚构规范资产；禁止改写向量迎合实现。
- 既有未提交改动（他人工作区状态)不覆盖、不回退、不混入自己的提交（逐路径 `git add`，禁 `git add -A`）。
- Console 车道未过后端 gate 前只维护依赖台账（`docs/plan/DEVELOPMENT-PLAN.md` Console 节），不启动实现。
- **`personal-blog/`**：唯一副本在本工作树该目录；远程固定 `https://github.com/agentkernel/blog.git`；禁止推入 Cos；禁止为对齐 Cos 基线而删除/清空嵌套仓；禁止在 `D:\blog-*` 等路径散落平行克隆。
