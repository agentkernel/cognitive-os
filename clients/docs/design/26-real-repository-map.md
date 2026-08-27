# 26 — Real Repository Map (Two-Repository Architecture)

- Phase 2.5 (audit / mapping / planning only — no implementation)
- Date: 2026-08-24
- Audit basis: `D:\agent-kernel` working tree at/around `main` `aeb9c3a9` (PR #271); `D:\cognitiveos-clients` worktree `personal/P7-T05-dsh-binding-cas` @ `0320c1a`, `main` @ `db56374`. All facts below were verified against source in Phase-1/2.5 audits; documentation-only claims are labeled as such.

## Current implementation (frozen audit baseline)

The accepted current implementation is integrated at `clients/pc/web/` and uses
Home / Work / Agents / Providers / Resources / Activity / System. The body is
preserved as P7-T05 evidence at the exact earlier repositories/revisions above.
It must not be silently updated to describe the later ADR-0054 layout; its
two-repository statements are historical observations, not instructions.

## Personal 2.0 target delta

The target design now lives in `01`–`25` and uses
Home / Agents / Work / Library / Activity / Settings, a global Agent Shell,
Adapter-backed conversation, first-class MCP, Account Hub, and provenance-aware
Activity. This map proves only the P7-T05 SPA/daemon seam; it does not prove
those target capabilities. Use [Recommended IA](06-control-plane-recommended-ia.md)
and [Design Decisions](10-control-plane-design-decisions.md) for the superseding
target, and keep this document as implementation history.

---

## 1. Repository identity

| | Repository A | Repository B |
|---|---|---|
| Repo | `agentkernel/cognitive-os` (local: `D:\agent-kernel`) | `agentkernel/cognitiveos-clients` (local: `D:\cognitiveos-clients`) |
| Owns | CognitiveOS architecture/contract layer + **all** CognitiveOS Personal implementation (Rust daemon, CLI, contracts, conformance, docs) | All client implementations (PC web SPA; future console/mobile/agent-hub) |
| Contains the Web UI? | **No** — serves the built bundle statically at `/ui/`; `apps/cognitiveos-console` is a deprecated compatibility stub (ADR-0053 forbids SPA work there) | **Yes** — `pc/web/` is the Control Plane SPA |
| Governance | AGENTS.md + docs/governance + PERSONAL-DEVELOPMENT-PLAN | own governance; kernel ADR-0053 binds the SPA stack/serving |

## 2. Repository A map (cognitive-os)

| Path | What it is | Control Plane relevance |
|---|---|---|
| `crates/cognitive-contracts` | public contract types (Rust) | DTO source of truth |
| `crates/cognitive-domain` | domain model | entity semantics |
| `crates/cognitive-store` | SQLite authority persistence (task/effect/event/installation/provider…) | what is *true* |
| `crates/cognitive-kernel` | tool registry, kernel primitives | tool catalog |
| `crates/cognitive-runtime` | installer, agent adapter manifests, sidecar sessions, pi runtime | agent lifecycle (library/CLI-only) |
| `crates/cognitive-management` | privileged management session model (file/SQLite-based, not HTTP) | CLI admin boundary |
| `crates/cognitive-akp` | agent kernel protocol (adapter boundary) | dsh/Pi adapters |
| `apps/kernel-server` | the Personal daemon: `src/personal/server.rs` front door + routers; `task_api.rs`, `resource_api.rs`, `resource_manager.rs`, `provider_control_plane.rs`, `tool_lifecycle.rs`, `observation.rs`, `readiness.rs`, `user_backup.rs`, `auth.rs`, `bounds.rs` | **the entire HTTP API the SPA may use** |
| `apps/admin-cli` | `cognitive` CLI (`personal_cli/`) + privileged store-direct admin verbs | operator fallback; lifecycle verbs live here |
| `apps/agent-shell` | agent shell client (Pi-hosted experience) | sibling client, not the Control Plane |
| `packages/contracts-ts` | TS contract bindings | potential typed-client source (audited in `27`/`28`) |
| `packages/sdk-ts` | TS SDK | same |
| `specs/` | normative schemas/registry/transitions | contract governance (Lane-CTR) |
| `conformance/`, `tests/` | conformance vectors, golden tests | validation |
| `docs/product/personal/` | canonical product design (incl. `web-ui-design.md`) | product contract |
| `docs/architecture/personal/` | architecture (incl. `web-ui-architecture.md`, `web-ui-route-inventory.json`) | route inventory = frozen UI↔API map |
| `docs/design/` | **this design workspace** (Phase 1/2/2.5 docs) | design contract |
| `handbook/` | bilingual operator/developer docs | capability-status reference |

## 3. Repository B map (cognitiveos-clients)

| Path | What it is | Relevance |
|---|---|---|
| `pc/web/` | **The Control Plane SPA** (React 18.3.1 + TS 5.6.3 + Vite 5.4.11; HashRouter; single-file `src/App.tsx` ~1485 lines + 9 logic modules; hand-rolled CSS 155 lines; Vitest) | the redesign target |
| `pc/app/` | reserved Console root — empty/blocked with 4 NO-GO conditions | not the Web UI |
| `pc/docs`, `pc/plan` | future Console docs (predate the SPA) | background only |
| `shared/` | shared client code (audited in `27`) | potential typed-client home |
| `mobile/` | mobile client area | out of scope |
| `agent-hub/` | separate product docs (AGPL-gated) | out of scope |

## 4. Ownership matrix (who owns what)

| Concern | Owner | Evidence |
|---|---|---|
| Product authority (all writes) | A — Rust daemon only (A1) | `server.rs` routers; `product-design.md:17-21` |
| HTTP API | A — `apps/kernel-server/src/personal/` | route inventory (`28`) |
| Contracts (public DTOs) | A — `crates/cognitive-contracts`, `packages/contracts-ts`, `specs/` | Lane-CTR process |
| Runtime state | A — `crates/cognitive-store` (SQLite), in-process session map | store audit (`30`/`31`) |
| UI | B — `pc/web/` | ADR-0053 |
| Client state | B — `pc/web/src/session.ts` (memory-only bearers), per-page `useState` | WebUI audit |
| Presentation | B | — |
| Session issuance | A (`POST /local/session`); B holds tokens in memory only | ADR-0053 §3 |
| Provider interaction (egress) | A only (`/provider/v1/*` proxy; SecretStore daemon-side) | `provider_control_plane.rs` |
| Task interaction (record/interpret/preview/admit/watch) | A serves; B consumes | `task_api.rs` |
| Agent observation | A serves (dsh snapshot; runtime inventory); B renders | `task_api.rs:766-803` |
| Static bundle serving | A (`GET /ui/` from `data_dir()/ui`, 503 when absent) | `server.rs:2943-2989` |
| Bundle build/deploy | B builds `dist/`; release process copies into daemon data dir | `pc/web/README.md` |

## 5. The delivery seam (how B reaches A)

```text
cognitiveos-clients/pc/web  --pnpm build-->  dist/  --copy-->  <daemon data_dir>/ui/
                                                                  │
agentkernel/cognitive-os daemon serves GET /ui/*  <---------------┘
        (same-origin; CSP default-src 'self'; no CORS; no cookies)
```

- No runtime coupling: the SPA is a static artifact; the daemon treats it as untrusted static content.
- No cross-repo type sharing today: `pc/web` hand-rolls `fetch` + untyped coercion; `packages/contracts-ts`/`sdk-ts` exist in A but are **not consumed** by `pc/web` (verified in WebUI audit; TS-layer audit in progress — see `27` §6 for the final answer).
- Version alignment is by pinned revision + bundle copy; there is no API version negotiation beyond schema_version fields in envelopes.

## 6. Governance facts that constrain implementation

1. Client repo write access for automation is **currently blocked** (cursor[bot] HTTP 403; P7-T05/D10 remediation pending with the owner). Any Phase-3 implementation must plan around this (owner-run publication or fixed access).
2. Kernel repo changes require PR + required CI (Ubuntu + Windows); Rust builds/tests are not runnable on this Windows GNU host (`RUST-LINK-DEV-WIN-GNU-01`) — validation routes through CI or exact-revision Linux.
3. Contract changes (new DTOs/routes) go through Lane-CTR (schema/registry/bindings/vectors together) — this is what makes BD-* items "backend work", not "add a route".
4. `apps/cognitiveos-console` must not be revived; `clients/**` must not be recreated in A (ADR-0053).
5. The dirty worktree files present before this phase (`.cursor/skills/*`, `tmp-*.sh`, `.cursor/environment.json`) are untouched by this audit.

---

*Next: `27-real-webui-architecture.md` (B's SPA as-built), `28-real-api-contract-map.md` (A's HTTP surface as-built).*
