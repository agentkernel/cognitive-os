# Personal unified cognitive-resource restructure handoff

- Date: 2026-08-02
- Lease: `lease/personal/P2-T02/unified-resource-baseline`
- Change class: owner-approved `product-semantic + structural documentation`
- Runtime/schema/codegen changes: none
- Commit/PR: none; the worktree remains uncommitted

## 1. Delivered scope

This documentation batch re-establishes CognitiveOS Personal as the local,
single-user unified cognitive-resource substrate for Agents. It replaces the
temporary coding-workspace-only framing and the earlier Linux 1.0 deferral of
all durable Memory/general Context with a bounded six-family release target:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process.

Budget, Permission, Model, Artifact, Intent/Effect, Evidence and Event remain
cross-cutting objects. Agent is the user-facing projection over Runtime
package, installation, registration, instance, sidecar, execution and process
facts. No universal Resource table, generic authority object, common lifecycle
or Process authority domain was introduced.

## 2. Decisions recorded

- ADR-0037 makes all six families part of the Linux 1.0 minimum real slice and
  partially supersedes ADR-0036 only where that ADR deferred all durable Memory
  and general Context.
- ADR-0038 establishes a pinned per-Agent sidecar as the Agent protocol,
  candidate and observation boundary. The sidecar remains a daemon-supervised
  client; the Rust daemon remains the sole authority writer.
- Pi remains the only Linux 1.0 qualified Agent/sidecar combination. Other
  Agents require independent package, protocol, lifecycle, recovery and
  negative qualification.
- Standard Workspace remains low-friction. Extended Home can add explicitly
  selected document/project roots and ordinary outbound network access, while
  credential stores, authority/bootstrap data, Docker/system sockets, system
  directories and privilege management remain hard-denied.
- Desktop, headless and foreground modes use one artifact, daemon and service
  model. Desktop uses Secret Service; headless targets an approved encrypted
  vault with locked diagnostic start, SSH TTY unlock and optional systemd
  encrypted-credential vault-unlock material.
- Linux/hardware evolution stabilizes narrow software ports only. Kernel
  modules, eBPF control planes, device schedulers and distributed authority are
  not Linux 1.0 prerequisites or claims.

## 3. Product, architecture and workload synchronization

The canonical product and architecture documents now define:

- the five-space information architecture: Home, Agents, Tasks, Resources and
  Activity;
- Memory candidate/admission/object lifecycle with SQLite + FTS5/metadata
  filtering;
- immutable local Skill packages/revisions compatible with `SKILL.md`, with
  scripts executable only through registered Tools;
- a useful static Tool family for workspace read/search/write/patch, bounded
  process/check and read-only HTTP fetch;
- one real `ContextRequest` and `ContextView` per admitted Task, with
  authorization before ranking and explicit loss;
- Task authority, server-issued preview, Effect/recovery and independent
  acceptance invariants;
- separate Agent/runtime/process identities and private framed sidecar
  control/data planes;
- ordered daemon restart, epoch fencing, original-key Effect reconciliation,
  Context rebuild and sidecar restart/quarantine.

The new UCR-01 target workload correlates:

```text
TaskContract
 -> admitted Memory
 -> pinned SkillRevision
 -> ContextView
 -> exact Tool descriptor
 -> sidecar/AgentExecution
 -> ProcessAttempt observation
 -> Artifact/Effect
 -> independent Verification/acceptance
```

It also defines cross-session recall, exact Skill reuse across two Tasks,
stable/changed Context repeated-input reduction, daemon/sidecar fault recovery,
six-family list/inspect/watch coverage and zero-tolerance safety assertions.
UCR-01 is not a new Gate, does not create B13 and cannot automatically pass
multiple Gates from one run.

## 4. Formal plan and release composition

The existing 53 task IDs remain unchanged. Their current counts remain:

- 15 `done`;
- 3 `in-progress`;
- 35 `not-started`;
- 0 `blocked`.

The Linux 1.0 work is organized into three active tracks without adding a new
phase or release Gate:

1. Runtime Spine;
2. Resource Value;
3. Product Operability.

`GMVP-LINUX` keeps the exact benchmark composition:

`B01 + B02 + B03 + B04 + B05 + B08 + B09 + B12`

P7-T08 separately requires release-operability and UCR-01 fixed-scenario
acceptance evidence. B06/B07 advanced benefit Gates, B10 MCP/dynamic Tool and
B11 Multi-Agent do not block Linux 1.0. The UCR-01 fixed-scenario `>=20%`
stable/changed Context repeated-input assertion remains an acceptance
condition without turning B06/B07 into release Gates or supporting a general
cross-workload Agent-benefit claim.

## 5. B01 statistical interpretation

The new B01 addendum preserves the original preregistration and immutable
attempt 1 while resolving its atomic one-attempt wording against the formal
campaign threshold:

- fixed current campaign `N = 20`;
- every started attempt remains in the denominator;
- at least 18/20 per-attempt successes;
- zero critical safety failures;
- no early pass, optional stopping or selective extension;
- aggregate median/p95 and confidence intervals;
- final independent verifier disposition.

B01 remains `running`. Attempt 1 remains one successful attempt and is not a
claim that reliability is at least 90 percent.

## 6. Contract evolution boundary

No file under `specs/`, generated bindings or runtime code changed. Future
Lane-CTR work must separately decide and synchronize, when actually needed:

- `skill-manifest`;
- `operation-descriptor`;
- `agent-adapter-manifest`;
- TaskContract resource bindings;
- server-issued preview reference;
- Memory Rust/TypeScript codegen.

The unified Personal projection remains private and versioned. A public
`ResourceSummary` is reconsidered only after a second real adapter/client
demonstrates the need.

## 7. Validation

Executed after semantic review and taxonomy correction:

| Check | Result |
|---|---|
| `pnpm run check:consistency` | pass: 273 requirements, 55 error codes, 63 schemas, 85 vectors, links, traceability, Personal plan/Gates, design sources and leases |
| `git diff --check` | pass |
| Rust/TypeScript build or tests | not-run; this batch changes documentation only |
| UCR-01 or product Gate campaign | not-run |

## 8. Preserved current facts and non-claims

- P1-T09 remains `in-progress`.
- B01 remains `running`, with only attempt 1 of the fixed 20-attempt campaign
  recorded as successful.
- P2-T01 and P2-T03 remain `in-progress`.
- All other existing task states remain unchanged.
- `GMVP-LINUX` remains `not-run`.
- Profile remains `implemented: 0`.
- No sidecar, Memory, Skill, Tool family, Context runtime, headless vault,
  UCR-01 runner, Gate, release or Profile behavior is claimed implemented by
  this batch.

## 9. Next implementation sequence

1. Claim a narrow Lane-CTR lease for only the public contracts proven necessary
   by the first P2/P3/P4/P5 implementation slice.
2. Continue P2-T02/P2-T03 Runtime Spine work without waiting for B01 as an
   implementation mutex.
3. Implement P3 Context ports after P2 application contracts stabilize, then
   P4 Memory/Skill against those ports.
4. Implement and qualify the pinned Pi sidecar under P5-T01/T02/T05.
5. Implement headless vault and Extended Home negative boundaries under
   Product Operability before P7-T08.
6. Preregister every UCR-01-contributing Gate separately before collecting
   release evidence.
