# 06 — Knowledge, Project Vault, Context, and Memory

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Product source:
  [Knowledge, conversation archive, Memory, and Project Vault](../../../../personal/docs/product/knowledge-memory-vault.md)
- Status: current interaction prototype is owner-approved v5 (2026-08-29); archived pre-v5 and V2 are historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v5**](personal-20-opc-e2e-optimized-v5.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Surface model

Knowledge is a first-level anchor with contextual master/detail for:

1. Owner-shared knowledge;
2. each Project's Obsidian-compatible Markdown Vault;
3. Member-private admitted Memory;
4. source/import and derived index state;
5. Project-group and Member-work conversation archives;
6. Context selection, omissions, conflicts, corrections, promotion, and forget.

Source bytes, index rows, retrieved Context, summaries, conversation archives,
and admitted Memory remain distinct objects.

## Import, rights, and automatic knowledge writing

Import selects files/directories/links/images/video metadata, destination
scope, copy/reference policy, rights, parsing/OCR, permission, and redaction.
Each source preserves origin, retrieval/import time, rights status, digest,
version, scope, and parser/index state. Duplicate, partial, failed, excluded,
secret-detected, and indexed outcomes remain visible; failure preserves the
original and destination.

Only Owner-owned, licensed, open-license, or public-domain material may be
copied for reuse. Other sources support analysis, citation, and original
creation with provenance. Ordinary knowledge may be written automatically with
author/source/version, then reindexed. A file that appears to change charter,
goal, Role, Member, Provider/model, permission, trigger, or workflow becomes a
candidate; it cannot write authority.

Credentials route to SecretStore and never enter Vault, Context, Memory,
Conversation, DOM, URL, browser storage, logs, export, or evidence.

## Project Vault

The Vault uses ordinary Markdown, stable relative links, attachments, and
inspectable metadata. It is compatible with Obsidian but does not embed,
redistribute, or require the proprietary application. The Knowledge surface
treats the Vault as Markdown files. Any future companion is optional, thin,
uninstallable, secret-free, and non-authoritative; it is not drawn inside the
App.

## Conversation archive and model-window-aware Context

Personal owns full local archives for global Assistant, Project group, and
Member work conversations. Archives are inspectable sources; ordinary chat is
not automatically Memory and the full archive is never implied to be injected.

Context assembly follows:

```text
authorize scope
  -> redact secret/PII
  -> filter provenance/freshness/conflicts
  -> current Task contract
  -> fixed decisions
  -> relevant source and artifact excerpts
  -> provenance-linked summaries
  -> older narrative
```

The selected model's context window sets the package bound. The assembly order
is Codex-inspired as a behavior reference only: Personal owns the layers; DSH
receives only the selected bounded Context. When over limit,
Personal reduces older summaries and narrative first, never the current Task
contract or fixed decisions. The UI shows omissions, truncation, stale or
conflicting sources, redaction loss, and a first-class **Why this fragment**
table for each selected excerpt. Memory is not silent auto-ingest: ordinary
chat does not admit Memory. Compression never deletes source archives, changes
authority, proves completion, or admits Memory.

## Memory and feedback

Explicit “remember” intent or a stable verified fact may create a Memory
candidate. Admission requires provenance, timestamp, scope, purpose, retention,
conflict policy, and deterministic policy. Agent repetition, self-report,
manager agreement, summarization, or retrieval score cannot admit it.

Memory detail supports inspect, correct, promote, and forget with lineage.
Cross-Project promotion requires Owner confirmation. Forget creates a durable
tombstone against index/cache resurrection.

Accept, reject, edit, and rate actions become Project feedback evidence. A
one-off event cannot silently alter a Member or global Role. Repeated stable
preference may produce a versioned proposal with comparison and rollback.

## States and capability honesty

The design covers empty, importing, loading, duplicate, partial, stale,
permission, parser/OCR error, secret-detected, conflict, excluded, indexed,
Memory-candidate, corrected, promotion-preview, forgotten, and archived states.

Personal Home paths, import/OCR/index, archives, retrieval, Context composition,
Vault conflict, Memory admission/promotion/forget, and privacy controls are
**Requires-backend**. Prototype actions are labelled state demonstrations, not
filesystem or daemon writes.
