# Knowledge, conversation archive, Memory, and Project Vault

- Status: adopted Personal 2.0 product target
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Architecture:
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
| Employee-private Memory | admitted working knowledge for one employee inside one Project | unavailable to other employees unless explicitly promoted/re-scoped |

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

Ordinary knowledge edits trigger reindexing. Edits that appear to change a
Project charter, Goal, role, permission, budget, Provider, trigger, or workflow
create a candidate; a file edit cannot bypass daemon authority.

## 4. Obsidian compatibility

The Project Vault uses ordinary Markdown, stable relative links, attachments,
and inspectable metadata so it remains readable by Obsidian and other tools.
Obsidian is proprietary and is not embedded, redistributed, or required.

An optional companion, URI action, or plugin may be qualified later. It remains
thin, uninstallable, non-authoritative, and secret-free. The Obsidian API and
sample plugin are informative interface references only.

## 5. Conversation archive and active retrieval

All Personal conversations form a local, scoped episodic archive:

- Personal Assistant conversations: Owner/system scope;
- manager and employee conversations: Project + employee scope;
- Task/Attempt links: explicit references, not inferred completion;
- source messages, receipts, and correction history: retained with provenance.

"All conversations participate in active memory" means they are eligible
retrieval sources. It does **not** mean all text is inserted into every prompt.
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

## 6. Semantic Memory admission

Conversation fragments, extracted facts, reflections, and external sources
produce Memory candidates. Durable semantic Memory requires:

- explicit Owner intent or a verified source fact;
- source/provenance and timestamp;
- scope, purpose, retention, and conflict policy;
- deterministic admission;
- inspect/correct/forget lifecycle.

Agent self-report, repeated text, manager agreement, or retrieval score does
not admit Memory. Correction creates a new version with lineage. Forget creates
a durable tombstone that prevents index/cache resurrection.

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
