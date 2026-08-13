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

### V18 — exact Linux production-source guard

- Revision: `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-fail-bc415f094d6` clean detached clone；
  `cargo test -p kernel-server --locked --test p2_t18_local_token_csprng`
- Started/retained denominator: 1/1 test
- Outcome: `pass`
- Result: production source guard 1/1；OS CSPRNG marker present，PID/time/DefaultHasher/
  handcrafted fallback 均被静态拒绝；输出无 token material。
- Disposition: 运行 auth 私有 seam 与既有 channel/expiry/revoke unit matrix。

### V19 — auth unit matrix 首次调用

- Revision: intended `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: Windows PowerShell → `DEV-LINUX-NATIVE-01` SSH
- Started/retained denominator: 0/auth unit tests
- Outcome: `instrument_error`
- Result: 本地 PowerShell 对嵌套 revision-check 引号进行了错误展开，远端 bash 在 Cargo
  启动前报 unmatched quote；不是产品测试失败。
- Disposition: clone 已由 V18 证明 exact HEAD；重试采用无嵌套 substitution 的远端命令，
  先打印 HEAD 再启动同一 test filter。

### V20 — exact Linux auth unit matrix

- Revision: `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: `DEV-LINUX-NATIVE-01` clean detached clone；
  `cargo test -p kernel-server --locked personal::auth::tests -- --test-threads=1`
- Started/retained denominator: 12/12 auth unit tests
- Outcome: `pass`
- Result: 12/12：failure/zero/short/repeated entropy 无文件，session entropy failure 无
  session，跨调用固定熵重复 token 被拒，128 个 bounded OS-RNG 样本形状/唯一性通过（无统计
  claim），Unix 0600、wrong channel、management boundary、idle expiry、revoke、bootstrap
  mismatch 与 Debug/log serialization redaction 全通过。输出无 token material。
- Disposition: D01/D02 focused matrix 通过；运行完整 kernel-server regressions。

### V21 — exact Linux 完整 kernel-server regressions

- Revision: `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: `DEV-LINUX-NATIVE-01` clean detached clone；
  `cargo test -p kernel-server --locked -- --test-threads=1`
- Started/retained denominator: 220/220 tests（203 unit + 17 integration）
- Outcome: `pass`
- Result: 220/220；P1-T04 real front-door auth/restart、P2-T02 channels/API、readiness、
  Provider、scheduler/Tool/verifier 与新增 P2-T18 guard 全通过。daemon 日志只记录 private
  bootstrap 路径，不记录 token bytes。
- Disposition: 运行 kernel-server all-target Clippy。

### V22 — exact Linux all-target Clippy 首轮

- Revision: `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: `DEV-LINUX-NATIVE-01` clean detached clone；
  `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`
- Started/retained denominator: 1/1 Clippy run
- Outcome: `fail`
- Result: 唯一诊断是新增 integration source guard 的 `.expect(...)` 违反 workspace
  `clippy::expect_used`；生产实现无诊断。
- Disposition: 用 marker 缺失时保持 fail-closed 的 non-panicking `split_once(...).map_or(...)`
  修复测试 helper，重新 fmt/commit/push 后在新 exact revision 重跑全部 required validation。

### V23 — Clippy 修复后本地格式

- Revision: working tree after `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: `DEV-WIN-GNU-01`；`cargo fmt --all -- --check`
- Started/retained denominator: 1/1 formatting unit
- Outcome: `pass`
- Result: source-guard helper 改为 non-panicking fallback 后格式通过；未在 Windows GNU
  编译/链接。
- Disposition: 刷新因 test source 变化而受影响的 handbook fingerprints。

### V24 — Clippy 修复后 fingerprint 核对

- Revision: working tree after `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: Windows Node；`fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 mapped locale pages checked
- Outcome: `pass`
- Result: 0 页需更新；fingerprint 算法追踪声明的 source digest，test-helper-only 改动不改变
  当前 source fingerprints。
- Disposition: 本地 consistency/docs-sync 后提交修复。

### V25 — Clippy 修复后 consistency

- Revision: working tree after `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:consistency`
- Started/retained denominator: 1/1 consistency run
- Outcome: `pass`
- Result: requirements/errors/schemas/vectors/trace/plan/slice/lease 全部一致。
- Disposition: stage test/report 并运行 docs-sync gate。

### V26 — Clippy 修复 staged docs-sync

- Revision: staged test/report correction after
  `72ca18c6164d03df553eaf4aa109b83f036bcdf7`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged change set
- Outcome: `pass`
- Result: 2 个路径均不改变已交付文档语义；先前同步的 mapped source/handbook 仍保持有效。
- Disposition: commit/push 新 exact candidate。

### V27 — Clippy helper 已提交但未推送

- Revision: `0535d3186cd4adb025c12d0e1c8a58c20174f241`
- Environment/instrument: isolated Windows worktree；Git status
- Started/retained denominator: 1/1 commit
- Outcome: `pass`（本地提交）
- Result: Clippy 修复 commit 存在于 `personal/P2-T18-local-token-csprng`，当时相对
  `origin/personal/P2-T18-local-token-csprng` ahead 1，尚未 push。Draft PR #215 仍指向
  `72ca18c6`，required CI 两端 Clippy 失败。
- Disposition: 在推送前补上验收要求的全长零熵块负例，避免再发一个只修 lint 的候选。

### V28 — 全长零熵块负例（本地静态）

- Revision: working tree after `0535d3186cd4adb025c12d0e1c8a58c20174f241`
- Environment/instrument: `DEV-WIN-GNU-01` source inspection；`cargo fmt --all -- --check`
- Started/retained denominator: 1/1 formatting unit plus 3 seam cases authored
- Outcome: `pass`（格式）；Rust 行为 `not-run`
- Result: 新增 `ZeroBlockEntropy` 覆盖全零、仅 token 半区为零、仅 probe 半区为零；三者均在
  创建 bootstrap 文件/目录前 fail closed。本地 GNU 不链接。
- Disposition: 刷新因 `auth.rs` 变化而漂移的 handbook fingerprints。

### V29 — 零熵负例后 fingerprint

- Revision: working tree after `0535d3186cd4adb025c12d0e1c8a58c20174f241`
- Environment/instrument: Windows Node；`fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 mapped locale pages
- Outcome: `pass`
- Result: 6 页 source fingerprint 已刷新（en/zh-CN daemon、安全边界、已知限制）；无 token
  material 写入文档。
- Disposition: 运行 handbook/consistency/generated 门禁。

### V30 — 零熵负例后 handbook / consistency / generated

- Revision: working tree after `0535d3186cd4adb025c12d0e1c8a58c20174f241`
- Environment/instrument: Windows Node/pnpm
- Started/retained denominator: 54×2 handbook + 1 consistency + 18 generated pages
- Outcome: `pass`
- Result: `check:handbook` OK；`check:consistency` OK（含 P2-T18 lease/slice）；
  `generate-handbook --check` 18/18 byte-identical。
- Disposition: stage 精确路径并运行 docs-sync gate，随后 commit/push。

### V31 — 零熵负例 staged docs-sync

- Revision: staged working tree after `0535d3186cd4adb025c12d0e1c8a58c20174f241`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged change set
- Outcome: `pass`
- Result: `auth.rs` 命中 `daemon-http`；6 页 handbook 已与刷新后的 fingerprints 同批暂存；
  54×2 handbook 与 18 generated 页面通过；无 `DOCS_IMPACT_NONE` 逃逸。
- Disposition: commit/push 后在 exact native Linux 重跑 focused tests 与 Clippy。

### V32 — exact Linux production-source guard（Clippy/zero-block 候选）

- Revision: `e65cb0d70ef878d4a92fdc40e3d33656b66cf03b`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-e65cb0d7` disposable clone via local bundle
  (GitHub fetch on the host was too slow); `cargo test -p kernel-server --locked
  --test p2_t18_local_token_csprng`
- Started/retained denominator: 1/1 test
- Outcome: `pass`
- Result: production source guard 1/1；OS CSPRNG marker present，PID/time/DefaultHasher/
  handcrafted fallback 均被静态拒绝；输出无 token material。
- Disposition: 运行含新零熵块负例的 auth unit matrix。

### V33 — exact Linux auth unit matrix（含零熵块）

- Revision: `e65cb0d70ef878d4a92fdc40e3d33656b66cf03b`
- Environment/instrument: `DEV-LINUX-NATIVE-01` 同一 detached clone；
  `cargo test -p kernel-server --locked personal::auth::tests -- --test-threads=1`
- Started/retained denominator: 13/13 auth unit tests（先前 12，新增
  `zero_entropy_block_creates_no_bootstrap_file`）
- Outcome: `pass`
- Result: 13/13：failure/short/repeated/zero-block（全零、token 半区、probe 半区）无文件，
  session entropy failure 无 session，跨调用重复 token 被拒，128 个 bounded OS-RNG 样本
  形状/唯一性通过（无统计 claim），Unix 0600、wrong channel、management boundary、idle
  expiry、revoke、bootstrap mismatch 与 Debug/log redaction 全通过。输出无 token material。
- Disposition: 运行 kernel-server all-target Clippy。

### V34 — exact Linux all-target Clippy（修复后）

- Revision: `e65cb0d70ef878d4a92fdc40e3d33656b66cf03b`
- Environment/instrument: `DEV-LINUX-NATIVE-01` 同一 detached clone；
  `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`
- Started/retained denominator: 1/1 Clippy run
- Outcome: `pass`
- Result: Finished `dev` profile in 30.64s；无 warning/error，包括先前失败的 integration
  source guard。
- Disposition: 等待 required GitHub Ubuntu/Windows CI。

### V35 — required CI Ubuntu

- Revision: `e65cb0d70ef878d4a92fdc40e3d33656b66cf03b`
- Environment/instrument: GitHub Actions `verify (ubuntu-latest)` run
  `31721293941` job `94518641191`
- Started/retained denominator: 1/1 required Ubuntu job
- Outcome: `pass`
- Result: Ubuntu required check SUCCESS on the Clippy/zero-block candidate.
- Disposition: Windows job still `IN_PROGRESS`；完成后追加。

### V36 — required CI Windows（实现候选）

- Revision: `e65cb0d70ef878d4a92fdc40e3d33656b66cf03b`
- Environment/instrument: GitHub Actions `verify (windows-latest)` run
  `31721293941` job `94518641261`
- Started/retained denominator: 1/1 required Windows job
- Outcome: `pass`
- Result: Windows required check SUCCESS，含 Clippy `-D warnings`、workspace tests 与
  handbook/consistency。该 revision 的 Ubuntu 已在 V35 通过，因此实现候选两端 required
  CI 均为 pass。
- Disposition: docs-only ledger commit `68f0f1b0` 触发的后继 run 另记。

### V37 — required CI Ubuntu+Windows（ledger HEAD）

- Revision: `68f0f1b0b944eeabe1ffa2beac764577aa24ff53`
- Environment/instrument: GitHub Actions run `31722030482`
  (`verify (ubuntu-latest)` job `94521093964`；`verify (windows-latest)` job
  `94521093962`)
- Started/retained denominator: 2/2 required jobs
- Outcome: `pass`
- Result: Draft PR #215 HEAD 两端 SUCCESS。ledger commit 不改变产品代码；实现行为以
  `e65cb0d7` 的 native Linux + 两端 CI 为准。
- Disposition: 保持 Draft；不合并，以免覆盖 sibling P2-T14 lease/PROGRESS。无
  Gate/release/Profile/B01 声明。

### V38 — defect-first redaction review 与 failure-first 测试

- Revision: working tree after `f0f05d9e20891a13e77db5b01b33d28824ffa40f`
- Environment/instrument: 全部 `LocalSessionAuthority` bootstrap/session 生成调用点、HTTP
  front door、stderr 路径与 secret-bearing `Debug` implementation 定向审查；
  `cargo fmt --all -- --check`
- Started/retained denominator: 1/1 review unit；1/1 formatting unit
- Outcome: review `fail`（预期的 defect-first finding）；formatting `pass`；新增 Rust 行为
  test `not-run`
- Result: bootstrap 与 session 生产生成均已集中到同一 `getrandom::fill` adapter，HTTP
  transport 只记录请求/响应 byte count，stderr 只记录 bootstrap 文件路径；但
  `SessionIssueRequest` 仍派生默认 `Debug`，会包含 `bootstrap_secret`。已先加入
  `session_issue_request_debug_redacts_bootstrap_secret`，断言失败时不打印 secret bytes；
  尚未修改生产 `Debug`。
- Disposition: 将 failure-first test 作为独立 immutable revision 推送，在
  `DEV-LINUX-NATIVE-01` 精确观察预期失败后，再实现 redacted `Debug`。

### V39 — failure-first staged docs-sync 首轮

- Revision: staged failure-first change after
  `f0f05d9e20891a13e77db5b01b33d28824ffa40f`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged gate
- Outcome: `fail`
- Result: gate 将 `auth.rs` 的 test-module-only 变更保守映射到 `daemon-http`，因本次提交
  未重复暂存已在分支中同步的 handbook 页面而拒绝；生产行为、公开合同及已同步文档语义均
  未改变。
- Disposition: 以
  `DOCS_IMPACT_NONE="Failure-first test only; production behavior and documented auth contract unchanged"`
  明确记录本提交的文档中性原因，重新暂存 report 后重跑同一 gate。

### V40 — failure-first staged docs-sync 文档中性重试

- Revision: staged failure-first change after
  `f0f05d9e20891a13e77db5b01b33d28824ffa40f`
- Environment/instrument: Windows Node；带 V39 精确 `DOCS_IMPACT_NONE` 原因重跑
  `docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged gate
- Outcome: `fail`
- Result: 文档中性声明被接受，但 handbook byte gate 仍发现 6 个 HB008 fingerprint drift；
  `auth.rs` 的新增 test module 会改变映射源 digest。受影响范围仅为已在 lease 中的
  en/zh-CN daemon、安全边界与已知限制页面；页面语义无需改写。
- Disposition: 运行 `fill-handbook-fingerprints.mjs` 刷新 6 页来源指纹，暂存精确页面并重跑
  handbook/docs-sync；提交说明继续保留 V39 的文档中性原因。

### V41 — failure-first handbook fingerprint 刷新

- Revision: working tree after `f0f05d9e20891a13e77db5b01b33d28824ffa40f`
- Environment/instrument: Windows Node；`fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 mapped locale pages
- Outcome: `pass`
- Result: en/zh-CN 的 daemon、安全边界与已知限制页面来源指纹全部刷新；没有改写页面语义，
  没有写入 token material。
- Disposition: 精确暂存 6 页和 running report，重跑 handbook 与 staged docs-sync gate。

### V42 — failure-first handbook 全门禁

- Revision: working tree after `f0f05d9e20891a13e77db5b01b33d28824ffa40f`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:handbook`
- Started/retained denominator: 54 documents × 2 locales；9 generated families
- Outcome: `pass`
- Result: coverage、link、fingerprint、status 与 secret checks 全部通过。
- Disposition: 暂存精确变更并以 V39 的文档中性理由重跑 staged docs-sync gate。

### V43 — failure-first staged docs-sync 恢复

- Revision: staged failure-first change after
  `f0f05d9e20891a13e77db5b01b33d28824ffa40f`
- Environment/instrument: Windows Node；带 V39 精确文档中性理由的
  `docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged gate
- Outcome: `pass`
- Result: mapped `daemon-http` source、6 个 handbook fingerprints、54×2 handbook 与
  18 个 generated 页面全部通过；无语义性 handbook 改写。
- Disposition: 提交并推送 failure-first immutable revision，在 exact native Linux 只运行
  新 redaction test 以观察预期失败。

### V44 — failure-first immutable revision

- Revision: `9e35d588`（完整 revision 由下一条 exact-native 记录固定）
- Environment/instrument: isolated Windows worktree；Git commit/push；pre-commit 与 pre-push
  docs-sync hooks
- Started/retained denominator: 1/1 commit；1/1 push；2/2 hooks
- Outcome: `pass`
- Result: failure-first redaction negative、6 个同步 fingerprint 与增量报告已推送到同一
  P2-T18 Draft PR 分支；两个 hooks 均通过 handbook/generated/docs-sync gate。未修改任何
  P2-T13/P2-T14 或根工作树路径。
- Disposition: 在 `DEV-LINUX-NATIVE-01` checkout 该 pushed exact revision，定向运行新增
  redaction negative。

### V45 — exact Linux request-Debug failure-first

- Revision: `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-red-9e35d588` clean detached clone；
  `cargo test -p kernel-server --locked
  personal::auth::tests::session_issue_request_debug_redacts_bootstrap_secret -- --exact
  --test-threads=1`
- Started/retained denominator: 1/1 test
- Outcome: `fail`（预期）
- Result: 新负例因默认派生的 `SessionIssueRequest::Debug` 含 bootstrap material 而失败，
  精确确认 V38 finding；没有输出任何真实 CSPRNG/bootstrap/session token。Rust 的 assertion
  failure 会回显源表达式中的 synthetic fixture literal，下一实现 revision 同时把断言改为
  runtime 变量，避免失败输出继续携带该 fixture material。
- Disposition: 移除请求类型的派生 `Debug`，实现只显示 channel、principal 与
  `[REDACTED]` 的手写 `Debug`；然后在新 pushed exact revision 上重跑定向负例和完整 auth
  matrix。

### V46 — request-Debug 修复与本地格式

- Revision: working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: isolated Windows worktree；定向 source review；
  `cargo fmt --all -- --check`
- Started/retained denominator: 1/1 implementation unit；1/1 formatting unit
- Outcome: implementation written；formatting `pass`；Rust behavior `not-run`
- Result: `SessionIssueRequest` 改为手写 `Debug`，只显示 channel、principal 与
  `[REDACTED]`；负例用 runtime 变量比较，assertion source 不再含完整 synthetic fixture
  material。本机 Windows GNU 未编译/链接。
- Disposition: 刷新映射 source fingerprints，运行 handbook/consistency/docs-sync 后生成
  pushed exact candidate。

### V47 — request-Debug 修复 fingerprint

- Revision: working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: Windows Node；`fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 mapped locale pages
- Outcome: `pass`
- Result: en/zh-CN daemon、安全边界与已知限制页面来源指纹刷新；页面既有 CSPRNG/redaction
  语义无需改写，无 token material。
- Disposition: 依次运行 handbook、repository consistency、generated byte 与 diff checks。

### V48 — request-Debug 修复 handbook 门禁

- Revision: working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:handbook`
- Started/retained denominator: 54 documents × 2 locales；9 generated families
- Outcome: `pass`
- Result: coverage、link、fingerprint、status 与 secret checks 全部通过。
- Disposition: 运行 repository consistency。

### V49 — request-Debug 修复 repository consistency

- Revision: working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:consistency`
- Started/retained denominator: 1/1 consistency run
- Outcome: `pass`
- Result: 275 requirements、55 errors、74 schemas、89 vectors，以及 task/slice/lease、
  command/environment routing 全部一致。
- Disposition: 运行 generated handbook byte gate。

### V50 — request-Debug 修复 generated byte gate

- Revision: working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: Windows Node；`generate-handbook.mjs --check`
- Started/retained denominator: 18/18 generated pages
- Outcome: `pass`
- Result: 18 页 byte-identical；本修复未手改 generated 页面。
- Disposition: 运行 branch diff whitespace check。

### V51 — request-Debug 修复 diff whitespace

- Revision: working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: Windows Git；`git diff --check`
- Started/retained denominator: 1/1 working diff
- Outcome: `pass`
- Result: 无 whitespace error。
- Disposition: 精确暂存 lease-owned paths，运行 staged docs-sync gate。

### V52 — request-Debug 修复 staged docs-sync

- Revision: staged working tree after
  `9e35d588b0dbe86fdc4122c9e488f9936de5b72c`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged gate
- Outcome: `pass`
- Result: mapped source、6 个 handbook fingerprints、54×2 handbook 与 18 个 generated
  页面全部通过；未使用 docs-sync escape。
- Disposition: 提交/push 新 immutable candidate，并在 exact native Linux 依次运行定向
  redaction、完整 auth matrix、source guard、kernel-server regressions 与 Clippy。

### V53 — request-Debug 修复 commit

- Revision: `51962e7e`（完整 revision 由 exact-native 记录固定）
- Environment/instrument: isolated Windows worktree；Git commit；pre-commit docs-sync hook
- Started/retained denominator: 1/1 commit；1/1 hook
- Outcome: `pass`
- Result: redacted `SessionIssueRequest::Debug`、不回显 fixture literal 的负例、6 个同步
  fingerprints 与 V44–V52 增量证据已提交；hook 通过。
- Disposition: 将 commit 与本条增量记录一起推送，随后只在该 pushed exact revision 上运行
  Rust 验证。

### V54 — request-Debug 修复 push

- Revision: pushed ledger HEAD `862fdb33`；product parent `51962e7e`
- Environment/instrument: isolated Windows worktree；Git push；pre-push docs-sync hook
- Started/retained denominator: 1/1 push；1/1 hook
- Outcome: `pass`
- Result: 同一 P2-T18 branch/PR 已前进到 redaction candidate；pre-push handbook/generated/
  docs-sync 全门禁通过，worktree 在追加本条前 clean。
- Disposition: 在 `DEV-LINUX-NATIVE-01` 将 V45 的 clean clone fetch/checkout 到 pushed full
  `862fdb33`，依次执行 required Rust units。

### V55 — exact Linux request-Debug redaction

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-red-9e35d588` clean detached clone；
  `cargo test -p kernel-server --locked
  personal::auth::tests::session_issue_request_debug_redacts_bootstrap_secret -- --exact
  --test-threads=1`
- Started/retained denominator: 1/1 test
- Outcome: `pass`
- Result: request `Debug` redaction 1/1；输出不含 real 或 synthetic bootstrap/session token
  material。
- Disposition: 在同一 exact checkout 运行完整 auth unit matrix。

### V56 — exact Linux 完整 auth unit matrix

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `DEV-LINUX-NATIVE-01` 同一 clean detached clone；
  `cargo test -p kernel-server --locked personal::auth::tests -- --test-threads=1`
- Started/retained denominator: 14/14 auth unit tests
- Outcome: `pass`
- Result: 14/14；OS RNG、failure/short/repeated/zero-block 无文件、session failure/duplicate
  无新 session、bounded uniqueness、0600、channel/expiry/revoke，以及 authority/request/view
  Debug redaction全部通过；输出无 token material。
- Disposition: 运行 production-source fallback guard。

### V57 — exact Linux production-source fallback guard

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `DEV-LINUX-NATIVE-01` 同一 clean detached clone；
  `cargo test -p kernel-server --locked --test p2_t18_local_token_csprng`
- Started/retained denominator: 1/1 test
- Outcome: `pass`
- Result: production token generation 只含 OS CSPRNG marker；已知 PID/time/
  `DefaultHasher`/handcrafted fallback markers 均不存在。输出无 token material。
- Disposition: 运行完整 kernel-server regressions，核对 front door 与日志路径未回退。

### V58 — exact Linux 完整 kernel-server regressions

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `DEV-LINUX-NATIVE-01` 同一 clean detached clone；
  `cargo test -p kernel-server --locked -- --test-threads=1`
- Started/retained denominator: 221/221 tests（205 unit + 16 integration）
- Outcome: `pass`
- Result: 221/221；真实 front door bootstrap/session、restart、channel、readiness、Provider、
  Task/Resource、scheduler/Tool/verifier regressions全通过。stderr 只输出 daemon lock、private
  bootstrap 文件路径与 loopback endpoint；没有 bootstrap/session token bytes，错误响应也
  未回显 bearer。
- Disposition: 运行 kernel-server all-target Clippy。

### V59 — exact Linux kernel-server all-target Clippy

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `DEV-LINUX-NATIVE-01` 同一 clean detached clone；
  `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`
- Started/retained denominator: 1/1 Clippy run
- Outcome: `pass`
- Result: all targets finished，无 warning/error。
- Disposition: 核对该 pushed exact HEAD 的 required GitHub Ubuntu/Windows CI；若仍运行则持续
  到终态，不以启动为停点。

### V60 — required CI Ubuntu（request-Debug candidate）

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: GitHub Actions run `31729398912`；
  `verify (ubuntu-latest)` job `94545791550`
- Started/retained denominator: 1/1 required Ubuntu job
- Outcome: `pass`
- Result: Ubuntu required check SUCCESS；PR HEAD 与 native Linux exact revision 一致。
- Disposition: Windows job `94545791435` 仍 `IN_PROGRESS`；继续等待其终态。

### V61 — required CI Windows 与 exact-HEAD aggregate

- Revision: `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: GitHub Actions run `31729398912`；
  `verify (windows-latest)` job `94545791435`；PR #215 status rollup
- Started/retained denominator: 1/1 required Windows job；2/2 aggregate jobs
- Outcome: `pass`
- Result: Windows required check SUCCESS；Ubuntu/Windows 2/2 均在同一 exact HEAD 通过。
  PR `mergeable=MERGEABLE`、`mergeStateStatus=CLEAN`。
- Disposition: 执行最终 defect-first call-site/secret-output review、completion verification，
  同步 task/slice/PR ready 状态后保持不合并。

### V62 — persisted bootstrap 完整调用点审查与 failure-first 格式

- Revision: working tree after
  `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `LocalSessionAuthority::initialize/load_existing`、旧生成器输出形状与
  restart caller 定向审查；`cargo fmt --all -- --check`
- Started/retained denominator: 1/1 review unit；1/1 formatting unit
- Outcome: review `fail`（defect-first finding）；formatting `fail`；新增 Rust behavior
  `not-run`
- Result: 新建 bootstrap/session 都集中使用 OS CSPRNG，但 `load_existing` 只拒绝空串，
  会接受旧 PID/time/hash 生成器留下的 16+16 hex bootstrap 并在升级后继续授权。已先加入
  `load_existing_rejects_non_csprng_bootstrap_material`，覆盖空文件与旧可预测形状；首次 fmt
  只报告一个链式调用换行，无语义问题。
- Disposition: 运行 `cargo fmt --all` 修正机械格式，再次 `--check`；随后提交/push
  failure-first revision 并在 exact native Linux 观察旧格式 case 的预期失败。

### V63 — persisted bootstrap failure-first 格式恢复

- Revision: working tree after
  `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: `DEV-WIN-GNU-01`；`cargo fmt --all` 后
  `cargo fmt --all -- --check`
- Started/retained denominator: 1/1 formatting write；1/1 formatting check
- Outcome: `pass`
- Result: failure-first test 符合 workspace Rust 格式；本机未编译/链接。
- Disposition: 刷新 6 个 mapped fingerprints，运行 handbook/docs-sync 后提交 pushed
  failure-first revision。

### V64 — persisted bootstrap failure-first fingerprint

- Revision: working tree after
  `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: Windows Node；`fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 mapped locale pages
- Outcome: `pass`
- Result: 6 页来源指纹刷新；页面语义尚未变化，无 token material。
- Disposition: 运行 handbook 全门禁。

### V65 — persisted bootstrap failure-first handbook

- Revision: working tree after
  `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:handbook`
- Started/retained denominator: 54 documents × 2 locales；9 generated families
- Outcome: `pass`
- Result: coverage、link、fingerprint、status 与 secret checks 全通过。
- Disposition: 精确暂存并运行 staged docs-sync gate；本 test-only revision 记录文档中性原因。

### V66 — persisted bootstrap failure-first staged docs-sync

- Revision: staged working tree after
  `862fdb3309a881528025c81d53b3e97489b2ac7e`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged gate
- Outcome: `pass`
- Result: mapped source、6 fingerprints、54×2 handbook 与 18 generated pages 全部通过。
- Disposition: 提交/push failure-first exact revision，在 native Linux 定向观察旧格式 case
  被当前代码错误接受。

### V67 — persisted bootstrap failure-first immutable revision

- Revision: `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: isolated Windows worktree；Git commit/push；pre-commit/pre-push
  docs-sync hooks
- Started/retained denominator: 1/1 commit；1/1 push；2/2 hooks
- Outcome: `pass`
- Result: failure-first persisted-bootstrap negative 与 V54–V66 增量证据推送到同一 P2-T18
  branch/PR；hooks 全通过。
- Disposition: exact native Linux fetch/checkout 后只运行
  `load_existing_rejects_non_csprng_bootstrap_material`，观察预期失败。

### V68 — persisted bootstrap failure-first Linux 首次 fetch

- Revision: intended `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: Windows PowerShell → `DEV-LINUX-NATIVE-01` SSH；既有 clean clone
  `git fetch --depth 1`
- Started/retained denominator: 0/1 test
- Outcome: `instrument_error`
- Result: GitHub HTTP/2 fetch 在 ref listing 阶段以 curl 16 framing error 退出，checkout/Cargo
  均未开始；不是产品测试失败。
- Disposition: 对同一 pushed revision 重试 shallow fetch；若传输仍失败，使用已登记的本地
  Git bundle exact-revision 路径，不修改源码。

### V69 — exact Linux persisted bootstrap failure-first

- Revision: `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: `DEV-LINUX-NATIVE-01`；
  `/home/wuz/cos-p2t18-red-9e35d588` clean detached clone；HTTP/1.1 shallow-fetch recovery；
  `cargo test -p kernel-server --locked
  personal::auth::tests::load_existing_rejects_non_csprng_bootstrap_material -- --exact
  --test-threads=1`
- Started/retained denominator: 1/1 test
- Outcome: `fail`（预期）
- Result: empty case 继续 fail closed；旧 16+16 hex bootstrap 被当前 `load_existing` 接受，
  `unwrap_err` 因得到 `Ok(LocalSessionAuthority)` 精确失败。失败输出中的 authority
  `Debug` 将 bootstrap 显示为 `[REDACTED]`，没有 token bytes。
- Disposition: 在 `load_existing` 中验证 current `boot-32hex-32hex` opaque shape；旧/畸形
  non-empty material 返回现有 `LOCAL_AUTH_INVALID_REQUEST`，不旋转、不 fallback。

### V70 — persisted bootstrap fail-closed 修复与格式

- Revision: working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: isolated Windows worktree；`cargo fmt --all -- --check`
- Started/retained denominator: 1/1 implementation unit；1/1 formatting unit
- Outcome: implementation written；formatting `pass`；Rust behavior `not-run`
- Result: `load_existing` 现在只接受 current lowercase `boot-32hex-32hex`；空值保持
  `LOCAL_BOOTSTRAP_MISSING`，旧/畸形 non-empty material 返回
  `LOCAL_AUTH_INVALID_REQUEST`，不自动旋转或 fallback。错误只含静态 detail。
- Disposition: 同步双语安全/daemon 文档的 upgrade fail-closed 行为，刷新 fingerprints 并
  运行本地门禁。

### V71 — persisted bootstrap 双语文档与 fingerprints

- Revision: working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: en/zh-CN daemon、安全边界与已知限制精确同步；
  `fill-handbook-fingerprints.mjs`
- Started/retained denominator: 6/6 semantic pages；6/6 fingerprints
- Outcome: `pass`
- Result: 双语页面登记 persisted legacy/malformed bootstrap 启动 fail-closed 以及
  daemon 停止后只删除该 runtime credential 的恢复路径；6 页来源指纹刷新，无 token
  material。
- Disposition: 运行 handbook 全门禁。

### V72 — persisted bootstrap handbook 全门禁

- Revision: working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:handbook`
- Started/retained denominator: 54 documents × 2 locales；9 generated families
- Outcome: `pass`
- Result: coverage、link、fingerprint、status 与 secret checks 全通过。
- Disposition: 运行 repository consistency。

### V73 — persisted bootstrap repository consistency

- Revision: working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: Windows Node/pnpm；`pnpm run check:consistency`
- Started/retained denominator: 1/1 consistency run
- Outcome: `pass`
- Result: requirements/errors/schemas/vectors、Personal task/slice/lease 与环境路由全一致。
- Disposition: 运行 generated byte 与 diff checks。

### V74 — persisted bootstrap generated byte gate

- Revision: working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: Windows Node；`generate-handbook.mjs --check`
- Started/retained denominator: 18/18 generated pages
- Outcome: `pass`
- Result: 18 页 byte-identical。
- Disposition: 运行 diff whitespace check。

### V75 — persisted bootstrap diff whitespace

- Revision: working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: Windows Git；`git diff --check`
- Started/retained denominator: 1/1 working diff
- Outcome: `pass`
- Result: 无 whitespace error。
- Disposition: 精确暂存并运行 docs-sync gate。

### V76 — persisted bootstrap staged docs-sync

- Revision: staged working tree after
  `3f46173214f7ab9fc455a25a47e26943673f7737`
- Environment/instrument: Windows Node；`docs-sync-gate.mjs --staged`
- Started/retained denominator: 1/1 staged gate
- Outcome: `pass`
- Result: mapped source、双语语义页与 fingerprints、54×2 handbook、18 generated pages
  全部通过；未使用 escape。
- Disposition: commit/push exact candidate 后重跑 native focused/full/Clippy 与 required CI。
