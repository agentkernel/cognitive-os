# Resource Manager — architecture

- Status: informative target/design for CognitiveOS Personal
- Change class: `implementation-only` companion (no new public contract)
- Product pair: [resource-manager-design.md](../../product/personal/resource-manager-design.md)
- Grounding: [system-architecture.md](./system-architecture.md) §3.1–3.2,
  [ADR-0037](../../adr/0037-personal-unified-cognitive-resource-substrate.md),
  [cognitive-resource-model.md](../../product/personal/cognitive-resource-model.md)

This document records how Personal implements the common
`ResourceApplicationService` vocabulary without collapsing six domains into one
schema or one giant resource state machine.

## Placement

The manager is a daemon-private HTTP surface in
`apps/kernel-server/src/personal/resource_manager.rs`. Routes are matched
before the `/management/resource/` family catch-all so they do not fall through
to Memory/Skill handlers, and before `/task/resource/` rewrite so task callers
receive an explicit channel denial instead of a projection 404.

Watch stays on existing `GET /resource/v1/watch`. The manager does not add a
second SSE.

## HTTP (management)

| Method | Path | Behavior |
|---|---|---|
| `GET` | `/management/resource/v1/list?family=` | bounded family page |
| `GET` | `/management/resource/v1/inspect?family=&id=` | one envelope |
| `POST` | `/management/resource/v1/bind` | typed bind |
| `POST` | `/management/resource/v1/unbind` | typed unbind |
| `POST` | `/management/resource/v1/enable` | typed enable |
| `POST` | `/management/resource/v1/disable` | typed disable |
| `POST` | `/management/resource/v1/revoke` | typed revoke |
| `POST` | `/management/resource/v1/create\|install\|execute\|complete` | fail closed |

The same paths under `/task/resource/v1/` return
`RESOURCE_MANAGER_CHANNEL_FORBIDDEN`.

Mutating bodies require `family`, `id`, integer `expected_version`, and a
non-empty, non-secret-shaped `idempotency_key`. Extra domain fields (for
example Skill `revision_id` / `reason`) travel in the same JSON object.

Stale `expected_version` returns `409 RESOURCE_MANAGER_VERSION_STALE`.
Unsupported family+operation pairs return
`400 RESOURCE_MANAGER_OPERATION_UNSUPPORTED`.

## Authority sources (no invented rows)

| Family | List/inspect source | Mutating sinks |
|---|---|---|
| Tool | `BUILTIN_TOOL_CATALOG` plus overlay lifecycle | existing tool overlay enable/disable/revoke. Default no-overlay version is Enabled = 1 |
| Memory | non-tombstoned `memory_objects` | none on this envelope (`remember`/`forget` stay family routes; forget is not revoke) |
| Skill | `skill_bindings` plus revocation existence | existing `bind_skill` / `revoke_skill_binding`. Active = version 1, revoked = 2; bind uses expected_version 0 |
| Task | current `task_contracts` | none |
| Context | honest empty, `authority_source: projection-only` | none |
| Runtime | honest empty, `authority_source: projection-only` | none |

Object versions are domain guards, not a universal CAS column. Tool overlay
states map Enabled=1, Disabled=2, Quarantined=3, Revoked=4 so a missing overlay
is not version 0.

## Invariants

- Daemon-only authority writer (A1).
- No public contract/schema/transition change.
- Envelope is a projection, not a writable aggregate (ADR-0037).
- Persist-before-dispatch Intent/Effect, fencing, budget, and independent
  verification are unchanged for any domain mutation that already required them.
- Secrets never appear in argv, env, config, logs, evidence, or the
  idempotency key.
