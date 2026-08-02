# CognitiveOS Architecture + CognitiveOS Personal

本仓库承载两层内容，但只有一个活动项目：

- **CognitiveOS Architecture**：Agent control plane 与 durable governed runtime 的设计、
  机器合同、符合性资产和可复用内核；它是架构参考层，不是并行产品项目。
- **`cognitiveos-personal` / CognitiveOS Personal**：当前唯一活动实现项目，复用并验证
  上述架构，目标是交付可安装、可治理、可验证的个人 Agent 产品。

项目身份和默认工作范围以
[PROJECT-IDENTITY.md](docs/governance/PROJECT-IDENTITY.md) 为准；Personal 正式任务以
[PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md) 为准，当前事实
以 [PROGRESS.md](docs/plan/PROGRESS.md) 的 `Current snapshot` 为准。规范存在不代表
实现存在；实现存在不代表 Gate、release 或 Profile 符合。

## 四区导航

### 1. 规范（normative assets）

| 路径 | 内容 |
|---|---|
| [CognitiveOS-Architecture.md](CognitiveOS-Architecture.md) | 白皮书 v1.0.2（informative，非机器合同） |
| [RFC-0001-cognitiveos-governance-context-access.md](RFC-0001-cognitiveos-governance-context-access.md) | 治理/Context/访问 normative companion RFC v0.2 |
| [specs/](specs/) | companion 规范、registry、状态迁移表与 JSON Schema（draft 2020-12）；具体数量由一致性检查报告 |
| [CognitiveOS-Review-Conclusions.md](CognitiveOS-Review-Conclusions.md) · [CognitiveOS-Architecture-Independent-Review.md](CognitiveOS-Architecture-Independent-Review.md) | 两轮评审（V1–V17 / IMP-01~18 / F-001~F-030） |

### 2. Personal 实现（唯一活动项目）

| 路径 | 内容 |
|---|---|
| [crates/](crates/) | Rust：contracts → domain → kernel/store → runtime/management/akp → conformance（依赖方向固定） |
| [apps/kernel-server](apps/kernel-server/) · [apps/admin-cli](apps/admin-cli/) | 单节点组合根 · 确定性管理 CLI |
| [apps/agent-shell](apps/agent-shell/) | Personal Task Shell 的共享客户端/会话核心（非 authority；供 Pi-hosted Shell 等入口复用） |
| [packages/pi-cognitiveos](packages/pi-cognitiveos/) | Pi-hosted Agent Shell Extension；Provider 流量经 daemon，Pi 工具 default-deny |
| [apps/pi-agent-adapter](apps/pi-agent-adapter/) | Pi compatibility/candidate adapter；不是 managed Pi lifecycle authority |
| [packages/](packages/) | TypeScript：contracts-ts、sdk-ts、Pi Shell host adapter |
| [docs/product/personal/](docs/product/personal/) · [docs/architecture/personal/](docs/architecture/personal/) | Personal canonical 产品设计 · 产品组合架构 |
| [apps/cognitiveos-console/](apps/cognitiveos-console/) | 兼容 stub（Console 产品文档正文迁至 [clients/pc/](https://github.com/agentkernel/cognitiveos-clients/tree/main/pc)，`planned`） |
| [cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)（独立仓库） | 客户端项目根：PC/mobile/shared/Agent Hub 文档域（实现 gate 阻断，ADR-0007）。2026-07-26 已从本仓库 `clients/` 拆出，本仓库不再包含该目录 |

### 3. 测试与证据（tests & evidence）

| 路径 | 内容 |
|---|---|
| [conformance/](conformance/) | 声明式规范向量与测试层定义；执行结果必须由 runner/evidence 报告，不由 README 固定计数 |
| [crates/cognitive-conformance](crates/cognitive-conformance/) | 符合性 runner；局部通过不自动构成 Personal Gate 或 Profile 结论 |
| [tests/](tests/) | golden（跨语言 canonical/digest 夹具）/ e2e / faults / security |
| [tools/](tools/) | 静态一致性检查（registry/schema/vector/docs/Personal 治理关系），接入 CI |
| `artifacts/evidence/` | 运行证据目录（gitignore，凭 digest 引用；由本地 runner 按需创建） |

### 4. 文档体系（docs system）

| 路径 | 内容 |
|---|---|
| [AGENTS.md](AGENTS.md) | 开发代理入口：命令速查、硬纪律、Definition of Done、会话协议 |
| [docs/README.md](docs/README.md) | 文档地图与分类口径 |
| [docs/product/personal/](docs/product/personal/) | 愿景、资源模型、Linux 1.0 范围和用户旅程（不拥有当前状态） |
| [docs/architecture/personal/](docs/architecture/personal/) | Agent Shell、managed Agent、authority、数据与恢复组合设计 |
| [clients/README.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md) | PC + 手机客户端项目地图与目录索引（canonical；`docs/clients/` 为兼容入口） |
| [docs/standards/](docs/standards/) | 机器可判定行为标准（canonical/digest、状态迁移、错误合同、文档同步等） |
| [docs/adr/](docs/adr/) | 架构与 Personal 产品决策记录；Linux 1.0/Pi 双角色见 ADR-0035/0036 |
| [docs/plan/](docs/plan/) | Personal 正式计划、current snapshot、lease、支持矩阵和测试环境注册表 |
| [docs/traceability/](docs/traceability/) | REQ 追溯矩阵 + F/IMP findings 台账 |
| [docs/checkpoints/](docs/checkpoints/) | 会话交接与里程碑评审 |
| [docs/prompts/](docs/prompts/) | 历史/复用提示词；不能生成当前 Personal task、lease、Gate 或状态 |

## 四类状态用语（全仓强制）

任何文档与声明必须区分以下四类状态，不得混用（详见 [conformance/README.md](conformance/README.md)）：

1. **规范已登记（specified）**：REQ/schema/vector 在 registry 中存在。不代表任何实现存在。
2. **实现已提供（implementation available）**：代码存在且构建通过。不代表被测试证明。
3. **测试已执行（test executed）**：runner 真实执行向量并保留证据。schema-valid ≠ behavior-pass。
4. **Profile 已符合（implemented）**：该 Profile 全部适用 MUST 有通过证据或有据 not-applicable。安全负例不可被降级豁免。

当前 Personal task、implementation evidence、Gate 和 claim scope 以
[PROGRESS](docs/plan/PROGRESS.md) 的 `Current snapshot` 为准。README 不复制易漂移的
REQ、vector 或结果计数；任何 release/Profile 声明必须链接对应 campaign 与独立 verifier
证据。

## 快速开始

```powershell
# Rust（工具链钉在 rust-toolchain.toml）
cargo build --workspace ; cargo test --workspace

# TypeScript（Node >= 22，pnpm 10）
pnpm install ; pnpm -r build ; pnpm -r test

# 静态一致性检查（registry↔schema↔vector↔docs）
pnpm run check:consistency

# 符合性 runner（输出五态报告、证据与样例 manifest）
cargo run -p cognitive-conformance --bin conformance-runner
```

`History/` 为冻结归档，不参与构建、schema bundle 与符合性声明，任何工具与文档不得引用。
