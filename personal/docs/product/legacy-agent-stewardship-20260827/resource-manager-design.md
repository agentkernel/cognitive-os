# Resource Manager — product design

- Status: current six-family envelope plus adopted Personal 2.0 direction
- Change class: product-semantic companion; no new public contract
- Architecture pair:
  [resource-manager-architecture.md](../../architecture/resource-manager-architecture.md)
- Current-status owner: [PROGRESS.md](../../../../docs/plan/PROGRESS.md)
- Current task evidence owner:
  [PERSONAL-DEVELOPMENT-PLAN.md](../../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
  `P8-T12`

The Resource Manager gives clients one versioned way to perform shared
inspection and governance operations without collapsing family semantics. Each
mutation resolves to a typed family workflow. The Rust daemon remains the sole
authority writer.

## 1. Reality ledger

| Boundary | Resource Manager truth |
|---|---|
| **Current implementation (Now)** | A private six-family projection and management envelope support bounded list, inspect, watch, bind, unbind, enable, disable, and revoke where a family defines the operation. Context and Runtime inventory can be empty/projection-only. |
| **Adopted Personal 2.0 target** | Seven-family navigation and federation: vendor-native resources are adapter-mapped, origin-owned, and governed/bound by Personal. MCP joins as a family through its own lifecycle. |
| **Requires-backend** | Authority-backed Context/Runtime inventory, general native-resource change detection, bidirectional synchronization, conflict handling, and MCP management are absent today. |
| **Requires-core (conditional)** | P10-T02/Lane-CTR is required only if MCP or federation adds/changes a public machine surface. A Personal-private projection may not require core changes. |

## 2. Why the common envelope exists

Memory, Skill, Tool, Context, Task, Runtime/Process, and target MCP have
different identities, storage, transitions, and retention. Clients still need
a predictable read envelope and a small common governance vocabulary.

The envelope is a projection, not a universal resource object. It cannot be
written back as a generic aggregate, and it never authorizes a family-specific
operation merely because two families use the same verb.

## 3. Current common operations

| Operation | Operator meaning | Not this operation |
|---|---|---|
| `list` | bounded family page at a declared projection version | full-table dump, content search, or ranking |
| `inspect` | one stable identity and current projected version | generic edit form |
| `watch` | resume the existing family watch cursor | a second or fabricated live stream |
| `bind` | typed relationship under expected-version guards | generic create |
| `unbind` | remove a typed relationship under guards | delete or purge domain history |
| `enable` | admit eligibility according to family semantics | execute |
| `disable` | stop new use without fabricating completion | uninstall |
| `revoke` | invalidate a grant, binding, or usable revision | Memory forget or source deletion |

Generic `create`, `install`, `execute`, and `complete` remain refused by the
common envelope. Acquisition, import, admission, execution, reconciliation,
retention, update, uninstall, and purge stay typed family or Task workflows.
MCP server install/update is therefore not created by reusing a generic
Resource Manager verb.

## 4. Read projection and inspector

Current list/inspect projections expose only available authority facts, such as:

- stable identity and family;
- origin/authority source;
- revision digest or explicit absence;
- owner and scope;
- health or availability;
- typed bindings;
- blocked reason;
- currently allowed common actions;
- object and projection versions.

Unknown, unavailable, not-backed, and stale remain explicit. The UI must not
turn an empty Context or Runtime projection into a claimed inventory.

Personal 2.0 adds target inspector concepts—native origin, adapter,
capability mapping, sync freshness, conflict, and projected clients—but those
facts are shown only when a backend projection exists.

## 5. Federated-resource behavior

### Adopted Personal 2.0 target

1. A vendor adapter maps native resources into the appropriate Personal family
   without copying authority from the Agent.
2. The origin side owns native content and native lifecycle.
3. Personal owns admitted governance, bindings, permission, sync intent, and
   authority receipts.
4. Agent connection establishes an explicit observation scope. Authorized read
   and change detection may be automatic only inside it; there is no
   speculative/global scan or surprise per-session enrollment.
5. Every Personal-to-native write-back uses daemon-owned Intent/Effect,
   dispatch, and reconciliation. It may run automatically inside an unchanged
   exact daemon grant/risk policy; new, broader, destructive, or conflicted
   scope requires preview and confirmation.
6. Conflicts fail closed. The global Agent Shell explains the conflict and
   requests a daemon-backed family-specific resolution.
7. Bidirectional synchronization never means last-writer-wins by default and
   never auto-promotes Native or Observed content into Governed state.

This behavior is **Requires-backend**. A shared public synchronization contract
conditionally requires P10-T02/Lane-CTR; a Personal-private projection may not.

## 6. MCP relationship

MCP is the adopted seventh family, not a Tool alias and not an Agent. The
Resource Manager may eventually list and inspect an MCP family projection, but
server installation, health, permission, update, and client projection remain
family-specific workflows.

P5-T03/P5-T04's current MCP Tool transport and bounded dynamic-Tool path remain
Tool-family implementation. They are not an MCP-family projection or lifecycle.

An MCP server can expose candidate capabilities. Those capabilities become
eligible Tool or Context inputs only through separate mapping and
authorization. Connection or client configuration grants no host-session
control.

MCP implementation remains **Requires-backend**. Only a new or changed public
MCP machine surface conditionally requires P10-T02/Lane-CTR; a
Personal-private projection may not. See [MCP resource family](mcp-resource-family.md).

## 7. Product navigation

### Current implementation (Now)

The current `/ui/` Resources hub covers Memory, Skills, Tools, and a Context
link into Work. Runtime is shown in Agents.

### Adopted Personal 2.0 target

**Library** contains Memory, Skills, Tools, and MCP. **Work** owns Context and
Task. **Agents** owns Runtime/Process. Navigation placement does not alter
family ownership or the common envelope.

## 8. Channels and fixed boundaries

- Current Resource Manager mutations use the management channel; Task-channel
  misuse fails closed.
- The deterministic CLI remains a client of daemon authority.
- Watch is not duplicated, and partial watch coverage is never presented as a
  unified live feed.
- The global Agent Shell may request list/inspect and propose actions but cannot
  write the envelope or exercise authority.

This design creates no public contract, universal `Resource` table, Gate,
release, Profile, performance, containment, or Agent-benefit claim.
