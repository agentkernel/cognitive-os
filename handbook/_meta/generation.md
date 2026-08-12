---
doc_id: meta.generation
locale: en
kind: meta
audience: [developer, ai]
generated: false
---

# Handbook generation and fingerprint specification

- Status: handbook meta specification (informative; creates no product or contract semantics)
- Owner tooling: `tools/src/generate-handbook.mjs`, `tools/src/check-handbook.mjs`

## 1. Page fingerprints

Every non-navigation, non-meta page declares `sources[]` and a `fingerprint` in its
frontmatter. The fingerprint is computed as:

1. Collect every `sources[].path`, sort lexicographically (byte order).
2. For each path, read the tracked file bytes and normalize CRLF (`\r\n`) to LF (`\n`).
3. Concatenate: `"cognitiveos-handbook-source/0.1\n"` once, then for each file
   `path + "\0" + normalized_bytes + "\n"`.
4. `fingerprint = "sha256:" + lowercase_hex(SHA-256(concatenation))`.

The checker recomputes fingerprints from the current worktree. A mismatch means the
mapped sources changed after the page was last reviewed: update the page (or rerun the
generator for generated pages) and refresh the fingerprint in the same PR. Fixed line
numbers are never part of the mapping; `symbols` entries must literally appear in their
source file and are checked instead.

This digest domain is handbook-private. It deliberately does not reuse
`docs/standards/canonical-encoding-and-digest.md` domains, so a handbook fingerprint can
never be confused with a registered contract digest.

## 2. Source-set record

`_meta/source-set.json` records the implementation reading baseline:

- `implementation_baseline_revision`: the exact `origin/main` commit whose tree the
  handbook content was authored against (never a self-referential "current HEAD");
- `digest`: SHA-256 over the sorted `path + "\0" + git_blob_sha + "\n"` lines of every
  tracked file at that revision, excluding `handbook/**`, `llms.txt`,
  `docs/plan/PROGRESS.md`, `docs/plan/PARALLEL-LANES.md`, `Cargo.lock`,
  `pnpm-lock.yaml`, and `History/**` (domain prefix
  `cognitiveos-handbook-source-set/0.1\n`).

The checker verifies internal consistency (the digest is reproducible from the recorded
revision via `git ls-tree`). It does not require the record to equal the current HEAD;
new files are instead caught by the coverage check, and stale pages by per-page
fingerprints.

## 3. Generated pages

`generated: true` pages are produced by `tools/src/generate-handbook.mjs` from
implementation and machine-contract sources only:

| Page family | Machine inputs |
|---|---|
| `ref.errors` | `specs/registry/errors.yaml` |
| `ref.transitions` | `specs/transitions/*.transitions.json` |
| `ref.schemas` | `specs/schemas/*.json` (`$id`, title, description) |
| `ref.cli-cognitive` | `COGNITIVE_USAGE` in `apps/admin-cli/src/personal_cli/mod.rs` + parsed verb set |
| `ref.cli-admin` | `USAGE` in `apps/admin-cli/src/main.rs` + parsed verb set |
| `ref.http-api` | route string literals in `apps/kernel-server/src/personal/{server.rs,task_api.rs,resource_api.rs}` cross-checked against `_meta/http-routes.json` annotations |
| `ref.env-vars` | `env::var*("...")` literals across `apps/**` and `packages/**` cross-checked against `_meta/env-vars.json` annotations |
| `ref.config-files` | config file-name literals in `crates/cognitive-secret` and `apps/admin-cli` cross-checked against `_meta/config-files.json` annotations |
| `ref.tool-catalog` | native tool ids/limits in `crates/cognitive-kernel/src/tool_registry.rs` |

Rules:

1. The generator writes both locales (`handbook/en/reference/…` and
   `handbook/zh-CN/reference/…`); page scaffolding text is bilingual, extracted
   machine values (usage text, codes, ids, states) stay verbatim.
2. `--check` regenerates in memory and fails on any byte difference; hand edits to
   generated pages are therefore build failures.
3. Annotation files (`http-routes.json`, `env-vars.json`, `config-files.json`) are
   hand-maintained machine inputs. The generator fails when an extracted literal has no
   annotation or an annotation has no extracted literal, so annotations cannot rot.

## 4. Capability status

`status` is judged from code, contracts, and tests together, at the recorded baseline:

- `implemented`: a real caller path exists, focused tests exist, and behavior matches
  the linked contracts;
- `partial`: meaningful pieces exist but a required caller, wiring, or surface is
  absent (the page must say exactly which);
- `designed`: contracts/design documents exist without a usable implementation path;
- `unavailable`: neither a usable implementation nor an accepted design exists, or the
  capability is explicitly excluded.

When a normative source and the implementation disagree, the page states both sides
("规范要求 / 当前实现", "contract requires / implementation does") and never silently
picks one. Dynamic facts owned by `docs/plan/PROGRESS.md` (task, Slice, Gate, lease,
campaign status) are linked, never copied.
