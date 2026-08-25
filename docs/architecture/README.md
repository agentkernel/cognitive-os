# Architecture documents (pointer)

The two architecture collections moved into their subproject roots under
ADR-0054. Neither owns current task, Gate, or evidence status
(`PROGRESS.md` Current snapshot does).

| Collection | New location |
|---|---|
| CognitiveOS architecture layer: whitepaper, RFC-0001, frozen reviews | [core/docs/architecture/](../../core/docs/architecture/README.md) |
| CognitiveOS Personal composition: Shell, Agent lifecycle, authority, data, recovery | [personal/docs/architecture/](../../personal/docs/architecture/README.md) |

Machine contracts stay in `core/specs/` and `core/conformance/`. Do not copy
those shapes into these documents.
