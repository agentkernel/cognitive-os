# 15 — Work Detail Spec

- Status: adopted Personal 2.0 Work-detail target; historical Task detail retained
- Updated: 2026-08-27
- Contract: `06` §3.2 (six sections), `04` (Run = presentation object; authority vs observation lanes), `08` (evidence-linked completion), Flow 2/6/7 in `07`.
- **Structure decision (researched, not mechanical):** the six sections were evaluated as (a) tabs, (b) accordion, (c) single scrolled document with local section navigator + facts inspector. **Chosen: (c).** Tabs hide exactly the thing supervision needs (the relationship between Run, Effects, Evidence is one read, not three destinations); accordion breaks deep links and scan; a supervised document with a local navigator keeps one continuous evidentiary narrative while preserving jump-to-section speed. Section *order* is supervision order (state → what happened → proof → what was agreed → what it saw), a refinement of the Phase-1 listing order; all six contracted sections are preserved. Logged as a Phase-2 refinement, not a contract change.

## Personal 2.0 Work detail amendment

The target detail is Goal-centered and keeps one continuous evidentiary read:

1. **Goal summary** — owner-approved objective, current disposition, participants,
   budget and latest verified outcome.
2. **Plan revisions** — immutable revision list, selected revision, change
   rationale, candidate source, daemon admission and supersession links.
3. **Tasks and attempts** — dependency-aware hierarchy with exact Task states,
   attempt identities, Agent assignments and current blockers.
4. **One provenance timeline** — `Native`, `Observed`, `Governed`, `Verified`
   items time-aligned but never merged.
5. **Context** — authorized inputs, source versions, explicit losses/conflicts,
   and resource pins.
6. **Effects** — persist-before-dispatch records, outcome and reconciliation.
7. **Evidence and acceptance** — independent verification, artifacts, currency
   and acceptance.

The inspector begins with plain-language "what is happening / what needs you"
and expands to full IDs, digests, epochs, source routes, coverage and raw
redacted projections.
The beginner-facing operational label is **execution flow**; `Task`, `attempt`,
`Effect`, and current Run-composition terms remain exact in the inspector.

### Timeline provenance

- **Native:** Agent conversation/history or Adapter-native event. It may explain
  intent but cannot advance authority.
- **Observed:** process, transport, heartbeat or bounded runtime fact.
- **Governed:** Goal/Plan/Task/admission/Effect state written by the daemon.
- **Verified:** independent report and acceptance.

The former two-lane Run timeline remains a valid current-Task detail inside this
four-provenance model. It is not discarded; it becomes the
Observed/Governed/Verified subset available today.

### Revision, attempt and progress rules

- Plan revision is append-only in the target; prior versions stay inspectable.
- An attempt never overwrites its Task or another attempt.
- A native message, process exit, stream close or provider response never marks
  progress or completion.
- Percent progress requires a real bounded denominator. Otherwise render state,
  current recorded step and age.
- Re-plan, retry, pause/cancel and multi-Agent role changes are
  `Requires-backend` until typed services exist.

The current API exposes per-task evidence/effects/observation but not Goal,
Plan revision, attempt or orchestration entities. Those sections must render an
explicit target delta rather than fabricated records. The existing six-section
Task specification below remains the P7-T05 current-backed fallback.

---

## 1. Page anatomy (1440 px)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ strip (daemon · session · alerts · watch · ⌘K)                           │
├────────┬─────────────────────────────────────────────────────────────────┤
│ side-  │ HEADER (persistent, hairline-separated)                        │
│ bar    │  task a3f9c2…   ● running · 4m        agent pi · epoch 3       │
│        │  verification: not yet terminal   evidence: —                  │
│        │  [Attach watch] [Copy ref]   cancel: not available (BD-1) ···  │
│        ├──────────┬──────────────────────────────────┬──────────────────┤
│        │ SECTIONS │ CONTENT (supervision order)      │ FACTS INSPECTOR  │
│        │          │                                  │                  │
│        │ Overview │ ── Overview ──────────────────── │ contract b91e…77 │
│        │ Run    ● │ state running · admitted 12:41   │ epoch 3          │
│        │ Effects  │ by principal://local/owner       │ intent i-44…     │
│        │ Evidence │ budget 4 steps · 0 retries       │ preview p-09…    │
│        │ Contract │ deadline 2027-12-31              │ interpretation   │
│        │ Context  │                                  │   i-44… (cand.)  │
│        │          │ ── Run ───────────────────────── │ agent pi         │
│        │          │ authority lane │ observation     │ binding deepseek │
│        │          │  12:41 admitted ───────────────  │   -main@rev 4    │
│        │          │  12:41 lease acquired            │                  │
│        │          │  12:42 dispatch ── 12:42 proc 4812 spawned (obs)    │
│        │          │  12:43 effect e-1 EXECUTED       │ watch live · 118 │
│        │          │  12:44 candidate complete        │ [detach]         │
│        │          │  …                               │                  │
│        │          │ ── Effects ───────────────────── │ related          │
│        │          │ e-1 workspace.search EXECUTED ✓  │ agent dossier →  │
│        │          │ e-2 …                            │ provider acct →  │
│        │          │ ── Evidence ── ────────────────  │ activity slice → │
│        │          │ ── Intent & Contract ──────────  │                  │
│        │          │ ── Context ────────────────────  │                  │
└────────┴──────────┴──────────────────────────────────┴──────────────────┘
```

Three columns: **local section navigator** (anchors, scroll-spy, keyboard `[`/`]`), **content** (the six sections), **facts inspector** (identity/digests/related links/watch controls). At 1280–1439 the inspector floats; below that it becomes a top-collapsed "Facts" disclosure.

## 2. Header (persistent)

- Left: object identity (`task` + short ref, mono) + state (category + verbatim label + one-line reason).
- Right: agent identity, contract epoch, verification disposition (S6 only with evidence link; otherwise honest "not yet terminal"), and the action row: Attach/Detach watch **[B]**, Copy ref **[B]**, class-C control line for cancel/pause (DD-08 text, not a button).
- The header never scrolls away; it is the page's answer to "where am I and what is true right now".

## 3. The six sections (supervision order)

### §1 Overview

State, reason, admission facts (when, by whom, digest), budget facts (steps/retries/deadline/cost ceilings), current loop/iteration when observable. One compact fact grid — the 5-second layer. No prose marketing of the task.

### §2 Run (the center)

The Run reading = one timeline, **two lanes**:

- **Authority lane** (left rail, solid nodes): lifecycle transitions from `GET /task/evidence` (`transitions[]`: sequence, event_type, after_state, reason_code, time) + admission + verification/acceptance events.
- **Observation lane** (right rail, hollow nodes, visually quieter): process/observation facts from `GET /task/observation` (o4/o5 families) and watch deltas — spawned/exited/heartbeat-class facts, each labeled `obs`.

Rules: lanes never merge; an observation node can never render as a state change; correlated nodes align by time; unknown gaps render as a dotted span labeled "no recorded facts"; watch state (live/stale/disconnected + cursor) sits on the timeline header with attach/detach (detach = observation-only, says so).

### §3 Effects

Table: effect_ref (short mono), stage, outcome class, reconcile class, mutation count, fixed-post-state ref presence, report ref. `OUTCOME_UNKNOWN` / `VERIFY_FAILED` rows get the S5 left keyline and sort first. Empty state: "No effects recorded — this task has not attempted external mutation." (a meaningful, calming fact).

### §4 Evidence

Terminal evidence block: verification report ref + digest (copyable), status, completed-at, currency (`current:true/false`), artifact refs, acceptance record (terminal transition ref/digest, currency). If 404: designed state — "No terminal evidence recorded" + what that means + link to Run. **This section is the only place the word "completed" may appear, and only with the acceptance record.**

### §5 Intent & Contract

The governance chain rendered as four linked artifacts: raw intent record (id, digest, recorded_at) → interpretation (objectives/constraints/forbidden/assumptions; **ambiguities + information gaps as first-class blocks**) → preview (digest, condition count, budget) → admission (accepted_by, accepted_digest, epoch). Each artifact shows its digest; the chain shows digest linkage. This section is the product's "the owner admitted exactly this" proof.

### §6 Context

The resolved Context view when the projection provides it: selected sources (memory/skill/tool/workspace) with versions/digests, and **explicit losses** (omitted/truncated/conflict/stale/budget) as a named list. When not backed (projection plane `not-backed` for context): the S7 honest state — "Context view is not exposed over HTTP for this task (BD register)" — plus what *is* available (consumption pins via `/task/resource/v1/consumption` when present).

## 4. Why not the alternatives (recorded)

- **Tabs:** hide the Run↔Effects↔Evidence relationship; force mode-switching during incidents; break the continuous evidentiary read. Rejected.
- **Accordion:** collapses destroy spatial memory and deep links; scroll-spy anchors give the same speed without hiding. Rejected.
- **Side inspector as primary:** inspector is facts, not narrative; making it primary inverts the reading. Kept as the facts rail.
- **Chat-like stream:** banned (the product is not a chat UI; the timeline is evidence, not conversation).

## 5. States

| State | Rendering |
|---|---|
| Loading | header + section frames stable; no shimmer |
| Unknown task_ref | designed 404-object state: "No task with this reference is known to this daemon" + back-to-Work + the Tier-1 honesty note |
| Evidence missing | §4 designed state; header verification shows "not yet terminal" |
| Watch stale/disconnected | timeline header state + reconnect **[B]**; last-good frames labeled with age |
| Truncated | `transitions_truncated` / `effects_truncated` render as "showing N of M (bounded)" with the bound named |
| Denied (wrong channel) | channel explanation + session gate |
| Class-C controls | header line: "Cancel/pause are not available over HTTP yet — `cognitive …` (BD-1)" |

## 6. Behavior rules

1. Scroll-spy navigator; deep links to `#/work/:taskRef#effects` etc.; back/forward preserves section and master state.
2. All digests/refs copyable inline (mono chip + copy affordance).
3. Live updates append to the timeline without moving read position (new-node marker + "N new" pill; jump-to-latest is explicit).
4. The facts inspector's related links are the contextual nav: agent dossier, provider account, activity slice — one step each, per the conceptual model's pivot rule.

---

*This page is the product's signature surface: the place where "never falsely completed" becomes visible. Its visual treatment is the visual direction's anchor (`24`).*
