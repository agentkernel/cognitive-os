# ADR-0034: Personal MVP-First Single-Service Release Train

- Status: Accepted
- Date: 2026-07-29
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal product planning and distribution implementation
  decision. This ADR does not add or change a registry requirement, schema,
  transition, conformance vector, Profile, or executed product Gate.
- Related: ADR-0018, ADR-0025, ADR-0026, ADR-0028 through ADR-0033,
  P1-T08, P1-T09, P2-T01 through P2-T08, P7-T01 through P7-T08.

## Context

The Personal implementation has established offline product-owned bundle
verification, bounded safe extraction, a cross-process installation lease,
atomic version publication, a bounded daemon liveness contract, and local
systemd-controller fixtures. It has not yet established a complete production
path from the rendered bootstrap script through native user-systemd, XDG
discovery, Provider selection, Pi loading, and the first response.

ADR-0032 and ADR-0033 selected a candidate/active two-unit promotion model.
That model is suitable for a later low-downtime upgrade train, but it makes a
second unit, a second port, candidate runtime isolation, and a larger
compensation state machine prerequisites for the first install-to-conversation
campaign. The first product campaign does not yet have measured upgrade
downtime or failure data that justifies those prerequisites.

The previous Personal critical path also placed Context efficiency, Memory,
the Agent/Tool ecosystem, and Multi-Agent work ahead of the first public
Linux product. That ordering delays validation of the smaller governed
single-Agent product.

## Decision

### 1. First production installation path

1. The first production path owns exactly one canonical user unit,
   `cognitiveos-personal.service`, and one canonical loopback liveness address,
   `127.0.0.1:48181`.
2. The MVP accepts a bounded service interruption during explicit upgrade.
   Zero-downtime promotion is not an Alpha or B01 requirement.
3. The deterministic transaction is: complete offline verification; acquire
   the stable per-root OS lease; safely extract into private staging; publish
   an immutable version; atomically publish the product-rendered canonical
   unit; run bounded fixed user-systemd actions; restart and probe the service;
   atomically publish and re-read the active pointer; confirm pointer, rendered
   unit, process and liveness consistency; then issue a non-secret receipt.
4. Upgrade failure restores the prior complete unit, pointer and service and
   confirms prior liveness. First-install failure stops any newly started
   service, leaves no active pointer, retains user data and issues no receipt.
   Any incomplete required compensation returns `RollbackIncomplete`.
5. Rust is the sole owner of verification, staging, unit rendering, service
   orchestration, compensation and receipt creation. The inspectable shell
   bootstrap may perform bounded downloads and authenticate the Rust adapter;
   it does not implement deployment mutation or service logic.

### 2. Relationship to ADR-0032 and ADR-0033

This ADR partially supersedes ADR-0032 and ADR-0033 only for the first
production Alpha/B01 path. Their product-owned candidate identity,
`127.0.0.1:48182`, staged executable and two-unit promotion sequence remain a
documented optional upgrade design. They must not be the default production
path until an executed single-service upgrade campaign demonstrates a need
for lower downtime and the candidate runtime/authority boundary is reviewed.

The following ADR-0032/0033 constraints remain binding for every path:

- product-owned fixed unit names, executable layout, ports and arguments;
- private synchronized atomic unit publication and unsafe-path rejection;
- `systemctl --user` only, bounded actions, kill/reap and output caps;
- `/personal/health` is liveness, not readiness or release acceptance;
- deterministic compensation, user-data retention and no success receipt on
  incomplete confirmation or rollback.

### 3. MVP-first release trains

The existing task IDs remain stable. They are reorganized into release trains:

1. Install-to-Conversation Alpha: P1-T08 and P1-T09/B01.
2. Governed Single-Agent MVP: P2-T01 through P2-T08.
3. Public Linux MVP: P7-T01, P7-T02 and P7-T03 converge at P7-T08 / the
   product-only `GMVP-LINUX` Gate.
4. Context efficiency and durable Memory follow a reliable task baseline.
5. Agent/Tool ecosystem, Windows install parity and Web UI are independent
   claim-scoped capability trains.
6. Multi-Agent is an optional go/no-go experiment. A measured no-go that keeps
   it disabled is a valid outcome and does not block Linux MVP or full RC.

`GMVP-LINUX` is a Personal product Gate, not a registry ID or Profile. It does
not claim Core Profile implementation.

### 4. B01 evidence layers

Development smoke, usability learning and the formal B01 campaign are distinct:

- development smoke uses real product binaries and contracts but remains a
  non-claim integration result;
- usability runs measure operator friction and recovery but do not pass B01;
- only the predeclared clean Linux VM campaign can pass B01.

WSL, fake-systemctl, ordinary CI and local fixtures cannot substitute for
Linux-native user-systemd or B01 evidence.

## Consequences

- P1-T08 remains `in-progress` and P1-T09 remains `not-started` when this
  decision is accepted.
- The single-service path reduces the first production state space while
  retaining the trust, secret, authority, extraction, lease, rollback and
  verification boundaries.
- The installer must explicitly connect XDG data ownership, daemon endpoint
  publication, Provider discovery, selected-model persistence and Pi startup;
  liveness alone cannot establish first-conversation readiness.
- Memory, MCP, Multi-Agent, Web UI and Windows installer parity do not block
  the scoped public Linux MVP.

## Non-claims

This decision is planning and architecture evidence only. It does not provide
the single-service implementation, execute native systemd, create production
signing material, pass B01 or `GMVP-LINUX`, establish containment, or change
Profile `implemented` status.
