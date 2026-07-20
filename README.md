# CognitiveOS Reference Implementation Monorepo

CognitiveOS 是一个"Agent control plane + durable governed runtime"的架构与参考实现工程。
本仓库同时承载**规范资产**（digest 固定的机器合同）与**参考实现**（Rust + TypeScript）。
规范与实现严格分离：规范资产的存在不代表实现存在；实现的存在不代表符合性成立。

## 四区导航

### 1. 规范（normative assets）

| 路径 | 内容 |
|---|---|
| [CognitiveOS-Architecture.md](CognitiveOS-Architecture.md) | 白皮书 v1.0.1（informative，非机器合同） |
| [RFC-0001-cognitiveos-governance-context-access.md](RFC-0001-cognitiveos-governance-context-access.md) | 治理/Context/访问 normative companion RFC v0.2 |
| [specs/](specs/) | 11 份 companion 规范 + registry（273 REQ / 55 错误码 / 5 状态域）+ 5 份状态迁移表 + 56 份 JSON Schema（draft 2020-12） |
| [CognitiveOS-Review-Conclusions.md](CognitiveOS-Review-Conclusions.md) · [CognitiveOS-Architecture-Independent-Review.md](CognitiveOS-Architecture-Independent-Review.md) | 两轮评审（V1–V17 / IMP-01~18 / F-001~F-030） |

### 2. 实现（reference implementation）

| 路径 | 内容 |
|---|---|
| [crates/](crates/) | Rust：contracts → domain → kernel/store → runtime/management/akp → conformance（依赖方向固定） |
| [apps/kernel-server](apps/kernel-server/) · [apps/admin-cli](apps/admin-cli/) | 单节点组合根 · 确定性管理 CLI |
| [apps/agent-shell](apps/agent-shell/) | 任务 Shell 客户端（非 authority） |
| [packages/](packages/) | TypeScript：contracts-ts（与 Rust 共享 golden fixtures）、sdk-ts |
| [apps/cognitiveos-console/](apps/cognitiveos-console/) | 兼容 stub（Console 产品文档正文迁至 [clients/pc/](clients/pc/)，`planned`） |
| [clients/](clients/) | 客户端项目根：PC/mobile/shared/Agent Hub 文档域（实现 gate 阻断，ADR-0007） |

### 3. 测试与证据（tests & evidence）

| 路径 | 内容 |
|---|---|
| [conformance/](conformance/) | 76 份声明式向量 + 15 测试层定义（数据包，非 runner） |
| [crates/cognitive-conformance](crates/cognitive-conformance/) | runner（M0：仅枚举，全量 `not-run`；执行能力 M1 交付） |
| [tests/](tests/) | golden（跨语言 canonical/digest 夹具）/ e2e / faults / security |
| [tools/](tools/) | 静态一致性检查（registry↔schema↔vector 无孤儿、断链检查），接入 CI |
| [artifacts/evidence/](artifacts/evidence/) | 运行证据（gitignore，凭 digest 引用） |

### 4. 文档体系（docs system）

| 路径 | 内容 |
|---|---|
| [AGENTS.md](AGENTS.md) | 开发代理入口：命令速查、硬纪律、Definition of Done、会话协议 |
| [docs/README.md](docs/README.md) | 文档地图与分类口径 |
| [clients/README.md](clients/README.md) | PC + 手机客户端项目地图与目录索引（canonical；`docs/clients/` 为兼容入口） |
| [docs/standards/](docs/standards/) | 12 份机器可判定标准（canonical/digest、状态迁移、错误合同…） |
| [docs/adr/](docs/adr/) | 架构决策记录（0001–0007） |
| [docs/plan/](docs/plan/) | 开发计划、进度仪表、并行车道 |
| [docs/traceability/](docs/traceability/) | REQ 追溯矩阵 + F/IMP findings 台账 |
| [docs/checkpoints/](docs/checkpoints/) | 会话交接与里程碑评审 |
| [docs/prompts/](docs/prompts/) | 各车道/里程碑接续提示词 |

## 四类状态用语（全仓强制）

任何文档与声明必须区分以下四类状态，不得混用（详见 [conformance/README.md](conformance/README.md)）：

1. **规范已登记（specified）**：REQ/schema/vector 在 registry 中存在。不代表任何实现存在。
2. **实现已提供（implementation available）**：代码存在且构建通过。不代表被测试证明。
3. **测试已执行（test executed）**：runner 真实执行向量并保留证据。schema-valid ≠ behavior-pass。
4. **Profile 已符合（implemented）**：该 Profile 全部适用 MUST 有通过证据或有据 not-applicable。安全负例不可被降级豁免。

当前状态：全部 273 条 REQ 为 `specified`；实现为 M0 骨架；全部向量 `not-run`；全部 Profile `planned`。

## 快速开始

```powershell
# Rust（工具链钉在 rust-toolchain.toml）
cargo build --workspace ; cargo test --workspace

# TypeScript（Node >= 22，pnpm 10）
pnpm install ; pnpm -r build ; pnpm -r test

# 静态一致性检查（registry↔schema↔vector↔docs）
pnpm run check:consistency

# 符合性 runner（M0：仅枚举，输出全 not-run 报告与样例 manifest）
cargo run -p cognitive-conformance --bin conformance-runner
```

`History/` 为冻结归档，不参与构建、schema bundle 与符合性声明，任何工具与文档不得引用。
