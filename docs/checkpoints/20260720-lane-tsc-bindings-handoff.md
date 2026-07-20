# 20260720 Lane-TSC Handoff（生成绑定换绑批）

## 1. 本次会话完成

按 `20260720-lane-ctr-gaps-handoff.md` §2 就绪清单，sdk-ts / agent-shell 全量换用 codegen 0.2.0 生成绑定（分支 `lane/tsc`，基线 `3b8461b` = PR #5 merge；提交哈希见 §6）。替换是重构不是重写：全部语义负例测试保持通过，仅字段来源/一处 wire 形状适配（D-015）。逐替换点终态：

| 替换点 | 终态 | 删除的临时机制 |
|---|---|---|
| `errors.ts` | 消费 `errorRegistry`（`ERROR_REGISTRY`/`parseErrorCode`/`REGISTRY_DIGEST` 再导出；§3 重试分类逻辑不变，未注册码经 `parseErrorCode` fail closed） | 手写 55 码钉扎表；`errors.test.ts` 的测试时 errors.yaml 行级解析对读门（平价已归 contracts-ts 生成层测试）；新增生成表全量一致性属性测试 + 55 计数/digest 形状钉扎 |
| `envelope.ts` | 消费 `akpRequestEnvelope`/`akpResultEnvelope` 生成类型 + 生成 const（协议版本类型）；字段名与机器 schema 全对齐（`in_reply_to`/`audit_ref` 原临时命名即 schema 定名，零改名）；schema 条件客户端先行执行：payload⊕payload_ref（`ambiguous-payload`）、error⇒机器错误、**partial⇒continuation（新增）** | 手工 `RequestEnvelope`/`ResultEnvelope`/`ExtensionEntry` 接口（`ResultEnvelope<R>` 仅存为生成形状上的 result 泛型薄封装，结构等同生成类型）；`RESULT_STATUSES` 保留为生成 union 的运行时伴生表（`satisfies` + 覆盖见证编译期钉扎） |
| `views.ts` | 消费 shell 族 6 生成模块（proposal/preview/status-view/watch-subscription/user-intent-record/**control-request**）+ `SCHEMA_DIGESTS` 聚合；命名空间与根类型别名再导出 | 5 个手工接口、`SHELL_SCHEMA_DIGESTS` 手写表、`CancelControl` 临时形状、`SHELL_CONTROL_PROVISIONAL_PIN` 临时 digest；`views.test.ts` 的 digest 重derive 漂移门（平价归 contracts-ts）；保留 ajv 样例校验（护 fixtures 有效性）+ 条件约束负例 + 状态表镜像钉扎 |
| `watch.ts` | 消费 `akpStreamFrame` 生成类型；**行为适配（D-015）**：流错误码从临时 `payload.code` 收口到 `error.code`（common-defs error 形状）；error 帧无机器 error 成员 → `malformed-frame` fail closed（新增负例：旧 payload.code 形状被拒） | 手工 `WatchStreamFrame` 接口；`SHELL_SCHEMA_DIGESTS` 引用改 `watchSubscription.SCHEMA_DIGEST` |
| `apps/agent-shell` | `session.ts` 跟随适配：cancel payload 用生成 `ShellControlRequest`（含 `schema_version` 成员），pin 用 `shellControlRequest.SCHEMA_DIGEST`；preview/submit pin 用 `shellActionProposal.SCHEMA_DIGEST`。语义零变化：detach 不取消、cancel 经 Effect 闭合、展示只来自投影——12 项语义测试原样通过 | 对 `SHELL_CONTROL_PROVISIONAL_PIN`/`CancelControl` 的引用 |
| `fixtures.ts` | 类型改指生成绑定；新增 `sampleControlRequest`（ajv 校验入列） | — |

文档联动：PROGRESS（最后更新行、TS 测试计数 75→79、车道栏、handoff 列表）+ 本 handoff。

## 2. 未完成 / 进行中（M5 前待办与剩余临时机制）

**剩余临时/本地机制（有意保留，非缺口）**：

1. `RESULT_STATUSES` / `SHELL_STATUS_VALUES` 运行时值表：生成层只发射类型（类型运行时擦除），解析器/守卫需要值表；已用 `satisfies` + 覆盖见证钉在生成 union 上（union 扩员 → 编译错误）。若 codegen 未来发射运行时枚举表可再收敛。
2. 信封/帧的手写运行时 shape 检查（`parseRequestEnvelope`/`parseResultEnvelope`/`parseFrame`）：codegen 不发射运行时校验器，客户端 fail-closed 门为手写、schema 条件按 schema 文本逐条对应；完整 schema 校验仍归 authority 与测试层（ajv）。
3. `isShellStatusView` 结构守卫：手写投影注入门（同上，无生成运行时守卫）。
4. `AKP_PAYLOAD_DIGEST_DOMAIN`（akp-payload/0.2）：golden fixture 钉扎的本地常量，非临时物。
5. `HttpSseTransport` 路由/头/SSE 格式：临时值，M5 对齐真 kernel-server。
6. management 操作名反向门：维持（D-016 deferred-to-v0.2）。

**M5 前/中待办（沿上批不变）**：真 HTTP+SSE 集成、`watch.ack` 服务端语义、intent.supersede 修正流命令、CLI 展示层、admin-cli 交互评审（Lane-RUN 属地协作）、M5 出口向量行为执行（归 runner/Lane-CFR）。

## 3. 测试与证据状态

- TS：`pnpm -r build; pnpm -r test` 全绿 = **114 测试**（contracts-ts 33 / tools 2 / **sdk-ts 67**（换绑前 63：errors 8[7 重组+1 属性] / envelope +2[payload⊕ref、partial⇒continuation] / watch +1[D-015 旧形状负例] / views 4 重组）/ **agent-shell 12**（零测试改动语义，仅 frame/pin 来源适配））。
- `pnpm run check:consistency` OK（273 REQ / 55 码 / 60 schema / 81 向量）；规范资产零触碰。
- Rust 侧零触碰。向量口径无变化（静态 30 pass / 51 not-run 归 CTR/CFR 批，本批不新增执行声明）；TS 测试仍为实现测试。
- CI 两 OS 状态见 PR checks（合并前全绿）。

## 4. 未决风险与漂移

- 无新漂移；D-013/D-014/D-015 的消费侧适配已完成（本批），D-016 维持 deferred。
- `ResultEnvelope<R>` 泛型薄封装与生成 `AkpResultEnvelope` 的等同性由类型别名构造保证（`Omit<...,"result"> & {result?: R}`）；若生成结构未来改 `result` 命名需同步一处。
- 生成类型数组成员为可变数组（codegen 形状），sdk 对外仍以 readonly 输入参数收敛；投影缓存冻结（deepFreeze）不受影响。

## 5. 下一步入口

- **Lane-TSC（下次会话）**：M5 gate 开后真集成（`docs/prompts/lane-tsc.md` + `docs/prompts/milestone-m5.md`）；第一个动作：读 `apps/kernel-server` AKP HTTP 路由，替换 `HttpSseTransport` 临时路由并做双 OS 集成测试。
- **Lane-CFR**：M2 行为向量执行批并行中；本批与其无代码交集，PROGRESS/handoff 冲突由后合并者 rebase。
- **Lane-CTR**：无新缺口登记；若 codegen 增加运行时枚举表/校验器发射，§2 第 1/2 项可再收敛。

## 6. 快照

- PROGRESS 已更新：是（最后更新行、TS 计数 79、车道栏换绑批、handoff 列表）。
- 本次提交列表：见 PR（提交 1 = sdk-ts + agent-shell 换绑；提交 2 = 文档联动批；哈希以 git log 为准，本 handoff 写于推送前）。
