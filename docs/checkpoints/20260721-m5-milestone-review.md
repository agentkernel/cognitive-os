# 20260721 M5 Milestone Review

## 1. 范围回顾

对照 `docs/plan/DEVELOPMENT-PLAN.md` M5：意图链 / Harness Loop / Shell / 管理面 / AKP HTTP+SSE / R1 审批 / TSC 真集成 / CFR 行为证据。

交付车道：KRN（意图/Loop/恢复 6–7）→ RUN 批 1–2b → CTR 绑定 → TSC HTTP/SSE → CFR M5 向量批。

## 2. 验收判据逐条对照

| # | 判据（摘要） | 结果 | 证据 |
|---|---|---|---|
| 1 | 意图链 record→admit→mint；修正 epoch fencing | **通过（实现+内核测试；向量 INTENT-SUPERSEDE 仍 not-run）** | KRN M5 handoff；store `m5_intent_chain` |
| 2 | 有界 Harness；停滞 Stop/Escalate；硬预算 | **通过** | RUN 批 2b `BoundedHarness`；KRN `LoopDriver` |
| 3 | Shell proposal/preview/submit/attach/detach/cancel | **通过（车道测试 + 2 向量行为 pass）** | RUN `ShellService`；CFR SHELL-CANCEL/DETACH |
| 4 | 管理面无模型四动词 + session | **通过** | RUN 批 1；MGMT-FALLBACK 向量仍 not-run（7 vs 4 verbs） |
| 5 | R1 结构化审批；三负例 dispatches=0 | **通过** | ApprovalGate + CFR F-011 三向量 **pass** |
| 6 | AKP HTTP JSON + SSE watch / cursor stale | **通过** | kernel-server + TSC HttpSseTransport + CFR SHELL-WATCH-RESUME **pass** |
| 7 | D-018 event envelope 组装 | **部分通过** | 组装器已交付；闭合仍待治理对象端口 + 更多 watch 证据 → D-018 仍 partially-implemented |
| 8 | TSC 客户端非 authority；通道隔离 | **通过（实现+测试；CHANNEL 向量仍 not-run）** | sdk-ts / agent-shell；live 3 |

## 3. 安全负例清单

- F-011 三负例：**行为 pass**（闭合 finding 行为部分）。
- Shell cancel pending / detach 不取消 / watch stale：**行为 pass**。
- 自检：33 corrupted gates 全翻 fail。

## 4. 未通过项与阻断

- 若干 M5 候选向量仍 not-run（intent-supersede、fallback 全动词、channel isolation、migration、delta scope）——**不阻断**以「实现已提供 + 关键行为负例已执行」定义的 M5 出口，但记入 M6 前持续清单。
- D-018 未完全闭合（治理对象端口）。
- Profile 符合 = 0（样例 manifest 仍 planned）。

## 5. 漂移与规范变更

- 无新漂移；F-011 → closed-by-M5-behavior；IMP-05 行为侧闭合。

## 6. 指标快照

| 口径 | 值 |
|---|---|
| 规范已登记 | 273 REQ / 55 码 / 61 schema / 84 向量 |
| 行为向量 pass | **52**（+6 M5） |
| not-run | **32** |
| 自检翻 fail | **33** |
| TS 客户端测试 | sdk-ts 72 / agent-shell 13 |

## 7. 结论

**GO 到 M6**，附带条件：

1. D-018 治理对象端口与剩余 shell/intent 向量继续由 KRN/RUN/CFR 消化，不阻塞 M6 入口的安装/适配主线。
2. Console/clients 实现 gate 仍 blocked（PoC/ADR）；与 M6 核心可并行 tracking-only。
3. F-017 仍为 M6 出口阻断（sandbox 平台矩阵）。
