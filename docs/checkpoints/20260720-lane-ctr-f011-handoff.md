# 20260720 Lane-CTR Handoff（F-011 R1 审批合同登记批）

## 1. 本次会话完成

F-011/IMP-05 的 R1 聊天内结构化确认最低集机器合同登记（M5 入口 gate 唯一剩余项；权威范围：F-011/IMP-05 台账 + M4 review §7 + CFR M4 handoff §5；分支 `lane/ctr`，基线 `7a7673f` = PR #13 merge）。

**冻结判定**（依据入 F-011 台账行）：RFC-0001 §7.6 与 REQ-AKP-MGMT-002 normative 描述结构化 challenge/approval 交换；白皮书 §12.12 明文预授权"REQ/向量登记按修正型通道另行提交"；F-011 本身即"合同未登记"的登记义务 → 修正型收敛，非新增对象族（management approval 族已有 proposal/decision 两 schema，request/challenge 是被描述的第三腿）。零新错误码（"宁少勿造"达成）。

### 登记交付物

- **新 schema `management-approval-request.schema.json`**（60→61）：OS 签发的审批挑战卡——schema_version/request_id(mar_)/proposal_ref+digest/risk_class/**confirmation_surface**（policy_auto|chat_structured|trusted_surface|dual_independent）/human_principal/proposer_principal+actor_chain_digest/**channel_identity**（OS 独立通道身份）/challenge_digest（一次性挑战）/method/**single_use（const true）**/aggregation_key（防疲劳聚合键，可选）/requested_at/expires_at。分级条件全 fail-closed：R0 ⇒ policy_auto；R1 禁 policy_auto；R2 ⇒ {trusted_surface,dual_independent} + 必绑 session_ref；R3 ⇒ dual_independent + session_ref（§12.12 矩阵的 schema 化：聊天永不是 R2/R3 完成面）。
- **`management-approval-decision.schema.json` 硬化**：新增 `request_ref`/`single_use` 成员 + R1-approve 条件（必绑 request_ref 且 single_use=true）；**既有 R2/R3 形状零破坏**（双语言合同测试含 non-breaking 正例证明；全仓无 schema 实例受影响——7 份 validate_against 向量均不指向本 schema）。R1 决策 `session_ref` 口径 = 一次性审批上下文（§12.12 聊天审批不建常驻 session，与 RFC §7.6 session 绑定相容），写入 authn-authz-capability §5。
- **3 份行为负例向量**（81→84，layer management-shell，profiles core_digital + intelligent_management_shell，登记 **not-run** 不虚报，行为执行归 CFR M5 批）：
  - `MGMT-APPROVAL-R1-009`（缺确认强执行）：无 decision 不 dispatch；自然语言"approved"不构成批准 → challenge + `MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED`，dispatches 0。
  - `MGMT-APPROVAL-SELF-010`（自批）：approver==proposer 与"伪造审批卡"链重叠两式 → `MANAGEMENT_SELF_AUTHORIZATION_DENIED`。
  - `MGMT-APPROVAL-FATIGUE-011`（防疲劳）：过期出示/单次重放/challenge digest 失配/25 次重发轰炸 → 拒绝 + fresh challenge + 聚合限流，`auto_approval_granted:false`。
- **registry 映射**：REQ-MGMT-APPROVAL-001（+3）、REQ-MGMT-GATE-001（+R1-009）、REQ-MGMT-AUTHZ-001（+SELF-010）、REQ-AKP-MGMT-002（+R1-009/FATIGUE-011）；matrix 再生成。
- **codegen 判定 = 本批入 CORE_SET**（30→32 模块 ×2 语言，SCHEMA_ID/SCHEMA_DIGEST 常量 + 聚合表更新）：消费方 = M5 Lane-RUN 管理面，被本登记 gate 且合并后即开工——**确定具名消费者**，与 membership 的条件式消费者（defer 先例）判然有别；理由入 ADR-0006 delivery record 第 7 条与 CORE_SET 注释。
- **计数钉扎同批红→绿**：static_check.py 61/84；ci.yml 84/46/0/0/0/38（pass 46 不变，self-check ≥27 不动——新向量非 schema-gate 形态，不入执行/自检面）；runner_execution.rs TOTAL 84/NOT_RUN 38；生成模块计数 32（generated_types.rs/generated-types.test.ts）；AGENTS 61/84；rules/12。
- **docs 联动**：authn-authz-capability（machine assets + §5 R1 登记段）；conformance/README 管理资产段；PRODUCT-DESIGN 漂移标注；F-011/IMP-05 台账行更新（**合同已登记，行为验证挂 CFR M5，非 closed**）；PROGRESS（M5 gate 达成行、计数、车道表、handoff 列表）。

## 2. 未完成 / 进行中（M5 消费方替换点）

- **Lane-RUN（M5，本批后即可启动）**：管理面消费生成绑定 `generated::{management_approval_request::ManagementApprovalRequest, management_approval_decision::ManagementApprovalDecision}`（信封 schema_digest 钉扎用各模块 `SCHEMA_DIGEST` 常量）；审批门行为语义按三份负例向量 expected 实现（挑战单次核销、过期拒、自批拒、重发聚合限流）。管理族其余 schema（action-proposal/privileged-management-session）需要生成绑定时一行 CORE_SET 追加（同 membership 流程）。
- **Lane-CFR（M5 批）**：三份 F-011 向量行为执行（脱 not-run）+ F-011 闭合判定；管理面其余 8 份 management 向量同窗口。
- **F-011 剩余**：R2/R3 完整审批机器化（passkey/FIDO2 签名绑定、双人 step-up）后置，不阻塞 v0.1。
- CTR 同窗口遗留：D-016、membership 绑定、D-018 实施支持（PR #11 已裁决路径）。

## 3. 测试与证据状态

- Rust：contracts 全绿（schema-contract **11**（+2 审批正负例套）+ generated-types 6（32 模块钉扎）+ 其余不变）；workspace build/test/clippy -D warnings/fmt --check 绿（数字见 PR）；conformance runner_execution 钉扎测试同批调整后绿。
- TS：contracts-ts **35** 测试（+2 审批套）；pnpm -r build/test 全绿。
- 静态门：check-consistency OK（273/55/**61**/**84**）；gen-matrix --check 无 drift；static_check.py ALL CHECKS PASSED（61/84 钉扎）；validate-manifest OK。
- runner：**84 枚举 / 46 pass / 0 fail / 38 not-run**；self-check **27/27**（新向量不入 corrupted 语料——行为负例非 schema-gate）；本地复跑确认。
- codegen regenerate-diff 空；golden 双语言 byte-identical（canonical 面未动，live schema-bundle digest 随 61 schema 自然更新且双侧一致）。
- 既有向量：**零改写、零删除**（81 份原样，+3 新增）。

## 4. 未决风险与漂移

- 无新漂移条目（登记依据入 F-011 行）；D-016/D-017 维持 deferred，D-018 维持 decided。
- 口径提示（接续者）：`MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED` 在 R1 语境 = "缺乏有效审批"（人类确认者对提议 workload 独立 ✓），非仅 R2/R3 双人语义——与 MGMT-APPROVAL-005 的用法一致。
- 本机资源纪律照旧（TMP=D:\tmp、降并发、llvm-mingw CC/AR 绝对路径；本批开工 D 盘 5.37GB 足用）。
- 无并行车道（M5 等本批合并后启动）；PROGRESS/ledger 冲突面无。

## 5. 下一步入口

- **M5 启动（Lane-RUN + Lane-KRN 协作 + Lane-TSC 集成）**：`docs/prompts/milestone-m5.md` / `lane-run.md`；入口 gate 全项达成（M4 出口 + F-011 R1 登记完成）。第一动作建议：读本 handoff §2 替换点 + D-018 台账（envelope 组装器路径）+ KRN M4 handoff §5。
- **Lane-CFR M5 批**：shell/management/harness 向量 + 本批 3 份审批负例行为执行。
- 第一个动作（任意接续车道）：`git fetch origin; git merge origin/main`，读 PROGRESS 车道表触碰通告。

## 6. 快照

- PROGRESS 已更新：是（M5 gate 达成、计数 61/84、五态 46/38、车道表、handoff 列表）。
- 本次提交：提交 1（schema 对 + 向量 + registry + matrix + 钉扎 + codegen + 测试）→ 提交 2（docs 批：台账/标准/README/PRODUCT-DESIGN/PROGRESS/本 handoff）；哈希见 git log。基线 `7a7673f`。
