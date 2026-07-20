# CognitiveOS Console v2 — 路线图与未来 Feature Brief

> 状态：Product roadmap hypotheses
>
> 基线：Windows v1 见 [windows-v1-scope.md](../docs/platforms/windows/windows-v1-scope.md)
>
> 规则：本页保留未来价值、不可变边界和进入门禁，不冻结未经验证的页面布局。
>
> Agent Hub 关系：Direct Takeover 与第三方 Agent 接管是一条独立于本路线图的产品线，其里程碑/DAG/gate 在 [agent-hub/planning/README.md](../../agent-hub/docs/planning/README.md) 与 [docs/plan/agent-hub-development-plan.md](../../agent-hub/plan/agent-hub-development-plan.md)，同样受 Console/M5/平台/AGPL gate 阻断。

## 1. 排序原则

1. 先闭合单机 Windows v1 的身份、状态、监督、Effect 和 Agent 生命周期。
2. 后续平台或模块不得通过削弱 authority、risk floor、通道隔离或状态语义来“复用”首版。
3. 每个 feature 进入设计前必须有目标 persona/JTBD、机器合同状态和可执行验收。
4. 未通过 gate 的能力保持 `planned/blocked`，不出现在生产导航。

## 2. Roadmap 概览

### Phase A — Windows v1

范围：

- 共享本机 Windows Service；
- 节点账号、TOFU、Owner bootstrap；
- Shell、任务监督、supervision lease；
- Agent 完整生命周期；
- R0/R1；
- 嵌入式最小治理。

出口：见 Windows v1 release gates。没有执行证据前不进入 Public Beta。

### Phase B — Windows 远程与多工作区

用户价值：

- 在同一 Console 管理本地开发节点和远程环境；
- 显式切换 tenant/workspace/node；
- 保持每个工作区独立 credential、store、cache 和 watch。

不可变边界：

- 远程/企业节点禁止复用本地 TOFU 默认；
- 使用组织预置信任、受管发现或带外指纹；
- 跨工作区切换清理草稿资源 token、管理 session 和不适用 cache；
- 节点/环境身份在 preview 和通知深链中常驻，不能只靠颜色。

进入门禁：

- remote AuthenticationSession/PKCE 或部署批准的等价合同；
- workspace/tenant/channel 隔离负例；
- deep link audience/version binding；
- 网络分区和 stale cursor 故障注入。

### Phase C — macOS 与受限 Linux

用户价值：把 Windows 已确认的任务与治理语义适配到两个独立、明确受限的平台产品切片。详细规格：

- [macOS v1 产品设计](../docs/platforms/macos/macos-product-design.md)
- [Linux v1 产品设计](../docs/platforms/linux/linux-product-design.md)
- [桌面 parity matrix](../docs/platforms/desktop-parity-matrix.md)

产品范围：

- macOS：Universal 2、macOS 14+ 设计候选、Developer ID 签名并公证 PKG、machine daemon + per-user broker；
- Linux：仅 Ubuntu 24.04 LTS x86_64、stock GNOME/Wayland、官网 `.deb`、24 个月且不越过 security floor；
- 两平台 Profile 均为 `planned`，implementation 未提供、test 未执行。

不可变边界：

- 不宣称像素一致；保持语义、状态和安全门禁一致；
- 平台证据不得外推；
- Console/broker/helper 非 authority/IdP/node/final arbiter；
- Linux 不简称为泛化“Linux 支持”，macOS 不承诺 Apple 固定生命周期；
- WebKit、secure storage、notification、helper、update 和 accessibility 必须按平台独立验证。

进入门禁：

- 通用 Console 依赖组 1/2/7 + M5 出口；
- [目标平台 Open PoC/GA gates](../../governance/readiness-gates.md#console-实现-gate) 使用真实 API/真实 OS 行为留证；
- 平台安装/签名/更新/恢复、secure storage、AT/键盘/contrast/motion、daemon/helper 权限、两次升级演练均通过。

<a id="phase-d--mobile-remote-companion"></a>
### Phase D — Mobile Restricted Console

产品方向已经按 iPhone 与 Android phone 分别冻结：

- [iPhone-only v1 产品设计](../../mobile/ios/docs/ios-product-design.md)
- [Android phone v1 产品设计](../../mobile/android/docs/android-product-design.md)
- [移动 parity matrix](../../mobile/shared/docs/mobile-parity-matrix.md)
- [移动平台决策记录](../../mobile/shared/docs/mobile-platform-decision-log.md)

首要任务：

- Conversation/Task 创建、监督、纠偏、请求暂停与安全对账；
- 在单一活动账号下选择 authority 返回的 tenant/node；
- 分离查看 Task、Loop、AgentExecution、Effect、Verification 五个 authority lifecycle 域，以及独立远端 Runtime projection；Runtime stop 不推进 Task 或 Effect；
- 处理 R0/R1、`CANDIDATE_COMPLETE`、`OUTCOME_UNKNOWN` 与 Inbox；
- 使用 authority catalog/package ref 管理远端 Agent install、upgrade、rollback、uninstall。

明确不做：

- 本地节点托管；
- 手机下载、解释、执行、缓存或转发 Agent executable/package；
- 任意系统进程管理；
- 无限后台 watch；
- R2/R3、通知 action 批准或离线控制队列；
- iPad/tablet/foldable/watch/widget/Live Activity 的 v1 GA；
- 大规模审计、图谱或批量运维。

不可变边界：

- App、APNs/FCM、系统通知、生物识别和 integrity signal 都不是 authority 或完成证据；
- push 只携 opaque hint，唯一 action 是打开 App；回前台后重新认证并 resnapshot；
- supervision lease 只在前台、已解锁、session/watch/UI fresh 时续；后台、锁屏、进程死亡、force-quit/force-stop 停止续租；
- R1 使用 digest-bound device key signature，系统生物识别只解锁 key，authority 最终决定；
- acquisition 仅向远端 node 提交 authority package ref，移动端无 executable bytes；
- 移动端能力由平台和远端 authority 决定，不因窄屏、MDM 或本地 risk signal 扩大。

进入门禁：

- 通用 Console 依赖组 1/2/7 + M5 出口；
- account/device/session/push/lease/R1/floor/revoke 等移动 carrier 已登记；
- iOS 与 Android 各自 Open PoC/GA gates 使用真实设备、真实商店和真实 OS 行为留证；
- Public/managed 独立身份、App Store/Google Play policy、动态代码边界和更新恢复闭合；
- VoiceOver/TalkBack 与其他已列辅助技术完成核心旅程；
- 设备绑定、远程 revoke、丢失/换机/restore、security floor 与商店审核延迟均有执行证据。

### Phase E — 完整治理工作区

包含：

- Users & Access；
- Membership、Delegation、Capability；
- Approval Center；
- Audit Search/Timeline/Export；
- break-glass；
- 设备和会话管理；
- Operation Catalog；
- 完整 System 配置/更新。

不可变边界：

- persona 不等于固定 RBAC；
- 管理员默认无用户正文读取权；
- break-glass 独立、限时、范围固定、多主体（按适用风险）且可审计；
- `acknowledged` 不替代 `handled`；
- audit 缺口、权限裁剪和签名无效分开显示。

进入门禁：

- AuditRecord/export/integrity manifest；
- Membership/AuthorizationDecision/ShareGrant；
- device/session binding；
- audit-readiness 与 fail-closed；
- 企业用户研究和职责分离测试。

### Phase F — R2/R3 可信确认

产品价值：支持高影响动作、独立审批和双人/quorum 决定。

不可变边界：

- 普通聊天、WebView 系统卡、密码和 push action 不能批准；
- authority-owned hardened origin 或经验证原生可信面；
- 固定 canonical display profile、digest、domain separation、nonce、expiry、principal/audience、RP/origin、trust rotation/revocation；
- proposal 发起者不能自批适用的独立审批；
- R3 按机器合同实现 quorum，不由客户端计数猜测。

进入门禁：

- signed proposal/display schema 闭合；
- Approval service/challenge/quorum 合同；
- passkey/FIDO2 与 display integrity threat model；
- renderer compromise、replay、phishing、跨设备竞态；
- 取消、断网、结果未知和另一设备已处理状态机；
- approval fatigue/anti-bombing。

### Phase G — Governed Memory 与认知发现

包含：

- MemoryCandidate 私有工作集；
- admission、conflict、stale、quarantine、invalidation、delete；
- Context 来源和增量；
- 停滞检测。

不可变边界：

- Memory 不等于聊天、Context、World State 或 Knowledge；
- writer visibility、Conversation/Activity/scope 隔离；
- 客户端不创造准入或冲突事实。

进入门禁：

- working-set carrier、状态/条件 schema 和 service；
- read-your-write/跨 scope 负例；
- invalidation/delete 审计；
- 当前 Memory schema 与评审状态重新核验。

### Phase H — Knowledge

包含：

- Evidence、Claim、KnowledgeObject；
- 支持/冲突/替代/派生；
- 搜索、详情、lineage graph 和结构化列表。

不可变边界：

- Claim 不等于 truth；
- 每个派生可追来源；
- source invalidation 传播且保留审计；
- 权限裁剪不泄露不可发现节点；
- 图谱始终有无障碍列表/表格等价物。

进入门禁：

- Evidence/Claim/KnowledgeObject/CompilationProfile 合同；
- knowledge authority/compiler；
- 大图性能和 AT；
- invalidation/删除影响测试。

### Phase I — Multi-Agent / Distributed

包含：

- 显式衰减委派；
- handoff/checkpoint/verifier；
- Lease/Mailbox/ConflictSet；
- 跨节点协作。

不可变边界：

- 消息只是 claim/evidence，不是 shared-state authority；
- child completed 不完成 parent Task；
- 每次委派显示 scope/data/budget/deadline/capability/verifier/handoff/escalation；
- 网络分区下不宣称停止或完成。

进入门禁：

- 分布式 companion 和 machine contracts；
- cross-node trust/ShareGrant/local reauthorization；
- partition/fencing/replay；
- parent/child acceptance 负例。

## 3. 暂不冻结的设计

在对应 phase 进入研究前，不把以下内容写成最终 UI：

- macOS/Linux 已确认切片之外的 distro/desktop/package、富内容和扩展系统集成；
- iPad/tablet/foldable/watch/widget/Live Activity 的导航与完整页面；
- R2/R3 浏览器还是原生可信面；
- Knowledge 图引擎；
- Audit export 格式；
- 企业角色名称和默认权限；
- break-glass quorum；
- 远程 notification gateway/provider；
- Android 本地 Runtime 实验；
- 跨平台字体包和安装渠道。

可以保留不可变安全边界、目标任务和验证门禁，但不得用页面 mock 暗示后端能力已存在。

## 4. Phase 入口模板

每个未来 feature 立项时补全：

1. 首要 persona 与最多三个 JTBD；
2. 当前痛点和现有替代；
3. 支持平台/部署 Profile；
4. 明确范围与非目标；
5. authority/contract/data/permission 模型；
6. 关键旅程和失败恢复；
7. 页面/组件/状态矩阵；
8. accessibility/privacy/security；
9. contract/implementation/evidence 状态；
10. 成功指标、baseline、target 和 release gate。

## 5. 不随路线图改变的产品红线

- Console 不拥有 authority、commit、root 或用户密码验证。
- Agent/网页/日志/包来源永远是不可信输入。
- 风险下界由 authority 决定，客户端不能降级。
- R2/R3 不在普通聊天或 push 中批准。
- Task/Loop/AgentExecution/Runtime/Effect/Verification 不合并为一个“运行状态”。
- `CANDIDATE_COMPLETE` 不等于 `COMPLETED`。
- `OUTCOME_UNKNOWN` 禁止盲重试。
- workspace/tenant/channel/session/cache 不能隐式复用。
- 平台证据不外推。
- specified、implementation available、test executed、Profile implemented 四类状态严格区分。
