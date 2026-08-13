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
  - path: packages/pi-cognitiveos/src/pi-route-observation.ts
  - path: packages/pi-cognitiveos/src/tool-policy.ts
  - path: apps/admin-cli/src/personal_cli/pi.rs
tests:
  - packages/pi-cognitiveos/src/extension.test.ts
  - packages/pi-cognitiveos/src/daemon-provider.test.ts
  - packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - packages/pi-cognitiveos/src/safety.test.ts
fingerprint: "sha256:101496c330fb9fe846099b4f53bf8599d4a24715180cdc42717faaed358cdd11"
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
- 模型参数由 daemon 的 selected model 固定。只有 Provider 返回完整且内部一致的计数
  时才显示 token 用量，否则保持不可用而不做估算；费用永不显示，因为 shell 没有绑定
  任何计价来源。

## Campaign 测量默认关闭

普通会话不做任何测量。启动 Pi 前同时设置
`COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` 与
`COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN=<campaign id>`。daemon 进程也必须看到
同一个启用变量，否则两个嵌套 daemon 阶段保持 `not_available` 而不会被 join。每次
请求会额外发布一条
内存中的观测：路由的七个阶段（请求准备、扩展派发、loopback 等待、daemon preflight、
Provider 网络、响应解析、事件投递）以单调时长记录，并由一个不透明 correlation id 与
daemon 侧连接，同时带上上文所述的 Provider 用量。

一条观测只含时长与计数——绝不含 prompt、响应、header、bearer 或 Provider key——且
shell 不为它向磁盘写入任何内容。`COGNITIVEOS_PI_ROUTE_OBSERVATION_SINK` 可为嵌入该扩展
的 campaign harness 指定一个绝对 `.ndjson` 路径；shell 自身绝不打开它，且位于
CognitiveOS state/runtime/config 目录内的路径一律拒绝。阶段计时只是测量，不是性能结论：
不支持任何收益、Gate、release 或 Profile 声明。

## Pi 的另一重角色

在 shell 之外，daemon 还把 Pi 作为受治理 agent 管理（获取、注册、sidecar 会话），并
能启动一个高度受限的 Pi 子进程经一次性私有 socket 产出 **candidate**——该路径禁用工
具、skill、会话与除钉住 candidate 扩展外的一切扩展。见
[Agent 与 Pi 生命周期](../developer/agent-and-pi-lifecycle.md)。
