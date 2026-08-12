---
doc_id: user.pi-shell
locale: zh-CN
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: packages/pi-cognitiveos/src/extension.ts
    symbols: ["registerCognitiveOsExtension"]
  - path: packages/pi-cognitiveos/src/daemon-provider.ts
  - path: packages/pi-cognitiveos/src/tool-policy.ts
  - path: apps/admin-cli/src/personal_cli/pi.rs
tests:
  - packages/pi-cognitiveos/src/extension.test.ts
  - packages/pi-cognitiveos/src/daemon-provider.test.ts
  - packages/pi-cognitiveos/src/safety.test.ts
fingerprint: "sha256:30bf86c0626cfa1b6dca27afa73cb3725fda1682ec19ff930a1621e01c8973e6"
non_claims:
  - Pi 始终是只产 candidate 的客户端；shell 中任何行为都不能推进权威状态，也不声明对话质量/收益。
---

# Pi 对话壳

`partial`：经 daemon 代理的对话、readiness 展示与状态命令已实现；agent 工具使用与资
源/任务浏览面有意尚未在 shell 中开放。

## 今天能用什么

通过 `cognitive pi launch` 启动 Pi。CognitiveOS 扩展随即：

- 经 `daemon-endpoint.json` 发现 daemon，用每次启动生成的 bootstrap secret 认证
  （management 与 task bearer 分开持有）；
- 注册 `cognitiveos` 模型 provider：你的输入经 Pi → daemon Provider 代理 → 你的
  Provider。Pi 进程永远看不到 API key；
- 会话开始时展示 daemon readiness，首次对话被阻塞时给出警告；
- `/cognitive-status` 命令只回答 daemon 事实。

响应为单发：daemon 请求非流式补全，扩展将其作为单块输出（仅文本；图像/工具调用被拒
绝）。

## 有意锁死的部分

- `project_trust` 恒拒绝，且工具策略拒绝**所有** Pi 内置工具（含只读工具）——shell
  无法触碰你的文件或执行命令。
- Pi 内尚无资源浏览、任务提交或 watch UI：这些客户端方法存在于
  `PersonalDaemonClient` 与 CLI（`cognitive resource|task`），但未接入 shell UX。
- 模型参数由 daemon 的 selected model 固定；Pi 中的用量/费用显示为零（客户端不计量）。

## Pi 的另一重角色

在 shell 之外，daemon 还把 Pi 作为受治理 agent 管理（获取、注册、sidecar 会话），并
能启动一个高度受限的 Pi 子进程经一次性私有 socket 产出 **candidate**——该路径禁用工
具、skill、会话与除钉住 candidate 扩展外的一切扩展。见
[Agent 与 Pi 生命周期](../developer/agent-and-pi-lifecycle.md)。
