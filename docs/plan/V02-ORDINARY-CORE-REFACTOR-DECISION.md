# CognitiveOS v0.2 Ordinary Core 重构决定

- Decision ID: `V02-ORDINARY-CORE-01`
- 日期：2026-07-23
- 目标用户：普通个人、普通企业与常规 SaaS 部署
- 状态：**Ordinary Core development gate open; registration/claim gates remain closed**

## 1. 范围

| 层级 | 内容 | 对 Core 的关系 |
|---|---|---|
| Ordinary Core | authenticated channel、server-side restricted session、五个 deterministic management operations、capability/policy、CAS/idempotency/fencing、AUDIT lower、recovery | 必需 |
| High-Assurance | 三类 configure、detached object signatures、R2/R3、独立部署 AUDIT、独立 TARGET verifier、checkpoint/export/legal-hold/key delegation | 可选；不阻塞 Core |

## 2. Core operation 集

Core 必须闭合以下五项：

1. `session.create_restricted`；
2. `status.inspect`；
3. `capability.revoke`；
4. `execution.stop`；
5. `effect.reconcile`。

三类 configure 不属于 Core operation set。它们只能在 High-Assurance extension
被显式选择、完整注册并重新授权时出现。

## 3. Core AUDIT lower

最小实现包含：

- `ManagementAuditPort` deterministic interface；
- closed internal `privileged_read_decision` candidate record；
- contiguous per-scope sequence 与 writer epoch；
- commit receipt，绑定 record ID/digest、sequence 与 committed timestamp；
- `ResultReleaseGate`：只接受已成功 commit 且 subject/request 匹配的 receipt；
- audit failure/mismatch 时零成功结果；
- denial/not-found 使用相同公开响应形状，并只记录 safe selector digest。

Core 可使用同一进程内 SQLite/durable authority boundary，但接口、事务与测试必须
证明 audit 责任独立。独立部署是 High-Assurance 选择。

## 4. Core SIG

Core 使用 authenticated channel/workload identity 与 server-side current session
record。现有 `authority_signature` 字符串不得单独授予 authority。Detached signature
envelope、key registry、R2/R3 与对象级不可抵赖签名保留为 High-Assurance extension。

## 5. 非循环开发 gate

| 阶段 | 允许 | 禁止声明 |
|---|---|---|
| tracer | 内部 candidate types、失败测试、deterministic port/实现 | machine registered、behavior vector pass、Profile implemented |
| candidate freeze | 根据 tracer 反馈形成 schema-valid bytes/digests | published/selected |
| final review | 隔离 technical review 审查 exact final bytes | implementation 自动证明 registration |
| registration | registry/schema/bindings/new vectors | behavior pass、Profile implemented |
| CA-0 Core review | 对已登记 Core 合同与 tracer evidence 独立复核 | High-Assurance GO |

## 6. 第一条 tracer

状态（2026-07-23）：**内部 tracer 与 lightweight durable file adapter implementation
已提供；`admin-cli inspect` 产品路径已强制接线；management/admin-cli tests 已执行；
machine registration、conformance behavior 与 Profile claim 尚未提供。**

第一批只实现 `status.inspect` 的 audit-before-result path：

```text
session/capability gate
→ authority read
→ safe privileged-read audit record
→ durable audit commit receipt
→ receipt subject/request validation
→ InspectReport release
```

测试必须覆盖 audit commit failure、receipt subject mismatch、not-found/denial
isomorphism，以及零可见成功结果。该 tracer 不创建公开 schema 或 Profile claim。

已提供的内部 candidate surface：`ManagementAuditPort`、
`PrivilegedReadDecision`、`AuditCommitReceipt`、`ResultReleaseGate` 与
`ManagementPlane::inspect_with_audit`。未审计 `inspect` 已收紧为 crate-private 实现
原语，外部 crate 无法编译调用；
`FileManagementAuditLog` 提供进程级单写锁、重启递增 writer epoch、连续 sequence、
canonical JSONL readback 校验与 `sync_all` 后回执。当前唯一外部 `status.inspect`
产品入口 `admin-cli inspect` 已强制使用 audited tracer；默认日志位于
`<store>.management-audit.jsonl`，可用 `--audit` 覆盖。日志不可打开、锁冲突、损坏或
提交失败时不输出 inspect 结果。未来新增 HTTP/API 入口只能调用公开的 audited
release gate，不能绕过审计。
