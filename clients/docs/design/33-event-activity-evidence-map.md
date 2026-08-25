# 33 — Event / Activity / Evidence Map (actual event source map)

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Question (brief §13/§14): what event/audit/effect/intent/verification/acceptance records actually exist, where they live, and what the frontend can reach. Rule enforced: **logs are not merged with authority events** — they are separate sources, labeled separately.

---

## 1. The durable authority event log (the spine)

- **Storage:** `events(sequence INTEGER PK AUTOINCREMENT, event_id UNIQUE, object_id, domain, object_version, event_type, canonical_json, UNIQUE(object_id, object_version))` — append-only by trigger (`schema.rs:71-88`). Companion: `transition_records` (append-only, `:90-105`), `outbox` (`:114-119`), global `fencing` singleton (`:145-150`).
- **Closed event-type set (6 types — replay rejects anything else, `replay.rs:223-227`):**
  - `cognitiveos.object.admitted`, `cognitiveos.state.transition.committed` (`engine.rs:38-41`)
  - `cognitiveos.intent.persisted`, `cognitiveos.user-intent.recorded`, `cognitiveos.intent-interpretation.recorded`, `cognitiveos.task-contract.minted` (`replay.rs:18-32`, provenance-only)
- **Ordering/integrity:** `sequence` is authoritative global order (`ports.rs:189-205`); the digest chain is computed at read time (O13 `chain_head_digest`, `observation.rs:906-915`), not stored.
- **What emits events:** governed-object admissions/transitions (engine) + intent-chain records. **What does NOT emit:** memory admission/tombstones, skill packages/bindings, provider control-plane mutations, installation DB changes, scheduler entries, tool-lifecycle overlay (a JSON file, `tool_lifecycle.rs:521-529`), observation samples. (`INSERT INTO events` exists only in `protocol.rs:117`, `store.rs:191`, `util.rs:231,436`.)
- **Unified feed?** Library level: **yes** — `read_events(after_sequence, limit)` is global (`ports.rs:261-267`). HTTP level: **no** — `/resource/v1/watch` is an in-process `VecDeque` ring (`resource_api.rs:50, 1383-1445`), and O13 replay is per-task bounded (4096 scanned / 64 returned).

## 2. Event source map (per rendered kind in the Phase-2 design, `19` §1)

| Design kind | Real source | Storage | Authority? | API | Frontend availability today |
|---|---|---|---|---|---|
| **Event** | durable `events` log | authority SQLite | yes | per-task via O13 replay (`/task/observation?family=o13`) + transitions in `/task/evidence` | per-task only; no global feed |
| **Change** (governance mutation) | provider plane: `llm_audit_events` table (redacted rows) | authority SQLite | yes (provider plane only) | `/management/audit` | yes (provider plane, unfiltered) |
| | memory/skill/tool/backup mutations | their stores (append-only tables/overlay) | yes | **no audit route** | **not reachable as events** (BD-5) |
| **Effect** | Effect governed objects (`governed_objects`, domain='effect', 14-state machine) | authority SQLite | yes | `/task/effects`, O5 family | per-task |
| **Error** | failure states inside the above (VERIFY_FAILED, DENIED, OUTCOME_UNKNOWN; provider `last_discovery_error`; audit outcome) | same | yes | same routes | composed |
| **Intervention** (owner acted) | `task_contracts.accepted_by` + intent chain + provider audit (account/key/binding actions) + alert ack | authority SQLite | yes | `/task/evidence` (intent_refs), `/management/audit` | composed, per-object |
| **Verification** | `verification_requests` + `verification_reports` (append-only; status passed/failed/indeterminate; fencing-epoch bound) | authority SQLite | yes | `/task/evidence` (`latest_verification`) | per-task |
| **Acceptance** | daemon-authored decision artifact in **Artifact CAS** (`artifact://sha256/…`) + committed terminal transition | CAS + events | yes | `/task/evidence` (`latest_acceptance`) | per-task |
| **Logs** (process output, daemon log) | bounded observation samples (O-families); dsh heartbeat | overlay files / process-local | **no — observation, not authority** | `/task/observation` | per-task, bounded, redacted; **never merged into the evidence stream** |

## 3. Evidence chain (what proves completion — the full authority path)

```text
task_contracts (current epoch, accepted_by, contract_digest)
  → fixed_post_states (subject version pinned)
  → verification_requests (verifier_ref/version, criteria, epochs)
  → verification_reports (latest per request, status=passed, current fencing epoch)
  → all task-bound Effects ∈ {RECONCILED, VERIFIED, VERIFY_FAILED}
  → acceptance decision artifact in CAS (artifact://sha256/<digest>)
  → committed CANDIDATE_COMPLETE→COMPLETED transition (events + transition_records)
```

Enforcement: `complete_task_from_persisted_verification` (`task_completion.rs:199-378`) with two atomic commits (`ports.rs:1000-1039`). Read path: `reconstruct_terminal_task_evidence` (`task_api.rs:1586-1726`) assembles `TerminalTaskEvidence` for `GET /task/evidence`.

## 4. Realtime / watch reality (brief §14)

| Mechanism | Exists? | Reality |
|---|---|---|
| WebSocket | **No** | none anywhere |
| SSE | Yes | `/task/watch`, `/resource/v1/watch` (+task twin), provider proxy stream passthrough |
| Long polling | No | — |
| Polling | Yes | what the shipped SPA actually does (manual GET of the SSE endpoint) |
| Delta stream | Partial | watch deltas are in-process rings (128 events); resource watch publishes only `projection.initialized` (inert after startup); task watch snapshot `tasks:[]` always empty |
| Snapshot refresh | Partial | 409 resume-stale → re-snapshot semantics exist |

**Can each space update in real time today?** Home: no (manual refresh). Work: per-task watch yes (thin), inventory no. Activity: no. Agents: no (dsh snapshot is pull). Providers: no. → Wave-1 design uses explicit refresh + watch-where-real + stale labels; **no fake realtime** (OQ-2 records the polling-policy decision).

## 5. Consequences for the Activity design (`19`)

1. The seven rendered kinds are all **real and distinguishable** — but their coverage differs per object, which is exactly what the coverage banner must state: "provider-plane audit + per-task authority events observed this session; memory/skill/tool/backup mutations are not emitted as events (BD-5)".
2. Per-object timelines (task/agent/provider) are fully backable; the cross-domain stream is the BD-5 gap.
3. Observation samples (logs) stay in the observation lane / observation plane UI — never rendered as Event rows.

---

*Feeds: `35` §8 (traceability), `37` BD-4/BD-5, `39` wave ordering.*
