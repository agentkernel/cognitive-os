# 20260721 Lane-CON clients Phase 0 audits Handoff

## 1. 本次会话完成

- **工作树**：`D:\agent-kernel-clients`，分支 `work/clients-full-development`（rebase 至含 rule 18 / ADR-0008 的 `origin/main`）。
- **Gate**：implementation-ready 仍 **NO-GO / blocked**（未写 clients 生产代码/manifest/mock）。
- **A. AH-CTR-02 文档级**：更新 `provider-interfaces-ledger.md`、六份 tier1 dossiers、`terms-and-licenses-ledger.md`；Hermes **decided** = `NousResearch/hermes-agent`（MIT，`~/.hermes/state.db`，无对外控制 API）；MCP current=2025-11-25；ACP 稳定版本 1。状态用语：接口已核验（文档级）/ evidence not-run。
- **B. POC-LIC 材料**：更新 `licensing-and-terms.md`（Paseo=`getpaseo/paseo` AGPL package or-later、§13 两要件、clean-room、候选栈义务）；evidence-index 三行 POC-LIC 保持 not-run + 备注「评估材料已整理，法务评估未执行」；risk-register 登记外部阻断 AH-EXT-01..04。
- **C. 威胁 / PoC**：纠正「21 威胁项实测」→「21 项威胁已规范登记，oracle/evidence 全 not-run」；新增 `threat-test-oracles.md`；正式登记 planned：POC-HOST-001、POC-TERM-003、POC-PROC-005、POC-RELAY-005、POC-GOV-001。
- **D. 进度同步**：`clients/plan/progress.md`、`clients/READINESS.md`、agent-hub plan/docs progress、全局 `docs/plan/PROGRESS.md` 最小 Console 行；design-system README 登记 planned 缺口指针。
- **Documentation Auditor**：transcript `0c341a8d-864e-4103-9d17-cf8cd89f8472` **已有完整结论**（146 md、1043 内链 0 断、零实现文件）；本回合交付前另做 §9 抽样核验。
- **关联**：CLIENTS-DEC-001 / ADR-0007 / ADR-0008（rule 18）/ AH-CTR-02 / POC-LIC；无 REQ 实现变更。

## 2. 未完成 / 进行中

- POC-LIC-001..003 法务评估执行与留证。
- Tier 1 / 五平台 Open PoC 真实执行（含本回合 5× planned）。
- 后端依赖组 1/2/7 + M5 出口；技术栈 ADR。
- 外部阻断：Anthropic 订阅自动化、OpenAI ChatGPT 包装意见、Apple PLA、Paseo 双许可。
- 完整五平台 PoC 执行手册正文、共享 design token 全文规格——本回合刻意暂缓。

## 3. 测试与证据状态

- CI：以 PR checks 为准（推送后观察）。
- 本地：`pnpm run check:consistency`；`git diff --check`；clients §9 手动 gate。
- 向量 / Profile：无变化（84 / 46 pass / 38 not-run；Profile 0）。
- Agent Hub PoC：28 not-run + 5 planned；evidence none。

## 4. 未决风险与漂移

- OpenClaw Gateway WS 字段级仍 partial；默认绑定地址官方 vs 三方冲突 → PoC。
- OpenCode 上游 troubleshooting 存储文档过时 → 引用时绕开。
- Claude TS SDK Commercial ToS / 订阅自动化外部确认。
- 禁止触碰本地 `D:\agent-kernel` main 上 blog 提交 `30ea63f`（不得 push）。

## 5. 下一步入口

- 建议提示词：`clients/agent-hub/prompts/lane-contract-capability.md`（AH-CTR-04 漂移监测）或法务/PoC 执行提示词（待开）。
- 工作分支：`work/clients-full-development`（经 PR 合入 main；禁止 clients 实质性文档直推 main，见 ADR-0008）。
- 第一个动作：合并本 PR 后，领取 POC-LIC 法务评估或首个 runtime PoC（须后端/ADR 策略允许的 informative/PoC 边界内）。

## 6. 快照

- PROGRESS 已更新：是（Console/Lane-CON/readiness 最小行）。
- 本次提交列表：见 PR / `git log origin/main..HEAD`。
