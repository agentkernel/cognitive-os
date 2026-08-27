# ADR-0056: Personal 2.0 Desktop Control Plane

- Status: Accepted (owner-directed, 2026-08-27)
- Date: 2026-08-27
- Decision owner: CognitiveOS Personal product owner
- Change class: **product-semantic** (Personal 2.0 entry, navigation, and
  supervision semantics; no public machine contract or implementation change)
- Task anchor: `P10-T01`
- Executed under: `lease/personal/P10-T01/desktop-mcp-semantics`
- Related: ADR-0035, ADR-0037, ADR-0038, ADR-0043, ADR-0053, ADR-0054,
  [ADR-0055](0055-personal-credential-import-boundary-and-a5-revision.md),
  [ADR-0057](0057-personal-2-0-mcp-resource-family.md)

## Context

Personal 1.0 is finalized as a Linux x86_64, six-family product whose primary
conversational entry is the Pi-hosted Agent Shell. The post-1.0 Web UI and
Provider Control Plane established a daemon-served, loopback-only supervision
surface, but they did not settle the primary Personal 2.0 product entry, its
top-level information architecture, or how conversations from independently
qualified installed Agents should appear together.

The owner now wants Personal to operate as the desktop supervision center for
their local cognitive system without replacing the native applications of
installed Agents. A global conversational surface can reduce navigation and
coordination cost, but it cannot become an authority writer or convert fluent
Agent output into a committed fact. Vendor conversation models also differ
materially, so a common experience must preserve capability gaps instead of
inventing false parity.

## Decision

### 1. Desktop-first primary product entry

Personal 2.0 adopts the **desktop Control Plane** as its primary entry and
supervision center. It is the owner's place to understand system readiness,
installed Agents, governed work, cognitive-resource state, evidence, and
configuration, and to initiate daemon-governed actions.

This is a Personal 2.0 product direction, not a retroactive Personal 1.0
change. Linux/Personal 1.0 remains finalized with its Pi-hosted Shell,
deterministic CLI, six resource families, existing support matrix, and existing
Gate composition.

The Control Plane complements rather than replaces Agent-native applications.
An installed Agent's own CLI, desktop application, or Web UI remains usable
when independently supported by that vendor and qualified by its Personal
adapter path.

### 2. Target information architecture

The Personal 2.0 target top-level spaces are:

1. **Home** — readiness, attention, current work, health, and bounded alerts;
2. **Agents** — installed Agent identities, capabilities, bindings,
   conversations, lifecycle, and health;
3. **Work** — governed Tasks, runs, Context, Effects, verification, and
   acceptance;
4. **Library** — Personal cognitive-resource families and their bindings;
5. **Activity** — time-ordered observations, Effects, evidence, and audit
   coverage;
6. **Settings** — Providers, models, System stewardship, sessions, and product
   configuration.

Providers and System are target sections under **Settings**, not top-level
spaces. Settings, Providers, Model, Budget, Permission, Artifact, Evidence, and
Event are not additional resource families. `Work` is the Personal 2.0
navigation label for governed Task/run supervision; it does not create a new
Task or Run authority domain.

### 3. Global Agent Shell remains candidate-only

The desktop Control Plane includes a global Agent Shell that may:

- explain daemon projections and why an item is blocked or degraded;
- navigate, search, compare, and summarize Control Plane information;
- draft goals, configuration changes, and action previews;
- propose next actions for explicit daemon admission.

The Shell is a client and candidate producer only. It cannot write authority
state, read raw secrets, silently widen scope, dispatch an unadmitted external
mutation, accept a Task, or present Agent/Provider output as completion. Exact
identity, policy, preview, admission, capability, budget, fencing,
Intent/Effect, reconciliation, and independent-verification decisions remain
daemon-owned.

### 4. Installed-Agent conversations use adapter projections

Personal 2.0 may embed conversations from installed Agents in the Control
Plane through **vendor-specific adapters** behind a common internal
conversation projection and capability matrix.

The internal projection provides common reading and supervision concepts while
retaining vendor distinctions such as conversation/thread identity, message
roles, streaming, attachments, tool-call observations, approval semantics,
resume support, and available lifecycle actions. Every adapter declares what
it can actually observe or request. Missing, stale, unsupported, or
unqualified capabilities render honestly and fail closed; the Control Plane
does not synthesize unsupported parity.

A vendor conversation, Agent execution, OS process, Personal Task, Effect, and
verification report remain separate identities. Agent messages and tool-call
requests are candidates or observations until admitted through existing daemon
authority paths. The common projection is internal in this decision; any
public contract, compatibility version, or migration surface belongs to the
Lane-CTR decision in `P10-T02`.

### 5. Credential import inherits ADR-0055

Any Control Plane import experience follows ADR-0055 exactly:

- the user initiates each import and consents to each exact source;
- the source and target Secret Store are shown before the read;
- only the Rust daemon reads the source and writes the approved Secret Store;
- raw secret material never reaches the UI, Agent, sidecar, ordinary
  configuration, SQLite, logs, evidence, or chat;
- source retention is the default and secure deletion is an explicit
  per-import choice.

This ADR authorizes no import implementation. Every import affordance remains
marked `Requires-backend` until a future formal implementation task supplies
the daemon path and focused negatives required by ADR-0055.

### 6. Delivery boundary

`P10-T01` records and synchronizes this product decision. `P10-T02` owns the
Lane-CTR contract and compatibility decision for any public desktop,
conversation, or MCP surface. `P10-T04` owns the future desktop Control Plane
experience. Existing ADR-0053 loopback, same-origin, session, and browser
secret-isolation controls remain the minimum safety boundary unless a later
approved decision replaces them.

## Consequences

- Personal 2.0 design and planning use one desktop-primary entry and the six
  target top-level spaces above.
- The global Shell can reduce operator effort without becoming a second
  authority writer.
- Conversation UX can converge at the projection layer while every vendor
  adapter remains independently versioned, capability-declared, and
  qualified.
- Native Agent applications continue to be valid product companions; embedded
  conversations do not imply exclusive routing through the Control Plane.
- Existing browser UI, Provider, dsh, and adapter implementation evidence does
  not by itself establish the Personal 2.0 desktop target as supported.

## Rejected alternatives

1. **Keep the Agent Shell as the Personal 2.0 primary product surface.**
   Rejected because supervision, cross-Agent state, Library management,
   Activity, and System stewardship require a persistent visual center.
2. **Replace every native Agent application.** Rejected because vendor-native
   workflows remain useful and cannot be truthfully normalized into one
   capability set.
3. **Let the global Shell commit actions directly.** Rejected because it would
   violate daemon-only authority and the candidate-only probabilistic boundary.
4. **Expose one vendor-neutral conversation contract immediately.** Rejected
   until `P10-T02` resolves compatibility, capability negotiation, versioning,
   and migration without flattening vendor semantics.
5. **Show credential-import controls as available before a backend exists.**
   Rejected by ADR-0055's `Requires-backend` boundary.

## Non-goals and non-claims

This ADR implements no desktop shell, navigation, Agent conversation adapter,
credential import, backend route, contract, schema, transition, registered
error, or negative vector. It creates no Gate and makes no support, release,
Profile, benchmark, B01, Provider-quality, or Agent-benefit claim.
