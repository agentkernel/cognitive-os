---
doc_id: dev.memory-skill
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-store/src/sqlite/memory.rs
  - path: crates/cognitive-store/src/memory_admission.rs
    symbols: ["admit_memory_candidate"]
  - path: crates/cognitive-kernel/src/memory_admission.rs
    symbols: ["decide_memory_admission"]
  - path: crates/cognitive-store/src/sqlite/harness_skill.rs
  - path: apps/kernel-server/src/personal/resource_api.rs
  - path: apps/kernel-server/src/personal/memory_skill_consumer.rs
    symbols: ["load_governed_memory_skill_candidates"]
  - path: crates/cognitive-store/src/memory_skill_consumption.rs
    symbols: ["memory_skill_consumption_migration_entry"]
tests:
  - crates/cognitive-store/tests/p4_t01_memory_store.rs
  - crates/cognitive-store/tests/p4_t02_memory_search.rs
  - crates/cognitive-store/tests/p4_t04_skill_store.rs
  - apps/kernel-server/tests/p4_t05_resource_api.rs
fingerprint: "sha256:9afe0527fd5ee53bed96ce4a1efb4e1e851661c6ecb7e7cf123d0f80b50389b3"
non_claims:
  - Lifecycle correctness evidence is focused-test evidence; B08-class Gate accounting is owned by the formal plan.
---

# Memory and Skill

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
tie-breaks.

## Skill: immutable packages, exact pins

Import rejects unsafe local provenance (absolute/UNC/`..` paths) and
digest/payload drift; package + revision commit atomically. Bindings demand a
`compatible` revision in the same workspace scope; revocations are separate
immutable facts (active = status active AND no revocation row); same-package
supersession appends lineage with one successor per revision, and existing
bindings keep their exact pins — they never drift to a successor.

## HTTP reach

Management channel: remember/forget, skill import/bind/revoke, object/explain
reads. `skill/binding/revoke` is matched before `skill/bind`, since the shorter
route is a prefix of the longer one and would otherwise handle every revoke. Task channel: task-bound projection/watch plus a production governed consumer.
`resolve_authorized_task_context` loads eligible Memory/Skill only after
metadata-first eligibility, exact scope/pin/digest checks, and current
forget/revoke revalidation. The resulting fragments enter the sealed
ContextView; an append-only v24 consumption record keyed by Task, epoch,
ContextRequest and session supports cross-session reuse. The latest row is the
last appended record, not the lexicographically greatest hashed identity.
Reuse reloads current authority facts and fails closed on forget, revoke, or
digest mismatch.
Task bearers are rejected before any management mutation.
