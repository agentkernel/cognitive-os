# 接续提示词 — Agent Hub Session+File 车道（SESS）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；安全负例先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；禁写 provider auth/entitlement/history/内部 DB（L6 阻断）；禁读非 documented 路径。

## 本车道任务

- canonical：[architecture/session-and-file-adoption.md](../docs/architecture/session-and-file-adoption.md)
- 计划：[clients/agent-hub/plan/lane-session-file.md](../plan/lane-session-file.md)
- 目标：官方 session 候选/只读 hydrate、条件写采用（单 writer 证明）、JSONL/SQLite 只读观察、安全路径解析。

## gate 与允许范围（当前 blocked）

依赖 HOST + CTR（接口核验）；未满足接口/平台 PoC gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：无 exclusive lease 不写（双 writer 拒绝）、JSONL 半写不产生完成事实、SQLite 写压力 snapshot 一致（禁 immutable(live)/checkpoint/主库单文件复制）、symlink/hardlink/reparse/cloud placeholder 逃逸全拒。oracle：POC-SESS-001/002、POC-FILE-001/002/003。任务 AH-SESS-01..05 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
