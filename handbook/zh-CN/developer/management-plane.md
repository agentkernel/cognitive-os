---
doc_id: dev.management-plane
locale: zh-CN
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: crates/cognitive-management/src/plane.rs
    symbols: ["ManagementPlane", "reconcile"]
  - path: crates/cognitive-management/src/session.rs
    symbols: ["PrivilegedManagementSession"]
  - path: crates/cognitive-management/src/approval.rs
    symbols: ["ApprovalGate"]
  - path: crates/cognitive-management/src/audit.rs
    symbols: ["FileManagementAuditLog", "ResultReleaseGate"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
  - path: apps/admin-cli/src/main.rs
tests:
  - crates/cognitive-management/tests/m5_session_approval.rs
  - apps/admin-cli/tests/m5_deterministic_fallback.rs
  - apps/admin-cli/tests/p2_t27_pi_lifecycle.rs
fingerprint: "sha256:7f3c8b16d4b03bce656e63d84fb00f38d28e37a7556cbdb9db6c5b535b97f156"
non_claims:
  - R0/R2/R3 审批流与治理台账的 daemon 生产接线未实现；只存在此处列出的部分。
---

# 管理面

确定性回退：当模型、Pi 或对话路径不可用时，`admin-cli` + `cognitive-management` 仍
可对同一权威库 inspect、stop、revoke、reconcile——该路径全程无模型 SDK。

## 会话与风险层级

每个动词都要求 `PrivilegedManagementSession` JSON 文档：schema 合法、purpose 绑定、
风险分层（R0–R3）、生命周期管理（issue/renew/revoke、绝对 + 空闲过期）。`inspect`
需 R1+，变更需 R2+，R1 类提案还需逐操作审批记录（`ApprovalGate`：独立
结构化确认、疲劳聚合、禁止一揽子批准）。超出层级检查的 R0/R2/R3 结构化流程未实现——
故 `partial`。

## 动词

- `inspect_with_audit`：特权读取只有在 canonical-JSON-lines 审计记录持久追加后才经
  `ResultReleaseGate` 释放（`FileManagementAuditLog` 强制序列/epoch/哈希链形状）。
- `stop`：fence 写者 epoch、取消调度工作、对在途 Effect 分类（保守地把
  `RECONCILED/VERIFIED/VERIFY_FAILED` 计为 pending）并报告剩余。
- `revoke`：追加能力撤销并推进撤销 epoch，context/授权现时性检查消费该 epoch。
- `reconcile`：驱动 kernel 恢复序列；未配置执行器时，仍未知结果隔离（fail-safe）而
  非强行了结。

## Agent 生命周期动词

`admin-cli install/register/activate/activate-root/rollback/agent-pause/agent-resume/agent-stop/
agent-recover/agent-health/uninstall` 调用
[Agent 与 Pi 生命周期](./agent-and-pi-lifecycle.md)所述运行时生命周期，全部会话把
门。

## Task 准入

`KernelTaskApplicationService::admit` 仍是确定性的 digest/权威/epoch 门。其生产
铸造现在只有在同一权威事务已发布 TaskContract、合同命名的 `START` Loop、硬
Budget 与当前 epoch 的 runnable 调度行后才返回成功。这只是调度引导：不创建 worker
Intent/Effect，不执行 Tool I/O，也不能完成 Task。

诚实缺口：usage 文本漏写 official install 的 `--package-id`；治理台账
（`revocation_epoch`/`capability_set_version` 持久化）有文件实现并被测试消费，
daemon 侧接线仍为 partial。
