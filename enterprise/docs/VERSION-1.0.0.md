# cognitiveos-enterprise 1.0.0 — 边界定义与验收标准（未开始）

- Status: **defined / not-started** — 仅定义边界与验收标准；无任何实现、
  测试、Gate、release 或 Profile 声明
- Date: 2026-08-25
- Decision anchor: [ADR-0054](../../docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
- Design sources（candidate，非 canonical）：本目录 `design/` 下的企业产品
  设计、交互规范与架构文档

## 1. 一句话定义

`cognitiveos-enterprise` 1.0.0 是**中央治理平面**：面向组织的身份/组织
结构、策略分发、节点编队（fleet）、Knowledge index 与证据投影服务。它治理
运行 `cognitiveos-personal` 的节点，但**永不直接写入任何节点的本地
authority store**——中央服务只能发送签名请求，节点 daemon 重新授权并保持
唯一本地 writer（公理 A1 的分布式延伸）。

## 2. 1.0.0 候选范围（boundary）

| 能力域 | 内容 | 边界 |
|---|---|---|
| 组织与身份 | 组织/成员/角色目录，节点注册与所属关系 | 身份是 overlay SoR；节点本地主体不被替换 |
| 策略分发 | 版本化、签名、带 digest 与有效期的 policy bundle 分发 | 节点侧 fail-closed 验签与本地重授权；`latest` 禁止 |
| 节点编队 | 节点清单、健康/版本/兼容窗口投影、受控升级建议 | 只读投影 + 请求；无远程强制写 |
| Knowledge index | 多租户知识索引与检索，ACL 与 purge | index 可重建；来源/ACL/policy 的 SoR 分离 |
| 证据投影 | 节点证据的最小化引用/投影（digest 优先） | 原始本地证据 SoR 留在节点 |
| 客户端 | Enterprise 管理 UI 经 `clients/` 消费版本化治理 API | 客户端永不拥有 authority |

**明确不在 1.0.0：** 替代节点 daemon 的中央执行器、远程节点 DB 写入、
跨组织联邦、市场/计费、Personal 未提供的任何能力假设。

## 3. 1.0.0 验收标准（预登记）

实现启动后，1.0.0 须全部满足：

1. **签名请求 → 节点重授权端到端**：中央请求在节点侧经验签、策略校验与
   本地 Intent/Effect 全链路后才生效；篡改/过期/未知签名者 fail closed，
   有 focused negatives。
2. **策略分发正确性**：policy bundle 的版本、digest、有效期、回滚与撤销
   均可机器验证；未知 major/未知 critical extension fail closed。
3. **编队投影真实性**：节点清单与健康/版本投影只来自节点上报事实；无
   推断状态；投影延迟与陈旧度显式标注。
4. **Knowledge 多租户隔离**：跨租户读写隔离、ACL 强制与 purge 可验证，
   含租户越权、purge 后残留、index 重建等 negatives。
5. **证据最小化**：证据投影仅含 digest/引用与脱敏事实；secret 与原始
   payload 永不入中央存储。
6. **自身符合性**：作为独立 IUT 通过适用的 core conformance 子集；拥有
   独立部署（自身 CI、安装/升级/回滚）证据。
7. **对 Personal 零侵入**：Enterprise 的存在不改变 Personal 单机语义；
   未接入 Enterprise 的 Personal 节点行为不变（负例验证）。

## 4. 激活门槛（activation gate）

在以下条件全部满足并由 owner 明确授权前，`enterprise/` 只承载设计文档，
禁止启动实现、登记实现任务或建立第二产品身份：

1. owner 正式激活 Enterprise 为实现项目（书面授权 + 正式计划登记）；
2. 至少一个真实目标组织/设计伙伴验证 JTBD 与架构假设；
3. 独立的部署、安全 ownership 与预算边界成立；
4. 所需 node protocol / policy / evidence 合同在 core 侧经 Lane-CTR 稳定。

## 5. 与其他子项目的关系

- 依赖方向：`core → enterprise`（合同消费）；`enterprise ⇢ personal` 仅为
  签名请求/投影，永不 import Personal store 实现；UI 一律经 `clients/`。
- 版本轴独立：Enterprise 使用自己的产品 SemVer，不与 core/personal 版本
  强制对齐；兼容窗口在实现启动后另行登记。
