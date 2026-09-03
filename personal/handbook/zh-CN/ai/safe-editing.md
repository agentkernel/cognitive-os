---
doc_id: ai.safe-editing
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: docs/governance/AXIOMS.md
    symbols: ["A1", "A8"]
  - path: docs/standards/docs-sync-contract.md
fingerprint: "sha256:2df0f823a66ee077a76bf5cafe7c26eb851d3efcc7fb7d3121c856e52a8c0736"
non_claims:
  - 本页仅为导引摘要；具约束力的措辞由所链接的治理文档拥有。
---

# 安全编辑边界

## 绝不放松（公理，由 [`AXIOMS.md`](../../../../docs/governance/AXIOMS.md) 拥有）

- A1/A2 —— 只有 Rust daemon 写权威状态；Pi、CLI、SDK、sidecar、fixture 与第三方
  agent 只产 candidate 与 observation。
- A3 —— 一切外部或不可逆变更先持久化 Intent/Effect 再派发，带幂等键与 fencing。
- A4 —— Task 完成需要独立验证；进程退出、Provider 响应或 `agent_end` 都不是完成。
- A5 —— secret 只经批准的 Secret Store 与批准的非日志输入路径进入（含 ADR-0055
  的用户发起、daemon 所有的凭据导入边界）；绝不进 argv、环境变量、CognitiveOS
  写入的配置、SQLite、日志、CI、测试、证据或聊天。
- A6 —— 合同与负例向量绝不为迎合实现而改写（合同变更走 Lane-CTR）。
- A7 —— 本地/fixture/WSL/普通 CI 证据绝不升格为 Gate、release 或 Profile 声明。
- A8 —— 未知工作树改动受保护：绝不覆盖、回退、暂存或混入；绝不使用 `git add -A`。

## 受保护目录

- `core/specs/**` 与 `core/conformance/**`：架构合同；禁止实现驱动的改写——真正的合同变更走
  Lane-CTR，并同批联动 registry、schema、绑定、transition 与向量。
- `docs/governance/**`、`docs/plan/**`：治理与正式计划来源；只能经各自治理流程与活动
  lease 修改。
- `History/**`：冻结；绝不读取、引用或修改。`personal-blog/` 绝不进入本仓库。
- 生成目录（`core/crates/cognitive-contracts/src/generated/`、
  `core/packages/contracts-ts/src/generated/`、手册生成参考页、
  `docs/traceability/matrix.yaml`、`core/tests/golden/*.json`）：只能经生成器再生；手改会
  被 CI 漂移门拦截。

## 写入前必做

1. 现读 `PROGRESS.md` Current snapshot 与 `PARALLEL-LANES.md` 活动 lease 表。可写权
   属是精确路径 lease；活动 lease 不得重叠。
2. 一个正式任务 = 一个 branch、一个 Draft PR、一个 lease，直至完整验收
   （[`DEVELOPMENT-OPERATING-MODEL.md`](../../../../docs/governance/DEVELOPMENT-OPERATING-MODEL.md)）。
3. 声明变更类别（`implementation-only`、`corrective`、`product-semantic`、
   `normative-semantic`、`structural`），并在同一交付内完成
   [`docs-sync-contract.md`](../../../../docs/standards/docs-sync-contract.md) 的联动义务。
4. 查[文档影响](docs-impact.md)：所改路径映射到的手册页必须在同一 PR 内更新或
   重生成。

## 本地环境硬事实

- 本地 shell 是 Windows PowerShell 5.1：`&&`/`||` 无法解析；使用分开的命令或
  `if ($LASTEXITCODE -eq 0) { … }`。
- 本地 Windows GNU 主机无法链接 Rust（已登记 linker exit 121）。自 2026-09-03 起，已登记的
  本地目录带有指向本机已装 MSVC 工具链的 rustup override：**只**在 `rustc -vV` 报
  `host: x86_64-pc-windows-msvc` 的目录里本机运行 workspace `cargo build/test/clippy`，结果
  只算开发证据，supported validation 仍路由到 CI 或 native Linux（见
  [验证命令](validation-commands.md)）。绝不在 feature 任务里改 `rust-toolchain.toml`、PATH 或
  `.cargo/config.toml` 来"修" linker 失败。
