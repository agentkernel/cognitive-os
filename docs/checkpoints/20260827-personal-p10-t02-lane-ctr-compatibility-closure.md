# P10-T02 Lane-CTR contract and compatibility closure

- Task: `P10-T02`
- Current slice: `P10-T02/D03`
- Change class: `normative-semantic Lane-CTR compatibility` (no public machine-contract change)
- Branch: `personal/P10-T02-lane-ctr-compatibility`
- Content revision (final head before this record): `c726ab11a789780462d8d0ef1161b99669518c92`
- Pull request: [#279](https://github.com/agentkernel/cognitive-os/pull/279) (Draft at this record)
- Status at this checkpoint: **acceptance recorded** — required CI green on the content head; merge, lease close, and branch deletion follow in the same task delivery

## Outcome

[ADR-0058](../adr/0058-personal-2-0-mcp-conversation-private-projection.md) freezes the Personal 2.0 public/private boundary:

1. MCP family identities and the common conversation/history projection stay Personal-private versioned envelopes (`cognitiveos.personal.mcp-family/0.1`, `cognitiveos.personal.conversation-projection/0.1`).
2. This batch adds or changes **no** Core public schema, transition, registered error, or generated binding.
3. The 1.0 six-family projection and Core `ConversationBinding` remain fail-closed for older clients.
4. P5-T03/P5-T04 Tool-transport records do not auto-migrate into seventh-family identities.

No MCP family store, conversation store, HTTP route, or UI was implemented. That work remains `P10-T03` / `P10-T05`.

## Acceptance mapping

| Acceptance item | Delivered evidence |
|---|---|
| Public vs private contract boundary | ADR-0058 §1–§2: Personal-private envelopes; later public surface requires a new Lane-CTR batch with schema, bindings, errors, transitions, and negatives together |
| Identity / version compatibility | Envelope identifiers and `0.1` private revision rule; unknown envelope id fail-closed; no silent coercion of older clients |
| Capability digest | SHA-256 over RFC 8785 canonical JSON (`cognitiveos.canonical-json/0.1`); drift ≠ auto-enable |
| Binding | Distinct MCP binding identity; does not grant Tool/Context/Skill authority; vendor conversation IDs stay opaque origin bindings on Core `ConversationBinding` |
| Health / quarantine | Distinct private identities; health is not enablement; quarantine isolates the MCP family object without silently revoking unrelated admitted Tool/Context/Skill objects |
| P5-era migration | ADR-0058 §7 plus focused negative: P5 adapter remains transport-only; no in-place rewrite into family identities |
| Older-client fail-closed | 1.0 `family=` stays `memory\|skill\|tool\|context\|task\|runtime`; `mcp` → `RESOURCE_PROJECTION_FAMILY_INVALID`; `ConversationBinding` `additionalProperties: false` |
| Schema / generated bindings / negatives only if machine contract changes | No Core schema change, therefore no generated-binding batch; focused absence/allowlist negatives in `tools/test/p10_t02_lane_ctr.test.mjs` |
| Product / architecture / handbook / plan / trace sync | Canonical product and architecture indexes, bilingual MCP/conversation pages, support matrix, ADR-0057 backlink, handbook source-map + bilingual pages + fingerprints, `PERS-PR-038` |

## Validation

| Unit | Result |
|---|---|
| `pnpm run check:consistency` | **pass** |
| `pnpm run check:handbook` | **pass** — 58 documents × 2 locales, 9 generated |
| `node tools/src/generate-handbook.mjs --check` | **pass** — 18 generated pages byte-identical |
| `node tools/src/docs-sync-gate.mjs --staged` | **pass** |
| `git diff --cached --check` | **pass** |
| `pnpm --filter @cognitiveos/repo-tools test` | **pass** — 106/106, including five P10-T02 cases |
| Required CI [33048210670](https://github.com/agentkernel/cognitive-os/actions/runs/33048210670) on content revision `c726ab11` | **pass** — resolver 3s, Ubuntu 3m32s, Windows 11m33s, `required-ci` |

Rust linking was **not-run** on `DEV-WIN-GNU-01` (`RUST-LINK-DEV-WIN-GNU-01`). This batch did not change schema, so Lane-CTR generated-binding CI was not required beyond the ordinary Ubuntu/Windows verify jobs.

## Explicit non-claims

- no MCP family store, route, UI, conversation store, or credential-import implementation;
- no Core schema, transition, registered error, generated binding, or public compatibility promise;
- no change to Linux/Personal 1.0, its six-family manifest, Pi qualification, support matrix claims, or Gate composition;
- no auto-migration of P5-T03/P5-T04 records into seventh-family identities;
- no Goal / Plan revision / attempt / handoff contract freeze (those remain `P10-T06` / `P10-T13`);
- no Gate, release, Profile, B01, performance, containment, Provider-quality, marketplace-safety, or Agent-benefit promotion.

`P10-T02` done does not mean MCP family authority is implemented. That is `P10-T03`.
