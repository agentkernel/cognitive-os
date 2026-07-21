# 20260721 Lane-CTR Handoff（M5 修正型生成绑定批）

## 1. 本次会话完成

- 基线：`759d30a`（本地 `origin/main`，含 KRN M5 与 RUN 批 1）；远端 fetch 因 HTTP 403 不可用，已用本地缓存 ref fast-forward。工作树开工前干净。
- 修正型 codegen 输入扩展：既有已登记 `intent-interpretation.schema.json`、`privileged-management-session.schema.json`、`management-action-proposal.schema.json` 纳入 `CORE_SET`。直接入口 30→33，含 `$ref` 闭包的生成 schema 模块 32→35；generator 保持 0.2.0，渲染语义不变。
- 双语言生成物：Rust `generated::{intent_interpretation, privileged_management_session, management_action_proposal}`；TS `generated/{intent-interpretation,privileged-management-session,management-action-proposal}`。各模块带 `SCHEMA_ID` / `SCHEMA_DIGEST`，聚合 `SCHEMA_DIGESTS` 同步。
- 测试先行：Rust/TS 先因三个模块不存在失败，再扩 CORE_SET 生成转绿。类型测试钉 required/enum；schema validator 双语言覆盖 material ambiguity ⇒ `clarification_required`、session risk enum、proposal required 与 unknown member 拒绝。
- 零规范资产变化：未改 schema/registry/vector/REQ 域/错误码/对象族/行为语义；无 findings 漂移需登记。

## 2. 消费者精确换装清单

- Lane-KRN 唯一换装点：`cognitive_kernel::intent_chain::record_interpretation_candidate`，将手工候选 canonical JSON 组装机械替换为 Rust `cognitive_contracts::generated::intent_interpretation::{IntentInterpretation, IntentInterpretationAmbiguitieItem, IntentInterpretationStatus}`；schema pin 用同模块 `SCHEMA_ID` / `SCHEMA_DIGEST`。不改变 `derive_candidate_status` 或 admission 行为。
- Lane-RUN session 换装点：`cognitive_management::session`，以 Rust `cognitive_contracts::generated::privileged_management_session::{PrivilegedManagementSession, PrivilegedManagementSessionScope, PrivilegedManagementSessionRiskCeiling, PrivilegedManagementSessionState}` 替换手工形状；schema pin 用模块常量。现有 fail-closed gate 行为保持。
- Lane-RUN proposal/approval 路径：消费 Rust `cognitive_contracts::generated::management_action_proposal::{ManagementActionProposal, ManagementActionProposalRiskClass}`；与既有 `management_approval_request` / `management_approval_decision` 生成模块串接。TS 同名 kebab-case 模块可供 wire/API 客户端。
- CTR 未修改 KRN/RUN crate；换装与消费者行为测试由各所有权车道完成。

## 3. 测试与证据状态

- Rust contracts：42 项通过（lib 15、generated-types 7、golden 2、projection 6、schema-contract 12）。
- TS contracts：37 项通过。
- 最终全仓门（本机）：workspace test **191** passed；clippy -D warnings 绿；fmt --check 绿；pnpm -r build/test 绿（contracts-ts **37**）；check:consistency OK（273/55/61/84）；向量 0 变化。
- 向量：0 变化、0 执行；Profile 状态不变。

## 4. 未决风险与漂移

- 远端 GitHub 当前返回 403，故 push/PR/CI/merge 尚不能执行；认证恢复后从 `lane/ctr` 推送并按双 OS/main CI 门完成。
- 生成器将 `ambiguities` 项类型命名为既有算法产物 `IntentInterpretationAmbiguitieItem`（语法不美观但稳定、可消费）；不在本修正型批修改渲染算法。
- 无规范漂移；schema bundle 集合未变，bundle digest 预期不变，最终门实测记录后更新本节。

## 5. 下一步入口

- 工作分支：`lane/ctr`。
- 先完成最终验证与提交；认证恢复后 push/PR。KRN/RUN 分别按 §2 换装，不改语义。

## 6. 快照

- PROGRESS 已更新：是。
- 本次提交：见 git log（修正型绑定，无行为 REQ 影响；关联 M5 / ADR-0006）。