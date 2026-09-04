---
doc_id: dev.memory-skill
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-store/src/sqlite/memory.rs
  - path: personal/crates/cognitive-store/src/memory_admission.rs
    symbols: ["admit_memory_candidate"]
  - path: personal/crates/cognitive-store/src/memory_privacy.rs
    symbols: ["screen_memory_admission", "recall_episodic_memory", "forget_episodic_memory"]
  - path: personal/crates/cognitive-store/src/knowledge_memory.rs
    symbols: ["KnowledgeMemoryStore", "auto_admit_chat", "request_promote", "confirm_promote"]
  - path: core/crates/cognitive-kernel/src/memory_admission.rs
    symbols: ["decide_memory_admission"]
  - path: personal/crates/cognitive-store/src/sqlite/harness_skill.rs
  - path: personal/apps/kernel-server/src/personal/resource_api.rs
  - path: personal/apps/kernel-server/src/personal/memory_skill_consumer.rs
    symbols: ["load_governed_memory_skill_candidates"]
  - path: personal/crates/cognitive-store/src/memory_skill_consumption.rs
    symbols: ["memory_skill_consumption_migration_entry"]
tests:
  - personal/crates/cognitive-store/tests/p4_t01_memory_store.rs
  - personal/crates/cognitive-store/tests/p4_t02_memory_search.rs
  - personal/crates/cognitive-store/tests/p4_t04_skill_store.rs
  - personal/apps/kernel-server/tests/p4_t05_resource_api.rs
  - personal/apps/kernel-server/tests/p8_t12_resource_manager.rs
  - personal/crates/cognitive-store/tests/p13_t07_knowledge_memory.rs
fingerprint: "sha256:819182730280e81fcbe3a669eb1e30a5969d43624dae763bd2f8084e78771333"
non_claims:
  - Lifecycle correctness evidence is focused-test evidence; B08-class Gate accounting is owned by the formal plan.
---

# Memory and Skill

Routine/Trigger (P11-T08) is a Project run submenu, not a Memory write path.

## Memory: candidate → decision → object

Nothing writes a `MemoryObject` directly. The service seam
(`admit_memory_candidate`, the daemon's only production caller path via
`POST /management/resource/v1/memory/remember`) reloads the current Context
source, re-derives the deterministic policy outcome (`decide_memory_admission`),
and rejects any caller-supplied decision that disagrees — then persists candidate
+ reason-coded decision + object + version row + FTS row in one transaction,
re-verifying the source binding (stale source ⇒ conflict).

Lifecycle is append-only facts: forget and expiry tombstones (exact deadline
checks, duplicate sweeps rejected), versioned replacement under expected-version
CAS with `UNIQUE(supersedes_memory_id)` lineage, and atomic FTS row moves. The
FTS5 index is disposable: rebuilds repopulate only from authority rows, so
tombstoned Memory can never resurrect through the index.

Retrieval (`search_memory_candidates`) runs the authority filter CTE first
(admitted decision, no tombstone, exact scope+purpose, unexpired retention,
current source binding) and only then `MATCH`, ranked by `bm25` with stable
tie-breaks. Scoped episodic recall (`recall_episodic_memory`) additionally
requires caller and target `opc://project/{id}/employee/{id}` to match before
the index is consulted; `screen_memory_admission` rejects secret/PII-shaped
text and Letta/Mem0/Agent-self envelopes. There is no second Memory store.

## Skill: immutable packages, exact pins

Import rejects unsafe local provenance (absolute/UNC/`..` paths) and
digest/payload drift; package + revision commit atomically. Bindings demand a
`compatible` revision in the same workspace scope; revocations are separate
immutable facts (active = status active AND no revocation row); same-package
supersession appends lineage with one successor per revision, and existing
bindings keep their exact pins — they never drift to a successor.

## HTTP reach

The management channel publishes lifecycle preconditions, admits Memory
remember/recall/correct/forget/index.rebuild plus Skill
import/revision-inspect/bind/supersede/revoke without direct SQLite access.
P13-T07 adds management `vault.labeled` / `vault.documents` (provenance,
rights, freshness, exclusion, untrusted-observation; files stay
`is_authority=false`; a stored document remains visible as `not-indexed`)
and `memory/auto-admit.chat` / `memory/promote.request` /
`memory/promote.confirm` / `GET memory/promotes` on the existing Memory
tables. Owner only; task aliases 403; Assistant self-admission and
tombstone promote fail closed; an unconfirmed promote does not copy. Chat
auto-admission stays honest-empty / Requires-backend on the Knowledge
surface (P13-T06 group-chat exists; this surface does not list those turns
as admit candidates and has no Admit button). Host filesystem E2E is
`not-run`.
Task-channel Memory mutation aliases (`/task/resource/v1/memory/*`) return
`403 RESOURCE_MEMORY_CHANNEL_FORBIDDEN`. The common Resource Manager (`GET
/management/resource/v1/list|inspect`) projects non-tombstoned Memory objects
and Skill bindings from those same authority rows; it is not a generic Resource
table, and Memory forget remains the family `forget` verb rather than Manager
`revoke`. Public remember accepts unsealed owner fields
(`text`, scope, purpose, retention); the daemon loads the persisted
`GovernanceSeed` and composes sealed headers. A sealed `WorkspaceContextSource`
plus `MemoryCandidate` envelope remains valid for callers that already have
one. Campaign or test code must not mint sealed headers on the public positive
path. `skill/binding/revoke` is matched before
`skill/bind`, since the shorter route is a prefix of the longer one and would
otherwise handle every revoke. All mutation rows and revision lineage remain
available after daemon restart. A policy-rejected Memory candidate may retry
the exact same sealed source without raw cleanup; a same-id source with
different fields remains a conflict. Task channel: task-bound projection/watch
plus a production governed consumer.
`resolve_authorized_task_context` loads eligible Memory/Skill only after
metadata-first eligibility, exact Task-or-workspace scope/pin/digest checks,
and current
forget/revoke revalidation. The resulting fragments enter the sealed
ContextView; an append-only v24 consumption record keyed by Task, epoch,
ContextRequest and session supports cross-session reuse. The public task
channel reads that record through `GET /task/resource/v1/consumption`
without `query_text` or a caller-supplied Skill binding: it returns only
redacted pins, session/`reuse_of` linkage, and `authorized_exact_pin`.
Forgotten, revoked, or digest-drifted pins fail closed before any pin is
returned. Session 2 and post-restart GET resume from that same durable row
(`reuse_of` set, exact pins, no chat replay); `POST /task/resource/v1/consumption`
with caller `query_text` cannot replace it. The latest row is the
last appended record, not the lexicographically greatest hashed identity.
Reuse reloads current authority facts, binds its deterministic record identity
to principal/tenant/scope/purpose/request digest and exact pins, and fails
closed on forget, revoke, digest mismatch, or a competing durable record.
Task bearers are rejected before any management mutation.

Public backup archives carry a digest-bound Memory/Skill export sidecar and
never a raw `authority.sqlite` copy.
