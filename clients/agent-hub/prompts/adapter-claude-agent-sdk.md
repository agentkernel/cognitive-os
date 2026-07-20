# 接续提示词 — Agent Hub Claude Agent SDK Adapter（AD-CLAUDE）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界。2. 规范优先级；冲突取保守解释。3. 四类状态用语严格区分。4. 测试先行；安全负例先行。5. 规范表面冻结；漂移登记后修正。6. P0 门禁。7. 可追溯提交。8. 红线：禁 `History/`；禁虚构规范资产；不复制 `.credentials.json`/Keychain；不写 native JSONL（L6）。

## 本 Adapter 任务

- dossier：[adapters/tier1/claude-agent-sdk.md](../docs/adapters/tier1/claude-agent-sdk.md)
- 计划：[docs/plan/agent-hub/adapter-claude-agent-sdk.md](../plan/adapter-claude-agent-sdk.md)
- 目标：接口一手核验（Agent SDK/`--resume`/JSONL schema）、L1/L2 控制映射、L3/L5 session 与 JSONL、账号与凭据。

## gate 与允许范围（当前 blocked）

独立 gate：Claude 接口一手核验（AH-B4）、平台 PoC（AH-B2）、条款允许性（AH-B5）。未过 gate 前只做接口核验文档，不写实现、不 mock 解阻。安全负例（不可豁免）：普通既有 claude 进程不可 send、外部并行 `claude --resume` 同写同 session 拒绝（TM-006）、fork/rewind 写 native 归 L6 阻断、零 secret 落盘。oracle：POC-SESS-002、POC-FILE-001、POC-SEC-003。任务 AH-AD-CLAUDE-01..04 见计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
