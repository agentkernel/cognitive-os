# Context evolution for OPC archive and Vault retrieval

- Status: current Context foundation plus Personal 2.0 target
- Related: AXIOMS P1,
  [Conversation/Memory/Vault](conversation-memory-vault.md), P3-T02/P3-T04,
  and P8-T05

## 1. Current baseline

Personal already requires authorization before ranking, required-source
fail-closed behavior, explicit loss, digest-bound source traces, stable-prefix/
delta metadata reuse with full revalidation, and independent Context
correctness evidence. Benefit observations remain non-blocking.

## 2. OPC source expansion

Personal 2.0 may source Context from Project charter/plan/Task facts, admitted
Memory/Skill, Tool summaries, Project Vault, Owner-shared knowledge, scoped
Conversation archive, artifacts/evidence, connector readback, and explicit
Owner inputs.

Source expansion preserves:

1. Owner/Project/employee/Task/purpose authorization;
2. secret/PII exclusion;
3. provenance, freshness, conflict, retention, and tombstone;
4. ranking only after filtering;
5. bounded fragment/token budget;
6. explicit omitted/truncated/stale/unavailable loss;
7. untrusted-observation labels for external/conversation content.

All conversations may be eligible sources, but no request receives the full
archive automatically.

## 3. Progressive disclosure and compaction

Business surfaces show concise source/basis/scope. Inspectors reveal selected
fragments, omissions, provenance, versions, and policy. DSH/Pi receive only the
resolved Context view.

Daemon-owned compaction may produce a digest-bound Context candidate with
explicit loss. It cannot self-authorize, become semantic Memory, or prove
completion. Adaptive budgets use durable telemetry without skipping body
reauthorization.

Codex provides informative separation patterns for history, compaction, and
memory; it does not own Personal Context or Memory.

## 4. Prompt-injection and stale-data boundary

Vault, Conversation, external source, MCP, and connector content remains
untrusted. Embedded instructions cannot invoke Tools, alter policy, expand
scope, import capabilities, or change Project authority. Stale/revoked/
forgotten material cannot re-enter through cache, index, compaction, or engine
checkpoint.

## 5. Non-claims

Archive/Vault retrieval and OPC Context composition are **Requires-backend**.
This chapter creates no retrieval-quality, performance, B06/B07, Gate, release,
Profile, or Agent-benefit claim.
