# Knowledge, conversation archive, Memory, and Project Vault

- Status: adopted Personal 2.0 product target
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: owner-approved 2026-08-30 current chrome is
  personal-20-opc-e2e-optimized-v9. v8 is the prior approved baseline (not overwritten). Archived V2 is not current chrome. Canvas-only HITL and daemon authority path remain.
- Existing architecture input:
  [Conversation, Memory, and Vault](../architecture/conversation-memory-vault.md)
- Current Memory/resource facts:
  [Cognitive resource model](cognitive-resource-model.md)

## 1. Personal Home and storage language

The Owner selects a Personal Home. Its product layout separates replaceable
application bytes from retained user data:

```text
Personal Home/
  app/
  data/
    shared/
    projects/<project-id>/
      vault/
      sources/
      artifacts/
      exports/
```

This is a product layout target, not an existing path contract. Daemon
authority state, indexes, SecretStore data, and human-readable files retain
separate ownership. A Project is not its directory; moving, losing, or
temporarily denying that directory must not silently delete Project authority.

## 2. Three knowledge scopes

| Scope | Purpose | Boundary |
|---|---|---|
| Owner-shared knowledge | facts and methods intentionally reusable across Projects | explicit Owner scope and provenance |
| Project Markdown Vault | human-readable Project research, notes, plans-as-projections, and artifacts | Project-isolated, Obsidian-compatible Markdown |
| Member-private Memory | admitted working knowledge for one Member inside one Project | unavailable to other Members unless explicitly promoted/re-scoped |

Secrets belong only in approved Secret Stores. Detecting credential-shaped
content routes the Owner to a secret-import flow; it never turns the material
into Knowledge, Context, Conversation evidence, export, or Memory.

## 3. Import and indexing

Import can accept Owner-selected files/directories, links, images, and video
metadata. The target flow:

1. select destination scope and copy/reference policy;
2. show source, rights, expected parsing/OCR, and permission;
3. preserve the source and content digest;
4. classify, deduplicate, parse/OCR, chunk, and index with progress;
5. retain failed originals and exact failure reasons;
6. let the Owner exclude, reclassify, retry, or remove from the index.

Only Owner-owned, licensed, open-license, or public-domain material may be
copied for reuse. Other sources may support analysis, citation, and original
creation with provenance.

Ordinary knowledge may be written automatically with author, source, and
version, then reindexed. Edits that appear to change a Project charter, Goal,
Role, Member, permission, Provider/model, trigger, or workflow create a
candidate; a file edit cannot bypass daemon authority.

## 4. Obsidian compatibility

The Project Vault uses ordinary Markdown, stable relative links, attachments,
and inspectable metadata so it remains readable by Obsidian and other tools.
Obsidian is proprietary and is not embedded, redistributed, or required. The
Knowledge surface treats the Vault as Markdown files; the optional Obsidian
companion is not drawn inside the App.

An optional companion, URI action, or plugin may be qualified later. It remains
thin, uninstallable, non-authoritative, and secret-free. The Obsidian API and
sample plugin are informative interface references only.

## 5. Conversation archive and active retrieval

All Personal conversations form a local, scoped episodic archive:

- Personal Assistant conversations: Owner/system scope;
- Project group conversations: Owner + Project scope;
- Member work conversations: Project + Member scope;
- Task/Attempt links: explicit references, not inferred completion;
- source messages, receipts, and correction history: retained with provenance.

"All conversations are retained" means they remain inspectable source records
and may be eligible retrieval sources. It does **not** mean all text is
inserted into every prompt or automatically admitted to Memory.
The retrieval boundary is:

```text
scope authorization
  -> secret/PII redaction
  -> freshness and provenance filtering
  -> relevance ranking
  -> bounded Context selection
  -> untrusted-observation label
```

Missing index, stale source, conflicting statement, or redaction loss remains
visible. DSH and Pi receive only the selected Context and cannot query the raw
archive directly.

Each Agent process receives a package bounded by its model's context limit.
The assembly order is Codex-inspired as a behavior reference only: Personal
owns the layers; DSH and Pi receive only the selected bounded Context.

```text
current Task contract
  -> fixed decisions
  -> relevant source and artifact excerpts
  -> provenance-linked summaries
  -> older narrative
```

When over limit, Personal reduces older summaries and narrative first. It does
not discard the current Task contract or fixed decisions. The UI shows a
**Why this fragment** table for selected excerpts. Compression never
rewrites the archive, proves completion, or admits Memory. Memory is not
silent auto-ingest.

## 6. Semantic Memory admission

Ordinary chat does not become Memory. Explicit instructions become formal
revisions; “remember” or stable verified facts may produce Memory candidates.
Conversation fragments, extracted facts, reflections, and external sources
otherwise remain source material. Durable semantic Memory requires:

- explicit Owner intent or a verified source fact;
- source/provenance and timestamp;
- scope, purpose, retention, and conflict policy;
- deterministic admission;
- inspect/correct/promote/forget lifecycle.

Agent self-report, repeated text, manager agreement, or retrieval score does
not admit Memory. Correction creates a new version with lineage. Forget creates
a durable tombstone that prevents index/cache resurrection.

Cross-Project Memory promotion requires Owner confirmation. Accept, reject,
edit, and rate actions are Project feedback evidence. Stable repeated
preferences may produce a versioned Member or global Role revision proposal;
one feedback event never silently changes global behavior.

Letta and Mem0 are extraction/retrieval candidate references only; they do not
write Personal Memory or own an external authority store.

## 7. Recovery and deletion

An index is derived and rebuildable. Archive bytes and admitted Memory have
separate retention and deletion rules. Project archive stops triggers but keeps
read/export access. Permanent deletion previews Project files, archive,
Memory, artifacts, exports, local restore points, and irreversibility.

Local restore points are same-disk versions. They are not disaster backups and
cannot protect against disk failure. Manual export excludes secrets by default.

## 8. Required states and non-claims

Knowledge surfaces cover empty, importing/loading, duplicate, partial, stale,
permission, parse/OCR error, secret-detected, conflict, excluded, indexed,
archived, and forgotten states. Input and originals are preserved after
recoverable failures.

This design is **Requires-backend**. It does not implement storage paths,
import, OCR, indexing, retrieval, Vault sync, Obsidian integration, semantic
admission, privacy controls, or deletion. It creates no backup, support, Gate,
release, Profile, market, or memory-quality claim.
