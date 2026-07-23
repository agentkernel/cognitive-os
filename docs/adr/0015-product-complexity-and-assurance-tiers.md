# ADR-0015: 产品复杂度边界与保障分层

- 状态：Accepted
- 日期：2026-07-23
- 决策范围：产品默认体验、近期执行优先级与 High-Assurance 延后策略

## 背景

CognitiveOS 的参考实现同时承载普通用户、一般企业和高保障场景的候选能力。
若将独立密码学审查、外部 verifier、长期留存/导出、多方审批和部署分离作为
默认产品或近期主战役前置条件，普通用户与一般企业会承担与其风险不相称的复杂度，
且会挤占停止、撤销、恢复、可解释状态和团队治理等更直接的价值。

## 决策

1. 采用“少即是多”：默认产品只暴露解决普通用户与一般企业真实任务所需的能力。
2. 当前默认范围是 Ordinary Core：认证通道、服务端授权、tenant/scope 隔离、
   capability 撤销、停止、reconcile、CAS、幂等、fencing、恢复和最小可读审计。
3. 独立签名审查、对象级 detached signature、外部 audit/verifier、checkpoint/export
   signing、法律留存/导出、R2/R3 多方审批及复杂配置 authority 统一归为
   **High-Assurance 扩展**；保留设计资产，但从近期产品交付和默认执行队列后置。
4. High-Assurance 未启用时，产品不得显示为“部分开启”或把简化路径宣传为独立审计、
   不可抵赖或监管合规。基础审计、失败关闭和权限隔离不因此削弱。
5. 重新启动任一 High-Assurance 工作项必须有明确客户/监管需求、指定受益用户、
   风险模型、预算，以及可验证的外部证据计划；仅“技术完整性”不是启动理由。

## 影响

- D-016/D-022 的事实与任何既有 registration/Profile claim 门禁保持不变；本决策仅
  调整产品优先级，不授权伪造独立审查、部署或法律证据。
- 当前工程优先处理 Ordinary Core 的可用性、可理解性、确定性安全边界和最小审计；
  High-Assurance 资产维持 deferred/tracking，不再作为普通产品开发的日常阻断。
- UI 与客户端后续必须按层级渐进披露：不向普通用户暴露 digest、epoch、fencing、
  key delegation 或复杂审批拓扑等内部机制。
