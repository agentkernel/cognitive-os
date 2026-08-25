# Control Plane baseline → Personal Desktop 1.0 candidate delta

Date: 2026-08-25
Status: **candidate delta index / non-canonical / no implementation authorization**

本文件把 `docs/design/01–41` 作为 2026-08-24 的 dated Control Plane baseline，映射到
[Personal Desktop 1.0 候选产品](./03-personal-product-design.md)、
[交互视觉](./04-personal-interaction-and-visual-spec.md)和
[架构](./05-personal-architecture.md)。它不编辑、移动、重命名或删除 baseline 文件，也不
宣称 candidate 文档已 supersede Accepted ADR、正式计划或活动任务。

## 1. Why a delta index

**FACT**：

- `docs/design/**` 当前是受保护的 untracked dated baseline；
- active `lease/personal/P7-T05/control-plane-foundation` 正在执行 P7-T05/D13
  Work inventory + governed Task creation，并明确消费 `docs/design/14`、`39`；
- Accepted ADR-0036 和正式 Personal plan 仍把 `1.0.0` 定义为 Linux x86_64 release，
  Web UI、Windows installer 与 non-Pi adapters 为 post-1.0；
- 2026-08-25 owner 新方向把“Personal Desktop 1.0”作为候选产品范围，尚未通过
  product-semantic ADR 或 formal-plan rebaseline。

因此，本轮只在 `docs/agent-work-system/**` 记录 delta，避免同时存在两个 writer 修改
D13 的 design inputs。

## 2. Disposition vocabulary

| Disposition | Meaning |
|---|---|
| KEEP | 设计原则/对象/交互仍直接有效 |
| AMEND VIA PERSONAL DOCS | baseline 保留；新范围先在 `03`–`05` 叠加 |
| SUPERSEDE AFTER ADR | 只有 accepted product-semantic ADR 与 formal-plan rebaseline 后才能 canonical supersede |
| STALE IMPLEMENTATION SNAPSHOT | 保留为 dated as-built evidence，不作当前事实 |
| RETAIN AS EVIDENCE | 保留研究、审查、设计推理或历史验收证据 |

## 3. Complete 01–41 map

| # | Baseline file | Disposition | Personal Desktop 1.0 delta |
|---:|---|---|---|
| 01 | `01-control-plane-product-model.md` | AMEND VIA PERSONAL DOCS | Product center 从通用 Control Plane 扩展为 local-first continuity workspace；新增 Conversations、Library 与 Retain loop；不使用 employee/company metaphor |
| 02 | `02-control-plane-jtbd.md` | AMEND VIA PERSONAL DOCS | 新增 office worker、programmer、researcher 三类 JTBD；共享对象，不拆三个产品 |
| 03 | `03-control-plane-capability-model.md` | KEEP + AMEND | 保留 capability honesty；增加 management modes、Conversation/Knowledge/Context/usage capability states |
| 04 | `04-control-plane-conceptual-model.md` | AMEND VIA PERSONAL DOCS | 保留 Task/Agent/Provider/Resource 边界；新增 Conversation、Continuation Package、Library family distinctions |
| 05 | `05-control-plane-ia-options.md` | RETAIN AS EVIDENCE | IA alternatives 仍解释旧选择；新候选 IA 在 `03`，不回写当前 options |
| 06 | `06-control-plane-recommended-ia.md` | SUPERSEDE AFTER ADR | 候选导航改为 Home、Work、Conversations、Agents、Library、Providers、Activity，System 为 footer utility |
| 07 | `07-control-plane-user-flows.md` | AMEND VIA PERSONAL DOCS | 增加 Ready→Continue→Review→Work→Verify→Retain、Conversation resume/import/export、Library/Context flows |
| 08 | `08-control-plane-agent-ux.md` | KEEP + AMEND | 保留 supervision/preview/evidence；增加 Agent readiness、Binding 与 Continuation Checkpoint |
| 09 | `09-control-plane-apple-design-principles.md` | KEEP | 保留 clarity/deference/depth/feedback/recovery；“macOS-like”不得复制 macOS chrome |
| 10 | `10-control-plane-design-decisions.md` | RETAIN AS EVIDENCE | 原 decision log 保留；新 owner scope 在 `02`，接受后再合并 canonical decision |
| 11 | `11-control-plane-design-system.md` | KEEP + AMEND | 保留 token/state foundation；`04` 增加 Windows-first system font、material、motion、high-contrast 和 Library/Conversation components |
| 12 | `12-control-plane-app-shell.md` | AMEND VIA PERSONAL DOCS | 从 Web app shell 扩展为 native shell + existing Web client + local daemon；shell 不是 authority |
| 13 | `13-control-plane-home-spec.md` | KEEP + AMEND | 保留 attention surface；Home 首位变为 Continue/Needs attention，而非 module launcher/KPI wall |
| 14 | `14-control-plane-work-spec.md` | KEEP / ACTIVE D13 INPUT | Work 仍是 governed Task truth；不由 Conversation 替代；当前不得修改 |
| 15 | `15-control-plane-work-detail-spec.md` | KEEP | dual authority/observation、Effects、Evidence、Context 继续是 Work proof surface |
| 16 | `16-control-plane-agent-spec.md` | AMEND VIA PERSONAL DOCS | 增加 Profile/Instance/Installation/Binding readiness、Conversation/Work links、capability source/freshness |
| 17 | `17-control-plane-provider-spec.md` | AMEND VIA PERSONAL DOCS | 从 egress governance 扩展到 plan/account/auth/entitlement/model/binding/usage/budget/cost truth taxonomy |
| 18 | `18-control-plane-resource-spec.md` | SUPERSEDE AFTER ADR | `Library` 是 navigation grouping，不是 generic Resource schema；Knowledge/Memory/Skill/Tool 各保留生命周期；Context 是 scoped assembly |
| 19 | `19-control-plane-activity-spec.md` | KEEP + AMEND | 保留 evidence stream；Conversation import/export、Context diff 只在 durable event 存在时进入 Activity |
| 20 | `20-control-plane-system-spec.md` | KEEP + AMEND | 增加 local storage、privacy、history/index deletion、desktop update、shell diagnostics |
| 21 | `21-control-plane-command-layer.md` | KEEP + AMEND | command surface 加 Continue、Conversation、Library、Usage；unsupported action 不显示为可执行 |
| 22 | `22-control-plane-state-system.md` | KEEP | Empty/partial/stale/permission/error/recovery 继续是跨产品 contract |
| 23 | `23-control-plane-component-spec.md` | AMEND VIA PERSONAL DOCS | 增加 economic fact、Binding readiness、Library family、Conversation/Context、Continuation Checkpoint anatomy |
| 24 | `24-control-plane-visual-direction.md` | KEEP + AMEND | 延续 calm/precise；Personal 更 spacious，运营列表保持高效；拒绝 decorative glassmorphism/card wall |
| 25 | `25-control-plane-ux-review.md` | RETAIN AS EVIDENCE | 原 UX finding 保留；Desktop 1.0 需按三 persona 和 Continuation scenarios 重跑 |
| 26 | `26-real-repository-map.md` | STALE IMPLEMENTATION SNAPSHOT | 保留 two-repo evidence；当前 repository facts 由 `11`、PROGRESS、Git 状态更新 |
| 27 | `27-real-webui-architecture.md` | STALE IMPLEMENTATION SNAPSHOT | 仍证明 external client boundary；native shell 与 local conversation/index 尚未实现 |
| 28 | `28-real-api-contract-map.md` | STALE IMPLEMENTATION SNAPSHOT | 保留当时 route map；Conversation、Knowledge、rich Context、desktop shell gaps 由 `05` 重新登记 |
| 29 | `29-real-control-plane-capability-matrix.md` | STALE IMPLEMENTATION SNAPSHOT | 不把旧 implemented/HTTP/tested 结果外推；新 capability matrix 必须逐 revision 更新 |
| 30 | `30-work-task-run-reality.md` | KEEP / RETAIN AS EVIDENCE | Work/Task/Run honesty 是 D13 与 future Continuation 的核心；Conversation 不等于 Run/Task |
| 31 | `31-agent-reality-map.md` | KEEP + AMEND | 保留 Agent identity distinctions；未来加入 Conversation session 与 adapter qualification |
| 32 | `32-provider-session-security-map.md` | KEEP | Secret/session/provider boundaries 继续约束 subscription/history/import UX |
| 33 | `33-event-activity-evidence-map.md` | KEEP + AMEND | 保留 event coverage honesty；新增 domain 不能凭 UI history 冒充 audit/evidence |
| 34 | `34-webui-current-state-audit.md` | STALE IMPLEMENTATION SNAPSHOT | D11–D13 已改变 Home/Work 状态；只作 dated audit |
| 35 | `35-design-to-code-traceability.md` | STALE IMPLEMENTATION SNAPSHOT | 新 candidate docs 无 code trace；canonicalization 后才重建 traceability |
| 36 | `36-refactor-vs-rewrite.md` | KEEP | 继续支持增量 refactor、复用 existing Web client，不做 greenfield rewrite |
| 37 | `37-backend-dependency-matrix.md` | STALE IMPLEMENTATION SNAPSHOT | 作为 gap evidence；Conversation/Knowledge/OSS adapters 需新增 BD rows 后才能实施 |
| 38 | `38-phase2-design-challenges.md` | RETAIN AS EVIDENCE | design-vs-reality discipline 继续适用；不把 owner scope 当 implemented |
| 39 | `39-control-plane-implementation-waves.md` | KEEP / ACTIVE D13 INPUT | 当前 D13 依赖；Desktop 1.0 waves 只在 `10` 作为 candidate，不能回写 |
| 40 | `40-phase3-first-slice.md` | RETAIN AS EVIDENCE | 原 first slice 已被当前 D11–D13 执行事实推进；不能变成 Desktop 1.0 first slice |
| 41 | `41-implementation-readiness.md` | STALE IMPLEMENTATION SNAPSHOT | 保留旧 readiness；Desktop 1.0 新 Gate/阻塞在 `10`，未获 implementation authorization |

## 4. Personal Desktop 1.0 candidate deltas

### 4.1 Product center

```text
Ready → Continue → Review → Work → Verify → Retain
```

- Home 推荐 continuation，不展示模块广告或 vanity KPI。
- Conversation 是可恢复的 dialogue/history container，不是 Task、Evidence 或 Memory。
- Work 保留 daemon-governed Task/Effect/Evidence authority。
- Library 统一导航 Knowledge、Memory、Skills、Tools，但不统一 schema/lifecycle。
- Context 是某 Conversation/Work 当前使用的 versioned scoped view。
- Usage 属于 Provider economic truth，但有 global deep link。

### 4.2 IA delta

```text
Home
Work
Conversations
Agents
Library
  ├─ Knowledge
  ├─ Memory
  ├─ Skills
  └─ Tools
Providers
  ├─ Accounts / Plans / Entitlements
  ├─ Models / Bindings
  └─ Usage / Cost
Activity
────────
System
```

Bindings 是 relation，不是 top-level navigation。System 是 utility footer，不是 primary product
space。

### 4.3 Product-form delta

- Windows-first native shell hosts existing Web client。
- Loopback HTTP/SSE remains product service channel。
- Native IPC 只承担 window/deep-link/tray/update/OS picker 等 allowlisted integration。
- Tauri 是 conditionally preferred candidate，需 fixed security/package/accessibility spike。
- Desktop shell、WebView、OSS adapter 都不能写 authority state。

### 4.4 Visual delta

- Personal：calm、spacious、precise、local。
- macOS-like discipline 指 hierarchy、restraint、depth、direct feedback，不是 copied chrome。
- Windows 使用 system conventions、Segoe UI Variable、Ctrl shortcut、high contrast。
- Cards 仅 Home/onboarding/readiness；inventory/comparison 使用 list/table/master-detail。
- Continuation Checkpoint 是 signature interaction。

### 4.5 Scope delta

Owner 把 Provider/subscription、Binding、Knowledge、Memory、Skills、Tools、token usage、Context、
Conversation history 全部列为 Desktop 1.0 priorities。候选文档使用 P0/P1/P2 表示同一
Desktop 1.0 candidate 内的交付深度，不表示这些能力已实现或已进入 Linux `1.0.0`。

## 5. What remains valid without change

以下 baseline principles 保持：

- daemon-only authority；
- candidate/observation 不等于 completion；
- Task、Effect、Evidence、Verification 分离；
- SecretStore-only credentials；
- Provider cost/source honesty；
- no fake action or lifecycle；
- master/detail 与 evidence layering；
- explicit empty/loading/partial/stale/permission/error/recovery；
- A8 unknown-worktree protection；
- real caller + supported validation；
- no Gate/release/Profile promotion from design docs。

## 6. Canonical conflict

**FACT**：Accepted
[`ADR-0036`](../adr/0036-personal-linux-1-0-and-official-pi-acquisition.md)和
[`PERSONAL-DEVELOPMENT-PLAN.md`](../plan/PERSONAL-DEVELOPMENT-PLAN.md)仍规定：

- `GMVP-LINUX` 是 Personal `1.0.0` Gate；
- Linux x86_64 是 1.0 product platform；
- Pi 是唯一 1.0 product-qualified Agent；
- Web UI、Windows installer、MCP/dynamic Tool、non-Pi adapters 为 post-1.0；
- ADR-0037 仅把 minimum Memory/Context/Skill 等资源 slice 纳入 Linux 1.0。

因此，候选文档只能使用：

> **Personal Desktop 1.0 candidate**

不得声称它已取代 Linux `1.0.0`、改变 GMVP-LINUX、授权 Windows/macOS release 或扩大
Profile/Gate claim。

## 7. D13 dependencies

P7-T05/D13 当前负责 Control Plane W4 Work inventory + governed Task creation，并消费：

- `14-control-plane-work-spec.md`
- `39-control-plane-implementation-waves.md`
- active client/kernel branches and Draft PRs

本轮不编辑 baseline 的原因：

1. active lease 已拥有 D13 implementation/governance coordination；
2. 新 Desktop scope 尚无 accepted ADR/formal task；
3. 同时修改 baseline 会使 D13 acceptance target 漂移；
4. untracked design baseline 需要 ownership/canonicalization decision；
5. candidate delta 可以独立评审，不必覆盖进行中的 Work delivery。

## 8. Safe future canonicalization sequence

只有按以下顺序：

1. **Ownership resolution**：P7-T05/D13 完整收口或明确 transfer；相关 worktree clean。
2. **Product-semantic ADR**：决定 Linux `1.0.0` 与 Desktop candidate 的命名、版本轴、release
   relationship、platform、scope。
3. **Canonical product/architecture scope**：更新 `docs/product/personal/` 与
   `docs/architecture/personal/`，不是由 untracked design baseline 直接拥有。
4. **Delta review**：逐条审查本文件 01–41 disposition，决定 KEEP/AMEND/SUPERSEDE。
5. **Design baseline index**：在 `docs/design/` 增加 canonical delta/index，而不是覆盖旧证据。
6. **Formal plan rebaseline**：注册 task、dependencies、acceptance、validation environment、
   Gate/non-claim。
7. **Lane-CTR**：只对真实 public contract/API changes 修改 schema/bindings/vectors。
8. **Implementation lease**：领取 exact writable paths，先 failure-first，再最小垂直 slice。
9. **Handbook routing**：按 source map 同步双语用户/开发者说明。
10. **Release/Gate**：只消费 exact revision 和 preregistered evidence。

## 9. Future baseline refactor guidance

若上述 sequence 获批：

- 不删除 01–41；保留为 dated baseline/evidence。
- 新增一份 baseline index，明确 “2026-08-24 Control Plane baseline”。
- 对 26–41 as-built audits 增加 revision/date/claim ceiling。
- 将当前事实从设计文档移回 canonical snapshot/code-derived inventory。
- 产品变化通过 accepted ADR + canonical product docs；implementation waves 只消费已接受范围。
- 避免把 Personal Desktop、Enterprise、Linux release 写成一个同步 backlog。

## 10. Non-claims

- `docs/design/**` 本轮完全未修改。
- 本文件没有接受 ADR、PRD、contract、formal task 或 release scope。
- 没有 UI/code/dependency/OSS migration。
- 没有 product test、usability test、Gate、Profile 或 release evidence。
- D13 status 仍只由 PROGRESS、active lease、PR/HEAD 事实拥有。

