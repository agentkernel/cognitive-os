# Lane-RUN Pi Agent candidate boundary handoff

## 1. 本次会话完成

- 新增 `apps/pi-agent-adapter`：固定 Pi 为 DeepSeek、无工具、无扩展、无技能、无
  项目上下文、无会话的 `uncontained_candidate_only` 启动/评测边界（ADR-0016）。
- 子进程环境清理除操作系统必需变量外的 ambient API tokens，只注入单次
  `DEEPSEEK_API_KEY`，并对捕获输出脱敏。
- 实际安装 Pi 0.81.1 并用 DeepSeek 运行 5 次：5/5 固定输出通过；实际模型
  `deepseek-v4-flash`；p50/p95/p99 = 6081/6451/6451 ms；零 tool result、零
  authority commit、零 Effect。

## 2. 未完成 / 进行中

- 这不是受治理 `AgentInstallation`。P2 需要受信任签名/provenance verifier；P3
  需要 durable InstallationStore；P4 需要 Linux-native OS sandbox；P5 需要
  lifecycle/I/O adapter。
- Windows-native sandbox 仍 unsupported。不得将该批标记为 C0/C1、Profile 或
  REQ-PERF-004/005 pass。

## 3. 测试与证据状态

- `cargo test -p pi-agent-adapter`: 8 pass。
- strict clippy / fmt: pass。
- 外部证据是本地临时目录；不进 Git，也不包含密钥。

## 4. 未决风险与漂移

- 未新增 schema/registry/vector；ADR-0016 仅冻结外部候选边界。
- Pi 请求别名与响应模型可能不同；启动器已记录 responseModel，后续评测不得只写
  requested model。

## 5. 下一步入口

- 建议提示词：`docs/prompts/lane-krn.md`，先实现 durable InstallationStore 的最小
  SQLite port/恢复测试；随后 Lane-RUN 接入管理 authority。
- 工作分支：`lane/run-pi-agent-integration`。
- 第一个动作：审计 `cognitive-store` 的现有 SQLite authority store，设计不扩权的
  installation persistence port。

## 6. 快照

- PROGRESS 已更新：待本批提交。
- 本次提交列表：待创建。
