# Personal 2.0 OPC Control Plane architecture

- Status: informative target over current daemon-served client
- Current stack/security: [ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md)
- Current product decision:
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product design: [Web UI](../product/web-ui-design.md)
- Interaction design:
  [client OPC corpus](../../../clients/docs/design/opc-2.0/README.md)

## 1. Current topology

The delivered React/TypeScript client remains at `clients/pc/web/` and is
served same-origin by the Personal daemon under `/ui/`. Separate Vite preview
is not the product origin. Loopback Host/Origin checks, no-cookie bearer
sessions, memory-only client credentials, channel separation, CSP, escaped
untrusted output, and browser secret isolation remain unchanged.

The current Linux-era routes and P7-T05 capabilities are factual but do not
implement the OPC IA, Windows host/tray, Project/Employee model, Personal
Conversations, or managed DSH product.

## 2. Target client composition

```text
OpcAppShell
  -> channel/session boundary
  -> route + Project/employee scope
  -> independent projection caches
  -> Today/Projects/Team/Knowledge/Inbox/Settings
  -> one Conversation rail/composer
  -> structured preview + receipt surface
  -> state/freshness/capability boundary
```

The client consumes separate daemon projections rather than one universal DTO:

- Project/Goal/Plan/Routine/Task/Attempt;
- Role/Assignment/Employee/runtime;
- Conversation/archive/retrieval/Memory;
- Inbox/approval/Effect/recovery;
- Provider/binding/budget/usage;
- Installed Agent/supply-chain/health;
- Windows host/background/missed state.

Each cache preserves source, version/cursor, freshness, coverage, and last safe
snapshot. The client may group/sort for presentation but cannot invent
authority, total order, completion, policy, or conflict resolution.

## 3. Route and state boundary

The target routes are design paths under Today, Projects, Team, Knowledge,
Inbox, and Settings. Current route inventory remains a frozen P7 input and is
not silently rewritten as backend support.

Every route defines loading, empty, partial, stale, permission, error, unknown,
offline, missed, success, and archived behavior. A `Requires-backend` action is
rendered as a dependency explanation, not an active or disabled command.

On route change, focus moves to the main heading. Back/forward restores
Project, filters, selection, and scroll. Narrow Windows layouts use drawers,
sheets, and detail routes while preserving the current recipient and drafts.

## 4. Personal Assistant and single composer

The right rail contains either the Personal Assistant or one employee
Conversation. It is one component with one active recipient:

- independent drafts are keyed by recipient/scope;
- switch saves/restores but never sends/merges/clears;
- submit label names recipient;
- receipts and structured previews live in the main surface;
- Assistant/employee output remains candidate/observation;
- raw chain-of-thought and secrets are never rendered.

Pi may support the Assistant behind the daemon boundary. The browser does not
call Pi or Provider directly. DSH supplies bounded candidate output through the
daemon-managed runtime/broker; no native DSH UI or Conversation is embedded.

## 5. Mutation protocol

```text
client candidate/request
  -> daemon auth + current version/policy/budget
  -> daemon-issued structured preview
  -> Owner confirm/edit/narrow/reject
  -> Intent/Effect before external dispatch
  -> result/reconcile
  -> independent evidence + receipt
```

Only the exact current preview can confirm. Client optimistic state is never a
durable receipt. After daemon restart, sessions/caches are discarded or stale;
mutations are not replayed from browser memory.

## 6. Secret and untrusted-data boundary

The browser has no direct SQLite, filesystem, process, shell, SecretStore, or
Provider network access. Provider keys, bootstrap/session tokens, resolvable
SecretRefs, raw headers, prompts, and unbounded content never enter DOM, URL,
storage, history, telemetry, support, or export.

Conversation/Vault/Agent/connector content is escaped and source-labelled.
Daemon redaction is authoritative; client redaction is defense in depth.

## 7. Current/target matrix

| Capability | Status |
|---|---|
| daemon-served same-origin client and current P7 surfaces | **Now** |
| OPC IA/app shell | **Requires-backend** |
| Project/Role/Employee and Routine/Attempt projections | **Requires-backend** |
| Personal Conversation archive/index/retrieval | **Requires-backend** |
| Personal Assistant/Pi integration and single-composer persistence | **Requires-backend** |
| managed DSH Installed Agent dossier/runtime actions | **Requires-backend + Requires-environment** |
| Inbox approval/recovery/missed-run composition | **Requires-backend** |
| Windows tray/background/close choice | **Requires-backend + Requires-environment** |
| X connector actions | **Requires-backend + Requires-environment** |

The Canvas prototype is design evidence only. This chapter creates no route,
backend capability, accessibility conformance, human usability, support,
qualification, Gate, release, or Profile claim.
