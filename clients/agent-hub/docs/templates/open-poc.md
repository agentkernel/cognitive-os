# 模板 — Open PoC

> 复制到 [../traceability/evidence-index.md](../traceability/evidence-index.md)。ID 使用 `CONSOLE-AGENTHUB-V1-POC-*`。Open PoC 是实现前必须用真实 API/真实 OS 行为验证的断言，未验证前保持 `not-run / none`。

## `CONSOLE-AGENTHUB-V1-POC-XXX` <断言标题>

- 领域：进程 / 终端 / session / 文件 / 安全 / Relay / 电脑控制 / 协作 / 法务。
- 断言（要证明为真或为假的具体命题）：
- 方法（真实环境、目标版本、隔离要求；不得用 mock 冒充）：
- 通过标准（oracle）：
- 安全负例（必须一并验证不被绕过）：
- 依赖（前置 PoC / gate）：
- 状态：not-run。
- 证据路径（执行后填 `artifacts/…`，gitignore）：none。

## 规则

- PoC 通过前，任何文档不得声明对应能力 implemented 或 Profile 已符合。
- 安全负例不可豁免。
