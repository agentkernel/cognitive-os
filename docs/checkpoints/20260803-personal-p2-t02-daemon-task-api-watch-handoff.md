# P2-T02 daemon Task API/watch blocker handoff

- Date: 2026-08-03
- Task and slice: `P2-T02/D01`
- Change class: corrective planning/evidence record; normative and product
  surfaces unchanged
- Lease: `lease/personal/P2-T02/daemon-task-api-watch`, closed
- Status at handoff: `blocked`; no task implementation was started

## Recovery tuple

| Field | Value |
|---|---|
| Branch | `lane/personal-p2-t02-daemon-task-api-watch` |
| Base revision | `main@962ea96f568131809960eb75f4b4475c1be16846` |
| Immutable blocker checkpoint | `335b063b46553643b5a503c2fccf9e0fe01d9896` |
| Upstream | `origin/lane/personal-p2-t02-daemon-task-api-watch` at the blocker checkpoint |
| Worktree | handoff metadata update pending checkpoint commit |
| Lease | closed in this delivery |
| Pull request | [#137](https://github.com/agentkernel/cognitive-os/pull/137), Draft |

## Verified blocker

`P2-T02/D01` requires a real, daemon-owned Task API/watch vertical path:
server-issued preview-to-admit and cursor-resume/dedup watch semantics.

The existing P2-T01 `KernelTaskApplicationService` exposes deterministic
preview and admission primitives, but the Personal daemon only returns a
generic authenticated-front-door response for `/task/*`. The TypeScript
transport preserves the exact `watch.open`/`watch.resume` envelope, but the
daemon has no registered wire representation to parse or authoritatively
return.

The formal plan explicitly lists the server-issued preview and Task watch
public wire semantics as Lane-CTR public-contract prerequisites. Implementing
ad hoc JSON request/result types or synthetic SSE behavior in Lane-RUN would
create an unregistered public contract and violate the contract boundary.

## Required next action

- `blocked_paths`: `specs/`, generated bindings, and the task/watch public
  wire contract.
- `blocked_task_ids`: `P2-T02/D01`.
- `blocked_gate_ids`: none.
- Owner: Lane-CTR.
- Next action: claim a narrow Lane-CTR lease to register and freeze the
  server-issued preview/admit request/result and authoritative watch-resume
  contract. Then create a new P2-T02 Lane-RUN continuation lease to add the
  daemon composition and failure-first Rust/TypeScript integration tests.

## Validation

| Check | Result |
|---|---|
| formal P2-T02/D01 slice and Lane-CTR prerequisite review | pass |
| active lease conflict review | pass; no other active lease |
| `pnpm run check:consistency` | not-run; pending this documentation checkpoint |
| `git diff --check` | not-run; pending this documentation checkpoint |
| Rust build/test/Clippy | not-run; no implementation was started and local GNU Rust linking is prohibited |

## Non-claims

This record adds no Task API implementation, test execution evidence, task
completion, Gate result, release claim, Profile claim, public schema, binding,
transition, vector, or normative contract change.
