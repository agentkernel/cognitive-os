# P10-T01 Personal 2.0 desktop and MCP semantics closure

- Task: `P10-T01`
- Current slice: `P10-T01/D03`
- Change class: `product-semantic + structural documentation`
- Branch: `personal/P10-T01-desktop-mcp-semantics`
- Content revision (final head): `a52815b9004c40d920696e7fd287962d7f2f7e77`
- Merge revision: `main@aa7ab42ebc84b59e3b3ab2e3278cf1c11dd6fd25`
- Pull request: [#278](https://github.com/agentkernel/cognitive-os/pull/278) (merged 2026-08-27)
- Status at this checkpoint: **closed** — merged, lease closed, task and
  remote branches deleted, local `main` fast-forwarded

## Outcome

The documentation delivery adopts two bounded Personal 2.0 product decisions:

1. [ADR-0056](../adr/0056-personal-2-0-desktop-control-plane.md) makes the
   desktop Control Plane the primary Personal 2.0 entry and supervision
   surface, with Home / Agents / Work / Library / Activity / Settings,
   vendor-specific conversation adapters, and a candidate-only global Agent
   Shell.
2. [ADR-0057](../adr/0057-personal-2-0-mcp-resource-family.md) makes MCP the
   seventh Personal 2.0 product family while preserving the finalized
   six-family Linux/Personal 1.0 boundary and deferring public compatibility
   semantics to `P10-T02`.

No implementation or public machine contract changed.

## Acceptance mapping

| Acceptance item | Delivered evidence |
|---|---|
| Adopt desktop-first Personal 2.0 semantics | ADR-0056, updated formal plan/trace/support sources, and canonical product/architecture indexes |
| Adopt MCP seventh-family product semantics without retroactive support claims | ADR-0057, bounded ADR-0037 supersession, MCP product pair, resource model, and architecture alignment |
| Separate current implementation from target capability | Canonical product and architecture documents use Current / Personal 2.0 target / Requires-backend / conditional Requires-core boundaries |
| Define beginner-first desktop IA and journeys | Product design, Web UI design, user journeys, Account Hub pair, Agent conversation pair, and all 44 client design documents |
| Preserve daemon authority and honest completion | Native / Observed / Governed / Verified provenance, daemon-only admission, Intent/Effect write-back, and independent-verification completion rules |
| Define Agent conversation and multi-Agent boundaries | Vendor-specific adapters behind a common private projection, explicitly authorized observation scope, Goal → Plan revision → Task → Attempt, and daemon-owned orchestration |
| Define Account Hub and credential-import target | Tiered providers, scoped model defaults, ADR-0055 per-source consent, daemon-only import, SecretStore custody, and `Requires-backend` labeling |
| Define MCP management and fallback boundaries | Server/package/connection/capability/binding/health/quarantine lifecycle; advertised objects remain Tool/Context/Skill candidates; MCP + rules is cooperative, not host-session control |
| Synchronize derived documentation | Bilingual handbook source routing, content, glossary, capability status, fingerprints, and byte-identical generated references |
| Correct dsh recovery guidance | Tracked stale-session diagnosis plus canonical environment, agent-rule, entry, and bilingual handbook guidance; post-restart recovery requires restarting `cognitive dsh web`, with no bearer extraction |
| Register future implementation work without starting it | `P10-T02` contract decision, `P10-T03` authority/runtime integration, and `P10-T04` Control Plane experience remain unclaimed and `not-started` |

## Validation

| Unit | Result |
|---|---|
| `pnpm run check:consistency` | **pass** |
| `pnpm run check:handbook` | **pass** — 58 documents × 2 locales, 9 generated |
| `node tools/src/generate-handbook.mjs --check` | **pass** — 18 generated pages byte-identical |
| `node tools/src/docs-sync-gate.mjs --staged` | **pass** |
| `git diff --cached --check` | **pass** |
| IDE documentation diagnostics | **pass** — no findings |
| Independent staged-document reviews | **pass after correction** — governance, product/architecture, complete client corpus, handbook/security |
| Required CI [33026909626](https://github.com/agentkernel/cognitive-os/actions/runs/33026909626) on content revision `bebe6680` | **pass** — resolver, Ubuntu, Windows, and `required-ci` |
| Required CI [33028175298](https://github.com/agentkernel/cognitive-os/actions/runs/33028175298) on final head `a52815b9` | **pass** — resolver 4s, Ubuntu 3m16s, Windows 19m5s, `required-ci` |

The CI annotations only report GitHub Actions' Node 20 action-runtime
deprecation; no repository check failed.

## Explicit non-claims

- no Control Plane, Account Hub, Agent adapter, Goal/Plan, federated sync,
  multi-Agent, MCP family, lifecycle-control, or credential-import
  implementation;
- no new route, DTO, schema, transition, registered error, conformance vector,
  or public compatibility guarantee;
- no change to Linux/Personal 1.0, its six-family manifest, Pi qualification,
  support matrix claims, or Gate composition;
- no Gate, release, Profile, B01, performance, containment, Provider-quality,
  marketplace-safety, or Agent-benefit promotion.

## Deterministic closure (completed)

1. acceptance record committed as `a52815b9` and pushed on 2026-08-27;
2. required CI [33028175298](https://github.com/agentkernel/cognitive-os/actions/runs/33028175298) on `a52815b9` passed resolver, Ubuntu, Windows, and `required-ci`;
3. PR [#278](https://github.com/agentkernel/cognitive-os/pull/278) marked ready and merged as `main@aa7ab42e` on 2026-08-27;
4. `P10-T01/D03` and `P10-T01` marked done; lease
   `lease/personal/P10-T01/desktop-mcp-semantics` moved to
   `PARALLEL-LANES.md` §3.1; local and remote task branches deleted;
   local `main` fast-forwarded to `aa7ab42e` and matches `origin/main`.
