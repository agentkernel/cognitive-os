# Personal Conversation, archive, retrieval, Vault, and Memory architecture

- Status: Personal 2.0 target; `Requires-backend`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product: [Knowledge, Memory, and Vault](../product/knowledge-memory-vault.md)
- Private-envelope boundary:
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)

## 1. Ownership layers

| Layer | Owner | Authority |
|---|---|---|
| Conversation archive | Personal data service | scoped append/read/correct/redact/retention; never Task completion |
| Project Vault source | Owner/Project filesystem | human-readable Markdown/source content; not Project authority |
| Derived index | Personal indexing service | rebuildable retrieval aid; never authorization or Memory authority |
| Context selection | daemon Context authority | authorization-before-ranking, bounded selection, explicit loss |
| Semantic Memory | daemon Memory authority | candidate -> deterministic admission -> version/correct/forget |
| Secret material | approved SecretStore | excluded from all layers above |

DSH and Pi consume only a bounded Context view and submit candidates. They have
no archive query, Memory write, or SecretStore authority.

## 2. Conversation identity

A Personal Conversation is scoped to Owner, Project, and Personal Assistant or
employee. It references Task/Attempt/artifact/receipt identities where known
but does not own them. Engine process/session identity is separate.

ADR-0058's `cognitiveos.personal.conversation-projection/0.1` described a
vendor/dsh history projection. ADR-0059 supersedes that first-slice role but
does not reinterpret the envelope. The OPC archive needs a new private version
or future Lane-CTR before implementation.

## 3. Ingestion and indexing

```text
source bytes/message
  -> scope + rights + secret/PII prefilter
  -> immutable source/provenance reference
  -> parser/OCR/chunk candidate
  -> derived index
  -> authorized retrieval candidate
```

Parser/index failures preserve the original and source metadata. Index rebuild
cannot resurrect forgotten Memory or bypass archive retention. Credential-like
content is refused from Knowledge/Conversation-derived indexes and routed to
SecretStore handling.

Ordinary Vault edits reindex. Goal/role/permission/budget/Provider/trigger/
workflow-shaped edits create candidates and cannot mutate daemon authority.

## 4. Retrieval

Retrieval applies:

1. principal/Project/employee/purpose scope;
2. retention/tombstone/permission filtering;
3. secret/PII redaction;
4. source freshness/provenance/conflict checks;
5. ranking within the authorized set;
6. token/fragment budget and explicit omission/loss;
7. untrusted-observation labelling.

The archive may be complete while a given Context is deliberately small.
Missing index, stale source, conflict, or truncation is visible. No full-archive
prompt injection occurs.

## 5. Memory admission and reflection

Conversation fragments, Vault facts, connector readback, and daily/weekly
reflection produce candidates. Durable Memory requires verified fact or
explicit Owner intent, source/provenance, scope/purpose, retention, conflict
disposition, and deterministic admission.

Correction creates a new version; forget writes a tombstone and invalidates
derived retrieval. Letta/Mem0 are extraction/retrieval references only and
cannot directly write Memory or own another authority store.

## 6. Obsidian companion

Vault format remains ordinary Markdown. Obsidian is proprietary and neither
embedded nor required. Any optional companion/plugin/URI adapter is a
secret-free non-authority client, independently versioned and removable. It
cannot become sync, archive, index, Memory, or Project authority.

## 7. Recovery and privacy

Archive and authority stores follow separate backup/restore policies. Manual
export identifies content and excludes secrets. Same-disk local restore points
are not disaster backups. Project archive stops triggers but preserves
read/export; permanent deletion previews all affected layers.

## 8. Non-claims

No archive schema, index, OCR/parser, retrieval, Vault sync, Obsidian adapter,
Memory admission integration, correction/forget UI, or privacy qualification
is implemented here. No support, Gate, release, Profile, recall-quality, or
Agent-benefit claim follows.
