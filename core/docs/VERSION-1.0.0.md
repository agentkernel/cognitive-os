# cognitiveos-core 1.0.0 — 边界定义与定稿说明

- Status: **finalized（定稿）** — 1.0.0 开发已完成；本文件与 annotated tag
  `core-v1.0.0` 共同构成定稿记录
- Date: 2026-08-25
- Decision anchor: [ADR-0054](../../docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
- Change authority: core 合同资产的任何语义变更仍必须走 Lane-CTR 契约流程
  （schema/transition/vector/生成物一体变更），本定稿不冻结缺陷修复，但冻结
  1.0.0 的范围定义

## 1. 一句话定义

`cognitiveos-core` 1.0.0 是**产品中立、可机器验证的合同与权威基底**：它定义
CognitiveOS 的机器合同、确定性权威原语、协议边界和可移植符合性资产，不拥有
任何 Personal / Enterprise 产品组合。

## 2. 1.0.0 范围（boundary）

| 资产 | 路径 | 内容 |
|---|---|---|
| 机器合同源 | `core/specs/` | registry（requirements）、schemas、transitions、error 目录及 normative companions |
| Rust 合同绑定 | `core/crates/cognitive-contracts` | 由 specs 生成的 schema bindings、canonical JSON、digest、ID、error 类型与 `contracts-codegen` |
| 领域状态机 | `core/crates/cognitive-domain` | 五个执行生命周期状态机与纯不变量（无 I/O） |
| 权威原语 | `core/crates/cognitive-kernel` | authority、CAS、capability、budget、Effect 协议、checkpoint/recovery 与 port traits |
| Agent 协议 | `core/crates/cognitive-akp` | AKP envelope 与 HTTP/SSE transport profile |
| TS 合同绑定 | `core/packages/contracts-ts` | 与 Rust 同源生成的 TS 合同与 golden fixtures |
| 符合性向量 | `core/conformance/` | normative conformance vectors 与 suite 说明 |
| 跨语言金样 | `core/tests/golden/` | canonical-json 与 digest/projection 金样，Rust/TS 双方消费 |
| 架构文档 | `core/docs/architecture/` | CognitiveOS 白皮书、RFC 与架构评审结论（informative） |

**明确不属于 core 1.0.0：** 任何产品组合（daemon 组合根、CLI、installer、
deploy）、带 Personal 语义的 store/runtime/management 实现、SecretStore 与
Provider transport、Pi/dsh 具体 adapter、UI。`cognitive-conformance` runner
crate 依赖 reference implementation 全栈，作为 reference-IUT harness 归属
`personal/`；本定稿的符合性资产指向量与 suite 语义本身。

## 3. 验收标准与达成证据

全部验收标准都映射到已存在并常绿的机器门（required CI 于 kernel `main`）：

| # | 验收标准 | 机器门 / 证据 | 状态 |
|---|---|---|---|
| 1 | 全部 registered schema 可编译且 registry↔schema↔vector 无孤儿 | `tools/src/check-consistency.mjs`（required CI 步骤） | pass |
| 2 | Rust/TS bindings 与 specs 同源、再生成零漂移 | CI codegen 再生成 + dirty-diff 门 | pass |
| 3 | 跨语言 canonical JSON / digest 金样相等 | CI golden digest 对比（`tests/golden` + `emit-golden`） | pass |
| 4 | conformance runner 对 reference implementation 全向量通过 | CI `conformance-runner` 步骤 | pass |
| 5 | core crates（contracts/domain/kernel/akp）零 Personal 依赖 | 根 `Cargo.toml` 依赖方向（contracts→domain→kernel；akp 仅依赖 contracts）；ADR-0054 依赖方向规则 | pass |
| 6 | 需求可追踪 | `docs/traceability/matrix.yaml`（`gen-matrix` 生成、CI 校验） | pass |

## 4. 定稿声明

在上述验收标准全部满足的前提下，owner 于 2026-08-25 定稿
`cognitiveos-core` 1.0.0：范围以本文件 §2 为准，达成证据以 §3 所列机器门在
定稿 revision（tag `core-v1.0.0`）上的通过为准。

定稿之后：

- 1.0.0 范围内合同的语义变更属于 post-1.0 合同演进，必须走 Lane-CTR 并附
  迁移说明；不得为实现方便弱化既有 negative/vector（公理 A6）。
- 本定稿是 owner 的版本边界决定，不将任何 MVP Gate 证据升级为 Profile、
  release-distribution 或性能声明。

## 5. Post-1.0 方向（core）

- 混合 crate（store/runtime/management）中可复用部分的内部拆分与 core 收编
  （ADR-0054 登记的后续重构）；
- 可发布 artifact 链（signed spec bundle、crates/npm prerelease、SBOM/
  provenance）——仅在出现真实外部消费者时启动；
- conformance suite 与 reference-IUT harness 的包级分离。
