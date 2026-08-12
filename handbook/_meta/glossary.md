---
doc_id: meta.glossary
locale: en
kind: meta
audience: [user, developer, ai]
generated: false
---

# Glossary / 术语表

Bilingual term table. English is the code-facing term; the Chinese column is the
handbook's consistent translation（中文列是手册统一译法；代码与合同中一律使用英文原
词）.

| Term | 中文 | Meaning (one line) |
|---|---|---|
| authority state | 权威状态 | durable facts only the daemon may write (A1) |
| candidate | 候选 | probabilistic output awaiting deterministic admission (A2) |
| admission | 准入 | deterministic acceptance of a candidate into authority state |
| Intent / Effect | 意图 / 效应 | persist-before-dispatch record pair for external mutations (A3) |
| idempotency key | 幂等键 | durable key making replays safe and conflicts detectable |
| fencing epoch | 围栏纪元 | monotonic writer generation; stale writers fail closed |
| CAS | 版本比较交换 | compare-and-swap on `expected_version` |
| TaskContract | 任务合同 | admitted, digest-bound definition of one Task |
| supersession | 更替 | epoch-advancing replacement that fences old work |
| WIA (Worker Iteration Authorization) | 工作迭代授权 | one-time authorization for one worker iteration |
| verified continuation | 已验证续期 | loop continuation minted only from verification |
| ContextRequest / ContextView | 上下文请求 / 视图 | authorized input request and its sealed resolved view |
| loss declaration | 损失声明 | explicit record of omitted/truncated context |
| Memory tombstone | 记忆墓碑 | append-only forget/expiry fact that survives rebuilds |
| Skill binding | 技能绑定 | scope-bound pin of an imported skill revision |
| native Tool | 原生工具 | one of six built-in governed operations |
| Provider | 模型服务商 | external LLM API reached only through the daemon proxy |
| SecretRef | 密钥引用 | opaque handle to material in the Secret Store (A5) |
| bootstrap secret | 引导密钥 | per-boot file exchanged for channel-bound bearers |
| channel (management/task) | 通道 | privilege class bound into every local bearer |
| sidecar session | 边车会话 | daemon-side lifecycle binding of a managed agent process |
| candidate socket | 候选套接字 | one-shot private socketpair for Pi candidate output |
| readiness / doctor | 就绪 / 诊断 | component projection vs redacted diagnostics |
| conformance vector | 符合性向量 | contract-derived behavioral test case |
| golden fixture | 金样 | cross-language canonical byte fixture |
| canonical JSON | 规范 JSON | deterministic encoding under registered digest domains |
| claim ceiling / non-claim | 声明上限 / 非声明 | highest honest evidence level; hypothesis-only record |
| Gate (B01…) | 门 | preregistered product acceptance campaign (owned by the plan) |
| lease | 租约 | exact-path write ownership in PARALLEL-LANES |
| Delivery Slice | 交付切片 | execution checkpoint inside one formal task |
| source fingerprint | 源指纹 | SHA-256 binding a handbook page to its mapped sources |
| source-set digest | 源集摘要 | digest of the whole implementation baseline tree |
