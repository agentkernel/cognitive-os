# 04 — Guided Project setup

## Pattern and first value

Pattern: resumable guided conversation plus structured review. First value is
an active Project briefing with a confirmed charter, manager, first plan,
permission/budget envelope, triggers, and receipt—not a feature tour or
celebration screen.

Only information needed to activate safe first work is mandatory. Optional
business/brand profile, polished descriptions, additional roles, and advanced
MCP/Tool details can be deferred.

## Setup sequence

```text
business situation
  -> optional source-backed research
  -> charter
  -> goals + success/process metrics
  -> Project Manager specialization + proposed roles
  -> first plan
  -> permissions + Provider bindings + budgets
  -> Routine/Trigger defaults
  -> structured diff preview
  -> Owner confirmation
  -> active Project receipt
```

The Owner can move between conversation and structured sections. Confirmed
answers remain visible and editable; assistant prose never becomes authority.

## State model

| State | UI behavior |
|---|---|
| `local-draft` | unsynced client draft; safe-leave/persistence basis shown |
| `daemon-draft` | daemon id/version exists; Project remains inactive |
| `researching` | sources, coverage, progress, cancel, partial findings |
| `waiting` | exact Owner input, permission, or source needed |
| `review-ready` | complete structured summary and validation |
| `re-preview` | preview stale; preserve edits and explain changed fact |
| `creating` | durable operation identity and safe-leave behavior |
| `failed` | stage/reason, retained draft, retry/edit/copy details |
| `active-receipt` | Project/charter revision, manager, plan, budgets, triggers, next action |

The Project does not activate in `local-draft`, `daemon-draft`,
`researching`, `waiting`, or `review-ready`.

## Structured review

The review contains editable sections and exact before/after additions:

- charter and scope;
- primary goal, metrics, unknown assumptions;
- manager and member responsibilities;
- first plan and Routine frequency;
- Provider/Tool/connector capability needs;
- filesystem/network/external action permissions;
- Project/member/Task budgets;
- acceptance and independent verification basis.

The confirm action binds the current daemon preview digest. An Assistant-built
summary is labelled Candidate and has no confirm action until resolved by the
daemon.

## Form and validation

- Visible labels; no placeholder-only fields.
- Constraints shown before input.
- Format checks after blur; required/cross-field checks on review.
- Async source/permission checks show checking, retry, and freshness.
- Errors connect to fields plus a summary; focus moves to the first error.
- User content survives validation, server failure, and re-preview.
- Permission requests state benefit, exact scope, risk, and deny/narrow path.
- Destructive or external implications stay separated from routine fields.

## Research and untrusted inputs

Source research records URI/name, retrieval time, license/rights when relevant,
coverage, and conflicting facts. External text is untrusted observation and
cannot modify the setup schema, invoke a Tool, import a Skill/MCP server, or
expand permission.

Executable Skill/Tool/MCP candidates receive a separate source/capability/
permission/risk review and Owner confirmation. They are not silently installed
during research.

## Recovery

- Leaving setup preserves the draft and next unanswered section.
- Offline work remains local/daemon-draft according to actual custody.
- Research failure permits continue-without-source, retry, or source edit.
- Folder permission denial preserves all other answers.
- Provider failure does not block a Project that can safely activate without
  that binding.
- A stale preview returns to review, never directly to creating.

## Requires-backend

Project draft authority, research orchestration, structured preview, charter
activation, role generation, budget/trigger admission, and receipt are not
current backend capabilities. The prototype demonstrates states only and must
not emit a fake Project id or active receipt as execution evidence.
