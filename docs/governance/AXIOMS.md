# CognitiveOS Personal — Axiom System

- Status: active repository governance
- Effective date: 2026-08-10
- Owner: this document (canonical axiom list)
- Decision carrier: [ADR-0041](../adr/0041-personal-axiom-system-revision.md)
- Related: [DEVELOPMENT-OPERATING-MODEL.md](DEVELOPMENT-OPERATING-MODEL.md),
  [PROJECT-IDENTITY.md](PROJECT-IDENTITY.md),
  [ADR-0042](../adr/0042-personal-three-pillar-engineering.md)

This document is the **sole owner** of CognitiveOS Personal's immutable axioms
and the engineering-principle layer beneath them. Entry documents such as
`AGENTS.md`, Cursor rules, and Operating Model summaries may link or quote
briefly; they must not maintain a divergent numbered list.

An axiom may change only through an owner-approved ADR that revises this file
in the same delivery. Ordinary implementation preference, local convenience, or
harness fashion cannot weaken an axiom.

## 0. First principles

CognitiveOS Personal exists so that agent work is **auditable, budgeted,
recoverable, and not falsely completed**. The product is a local **operating
system for cognitive resources**—not merely an agent runtime—because it owns
resource abstraction, scheduling semantics, isolation/protection, a unified
agent-facing interface (AKP + typed domain ops), and durable state management.

Research and industry evidence that justify the axioms (informative, not
normative):

- Independent verification and out-of-band evaluators are required because
  in-band self-evaluation can optimize narrative progress while external ground
  truth does not move ("elaborate stagnation"); nontrivial proxy rewards are
  formably gameable under broad policy classes (Skalse et al., 2022; 2026 loop
  engineering and reward-hacking agent benchmarks).
- Production agent systems are overwhelmingly deterministic scaffolding around a
  small probabilistic core (Claude Code reverse analyses report ~98% harness /
  ~2% model decision logic). CognitiveOS therefore places authority in the
  Rust daemon and treats every agent as a candidate producer.
- AIOS (Mei et al., arXiv:2403.16971) formalizes an agent OS kernel with
  scheduling, context, memory, storage, tool, and access managers. Personal's
  six-family model and AKP envelopes map to that kernel vocabulary while adding
  Intent/Effect, fencing, and independent verification that AIOS does not
  specify.

## 1. Immutable axioms

### A1 — Daemon-only authority writer

**Motivation.** Prevent any probabilistic component, client, fixture, or
sidecar from becoming a hidden writer of Task, Effect, Verification, budget, or
governance state.

**Judgable statement.** Only the Rust daemon may authorize, apply CAS/epoch
guards, advance Task/Effect/Verification state, mint budget/capability charges,
persist and reconcile Effects, and accept Tasks. Pi, CLI, SDK, UI, sidecars,
fixtures, and third-party agents are clients.

**Allowed freedom.** Clients may propose candidates, render projections, and
stream observations. They may not write authority SQLite, expand capability
snapshots, or treat Provider/`agent_end` receipts as Task completion.

### A2 — Candidate-only probabilistic boundary

**Motivation.** Separate proposal generation from commitment so model error,
prompt injection, or adapter drift cannot silently commit authority.

**Judgable statement.** Any probabilistic component and any third-party agent
may produce only candidates/observations. Admission, scheduling, Effect commit,
reconciliation, and final acceptance are deterministic daemon operations.

**Allowed freedom.** Agents may propose Memory/Skill candidates, Tool calls,
Context fragments, and multi-agent messages. Admission policy and negative
vectors decide what becomes durable authority.

### A3 — Persist-before-dispatch external mutation

**Motivation.** Crash, duplicate delivery, and unknown outcomes must be
reconcilable from durable Intent/Effect records rather than from live process
memory.

**Judgable statement.** Every external or irreversible mutation uses an Intent
and Effect that are persisted before dispatch, keyed for idempotency, and
reconciled under fencing. External tool success alone is never Task completion.

**Allowed freedom.** Read-only and reversible local observations may use
narrower paths when the formal task acceptance and threat model allow it.

### A4 — Independent verification for completion

**Motivation.** Reward hacking and self-evaluation bias make in-band "success"
untrustworthy for open-ended goals.

**Judgable statement.** Task completion requires independent verification
against durable criteria and closed, reconciled Effects under a non-stale epoch.
Process exit, Pi `agent_end`, Provider response, or external receipt alone is
insufficient.

**Allowed freedom.** Deterministic checks (tests, digests, schema validators)
are preferred; model-as-judge may assist only when mechanical checks cannot
exist, and must remain out-of-band from the actor's writable surface.

### A5 — Secret isolation

**Motivation.** Secrets in argv, config, SQLite, logs, CI, tests, or evidence
create irreversible leakage and false trust in redacted artifacts.

**Judgable statement.** Provider and user secrets enter only approved Secret
Stores and approved non-logging input paths. They must never appear in argv,
ordinary configuration, SQLite, logs, CI output, test output, or evidence.

**Allowed freedom.** Opaque `SecretRef` handles and daemon-mediated Provider
proxying are permitted. Explicit, time-bounded development exceptions require an
ADR expiry and cannot enter CI/release.

### A6 — Contracts and negatives are not implementation-shaped

**Motivation.** Weakening vectors to fit a buggy implementation destroys the
only objective regression surface.

**Judgable statement.** Public contracts, transition tables, registered errors,
and negative vectors cannot be deleted, relaxed, or rewritten to accommodate an
implementation. Contract change uses Lane-CTR. Axiom-level revision requires an
ADR that updates this document.

**Allowed freedom.** Implementation-only and corrective documentation changes
may clarify unchanged semantics. Product-semantic Personal scope changes may use
ADRs without altering CognitiveOS machine contracts.

### A7 — Evidence promotion is campaign-bound

**Motivation.** Local smoke, fixtures, WSL, and ordinary CI are necessary for
development velocity but must not launder Gate/release/Profile claims.

**Judgable statement.** Local, fixture, WSL, fake-systemd, and ordinary CI
evidence may advance implementation evidence only. Formal Gate, release,
containment, and Profile claims require a preregistered campaign environment and
procedure. `not-run` is never pass.

**Allowed freedom.** Experimental-local and tested-local tracks may proceed in
parallel with incomplete Gates when `implementation_requires` are satisfied.

### A8 — Unknown worktree changes are protected

**Motivation.** Autonomous agents otherwise overwrite or mix concurrent owner
edits, destroying recoverability.

**Judgable statement.** Unexpected uncommitted changes are never overwritten,
reverted, staged, or mixed into another delivery. Ownership must be resolved or
explicitly released first.

**Allowed freedom.** Task-owned coherent work may be checkpointed under standing
delivery authorization when lease scope and secret checks pass.

## 2. Engineering principle layer (not axioms)

These principles sit beneath the axioms. They guide design and task acceptance
but may evolve through product-semantic ADRs without rewriting A1–A8.

### P1 — Context engineering

Assemble and govern the information payload an agent sees: authorize before
rank, make losses explicit, bind digests, support stable-prefix/delta reuse,
and evolve compaction plus adaptive budgets without skipping body
reauthorization. Aligns with context-engineering taxonomy (retrieval,
processing, management) while keeping Personal's fail-closed authority order.

### P2 — Harness engineering

Treat the deterministic scaffolding as the primary product surface: WIA,
budget, fencing, independent verifier, Tool pre-validators, and (as designed
in Phase 8) graded extension primitives and deterministic lifecycle hooks.
Hooks and skills must never relax A1–A8.

### P3 — Loop engineering

Design the control loop around the model: ACT→VERIFY→CONTINUE→OBSERVE, layered
termination (budget + no-progress + escalate), externally grounded verification,
and cross-episode learning only through candidate→admission into Memory/Skill
authority. Daemon-owned wait/switch/escalate strategies are preferred over
silent infinite retry.

## 3. OS positioning statement

Personal is an **operating system for cognitive resources** when judged by:

| OS element | Personal coverage |
|---|---|
| Resource abstraction | Six families + cross-cutting Budget/Permission/Artifact/Intent/Effect/Evidence |
| Scheduling | Lease/CAS fencing, STOP-before-lease, WIA/continuation admission |
| Isolation / protection | Tenant/scope (RFC-0001 compatible), channel isolation, capability attenuation, embodied safety domain (extension) |
| Unified interface | AKP envelopes + typed Core/Personal operations (agent syscall ABI) |
| Durable state | SQLite WAL authority store, Intent/Effect, Event, Checkpoint/Resume |

It is **not** a replacement for the Linux kernel, a device firmware ABI, or an
enterprise multi-tenant control plane by default. Those remain architecture
headroom (see architecture headroom chapter), not Linux 1.0 acceptance.

## 4. Supersession

This document supersedes the previously divergent axiom lists in `AGENTS.md`
and Operating Model §8 as of 2026-08-10. Those files now defer here.
