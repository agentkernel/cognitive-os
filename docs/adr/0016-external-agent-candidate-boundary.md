# ADR-0016: 外部 Agent 的候选执行边界

- 状态：Accepted
- 日期：2026-07-24
- 决策范围：Pi 等外部 Agent 的接入、测试与安装声明边界

## 背景

Pi 是独立发布的终端 Agent。即使关闭其工具，它仍是一个外部进程和模型客户端；
其输出不能天然成为 CognitiveOS 的授权、Effect 或 Task 完成事实。当前 Windows-native
sandbox 证据为 unsupported，InstallationLedger 仍为进程内实现，且通用安装器的
`SignatureProvenancePort` 尚无 Pi 发布物的受信任验证器。

## 决策

1. 新增 `pi-agent-adapter` 仅作为**候选执行**边界：固定 DeepSeek provider，关闭 Pi
   工具、扩展、技能、项目上下文、会话与项目文件信任，并以清理后的子进程环境传递
   单次 `DEEPSEEK_API_KEY`。
2. 该边界必须输出 `uncontained_candidate_only`，并恒定声明
   `authority_committed=false` 与 `effects_created=false`。它不是 C0/C1、不是
   `AgentInstallation` commit，不能推进 Task、发放 capability 或创建 Effect。
3. 性能输出必须记录 Pi 请求模型与服务端观测模型，且标为
   `not_a_REQ-PERF-004_campaign`；不允许把外部 Agent smoke 当作治理开销或收益证明。
4. 真正的受治理安装须在下列条件都满足后另行启用：受信任供应链验证、持久化
   InstallationStore、目标平台 OS sandbox 负例证据、adapter 的 I/O/lifecycle 映射、
   管理 authority commit 与恢复/对账行为证据。
5. 用户可显式选择 Custom User-Provided 模式，登记其自行提供的本地项目包。该模式
   必须先展示固定风险提示并取得用户明确确认；确认绑定 `principal://` 操作者、
   `file://` 不可变项目包引用和精确 artifact digest。它是用户来源声明，不是上游
   发布者签名。确认后，项目与正常安装走同一后续授权、运行与 lifecycle 路径；与正常
   安装相同，安装本身不自动授予 capability、不创建 Task completion 或 Effect。它仍
   不能单独成为 C0/C1、Profile、sandbox 或供应链通过声明。

## 影响

- 普通用户可在明确标注的实验候选模式下验证模型连通性，而不会扩大授权面。
- Custom User-Provided 模式允许已确认的本地自带项目进入与正常安装相同的 durable
  installation 及后续授权路径；项目运行仍须通过各平台 sandbox 和 lifecycle/I/O gate。
- 本决策不改动 v0.1 registry/schema/vector，也不降低 REQ-AGENT-INSTALL-001、
  REQ-AGENT-ADAPTER-001、REQ-AGENT-SANDBOX-001 或 REQ-AGENT-COMPLETE-001 的门槛。
- Windows-native 测试不得被叙述为 sandbox pass；Linux-native 证据也不得回填到
  Windows-native 行。
