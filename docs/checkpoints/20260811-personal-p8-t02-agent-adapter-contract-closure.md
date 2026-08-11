<!--
Task: P8-T02
Slice: D04
Classification: MVP task closure
Status: acceptance mapped; awaiting required CI, PR merge, and lease closure
-->

# P8-T02 Universal Agent Adapter Contract closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| AKP-only adapter capability declaration/registration; candidate-only; no public listener | D01 `register_agent_adapter` / `verify_registered_agent_adapter`; Linux `agent_adapter_manifest` 3/3 + Clippy at `d5b12a9` |
| adapter lifecycle activate/pause/stop over registered declaration digests; channel isolation | D02 lifecycle handle + management-channel gate; Linux `agent_adapter_manifest` 5/5 + Clippy at `b94d4c6`; stale-digest and Task-channel negatives |
| Lane-CTR `agent-adapter-manifest` contract + generated bindings | D03 `specs/schemas/agent-adapter-manifest.schema.json` + Rust/TS bindings; Linux schema tests 2/2 + Clippy at `791d5ff`; ADR-0043 mention for reachability |
| final acceptance / docs / PR / lease / branch closure | this checkpoint + Draft PR #187 |

## Non-claims

No Gate, release, Profile, GMVP-LINUX, non-Pi agent qualification (P8-T03),
public A2A listener, live Provider/Pi campaign, Task/Effect completion, or
SidecarSession mutation claim.

## Remaining delivery actions

Mark Draft PR #187 ready after required CI for the closure HEAD, merge, close
`lease/personal/P8-T02/agent-adapter-contract`, delete the task branch, and
reconcile local `main`. Then claim the next ready Personal task.
