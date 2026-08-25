# 18 — Resources Spec (Memory · Skills · Tools · Context)

- Phase 2 (design-only)
- Date: 2026-08-24
- Contract: `06` §3.5, `04` §4 (families differ; flattening is the failure), capability model D5. Anti-goal from the brief: **no resource card wall** — the hub is navigation, families have real hierarchy.

---

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
