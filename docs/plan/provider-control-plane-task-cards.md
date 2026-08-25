# Provider Control Plane Task Cards

## Planning status and mapping rule

These are design-package cards, not formal Personal task records. `PCP-T01`
through `PCP-T07` are temporary identifiers and must be mapped into existing
`P*-T*` work before implementation. They do not change status, create a parallel
lease, or bypass Lane-CTR. Reuse baseline: P1-T02 Secret Store/provider
config; P1-T03 discovery/snapshot; P1-T04 daemon/auth; P1-T06 CLI; P1-T07 Pi
proxy; existing P1/P2 application and migration contracts.

## Shared delivery contract

After formal mapping, each card uses one task branch, Draft PR, and lease;
keeps implementation/acceptance/promotion dependencies distinct; writes
failure-first tests; preserves Secret Store, Intent/Effect, audit, redaction,
and verifier boundaries; and routes Rust validation to supported CI/Linux (not
Windows GNU). Local fixtures/ordinary CI cannot establish Gate, release,
Profile, B01, provider-quality, or agent-benefit claims.

## PCP-T01: account, endpoint, and secret contract

**Requires:** P1-T02, P1-T04, management auth. **Proposed paths:**
`personal/crates/cognitive-secret/**`, `personal/apps/kernel-server/src/personal/**`, focused
tests, and formally mapped ADR/trace.

**Acceptance:** multiple named accounts and opaque refs; no key in DB/config/
argv/env/log/CLI/audit/fixtures/evidence; active bindings block deletion;
HTTP/private trust is explicit and renewed on authority/scheme change; userinfo,
redirects, arbitrary headers, unsupported protocol, DNS rebinding, and
unauthorized targets fail before key disclosure; mutations have Intent/Effect/
verification and redacted audit; discovery failure yields a degraded but
preserved account, while a missing key yields a non-callable revoked account.

**Negatives/validation:** unauthorized loopback/private target, insecure HTTP,
embedded credentials, redirect, bearer override, stale ref, concurrent
rotate/delete; supported Rust tests and static redaction scan.

## PCP-T02: model discovery and transport

**Requires:** PCP-T01, P1-T03, approved HTTP/TLS dependency decision.
**Proposed paths:** `personal/crates/cognitive-secret/**`, daemon provider routes,
transport tests, dependency manifest, mapped ADR.

**Acceptance:** creation performs one foreground discovery; compatible path is
bounded `GET /v1/models`; refresh is explicit; failure preserves catalog and
binding; manual models are labelled; Bearer-only custom transport and fixed
official auth; redirects, caller paths/headers, and Anthropic-compatible custom
endpoints fail closed.

**Negatives/validation:** 401/403/404/429/5xx, timeout, malformed/oversized
JSON, redirect, wrong protocol, DNS target change, missing price; mock/loopback
tests locally and supported Rust tests on CI.

## PCP-T03: Pi fixed binding and proxy

**Requires:** PCP-T02, P1-T07, current Pi pin and sidecar contracts.
**Proposed paths:** `personal/apps/kernel-server/src/personal/**`,
`personal/packages/pi-cognitiveos/**`, integration tests, trace.

**Acceptance:** no Pi Secret Store/key env/auth file/SQLite access; one binding
revision/endpoint/model; missing/expired/revoked/mismatched auth fails closed;
provider failure is audited without fallback; usage is exactly-once across
reconcile/retry. Test direct provider access, changed target, expired session,
revoked key, proxy failure, malformed usage, duplicate event. Use Pi fixture
tests and designated Linux-native smoke when available; fixture is non-claim.

## PCP-T04: DeepSeek harness adapter

**Requires:** PCP-T02 and verified installed harness launch/config surface.
Implement an independent adapter, not assumed Pi parity. Acceptance is no
durable key exposure, fixed target, rotation/revocation enforcement, auditable
failure, and no fallback. If the harness cannot consume a scoped daemon
proxy/session without becoming key authority, record `blocked` with exact path
and owner decision; never weaken Secret Store boundaries.

## PCP-T05: usage, retention, pricing

**Requires:** PCP-T02 and approved migration/application-service path.
**Proposed paths:** approved store migration, daemon usage/pricing services,
query projection, focused tests; no second DB.

**Acceptance:** four nullable token categories; distinct reported/estimated/
unavailable sources with estimation method; queries filter by time range,
account, provider, model, agent, and outcome; cache-hit rate is calculated only
with a known provider denominator; 30-day events and 90-day
aggregates; idempotent cleanup; per-event price version; historical cost
stability; no prompt/completion/key/header/payload retention. Test provider
fixtures, cache categories, missing fields, retention boundary, duplicate,
cleanup retry, and later price update.

## PCP-T06: budgets, alerts, audit, CLI

**Requires:** PCP-T05, P1-T04, P1-T06, existing API envelope.

**Acceptance:** owner-authenticated account/agent monthly token/cost budgets;
deduplicated 80%/100% period alerts; `cost_unavailable` not zero; alerts never
block/reroute; CLI never opens SQLite/resolves secrets; audit exposes redacted
endpoint, outcome, actor/session class, and correlation ID only. Test wrong
channel, unauthenticated caller, duplicate threshold, period rollover, unknown
cost, and alert retry.

## PCP-T07: integrated validation and closure

Requires all accepted cards and exact pushed revision. Validate official
OpenAI, official Anthropic, compatible endpoint, manual model, key rotation/
removal, trust changes, both agent paths, usage categories, retention,
cost-unavailable, and 80%/100% alerts. Complete supported validation, final
acceptance mapping, docs-sync, and one final handoff. Non-claims remain UI,
OAuth, routing, hard limits, provider quality, agent benefit, Gate, release,
Profile, and B01 unless separately qualified.

## Dependency graph

```text
PCP-T01 -> PCP-T02 -> PCP-T03 -> PCP-T07
                    \-> PCP-T04 -/
             PCP-T02 -> PCP-T05 -> PCP-T06 -/
```

PCP-T03 and PCP-T04 are independent qualification paths. PCP-T05 can begin
after normalized usage is stable, but its acceptance needs real callers or an
explicit blocked path. Formal mapping, lease ownership, writable paths, and
current status must be resolved before implementation starts.
