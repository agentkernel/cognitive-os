# clients/ 迁移对照表（MIGRATION-MAP）

> 类别：informative migration map ｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 迁移决策：`CLIENTS-DEC-001`（[governance/decision-log.md](governance/decision-log.md)）；批准记录：用户 2026-07-20 批准文件级迁移方案。

本文件是客户端文档迁移的唯一 old→new 对照真相：逐文件映射、anchor/ID 保全方案、兼容 stub 清单、批次提交哈希与"不迁移"清单。执行原则：`git mv` 保留历史；每批独立提交并绿 `check:consistency` + `git diff --check` + clients 手动链接检查。

## 1. 批次与提交哈希

| 批次 | 内容 | 提交哈希 |
|---|---|---|
| P0-1 | Agent Hub canonical 文档 + Master/车道计划 + 提示词落地（迁移前封口） | `03250e1b3e3249d2cc600346f2861886a351a079` |
| P0-2 | Lane-CON 仓库联动 + D-012 计数修正（迁移前封口） | `8e1de44150c3ded0a7561fd69a7cbf1f7df89ad4` |
| P0-3 | agent-hub 设计 handoff（哈希回填）+ 目录索引会话提示词 | `dc55eebcec95916b6bc0971857a628aeb21f173e` |
| B1 | clients/ 根、治理件、readiness、迁移表、薄 README 骨架 | `dedd082a3a8b3271fd0c106727c4f865d278b0e2` |
| B2 | PC 产品/平台文档迁移（13 文件） | `41609ce712412ae45a1074743fc94d0359eef09e` |
| B3 | mobile shared/iOS/Android 文档迁移（4 文件） | `7591fe8ae837530c0c9e6d3f8ec82193946104bd` |
| B4 | shared 新文档（测试策略、遥测/脱敏/留存政策） | `8afce717816a075e3639b57ac331d72f1fcc5aeb` |
| B5 | Agent Hub docs/plan/prompts 迁移（86 文件） | `85331bb21391abe4e6b7199f9dd717e2d06c7f92` |
| B6 | 旧路径 stub 定稿、Console 实现 gate canonical 迁移、全仓链接修复 | `b2c1f63205ed209234fa8f1fa79a39f0612c9210` |
| B7 | Cursor rules 与治理入口联动、ADR-0007 | `5902a252b5e4b5d80239e353e72f8e50930554e9` |
| B8 | readiness review、PROGRESS、handoff、pc/docs/architecture 薄 README 补齐 | 本提交（见 handoff） |

## 2. old → new 逐文件映射

状态列：`done`（已迁移）/ `pending-B<N>`（排定批次）/ `stub`（旧位置保留兼容入口）。

### 2.1 B1 目录索引

| old | new | 批次 | 状态 |
|---|---|---|---|
| `docs/clients/README.md` | `clients/README.md`（git mv + 全面改写为项目地图） | B1 | done；旧位置留 stub |

### 2.2 B2 PC 文档（13 文件）

| old | new | 批次 | 状态 |
|---|---|---|---|
| `apps/cognitiveos-console/docs/product-brief.md` | `clients/pc/docs/product/product-brief.md` | B2 | done |
| `apps/cognitiveos-console/docs/decision-log.md` | `clients/pc/docs/product/decision-log.md` | B2 | done |
| `apps/cognitiveos-console/docs/requirements-traceability.md` | `clients/pc/docs/product/requirements-traceability.md` | B2 | done |
| `apps/cognitiveos-console/docs/information-architecture.md` | `clients/pc/docs/ux/information-architecture.md` | B2 | done |
| `apps/cognitiveos-console/docs/journeys-and-screens.md` | `clients/pc/docs/ux/journeys-and-screens.md` | B2 | done |
| `apps/cognitiveos-console/docs/design-system.md` | `clients/pc/docs/ux/design-system.md` | B2 | done |
| `apps/cognitiveos-console/docs/trust-safety-ux.md` | `clients/pc/docs/security/trust-safety-ux.md` | B2 | done |
| `apps/cognitiveos-console/docs/windows-v1-scope.md` | `clients/pc/docs/platforms/windows/windows-v1-scope.md` | B2 | done |
| `apps/cognitiveos-console/docs/roadmap.md` | `clients/pc/plan/roadmap.md` | B2 | done |
| `docs/platforms/macos-product-design.md` | `clients/pc/docs/platforms/macos/macos-product-design.md` | B2 | done |
| `docs/platforms/linux-product-design.md` | `clients/pc/docs/platforms/linux/linux-product-design.md` | B2 | done |
| `docs/platforms/desktop-parity-matrix.md` | `clients/pc/docs/platforms/desktop-parity-matrix.md` | B2 | done |
| `docs/platforms/platform-decision-log.md` | `clients/pc/docs/platforms/platform-decision-log.md` | B2 | done |

### 2.3 B3 mobile 文档（4 文件）

| old | new | 批次 | 状态 |
|---|---|---|---|
| `docs/platforms/ios-product-design.md` | `clients/mobile/ios/docs/ios-product-design.md` | B3 | done |
| `docs/platforms/android-product-design.md` | `clients/mobile/android/docs/android-product-design.md` | B3 | done |
| `docs/platforms/mobile-parity-matrix.md` | `clients/mobile/shared/docs/mobile-parity-matrix.md` | B3 | done |
| `docs/platforms/mobile-platform-decision-log.md` | `clients/mobile/shared/docs/mobile-platform-decision-log.md` | B3 | done |

### 2.4 B4 shared 新文档（新建，非迁移）

| old | new | 批次 | 状态 |
|---|---|---|---|
| —（缺口新建） | `clients/shared/docs/test-strategy.md` | B4 | done |
| —（缺口新建） | `clients/shared/docs/telemetry-evidence/telemetry-redaction-retention-policy.md` | B4 | done |

### 2.5 B5 Agent Hub（86 文件）

| old | new | 批次 | 状态 |
|---|---|---|---|
| `apps/cognitiveos-console/docs/agent-hub/**`（41 文件，14 子目录结构原样） | `clients/agent-hub/docs/**` | B5 | done |
| `docs/plan/agent-hub-development-plan.md` | `clients/agent-hub/plan/agent-hub-development-plan.md` | B5 | done |
| `docs/plan/agent-hub/*`（24 文件：README/progress/milestones/dependency-dag/evidence-index/risk-register + 12 宏车道 + 6 Adapter） | `clients/agent-hub/plan/*` | B5 | done |
| `docs/prompts/agent-hub/*`（19 文件：README + 12 宏车道 + 6 Adapter） | `clients/agent-hub/prompts/*` | B5 | done |
| `docs/platforms/agent-hub-platform-parity.md` | `clients/agent-hub/docs/platforms/agent-hub-platform-parity.md` | B5 | done |

### 2.6 B6 gate canonical 迁移

| old | new | 批次 | 状态 |
|---|---|---|---|
| `docs/platforms/README.md` 的"Console 实现 gate"正文（含四条 gate 与激活规则） | `clients/governance/readiness-gates.md`（成为 canonical） | B6 | done；旧文件已压成 stub 并保留 anchor |

## 3. anchor / ID 保全方案

- 显式 `<a id>` 保全：`cognitiveos-client-directory-index`（clients/README.md 与 docs/clients stub 同时保留）；`cognitiveos-console-桌面平台产品设计`、`implementation-gate`（docs/platforms/README.md stub 原样保留，heading `## Console 实现 gate` 一字不差）；PRODUCT-DESIGN 的 `doc-top`、`sec-1`~`sec-20`、`appendix-a/b` 全部 23 个 anchor 原样保留在兼容 stub。
- readiness-gates.md 在 B6 增加显式 `<a id="console-实现-gate"></a>` 与 `<a id="implementation-gate"></a>` 别名，使新位置可被旧式锚点样式引用。
- heading 锚点保全：`docs/platforms/mobile-platform-decision-log.md` 的 `#console-ios-v1-dec-*` / `#console-and-v1-dec-*`（32 个被显式引用）与 `platform-decision-log.md` 的 22 个 heading 锚点随文件迁移后由引用方改指新路径，heading 文字不动。
- 产品 ID 不重编号、不重用、不删除：`CONSOLE-V2-*`、`CONSOLE-{MAC,LNX}-V1-*`、`{MAC,LNX,IOS}-POC-*`、`POC-*`（Android/Agent Hub）、`IOS/AND-TM-*`、`CONSOLE-IOS/AND-V1-*`、`CONSOLE-AGENTHUB-V1-*`、`AH-*`；总表见 [governance/traceability.md](governance/traceability.md)。
- 新结构决策只允许新增 `CLIENTS-DEC-*` 命名空间。

## 4. 兼容 stub 清单（4 个）

| 旧路径 | stub 义务 | 批次 |
|---|---|---|
| `docs/clients/README.md` | deprecated + successor 链接；保留 `<a id="cognitiveos-client-directory-index"></a>` 与 heading `## 9. 持续维护与手动 gate`；不复制表格正文 | B1（已生效） |
| `apps/cognitiveos-console/README.md` | 薄兼容入口；doc map 链接改指 clients/ 新路径 | B2 改链，B6 定稿 |
| `apps/cognitiveos-console/PRODUCT-DESIGN.md` | v2 兼容入口；23 个 anchor 与漂移登记表原样保留；doc map 改指新路径 | B2 改链，B6 定稿 |
| `docs/platforms/README.md` | deprecated + successor；保留首行 `<a id>`、`<a id="implementation-gate"></a>`、heading `## Console 实现 gate`；gate 正文不复制 | B2 改链，B6 stub 化 |

若在 B5 迁移中发现 historical checkpoints 存在**真实 markdown 链接**指向被移空路径，处置方式是为该路径建最小 stub 而非回改历史文件，并在本表登记。

## 5. 跨批链接指针义务（B1 登记）

为保证每批 `check:consistency` 绿灯，B1 建立的文件中指向"将于 B2/B3/B5 迁来"的内容一律先写**当前真实存在**的旧路径；负责迁移该文件的批次必须同批改写以下指针：

- B2：`clients/README.md` §2/§7.2（console docs 与 docs/platforms 桌面 4 件）、`clients/pc/**` 各 README、`clients/shared/docs/{design-system,identity-session,security-privacy}/README.md`、`clients/governance/canonical-sources.md`、`clients/plan/*`；
- B3：`clients/README.md` §3/§7.4（ios/android/mobile-parity/mobile-decision）、`clients/mobile/**` 各 README、`clients/governance/canonical-sources.md`；
- B4：`clients/pc/docs/quality/README.md`（test-strategy 链接）、`clients/shared/docs/telemetry-evidence/README.md`（政策文档链接）；
- B5：`clients/README.md` §3/§6/§7.4/§7.5、`clients/agent-hub/README.md`、`clients/GOVERNANCE.md` §5、`clients/governance/*`、`clients/plan/*`、`clients/prompts/README.md`、`clients/mobile/**`、`clients/shared/docs/relay-pairing/README.md`；
- B6：全仓仍指 `docs/platforms/README.md#console-实现-gate` 的活文档链接改指 `clients/governance/readiness-gates.md`。

## 6. 不迁移清单（含原因）

| 路径 | 原因 |
|---|---|
| `apps/agent-shell/`、`packages/sdk-ts/` | Lane-TSC 所有的代码 package；未经所属车道批准不移动、不重命名；`clients/` 只建立消费侧说明 |
| `packages/contracts-ts/` | Lane-CTR 所有的机器契约 package；同上 |
| `apps/kernel-server/`、`apps/admin-cli/`、`crates/**`、`tools/**` | 不是客户端目录（服务端组合根/管理 CLI/内核实现/工具），明确不纳入 `clients/` |
| `specs/**`、`conformance/**`、`tests/**` | 机器合同与测试资产；`clients/` 只引用不搬迁 |
| `docs/prompts/console-*.md`、`docs/prompts/lane-con.md` | Console/车道提示词留在全局 prompts 域，由 `clients/prompts/README.md` 索引 |

## 7. 手机代码载体措辞变化记录

- 迁移前（docs/clients/README.md）：手机"**没有独立 iOS 或 Android 客户端代码目录，也没有已分配的未来代码路径**"，明令不得创建 `apps/ios` 类虚构索引项。
- 迁移后（clients/README.md）：`clients/mobile/{ios,android}/app/` 成为**已分配的保留入口**——入口已分配但**无任何实现**，不得出现 manifest、源码、构建脚手架或"实现已启动"暗示。该措辞变化由 `CLIENTS-DEC-001` 授权，边界不变：手机仍是 remote companion，不承载 runtime/authority/node/Vault。
