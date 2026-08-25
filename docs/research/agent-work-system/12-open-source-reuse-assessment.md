# CognitiveOS Personal 1.0 开源复用评估

Date: 2026-08-25
Status: **candidate / owner-confirmed scope input / non-canonical / no implementation authorization**

本文件记录 Personal Desktop 1.0 的开源项目复用研判。它以
[产品设计](./03-personal-product-design.md)、[候选架构](./05-personal-architecture.md)、
[共享边界](./09-shared-domain-and-contract-boundaries.md)和
[验证计划](./10-validation-and-delivery-readiness.md)为约束，不导入代码、资产、配置、状态或
依赖，不注册正式任务、ADR、contract、release 或 Gate。

标记：

- **FACT**：由 2026-08-25 官方仓库、release、文档、LICENSE/NOTICE 或安全公告支持。
- **INFERENCE**：由事实推导的适配结论。
- **RECOMMENDATION**：进入 PoC 前仍需 formal task、license/security review 与 exact-path lease。
- `DIRECT DEPENDENCY`、`OUT-OF-PROCESS ADAPTER`、`DATA IMPORT-EXPORT`、
  `UI-PATTERN REFERENCE`、`CONCEPT ONLY`、`REJECT` 是候选复用模式，不是已接受实现。

## 1. Executive decision

**RECOMMENDATION：不从任何功能相似项目迁移 authority code 或 authority state。**

没有一个评估项目同时提供：

1. daemon-only authority writer；
2. external dispatch 前 durable Intent；
3. Effect reconciliation；
4. epoch/CAS fencing 与 hard Task budget；
5. approved SecretStore-only credential custody；
6. independent verification before Task acceptance；
7. unknown-worktree protection。

因此：

| CognitiveOS surface | Replacement decision |
|---|---|
| Task / Intent / Effect / Evidence authority | **不替换** |
| Provider Control Plane / Binding | **不替换** |
| Memory admission/version/tombstone | **不替换** |
| Knowledge / Context authorization | **不替换** |
| Web client / product IA | **不替换** |
| Desktop shell | Tauri 仅是条件式 packaging candidate |
| External Agent / RAG / telemetry | 只允许可拆除的 out-of-process adapter |
| Usage/history import | 只允许 source-typed、read-only、可删除的 import |

## 2. 项目名称核验

### 2.1 Paperclip

**FACT**：本评估中的 Paperclip 指
[`paperclipai/paperclip`](https://github.com/paperclipai/paperclip)，不是同名 PDF 或文档项目。
研究基线为 release
[`v2026.817.0`](https://github.com/paperclipai/paperclip/releases/tag/v2026.817.0)，MIT。

### 2.2 CC Switch / `ccswitch`

名称有歧义：

1. [`farion1231/cc-switch`](https://github.com/farion1231/cc-switch)：
   Tauri desktop GUI，研究基线
   [`v3.20.0`](https://github.com/farion1231/cc-switch/releases/tag/v3.20.0)，MIT。
2. [`fairy-pitta/cc-account-switcher`](https://github.com/fairy-pitta/cc-account-switcher)：
   shell account switcher，研究基线
   [`v0.5.0`](https://github.com/fairy-pitta/cc-account-switcher/releases/tag/v0.5.0)，MIT。

两者不能因命令名相近而被视为同一项目。

### 2.3 Cockpit

**FACT**：本评估中的 Cockpit 指
[`cockpit-project/cockpit`](https://github.com/cockpit-project/cockpit)，Linux server Web
console，研究基线
[`366`](https://github.com/cockpit-project/cockpit/releases/tag/366)。它不是 AI runtime 或
Personal desktop shell。

## 3. Evidence-ranked shortlist

“Adopt”表示允许进入有边界 PoC，不表示生产接受。

| Rank | Project | Candidate mode | 主要价值 | 当前结论 |
|---:|---|---|---|---|
| 1 | [Tauri 2.11.5](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.5) | DIRECT DEPENDENCY | 包装现有 React/Vite client | 条件式 ADOPT |
| 2 | [MCP TypeScript SDK 2.0.0](https://github.com/modelcontextprotocol/typescript-sdk/releases/tag/%40modelcontextprotocol/core%402.0.0) | DIRECT DEPENDENCY | 避免重写 MCP framing/schema/negotiation | post-1.0 ADOPT candidate |
| 3 | [ccusage 20.0.20](https://github.com/ccusage/ccusage/releases/tag/v20.0.20) | DATA IMPORT-EXPORT | 本地 token/cost observation import | ADAPT |
| 4 | [OpenHands 1.15.0](https://github.com/OpenHands/OpenHands/releases/tag/v1.15.0) | OUT-OF-PROCESS ADAPTER | coding Agent/workspace/event path | ADAPT after qualification |
| 5 | [LiteLLM 1.98.0](https://github.com/BerriAI/litellm/releases/tag/v1.98.0) | OUT-OF-PROCESS ADAPTER | Provider wire normalization/price metadata | cautious ADAPT |
| 6 | [RAGFlow 0.27.0](https://github.com/infiniflow/ragflow/releases/tag/v0.27.0) | OUT-OF-PROCESS ADAPTER | parsing/rebuildable retrieval index | ADAPT |
| 7 | [Mem0 2.0.18](https://github.com/mem0ai/mem0/releases/tag/v2.0.18) | OUT-OF-PROCESS ADAPTER | Memory candidate extraction/search | ADAPT |
| 8 | [OpenLLMetry 0.62.3](https://github.com/traceloop/openllmetry/releases/tag/0.62.3) | DIRECT DEPENDENCY in adapter | redacted OTel observations | bounded ADOPT |

只有 Tauri 接近 Desktop 1.0 的近期依赖候选；其余项目不进入 authority process，并且必须
能在不迁移 CognitiveOS authority state 的情况下移除。

## 4. Provider、account、subscription 与 usage

### 4.1 CC Switch GUI

| Item | Assessment |
|---|---|
| Repository | [`farion1231/cc-switch`](https://github.com/farion1231/cc-switch) |
| License | [MIT](https://github.com/farion1231/cc-switch/blob/main/LICENSE) |
| Useful for | Provider/model/MCP/Skill/tray/setup interaction reference |
| Mode | `UI-PATTERN REFERENCE` |
| Do not reuse | SQLite SSOT、credential/config writer、Provider presets、icons、branding |

**FACT**：官方文档说明其使用 `~/.cc-switch/cc-switch.db` 并写 live CLI configuration。
公开安全报告涉及 plaintext API key 与 file-permission 风险。

**RECOMMENDATION**：只研究 Provider selector、configuration diff、tray 与切换反馈。不得复制
secret storage、自动 fallback 或 config mutation；CognitiveOS Binding、SecretStore 与
daemon validation 继续 authoritative。

### 4.2 CC account switcher

| Item | Assessment |
|---|---|
| Repository | [`fairy-pitta/cc-account-switcher`](https://github.com/fairy-pitta/cc-account-switcher) |
| License | MIT |
| Mode | `REJECT` |
| Reason | credential profile rotation、settings file mutation、automatic fallback |

它只能作为多 account 切换需求的弱证据，不能成为实现或 credential migration 来源。

### 4.3 ccusage

| Item | Assessment |
|---|---|
| Repository | [`ccusage/ccusage`](https://github.com/ccusage/ccusage) |
| Release | [`v20.0.20`](https://github.com/ccusage/ccusage/releases/tag/v20.0.20) |
| License | [MIT](https://github.com/ccusage/ccusage/blob/main/LICENSE) |
| Mode | `DATA IMPORT-EXPORT` |

PoC requirements：

- 只读取 owner 显式选择的 sanitized fixture/path，不扫描 ambient home；
- imported row 标 `external_local_log`、source tool、period、parser version、coverage；
- estimate 保留 price source/version；
- missing price 映射 `unknown`，绝不映射 0；
- imported usage 不成为 invoice、entitlement 或 hard budget charge；
- import 可重复、可删除、可重新构建。

### 4.4 LiteLLM

| Item | Assessment |
|---|---|
| Repository | [`BerriAI/litellm`](https://github.com/BerriAI/litellm) |
| Release | [`1.98.0`](https://github.com/BerriAI/litellm/releases/tag/v1.98.0) |
| License | MIT core；`enterprise/` commercial |
| Mode | `OUT-OF-PROCESS ADAPTER` |

可借用 Provider wire normalization 和 price metadata，但必须禁用或 subordinate：

- LiteLLM routing/fallback；
- virtual-key authority；
- upstream DB as canonical store；
- upstream budget enforcement；
- prompt/content logging；
- environment/config secret injection。

Acceptance：exact image/package digest、no Provider secret outside approved handoff、fixed model
binding、no automatic route/fallback、daemon usage ledger remains canonical、adapter loss does not
change Task/Binding authority。

### 4.5 Subscription gap

**FACT**：没有发现能统一、可靠读取各 consumer plan entitlement/remaining allowance 的开源
项目。Local usage 不能推导 entitlement。

产品必须继续使用：

- `Managed here`
- `Link/Reauthenticate`
- `Observe read-only`
- `Open Provider`
- `Record manually`
- `Unavailable`

而不是发明通用 Subscription mutation。

## 5. Desktop、Conversation 与 history reference

### 5.1 Jan

| Item | Assessment |
|---|---|
| Repository | [`janhq/jan`](https://github.com/janhq/jan) |
| Release | [`0.8.4`](https://github.com/janhq/jan/releases/tag/v0.8.4) |
| License | [Apache-2.0](https://github.com/janhq/jan/blob/main/LICENSE) |
| Mode | `UI-PATTERN REFERENCE` |

参考 Provider setup、model selection、history search 与 local-first explanation。不得迁移其
chat/runtime state；desktop loopback trusted-host 和 downgrade snapshot 风险必须进入 threat
review。

### 5.2 LibreChat

| Item | Assessment |
|---|---|
| Repository | [`danny-avila/LibreChat`](https://github.com/danny-avila/LibreChat) |
| Release | [`0.8.8-rc1`](https://github.com/danny-avila/LibreChat/releases/tag/v0.8.8-rc1) |
| License | [MIT](https://github.com/danny-avila/LibreChat/blob/main/LICENSE) |
| Mode | `UI-PATTERN REFERENCE` / `DATA IMPORT-EXPORT` candidate |

可参考 history search、Provider comparison、MCP/Agent presentation。History import 只能成为带
provenance 的 archive；embedded tool、secret、MCP config 必须 inert。安全公告中的 OAuth
token theft 与 environment-secret exfiltration 要进入 import threat test。

### 5.3 Cherry Studio

| Item | Assessment |
|---|---|
| Repository | [`CherryHQ/cherry-studio`](https://github.com/CherryHQ/cherry-studio) |
| Release | [`2.0.8`](https://github.com/CherryHQ/cherry-studio/releases/tag/v2.0.8) |
| License | custom dual-license / AGPL-derived with organization-size condition |
| Mode | `REJECT` source migration；reference only after trademark review |

不得复制 source、theme、Provider preset、assistant prompt、icon 或 asset。

### 5.4 Open WebUI

| Item | Assessment |
|---|---|
| Repository | [`open-webui/open-webui`](https://github.com/open-webui/open-webui) |
| Release | [`0.11.0`](https://github.com/open-webui/open-webui/releases/tag/v0.11.0) |
| License | custom Open WebUI License，存在历史 MIT/BSD strata |
| Mode | `REJECT` |

Branding restriction、file-level provenance 复杂度以及 OAuth/RCE/SSRF advisories 使 source/UI
迁移不成立。

### 5.5 Conversation import rule

任何 upstream history：

1. 先进入 read-only archive candidate；
2. 保留 source tool/version/conversation/turn range/digest/import loss；
3. 不自动成为 Memory、Context、Evidence 或 Task completion；
4. secret、hidden state、unauthorized body 不进入 package；
5. delete/export/reimport 有 deterministic receipt。

## 6. Agent work 与 orchestration

### 6.1 Paperclip

| Item | Assessment |
|---|---|
| Repository | [`paperclipai/paperclip`](https://github.com/paperclipai/paperclip) |
| License | MIT |
| Mode | `CONCEPT ONLY` / `UI-PATTERN REFERENCE` |

吸收 atomic checkout、wakeup reason、attempt timeline、blocker、budget、confirmation 与 recovery
pattern。拒绝 Company/Employee metaphor、Agent task-state mutation、adapter-reported cost as
truth、second scheduler 和 process result as Task completion。

### 6.2 OpenHands

| Item | Assessment |
|---|---|
| Repository | [`OpenHands/OpenHands`](https://github.com/OpenHands/OpenHands) |
| Release | [`1.15.0`](https://github.com/OpenHands/OpenHands/releases/tag/v1.15.0) |
| License | MIT core；`enterprise/` Polyform Free Trial |
| Mode | `OUT-OF-PROCESS ADAPTER` |

适合 coding persona 的隔离 PoC。Required controls：

- one Task / one isolated workspace；
- no Docker socket、ambient home、credential directory；
- exact adapter/package identity；
- daemon-issued bounded tool permits；
- before/after worktree digest；
- A8 unknown-change rejection；
- Agent event/result remains observation；
- independent post-state verifier；
- command-injection、network、timeout、crash、duplicate request negatives。

### 6.3 LangGraph

| Item | Assessment |
|---|---|
| Repository | [`langchain-ai/langgraph`](https://github.com/langchain-ai/langgraph) |
| Release baseline | `1.2.11` / SDK `0.4.3` |
| License | MIT |
| Mode | `CONCEPT ONLY` |

Checkpoint/durable-loop pattern 有参考价值，但 interrupted node side effects 可能重跑，且历史
advisory 涉及 unsafe checkpoint deserialization。不得替换 CognitiveOS scheduler、Task 或
Effect state machine。

## 7. Knowledge 与 RAG

### 7.1 RAGFlow

| Item | Assessment |
|---|---|
| Repository | [`infiniflow/ragflow`](https://github.com/infiniflow/ragflow) |
| Release | [`0.27.0`](https://github.com/infiniflow/ragflow/releases/tag/v0.27.0) |
| License | [Apache-2.0](https://github.com/infiniflow/ragflow/blob/main/LICENSE) |
| Mode | `OUT-OF-PROCESS ADAPTER` |

定位为 disposable/rebuildable derived index：

- immutable content-addressed input；
- daemon authorization before query；
- source/version/classification/provenance preserved；
- bounded result evidence；
- no credential in authority DB；
- delete/rebuild/purge denominator；
- unavailable adapter does not remove source registry；
- upstream index 永不成为 source or policy SoR。

### 7.2 AnythingLLM

| Item | Assessment |
|---|---|
| Repository | [`Mintplex-Labs/anything-llm`](https://github.com/Mintplex-Labs/anything-llm) |
| Release | [`v1.16.0`](https://github.com/Mintplex-Labs/anything-llm/releases/tag/v1.16.0) |
| License | MIT core；部分 component 另有 AGPL |
| Mode | `CONCEPT ONLY` / `REJECT for 1.0` |

Whole-instance state 跨 DB、storage、vector backend，不适合作为 portable Knowledge authority。
公开 IDOR 风险强化了 source ownership/ACL test 的必要性。

### 7.3 Dify

| Item | Assessment |
|---|---|
| Repository | [`langgenius/dify`](https://github.com/langgenius/dify) |
| Release | [`1.16.1`](https://github.com/langgenius/dify/releases/tag/1.16.1) |
| License | modified Apache；multi-tenant/branding restrictions |
| Mode | `REJECT` |

不得复制 Web UI、workflow component、DSL 或 secret-bearing export。

## 8. Memory

### 8.1 Mem0

| Item | Assessment |
|---|---|
| Repository | [`mem0ai/mem0`](https://github.com/mem0ai/mem0) |
| Release | [`2.0.18`](https://github.com/mem0ai/mem0/releases/tag/v2.0.18) |
| License | [Apache-2.0](https://github.com/mem0ai/mem0/blob/main/LICENSE) |
| Mode | `OUT-OF-PROCESS ADAPTER` |

Mem0 output 只能是 Memory candidate。PoC 必须证明：

- daemon assigns source/scope/purpose；
- deterministic admission and immutable version；
- explicit preview before retention；
- forget + tombstone prevents resurrection；
- delete-all completeness；
- vector service unavailable fails closed；
- full portable export；
- extractor cannot widen authority or Context。

### 8.2 Graphiti

| Item | Assessment |
|---|---|
| Repository | [`getzep/graphiti`](https://github.com/getzep/graphiti) |
| Release | [`0.29.3`](https://github.com/getzep/graphiti/releases/tag/v0.29.3) |
| License | Apache-2.0 |
| Mode | `CONCEPT ONLY` |

Temporal provenance/edge pattern 有价值；graph Memory 仍超出 1.0 baseline，不得借适配器绕过
正式产品决策。

### 8.3 Letta

| Item | Assessment |
|---|---|
| Repository | [`letta-ai/letta-code`](https://github.com/letta-ai/letta-code) |
| Release | [`0.30.6`](https://github.com/letta-ai/letta-code/releases/tag/v0.30.6) |
| License | Apache-2.0 |
| Mode | `DATA IMPORT-EXPORT` / `CONCEPT ONLY` |

AgentFile 可作为 portability 参考；agent-writable block 与 last-write-wins 不能替代 Memory
admission/CAS。Imported tools/source/environment 必须 inert。

## 9. Skills、Tools 与 MCP

### 9.1 MCP TypeScript SDK

| Item | Assessment |
|---|---|
| Repository | [`modelcontextprotocol/typescript-sdk`](https://github.com/modelcontextprotocol/typescript-sdk) |
| Release | [`@modelcontextprotocol/core@2.0.0`](https://github.com/modelcontextprotocol/typescript-sdk/releases/tag/%40modelcontextprotocol/core%402.0.0) |
| License | 新 code Apache-2.0；未 relicensed old code MIT；docs CC-BY-4.0 |
| Mode | `DIRECT DEPENDENCY` after formal activation |

必须 pin package、spec revision、protocol digest 并复核 file-level provenance。MCP descriptor/result
仍是 discovery/observation；daemon Tool descriptor、permission、Intent/Effect 和 verifier 保持
authoritative。

### 9.2 MCP Inspector

| Item | Assessment |
|---|---|
| Repository | [`modelcontextprotocol/inspector`](https://github.com/modelcontextprotocol/inspector) |
| Release | [`2.3.0`](https://github.com/modelcontextprotocol/inspector/releases/tag/2.3.0) |
| Mode | `CONCEPT ONLY` / isolated qualification tool |

其 proxy 可启动本地 process，历史 XSS 可导致 command execution。只能在 disposable
qualification environment 使用，不能成为 product runtime。

### 9.3 MCP Registry

| Item | Assessment |
|---|---|
| Repository | [`modelcontextprotocol/registry`](https://github.com/modelcontextprotocol/registry) |
| Release | [`1.8.1`](https://github.com/modelcontextprotocol/registry/releases/tag/v1.8.1) |
| Mode | `CONCEPT ONLY` / `REJECT automatic install` |

Registry presence 不是 trust、qualification、permission 或 compatibility proof。

## 10. Observability 与 token/cost

### 10.1 Langfuse

| Item | Assessment |
|---|---|
| Repository | [`langfuse/langfuse`](https://github.com/langfuse/langfuse) |
| Release | [`4.17.0`](https://github.com/langfuse/langfuse/releases/tag/v4.17.0) |
| License | MIT core；`ee/` commercial |
| Mode | `OUT-OF-PROCESS ADAPTER` candidate，not shortlist |

若使用，只能作为 optional redacted sink。Prompt/completion/secret 不发送；sink loss 不影响执行；
trace 不成为 Evidence/Task authority；telemetry off；delete/export proven。

### 10.2 OpenLLMetry

| Item | Assessment |
|---|---|
| Repository | [`traceloop/openllmetry`](https://github.com/traceloop/openllmetry) |
| Release | [`0.62.3`](https://github.com/traceloop/openllmetry/releases/tag/0.62.3) |
| License | Apache-2.0 |
| Mode | `DIRECT DEPENDENCY in adapter` |

官方 instrumentation 默认可能捕获 prompt、completion、embedding。Acceptance 强制
`TRACELOOP_TRACE_CONTENT=false`、schema allowlist、local collector、secret canary，并证明
span 不能推进 Task。

## 11. Desktop shell 与 admin pattern

### 11.1 Tauri

| Item | Assessment |
|---|---|
| Repository | [`tauri-apps/tauri`](https://github.com/tauri-apps/tauri) |
| Release | [`2.11.5`](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.5) |
| License | dual MIT / Apache-2.0 |
| Mode | `DIRECT DEPENDENCY` candidate |

Tauri 只作为 existing Web client 的 thin shell。PoC gate：

- no authority DB in shell；
- strict IPC allowlist；
- CSP、no remote content；
- memory-only renderer auth；
- no generic file/network/exec/secret commands；
- signed install/update/rollback；
- updater/source provenance；
- WebView origin-confusion regression；
- Windows accessibility、deep links、tray、crash recovery；
- cold/warm startup、RSS、package size；
- future macOS/Linux separately qualified。

### 11.2 Electron

| Item | Assessment |
|---|---|
| Repository | [`electron/electron`](https://github.com/electron/electron) |
| Release | [`v44.0.0`](https://github.com/electron/electron/releases/tag/v44.0.0) |
| License | MIT |
| Mode | `REJECT` unless fixed spike disproves current evidence |

现有 Rust composition + Tauri candidate 已满足 packaging need；Electron 增加 privileged
Chromium/Node runtime 与 patch burden。仍保留固定 spike 的公平 comparison，不提前伪造 winner。

### 11.3 Cockpit

| Item | Assessment |
|---|---|
| Repository | [`cockpit-project/cockpit`](https://github.com/cockpit-project/cockpit) |
| Release | [`366`](https://github.com/cockpit-project/cockpit/releases/tag/366) |
| License | mixed LGPL-2.1+/GPL-3/MIT/BSD/CC-BY-SA；branding/trademark constraints |
| Mode | `UI-PATTERN REFERENCE` |

参考 status、forms、tables、reconnect、degraded-state UX。不得嵌入 privileged Linux bridge、
source、icons 或 OS branding。

## 12. Adopt / Adapt / Reject summary

### Adopt conditionally

- Tauri：Desktop packaging only。
- MCP TypeScript SDK：dynamic MCP formal activation 后的 official protocol implementation。
- OpenLLMetry：仅 adapter 内的 redacted telemetry。

### Adapt

- ccusage：sanitized read-only usage importer。
- OpenHands：isolated coding Agent adapter。
- LiteLLM：fixed transport adapter；no routing/fallback/secret/budget authority。
- RAGFlow：rebuildable derived index。
- Mem0：Memory candidate extractor。
- Jan、LibreChat、CC Switch、Cockpit：interaction/import/reference only。
- Paperclip、LangGraph、Graphiti、Letta：concept/portability reference。

### Reject source migration

- Cherry Studio：custom license / commercial condition。
- Open WebUI：custom license、branding/provenance/security surface。
- Dify：modified license、product overlap、secret/export risk。
- AnythingLLM：1.0 whole-instance/product overlap 与 state portability risk。
- account-switching scripts：credential/config mutation and fallback。

## 13. Universal no-copy list

即使主 license permissive，也不得未经独立核验复制：

- project name、logo、icon、screenshot、illustration、theme；
- Provider preset、prompt、persona、sample data；
- model weight、dataset、embedding/index；
- registry content、community package；
- generated code 或 vendored dependency；
- `enterprise/`、`ee/`、commercial path；
- docs text（可能是 CC 或不同 license）；
- secret/config/database/history archive；
- trademarked OS/provider visual language。

## 14. Common PoC acceptance

所有候选必须满足：

1. exact tag/commit/package/digest；
2. stable interface 与 upstream security owner；
3. external mutation 仍由 daemon authorize、fence、Intent-before-dispatch；
4. upstream result/trace/checkpoint 不完成 Task；
5. no secret in env/argv/config/browser/upstream DB/log/trace/evidence；
6. import 带 provenance/digest/scope/loss；
7. unknown workspace changes fail closed；
8. crash/timeout/duplicate/stale epoch/unknown outcome negatives；
9. export/delete/rollback/rebuild proven；
10. dependency removal 不迁移 CognitiveOS authority state；
11. no mutable `latest`；
12. capability absent 时 honest unavailable。

## 15. License due diligence

- 记录 exact source tag、commit、package、artifact digest、retrieval date。
- 逐 revision 复核 root/per-directory LICENSE、NOTICE、LICENSE_HISTORY、vendored/generated files。
- MIT/BSD：保留版权与许可文本。
- Apache-2.0：保留 license、NOTICE、modification notice、patent terms。
- LGPL：保持 replaceability/linking obligation，为修改组件提供对应 source。
- GPL/AGPL：评估 derivative/network corresponding-source obligation。
- Custom/source-available 条款不得简称 AGPL/Apache。
- 排除 commercial/enterprise paths，除非另有 license。
- 复核 CLA/relicensing power 与 license transition boundary。
- 单独进行 trademark、logo、icon、screenshot、theme review。
- model、dataset、prompt、preset、generated code 分别核验。
- Mixed-license binary 发行前需要法律审查。

## 16. SBOM、provenance 与升级治理

每个 adopted artifact：

- 生成 SPDX 2.3 与 CycloneDX 1.6 SBOM；
- 记录 purl、version、commit、dependency graph、license expression、NOTICE、hash；
- 包含 toolchain、lockfile、container base、native library、optional feature；
- container image pin digest；
- 验证 signature/attestation；
- 生成 source revision→builder→recipe→output provenance；
- 运行 license/vulnerability/secret/malicious-package scan；
- 订阅 upstream advisory；
- 指定 patch/rollback owner 与响应时限；
- 保存 reproducible source bundle 和 redistribution material；
- 每次升级重新评审，不继承旧版本批准。

## 17. Future adoption runbook

1. 建立 Candidate ADR，写明问题和“不采用”基线。
2. 冻结 exact upstream revision、license inventory、security owner。
3. 在 ignored/disposable environment 建立 PoC，不连接真实 secret/data。
4. 运行本文件 §14 negatives 与 domain-specific acceptance。
5. 证明 dependency 可移除、state 可重建/导出/删除。
6. 决定 direct dependency、adapter、import 或 reference。
7. 若需要 public contract，进入 Lane-CTR；不得在 adapter 内发明第二协议 authority。
8. 生成 SBOM/provenance/NOTICE。
9. 将正式 task、writable paths、validation route、rollback 注册后才能实现。
10. 每次升级重新 qualification；security regression 触发 disable/rollback。

## 18. Non-claims

- 本评估没有 clone、copy、install、import、migration 或 dependency change。
- 没有 upstream code、asset、state 或 credential 进入仓库。
- 没有 formal task、ADR、contract、branch、PR、release、Profile 或 Gate。
- Project release/version 是 2026-08-25 research snapshot，不保证未来不变。
- 开源项目的功能存在不证明 CognitiveOS capability 已实现。

