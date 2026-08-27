# 06 — Knowledge, Vault, and Memory

## Surface model

Knowledge uses master/detail across:

1. Owner-shared knowledge;
2. each Project Markdown Vault;
3. employee-private Memory;
4. source/import and index status;
5. Conversation archive retrieval;
6. conflicts, corrections, exclusions, and forget.

The surface never merges source files, index rows, retrieved Context, and
admitted semantic Memory into one object.

## Import flow

1. Choose files/directories/links/images/video metadata.
2. Select shared or Project destination and copy/reference policy.
3. Review source rights, expected parsing/OCR, permissions, and redaction.
4. Start import and show per-source progress.
5. Classify/deduplicate/parse/OCR/index.
6. Review indexed, duplicate, failed, excluded, or secret-detected outcomes.

Failure preserves the original and destination choice. The Owner can retry,
reclassify, exclude, or remove from index. Credentials route to a SecretStore
flow and never enter the Vault, archive, Memory, DOM, logs, or evidence.

## Rights and provenance

Only Owner-owned, explicitly licensed, open-license, or public-domain material
may be copied for reuse. Other sources may be analyzed, cited, and used to
create new work with provenance. Every source shows origin, retrieval/import
time, rights status when known, digest/version, scope, and parser/index state.

## Project Vault

The Vault stays ordinary Markdown with stable relative links and attachments.
It is Obsidian-compatible, but the proprietary Obsidian application is not
embedded, bundled, or required. A future companion remains optional,
uninstallable, non-authoritative, and secret-free.

Ordinary content changes trigger reindex. A file that appears to alter charter,
goal, role, permission, budget, Provider, trigger, or workflow is a candidate
with a structured diff; it cannot update daemon authority directly.

## Conversation retrieval

All Personal conversations may be indexed by Owner/Project/employee scope.
Active retrieval follows:

`authorize scope -> redact -> filter freshness/provenance -> rank -> bound ->
label untrusted observation`.

The UI shows why a fragment was selected, what was omitted, and whether the
index/source is stale. It never suggests that the full archive is injected.
DSH and Pi receive only the selected bounded Context.

## Semantic Memory

A Conversation/source/reflection yields a candidate. Admission requires
explicit Owner intent or verified facts plus provenance, scope, purpose,
retention, and conflict policy.

Memory detail shows candidate/admission source, versions, use scope,
corrections, conflicts, expiry, and forget tombstone. Agent repetition,
self-report, manager agreement, or retrieval score cannot admit Memory.

## States

Visible prototype states include:

- first-run empty and select-import action;
- importing/loading with real source counts;
- duplicate and partial parsing;
- stale index;
- permission denied;
- parser/OCR failure with original retained;
- secret detected and redirected;
- conflict/candidate awaiting review;
- indexed success;
- Memory corrected/forgotten;
- archived Project Vault.

## Requires-backend

Personal Home paths, import/OCR/index, archive retrieval, Vault conflict,
Conversation indexing, semantic admission, and privacy/forget composition are
unimplemented target behavior. Prototype actions are state demonstrations, not
filesystem or daemon writes.
