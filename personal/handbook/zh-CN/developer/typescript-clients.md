---
doc_id: dev.clients-ts
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/packages/sdk-ts/src/client.ts
    symbols: ["TaskChannelClient", "ManagementChannelClient"]
  - path: personal/packages/sdk-ts/src/channel.ts
  - path: personal/packages/sdk-ts/src/watch.ts
  - path: personal/packages/pi-cognitiveos/src/daemon-client.ts
    symbols: ["PersonalDaemonClient"]
  - path: personal/packages/pi-cognitiveos/src/pi-route-observation.ts
    symbols: ["assemblePiRouteObservation"]
  - path: personal/apps/agent-shell/src/session.ts
    symbols: ["ShellSession"]
tests:
  - personal/packages/sdk-ts/src/client.test.ts
  - personal/packages/pi-cognitiveos/src/daemon-client.test.ts
  - personal/packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - personal/apps/agent-shell/src/session.test.ts
fingerprint: "sha256:577d07f7dd3c2a0301e584fa0763fcb9134ada885e0441df45c1b74225ec5729"
non_claims:
  - 全部 TypeScript 表面都是 candidate/observation 客户端；任何一个都不能持有权威或完成 Task。
---

# TypeScript 客户端

三层客户端，全部严格非权威：

## `packages/sdk-ts` —— AKP 客户端 SDK

基于生成合同类型的通道隔离 `TaskChannelClient`/`ManagementChannelClient`：请求信封携带协议 pin、幂等键、
canonical digest；响应把注册错误码映射为类型化错误。`channel.ts` 在类型层与运行时同
时阻止 task 通道客户端发起管理调用。`watch.ts` 实现有界、快照先行的 watch 消费器，
带 resume 游标与缺口检测（`RESUME_STALE` 处理与 daemon 镜像）。传输：测试用内存
fake 加 loopback HTTP。

## `packages/pi-cognitiveos` —— Pi 扩展客户端

`PersonalDaemonClient` 负责发现（`daemon-endpoint.json` + bootstrap secret）、分离
的 management/task 会话铸造、health/status/doctor 读取、provider chat completion、
资源投影/watch 与 task watch——均带有界超时/大小与类型化 `PERSONAL_*`/
`PI_EXTENSION_*` 错误。每次 completion 派发附带不透明的 `campaign-…` 关联 id 头，并
报告实测 loopback 耗时、daemon 上报的嵌套耗时与真实 token 用量——或
`not_available`；绝不伪造零值。

在显式 campaign 授权下，同一次派发还会发布一条 `personal-pi-route-observation/1`
记录：五个由「同一时刻只能打开一个阶段」的记录器产出的 Pi 域顺序阶段，加上嵌套在
loopback 等待内、由回显 correlation id 连接的两个 daemon 域阶段。daemon 仅在自身环境
也设置了 `COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` 时才回显该 id 并报告 preflight，
否则嵌套一对降级为 `not_available`。两个时钟域之间绝不
相加或相减，跨域只断言包含关系。未上报、未回显、不匹配、只报一半，或大于包含它的等待
时长的 daemon 阶段，一律带原因丢弃，而不是裁剪或估算。插桩默认拒绝，不持有文件系统或
权威面（持久 sink 是注入端口，指向 Personal 根内的 sink 一律拒绝），发布内容只有标签、
不透明 id、时长与计数。扩展注册的 provider 桥与工具策略见
[Pi 对话壳](../user/pi-shell.md)。

记录还携带 `requestMode`、`outcome`、`terminalStage` 与固定的无内容
`failureClass`。成功请求必须具备全部五个 Pi 阶段；取消/错误请求只保留实际测得的精确
前缀。Provider 路径固定非流式（`stream:false`）；`stream:true` 在解析 secret 前即以
稳定错误拒绝。实测 usage 带有仅由已认证 daemon 响应解析器创建的进程内来源标记，因此
嵌入式 runner 不能发布自行断言的计数。这阻止 instrumentation 侧伪造，但不对上游
Provider 的计数作密码学背书。

## `apps/agent-shell` —— 会话库

`ShellSession` 以显式状态机驱动 preview → submit（admit）→ attach/cancel，支持断连
缓冲重放与幂等提交（同 preview digest ⇒ 同 task）。它是带测试的库，不是随发 TUI。

共享不变量：任何 secret 材料不达这些层（bearer 是进程本地会话令牌，非 Provider
key）；每个变更形调用都带幂等键；所有 list/watch 表面有界；JSON 解析经
`packages/contracts-ts` 生成类型成形，并与 Rust 保持 canonical digest 奇偶
（`core/tests/golden/`）。
