# Lane-CTR 接续提示词：V02-CA-OPS-01

> 当前唯一入口：v0.2 Configuration Authority Management operation-set 规范设计与评审。此提示词不授权 TARGET/SIG/AUDIT 字段设计或任何实现。

你是 CognitiveOS 参考实现的 Lane-CTR 工程代理，工作目录为仓库根 `agent-kernel`。开工先保护一切既有未提交/未跟踪内容：只记录路径，不读取旁路业务内容，不清理、不覆盖、不暂存；禁止读取 `History/**`，禁止访问或触碰 `personal-blog/**`。

## 接入顺序

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最新 `docs/checkpoints/*-handoff.md`。
4. 读 `docs/plan/PARALLEL-LANES.md`。
5. 重点读：
   - `docs/plan/V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md`
   - `docs/adr/0009-v02-configuration-authority-surface-expansion-governance.md`
   - `docs/plan/CONFIGURATION-AUTHORITY-SPEC-CORRECTION-DECISION.md`
   - `docs/plan/CONFIGURATION-AUTHORITY-CONTRACT-DECISION.md`
   - findings-ledger 的 D-016、D-022、IMP-01
   - `docs/standards/normative-source-and-versioning.md`
   - `docs/standards/canonical-encoding-and-digest.md`
   - `docs/standards/docs-sync-contract.md`

从最新 `origin/main` 创建独立 Lane-CTR 分支；一个原子任务、一个 PR。不得复用旧 `lane/ctr-config-authority-spec-correction` 历史。

## 任务：V02-CA-OPS-01

仅对 Management operation set 做独立、逐项、先失败规范设计与评审，承接所有者批准的模型：

> v0.2 封闭核心集合 + digest-pinned、显式协商的版本化扩展集合

必须裁决核心集合准确成员，并为每个获准 operation 绑定 request/result、channel、risk/permission、target/readback 与 error mapping。未知或未协商 operation 必须 fail closed；operation 名称不得自动扩大 session scope、capability 或 authorization；reachability vector 不得扩张为完整业务、wire 或权限合同。

本批必须遵守 v0.2 breaking Draft 边界：新 specification set/digest、release notes、有限 compatibility window、migration plan 与新 negotiation epoch；禁止修改既有已发布/digest-pinned 资产字节或复用旧 SemVer/digest 身份。

## 边界

- TARGET 只记录 OPS 的 configure operation 决定和待承接接口，不在 OPS 批预定义 configuration 对象族、payload 字段、consumer/readback/receipt 细节。
- 不预选 signature 算法、密钥体系或允许集合；不设计 audit record 字段或 persistence 结构。
- 不新增 REQ 域、Profile 或第六 execution lifecycle；一般 authority-managed state domain 不得解释成 execution lifecycle。
- 不启动 CA-1～CA-8、KRN/RUN/CFR 实现；不修改既有 vector `expected`；runner 对真实实现执行前相关向量保持 not-run。
- 如果 OPS 设计需要超出授权决议 allowed surface，立即 NO-GO 并请求新的所有者裁决。

## 状态纪律与门禁

当前状态必须保持：v0.2 surface **owner-approved / design pending**；D-016 不 closed；D-022 继续 blocker；machine contracts 未登记（直到 OPS PR 实际登记并合入）；实现未提供；新行为测试未执行；Profile implemented = 0；CA-1～CA-8 blocked。

OPS PR 合入不自动解除 D-022，也不允许启动实现。必须等待 TARGET、SIG、AUDIT 四类机器合同全部合入，并通过独立 CA-0 re-review GO。

## 完成与交接

按 docs-sync-contract 完成适用的 companion/registry/error/schema/generated/vector/matrix/ADR/migration/release-note 联动；新增安全负例，不修改既有 vector expected。完成前执行全套一致性、构建、测试与 CI 两 OS 门禁；逐路径暂存。更新 PROGRESS、PARALLEL-LANES、findings-ledger 并按模板新建 handoff。

第一个动作：核对 `origin/main`、工作区、最新 CI 和 `V02-CA-GOV-00`，建立 OPS 先失败 GO/NO-GO checklist；在 checklist 通过前不写机器合同。
