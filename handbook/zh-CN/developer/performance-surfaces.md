---
doc_id: dev.performance-perf
locale: zh-CN
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: crates/cognitive-runtime/src/perf.rs
    symbols: ["GovernanceOverheadSample", "GovernedPathStageCollector"]
  - path: crates/cognitive-runtime/src/performance_campaign.rs
  - path: crates/cognitive-runtime/src/campaign_runner.rs
  - path: crates/cognitive-runtime/src/loopback_transport.rs
  - path: crates/cognitive-runtime/src/resource_sampler.rs
  - path: crates/cognitive-runtime/src/provider_route_policy.rs
  - path: crates/cognitive-runtime/src/task_scenario_harness.rs
  - path: packages/dsh-akp-adapter/src/index.ts
    symbols: ["DshAkpTiming"]
  - path: packages/dsh-akp-adapter/src/plugin.ts
    symbols: ["applyDshAkpCordisPlugin"]
  - path: packages/dsh-akp-adapter/scripts/dsh-real-process.mjs
  - path: packages/dsh-akp-adapter/scripts/paired-path.mjs
tests:
  - crates/cognitive-runtime/src/bin/p7_t04_module_benchmark.rs
fingerprint: "sha256:b55f131eb2941a48dc2a0939f1579ea56e1bd808898e6b608e63c6964d2af8e4"
non_claims:
  - 此处所有表面只产 hypothesis 级 non-claim 观察；这些代码不产生任何收益、Gate、release 或 Profile 结论，campaign 执行结果由正式计划的证据记录拥有。
---

# 性能面

本仓库的所有性能代码都是**带 fail-closed 诚实规则的测量管道**，绝非收益声明。存在两
代：

## P7-T04 代（回归地板）

`perf.rs`：`GovernanceOverheadSample`（固定治理路径 stage 词汇）、确定性模块基准
（`p7_t04_module_benchmark` 二进制，`COGNITIVEOS_BENCHMARK_SAMPLES`）、断言完整不相
交覆盖一次受治理交换的 stage 收集器，以及被后续结构工作消费的模块回归地板策略。
`p9_t01_async_decision_gate` 产出了 hypothesis-only 的 "conservative-no-migration"
异步决策。

## P9-T04 代（全面 campaign，ADR-0051）

由 campaign 任务添加、供预注册运行消费，daemon 不消费：

- `performance_campaign.rs` + `campaign_report.rs`：类型化 L0–L5 campaign 策略——
  retained 分母记账、八个硬安全计数器（任一非零 ⇒ 结果不可晋升）、cleanup 事实、缺
  独立验证时 claim 上限强制 `hypothesis`、无完成的 A/B 臂则 `benefit_claimed=false`。
- `campaign_runner.rs` + `p9_t04_l0_l1_campaign_runner` 二进制：准入拒绝 secret 形
  环境/参数与未注册环境；报告拒绝未脱敏或自我晋升的观察。
- `loopback_transport.rs`：把真实 loopback 前门分解为不相交 stage，显式声明不含
  `effect_persistence`、`provider_network`、`pi_process_launch`、`scheduler_wait`。
- `resource_sampler.rs`：有界 `/proc` 采样器，绝不打开 `cmdline`/`environ`、绝不解
  析描述符目标、把递减累计计数视为 PID 复用。
- `provider_route_policy.rs`：L3 规则——`retry=0`、每个已起请求必须留下分类结果、禁
  止伪造 TTFT/费用、计数不全时 usage 为 `not_available`。
- `task_scenario_harness.rs`：L4 受治理 Task 场景由冻结 oracle + 独立验收判定；只读
  场景发生任何变更即边界违规，优先级压倒一切其他结果。

`tools/personal/` 存放操作者驱动的 runner 脚本（smoke、L3 route、L3 冷旅程、L4
T1）。campaign **结果**（哪些 cell 执行、retained 计数、digest）由正式计划的证据记
录拥有——只链接、绝不复制。

## dsh AKP 适配器计时（P8-T09 / P8-T10）

`@cognitiveos/dsh-akp-adapter` 为每次仅 candidate 的 submit 记录 serialization、
transport 与 total。这些字段只是 Path A（dsh → DeepSeek Flash）与 Path B
（dsh → AKP → daemon → Flash）配对观察的测量入口，不构成零开销、无损或任何
Gate/release/Profile/B01/Agent-benefit 结论。linux-002 harness
`scripts/linux002-e2e.mjs` 会在 shim submit 上记录这些计时并等待 Task
`COMPLETED`；`scripts/dsh-real-process.mjs` 记录真实 dsh 进程墙钟时间与首次
stdout（TTFT hook）：Path B 经 loopback SSE-to-unary 桥接到 daemon Provider
代理，Path A 直连 Flash；`scripts/paired-path.mjs` 在同一主机重复两条路径。
Workspace* `startupEvents` 仍是 candidate 事件。都不是 Gate 样本。

状态 `partial` 是因为 daemon 自身不暴露持续性能仪表；这里的一切都是选择性使用的测量
工具。
