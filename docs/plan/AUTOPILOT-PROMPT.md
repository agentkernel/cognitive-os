# CognitiveOS Personal 自动推进提示词

> 用途：复制下方 `---` 之间全文到新窗口，作为该窗口的首条消息。文档性质：操作提示词，documentation-only，不是计划、规范、Gate、Profile 或 release 声明。

---

你在 `D:\agent-kernel`（CognitiveOS 仓库）工作。目标是继续推进 **Personal P1-T08 第二个 failure-first 原子批**：在已合并的离线 Linux bundle manifest/digest/staged-activation foundation 上，选择并记录具体的可信加密 attestation 验证机制，先写失败测试，再实现严格离线、固定 trust root、不可由 bundle 自选信任锚的验证边界。完成本批的代码、调试、测试、文档、提交、PR 和支持矩阵 CI 收口后，P1-T08 仍保持 `in-progress`；随后在对话中生成一份可一键复制的“下一批动作提示词”。本提示词是持续授权；除仓库规则明确要求的 owner 决策点外，不要等待逐步指示。

## 0. 已知基线与本批唯一目标

- 当前已知主干基线：PR [#107](https://github.com/agentkernel/cognitive-os/pull/107) 已普通 merge，merge commit 为 `160d9e728d1aafa10a39b4806d2790be36c990cd`。必须先 `git fetch origin` 并以实际 `origin/main` 为准，不得假设该哈希仍是最新主干。
- 正式台账：`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`。P1-T07 为 `done`；P1-T08 为 **`in-progress`**，开发轨道为 `experimental-local-only`；P1-T09 仍 `not-started`。
- 最近交接：`docs/checkpoints/20260727-personal-p1-t08-bundle-foundation-handoff.md`。
- 已有实现：`crates/cognitive-runtime/src/linux_bundle.rs` 提供严格 schema-v1/Linux x86_64 manifest、artifact SHA-256、HTTPS attestation-reference 结构、caller-provided Pi pin、拒绝 vendored Node/Pi、staged activation、health 前不切换 active、失败/中断保留旧版本与用户数据。
- 已有实现**没有**加密验证 attestation；URL 形状检查不能称为 trusted verification。本批只闭合这一缺口的可信验证机制及其 failure-first tests。
- 本批不实现 downloader、GitHub Release 发布、真实 release signing ceremony、systemd user service、`deploy/linux/install.sh`、uninstall、跨进程安装 lease、Linux-native B01 campaign 或 P1-T08 完成声明。

## 1. 强制启动与工作区安全

1. 按顺序读取：根 `AGENTS.md`、`docs/plan/PROGRESS.md`、最近 handoff、`docs/plan/PARALLEL-LANES.md`、`.cursor/rules/`、`PERSONAL-DEVELOPMENT-PLAN.md` 的 P1-T08/P1-T09 行、`plan.md` 的 P1-T08 完整卡、ADR-0025，以及 `linux_bundle.rs`。
2. 执行并解释实际结果：
   - `git status --short --branch`
   - `git fetch origin`
   - `git log --oneline --decorate -12`
   - `git log --oneline origin/main..HEAD`
   - `git worktree list`
   - `gh pr view 107 --json state,mergedAt,mergeCommit,url`
3. 开工前必须确认主工作树没有未知未提交改动。`.cursor/`、`.vscode/` 是本机配置；若已由 `.git/info/exclude` 本地忽略，不要提交或删除。`personal-blog/` 是嵌套独立仓，禁止读取、修改、暂存或推入 CognitiveOS。禁止读取或引用 `History/`。
4. 基于最新 `origin/main` 建单用途代码分支和隔离 worktree，例如 `lane/personal-p1-t08-attestation-verifier`。代码批必须走 PR；不得直接在 main 开发，不得覆盖其他 worktree，不得 `git add -A`。
5. 若发现提示中的基线与实际状态不符，停止写入，先用 status/log/diff/PR 状态重建事实，选择最小风险路径；禁止 destructive reset、stash 用户改动或回退未知内容。

## 2. 先决设计：可信验证机制必须明确

在改实现前，先写一份窄幅 ADR（预计下一可用编号，先检查 `docs/adr/`，不要抢号或覆盖既有 ADR），记录本批选定的机制。默认优先采用**离线 detached signature + 产品侧固定 trust root/keyring**的最小机制；可用成熟、审计面较小的 Rust 加密库实现，但必须通过包管理器添加实际最新兼容版本，不得手写密码学或编造版本。

ADR 至少回答：

1. 签名的精确对象是什么：必须绑定 product/platform/version、artifact filename/digest、Pi version/integrity 和 attestation schema/version；不能只签一个可替换 URL。
2. bytes-to-sign 如何确定：优先签原始、版本化的 attestation statement bytes，或采用仓库已有明确 canonical JSON 规则；不得依赖普通 pretty JSON 的不稳定序列化。
3. trust root 从哪里来：生产验收只能来自产品侧固定、版本化、受审核的 keyring/key ID allowlist；bundle/manifest/attestation 不得携带并自选一个新公钥让自己通过。
4. key ID、算法、signature 编码如何严格解析；未知 key、算法漂移、长度异常、重复字段、额外字段、base64/hex 非规范编码必须 fail-closed。
5. key rotation/revocation 在本批如何表示：可以只冻结接口和拒绝语义；没有真实 owner signing key 时只能用测试 fixture key，不得把 fixture key 或 local test signature称为 release trust/evidence。
6. 为什么此机制是 P1-T08 的离线 verifier foundation，而 P7-T01 仍负责真实 SBOM/attestation 生成、release signing 与发布证据。
7. 明确拒绝运行 `curl`、`gh`、`cosign` 或任意外部命令作为 runtime library verification 的隐式依赖，除非仓库事实和 ADR 评审证明该依赖是受支持且有完整安装/版本/离线边界；不要在无决策时引入 subprocess verifier。

若调查发现现有机器合同强制要求不同 attestation 格式，或需要新增/修改 registry/schema/vector，立即停止实现并走 Lane-CTR；不得创建平行规范表面迎合本批。

## 3. Failure-first 测试要求

先添加失败测试并确认它们在旧实现上失败，再实现。测试应靠近 `linux_bundle` 现有 focused tests，覆盖至少：

- 合法 fixture attestation + 合法 detached signature + trusted key 通过；
- artifact 或 signed statement 任一字节被篡改后拒绝；
- manifest 的 version/platform/artifact digest/Pi pin 与 signed statement 不一致时拒绝；
- wrong key、unknown key ID、unsupported algorithm、malformed signature 拒绝；
- bundle 自带公钥/self-selected key 即使数学签名有效也拒绝；
- attestation reference 缭绕、路径穿越、缺文件或非 HTTPS provenance reference 拒绝；
- trust keyring 为空、重复 key ID 或含不合法 key material 时 fail-closed；
- 错误输出不包含 artifact bytes、signature bytes、key material、secret 或用户数据；
- attestation 失败发生在 stage/active pointer 修改之前，旧 active version 与用户数据保持不变；
- 既有四个 manifest/activation tests 继续通过。

测试 fixture 只能使用显式标注的非生产测试 key，不能把 private signing key 放进生产配置、普通文档、证据或 release 路径。若测试库需要随机数，保证断言确定、无 flaky；优先固定 RFC/库官方测试向量或测试时生成的临时 key，并确保私钥不进入日志。

## 4. 实现边界与代码质量

- 概率组件不得参与；verification、trust selection、digest comparison、stage 与 activation 都是确定性边界。
- `verify_linux_bundle` 或其替代入口必须只有在 digest、Pi pin、signed statement binding 和 cryptographic signature 全部通过后才返回可 stage 的 verified value。尽量用类型区分 parsed/untrusted manifest 与 verified bundle，避免调用方拿未验证 manifest 调 `stage_verified_bundle`；若改变公共 API，更新全部调用点和 tests，但不扩大 crate 之外的合同表面。
- key selection 必须由 caller/product policy 提供的 trusted keyring 驱动；manifest 只能引用允许的 key ID，不能注入 trust root。
- 使用常见成熟库；禁止自制 Ed25519/RSA/ECDSA 实现、非恒定时间手写比较、shell-out、网络下载、环境变量 trust root 或当前目录隐式查找。
- 错误类型应稳定区分 malformed attestation、unknown/untrusted key、signature mismatch、statement binding mismatch 和 I/O；错误文本保持 non-secret。
- 不 vendor Node/Pi；不下载 Pi；不接 systemd；不修改 SQLite/authority/Task/Effect/capability；不声称 containment。
- 高可读性命名，避免短变量名；复杂安全条件使用具名 helper，少量注释说明“为何”，不重复代码本身。

## 5. 调试与验证矩阵

根据实际依赖和 crate API 调整 focused test 名称，但至少真实运行：

```powershell
cargo fmt --all -- --check
pnpm run check:consistency
git diff --check
git diff --check origin/main...HEAD
```

Windows GNU linker exit 121 不是支持基线。优先在 WSL 使用独立 target：

```powershell
wsl bash -lc 'cd /mnt/d/<isolated-worktree> && CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-attestation /root/.cargo/bin/cargo test -p cognitive-runtime linux_bundle --locked && CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-attestation /root/.cargo/bin/cargo clippy -p cognitive-runtime --all-targets --locked -- -D warnings'
```

必须先用 `wsl wslpath` 或 WSL `pwd` 确认路径。若 `pnpm` 因隔离 worktree 无 `node_modules` 缺 `ajv`，先在该 worktree 执行 `pnpm install --frozen-lockfile` 后重试。WSL 结果标为 `windows_wsl2_linux_guest` local evidence，不升级为 Linux-native Gate。修改后用 IDE lints 检查相关 Rust/Markdown 文件。

提交/推送前还必须执行：

```powershell
git status --short --branch
git log --oneline origin/main..HEAD
git log --name-only --pretty=oneline origin/main..HEAD
git diff --name-only origin/main...HEAD
git diff --check origin/main...HEAD
```

另外扫描 `<<<<<<<`、`=======`、`>>>>>>>`，防止 clean worktree 中已提交的冲突标记漏过；不能只运行无参数 `git diff --check`。

## 6. 文档、提交、PR 与支持矩阵收口

本批同一原子提交/PR 应更新：

- 选定机制的 ADR；
- `crates/cognitive-runtime` 实现、依赖与 focused tests；
- `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`：P1-T08 保持 `in-progress`，只追加本批真实 evidence/non-claims；
- `docs/plan/PROGRESS.md`：顶部追加 concise attestation-verifier slice 状态；
- 新 handoff：`docs/checkpoints/YYYYMMDD-personal-p1-t08-attestation-verifier-handoff.md`；
- 若机制使 `plan.md` 的范围/步骤实际变化，进行最小同步；不要擅自改验收或任务依赖。

按 ADR-0008 / `.cursor/rules/18-auto-commit-and-doc-sync.mdc` 自动完成原子任务：逐路径暂存，检查 staged diff，提交并 push；代码批创建 lane PR。PR 正文包含 Summary、Tests、Trust model、Failure cases、Explicit non-claims。等待 Ubuntu + Windows/MSVC 所有检查绿色且 PR `CLEAN`/`MERGEABLE` 后普通 merge；不得 squash/rebase/force，除非仓库默认策略明确要求。CI 红灯必须查日志、修根因、补测试、新提交重推，禁止带红合并或跳过 hooks。

合并前再次 fetch 并审核 `origin/main..HEAD` 全部文件，确保无 `.cursor/`、`.vscode/`、`personal-blog/**`、私钥、真实 key、secret、临时 artifact 或无关文件。合并后核验 PR state、merge commit、`origin/main` 和主工作树；不要删除用户配置。仅在隔离 worktree clean 且不再需要时移除它。

## 7. 完成定义与明确非声明

本批完成仅表示：仓库选择并记录一个具体的离线 trust model，代码可用产品侧固定 keyring 对 bundle attestation 做真实 cryptographic verification，并由 focused negative tests 与支持矩阵 CI 证明实现行为。

即使本批全绿，P1-T08 仍为 `in-progress`。不得声称：

- 存在真实 release bundle 或 production signing key；
- GitHub Release/SBOM/provenance 已发布；
- downloader、inspected install script、systemd user service、uninstall 已实现；
- P1-T08/P1-T09、B01、G1、Profile、containment、Linux-native campaign 或 RC 已完成。

## 8. 会话结尾必须生成下一批提示词

完成并合并本批后，基于当时真实 `origin/main`、台账、PROGRESS、最新 handoff 和未决项，在**对话中**输出一份完整 Markdown code block，标题为“下一批动作提示词（可一键复制）”。下一批通常应是 P1-T08 的 inspected local installer/download boundary 与 staged activation integration，或若 attestation 批暴露 blocker，则是最小 blocker-resolution 批；不得提前硬编码结论。

该下一批提示词必须包含：准确 main/PR/merge commit、工作区安全状态、唯一原子目标、必读文件、failure-first tests、实现/非实现范围、精确验证命令、docs-sync、自动 commit/push/PR/CI/merge 流程、non-claims，以及“批次结束再生成下一批提示词”的同样要求。不要只给摘要，确保新窗口无需依赖聊天历史即可执行。

当上下文接近 180k tokens 或预计下一阶段超过 220k 时，不开始新复杂修改；先完成当前原子批、更新 PROGRESS/handoff、提交可验证状态，并生成下一批提示词。禁止依赖对话历史承载工程状态。

---
