# 20 — Settings / System Spec

- Status: adopted Personal 2.0 Settings/System target; current System spec retained
- Updated: 2026-08-27
- Contract: `06` §3.7, Flow 8 in `07`, capability domain D1/D7. Job: make a complex substrate legible — **the surface an operator reads when something is wrong**, so its design bar is clarity under stress, not feature breadth.

## Personal 2.0 Settings / System amendment

System is no longer a first-level destination. It is a high-stakes group under
**Settings**, alongside Account Hub, Appearance & Accessibility, and
Diagnostics.

```text
Settings
  Account Hub
  System
    Readiness
    Doctor
    Backup / restore
    Session
    About
  Appearance & accessibility
  Diagnostics
```

Global readiness remains visible from every route and deep-links to
Settings/System. The System content below—readiness, doctor, stewardship,
session, about—remains the current-backed core and keeps all P7-T05 honesty
notes.

### Settings behavior

- Groups are searchable by concrete labels; search never implies a backend
  global index.
- Forms preserve dirty input, validate near fields, expose save/current state,
  and separate destructive operations.
- Appearance covers system-following theme, density where supported, reduced
  motion/transparency and contrast preferences. Client-local preferences are
  labeled local; no server preference API is implied.
- Account Hub follows `17-control-plane-provider-spec.md`.
- Diagnostics expose redacted facts, source and age. Unsupported export or
  service-control actions are `Requires-backend`.

Daemon start/stop/restart, product upgrade/uninstall, session revoke,
unimplemented doctor probes, and any unsupported backup/restore extension
remain unavailable or `Requires-backend`. They must not appear as active
controls. The target placement changes IA, not the authority boundary.

---

## 1. Structure (secondary nav inside System)

```text
┌──────────────────────────────────────────────────────────────────┐
│ System                                                            │
│ ├ Readiness ── six components, overall, first_conversation_ready  │
│ ├ Doctor ────── facts + guidance per component; sub-sections      │
│ ├ Stewardship ─ backup / restore                                  │
│ ├ Session ───── principal, channels, expiry, re-auth, clear       │
│ └ About ──────── versions, digests, diagnostics bundle guidance   │
└───────────────────────────────────────────────────────────────────┘
```

## 2. Readiness

```text
┌──────────────────────────────────────────────────────────────────┐
│ Readiness                                        last check 12s ago│
│  overall: ◆ degraded — provider degraded                           │
│  first conversation: ■ not ready (provider)                        │
│                                                                   │
│  system    ● ready       XDG dirs ok                              │
│  database  ● ready       files present · integrity not-claimed    │
│  secret    ● ready       backend: secret-service · available      │
│  provider  ◆ degraded    catalog stale · secret resolvable        │
│  daemon    ● ready       listening 127.0.0.1:48181 · lock held    │
│  pi        ● ready       pinned 0.81.1 · extension present        │
│                                                                   │
│  static checks are not runtime readiness · no Gate/Profile claim  │
└───────────────────────────────────────────────────────────────────┘
```

- One row per component: category + word + one-line fact + "facts →" disclosure into doctor detail. Worst component named in the overall line.
- Standing captions (honesty furniture): "static check is not runtime ready"; `integrity_claim: not-claimed` rendered as text, not hidden; `first_conversation_ready` explained in one sentence.
- Unknown/not-probed facets render S7 — never omitted to keep the page green.

## 3. Doctor

- Per-component detail: facts list + guidance list (daemon-provided `facts[]`/`guidance[]`), each fact with `source` + `observed_at`.
- Sub-sections (six-resource / headless-vault / operability): rendered in their true placeholder state — "not probed over HTTP (BD register)" — with the topic list visible so the operator sees what *would* be covered. This is the designed treatment of PARTIAL capability, per the capability-honesty contract.
- Support posture: "doctor output is redacted facts and digests; it never contains secrets, raw prompts, or provider traffic" — one caption; bundle guidance points to `cognitive doctor --bundle` (CLI), no browser bundle download in wave 1 (browser-side export of even redacted bundles is a visual-phase/owner decision — recorded OQ-3 in `25`).

## 4. Stewardship (backup / restore)

- **Backup [A]:** preview-first — what is included (authority data, memory, skill registry, task/context metadata, runtime registrations, evidence) and what is **never** included (secrets; raw SQLite) stated before the action; result shows archive id, path, manifest digest, excluded-secret count (a number the owner wants to see be ≥ the expected count).
- **Restore [A, highest-friction surface in the product]:** archive id → preflight (digest/compatibility) → consequence copy naming the live-apply nature and the 409 failure classes (tampered / schema-incompatible / incomplete / partial-refused / daemon-lock) → explicit confirm. Restore never touches secrets; re-binding secrets after a machine move is explained as the follow-up step.
- Upgrade/uninstall: class-C — CLI/systemd-owned; rendered as guidance with exact verbs.

## 5. Session

- Current principal, channels held, idle/absolute expiry (live countdown), re-authenticate (inline gate), clear session (client-side; the daemon-side expiry model stated, BD-7).
- The bootstrap-secret field pattern is shared with the shell gate (non-echoing, memory-only, cleared on submit, "not a Provider key" copy).

## 6. About / diagnostics

- Product/daemon version facts, schema/surface versions, build digests where exposed; the claim-ceiling caption ("local facts; no Gate/release/Profile claim") — the product's epistemic signature, present here because this is where an operator copies diagnostics from.

## 7. Design-for-comprehension rules (the brief's §12 question)

1. **Vocabulary is the daemon's**, explained once in plain language per component — never translated into vague friendly words ("provider degraded — discovery failed (auth)" beats "connection issues").
2. **Hierarchy by failure:** the broken thing is visually first (attention ordering), healthy things compress to one line each.
3. **Every fact has a source and an age.** Complexity becomes legible when provenance is uniform.
4. **Recovery is a link, not a lecture:** each degraded/blocked row ends in exactly one next action.
5. **Calm under stress:** no red pages; S5 is a row treatment, not a surface treatment. The surface stays neutral so the failure stands out.

---

*System is the System Operator mode's home and the degraded-mode destination for every other space's failure links.*
