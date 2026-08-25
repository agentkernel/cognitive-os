# 02 — Control Plane Jobs-to-be-Done

- Phase: Product Redesign Phase 1 (design-only)
- Date: 2026-08-24
- Method: `jtbd-analysis` (job statements = When [situation] / I want to [motivation] / so I can [outcome]; Forces of Progress), grounded in the documented product JTBD (`product-design.md:36-52`), the Web UI's four product questions (`web-ui-design.md:19-24`), the audited current state, and the capability inventory. No invented user research: every job cites its evidence source.
- User frame: one owner in five modes (see `01-control-plane-product-model.md` §4). Primary mode: **Individual Operator**.

---

## 1. How these jobs were derived

Sources of evidence, strongest first for this product:

1. **Observed system behavior / current-state audit** — what the shipped UI can and cannot do (Current State Map §8, §10).
2. **Documented product jobs** — the seven canonical JTBD and four Web UI questions (canonical product intent).
3. **Capability reality** — what the daemon can actually back (Capability Inventory).
4. **Owner direction** — this redesign brief itself (the questions in §2 below restate the brief's supervision questions).

There are no external user interviews; the owner is the user. Jobs are therefore validated against *system truth and documented intent*, and each job carries an honesty note where today's surface fails it.

---

## 2. The supervision questions (the brief's core, answered as jobs)

The brief asks: as agent count grows, what does the user most need to know / control / investigate / intervene in / recover / fear? Reframed as job statements, then generalized:

### 2.1 Know

- **J-K1.** When I open the Control Plane, I want to see the system's authority state at a glance — ready/degraded/blocked, what is running, what needs me — so I can decide in seconds whether to act or leave.
- **J-K2.** When an agent is working, I want to see which task it serves, what phase the task is in, and what it last verifiably did — so I can understand behavior without reading logs.
- **J-K3.** When something is blocked, I want the blocked reason in the system's own vocabulary (missing binding, stale epoch, secret unresolvable, budget stop) — so I can fix the cause, not guess.

### 2.2 Control

- **J-C1.** When I change anything (binding, key, tool availability, skill binding, budget), I want a server-issued preview with exact targets, versions and consequences — so the mutation I admit is the mutation that happens.
- **J-C2.** When work misbehaves, I want to stop or redirect it through typed controls — so I can intervene without killing processes blindly. *(Honesty: task cancel and agent lifecycle verbs are NOT AVAILABLE over HTTP today; the job is real, the surface must render the gap and the CLI path, not fake the verb.)*
- **J-C3.** When I delegate, I want to set the boundaries (workspace, tools, budget, model) before the run — so autonomy never exceeds my intent.

### 2.3 Investigate

- **J-I1.** When I suspect something went wrong, I want a chronological, identity-preserving trail (task transitions, effects, evidence, process facts) — so I can reconstruct what happened without SQLite.
- **J-I2.** When a task claims completion, I want the verification and acceptance record — so I can distinguish "agent said so" from "verified".
- **J-I3.** When I review cost/usage, I want honest counters (unknown ≠ zero) per account/model/agent — so I can trust the bill story.

### 2.4 Recover

- **J-R1.** When the daemon/agent/provider/secret store is unavailable, I want the surface to degrade into a diagnostic client with stable error classes and next actions — so I can restore service from the UI's guidance.
- **J-R2.** When a restart or crash happens mid-work, I want to see reconciliation state (unknown outcomes, fenced epochs, pending effects) — so I know what the system is unsure about.

### 2.5 Fear (the negative jobs — what the design must never do)

- **J-F1.** I fear a green "completed" that means "process exited". → Design rule: completion is always evidence-linked.
- **J-F2.** I fear clicking something whose blast radius I can't see. → Design rule: preview + consequence + CAS before mutation; destructive separated.
- **J-F3.** I fear the UI hiding a failure to keep the surface calm. → Design rule: calm ≠ quiet; blocked/failed/unknown are visually distinct and never color-only.
- **J-F4.** I fear secrets leaking through the UI. → Design rule: presence/absence only; no secret-shaped strings rendered ever.

---

## 3. The canonical jobs, re-examined

The seven documented JTBD (`product-design.md:36-52`) remain valid; the Control Plane's share of each:

| # | Canonical job | Control Plane share today | Verdict for redesign |
|---|---|---|---|
| 1 | Install → first conversation without credential distribution | Readiness/doctor projection; provider+key setup | Keep; elevate readiness to the Home core |
| 2 | Import/inspect/pin/disable Skills | Raw list JSON only | **Under-served** — needs real family depth |
| 3 | Remember / review proposals / search / forget Memory | remember/forget exist; no search UI, no review queue | **Under-served** |
| 4 | Create bounded Task with inspectable Context | record→interpret→preview→admit exists; one hardcoded task type; no context view | **Partially served** — the flagship flow is present but skeletal |
| 5 | Standard Workspace + deliberate Extended Home | not surfaced | Deferred (backend surface not found) |
| 6 | Supervise/pause/resume/recover/upgrade/remove Agent with distinct identities | identity cards only; all verbs not-run | **Honestly blocked** — design must render the gap, CLI path, and identity model |
| 7 | Diagnose/restore when model/Provider/Agent/sidecar unavailable | readiness/doctor core live; sub-sections placeholder | Keep; strengthen degraded-mode design |

---

## 4. Job map by mode (priority)

Jobs ranked by (frequency × consequence-of-failure) for the primary mode, with the pattern each implies. This is the prioritization input for IA and flows (RICE-style scoring is meaningless without reach data for a single-owner product; frequency×consequence is the honest proxy).

| Rank | Job | Mode | Frequency | Consequence if failed | Pattern it demands |
|---|---|---|---|---|---|
| 1 | J-K1 system state at a glance + attention queue | Operator | many×/day | high (missed blockage) | Status board with priority stack (not metric cards) |
| 2 | J-I2 verify completion from evidence | Operator | daily | critical (false completion = product betrayal) | Evidence-linked task/run detail |
| 3 | J-K2 what is the agent doing now | Operator | daily | high | Task/run detail with live-ish state + identity clarity |
| 4 | J-I1 reconstruct what happened | Operator | weekly, urgent when needed | high | Unified timeline (task transitions + effects + evidence + process facts) |
| 5 | J-C1 govern bindings/keys/tools safely | Power User | weekly | high (wrong binding = wrong egress) | Preview + CAS + confirmation with exact IDs |
| 6 | J-K3 see blocked reason precisely | Operator | daily | medium-high | First-class blocked/unknown state vocabulary |
| 7 | J-R1 degrade to diagnostic client | System Operator | rare, critical | high | Readiness/doctor surface with recovery links |
| 8 | J-C3 set delegation boundaries | Power User | per task | high | Task creation with inspectable contract/context |
| 9 | J-I3 honest usage/cost review | Operator | weekly | medium | Usage views with unknown-as-unknown |
| 10 | J-C2 stop/redirect work | Operator | rare today (backend-blocked) | critical when needed | Typed controls where they exist; honest not-run + CLI guidance where they don't |
| 11 | Memory proposal review / curation | Power User | weekly | medium | Review queue (backend review-queue route UNKNOWN — flag as dependency) |
| 12 | J-R2 post-restart reconciliation view | System Operator | rare | high | Reconciliation/unknown-outcome surface |

---

## 5. Forces of Progress (for adopting the redesigned surface over the CLI workaround)

The redesign competes against the owner's current workaround: the `cognitive` CLI, admin-cli, and raw endpoint probes.

```
DRIVING                                          RESISTING
Push: raw-JSON UI can't answer supervision       Anxiety: redesign might dilute capability
      questions; no task inventory; no live          honesty; might hide gaps behind polish;
      state; CLI context-switching is slow           might break the working Provider flow
Pull: one legible surface where authority        Habit: CLI is scriptable, precise, trusted;
      state, evidence and governance are            checkpoint reports already answer
      scannable and actionable                       "what's true" for this owner
```

Design responses required by the forces:

- **Against anxiety:** the honesty contract is carried into the redesign verbatim (not-run states, evidence-linked completion, unknown≠zero) and strengthened (200-stub whitelist defense). The redesign must be *more* honest than the CLI, not less.
- **Against habit:** speed paths for the returning operator — stable layout, keyboard, command surface, deep links — so the UI beats the CLI on scan speed, not on abstraction.
- **Amplify pull:** the CLI cannot show relationships (task↔effects↔evidence↔agent↔binding) spatially; the UI's unique value is *relational legibility* — make that the differentiator.

---

## 6. What the Control Plane is NOT hired for (anti-jobs)

1. Chatting with agents (the Shell's job).
2. Authoring code/content (the agents' job).
3. Metric dashboards / analytics theater (no honest denominator exists).
4. Administering other users (single-owner boundary).
5. Hiding system complexity behind "AI magic" (the product's anti-identity).

---

*Output feeds: `03-control-plane-capability-model.md` (what the system can back), `05-control-plane-ia-options.md` (structures ranked by job fit), `07-control-plane-user-flows.md` (jobs as flows).*
