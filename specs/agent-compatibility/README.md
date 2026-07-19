# CognitiveOS Agent Compatibility Companion Specification

> 版本：v0.1 Draft  
> 标识：`cognitiveos.agent-compatibility/0.1`  
> 范围：Agent package、安装事务、adapter、sandbox、兼容声明与不可绕过性

## 1. 规范约定与边界
大写 **MUST**、**MUST NOT**、**SHOULD**、**MAY** 按 RFC 2119/8174。此 Profile 不授予运行权限；C0—C3 是可观察/适配能力，不是 R0—R3 安全等级。

## 2. Package 与 Installation
`AgentPackageManifest` 声明 publisher/version/artifact digest/signature/provenance、CognitiveOS/AKP 版本、workload identity、Runtime/model、memory、tool/network/filesystem/secret/device、sandbox、checkpoint/recovery、remote hidden state 与 retention。声明不是证据或 capability。

安装状态为 `SUBMITTED → VERIFIED → ANALYZED → ADAPTED → TESTED → ADMITTED → COMMITTED`，任一步可 `REJECTED|QUARANTINED`。`AgentInstallation` 固定 package、adapter、sandbox、compatibility report、test evidence、profile ceiling、degradations 与 rollback point。upgrade 创建新版本；remove 是 governed Effect，保留 pending Effect、audit、retention/legal hold。

[REQ-AGENT-INSTALL-001] 安装 **MUST** 验证 artifact digest、signature/provenance、manifest/schema、adapter/sandbox digest 与 compatibility evidence，并经 management authority commit。

[REQ-AGENT-INSTALL-002] 安装 capability 与 Task/runtime capability **MUST** 分离；安装成功 **MUST NOT** 自动签发高风险运行能力。

## 3. Compatibility Profiles
- C0 Contained Legacy：沙箱包含；内部长期状态不可信；默认只声明 R0。
- C1 Governed I/O：identity、Conversation、memory/knowledge、tool、filesystem/network/secret 全部被中介。
- C2 Lifecycle-aware：增加 TaskContract、ActivityContext、delta/fault、cancel、checkpoint/resume、pending Effect、unknown outcome、verification 与 candidate completion。
- C3 Native：原生 AKP/Core 对象和系统调用。

[REQ-AGENT-COMPAT-001] Compatibility Report **MUST** 逐特征记录 supported/unsupported/degraded、测试引用与不可观察 hidden state；**MUST NOT** 仅凭 C 标签推导风险许可。

## 4. Adapter Interface Family
Identity Adapter 不信任 Agent 自报 user/role；Memory Adapter 把 search/get/add/update/delete 映射为 resolve/expand/propose/admit/invalidate/tombstone；Tool Adapter 把 list/call 映射为 discover/describe/bind 与 Intent/Effect；Completion Adapter 只产生 `CANDIDATE_COMPLETE`；Checkpoint Adapter 保存行动级事实和 pending Effect；Sandbox Adapter 中介 filesystem/network/subprocess/secrets/device/model/MCP/A2A/IPC。

[REQ-AGENT-ADAPTER-001] C1+ 的所有已声明跨边界 I/O **MUST** 经过 adapter 和本地 authorization，且 adapter failure **MUST** fail closed 或降低兼容声明。

[REQ-AGENT-COMPLETE-001] Agent 的 done/success/completed/final_answer **MUST NOT** 直接推进 Task 到 `COMPLETED`。

## 5. 不可绕过性与恢复
必须测试未声明网络、宿主 secret、tool proxy、未登记 MCP、其他 Conversation cache、撤销 capability、旧 epoch、伪 capability 文本、Conversation KV 隔离、timeout 幂等与恢复前 Effect 对账。

[REQ-AGENT-SANDBOX-001] 安装声称拦截的边界 **MUST** 有负面测试证据；绕过或不可观察路径 **MUST** 降级 Profile 并限制 capability。

[REQ-AGENT-RECOVERY-001] C2+ 恢复 **MUST** fence 旧执行者、reconcile pending Effect、重验治理绑定并重建 Context 后才恢复 Loop。

## 6. 符合性
声明固定 package/adapter/sandbox/schema/suite digest、C0—C3 feature matrix、最高已验证 R 等级、degradation 与 rollback evidence。C0—C3 和 R0—R3 必须分别报告。

机器 schema：[agent-package-manifest.schema.json](../schemas/agent-package-manifest.schema.json)、[agent-installation.schema.json](../schemas/agent-installation.schema.json)、[agent-compatibility-report.schema.json](../schemas/agent-compatibility-report.schema.json)。


## Shell 与用户旅程映射
Agent adapter 暴露的 start/pause/resume/cancel/checkpoint/status 必须映射到 Core lifecycle；宿主 PID 终止不能伪报 Task 完成。

[REQ-AGENT-SHELL-001] C2+ adapter **MUST** 保留 Shell 控制中的 Task、AgentExecution、Runtime 与 Effect 状态区分，attach/disconnect 不恢复隐藏 authority。
