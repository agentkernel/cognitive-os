# AGENTS.md — CognitiveOS Personal 开发代理入口

本仓库的唯一活动实现项目是 **`cognitiveos-personal`（CognitiveOS Personal）**。
原 CognitiveOS 设计、白皮书、规范和通用内核是架构参考与合同基础，不是第二个待交付
项目。完整边界见 [PROJECT-IDENTITY.md](docs/governance/PROJECT-IDENTITY.md)；本文件只
保留代理必须立即知道的操作规则，通用治理正文见
[DEVELOPMENT-OPERATING-MODEL.md](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)。

## 新会话启动顺序

1. 阅读本文件和 [项目身份](docs/governance/PROJECT-IDENTITY.md)；
2. 阅读 [Development Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)；
3. 阅读 Personal 正式计划 [PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)；
4. 只读 `PROGRESS.md` 的 `Current snapshot`；
5. 只读 [PARALLEL-LANES.md](docs/plan/PARALLEL-LANES.md) 的活动 lease；
6. 产品/架构任务再读 [Personal 产品设计](docs/product/personal/README.md) 与
   [Personal 架构](docs/architecture/personal/README.md) 的相关章节；
7. 再阅读所选任务对应的最新 matching handoff 和根 `plan.md` 任务卡。

正式计划决定任务和 Gate，`PROGRESS.md` 决定当前事实，Parallel Lanes 决定当前可写路径，
handoff 只提供操作连续性，根 `plan.md` 只提供研究和细节。历史 handoff、旧提示词和
聊天上下文不能覆盖正式来源。禁止读取或引用 `History/`。

## 任务领取与反循环协议

- 默认只领取 `cognitiveos-personal` 的 `P*-T*` 任务；架构层改动必须服务于当前 Personal
  切片，规范合同改动必须走 Lane-CTR。
- 纯阅读、研究、计划草稿不会改变任务状态。第一个任务专属实现或测试 slice（包括
  failure-first 测试）开始时，将任务设为 `in-progress`。
- 领取前确认没有路径重叠的活动 lease；领取后立即实施最小垂直切片。不要把 acceptance
  或 promotion Gate 当成 implementation mutex。
- 每次会话必须产出实现、失败优先测试、可验证文档修正，或带
  `blocked_paths` / `blocked_task_ids` / `blocked_gate_ids` / owner / next action 的阻塞记录。
  已确认依赖后不得继续无出口审计。
- 结束会话前同步正式计划、`PROGRESS.md` current snapshot 和 handoff；未执行检查写
  `not-run`，不可推断为通过。

## 不可放松的不变量

1. Rust daemon 是唯一 authority writer；Pi、CLI、SDK、UI 和 fixture 都是客户端。
2. 概率组件只能产 candidate/proposal；授权、CAS、状态迁移、预算、幂等、fencing、
   Effect 提交、reconcile 和最终验收由确定性服务端执行。
3. 外部 mutating operation 必须使用 persist-before-dispatch 的 Intent/Effect；外部成功、
   Provider response 或 Pi `agent_end` 都不等于 Task 完成。
4. Provider/user secret 只能进入批准的 Secret Store，绝不进入 argv、普通配置、SQLite、
   日志、CI、测试输出或 evidence。
5. 负例、规范向量和合同不能为了迎合实现而删除、放宽或改写。
6. local、WSL、fixture 和 ordinary CI 只有在 Gate 预注册 campaign 明确纳入时才能贡献
   Gate 证据；否则只能是 non-claim implementation evidence。

## 命令速查

| 目的 | Windows PowerShell（本地） | CI（bash） |
|---|---|---|
| Rust 构建 | `cargo build --workspace` | 同左 |
| Rust 测试 | `cargo test --workspace` | 同左 |
| Rust lint | `cargo clippy --workspace --all-targets` | 同左 |
| TS 安装 | `pnpm install` | `pnpm install --frozen-lockfile` |
| TS 构建/测试 | `pnpm -r build ; pnpm -r test` | `pnpm -r build && pnpm -r test` |
| 静态一致性检查 | `pnpm run check:consistency` | 同左 |
| 符合性 runner | `cargo run -p cognitive-conformance --bin conformance-runner` | 同左 |

本机若 `cargo` 不在 PATH：`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`。
工具链钉在 `rust-toolchain.toml`（1.97.1）。PowerShell 5.1 不支持 `&&`；需要条件
串联时使用 `if ($LASTEXITCODE -eq 0) { <next-command> }`。

## Personal 实验环境边界

`personal-linux-native-01`（`wuz@192.168.1.2`）只是已资格化的实验性开发主机，优先
用于明确授权的 `experimental-local-only` / `tested-local` 验证，不是 B01、release、
Profile、containment 或正式产品 Gate 环境。每个 Pi slice 都必须重新确认 Linux/native
user-systemd、Rust/Node、exact Pi `0.81.1` 和可清理目录；`pi` 不在 PATH 不是 pin 已
满足的证据。真实 load 只允许 `--extension <absolute-path>` 与脱敏 observation。

SSH 仅使用非交互、无 secret 探针，例如：
`ssh -o BatchMode=yes -o ConnectTimeout=10 "wuz@192.168.1.2" "<redacted command>"`。

## 目录和变更边界

- `specs/`、`conformance/`：架构合同和符合性资产；不得为实现改写。
- `crates/`、`apps/`、`packages/`、`tests/`、`tools/`：Personal 实现及其验证工作面。
- `docs/governance/`、`docs/plan/`、`docs/checkpoints/`：治理、正式计划、快照和移交。
- `apps/cognitiveos-console/`、独立客户端和其他 deferred 能力：默认只维护设计/台账，
  不启动独立实现。
- `personal-blog/` 是独立仓库，禁止推入本仓库；禁止在其他路径建立平行副本。

变更必须声明 `implementation-only`、`corrective`、`product-semantic`、
`normative-semantic` 或 `structural`，
并按 [docs-sync-contract](docs/standards/docs-sync-contract.md) 完成联动。提交/PR 必须
关联 Personal 任务或 REQ/F/IMP；没有关联时说明原因。未知工作树改动不得覆盖、回退、
混入或使用 `git add -A`。
