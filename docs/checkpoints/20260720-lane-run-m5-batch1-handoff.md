# 20260720 Lane-RUN Handoff（M5 批 1：确定性管理 fallback）

## 1. 本次会话完成

按 `docs/prompts/milestone-m5.md` / `lane-run.md` 交付 M5 第一个原子任务（分支 `lane/run`，基线 `19f2c22` = 治理提交后的 main）。**测试先行**：先写测试并实录 RED（admin-cli e2e 对 M0 空壳 8/8 行为失败——二进制只打印 skeleton 横幅；管理库套件因 API 不存在编译失败），再最小实现转 GREEN。零新第三方依赖（全部来自 workspace.dependencies）。

- **治理提交（独立小批，已直推 main 且 CI 绿）**：`.cursor/rules/18-auto-commit-and-doc-sync.mdc` + `docs/adr/0008-auto-commit-and-push.md` + AGENTS.md 引用节——所有者授权的自动提交/自动 push 政策（提交 `19f2c22`，main CI run 29759681601 success）。
- **`cognitive-management`（提交 `8c483bb`）**：
  - `session.rs`：PrivilegedManagementSession 按注册 schema 形状手工建模（codegen 尚无该绑定——CORE_SET 追加留给 Lane-CTR，见 §4）+ fail-closed 形状校验（模式/枚举/区间全查）；确定性门：状态（expired→`MANAGEMENT_SESSION_EXPIRED`、revoked→`MANAGEMENT_SESSION_REVOKED`、closed→EXPIRED、pending→`MANAGEMENT_STEP_UP_REQUIRED` challenge）→ 绝对过期 instant 比较 → domain/action/resource scope（前缀覆盖）→ risk ceiling → step-up challenge；全部只出注册码（经 `generated::error_registry` 绑定，无手写码表）。
  - `plane.rs`：四动词。**inspect**＝纯读 authority 事实（state/version/事件计数/fencing epoch；missing≡denied 的 protected-read 同构）；**stop**＝经中央 transition 门的 `TERMINATION_REQUESTED`（guard 只派生不虚证：`writer_fenced` 由真实 epoch 推进成立，`pending_effects_closed_or_quarantined` 由耐久 effect 表全局保守检查派生，在途 effect 存在时先拒并点名 guard；非法目标态交中央门拒，注册码）；**revoke**＝governance ledger revocation epoch +1（M3 `revalidate_grant` 算术即刻判 stale，F-007 两个 race 点被真实 gate 拒）；**reconcile**＝M4 `run_recovery` 八步全序（原幂等键查询、不重派发、不可判则 quarantine fail-safe）。
  - `governance.rs`：GovernanceLedger——管理面持有的授权侧 currency（revocation_epoch/capability_set_version）耐久档（canonical JSON 文件，fail-closed 读写）。
  - `error.rs`：ManagementError → 注册码 wire 部件的全射映射；`model.rs`：ModelProvider 实验缝（确定性路径零调用，测试探针钉死）。
  - `executor_port` re-export：admin-cli 免直接 kernel 依赖。
- **`apps/admin-cli`（同提交）**：std::env 手写解析（零新依赖），子命令 inspect/stop/revoke/reconcile，canonical JSON stdout / 注册错误 JSON stderr / 退出码 0/1/2；reconcile 用未配置执行器（查询 Indeterminate→quarantine，dispatch 大声失败——结构性证明恢复不重派发）。
- **测试（同提交；全部对真实 SQLite WAL store，断言一律 reload 实测）**：admin-cli 8 项进程级 e2e（spawn 真二进制 + 模型环境变量剥离）+ 2 单元；管理库 6 项行为 + 2 单元。覆盖：四动词 happy path、过期/陈旧/撤销 session 全动词拒（事件零增、pending effect 与 intent 原样保留、epoch 不动、ledger 不动）、scope 失配拒、step-up challenge、非法 stop 目标拒、在途 effect 阻断 stop→reconcile 后 stop 通、撤销后 stale grant 派发被真实 gate 拒（dispatches=0）、三种在途形态对账收敛（executed/vanished/authorized-undispatched，原键、零重派发）、畸形 session 文档 fail-closed、**零模型调用探针（wired 但 calls==0）**。
- **文档联动（docs 批提交，哈希见 git log）**：matrix 回填 5 REQ（REQ-MGMT-FALLBACK-001、REQ-MGMT-SESSION-002/003、REQ-MGMT-GATE-001、REQ-MGMT-SESSION-LIFECYCLE-001，notes 注明批 1 分量与剩余分量）+ `gen-matrix` 再生成无 drift；PROGRESS（M5 行、实现计数 34→39、测试行 +16、车道表、handoff 列表）；`tests/e2e/README.md` 结构性索引；本 handoff。

## 2. 未完成 / 进行中

- **RUN 批 2（下一批）**：Management API HTTP 面 + PrivilegedManagementSession 签发/续期生命周期 + ManagementActionProposal/approval（消费 `generated::{management_approval_request,management_approval_decision}`，R1 结构化确认三负例语义）+ AKP envelope + Operation 执行器 + Harness Loop（等 KRN M5 端口：UserIntentRecord/interpretation 绑定、Loop 驱动、admission 编排）。
- gateway.configure / diagnostics.configure（fallback 向量 requested_operations 其余两项）＝批 2 Management API 面。
- authority_signature 密码学验证（本批仅形状校验）+ idle-timeout 时长运算（本批只做绝对过期 instant 比较；时长算术需日历运算）＝批 2 session 生命周期。
- stop 的 pending-effects 检查是全局保守版（任何非终态 effect 阻断任何 stop）；per-execution 绑定需 kernel 暴露 execution↔effect 关联（见 §4 给 KRN 请求）。

## 3. 测试与证据状态

- **RED 实录**：e2e 对 M0 空壳 8/8 失败（`expected JSON on stdout, got …"admin-cli M0 skeleton…"`、退出码 0≠1）；库套件 E0432（API 不存在）。GREEN：全 workspace **35 套件 166 项测试 0 失败**（新增 16 项：管理库 7 + admin-cli 9）。
- 验证矩阵全绿（本机）：`cargo fmt --all -- --check`；`cargo check --workspace`；`cargo test --workspace`（166 项）；`cargo clippy --workspace --all-targets -- -D warnings`；`pnpm -r build`；`pnpm -r test`（contracts-ts/sdk-ts 67/agent-shell 12/tools 全绿）；`pnpm run check:consistency` OK（273/55/61/84）；`gen-matrix --check` 无 drift；`git diff --check` 干净。CI 结论以 PR 页为准（push 后观察）。
- **向量：零改动、零执行**——84 份原样；management 组（含 MGMT-FALLBACK-008）保持 **not-run**，行为执行归 Lane-CFR M5 批（本批断言按其 expected 语义设计：`management_api_available/deterministic_cli_available/model_required:false/same_authorization_effect_and_audit_gates` 的车道侧证据）。
- 本批不构成任何 Profile 覆盖声明；状态用语＝「实现已提供 + 车道测试已执行」。

## 4. 未决风险与漂移（含跨车道请求）

无新漂移登记（未触碰 specs/conformance；F/IMP 台账无状态变化）。跨车道请求与口径：

1. **给 Lane-CTR**：`privileged-management-session.schema.json`（及 `management-action-proposal`）生成绑定一行 CORE_SET 追加（F-011 handoff §2 预告的流程）；落地后 `session.rs` 手工类型换绑（成员集一致，替换点已在模块文档注明）。
2. **给 Lane-KRN**（批 2 前协商，不阻塞本批）：① governance currency（revocation epoch/capability set version）建议收编为 store 表 + ProtocolStore 端口（当前为管理面 canonical-JSON 文件档，单写者假设，非 authority store 事务单元）；② execution↔effect 关联暴露（loop/task 归属查询），使 stop 的 pending-effects 检查从全局保守版收窄为 per-execution。
3. **口径**：stop 在 RUNNABLE/WAITING 前置检查通过后即推进 fencing epoch 再提交（epoch 推进先于 CAS，竞态下可能推进了 epoch 但 CAS 拒——fail-safe 方向，只损可用性不损安全）；reconcile 把「当前 epoch 写者」视为 crashed lease 交 `run_recovery`（管理对账语义 = 强制 fence 现役写者，单节点参考实现无并发写者）。
4. 本机工具链新坑（本批发现并已固化打法）：rustup gnu 宿主装了 llvm-mingw 且其 bin 在系统 PATH 尾部，rustc 链接器解析到它触发 `-lgcc_eh` 失败——**每个新 shell 都要**设 dlltool-shim 前置 PATH + CC/AR 绝对路径 + `RUSTFLAGS="-C link-self-contained=yes"`（本批实测缺一不可）；TMP=D:\tmp 照旧。

## 5. 下一步入口

- 建议提示词：`docs/prompts/milestone-m5.md` + `docs/prompts/lane-run.md`（RUN 批 2 范围见 §2）；Lane-CFR M5 行为批可直接消费本批：management 向量执行可复用 `cognitive-management` 公开 API + `cognitive_store::faults`。
- 工作分支：`lane/run`（本批合并后按需重建）。
- 第一个动作：`git fetch origin ; git merge origin/main`，读 KRN M5 端口冻结公告（PROGRESS 车道表），先为 Management API 的 proposal→approval→effect 全链写失败测试。

## 6. 快照

- PROGRESS 已更新：是（里程碑 M5 行、实现计数 39、测试行、车道表、handoff 列表）。
- 本次提交：`19f2c22`（治理，已在 main）→ `8c483bb`（实现+测试+e2e 索引）→ docs 批（matrix/PROGRESS/本 handoff，哈希见 git log）。
