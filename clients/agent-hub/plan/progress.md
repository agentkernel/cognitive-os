# Agent Hub 开发进度（局部）

> 类别：plan（informative）｜ 日期：2026-07-20（2026-07-21 Phase 0 注记）｜ owner：Lane-CON
>
> 只跟踪 Agent Hub 计划/文档进度；工程里程碑真相以全局 [../PROGRESS.md](../../../docs/plan/PROGRESS.md) 为准。不在此声明实现/测试/Profile 状态。

## 车道状态

| 车道 | 计划文档 | 实现状态 | 文档级进展（2026-07-21） |
|---|---|---|---|
| GOV 产品治理 | done（计划草案） | not-implemented / blocked | 威胁口径纠正；POC-GOV-001 planned |
| CTR 合同与能力协商 | done | not-implemented / blocked | **AH-CTR-02 文档级完成**（ledger+dossiers）；evidence not-run |
| HOST Host/Control/Ledger | done | not-implemented / blocked | POC-HOST-001 planned |
| PROC Process+Terminal | done | not-implemented / blocked | POC-TERM-003 / POC-PROC-005 planned |
| SESS Session+File | done | not-implemented / blocked | — |
| CRED Credential+Workspace+Verifier | done | not-implemented / blocked | — |
| RELAY Relay/Pairing | done | not-implemented / blocked | POC-RELAY-005 planned |
| DESK Desktop | done | not-implemented / blocked | — |
| IOS iPhone | done | not-implemented / blocked | Apple PLA 外部阻断已登记 |
| AND Android | done | not-implemented / blocked | — |
| MULTI Multi-Agent | done | not-implemented / blocked | — |
| QRM Quality/Release/Migration | done | not-implemented / blocked | — |
| 6× Tier 1 Adapter | done | not-implemented / blocked | 接口文档级已核验；实现仍 blocked |

## 证据状态（固定）

- implementation：not-implemented。
- Open PoC：28× not-run + 5× planned（见 [../docs/traceability/evidence-index.md](../docs/traceability/evidence-index.md)）；**零 pass**。
- POC-LIC-001..003：not-run（评估材料已整理，法务评估未执行）。
- 既有 conformance vectors：84（全局 46 pass / 38 not-run）；Agent Hub 平台证据仍为 none。
- Direct / Governed Profile：not implemented。
- 21 项威胁：已规范登记；oracle/evidence 全 not-run（非「实测」）。

## AH-CTR-02 / AH-M1 指针

- AH-CTR-02：文档级交付见 [lane-contract-capability.md](./lane-contract-capability.md)；canonical 事实见 provider-interfaces-ledger + tier1 dossiers。
- AH-M1（计划里程碑）：仍依赖后端/ADR/PoC/法务 gate——**未解阻**；本回合仅 informative 文档。

## 更新规则

- 计划语义变化在此更新一行；里程碑 GO/NO-GO 属全局 PROGRESS 与 milestone review，不在此声明。
