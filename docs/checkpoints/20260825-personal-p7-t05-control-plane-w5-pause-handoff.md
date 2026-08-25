# P7-T05 Control Plane — D13 closure and D14/W5 pause handoff

- 状态：handoff（owner-directed pause for repository reorganization）；类别 checkpoint
- 日期：2026-08-25
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- 当前 Slice：`P7-T05/D14` — Control Plane W5 Work detail + composed Run timeline
- Slice 状态：`in-progress`（实现完成并已推送；**rendered review 未执行**）
- 停止原因：owner 指示收口并让出仓库，另有窗口要整理仓库。不是外部阻塞，不是任务完成。

---

## 1. 为什么存在这份 handoff

owner 在本会话中先要求「W5 完成后停止，不提交、不推送」，随后改变范围，要求
「写一份 handoff 文档然后收口并关闭分支，有其他窗口要整理仓库」。因此 W5 的
coherent 改动已按 `CHECKPOINT-DELIVERY-01` 提交并推送（不得留下未提交的 coherent
任务改动），而 D14 因缺少 rendered review 仍保持 `in-progress`。

本文件是 `TASK-ATOMIC-DELIVERY-01` 下的**唯一**一次 handoff：它对应
「explicit pause/transfer」触发条件，不是 Slice 汇报，也不是任务收口记录。

---

## 2. 精确恢复坐标

| 项 | 值 |
|---|---|
| Kernel repo | `D:\agent-kernel`（`agentkernel/cognitive-os`） |
| Kernel branch | `personal/P7-T05-control-plane-foundation` |
| Kernel HEAD | `95fe07f7de0956ff53ab48c64c3c620a425683f1` |
| Kernel upstream | 同步（`origin/personal/P7-T05-control-plane-foundation`） |
| Kernel PR | [#272](https://github.com/agentkernel/cognitive-os/pull/272) — **Draft**，head `95fe07f7` |
| Kernel required CI | run `32809902836` at `95fe07f7` **green**（`resolve validation route`、`verify (ubuntu-latest)`、`verify (windows-latest)`、`required-ci`） |
| Clients repo | `D:\cognitiveos-clients`（`agentkernel/cognitiveos-clients`） |
| Clients branch | `personal/P7-T05-control-plane-foundation` |
| Clients HEAD | `d7f68164abebb3c88ae8d210433b694bdebf033e` |
| Clients upstream | 同步 |
| Clients PR | [#7](https://github.com/agentkernel/cognitiveos-clients/pull/7) — **Draft**，head `d7f6816` |
| Task lease | `lease/personal/P7-T05/control-plane-foundation` — 本次 handoff 中**关闭**（owner-directed pause，见 §7） |

### Clients commit 链（本任务）

```
8b4c516  W1 Foundation                         (D11)
c4b699f  W2 Providers                          (D11)
724af21  W2 rendered-review corrections        (D11)
001595b  D6 action receipts survive refresh    (D11)
4c674b1  W3 Home attention surface             (D12)
2944945  W3 contrast correction                (D12)
61db8cf  W4 Work inventory + governed creation (D13)
7a7c159  W4 rendered-review corrections        (D13)
d7f6816  W5 Work detail + Run timeline         (D14, review 未跑)
```

没有任何 commit 被 amend，没有 force push。

---

## 3. 已完成（D13 收口，accepted）

`P7-T05/D13` Control Plane W4 已在本会话正式收口为 `done`。

实现在 clients `61db8cf` + `7a7c159`。验证于 `7a7c159`：`pnpm test` **162/162**、
`pnpm build` **pass**、`git diff --check` clean；九个受保护逻辑模块、`package.json`
与 `pnpm-lock.yaml` 字节未变。

**Rendered review 通过**：headless Chrome 151.0.7922.174 经 CDP 对推送树的干净重建
（JS `sha256:56da85abce977d66c…`，CSS `sha256:ed161d1606437e1e3…`）执行，
**33/33 cells、310/310 assertions 通过**，每个 cell `overflow=0`、`clipped=0`、
`consoleErrors=0`，且**屏内与屏外 contrast findings 均为 0**——本分支不再携带任何
accepted contrast/focus finding。每个 cell 另由 fixture server 自身的请求日志
（非 client spy）在服务端证明未调用 W4 路由集之外的任何路由。

Review 发现并在 `7a7c159` 修复 4 个真实缺陷，无一作为 accepted 保留：

| 缺陷 | 实测 | 成因 |
|---|---|---|
| 非 submit 的 primary 控件丢失 accent ink | 3.89:1 light / 1.78:1 dark | `.cp-app button`(0,1,1) 压过 `.cp-button--primary`(0,1,0)；W3 只桥接了 `button[type=submit]` |
| button 形状的 link 把 accent-as-text 画在 accent-as-fill 上 | 1.28:1 | W3 的 `--cp-link` 分离首次触达 `<a class="cp-button--primary">` |
| 选中行内 digest chip 的 tint 叠加成第三种更深色 `#d3ddee` | 4.37:1 / 4.26:1 | `--cp-unknown-tint` 叠在 `--cp-accent-soft` 上 |
| 零行 inventory 声称了它没有的知识 | 读取 pending / failed / 200-stub 时都渲染「knows of no task」 | 一个 empty 分支承担三种不同语义 |

同时修复的 D13 治理缺陷：`PERSONAL-DEVELOPMENT-PLAN.md` 的 D13 行曾被写入路径吞掉
反引号并把后续字符变成 C0 控制字节（`docs/design/14/<U+0003>9`、
`clients <U+0002>944945`、`<U+0009>askDraft.ts`），且整个 acceptance 列丢失 code
span。已恢复为正常 UTF-8/Markdown。三个 plan 文档的控制字符扫描结果为
**除 LF 外 0 个 C0/DEL 字节**，无 BOM，仅 LF 换行。

---

## 4. 已实现但未验收（D14 / W5）

W5 在 clients `d7f6816`，**已推送**。

新增：`data/projections/workDetail.ts`；`views/work/detail/` 下 `WorkDetailPage`、
`WorkHeader`、`SectionNavigator`、`OverviewSection`、`RunTimeline`、
`EffectsSection`、`EvidenceSection`、`IntentContractSection`、`ContextSection`、
`FactsInspector`、`workDetail.test.tsx`。修改：`router.tsx`（`/work/:taskRef`，
排在静态 `/work/new` 之后）、`app.css`、`WorkPage`/`WorkInventory`/`WorkInspector`
（scope/filter/selection 进入 URL，往返无损）、`work.test.tsx`。

响应结构取自 daemon 自身 handler，非假定：`/task/evidence` 确实返回
`lifecycle.transitions` 与 `transitions_truncated`；`/task/effects` 确实返回
`mutation_count`、`fixed_post_state_ref`、`effects_truncated`。

已落实的 honesty 边界：六个连续章节，无 tabs/accordion；Run 为 authority 与
observation 两条结构分离的 lane（实心 vs 空心 marker），observation 永不渲染为状态
迁移；`transitions_truncated` 渲染为前导 bounded 行，`after_version` 跳变渲染为
`no recorded facts` gap 行；不声称 streaming，watch 记为 not attached，detach 不表示
控制 Task/Agent；effects 中 `OUTCOME_UNKNOWN`/`VERIFY_FAILED` 上浮，空集为「未记录
external mutation」而非成功；`completionReading` 是唯一可产出 `completed` 的位置，
要求 verification passed 且 current **并且** 存在 current terminal acceptance；
evidence 404 读作 `No terminal evidence recorded`；未由本会话 admit 的 ref 明确声明
无 chain 并给出
`Previews are ephemeral by design; the admitted contract is the durable record.`；
consumption 的每一类拒绝分别具名（mismatch 是真冲突，不是空 context）；Loop/WIA/
Context assembly 具名 unavailable；未知 ref 给设计化 object-404 且不渲染任何章节。

### 验证结果

| 项 | 结果 |
|---|---|
| `pnpm test` | **192/192 pass**（20 files） |
| `pnpm build` | **pass**（110 modules；CSS 27.42 kB，JS 343.11 kB） |
| `git diff --check` | clean |
| ReadLints | clean |
| **Rendered browser review** | **not-run** |

`not-run` 是本 Slice 未收口的唯一原因。因此对 `d7f6816` **不作**任何 rendered、
contrast、overflow、clipping、console-error 或 keyboard/focus 声明。

---

## 5. 唯一下一动作

在 clients `personal/P7-T05-control-plane-foundation` 上，对精确推送 revision
`d7f6816` 执行 W5 rendered browser review。

harness 可复用：`d:\tmp\cp-review-w4\`（`server.mjs` fixture server、`review.mjs`
CDP driver、`probe.mjs` 页面探针），位于两个 Git 树之外，产物在 `out/`。
Chrome 位于 `C:\Program Files\Google\Chrome\Application\chrome.exe`（151.0.7922.174）。
启动方式：`node server.mjs <dist 绝对路径> 8791`，然后 `node review.mjs`。

复用前必须补充 W4 矩阵未覆盖的 cell：

1. authority/observation 两 lane 在渲染层永不混淆（含 1920/1440/1280/960 × light/dark）；
2. timeline ordering、`after_version` gap 行、`transitions_truncated` bounded 行；
3. effects failure priority、truncation 标记、空集文案；
4. evidence 404、verification pass ≠ completed、非 current verification/acceptance、
   仅 current acceptance 才允许 `completed`；
5. ephemeral preview honesty（有 chain / 无 chain 两种）；
6. context 的 not-found / context-missing / mismatch / not-eligible / unavailable 分支；
7. 未知 task_ref 的 object-404，且不渲染任何 section；
8. section deep link 与返回 Work 时保留 selection/filter；
9. denied / disconnected / 200-stub / error；
10. route whitelist（服务端请求日志证明，o4/o5 之外不得出现其他 family）；
11. keyboard/focus/ARIA；overflow/clipping/console errors/contrast。

harness 已知陷阱（本会话踩过，勿重犯）：

- **cell 间必须真实跨文档重载。** 只改 fragment 的导航是 same-document navigation，
  内存 session 与 projection store 会存活并把上一 cell 的数据泄漏进下一 cell。
  `review.mjs` 现用 `?cell=<n>` 查询串强制重载。
- **alpha 合成必须累积 alpha。** 早期 `blend()` 把结果 alpha 硬编码为 1，把 10% tint
  当作不透明，凭空造出 contrast failure。
- **held-open 的 loading 请求必须在 scenario 切换时 destroy。** 否则 keep-alive
  socket 上的 head-of-line 阻塞会让后续 cell 读到错误 scenario 的响应。
- **Chrome 的网络日志条目（4xx/5xx）不是应用 console error。** 按 `entry.source`
  区分，否则每个负例 cell 都会假红。
- 端口占用：`server.mjs` 不处理 `EADDRINUSE` 会直接退出 1；启动前先确认 8791 空闲。

review 通过后：修复所有发现（新 commit，不 amend）→ 将 D14 标为 `done` 并记录精确
revision/测试数/build/矩阵 → 登记 W6（Agents dossier）→ 领取新 lease → 继续。

---

## 6. 剩余范围（Control Plane redesign 全量）

权威来源 `docs/design/39`。Wave 0–4 已收口（D10–D13），W5 待验收。其后剩 7 个波次：

| 波次 | 内容 | 后端约束 |
|---|---|---|
| W6 | Agents dossier | 生命周期控制被 BD-2 挡住，只能 read + class-C |
| W7 | Resources（Memory/Skills/Tools/Context） | 基本无阻塞，是剩余最大一块 |
| W8 | Activity 七类 evidence 流 | 统一 audit feed 被 BD-5 挡住 |
| W9 | System（readiness/doctor/backup-restore/session/about） | 无 |
| W10 | Command layer（⌘K） | 无 |
| W11 | Watch streaming + refresh policy | 深度被 BD-4 挡住；task watch 的 snapshot 恒空、event ring 进程本地 |
| W12 | Accessibility / QA gate | 无 |

另有两类不在波次列表但属于「这部分做完」的收尾项：

1. **遗留层退休。** `styles.css`（491 行）仍被 `main.tsx` 全局 import，legacy
   `#/tasks` 页（`views/legacy/legacyPages.tsx`，328 行）仍在路由中。W5 **没有**
   迁移它的 watch/observation 诊断能力——迁移 + 退休 styles.css 是一件尚未开始的
   独立工作。
2. **任务级收口。** 两个 PR 均为 Draft；完整 P7-T05 acceptance、ready 翻转、merge、
   lease 关闭、branch 清理均未做。

规模参考：W1–W5 共 99 个文件、15,201 行（含约 3,900 行测试）、192 个测试。

---

## 7. Lease、分支与 owner-owned 内容

### Lease

`lease/personal/P7-T05/control-plane-foundation` 在本次 handoff 中**关闭**，原因是
owner 指示让出仓库给另一窗口整理。关闭**不表示** D14 完成，也不表示 P7-T05 完成。
恢复 D14 时必须重新领取一个 `lease/personal/P7-T05/...` lease。
`PROGRESS.md` 的 Active task lease 行相应写为 `none`。

### 分支

远端两个 task branch 与两个 Draft PR **全部保留**，W5 工作已在 `d7f6816` 上持久化。
本地 clients task branch 已删除并切回 `main`（远端有同一 commit，可随时
`git checkout -b ... origin/...` 恢复）。

**Kernel 工作树刻意留在 task branch 上**，未切回 `main`。原因：
`docs/plan/PARALLEL-LANES.md` 在 branch 与 `origin/main` 之间存在差异，而 owner 的
未提交 discovery closure 行正叠在该文件上；切换分支会危及这份 owner-owned 改动。
整理仓库的窗口若需要 kernel 在 `main` 上，必须**先由 owner 决定如何处置那行未提交
改动**，不得直接 checkout、stash、revert 或 `git add -A`。

### 未合并说明（owner 决定项）

两个 PR 均未 merge，这是刻意的：完整 P7-T05 acceptance 未满足（W6–W12 未做），
且 W5 的 rendered review 未执行。按 Operating Model，task PR 在完整 acceptance 前
必须保持 Draft，禁止 merge。是否要把已验收的 W1–W4 部分单独落地到 `main`，属于
owner 的范围决定，本会话不代为执行。

### 必须保护的 owner-owned 内容（未被本会话触碰）

以下均为 untracked 或未提交，本会话全程未编辑、未暂存、未删除、未混入任何 commit：

- `docs/agent-work-system/`（untracked）
- `docs/design/`（untracked）
- `.cursor/skills/`（untracked）
- `tmp-ff-launch.sh`、`tmp-ff-launch2.sh`、`tmp-ff-restart.sh`、`tmp-fix-proxy.sh`、
  `tmp-ignore.sh`、`tmp-session.sh`（untracked，6 个）
- `docs/plan/PARALLEL-LANES.md` 中 `lease/personal/DISCOVERY-personal-1.0/oss-and-ux-refinement`
  的 discovery closure 行（tracked 文件的未提交 hunk）

隔离方式：`PARALLEL-LANES.md` 的两次治理提交都通过 `git diff` → 过滤为仅 active-lease
hunk → `git apply --cached` 完成；每次提交前均以 `git diff --cached` 证明
`DISCOVERY-personal-1.0` 出现次数为 **0**，提交后该行仍存在于工作树。

---

## 8. Non-claims

- Claim ceiling `hypothesis`；证据为 local/container-class。
- Rendered review 是本地浏览器观察，**不是** product Gate。
- 对 W5（`d7f6816`）不作任何 rendered/contrast/overflow/focus 声明。
- 未产生 Gate、release、Profile、B01、EVAL 或 Agent-benefit 提升。
- 未改动 daemon、contract、negatives、specs、conformance 或 handbook 生成源。
- 未新增依赖；`package.json`、`pnpm-lock.yaml` 字节未变。
- Review harness、fixture、报告与截图全部位于 `d:\tmp\cp-review-w4`，在两个 Git 树
  之外；产品代码中不存在 mock route、fixture 或 backdoor。
