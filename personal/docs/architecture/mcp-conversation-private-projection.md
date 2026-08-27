# Personal-private MCP and conversation envelope boundary

- Status: retained ADR-0058 boundary with ADR-0059 partial supersession
- Decisions:
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)
  and [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- OPC conversation architecture:
  [Conversation/Memory/Vault](conversation-memory-vault.md)

## 1. Retained envelope and compatibility facts

| Envelope | Identifier | Current decision |
|---|---|---|
| MCP family | `cognitiveos.personal.mcp-family/0.1` | retained advanced private target |
| former common vendor conversation/history | `cognitiveos.personal.conversation-projection/0.1` | identifier retained; dsh Path B first-slice role superseded |

Both identifiers remain Personal-private, not Core schema IDs. Unknown
identifiers fail closed.

Core `ConversationBinding` remains unchanged with no vendor transcript fields.
The current 1.0 private family vocabulary remains exactly
`memory|skill|tool|context|task|runtime`; `mcp` is rejected with
`RESOURCE_PROJECTION_FAMILY_INVALID`, unsupported version remains
`RESOURCE_PROJECTION_VERSION_UNSUPPORTED`, and P5 MCP Tool records do not
auto-migrate.

## 2. MCP private facts retained

Server, package, connection, capability, binding, health, and quarantine remain
distinct. Binding is not Tool/Context/Skill admission; health is not
enablement; quarantine does not silently mutate other families. Capability
digest and drift rules from ADR-0058 remain.

MCP is deferred from the 2.0 OPC P0 path but not cancelled as an advanced
family decision.

## 3. Conversation partial supersession

ADR-0059 changes the product model from external/vendor conversation
aggregation to Personal-owned employee Conversations. DSH has no native UI or
conversation synchronization in the 2.0 product.

Therefore `conversation-projection/0.1` MUST NOT be silently interpreted as the
new archive/index/retrieval model. Before implementation, Phase 11 must choose:

1. a new Personal-private envelope version for the OPC Conversation archive
   and projection; or
2. a new Lane-CTR public contract decision if a public surface is justified.

The old dsh Path B transcript may remain historical observation input. It is
not the first canonical OPC conversation slice.

## 4. OPC constraints

Personal Conversation references Owner/Project/employee and may link Task/
Attempt/artifact/receipt identities. Archive content is redacted before
persistence/projection, scoped, provenance-bearing, and observation-only until
separate Context/Memory/Task admission. DSH/Pi receive bounded Context and no
raw archive or secret access.

## 5. Non-claims

This page changes no Core or private machine contract, route, table, error,
transition, vector, or implementation. No MCP/Conversation support, Gate,
release, Profile, B01, or Agent-benefit claim follows.
