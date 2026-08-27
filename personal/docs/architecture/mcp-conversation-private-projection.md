# Personal-private MCP family and conversation projection envelopes

- Status: Lane-CTR frozen Personal-private envelope (ADR-0058)
- Change class: informative architecture; not a Core public contract
- Decision:
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)
- Related:
  [ADR-0056](../../../docs/adr/0056-personal-2-0-desktop-control-plane.md),
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md),
  [MCP resource family](../product/mcp-resource-family.md),
  [Agent integration and conversations](../product/agent-integration-and-conversations.md)
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)

This page records the Personal-private envelope identifiers and fail-closed
rules that `P10-T03` and `P10-T05` must implement. It does not create a Core
schema, HTTP route, SQLite table, UI, Gate, or support claim.

## 1. Envelope identifiers

| Envelope | Identifier | First implementation task |
|---|---|---|
| MCP family | `cognitiveos.personal.mcp-family/0.1` | `P10-T03` |
| Common conversation/history | `cognitiveos.personal.conversation-projection/0.1` | `P10-T05` |

Any other identifier fails closed. These strings MUST NOT become Core schema
`$id` values and MUST NOT become a `family=` query on
`GET /resource/v1/projection`.

## 2. What stays public / 1.0

| Surface | Frozen behavior |
|---|---|
| Core `ConversationBinding` | Unchanged; `additionalProperties: false`; no vendor transcript fields |
| `GET /resource/v1/projection` family vocabulary | Exactly `memory\|skill\|tool\|context\|task\|runtime` |
| Unknown family, including `mcp` | `RESOURCE_PROJECTION_FAMILY_INVALID` |
| Unsupported projection version | `RESOURCE_PROJECTION_VERSION_UNSUPPORTED` |
| P5-T03/P5-T04 MCP Tool adapter | Tool-transport observation only; no auto-migration |

## 3. MCP family private facts

The envelope may project the seven distinct identities from ADR-0057: server,
package, connection, capability, binding, health, quarantine. Binding is not a
Tool/Context/Skill grant. Health is not enablement. Quarantine does not silently
revoke other families.

Capability digest: SHA-256 over RFC 8785 canonical JSON of the observed
advertised set (`cognitiveos.canonical-json/0.1`). Drift is not auto-enablement.

## 4. Conversation projection private facts

The envelope may reference Core `Conversation` / `ConversationBinding` when a
Personal Conversation exists. Vendor conversation/thread IDs remain opaque
origin bindings. Daemon-side redaction is required before persistence or HTTP.
History is observation only.

## 5. Older-client fail-closed

A client that does not understand an envelope identifier in §1 is refused that
envelope. Implementations must not down-convert it into a six-family row or a
Core `ConversationBinding` extension.

## 6. Non-claims

This page is not a public machine contract. Linux/Personal 1.0 remains
six-family. No Gate, release, Profile, B01, or Agent-benefit claim follows.
