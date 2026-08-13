# P2-T18 本地令牌 CSPRNG 修正 — 增量验证报告

- Task: `P2-T18`
- Slice queue: `P2-T18/D01` → `P2-T18/D02`
- Branch: `personal/P2-T18-local-token-csprng`
- Base: `origin/main@326f97728ab6aaaacceaedd2156d953231b32e01`
- Lease: `lease/personal/P2-T18/local-token-csprng`
- Classification: `implementation-only` security corrective；既有 ADR-0019/0022
  安全语义不变。
- Evidence ceiling: ordinary implementation evidence only；不产生 Gate、release、
  Profile、B01 或 campaign 结论。
- Secret discipline: 本报告、测试输出与证据只记录控制流和计数，绝不记录 token bytes。

本文件是本任务唯一 running validation report。每个完成的验证单元在下一单元开始前追加；
已记录条目不改写，必要时以 superseding entry 更正。

## 增量条目

### V00 — 基线令牌来源静态检查

- Revision: `326f97728ab6aaaacceaedd2156d953231b32e01`
- Environment/instrument: isolated Windows worktree；针对
  `apps/kernel-server/src/personal/auth.rs` 的定向 source inspection
- Started/retained denominator: 1/1 source unit
- Outcome: `fail`（预期的 failure-first 基线）
- Result: 生产 `generate_opaque_token` 明确引用 `DefaultHasher`、进程 ID、`Instant`
  timing 与自制 `random_u64`，且没有 OS CSPRNG 调用。已加入
  `p2_t18_local_token_csprng::production_token_generation_uses_only_the_os_csprng`
  静态负例；断言文本不包含任何 token material。
- Disposition: 保留该失败测试并在下一实现 revision 使其通过；禁止 PID/time/hash fallback。

### V01 — failure-first Rust 测试执行路由

- Revision: 首个 failure-first commit 待生成
- Environment/instrument: `DEV-WIN-GNU-01`
- Started/retained denominator: 0/1
- Outcome: `not-run`
- Reason: 本地主机是已登记不支持 Rust linking 的 Windows GNU 环境；未违反
  `RUST-LINK-DEV-WIN-GNU-01` 重跑 `cargo test`。
- Disposition: 首个 commit 推送后由 exact-revision `DEV-LINUX-NATIVE-01` 观察预期失败；
  最终候选同时由 GitHub Ubuntu/Windows CI 验证。

### V02 — failure-first Linux 首次调用

- Revision: `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-fail-bc415f094d6` 独立 clone
- Started/retained denominator: 0/1 test
- Outcome: `instrument_error`
- Result: PowerShell here-string 的 CRLF 被原样送入 `bash -s`，目标名尾部多出 carriage
  return；Cargo 在启动测试前以“no test target”退出。该结果不是产品测试失败。
- Disposition: 保留 clone 与 exact detached revision；重试时在远端先移除输入的 CR，
  只运行同一 test target。

### V03 — failure-first Linux 预期失败

- Revision: `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-fail-bc415f094d6` exact detached clone；Rust test profile
- Started/retained denominator: 1/1 test
- Outcome: `fail`（预期）
- Result: `production_token_generation_uses_only_the_os_csprng` 0/1，断言生产路径
  尚未调用 OS CSPRNG；测试与输出均未打印 token bytes。定向 source inspection 同时保留
  `DefaultHasher`、PID、timing 与 `random_u64` 的缺陷归因。
- Disposition: failure-first 已在支持环境实际观察；开始 D01 实现并保留该负例。

### V04 — 本地 Rust 格式

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: `DEV-WIN-GNU-01`；`cargo fmt --all -- --check`
- Started/retained denominator: 1/1 formatting unit
- Outcome: `pass`
- Result: 全 workspace Rust 格式检查通过；该命令不编译、不链接。
- Disposition: 继续本地静态 source guard；Rust 行为验证只走 supported Linux/CI。

### V05 — 生产令牌来源静态 guard

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: workspace source search；与 integration negative 相同的 production/
  test-module 边界
- Started/retained denominator: 5/5 forbidden/required markers
- Outcome: `pass`
- Result: 生产 auth source 中 `DefaultHasher`、`random_u64`、
  `Instant::now().hash`、`elapsed().as_nanos` 均为 0 match；唯一生产熵入口为
  `getrandom::fill`。`std::process::id()` 只存在于 `cfg(test)` 临时路径命名，integration
  negative 会排除 test module 后检查生产代码。
- Disposition: 静态防回归满足；下一步以 supported Linux 编译并执行私有 seam 与既有 auth
  regressions。

### V06 — handbook 指纹生成首次调用

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node 22；
  `node tools/src/fill-handbook-fingerprints.mjs`
- Started/retained denominator: 0/1 generation unit
- Outcome: `not-run`
- Reason: 当前 sibling worktree 尚无 `node_modules`，入口在执行前因缺少 `yaml` package
  退出；没有 handbook 文件被生成器修改。
- Disposition: 按 lockfile 执行 `pnpm install --frozen-lockfile`，随后重试同一生成器。

### V07 — handbook 指纹生成恢复

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node 22；lockfile 安装后重跑
  `fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 affected locale pages
- Outcome: `pass`
- Result: en/zh-CN 的 `user.security-boundaries`、`user.limitations` 与 source-map 同时命中的
  `dev.daemon-http-surface` 共 6 页来源指纹已刷新；内容页仍由本任务显式维护，生成器未写
  token material。
- Disposition: 将额外映射的两份 daemon 页面精确加入 lease，随后运行 handbook check。

### V08 — handbook 全门禁

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node/pnpm；
  `pnpm run check:handbook`
- Started/retained denominator: 54 documents × 2 locales；9 generated families
- Outcome: `pass`
- Result: coverage、link、fingerprint、status 与 secret checks 全部通过。
- Disposition: 双语文档与来源指纹闭合；继续 repository consistency。

### V09 — repository consistency 首轮

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:consistency`
- Started/retained denominator: 1/1 consistency run
- Outcome: `fail`
- Result: 5 项均为 P2-T18 尚未同步到 `PROGRESS.md` Current snapshot：总计、D01/D02
  状态、active lease 引用及当前 in-progress slice；无代码/合同漂移项。
- Disposition: 对 Current snapshot 做仅限 P2-T18 的窄幅同步，不改 campaign/B01/P2-T14
  状态，然后重跑。

### V10 — repository consistency 恢复

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:consistency`
- Started/retained denominator: 1/1 consistency run
- Outcome: `pass`
- Result: 275 requirements、55 error codes、74 schemas、89 vectors，以及 Personal
  plan/Gate、trace、task/slice/lease 和 command/environment routing 全部一致。
- Disposition: 本地静态收敛；执行 staged docs-sync/diff 检查后生成 pushed implementation
  candidate。

### V11 — generated handbook byte gate

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node；`generate-handbook.mjs --check`
- Started/retained denominator: 18/18 generated pages
- Outcome: `pass`
- Result: 18 页 byte-identical；本修正没有手改 generated 页面。
- Disposition: 继续 staged docs-sync gate。

### V12 — diff whitespace

- Revision: working tree after `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Git；`git diff --check`
- Started/retained denominator: 1/1 branch diff
- Outcome: `pass`
- Result: 无 whitespace error。
- Disposition: stage 精确 lease paths 并运行 docs-sync gate。

### V13 — staged docs-sync gate

- Revision: staged implementation candidate after
  `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 mapped change set
- Outcome: `pass`
- Result: `daemon-http` 与 handbook-self 路由正确；54 × 2 handbook checks 和 18 页
  generated byte gate 一并通过。安全边界/已知限制/daemon 页面双语同步，无
  `DOCS_IMPACT_NONE` 逃逸。
- Disposition: 生成 immutable implementation checkpoint，随后只在该 pushed revision
  上运行 Rust。

### V14 — 复审后 Rust 格式

- Revision: staged/working implementation candidate after
  `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: `DEV-WIN-GNU-01`；`cargo fmt --all -- --check`
- Started/retained denominator: 1/1 formatting unit
- Outcome: `pass`
- Result: 增补 partial-error buffer clearing 与跨调用重复 session token 负例后，workspace
  Rust 格式仍通过。
- Disposition: 重新 stage 并复跑 docs-sync gate 后 commit。

### V15 — 最终 staged docs-sync 首轮

- Revision: staged/working implementation candidate after
  `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 mapped change set
- Outcome: `fail`
- Result: 复审新增的 auth source 行使 6 个已同步页面的来源 digest 再次变化，HB008
  正确拒绝旧 fingerprint；其余路由未报告问题。
- Disposition: 重跑 fingerprint filler 并再次执行 staged gate。

### V16 — 最终 fingerprint 恢复

- Revision: working implementation candidate after
  `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node；`fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 affected locale pages
- Outcome: `pass`
- Result: 复审后 source digest 已同步到 en/zh-CN 的 daemon、安全边界与已知限制页面。
- Disposition: stage 6 页与 report，重跑 docs-sync gate。

### V17 — 最终 staged docs-sync 恢复

- Revision: final staged implementation candidate after
  `bc415f094d655c0dc30bd205c3e60dced5ec0fc8`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 mapped change set
- Outcome: `pass`
- Result: 54 × 2 handbook、6 个最终 source fingerprint 与 18 个 generated 页面全部通过；
  无 docs-sync escape。
- Disposition: commit/push immutable candidate 并转入 supported Rust validation。
