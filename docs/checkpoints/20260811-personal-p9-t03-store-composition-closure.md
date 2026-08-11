<!--
Task: P9-T03
Slice: D04
Classification: MVP task closure
Status: closed; PR #193 merged
-->

# P9-T03 Store access and composition-root optimization closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| Eliminate per-request `SqliteAuthorityStore::open` with a long-lived single-writer store and fail-closed semantics | D01: startup recovery+tick share one store at `54be4c1` (Linux scheduler_authority 39/39 + Clippy). D02: request-path handlers reuse `Arc<SqliteAuthorityStore>` at `2eb82c9` (Linux request_path 1/1, scheduler_authority 39/39, Clippy); only daemon startup + fixture helper open remain |
| Sink one Personal vertical composition seam out of `kernel-server` | D03: `admit_memory_candidate` moved to `crates/cognitive-store/src/memory_admission.rs`; `resource_api` consumes `cognitive_store::admit_memory_candidate`; Linux memory_admission 1/1 at `648e69f` |
| Stage-timing non-claim comparison for store access modes | D03: `cognitive-runtime` `store_access` collector records per-open vs long-lived raw stage nanos; hypothesis-only validator rejects forged agent-benefit claim levels; Linux store_access 3/3 + Clippy at `648e69f` |
| Final acceptance / docs / PR / lease / branch closure | this checkpoint; required CI run `31476761080` on PR #193 HEAD `64f89cd`; no Gate/release/Profile claim |

## Non-claims

No Gate, release, Profile, GMVP-LINUX, B06/B07 product claim, or Agent-benefit claim.
Store-access stage timings are hypothesis-only observations.

## Closure

Required Ubuntu/Windows CI run `31476761080` passed for HEAD
`64f89cd054765766b5c53d1bc22402e835987a37`. PR #193 is the task closure PR.
Lease `lease/personal/P9-T03/store-composition` closes with merge; task branch
deleted after merge; local `main` reconciled.
