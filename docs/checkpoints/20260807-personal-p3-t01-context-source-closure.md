# P3-T01 Context source/retrieval port closure

- **Date:** 2026-08-07
- **Classification:** `implementation-only`
- **Task:** `P3-T01`
- **Final slice:** `P3-T01/D01`
- **Branch:** `personal/P3-T01-context-source-closure`
- **Draft PR:** #161
- **Exact task revision:** `0ad1ddb95f4e347d0c205597e69ad8818819948e`

## Acceptance mapping

| Formal acceptance | Delivered boundary and evidence |
|---|---|
| Real workspace, task, and evidence Context source | The immutable TaskContract v0.4 ContextRequest binds Context to its task. Daemon-admitted workspace sources preserve immutable provenance and typed roles, including working, authoritative-state, and evidence inputs. ContextRequest, source and sealed ContextView rows are append-only. |
| Scope filtering before ranking | The daemon performs tenant, resource-scope, and optional conversation filtering on source metadata before any body load or ranking. The real scheduler path resolves and persists the sealed request-bound ContextView before candidate-only Pi transport. |
| Revocation safety | Before every source body read, the resolver reconstructs current durable authorization and revocation facts. The integration negative proves a revocation after metadata discovery blocks body materialization, ranking, ContextView persistence, private Pi transport, and candidate admission. |
| Owner-local MVP authorization | One owner-local management session admits the required authorization/revocation facts. The daemon remains the sole writer; Pi only receives bounded resolved Context and returns an untrusted candidate shape. |

## Validation

- `cargo fmt --all -- --check`: passed locally on the registered static-only
  Windows host.
- `pnpm run check:consistency`: passed locally.
- `git diff --check`: passed locally.
- Exact native Linux `DEV-LINUX-NATIVE-01`, clean detached checkout at
  `0ad1ddb95f4e347d0c205597e69ad8818819948e`:
  - `cargo test -p kernel-server`: passed;
  - `cargo test -p cognitive-store --test m5_context_store`: passed, 9/9.
- Required CI for exact PR head revision: Ubuntu and Windows/MSVC `verify`
  jobs passed in GitHub Actions run `31172218046`.

## Non-claims

This closes P3-T01 only. B03 remains `not-run`. No Context benefit result,
Tool execution, Task acceptance/completion, release, or Profile claim is made.
