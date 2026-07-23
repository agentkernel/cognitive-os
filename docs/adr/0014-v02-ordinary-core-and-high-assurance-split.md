# ADR-0014: v0.2 Ordinary Core 与 High-Assurance 扩展分层

- 状态：Accepted
- 日期：2026-07-23
- 决策范围：v0.2 Configuration Authority / Management / AUDIT / SIG / TARGET

## 背景

原 v0.2 治理方案把八个 operation、四条 registration family、对象级签名、
独立部署 consumer、独立 readback/verifier、checkpoint/export signing、法律保留
与新 negotiation epoch 同时作为开发前置条件。这适合高保障控制面，但对普通个人
和普通企业形成循环 gate：没有 final contract 就无法构建真实 consumer，没有真实
consumer 又不能冻结 contract，而 CA-0 GO 前又禁止实现。

## 决策

1. v0.2 默认交付范围为 **Ordinary Core**，面向单组织或常规多租户企业部署。
2. Ordinary Core operation 集为：`session.create_restricted`、`status.inspect`、
   `capability.revoke`、`execution.stop`、`effect.reconcile`。
3. `system.configure`、`gateway.configure`、`diagnostics.configure` 属于
   **High-Assurance 扩展**，不阻塞 Ordinary Core 开发或发布。
4. Ordinary Core 仍强制 authenticated management channel、server-side session
   authority、capability/policy intersection、CAS、idempotency、fencing、AUDIT
   before result visibility 与 recovery/reconcile。
5. Ordinary Core 不要求对象级 detached signature、R2/R3、独立部署 audit service、
   独立 TARGET verifier、checkpoint/export signing、tenant key delegation 或法律
   export profile。它们保留为 High-Assurance 扩展，不删除、不弱化其适用场景安全性。
6. Ordinary Core AUDIT lower 可以是同一受信二进制内的独立 deterministic port 与
   durable store；逻辑责任、失败语义和测试必须独立，但部署分离不是 Core MUST。
7. Core consumer gate 由真实代码路径和行为测试证明：ResultReleaseGate 必须消费
   audit commit receipt；fixture、布尔值或日志文本不能代替。
8. 开发顺序改为：候选合同/内部类型 → 先失败测试 → tracer implementation →
   implementation feedback → final candidate bytes/digests → 独立 final review →
   registration → generated bindings/vectors → CA-0 Core review → broader rollout。
9. 独立审查是 final candidate registration 前的最后 gate，不阻塞早期 tracer
   implementation，但未经审查不得 registration 或 Profile claim。

## 安全不变量

- 概率组件不得授权、提交、修改状态或生成成功声明。
- AUDIT commit/receipt 失败时不得释放成功结果。
- 未认证 channel、过期 session、越权 capability、stale CAS/writer epoch 均 fail closed。
- High-Assurance 功能未启用时必须显式不可选，不能静默降级为 Core 语义。

## 影响

该决策允许立即开发最小 `status.inspect` + AUDIT lower tracer path，同时保持所有
High-Assurance 设计资产为 later extension。原 D-016 “八项同时完成”决定被本 ADR
取代；D-022 分为 Core blocker 与 High-Assurance deferred blocker。该决策本身不
登记 machine asset、不执行行为向量、不产生 Profile implemented claim。
