---
doc_id: dev.clients-ts
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: packages/sdk-ts/src/client.ts
    symbols: ["TaskChannelClient", "ManagementChannelClient"]
  - path: packages/sdk-ts/src/channel.ts
  - path: packages/sdk-ts/src/watch.ts
  - path: packages/pi-cognitiveos/src/daemon-client.ts
    symbols: ["PersonalDaemonClient"]
  - path: apps/agent-shell/src/session.ts
    symbols: ["ShellSession"]
tests:
  - packages/sdk-ts/src/client.test.ts
  - packages/pi-cognitiveos/src/daemon-client.test.ts
  - apps/agent-shell/src/session.test.ts
fingerprint: "sha256:f43c186f4add0d27d98de529de9be44b526aa1cc8ab23b3b9313038b33ceaa79"
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
`PI_EXTENSION_*` 错误。扩展注册的 provider 桥与工具策略见
[Pi 对话壳](../user/pi-shell.md)。

## `apps/agent-shell` —— 会话库

`ShellSession` 以显式状态机驱动 preview → submit（admit）→ attach/cancel，支持断连
缓冲重放与幂等提交（同 preview digest ⇒ 同 task）。它是带测试的库，不是随发 TUI。

共享不变量：任何 secret 材料不达这些层（bearer 是进程本地会话令牌，非 Provider
key）；每个变更形调用都带幂等键；所有 list/watch 表面有界；JSON 解析经
`packages/contracts-ts` 生成类型成形，并与 Rust 保持 canonical digest 奇偶
（`tests/golden/`）。
