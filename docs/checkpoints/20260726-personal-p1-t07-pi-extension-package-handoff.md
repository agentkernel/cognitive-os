# 20260726 Personal P1-T07 — Pi Extension 包（第一个原子部分）Handoff

## 1. Session snapshot

- 日期：2026-07-26。
- 分支：`lane/personal-p1-t07-pi-package`（自 `origin/main@e8990f4` 建立）。
- 任务：**P1-T07 已从 `not-started` 认领为 `in-progress`**，`development_track: experimental-local-only`。
  本批交付 Extension 包这一半，**P1-T07 未完成**。
- 非声明边界：不构成 G0、B01-B12、C0/C1、Profile 或 release 声明；WSL2 guest 上的
  本机执行是 `tested-local`，不是 Linux-native evidence。

## 2. 交付内容

新增 `packages/pi-cognitiveos/`（`@cognitiveos/pi-cognitiveos`），把"Pi 是 Shell，
不是 authority"从纸面约束变成进程内可执行的拒绝。

| Pi hook | 行为 |
|---|---|
| `project_trust` | 恒返回 `{ trusted: "no" }` |
| `tool_call` | **默认拒绝**：`bash`/`edit`/`write` 给 mutating 理由，其余工具给 ungoverned 理由 |
| `session_start` | 读 `GET /personal/status` 并展示真实投影；first conversation 被阻断时告警 |
| `/cognitive-status` | 按需打印同一批 daemon 事实 |

设计要点与依据：

1. **工具默认拒绝而不是仅拒三个 mutating 内建。** Extension 没有 catalog、没有
   capability、没有 Effect 协议，无法授权任何工具；ADR-0026 的"未知或不可分级的
   operation 一律 Tier 2"从另一侧给出同一结论。`READ_ONLY_TOOL_ALLOWLIST` 显式为空，
   是将来放行某个工具时唯一需要评审的地方。受治理工具执行归 P2-T05/P2-T06 的 daemon 侧。
2. **不 vendor Pi（ADR-0025）。** 包内 `src/pi-api.ts` 是固定 API 子集的结构镜像，
   不 `import` 任何 `@earendil-works/*`，Pi 不进入 `pnpm-lock.yaml`（有断言测试）。
   `src/pin.ts` 与 Rust `PiCompatibilityPin::expected()` 逐字段 drift 校验。
3. **发现路径与 `cognitive` 一致。** 只读两个既有本地文件
   （`daemon-endpoint.json`、`local-bootstrap.secret`）；非 loopback endpoint 拒绝；
   `XDG_RUNTIME_DIR` 缺失 fail-closed（不为 0600 secret 发明兜底位置）。
   bootstrap secret 只用于一次 `POST /local/session`，不落盘、不展示、不进错误消息。
4. **显式失败，绝不合成 ready。** daemon 不可达、bearer 被拒、投影畸形分别映射到
   稳定错误码；daemon 重启导致 bearer 失效时**恰好重发一次**，第二次仍被拒即显式失败。
   任何失败路径的状态行都不会渲染成 ready。

## 3. 本窗口真实执行的验证（`tested-local`，`windows_wsl2_linux_guest`）

| 命令 | 结果 |
|---|---|
| `pnpm --filter @cognitiveos/pi-cognitiveos build` | exit 0（`tsc` strict + `noUncheckedIndexedAccess` + `exactOptionalPropertyTypes`） |
| `pnpm --filter @cognitiveos/pi-cognitiveos test` | **45 passed / 0 failed** |
| `pnpm -r build` | 全部 Done（6 个 workspace 项目） |
| `pnpm -r test` | pi-cognitiveos 45 / contracts-ts 39 / sdk-ts 69 / agent-shell 13 / repo-tools 4，**fail 0** |
| `pnpm run check:consistency` | OK |
| `node tools/src/gen-matrix.mjs --check` | matrix up to date |
| `git diff --check` | 通过 |

测试覆盖的负例（节选）：工具名大小写/空白变形不能绕过denylist；未知工具一律拒绝；
`XDG_RUNTIME_DIR` 缺失/空/空白三种形态都 fail-closed；endpoint 文档的 8 种损坏形态
（非 JSON、非对象、版本不符、surface 不符、缺字段、空字段、非 loopback、外部主机）全部拒绝；
bootstrap secret 错误时返回 `LOCAL_BOOTSTRAP_MISMATCH` 且错误消息不含该 secret；
端口关闭时报 unreachable 而不是降级读；投影的 9 种畸形形态全部变协议错误；
`authority_side_effects: true` 的投影被拒；任何 UI 表面都不含 bootstrap secret 或 session token。

Rust 侧本批无改动；`cargo` 结果沿用同日更早的 `cargo test --workspace --locked`
**358 passed / 0 failed** 与 clippy/fmt 通过，并由本 PR 的 CI 重新执行。

## 4. 未完成（P1-T07 仍 `in-progress`）

1. **daemon 侧 provider proxy 路由与生产 `ProviderTransport`。** 仓库当前
   **没有**任何 provider proxy 端点，`ProviderTransport` 的唯一实现是测试用
   `MockTransport`，且 `Cargo.lock` 内**没有** HTTP client 或 TLS 依赖。落地需要
   一个显式决策（引入 HTTP/TLS 依赖，或走 `linux_secret_tool.rs` 那样的子进程方案），
   属供应链决策，必须在实施批次中记录理由。另有一个硬约束：Personal front door
   单请求单连接、无 SSE，**流式补全在当前表面不可表达**，proxy 批次必须明确取舍。
2. **`readiness.rs` 的 `pi` 组件翻转。** 目前硬编码 `not_configured`，因此
   `first_conversation_ready` 结构上永远为 `false`。翻转必须遵守 ADR-0023：
   不改动既有聚合规则，只把该组件换成真实检查，并同步 doctor guidance 文案。
3. **真实 Pi 进程加载证据。** 依赖 P0-T06 的 `extension-load` 动词，需 Linux-native
   主机，仍 **not-run**。本包尚未被任何真实 Pi 进程加载过。

## 5. Owner 待办一次性清单（与上一 handoff 相同，未新增）

1. `hal9000@192.168.1.2` 的 SSH 认证——Linux-native 主机是 P0-T06 收尾与本包真实加载
   证据的唯一路径。
2. 该主机 native Secret Store 中已配置的 DeepSeek Provider key（仅经 ADR-0018 例外路径）。
3. 干净 Linux VM（P1-T09 / B01 的 20 次 clean-run）。

## 6. 下一步

1. 本 PR CI 双平台绿后合并 `main`。
2. 继续 P1-T07 第二个原子部分：先做 **`readiness.rs` 的 `pi` 组件翻转**（纯 Rust、
   无新依赖、可本地完整测试），再单独处理 provider proxy 的传输依赖决策——把决策
   风险与可立即交付的部分分开，避免一个未决依赖阻塞整个任务。
3. provider proxy 决策落地前，P1-T08（可检查 Linux bundle installer）按 §12.1 是
   可并行的备选任务。

## 7. 禁止重复尝试

- 不要给 `packages/pi-cognitiveos` 添加 `@earendil-works/*` 依赖：`pin.test.ts` 会失败，
  且违反 ADR-0025 的不 vendor 决定。
- 不要在 Extension 里实现审批链或逐动作确认：ADR-0026 规定任务准入预览是唯一默认
  人工授权点，Extension 不是授权者。
- 不要把 `READ_ONLY_TOOL_ALLOWLIST` 当作"顺手补全"的地方——每加一项都要有台账记录的决策。
