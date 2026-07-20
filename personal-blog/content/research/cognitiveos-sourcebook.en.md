# CognitiveOS Research Sourcebook (en)

> Purpose: a reviewable authoring ledger for the blog article and local diagrams. It is not routed publicly, is not a specification asset, and supplies no implementation evidence.
>
> Snapshot captured at `2026-07-20T07:41:57+08:00`; parent HEAD `b626e88be3b985399051e6e7624223b9cb38e7c6` (merge commit timestamp `2026-07-20T07:30:03+08:00`). The tracked parent source paths read for this ledger were clean at capture; authorized `personal-blog/` changes are outside the parent-source status.
>
> Hash notice: every SHA-256 below is a **research snapshot file hash** only. It is not a registered CognitiveOS specification-set, requirement-set, schema-bundle, or conformance digest.

## 1. Source tiers and use

1. **Tier 1 — registered machine and test assets:** registries, schemas, transition JSON, and declarative conformance vectors. A schema governs the shape it can express; a transition or vector does not prove that behaviour ran.
2. **Tier 2 — normative behaviour:** versioned Core, RFCs, and standards. These define authority, order, failure, recovery, and acceptance semantics. Draft claims must name a version and snapshot.
3. **Tier 3 — informative architecture:** `CognitiveOS-Architecture.md` and product explanation. These explain the dual kernel, three planes, seven layers, and design rationale, but do not override Tiers 1–2.
4. **Tier 4 — plan and status evidence:** PROGRESS, findings ledger, and handoff. These describe one engineering moment and create no normative requirement.

When sources disagree, use the interpretation that does not enlarge authority, data scope, risk, budget, or completion claims. Public copy says “the draft requires,” “the transition table defines,” or “the research snapshot shows,” never an unsupported “the system guarantees.”

## 2. Problem, scope, and non-goals

### FACT-COS-001 — Problem boundary

CognitiveOS addresses agent workloads that persist over time, combine probabilistic reasoning with deterministic programs, and may change a digital or physical world. The core problem is not adding model calls; it is fixing state interpretation, execution identity, Context visibility, authorization, external Effects, recovery, and acceptance.

- Source: `CognitiveOS-Architecture.md` §0 “执行摘要” and §1.4 “明确非目标”
- Tier: 3 informative
- Snapshot hash: `e71b5dd83549ec7b7eb278e609d22fcea503279ef5918e3aceab10a62dd54b0c`
- Boundary: it does not replace a host OS, database, messaging system, infrastructure orchestrator, industry certification, or intelligent algorithm, and does not claim a theory of general intelligence.

### FACT-COS-002 — An architecture name cannot supply guarantees

Models and verifiers can still be wrong; general cross-system side effects cannot promise unconditional exactly-once behaviour; safety, performance, SLOs, RPO/RTO, and industry certification require deployment evidence.

- Sources: `CognitiveOS-Architecture.md` §1.5; `specs/core/README.md` §7
- Tier: 2 + 3, with Core taking precedence for behaviour
- Core hash: `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`

## 3. Informative overall architecture

### FACT-COS-003 — Dual kernel, three planes, seven layers, cross-cutting Context

The whitepaper offers four orthogonal explanatory views:

- **Dual kernel:** a cognitive microkernel for durable execution, state, Context gates, authorization, Effects, budgets, and recovery; a real-time safety kernel for hard real-time envelopes, watchdogs, emergency stop, and final actuator arbitration.
- **Three planes:** experience; control; execution and data.
- **Seven layers:** (1) host/network/device/physical world; (2) resource fabric and heterogeneous compute; (3) operations/skills/runtime; (4) cognitive microkernel and AKP; (5) Context/state/knowledge/memory/catalogue; (6) Harness and cognitive services; (7) agents and applications.
- **Context Engineering:** a service cutting across planes and layers, not a fourth authority plane.

- Source: `CognitiveOS-Architecture.md` §4.1–§4.5
- Tier: 3 informative
- Hash: `e71b5dd83549ec7b7eb278e609d22fcea503279ef5918e3aceab10a62dd54b0c`
- Public limit: call it an “informative whitepaper view,” not a deployed dual kernel or proven real-time safety mechanism.

## 4. Probabilistic proposal and deterministic authority

### FACT-COS-004 — Probabilistic components produce candidates only

LLMs, retrieval, embeddings, rankers, matchers, and summarizers may discover, reorder, shrink, or transform candidates. Deterministic mechanisms perform authorization, CAS, schema checks, state-machine legality, hard budgets, idempotency decisions, fencing, and final commits.

- Source: `specs/core/README.md` §2 (`REQ-CHARTER-DET-001`), §6.5, and §15.5 (`REQ-CORE-CANDIDATE-001`)
- Tier: 2 normative behaviour
- Hash: `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`

### FACT-COS-005 — Authority roles

The target state-domain authority owns writes and arbitration; task-acceptance-authority owns Task completion; effect-authority and verification-authority own their lifecycles. Observations, remote reports, receipts, model narratives, and ContextView do not receive authority by default.

- Sources: `specs/registry/state-domains.yaml` `domains[*].authority_role`; `specs/core/README.md` §4–§5; `specs/transitions/task.transitions.json`
- Tier: 1 + 2
- Registry hash: `17ee88cabe13b0c539559e507e36c417455eeb6204a1907848f9c17757296d40`

## 5. OperationDescriptor and AuthorizationCapability

### FACT-COS-006 — Description and permission remain separate

`OperationDescriptor` answers what an endpoint can do and how to call it: input/output schema, effect class, idempotency, cancellation, query/reconciliation, version, endpoint, and limits. `AuthorizationCapability` answers who may do what, for which purpose, against which resources and parameters, and until when. Local authority issues capability; derivation may only narrow it.

- Source: `specs/core/README.md` §4 and §8 (`REQ-OP-001/002`, `REQ-CAP-001..005`)
- Tier: 2
- Hash: `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`
- Machine limit: related operation-summary, catalogue-snapshot, and match-report schemas exist, but there is no complete `OperationDescriptor` schema.
- Open item: F-023’s admission matrix for non-queryable or non-idempotent executors remains open.

## 6. Five independent execution lifecycles

### FACT-COS-007 — Domains, initial states, and state names

The five domains are independently persisted and may not be collapsed into one progress value.

1. **agent-execution**, `execution-authority`, initial `CREATED`: `CREATED, ADMITTED, RUNNABLE, WAITING, CHECKPOINTED, RECOVERING, SUSPENDED, QUARANTINED, TERMINATED`.
2. **task**, `task-acceptance-authority`, initial `DRAFT`: `DRAFT, READY, ACTIVE, BLOCKED, CANDIDATE_COMPLETE, COMPLETED, FAILED, CANCELLED, ESCALATED`.
3. **loop**, `execution-authority`, initial `START`: `START, OBSERVE, RESOLVE, ORIENT, DECIDE, ACT, VERIFY, CONTINUE, DIAGNOSE, WAIT, QUARANTINE, RECONCILE, ESCALATE, STOP, END`.
4. **effect**, `effect-authority`, initial `PROPOSED`: `PROPOSED, AUTHORIZED, DENIED, EXECUTING, EXECUTED, OUTCOME_UNKNOWN, RECONCILED, VERIFIED, VERIFY_FAILED, COMPENSATING, NOT_EXECUTED, COMMITTED, ABORTED, QUARANTINED`.
5. **verification**, `verification-authority`, initial `NOT_REQUESTED`: `NOT_REQUESTED, PENDING, EVIDENCE_READY, PASSED, FAILED, INCONCLUSIVE, EXPIRED`.

- Sources: `specs/registry/state-domains.yaml` and all five `specs/transitions/*.transitions.json`
- Tier: 1
- Transition hashes:
  - agent-execution `d29c2926ee9a2ceae945201df25e712ddfbc65000323b24af70466e81652f616`
  - task `770484aeb2b04f2afd76e07512fb01b5b9760517ce5b879bb58714ed1b375f17`
  - loop `db836e190920cc008804b8a195f3c01cf2cf696039003914464badd0539509ef`
  - effect `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e`
  - verification `4975c774094ae4f48cd4217497bde27cb71005be41bfd2a1df5ea21aa65e0ade`

### FACT-COS-008 — CANDIDATE_COMPLETE is not completion

A Task moves from `CANDIDATE_COMPLETE` to `COMPLETED` only through `ACCEPTANCE_GRANTED`. Its guards require matching acceptance authority, current passed Verification, and an unchanged fixed post-state. Failure, expiry, insufficient evidence, or dispute follows return, block, fail, or escalation paths.

- Sources: `specs/transitions/task.transitions.json` `CANDIDATE_COMPLETE -> COMPLETED`; `docs/standards/task-loop-verification.md` §5
- Tier: 1 + 2
- Standard hash: `5075759a0ce6940707bf47b65d7b930819b841978996307620345f8c93996a42`

## 7. ContextView and two nine-stage vocabularies

### FACT-COS-009 — ContextView is non-authority

A ContextView is a short-lived Activity-bound working projection reporting loaded/rejected items, loss, pinned versions, cost, lineage, and completeness. Authorization, CAS, transitions, and commits re-check authority state. Untrusted content cannot promote itself from data to control.

- Sources: `specs/core/README.md` §6.4–§6.5; `docs/standards/context-resolution-and-cache.md` §6
- Tier: 2
- Standard hash: `e973a36801a5e66393a1942186a5b1a82f80e200393e331d24f23fb2f95683a8`

### FACT-COS-010 — Nine-stage vocabulary discrepancy

- Core: `discover → filter → authorize → rank → budget → transform → verify → render → audit`.
- Context standard: `ContextRequest admission → governance pre-filter → candidate retrieval → per-object authorization re-validation → semantic ranking/selection → budget fitting → loss declaration → deterministic rendering → ContextView emission with provenance`.

The standard emphasizes implementation order; Core provides conceptual stages. They must not be silently treated as one-to-one aliases. Diagram 5 shows both and labels the discrepancy.

- Sources: `specs/core/README.md` §6.3; `docs/standards/context-resolution-and-cache.md` §2
- Tier: 2
- Hashes: Core `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`; standard `e973a36801a5e66393a1942186a5b1a82f80e200393e331d24f23fb2f95683a8`

### FACT-COS-011 — ContextRequest prose/shape mismatch

Core, RFC, and standard prose refer to governance bindings across Tenant, ActorChain, Conversation/Task, ActivityContext, and ResourceScope. `context-request.schema.json` provides a governed header, but `perspective` directly requires only `principal`, `task`, and `episode`; it does not directly encode every prose binding in that object. The blog must not invent fields to close the mismatch.

- Sources: `specs/schemas/context-request.schema.json` `/properties/header` and `/properties/perspective`; `specs/core/README.md` §0 and §6.2; Context standard §2 and §4
- Tier: Tier 1 for shape, Tier 2 for behaviour
- Schema hash: `080747d0f1510cab9d94c1cca30dae43bda9f51f2eb2a2a75ac16cad9062b104`

## 8. Intent, Effect, idempotency, unknown outcomes, reconciliation

### FACT-COS-012 — No Intent, no dispatch

A governed external side effect is preceded by persisted Intent fixing a stable idempotency key, parameter digest, expected state version, and authorization binding. Persisting Intent and its event forms one atomic commit; an unavailable authoritative store fails before effect.

- Source: `docs/standards/intent-effect-idempotency.md` §2, §3, and §6
- Tier: 2
- Hash: `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c`

### FACT-COS-013 — Reconcile OUTCOME_UNKNOWN first

A timeout, disconnect, or missing receipt after dispatch means execution may have occurred and enters `OUTCOME_UNKNOWN`. There is no direct transition to `VERIFIED/COMMITTED`, and recovery cannot mint a new idempotency key for blind retry. Reconciliation binds the original key and records executed, not_executed, or still_unknown.

- Sources: `specs/transitions/effect.transitions.json` `EXECUTING -> OUTCOME_UNKNOWN` and `OUTCOME_UNKNOWN -> RECONCILED`; Intent/Effect standard §4
- Tier: 1 + 2
- Hashes: transition `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e`; standard `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c`; negative vector `effect-unknown-outcome.json` `6364e3eadce30c7918b83087ae8bf6a7779ce7d945508ab5e1d8f5e3da69c512`

### FACT-COS-014 — Quarantine and separately authorized compensation

A still-unknown result does not rejoin the ordinary success path. It enters quarantine or starts separately authorized compensation. Compensation is a new governed Effect with `compensation_intent` and independent authorization; it does not inherit the original capability.

- Sources: effect transitions `RECONCILED -> COMPENSATING|QUARANTINED`; Intent/Effect standard §4
- Tier: 1 + 2
- Hashes: effect table `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e`; standard `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c`

## 9. Verification and Acceptance

### FACT-COS-015 — Verification is independent and can expire

Verification binds subject, criteria, verifier version, and fixed post-state. It can be `PASSED`, `FAILED`, or `INCONCLUSIVE`; even `PASSED` can become `EXPIRED` after post-state change, evidence invalidation, or verifier revocation.

- Source: `specs/transitions/verification.transitions.json`
- Tier: 1
- Hash: `4975c774094ae4f48cd4217497bde27cb71005be41bfd2a1df5ea21aa65e0ade`

### FACT-COS-016 — Acceptance separately advances completion

Remote completed, receipts, tool exit codes, and model self-report are not acceptance. Task acceptance authority consumes current Verification evidence and decides completion.

- Sources: `docs/standards/task-loop-verification.md` §5; vectors `remote-completed-not-acceptance.json` and `intent-acceptance-007.json`
- Tier: 2 + Tier 1 normative-test; vectors remain not-run
- Limit: no complete `AcceptanceDecision` schema exists in the current schema directory.
- Hashes: standard `5075759a0ce6940707bf47b65d7b930819b841978996307620345f8c93996a42`; remote-completed vector `b7d8df6d38111452d868520972306578e57c051de3e8e6f885fb23f8edb320b1`; intent-acceptance vector `ad859a72d8aa1e2670be9e75290c0e7a33edaed09cca79cf10c400bcc79c3c6c`

## 10. Current four status categories and counts

### FACT-COS-017 — M1 engineering status at b626e88

| Category | Snapshot fact | Does not mean |
|---|---:|---|
| Specified | 273 REQs; 55 error codes; 56 schemas; 5 lifecycle tables | implementation exists |
| Implementation provided | 0 REQs; Lane-CTR contract code exists, but the matrix makes no REQ-level implementation claim | behaviour passed or a REQ is implemented |
| Behaviour executed | 0 | Profile conformance |
| Conformant Profiles | 0 | — |
| Vector report state | 76 / 76 `not-run` | pass or fail |

M1 is **in progress**. Lane-CTR delivered the contract batch: bilingual schema-contract tests show that all 56 schemas compile and that two F-003 legacy instances are rejected, while code generation, registered bundle-digest, projection, and golden contract code now exist. “Contract tests ran” is not “conformance-vector behaviour executed”: the new `GOBJ-LEGACY-METADATA-001` and `GOBJ-LEGACY-STRONGREF-001` vectors remain `not-run` with all other vectors. F-003’s only remaining gate is real execution by the Lane-CFR runner.

This read-only recount also confirms that all 56 schemas have a top-level `$id` exactly equal to the file name, with zero absolute URL IDs.

- Sources: `docs/plan/PROGRESS.md` milestone, REQ-coverage, and vector-count sections; `docs/checkpoints/20260720-lane-ctr-handoff.md` §1–§3; read-only recount of 56 schemas, 76 vectors, 0 absolute `$id`, and 0 non-filename `$id`
- Tier: 4 status evidence
- Registry hashes: requirements `26d514db49b37df09312f7faec5367048d4af1ec3f179320d54cd10f61cb82d9`; errors `b0499ef3f14e4f3d071bbd5b3f445e1b7cab17894a7e815c21135d1c5d22716a`
- Status hashes: PROGRESS `29386c6e5ad4301fcfe5e0f05ef24b6072dba40c405b64ec7354d865b476cb00`; Lane-CTR handoff `d4b61f8c5fb3725c10f9772bb576e0ca822f078b7a49ffca51a39244138d0957`
- New-vector hashes: legacy metadata `7588782abd50a1a3f7e51026326ed810cef9cf589c9eff32b2646ba0c4d9fa79`; legacy strongRef `de04ad8b9f6d983bc5f97f7da30447c9b6d61a9c3b2fb1b1954153abcf9f94f7`

## 11. Discrepancies and open items that must remain visible

### FACT-COS-018 — Source conflict, not repaired inside the blog

1. The whitepaper header says v1.0.2; root `README.md` still says v1.0.1.
2. `conformance/README.md` literally says the repository has no conformance runner. Current code and PROGRESS more precisely describe an enumerate-only skeleton that can report 76 `not-run` entries but cannot execute behaviour. The root README still lists 74 vectors and M0 status, and the findings ledger’s IMP-17 summary row also retains 74 even though its F-003 entry and PROGRESS use 76; those stale lines are not current count sources.
3. F-003 remains labelled partially closed, but all contract-layer obligations are complete; its only remaining gate is Lane-CFR’s real execution of the two negatives. F-001 remains an evidence gap. F-011/F-014/F-023/F-017 are open. F-015 remains partially closed.
4. D-001, D-006, and D-011 are closed by M1 Lane-CTR; D-004 remains open. There are no absolute, missing, or non-filename schema `$id` values in this snapshot.
5. The ContextRequest prose/shape mismatch remains.
6. There is no complete OperationDescriptor schema and no AcceptanceDecision schema.
7. Core and the Context standard use two nine-stage naming schemes.

- Sources: root `README.md`; whitepaper header; `conformance/README.md` §Running; `docs/traceability/findings-ledger.md` F/D tables; schema enumeration
- Tier: 1/3/4 discrepancy record
- Hashes: root README `22a4b6d9c1da1d4ea4308faafe82c6615fd69ccb476fba1ad24c2387d06133f1`; conformance README `a36b8e4e2d47384fa2f28da50f432e53f1f3b87e6ee72924eab1d1254ba17d19`; ledger `419b23c6b4c855a7683a81106ee6fd5f42e2b623ba1c7feec78b1a50d9b38066`; Lane-CTR handoff `d4b61f8c5fb3725c10f9772bb576e0ca822f078b7a49ffca51a39244138d0957`

## 12. Public wording guardrail

### Safe

- “The current Draft registers 273 requirements.”
- “Five execution lifecycles are defined by independent transition tables.”
- “Core requires probabilistic components to produce candidates or proposals; authority decides authorization, transitions, and commits.”
- “ContextView is a non-authority projection.”
- “The Effect transition table has no direct commit from `OUTCOME_UNKNOWN`.”
- “Task completion needs current Verification and an acceptance-authority decision.”
- “Lane-CTR delivered contract-layer code and tests; the current snapshot still records zero REQ-level implementation claims, zero behavior-executed vectors, and zero conformant Profiles.”

Every sentence should include a snapshot commit/date or locatable source.

### Forbidden

- “CognitiveOS is implemented, production-ready, or proven safe.”
- “All 76 tests passed” or “all vectors are verified.”
- “A Profile conforms.”
- “Every sink is fenced.”
- “The Console is implemented.”
- Claims of improved agent performance, success, revenue, users, or latency.
- Calling a research snapshot hash a registered specification-set digest.

## 13. Bilingual glossary

| English | 中文 | Use |
|---|---|---|
| authority | 权威主体/权威协议 | final writer, arbitrator, or acceptance authority for a domain |
| candidate | 候选 | probabilistic output, not a decision |
| proposal | 提议 | input awaiting deterministic gates |
| OperationDescriptor | 操作描述 | what can be done; grants no permission |
| AuthorizationCapability | 授权能力 | who may do what within fixed bounds |
| ContextView | 上下文视图 | Activity-bound non-authority working projection |
| Intent | 意图记录 | governed action intent persisted before dispatch |
| Effect | 效果 | lifecycle record for a governed or external change |
| OUTCOME_UNKNOWN | 未知结果 | execution may have happened; reconcile first |
| reconciliation / Reconcile | 对账 | query and close the external execution fact |
| quarantine | 隔离 | safe disposition blocking normal commit or dispatch |
| compensation | 补偿 | a new independently authorized governed Effect |
| Verification | 验证 | evidence judgement against fixed criteria and post-state |
| Acceptance | 验收 | authority decision advancing Task completion |
| fencing token / epoch | 栅栏令牌/代际 | rejects an obsolete writer |
| idempotency key | 幂等键 | stable identity across one logical attempt chain |

## 14. Diagram scripts, captions, and text alternatives

### Diagram 1 — Overall architecture (informative)

- Script: stack seven layers; group layers 7–6 as experience, 5–4 as control, 3–1 as execution/data; place the cognitive microkernel at layer 4; connect a separate real-time safety kernel to layers 3 and 1; draw Context as a dashed cross-cutting band.
- Caption: The dual kernel, three planes, seven layers, and cross-cutting Context are a responsibility view, not a substitute for machine contracts.
- Alt/long text: Seven layers connect the host world to agents; Context crosses all three planes; the cognitive microkernel holds deterministic gates while a separate safety kernel arbitrates actuators.
- Source: whitepaper §4.1–§4.5.

### Diagram 2 — Probabilistic/deterministic boundary

- Script: LLM/retriever/ranker on the left emit candidate/proposal only; a heavy authority boundary in the centre; schema/auth/CAS/budget/idempotency/fencing/transition/commit on the right.
- Caption: Open semantic search may propose; shared facts change only through a deterministic authority path.
- Alt/long text: Probabilistic output stops at the boundary. Deterministic gates must pass before authorization, transition, or commit.
- Source: Core §2 and §15.5.

### Diagram 3 — Governed Flow Thread

- Script: Context → Proposal → Persisted Intent → Authorization → Effect → Reconcile → Verification → Acceptance. Branch `OUTCOME_UNKNOWN` back to Reconcile; still unknown goes to independently authorized compensation or quarantine.
- Caption: A static semantic chain, never live progress.
- Alt/long text: A proposal persists Intent and gains authorization; unknown Effects reconcile first; Verification does not equal Acceptance; compensation needs separate authorization.
- Sources: Intent/Effect standard §2–§5; Task and Verification transitions.

### Diagram 4 — Five orthogonal lifecycles

- Script: five independent swim lanes containing every state name; thin evidence references may cross lanes, but do not create one serial super-machine.
- Caption: AgentExecution, Task, Loop, Effect, and Verification may occupy different states at once.
- Alt/long text: Each domain has its own authority, initial state, terminal states, and recovery path. Completing a Task does not automatically terminate other domains.
- Sources: state-domain registry and five transition JSON files.

### Diagram 5 — Context pipeline and vocabulary discrepancy

- Script: use the Context standard’s nine stages on the main axis; align the Core vocabulary beneath; break connectors where transform/verify and loss declaration are not one-to-one, with a “vocabulary differs” note.
- Caption: The standard ordering is an implementation contract; Core uses another conceptual vocabulary, and the two are not silently merged.
- Alt/long text: Governance pre-filter precedes candidate retrieval; per-object authorization precedes semantic ranking; both nine-stage vocabularies appear side by side.
- Sources: Context standard §2; Core §6.3.

## 15. Research snapshot hash index

| Source path | SHA-256 research snapshot hash |
|---|---|
| `CognitiveOS-Architecture.md` | `e71b5dd83549ec7b7eb278e609d22fcea503279ef5918e3aceab10a62dd54b0c` |
| `specs/core/README.md` | `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11` |
| `docs/standards/context-resolution-and-cache.md` | `e973a36801a5e66393a1942186a5b1a82f80e200393e331d24f23fb2f95683a8` |
| `docs/standards/intent-effect-idempotency.md` | `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c` |
| `docs/standards/task-loop-verification.md` | `5075759a0ce6940707bf47b65d7b930819b841978996307620345f8c93996a42` |
| `docs/standards/normative-source-and-versioning.md` | `c0d0dd4b5ef6b97da1c0ce82d2947905774cce9e9814b7d057b7f008a44a7e6b` |
| `specs/registry/requirements.yaml` | `26d514db49b37df09312f7faec5367048d4af1ec3f179320d54cd10f61cb82d9` |
| `specs/registry/errors.yaml` | `b0499ef3f14e4f3d071bbd5b3f445e1b7cab17894a7e815c21135d1c5d22716a` |
| `specs/registry/state-domains.yaml` | `17ee88cabe13b0c539559e507e36c417455eeb6204a1907848f9c17757296d40` |
| `specs/transitions/agent-execution.transitions.json` | `d29c2926ee9a2ceae945201df25e712ddfbc65000323b24af70466e81652f616` |
| `specs/transitions/task.transitions.json` | `770484aeb2b04f2afd76e07512fb01b5b9760517ce5b879bb58714ed1b375f17` |
| `specs/transitions/loop.transitions.json` | `db836e190920cc008804b8a195f3c01cf2cf696039003914464badd0539509ef` |
| `specs/transitions/effect.transitions.json` | `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e` |
| `specs/transitions/verification.transitions.json` | `4975c774094ae4f48cd4217497bde27cb71005be41bfd2a1df5ea21aa65e0ade` |
| `specs/schemas/context-request.schema.json` | `080747d0f1510cab9d94c1cca30dae43bda9f51f2eb2a2a75ac16cad9062b104` |
| `conformance/vectors/remote-completed-not-acceptance.json` | `b7d8df6d38111452d868520972306578e57c051de3e8e6f885fb23f8edb320b1` |
| `conformance/vectors/intent-acceptance-007.json` | `ad859a72d8aa1e2670be9e75290c0e7a33edaed09cca79cf10c400bcc79c3c6c` |
| `conformance/vectors/effect-state-closure-008.json` | `e74aa5bf26ddc8b900a0c0b213b522bb40f4fbe8090d1e295cec4d7d18d82b52` |
| `conformance/vectors/effect-unknown-outcome.json` | `6364e3eadce30c7918b83087ae8bf6a7779ce7d945508ab5e1d8f5e3da69c512` |
| `conformance/vectors/governed-object-legacy-metadata-001.json` | `7588782abd50a1a3f7e51026326ed810cef9cf589c9eff32b2646ba0c4d9fa79` |
| `conformance/vectors/governed-object-legacy-strongref-001.json` | `de04ad8b9f6d983bc5f97f7da30447c9b6d61a9c3b2fb1b1954153abcf9f94f7` |
| `conformance/README.md` | `a36b8e4e2d47384fa2f28da50f432e53f1f3b87e6ee72924eab1d1254ba17d19` |
| `docs/plan/PROGRESS.md` | `29386c6e5ad4301fcfe5e0f05ef24b6072dba40c405b64ec7354d865b476cb00` |
| `docs/checkpoints/20260720-lane-ctr-handoff.md` | `d4b61f8c5fb3725c10f9772bb576e0ca822f078b7a49ffca51a39244138d0957` |
| `docs/traceability/findings-ledger.md` | `419b23c6b4c855a7683a81106ee6fd5f42e2b623ba1c7feec78b1a50d9b38066` |
| `README.md` | `22a4b6d9c1da1d4ea4308faafe82c6615fd69ccb476fba1ad24c2387d06133f1` |

Final authoring check: every public factual claim must resolve to one `FACT-COS-*` entry. A later parent commit requires new evidence; do not reuse these counts or hashes.
