# 18 — Library Spec (Memory · Skills · Tools · MCP)

- Status: adopted Personal 2.0 Library/MCP target; historical resource spec retained
- Updated: 2026-08-27
- Decision:
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)
- Current implementation: Memory/Skill/Tool depth; MCP family
  Requires-core + Requires-backend

## Personal 2.0 Library spec

Resources becomes **Library**, organized around recurring owner tasks and four
family-native sections: Memory, Skills, Tools, and MCP.

Personal 2.0 has seven families overall: Memory, Skill, Tool, Context, Task,
Runtime/Process, and MCP. Context and Task live in Work; Runtime/Process lives
in Agents. Model, Permission, Artifact, Budget, Evidence, and Event remain
cross-cutting objects. This placement does not claim a universal Resource table
or shared lifecycle.

### Library shell

The landing surface is a compact family index plus recent/conflicted items, not
a card wall. Each family page uses master/detail/inspector and answers:
what is it, where did it come from, what revision is current, who may use it,
what is stale/conflicted, and which typed actions really exist.

Families may group entries by task (for example "prepare an Agent", "ground a
Goal", "connect external capabilities") without hiding the underlying family,
source or identity.

### MCP first-class page

MCP gets its own family route with server/source identity, transport and trust
facts, advertisement summaries and their candidate mappings, capability/policy
status, connected Agents/Work, freshness and errors. Discovery is observation;
binding and use require daemon authorization. MCP plus rules never controls the
host Agent session. The current MCP adapter/Tool evidence does not provide this
product surface: all unsupported discovery, lifecycle, binding and
observation/writeback operations are `Requires-core + Requires-backend`.

The inspector keeps seven MCP identities distinct: server, package, connection,
capability revision/digest, binding scope, health, and quarantine. The target
lifecycle covers acquire/import, registration, inspection, connection,
capability refresh, binding, enable/disable, quarantine, requalification,
reconciliation, and removal; this document defines no transition names or API.

Advertised MCP objects stay candidates:

- tools map into Tool admission and never auto-enable;
- resources/content map into Context admission;
- prompts/instructions map into Skill admission.

They are not MCP-family members merely because an MCP server advertised them.
Connecting a server or projecting configuration grants no Tool, Context,
workspace, network, model, secret, or host-session authority. After a first
explicit authorization, admin-preauthorized projection may proceed only inside
the exact admitted scope.

### Support preference and reconciliation

The preference order is:

1. vendor-native API;
2. managed Adapter;
3. MCP plus rules as a cooperative fallback.

`MCP-cooperative` cannot establish native host-session control, ordered native
history, runtime attachment, interrupt, fork, close, or login semantics unless
those facts independently come from the native integration.

Automatic reconciliation is allowed only when the prior grant is exactly
unchanged: same permission set, client identity, trust boundary, target, and
compatible server/package/capability revision. A fresh daemon preview and
confirmation are required for permission expansion, a new client,
trust-boundary or target expansion, or an incompatible update. Reload/restart
is a separate typed supported action with its own runtime condition, effect,
and receipt; it is not an auto-reconcile side effect.

### Federated observation and writeback

Library can target a federated view across Personal and Agent-native sources.
Every copy carries source, revision/digest, observed time and authority class.
Conflicts show both versions. The Agent Shell may propose a resolution, but a
writeback is actionable only through a typed daemon flow:

`candidate -> preview -> confirmation -> persisted Intent/Effect -> dispatch ->
verification/receipt`.

Without that path the page explains `Requires-backend` and provides no active
control or optimistic update.

### Current-backed depth

The existing Memory, Skill and Tool routes remain current implementation. MCP
is target-only and Requires-core + Requires-backend. The older
Memory/Skill/Tool/Context specification below remains the P7-T05 fallback and
audit rationale; Context's target placement is Work.

---

## Historical 2026-08-24 Resources specification

## 1. Hub (the space's landing)

Not a card wall: a **family index** — four quiet rows, each carrying one live fact and one entry action:

```text
┌──────────────────────────────────────────────────────────────────┐
│ Resources                                                         │
│  Memory     12 admitted · 1 tombstoned (30d)        → browse      │
│  Skills     3 packages · 2 bound                    → browse      │
│  Tools      7 registered · 6 enabled · 1 quarantined → browse     │
│  Context    per-task views — open from a task       → Work        │
└───────────────────────────────────────────────────────────────────┘
```

Each row's fact line comes from the real list endpoints (envelope counts, labeled when envelope-only). Context's row is honest by design: Context is per-task, so its "browse" is a pointer into Work, not a fake standalone browser (no standalone HTTP surface — `06` §3.5).

## 2. Memory

```text
┌──────────────────────────────────────────────────────────────────┐
│ Resources / Memory                              [+ Remember]      │
│ ┌───────────────────────────────────────────┬───────────────────┐│
│ │ master (envelope-honest)                  │ inspector         ││
│ │ ● mem-7f2a…  workspace · v3 · exp 90d     │ mem-7f2a…         ││
│ │ ● mem-91bc…  owner · v1 · no expiry       │ admitted · v3     ││
│ │ ○ mem-44de…  tombstoned 12d ago           │ scope workspace   ││
│ │                                           │ purpose formatting││
│ │ list is id-envelope only (limit 64)       │ provenance:       ││
│ │ content search is BD-6                    │  candidate c-11…→ ││
│ │                                           │  decision d-08…   ││
│ │                                           │ [Forget…]         ││
│ └───────────────────────────────────────────┴───────────────────┘│
└───────────────────────────────────────────────────────────────────┘
```

- **Master:** id (mono short), scope, version, expiry/tombstone; footer honesty: envelope limit 64; content search labeled BD-6 (not available over HTTP).
- **Inspector/detail (explain):** the provenance chain candidate→decision→object, scope, purpose, versions, expiry, tombstone state, canonical content (redacted-rendered, escaped). Actions: **Remember [A]** (governed form: text, scope, purpose, retention ≤ 365 d cap stated, provenance ref) and **Forget [A]** (consequence copy: "creates a durable tombstone; stale copies cannot resurrect it").
- Rules: unknown fields stay unknown; tombstoned objects are visible as tombstones (the product's forget-proof), never silently absent.

## 3. Skills

- **Master:** package · revision (digest short) · binding status (bound/revoked + scope/target) · source provenance.
- **Detail:** revision inspect (manifest digest, content digest, compatibility) + binding explain (binding_id, revision, workspace scope, target kind/ref, status, revocation reason). Actions: **Import [A]** (local path flow: preview of normalized manifest + digests + compatibility before persist), **Bind [A]** (CAS), **Revoke [A]** (reason required; consequence stated).
- Standing annotation (content ≠ permission): "An enabled skill grants no tool, filesystem, network, or model capability. Scripts execute only through registered tools." Rendered once per page as quiet caption — a principle, not a nag.

## 4. Tools

```text
┌──────────────────────────────────────────────────────────────────┐
│ Resources / Tools                                                 │
│  operation              risk     lifecycle    readiness           │
│  native.workspace.read  read     ● enabled    execution-ready*    │
│  native.workspace.write write    ● enabled    execution-ready*    │
│  native.process.check   process  ■ quarantined — (one-way; revoke │
│  …                                to remove)                      │
│  *execution-ready ≠ production-wired for all families (see note)  │
└───────────────────────────────────────────────────────────────────┘
```

- Table, not cards: operation id (mono), action/family, risk class, descriptor digest (short), lifecycle state, execution readiness **with the standing caveat** ("registered/enabled ≠ production call chain wired" — `tool_executor/mod.rs:48-52` honesty, rendered as a page-level note, not per-row noise).
- Detail/inspector: descriptor fields (input/output limits, required capability), per-task exposure when a task_ref context exists.
- Actions **[A]**: enable / disable / quarantine / revoke — each with its transition rules in the confirm copy (quarantined→enabled refused; revoked terminal; quarantine is one-way except revoke — stated *before* confirm, per blast-radius audit `08` §5).

## 5. Context

- No standalone browser (honest absence). The family page explains: "Context is the per-task authorized input view. Open a task to inspect its Context view, selected sources, and explicit losses." + link to Work. Where `/task/resource/v1/consumption` pins exist for a task, they render in that task's Context section (`15` §6).

## 6. Shared states & rules

| State | Rule |
|---|---|
| Empty family | how objects arrive (import/remember/register) + one entry action |
| Envelope-only facets | labeled (id-only, limit 64) — never padded with fake columns |
| `not-backed` projection families | S7 + named dependency; authority-backed reads linked where they exist |
| Denied/disconnected/stale | per the route-state matrix (`06` §5) |
| Actions | class-A confirm with exact IDs/digests; class-C verbs (e.g. memory edit) as text + CLI path |

---

*This space is the Power User mode's home. Its hierarchy discipline (families → objects → provenance) is what keeps "Resources" from collapsing into the shipped family-picker + JSON dump.*
