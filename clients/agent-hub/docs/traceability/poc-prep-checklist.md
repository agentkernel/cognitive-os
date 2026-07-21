# Agent Hub Open PoC — 准备清单（非证据）

> 类别：informative prep checklist ｜ owner：Lane-CON ｜ 日期：2026-07-21
>
> **不是执行证据。** 全部 Open PoC 仍为 `not-run` 或 `planned`，evidence `none`。本清单只登记执行前准备项。

## 1. 既有指针（勿复制正文）

| 资产 | 路径 |
|---|---|
| 条目模板 | [../templates/open-poc.md](../templates/open-poc.md) |
| 证据索引（canonical） | [evidence-index.md](evidence-index.md) |
| 计划侧索引 | [../../plan/evidence-index.md](../../plan/evidence-index.md) |
| 威胁 oracle 设计 | [../security/threat-test-oracles.md](../security/threat-test-oracles.md) |
| 共享执行记录模板 | [../../../shared/docs/poc-execution-record.md](../../../shared/docs/poc-execution-record.md) |
| 法务材料 | [../legal/licensing-and-terms.md](../legal/licensing-and-terms.md)（POC-LIC 评估仍 not-run） |

## 2. 准备清单

| # | 准备项 | 状态 | 备注 |
|---|---|---|---|
| 1 | 后端依赖组 1/2/7 + M5 出口允许的 PoC 边界确认 | blocked | 见 [READINESS](../../../READINESS.md) |
| 2 | 目标 OS 矩阵（Windows/Linux Host）与隔离沙箱可用 | not-ready | 禁用 mock 冒充进程/终端/socket |
| 3 | Tier 1 provider 可安装版本钉扎与 AH-CTR-02 接口文档对照 | partial | 接口文档级已回填；runtime Adapter 仍 blocked |
| 4 | POC-LIC-001..003 法务评估完成并留证 | not-run | 材料已整理；评估未执行 |
| 5 | 签名/开发者账号（若触及分发或公证路径） | external-blocked | 见 risk-register AH-EXT-* |
| 6 | artifacts 目录约定与 digest 工具链 | planned | `artifacts/evidence/clients/agent-hub/<poc-id>/`（gitignore） |
| 7 | 安全负例 corpus 与 oracle（威胁表）可定位 | designed | oracle 设计有；执行 none |
| 8 | 执行记录：每 PoC 复制 open-poc 模板 + 共享环境指纹字段 | template-ready | 零填写实例 |

## 3. 计数提醒

- 登记：28 `not-run` + 5 `planned` = 33 项；**执行证据仍为 none**。
- 准备清单勾选 ≠ PoC pass；不得据此改写 implementation-ready。
