---
doc_id: user.cli-basics
locale: zh-CN
kind: guide
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["parse_cognitive_args", "COGNITIVE_USAGE"]
  - path: apps/admin-cli/src/personal_cli/daemon.rs
tests:
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - apps/admin-cli/tests/p2_t02_cli_parity.rs
fingerprint: "sha256:1a6ec60d8017f7702296d27c726cd5c50cd8a2772f02b6dcd262ed05494eb9e2"
non_claims:
  - CLI 是非权威客户端；它打印的任何内容都不意味着 Task 完成或 Gate 结果。
---

# CLI 基础

`cognitive` 二进制是确定性的产品入口。它从不直接写权威状态——只准备配置、启动/停止
daemon 进程、读取已认证投影。退出码：`0` 成功、`1` 运行错误、`2` 用法错误；成功输出
为 JSON。

| 动词 | 实际行为 |
|---|---|
| `cognitive init` | 准备 XDG 布局与数据库（带备份），把 Provider key 存入 Secret Service，探测 Provider，持久化 `provider.json` 与 `selected-model.json` |
| `cognitive status` | 已认证的组件投影（system、database、secret、provider、daemon、pi） |
| `cognitive doctor` | 同一投影外加脱敏诊断小节 |
| `cognitive daemon start` | 启动绑定 `127.0.0.1:48181` 的 `kernel-server --personal`（可用 `--bind`、`--kernel-server` 或 `COGNITIVE_KERNEL_SERVER` 覆盖） |
| `cognitive daemon status` | 报告 daemon 锁/endpoint 存活状态 |
| `cognitive daemon stop` | 向记录的 PID 发信号；确认退出后才移除锁与 endpoint |
| `cognitive pi configure` | 写非 secret 的 `pi.json`（可执行文件与扩展入口的绝对路径） |
| `cognitive pi launch` | doctor 全就绪且版本精确匹配后 fail-closed 启动 Pi |
| `cognitive resource get/watch --family <memory\|skill\|tool\|context\|task\|runtime>` | 读取私有六族投影（management 通道） |
| `cognitive task watch [--resume-from N]` | 跟随有界 Task watch 流（task 通道） |

两个诚实的怪癖（生成的 [CLI 参考](../reference/cli-cognitive.md)中同样标注）：内置
usage 文本尚未列出 `resource`/`task`；所有动词都接受的 `--runtime-root <dir>` 是密封
测试逃生口，会整体搬迁布局。

独立的 `admin-cli` 二进制是管理回退入口（inspect / stop / revoke / reconcile 及
agent 生命周期动词），需要特权会话文档；见 [admin-cli 参考](../reference/cli-admin.md)
与[管理面](../developer/management-plane.md)页面。
