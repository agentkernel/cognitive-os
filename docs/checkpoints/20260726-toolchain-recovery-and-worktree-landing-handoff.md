# 20260726 本机工具链恢复、工作树落盘与客户端仓库拆分收口 Handoff

## 1. Session snapshot

- 日期：2026-07-26。
- 分支：`lane/doc-clients-extraction-and-personal-p0-t06`（自 `origin/main@946bffe` 建立）。
- 性质：**环境恢复 + 既有工作树落盘 + 客户端仓库拆分收口**。本窗口没有把任何
  正式任务从 `not-started`/`in-progress` 推进为 `done`；P0-T06 仍 `in-progress`。
- 非声明边界：本批不构成 G0、B01-B12、C0/C1、Profile 或 release 声明，也不把
  WSL2 guest 结果升级为 Linux-native evidence。

## 2. 关键环境事实变更（本窗口最重要的结果）

此前多个窗口把"本机不可测试"当作既定条件，理由有二：会话沙箱无 shell，以及
Windows GNU `x86_64-w64-mingw32-gcc` linker `exit 121`。本窗口核实：当前会话运行在
**WSL2 Linux guest**（`6.18.33.2-microsoft-standard-WSL2`）内，root 权限、网络可用，
但该 guest 内**没有** Linux-native 的 `cargo`/`node`；`PATH` 上只有无法执行的 Windows
二进制（interop 未注册，`Exec format error`）。

已执行的恢复动作：

1. 注册 `binfmt_misc` 的 `WSLInterop`，使 Windows 侧 `gh.exe` 可用于 git 凭据与
   GitHub API（`gh auth status` → 已登录 `agentkernel`，scopes `repo`/`workflow`）。
2. 安装与 `rust-toolchain.toml` 完全一致的 **Rust 1.97.1**（rustup，含 rustfmt/clippy）
   + `build-essential`（提供 `cc`/`ld`）。
3. 安装 **Node v22.14.0**（`/usr/local/node`）与 **pnpm 10.33.2**（与
   `package.json` 的 `packageManager` 一致）。
4. `~/.bashrc` 持久化 `PATH` 与 `CARGO_TARGET_DIR=/root/cargo-target`
   （把构建产物放在 Linux 文件系统而非 `/mnt/d` drvfs 挂载，避免 9p 慢路径）。
5. `git config --local credential.helper` 指向 `gh.exe auth git-credential`；
   `git push --dry-run` 验证具备推送权限。

**结论：本机现在可以完整执行受支持的测试面。** Windows GNU linker 的非支持基线结论
不变，但它不再等于"本机无法测试"——Linux guest 路径是可用且受支持的。

## 3. 本窗口真实执行的命令与结果（`tested-local`）

全部在 `/mnt/d/agent-kernel`、`CARGO_TARGET_DIR=/root/cargo-target` 下执行，
平台标签 `windows_wsl2_linux_guest`：

| 命令 | 结果 |
|---|---|
| `cargo build --workspace --locked` | exit 0 |
| `cargo test --workspace --locked` | **358 passed / 0 failed / 0 ignored（67 个 suite）**，exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `pnpm install --frozen-lockfile` | exit 0 |
| `pnpm -r build` | 4/5 workspace projects，全部 Done |
| `pnpm -r test` | 全部通过（含 `apps/agent-shell` 13 subtests pass） |
| `pnpm run check:consistency` | OK（273 requirements / 55 error codes / 63 schemas / 85 vectors） |
| `git diff --check` | 通过（仅 CRLF→LF 提示） |

**未执行（保持 `not-run`，不得推断为通过）：**

- `pi-agent-adapter` 新增的 `extension-load` 证据动词——当前 guest 是 WSL2，
  按设计在解析任何 credential 之前就被 fail-closed 拒绝；本机不具备执行资格。
- 任何 Personal Gate（G0、B01-B12）、正式 performance campaign、Linux-native
  sandbox 证据、Windows-native 测试。
- `pnpm run verify:local` 全流程（本窗口未重跑；上一窗口的 L3 结果未被本批复用为新证据）。

## 4. 本窗口交付的两个提交

### 提交 1 — `docs(clients): extract client documentation domain to cognitiveos-clients`

Owner 在本窗口进行中把整个 `clients/` 文档域迁出到独立仓库
[agentkernel/cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)
（保留 subtree 历史；外仓根对应原 `clients/` 目录），并把删除留在 index 里未提交。
本批把它收口：

- 提交 155 个 `clients/**` 文件的删除；
- 跨仓引用统一改为 `https://github.com/agentkernel/cognitiveos-clients/blob/main/<path>`；
- 修复由拆分产生的 **9 条断链**（`docs/checkpoints/20260721-lane-con-clients-phase0-poc-prep-handoff.md`、
  `docs/prompts/console-agent-hub-direct-mode-product-design.md`、`docs/prompts/lane-con.md`；
  只改链接目标，不改正文）；
- `docs/plan/PARALLEL-LANES.md` §2.1 记录：2026-07-20 的 Lane-CON 例外作为历史保留，
  在本仓其适用范围现在只剩兼容 stub，且 `clients/**` 不得重建；
- ADR-0007、CLIENTS-DEC-001 等历史记录**注记而非删除**。

拆分完整性已核验：外仓 `pushed_at=2026-07-26T14:37:55Z`，根目录含
`agent-hub/governance/mobile/pc/plan/prompts/review/shared` 等全部子树；本批引用到的
**12 条外仓路径逐条经 GitHub API 确认存在**。先前工作树中未跟踪的
`clients/review/*`、`clients/plan/development-plan.md`、
`clients/prompts/continuous-development-execution.md` 均已在外仓中，无内容丢失。

### 提交 2 — Personal P0-T06 `extension-load` 证据模式与计划同步

代码：

- `apps/pi-agent-adapter`：新增 `extension-load` 动词。只接受 pinned fixture
  路径与已注册的 `/cognitiveos-p0-t06-status` 命令；以 `--mode rpc` 驱动一次真实 Pi
  子进程，写入 `get_commands`/`get_state`/`prompt` 三条 RPC 请求，30s 超时强杀；
  输出**脱敏且不含原始 stdout/stderr** 的证据记录，固定
  `authority_committed=false`/`effects_created=false`/`task_transitions=0`/
  `capabilities_granted=0`/`classification=uncontained_candidate_only`。
- host 分类从 `cfg!(target_os)` 升级为读取 `/proc/version`、
  `/proc/sys/kernel/osrelease` 与 `WSL_DISTRO_NAME`/`WSL_INTEROP`，
  使 WSL2 guest 与 enabled CI 在解析任何 credential **之前**被拒绝（含单元负例）。
- `crates/cognitive-runtime`：抽出 `GovernanceOverheadSample::documented_builder_sample()`，
  消除 runner 与单元测试各写一份 sample 数值的漂移风险。
- `crates/cognitive-conformance`：runner 现在落盘 schema-shaped
  `performance-report-m6-overhead.json` 并用其真实 digest 填 RC manifest。
- `tools/src/evidence-graph.mjs`（新增）+ `validate-manifest.mjs`：校验 manifest 的
  本地 evidence 图——result 引用的 sha256、performance report 的域分隔 canonical
  digest 与 schema，外部 URI 一律判为不可本地验证。
- `scripts/v01-auto-run.{sh,ps1}`：尊重 `CARGO_TARGET_DIR`；PERF-004 改用完整 Rust
  test path 并解析日志确认真实 `1 passed; 0 failed`（避免 `--exact` 命中 0 个测试仍
  exit 0）；复用 runner 产出的 builder report 而不是内联 heredoc；新增
  `VERIFY-MANIFEST` 步骤；PERF-004 失败时置 `STOPPED`。
- `tools/test/check.test.mjs`：新增 evidence-graph 与"两个编排器共享同一套证据
  防护"的对齐测试。

计划/治理文档：ADR-0026（Personal 低摩擦授权 trust profile，DEC-P-20）落地；
`plan.md` 的 P2-T01..P2-T08 压缩卡按 §11.1 扩写为完整强制字段集（语义零变更）；
`PERSONAL-DEVELOPMENT-PLAN.md`/`personal-trace.yaml`/`PI-AGENT-INTEGRATION-PLAN.md`/
`PERSONAL-SUPPORT-MATRIX.md`/`V01-AUTO-RUN-VERIFY-PERF-PLAN.md`/`PROGRESS.md` 同批对齐；
新增研究评审 `docs/research/20260726-frontier-review-and-environment-perception.md`
与操作提示词 `docs/plan/AUTOPILOT-PROMPT.md`（均 documentation-only）。

## 5. 任务状态（未变更）

| 项 | 状态 |
|---|---|
| P0-T06 | 仍 `in-progress`。缺口未变：隔离合规主机上的真实 Extension session/RPC load evidence（**not-run**） |
| G0 | 仍待 P0-T06 全部验收 |
| P1-T07 及之后 | 仍 `not-started` |
| 进度汇总 | 52 个任务：done 12 / in-progress 1 / not-started 39（未变） |

## 6. Owner 待办一次性清单

1. **`hal9000@192.168.1.2` 的 SSH 认证**（操作侧可登录）。这是当前 critical path 上
   唯一真正的阻塞项：它是已登记的 Linux-native 主机，`extension-load` 动词只有在
   Linux-native 主机上才被允许执行，P0-T06 的收尾验收依赖它。
2. 该 Linux-native 主机 native Secret Store 中**已配置的 DeepSeek Provider key**
   （仅经 ADR-0018 例外路径使用；绝不入库/日志/argv/证据）。
3. **干净 Linux VM 环境**（P1-T09 / B01 的 20 次 clean-run 需要）。

沙箱磁盘空间一项已随本窗口环境恢复消解，不再需要 owner 处理。

## 7. 下一步

1. 本 PR CI 双平台绿后合并 `main`。
2. P0-T06 收尾在 owner 提供第 6 节第 1、2 项之前**保持 blocked**；按 §12.1
   不空转，转入 **P1-T07（CognitiveOS Pi Package/Extension 与 proxy）** 的可本地
   测试部分——其依赖 P0-T06/P1-T03/P1-T04/P1-T05 中，后三者已 `done`，P0-T06 的
   接口面（版本 pin、fixture、RPC parser、host 分类）已可复用。
3. P1-T07 实现时直接使用已扩写的 P2-T02 卡中的 Pi 表面约束。

## 8. 禁止重复尝试

- 不要在 WSL2 guest 上尝试执行 `extension-load` 或 ADR-0018 secret 路径：这是
  设计上的 fail-closed 拒绝，不是可以绕过的配置问题。
- 不要重复扩写 P2 卡（已完成）。
- 不要因为"Windows GNU linker exit 121"就断言本机无法测试——Linux guest 工具链
  已装好，见第 2、3 节。
- 不要重建 `clients/` 目录，也不要新增 `clients/**` 相对链接。
