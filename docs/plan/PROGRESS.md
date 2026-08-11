# PROGRESS — 单页进度仪表

## Current snapshot (2026-08-11)

This section is the authoritative current view. Entries below `Historical
evidence journal` preserve execution-time facts and cannot override it.

Repository identity: `cognitiveos-personal` is the only active implementation
project. CognitiveOS design, specifications, conformance assets, and reusable
kernel code are the architecture/contract foundation for Personal, not a
second product backlog. See [PROJECT-IDENTITY.md](../governance/PROJECT-IDENTITY.md).

| Area | Current status | Evidence boundary | Next actionable step |
|---|---|---|---|
| Project focus | `cognitiveos-personal`: active and sole implementation project | CognitiveOS architecture assets remain reference/contract inputs; no second product backlog | continue P5-T04/D01–D04 on task branch |
| Active task lease | `lease/personal/P5-T04/dynamic-tool-ecosystem` | branch `personal/P5-T04-dynamic-tool-ecosystem`; writable paths listed in PARALLEL-LANES | exact native Linux `dynamic_tool_ecosystem` tests + Clippy; Draft PR + required CI; ADR-0050 B10 disposition |
| P5-T04 Post-1.0 dynamic Tool + B10 | **in-progress** | D01–D03 authority-path module + ADR-0050 + tools B10 harness written; Linux/CI `not-run` | push checkpoint; run Linux focused tests; close D04 |
| P5-T03 Post-1.0 MCP Tool adapter | **done** | D01–D04 closed; Linux 4/4 + Clippy at `a83bdb8`; required CI `31482773002` on `4c06161`. Closure checkpoint `20260811-personal-p5-t03-mcp-tool-adapter-closure.md`. | retain closure evidence; no B10/Gate/release/Profile claim |
| P7-T08 Public Linux 1.0 Gate (`GMVP-LINUX`) | **done** | D01–D04 closed; B08 MVP `pass` (ADR-0048) at `65a736c` + CI `31479512940`; GMVP-LINUX MVP `pass` (ADR-0049) at `b3f4b88` + CI `31480604511`. Closure checkpoint `20260811-personal-p7-t08-gmvp-linux-closure.md`. | retain closure evidence; no Profile/Windows B01-W claim |
| B08 Memory + Skill Gate | **pass** (MVP, ADR-0048) | Fixed 11-row authority-path matrix + harness at `65a736c`; Linux 14/14+1/1+Clippy; CI `31479512940` | retain evidence |
| GMVP-LINUX / Linux 1.0 | **pass** (MVP, ADR-0049) | Fixed composition binder + prior Gate MVP dispositions; CI `31480604511` on `b3f4b88` | retain evidence; no Profile claim |
| P9-T03 Store access and composition-root optimization | **done** | D01–D04 closed; PR #193 merged. Linux evidence through `648e69f`; required CI `31476761080` on `64f89cd`. Closure checkpoint `20260811-personal-p9-t03-store-composition-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P9-T02 Authority-path structure debt | **done** | D01–D04 closed; PR #192 merged at `main@cff740192601f97fd7071f9f0e1a00f824ae6141`. Linux evidence through `a11d0bd`; required CI `31471319404` on `b1cc8a7`. Closure checkpoint `20260811-personal-p9-t02-structure-debt-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P8-T03 First non-Pi Agent qualification | **done** | D01–D04 closed; PR #191 merged at `main@47478e40aed0c96808875225df91d6452ca1fb49`. Codex fixture identity/lifecycle/non-claim matrix. Required CI `31463130827`. Closure checkpoint `20260811-personal-p8-t03-non-pi-agent-closure.md`. | retain closure evidence; no Gate/release/Profile/Pi-transfer claim |
| P8-T06 Cross-episode learning loop | **done** | D01–D04 closed; PR #190 merged at `main@ad6656566ca0ea365b532b8e059d50d061c5c1df`. Reflexion Memory/Skill candidate planners + daemon admission wiring. Required CI `31461384771` / closure `31462013806`. Closure checkpoint `20260811-personal-p8-t06-learning-loop-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P8-T05 Context compaction and adaptive budgets | **done** | D01–D04 closed; PR #189 merged at `main@fa4f74a8feaadaa74affca90cb37660f40cdeb25`. Digest-bound compaction, adaptive budgets, UCR-01 non-claim benefit observation. Required CI `31459558236` / closure `31460220901`. Closure checkpoint `20260811-personal-p8-t05-context-compaction-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P8-T04 Deterministic harness hooks | **done** | D01–D04 closed; PR #188 merged at `main@f85a14b5eba20311e49367bf1e6a3222767691b0`. Lifecycle hooks + graded Skill/rule load. Required CI `31458052642`. Closure checkpoint `20260811-personal-p8-t04-harness-hooks-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P8-T02 Universal Agent Adapter Contract | **done** | D01–D04 closed; PR #187 merged at `main@f31eefd69f6992d7b7957fef8f6fe00afaa1ae3c`. Private AKP registration/lifecycle + Lane-CTR `agent-adapter-manifest` + generated bindings. Required CI `31453659735`. Closure checkpoint `20260811-personal-p8-t02-agent-adapter-contract-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P8-T01 documentation restructure and 2.0 design baseline | **done** | D01-D03 deliver AXIOMS/governance convergence, whitepaper/product/architecture/ADR-0041+ design baseline, plan/ledger repair, Phase 8/9 registration, and closure checkpoint. Local consistency/diff and tools failure-injection passed; required Ubuntu/Windows CI run `31383446541` passed for design revision `cd08da7`. PR #180 merged at `main@aa18ec2296cdf317ef0689365ea466652add816b`. Documentation-only; no implementation, Gate, release, or Profile claim. | retain closure evidence |
| P5-T02 Sidecar contract, registration, instance identity | **done** | D01-D03 satisfy the unchanged acceptance at `58ff0a723a8eae0f7fc89d9a99e9fdd55406aa92`: inactive registration, epoch-fenced SidecarSession activate, pause/resume/stop/recover, redacted health with `process_bound=false`, and management-session admin-cli callers. Exact native Linux focused runtime 11/11, admin 1/1, Clippy, and required Ubuntu/Windows CI run `31391916831` passed. Identity separation keeps AgentExecution/PiSession/process/Task absent and non-conflated. Closure checkpoint `20260810-personal-p5-t02-sidecar-foundation-closure.md`; PR #181 merged at `main@e0c3fb85e8a99cbcb4d8fd014fb7cc89d0b23a79`. | retain closure evidence; no B09/release/Profile claim |
| P2-T08 Runtime Spine E2E Gate | **done** | D01–D04 closed under ADR-0046. Fixed matrix + non-claim report at `be7febb490fcbdf9970a700b6b6975ae49aadffe`; CI `31407542786`; owner `affirm all` → B02/B04/B05/B12 MVP `pass`. Closure checkpoint `20260811-personal-p2-t08-runtime-spine-closure.md`; PR #182 merged at `main@9a1befe475e35902447ab1562cb8d747aa908a69`. | retain closure evidence; no GMVP-LINUX/B08/B09/release/Profile claim |
| P5-T05 B09 managed Pi + sidecar qualification | **done** | D01–D04 closed under ADR-0047. Fixed matrix + non-claim report at `548f138`; required CI `31423464703` on PR #183 head `ed1d1a9`; owner `affirm B09` → B09 MVP `pass`. Closure checkpoint `20260811-personal-p5-t05-b09-managed-pi-closure.md`. Standing §2.3 now also covers equivalent fixed-denominator Gate MVP self-disposition and owner-designated test Provider key import into approved Secret Store. | retain closure evidence; no GMVP-LINUX/B08/release/Profile/non-Pi claim |
| P7-T01 Release pipeline, six-resource manifest, SBOM & attestation | **done** | D01–D04 closed and PR #184 merged at `main@3198614496571ac251821d2eff1f982274959f06`. Signed six-family manifest, SBOM/artifact digest binding, immutable toolchain pins, acceptance mapping. Exact native Linux `release_manifest` 11/11 + Clippy at `34812f8`. Closure checkpoint `20260811-personal-p7-t01-release-pipeline-closure.md`. | retain closure evidence; no Gate/release/Profile/GMVP-LINUX claim |
| P7-T02 Transactional lifecycle, Memory/Skill backup/restore | **done** | D01–D04 closed on PR #185 merged at `main@a1f6083916209dd43252984ae335525230723e4a`. Secret-excluding inventory, digest-bound Memory/Skill/bindings export, restore preflight, and transactional update/rollback/uninstall authority path. Exact native Linux `personal_backup` 15/15 + Clippy at `68abc82`; required CI `31449589853`. Closure checkpoint `20260811-personal-p7-t02-lifecycle-backup-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P7-T03 Six-resource doctor, headless vault, sidecar/process/effect support | **done** | D01–D04 closed; PR #186 merged at `main@47e88a19aa9de732c22887009ea36db01d2061a8`. Redacted six-resource, headless vault, and operability doctor sections on `/personal/doctor`. Linux evidence through `749a0c3`; required CI `31451402260`. Closure checkpoint `20260811-personal-p7-t03-six-resource-doctor-closure.md`. | retain closure evidence; no Gate/release/Profile claim |
| P1-T09 route implementation | `done` | `experimental-local-only`; retained campaign `001` is failed at 2 successes / 8 failures after 10 attempts. Under ADR-0039 successor `002` completed its fixed six counted outcomes: Attempts 1, 3, 4, 5, and 6 passed the full clean route, while Attempt 2 failed bounded graphical Desktop readiness before product activation. Required aggregate, verifier, and CI closure completed. | select the next ready formal task; Provider credential entry remains graphical hidden input only |
| B01 first-install/first-conversation Gate | **pass** | Retained campaign `001` is `fail`; successor `B01-clean-linux-first-install-first-conversation-002` completed Attempt 6 of formal minimum 6 with 5 successes, 1 failure, success rate 83.33%, zero critical safety failures, complete aggregate statistics, and affirmative independent verifier disposition. Required Ubuntu and Windows CI passed for closure revision `0ef0b21`. Owner-waived transition Attempt 7 is retained outside the denominator because no product operation occurred. | retain the redacted closure evidence; B01 pass does not pass G1, GMVP-LINUX, release, or Profile |
| P2-T01 TaskApplicationService | **done** | `P2-T01/D01` satisfies the unchanged task acceptance: Linux focused service 4/4, management 3/3, store 6/6, Clippy/fmt and required CI passed at `main@7f763c8`; B02/B04/B05/B12 remain `not-run` | P2-T02/D01 may consume the stable service; task completion creates no Gate/release/Profile claim |
| P2-T02 Personal application service | **done** | D01-D04 now satisfy the unchanged acceptance: authenticated daemon-owned intent record/interpret, server-issued preview/admit and bounded Task watch; private six-family projection/watch; deterministic CLI and Pi sidecar parity with isolated Task/management channels and read-only client boundaries. Each slice has exact Linux evidence and required Ubuntu/Windows CI. | P2 Gates B02/B04/B05/B12 remain `not-run`; select an unrelated ready formal task |
| P2-T03 scheduler/runtime | **done** | D01-D05 satisfy the unchanged acceptance at exact immutable `08932f7868d46f494aaa76835f4818fd7a1f2962`: durable scheduler persistence/CAS fencing, STOP-before-lease budget authority, fail-closed durable Effect resolution, exact owner+epoch Effect closure, and restart-safe one-time WIA/verified-continuation worker handoff. Native Linux focused validation plus workspace fmt/build/test/Clippy and required Ubuntu/Windows CI passed at that checkpoint. Candidate WIA remains limited to atomic `DECIDE -> ACT`; only independently verified continuation authority can enter `CONTINUE -> OBSERVE`. PR #160 passed both required jobs and merged at `main@678b653c588c45ea02bf393ad7038ef760c0971b`. | P2-T06/P2-T07 may consume the scheduler boundary; B05/B12 remain `not-run` |
| P2-T07 verifier persistence | `done` | `P2-T07/D01` and `P2-T07/D02` are complete: exact immutable `08932f7868d46f494aaa76835f4818fd7a1f2962` covers the durable fixed-post-state, verification request/report, currentness, checkpoint, continuation-authority boundary, and verifier identity/evidence negatives; exact remote Linux validation at `df7d483282f3ef0a6bbb17bae3d29bb24f13e0f7` passed the focused verifier test module 7/7 and `cargo clippy -p kernel-server --all-targets -- -D warnings`; local `cargo fmt --all`, `git diff --check`, and lints passed. The path remains append-only and non-authoritative; it does not become Provider/Tool execution, Artifact closure, Task completion, a Gate, release, or Profile claim. | select the next ready formal task |
| P3-T01 Context source/retrieval port | **done** | `P3-T01/D01` closes the unchanged acceptance at implementation revision `0ad1ddb95f4e347d0c205597e69ad8818819948e`: task-bound ContextRequest plus immutable workspace source and ContextView persistence; tenant/scope metadata filtering before body access/ranking; current durable authorization/revocation revalidation per body; and owner-local management-session admission. Exact native Linux passed `cargo test -p kernel-server` and Context-store 9/9; required Ubuntu/Windows CI passed in PR #161. The real scheduler resolves/persists Context before candidate-only Pi transport. | B03 remains `not-run`; P3-T02 may consume the stable Context source port without a Gate/release/Profile claim |
| P3-T02 Context Builder and budgets | `done` | D01-D02 satisfy the unchanged acceptance at `0d8f5628a897aea32ee4cb7929bac1320ccb2a96`: daemon-owned System/Task fragments, required fail-closed budgets, semantic duplicate loss, source-family filtering, role-specific freshness before body loading, and digest-bound loaded/excluded source trace. Exact native Linux focused stale-source validation and Clippy passed; required Ubuntu/Windows CI passed in merged PR #166 at `main@c78c58c096765caffb638e32dc8d74fd412765a9`. B03, UCR-01, release, and Profile remain `not-run`/non-claim. | P3-T03 and P4-T01 may consume the stable Context Builder port; B03 remains `not-run` |
| P3-T03 Artifact CAS | `done` | D01-D02 satisfy the unchanged acceptance at `87e436e22dae3722fb0ced6c8ceeb8f0f4deddc8`: one daemon-owned bounded filesystem CAS, immutable metadata, digest validation, atomic publish, authorized access, partial-write cleanup, and verifier-side validation before report persistence. Exact native Linux focused verifier/CAS tests, Clippy, and fmt passed; required Ubuntu/Windows CI passed in PR #168. | B03, Task completion, Gate, release, and Profile remain separate/unclaimed |
| P3-T04 Context cache and telemetry | `done` | D01-D02 close the unchanged acceptance at `128915e15d4f4b4b98f195f0b6a49a6de76f34f2`: digest-only stable-prefix/delta metadata reuse repeats every Context authorization/body validation; stale/revoked content is rejected. Durable action/error/evidence-digest signatures bound repeat/no-progress control to daemon facts and fail closed without an owned alternate strategy. Exact native Linux and required Ubuntu/Windows CI passed. | B03, Gate, release, and Profile remain separate/unclaimed |
| P3-T05 UCR-01 benefit runner and stable baseline | `done` | D01 closes the unchanged acceptance at `72690e028c7f3fb3896782c1874f575f35ebe165`: the fixed UCR-01 runner binds six-family fixture/trace/baseline digests and bounded measurements, and rejects authority-shaped claims. Exact native Linux and required Ubuntu/Windows CI passed. | B03/B06/B07, Gate, release, and Profile remain separate/unclaimed |
| P3-T06 B03 Context correctness and benefit collection | `done` | B03 MVP passed under ADR-0040 at `7ea39472899e8ac77f30e589da89b7b4e0b316a2`: the fixed matrix passed 22/22 Rust authority-path tests and 11/11 evaluator/tooling tests, native Linux/Clippy, cleanup/redaction, owner review, and required Ubuntu/Windows CI run `31347323835`. PR #171 merged at `main@175b1974183fcfd4063bc5a3cd8e9f110af3980e`; lease, branch, and local main reconciliation are complete. B06/B07 remain optional raw performance observations. | select the next ready Personal task; B03 does not pass GMVP-LINUX, release, Profile, B06/B07, or UCR-01 utility |
| P4-T01 Memory store, admission, and policy | `done` | D01 closes at `e4eb38ad9aaba13f04fb51657dfdc884af66cdc5`: deterministic proposal policy, append-only SQLite v16 candidate/decision/object records, atomic Context-source revalidation, daemon-private admission service, and failure-first direct-admit/source-mismatch/retention/scope/no-partial-object coverage. Exact native Linux focused tests and Clippy passed; required Ubuntu/Windows CI passed. No FTS/retrieval, lifecycle/forget, public API, B08, Gate, release, or Profile claim. | retain closure evidence; P4-T02 may consume the stable Memory admission boundary |
| P4-T02 Memory FTS5 retrieval baseline | `done` | `P4-T02/D01` closes at `aca44d13ba2ee97f758dc36ffc96066dc43af722`: migration v17 provides a rebuildable daemon-private FTS5 index; authoritative admitted-decision/scope/purpose/retention/current-source filtering occurs before ranking; and candidates are metadata-only. Exact native Linux focused migration 8/8 and FTS 4/4 regressions, plus required Ubuntu/Windows CI, passed. Memory lifecycle/forget, public API/projection, B08, Gate, release, and Profile remain unclaimed. | P4-T03 may consume the stable derived-index boundary; retain P4-T02 closure evidence |
| P4-T03 Memory lifecycle, retention and forget | `done` | `P4-T03/D01-D03` satisfy the unchanged acceptance: forget, expiry-boundary, and version/update/conflict lifecycle facts preserve immutable admission history, enforce expected-version CAS lineage, and atomically invalidate/move derived FTS rows. Exact native Linux focused tests passed 16/16, native Clippy passed, and required Ubuntu/Windows CI passed. PR #174 merged at `main@e1454f3775eab5c72d9cb2b8e0a5c1e98b895f0f`; the lease and remote task branch are closed. | select the next unblocked, implementation-ready Personal task; no public Memory API, B08, Gate, release, or Profile claim |
| P4-T04 Skill package, revision, local import, and binding | `done` | `P4-T04/D01-D03` complete daemon-private immutable package/revision/digest/import, scope-bound binding, append-only revoke, same-package supersede, exact-pin explanation, digest-drift rejection, and management-session import authorization. Exact native Linux focused tests/Clippy and required Ubuntu/Windows CI passed at `883cd5fca9b14182cc5b5632948476b31b8744a3`. | public API/projection, Context/Task consumption, B08, Gate, release, and Profile remain separate |
| P4-T05 Memory/Skill API and unified projection | `done` | Closure checkpoint `20260810-personal-p4-t05-memory-skill-api-closure.md`; D01-D05 provide task-bound projection, authority-backed explain reads, durable Memory forget/remember, Skill revoke/import/bind callers, and failure-first channel/payload/authority negatives. PR #176 is merged and its task branch/lease/main reconciliation is complete. Required Ubuntu/Windows CI run `31335218082` passed; local fmt, diff, and consistency passed. | retain closure evidence; public contract generation, B08, Gate, release, and Profile remain separate |
| P5-T01 Agent + sidecar package acquisition/install lifecycle | `done` | Closure checkpoint `20260810-personal-p5-t01-pi-acquisition-closure.md`; D01-D03 deliver authenticated official-Pi acquisition evidence, durable versioned activation/rollback, and stopped/absent uninstall quarantine. Exact native Linux focused runtime/store/admin validation and Clippy passed at `3413598e19746807674c31b12bc7814a848edcdf`; required Ubuntu/Windows CI run `31355388291` passed. PR #178 merged at `main@c20c13c09953b97dbcf023b9f4ad6d9458039c71`; branch, lease, and local main reconciliation are complete. | retain closure evidence; no AgentInstance, sidecar session, process supervision, Effect, Task completion, B09, release, or Profile claim |
| P7-T04 Performance campaign and regression floor | **done** | Closure checkpoint `20260810-personal-p7-t04-performance-governance-closure.md`; D01-D05 deliver deterministic module benchmarks, governed-path stage timing, B06/B07 non-claim observations, module regression-floor policy, and fixed-native governance A/B non-inferiority (`sha256:b90b8452e5d7b833ada423fb6d9d8e6ae5db92830c22ebd2363d435e4fc4aad9`) on Draft PR #179. Required Ubuntu/Windows CI passed for the implementation revisions. | retain closure evidence; B06/B07, Gate, release, Profile, and GMVP-LINUX remain non-claims |
| P2-T04 private worker composition | **done** | `P2-T04/D01` closes at immutable `a8ef5c00654e1c05a4c30beb193b9c026654c2f1`: the daemon resolves and seals a request-bound ContextView before bounded private Pi proposal; Pi is opaque candidate-only; the daemon owns candidate admission, budget, WIA/continuation, Effect, progress, evidence, and Task state. Real SQLite negatives cover post-discovery revocation, required Context failure, duplicate candidate retry suppression, atomic candidate/WIA one-time handoff, stale/replaced lease rejection, and authority-shaped Pi response rejection. Exact native Linux and required Ubuntu/Windows CI passed. | consumed as a completed prerequisite; B03, release, Profile, Tool execution, and Task completion remain separate/unclaimed |
| P2-T05 native Tool registry | **done** | The unchanged task acceptance closes at `72a7e55e5a780827438bfb0fb42172cfd1e5bec1`: static daemon-owned six-family catalog; descriptor/version/digest/risk binding; persisted-descriptor verification; private Tool projection; and workspace/process/HTTP pre-executor validators. Exact native Linux focused Tool registry tests passed 7/7 with fmt; required Ubuntu and Windows CI passed in PR #159. | P2-T06 may consume this pre-executor boundary; Tool execution, external I/O, mutation, reconciliation, Task completion, Gates, release, and Profile remain unclaimed |
| P2-T06 Tool/process executor | **done** | D01-D04 satisfy the current formal acceptance at immutable `bfcc684db6685e1077050a4b3c82fcf84c524711`: bounded WorkspaceRead and ProcessCheck execution; durable Intent/Effect persist-before-dispatch; original-key idempotency and unknown-outcome reconciliation; bounded cursor/output and redaction; before/mid/after fault coverage; and daemon-private supervisor registration, ownership, fencing, orphan/recovery/shutdown, timeout, and fail-closed observation. Exact native Linux focused tests (26 passed), Clippy/fmt, consistency, and required Ubuntu/Windows CI passed in Draft PR #162. The injected `FailClosedProcessObservationSource` is the accepted safety boundary: no arbitrary PID attach, public Process resource, Task completion, release, Gate, or Profile claim is made. Production platform observation remains deferred to the managed-Pi/process-supervision path. | task acceptance is closed; retain D01-D04 evidence and keep B05/B12, release, Gate, Profile, and managed-Pi production supervision separate |
| Personal 1.0 design baseline | `documented` | ADR-0035..0038, six-family product/architecture docs, Pi sidecar map, UCR-01, B01 statistical addendum, typed release dependencies, support/environment registry and handoff are synchronized; consistency and diff checks passed; this is documentation/tooling evidence only | select the next non-overlapping Lane-CTR, Runtime Spine or B01 campaign slice from the formal plan |
| Local command/test routing | `fail-fast baseline` | `COMMAND-SHELL-PS51`: local commands use Windows PowerShell 5.1, so `&&`/`||` are forbidden. `RUST-LINK-DEV-WIN-GNU-01`: local Windows GNU Rust compiling/linking is unsupported with known linker exit 121; required Rust validation routes to supported CI or exact-revision native Linux | do not repeat parser/linker failures in feature Slices; use the environment registry before selecting validation commands |
| Profile conformance | `implemented: 0` | non-claim | independent applicable-MUST evidence only |

### Layer 1 — Formal task progress

| Total | Done | In progress | Blocked | Not started | Remaining |
|---:|---:|---:|---:|---:|---:|
| 62 | 53 | 1 | 0 | 8 | 9 |

`P5-T04` is `in-progress` (D01–D03 implementation + ADR-0050/tools harness;
Linux/CI pending). Formal task completion remains independent from
GMVP-LINUX, release, Profile, and Windows B01-W claims.

### Layer 2 — Current Delivery Slice queue

| Slice | Status | Actual evidence boundary | Executable next action |
|---|---|---|---|
| `P5-T04/D01` | `in-progress` | dynamic package bind + disabled discovery in `dynamic_tool_ecosystem.rs`; Linux `not-run` | exact native Linux focused tests; then close D01 and enter D02 |
| `P5-T04/D02` | `ready` | enable/disable/quarantine + TaskContract exposure implemented pending D01 Linux exit | start after D01 Linux pass |
| `P5-T04/D03` | `ready` | reconcile/composite/cache/bypass implemented pending D01–D02 exit | start after D02 |
| `P5-T04/D04` | `ready` | ADR-0050 + `tools` B10 non-claim harness written; disposition pending | close after D01–D03 evidence + CI |
| `P5-T03/D01` | `done` | fixture manifest + transport-only init at `a83bdb8`; Linux 4/4 | consumed by D02 |
| `P5-T03/D02` | `done` | drift/timeout/no-auto-enable covered in same Linux 4/4 | consumed by D03 |
| `P5-T03/D03` | `done` | direct-bypass + non-claim report covered in same Linux 4/4 | consumed by D04 |
| `P5-T03/D04` | `done` | acceptance + required CI `31482773002` on `4c06161`; PR #195 | consumed by completed P5-T03 task |
| `P7-T08/D01` | `done` | ADR-0048 + `tools` B08 non-claim harness | consumed by D02 |
| `P7-T08/D02` | `done` | ADR-0048 matrix at `65a736c`; CI `31479512940`; B08 MVP pass | consumed by D03 |
| `P7-T08/D03` | `done` | ADR-0049 + `gmvp-linux-gate` composition binder at `b3f4b88`; CI `31480604511` | consumed by D04 |
| `P7-T08/D04` | `done` | GMVP-LINUX MVP pass + acceptance closure; PR #194 | consumed by completed P7-T08 task |
| `P7-T01/D01` | `done` | signed six-family release-manifest authority path; native Linux 6/6 + Clippy at `3108889`; required CI `31425522168` for `3bde68c`; merged onto main after P5-T05 | consumed by D02 |
| `P7-T01/D02` | `done` | SBOM/artifact digest binding + contaminated-inventory rejection; Linux `release_manifest` 9/9 + Clippy at `c1f06f4` | consumed by D03 |
| `P7-T01/D03` | `done` | immutable toolchain pins + acquisition-lock trust; Linux `release_manifest` 11/11 + Clippy at `34812f8` | consumed by D04 |
| `P7-T01/D04` | `done` | acceptance mapping + closure; PR #184 merged at `main@3198614` | consumed by completed P7-T01 task |
| `P7-T02/D01` | `done` | secret-excluding Personal backup inventory planner; Linux `personal_backup` 3/3 + Clippy at `8750666`; required CI `31430152517` on PR #185 | consumed by D02 |
| `P7-T02/D02` | `done` | digest-bound Memory/Skill/bindings export; Linux `personal_backup` 7/7 + Clippy at `6b6d245` | consumed by D03 |
| `P7-T02/D03` | `done` | restore preflight rejects schema/incomplete/migration/digest mismatches; Linux coverage included in 15/15 at `68abc82` | consumed by D04 |
| `P7-T02/D04` | `done` | transactional update/rollback/uninstall + acceptance; Linux 15/15 + Clippy at `68abc82`; required CI `31449589853` on `8b388e8`; closure checkpoint written; PR #185 merged at `main@a1f6083` | consumed by completed P7-T02 task |
| `P7-T03/D01` | `done` | redacted six-resource doctor health on `/personal/doctor`; Linux `six_resource_doctor` 4/4 + Clippy at `13e46eb` | consumed by D02 |
| `P7-T03/D02` | `done` | headless vault locked/TTY/unattended redacted diagnostics; Linux `headless_vault_doctor` 3/3 + Clippy at `9dc2dcd` | consumed by D03 |
| `P7-T03/D03` | `done` | sidecar/process/effect/migration doctor facts; Linux `operability_doctor` 3/3 + Clippy at `749a0c3` | consumed by D04 |
| `P7-T03/D04` | `done` | acceptance mapping + closure; required CI `31451402260` on `a8db5cf`; checkpoint written; PR #186 merged at `main@47e88a1` | consumed by completed P7-T03 task |
| `P8-T02/D01` | `done` | daemon-private AKP-only adapter capability registration; Linux `agent_adapter_manifest` 3/3 + Clippy at `d5b12a9` | consumed by D02 |
| `P8-T02/D02` | `done` | adapter lifecycle activate/pause/stop; Linux `agent_adapter_manifest` 5/5 + Clippy at `b94d4c6` | consumed by D03 |
| `P8-T02/D03` | `done` | Lane-CTR `agent-adapter-manifest` schema + generated Rust/TS bindings; Linux schema tests 2/2 + Clippy at `791d5ff` | consumed by D04 |
| `P8-T02/D04` | `done` | acceptance mapping + closure; required CI `31453659735` on `f5e427f`; checkpoint written; PR #187 | consumed by completed P8-T02 task |
| `P8-T04/D01` | `done` | daemon-owned lifecycle hook registry; Linux `harness_hooks` 3/3 + Clippy at `3103a80` | consumed by D02 |
| `P8-T04/D02` | `done` | digest-bound management-channel invoke; Linux `harness_hooks` 4/4 + Clippy at `169b303` | consumed by D03 |
| `P8-T04/D03` | `done` | graded Skill/rule load by context cost; Linux `graded_load` 2/2 + Clippy at `bc3dacd` | consumed by D04 |
| `P8-T04/D04` | `done` | acceptance mapping + closure; required CI `31457314002` on `15e7200`; checkpoint written; PR #188 | consumed by completed P8-T04 task |
| `P8-T05/D01` | `done` | digest-bound compact artifact with explicit loss; Linux `context_compaction` 2/2 + Clippy at `8544b1e` | consumed by D02 |
| `P8-T05/D02` | `done` | adaptive fragment budgets without skipping body reauthorization; Linux `adaptive_budget` 2/2 + Clippy at `0f0f65c` | consumed by D03 |
| `P8-T05/D03` | `done` | UCR-01-compatible non-claim benefit observation; Linux `compaction_benefit` 2/2 + Clippy at `e15492a` | consumed by D04 |
| `P8-T05/D04` | `done` | acceptance mapping + closure; required CI `31459558236` on `1d2103e`; checkpoint written; PR #189 merged at `main@fa4f74a` | consumed by completed P8-T05 task |
| `P8-T06/D01` | `done` | Reflexion failure-lesson → digest-bound Memory candidate planner; Linux `learning_loop` 2/2 + Clippy at `8ba3fe0` | consumed by D02 |
| `P8-T06/D02` | `done` | admit only via `decide_memory_admission`; forged outcome + source mismatch + explainable forget; Linux 4/4 + Clippy at `31e7384` | consumed by D03 |
| `P8-T06/D03` | `done` | Skill candidate plan + capability-grant reject + explainable binding revoke; Linux 5/5 + Clippy at `b81414d` | consumed by D04 |
| `P8-T06/D04` | `done` | acceptance mapping + closure; required CI `31461384771` on `29399b7`; checkpoint written; PR #190 merged at `main@ad66565` | consumed by completed P8-T06 task |
| `P8-T03/D01` | `done` | Codex fixture package identity + AKP registration independent of Pi/B09; Linux `non_pi_agent` 2/2 + Clippy at `12c8b3e` | consumed by D02 |
| `P8-T03/D02` | `done` | Codex fixture activate/pause/stop on management channel; Linux 3/3 + Clippy at `6847889` | consumed by D03 |
| `P8-T03/D03` | `done` | fixed-denominator non-claim qualification report; Linux 4/4 + Clippy at `b41f06f` | consumed by D04 |
| `P8-T03/D04` | `done` | acceptance mapping + closure; required CI `31463130827` on `5d1c0c7`; checkpoint written; PR #191 merged at `main@47478e4` | consumed by completed P8-T03 task |
| `P9-T02/D01` | `done` | extract `scheduler_authority` embedded tests; Linux 38/38 + Clippy at `c4bbbde` | consumed by D02 |
| `P9-T02/D02` | `done` | production helpers split into cohesive submodules; Linux `scheduler_authority` 38/38 + Clippy at `dba5e2b` | consumed by D03 |
| `P9-T02/D03` | `done` | `tool_executor/` + `sqlite/` directory splits with focused-test parity; Linux tool_executor 27/27, sqlite 1/1, scheduler 38/38, Clippy at `a11d0bd` | consumed by D04 |
| `P9-T02/D04` | `done` | acceptance mapping + closure; required CI `31471319404` on `b1cc8a7`; checkpoint written; PR #192 merged at `main@cff7401` | consumed by completed P9-T02 task |
| `P9-T03/D01` | `done` | daemon startup recovery+tick share one `SqliteAuthorityStore`; Linux scheduler_authority 39/39 + Clippy at `54be4c1` | consumed by D02 |
| `P9-T03/D02` | `done` | request-path handlers reuse daemon-owned `Arc<SqliteAuthorityStore>`; Linux request_path 1/1, scheduler_authority 39/39, Clippy at `2eb82c9` | consumed by D03 |
| `P9-T03/D03` | `done` | Memory admission sunk to `cognitive-store`; store_access stage-timing non-claim in `cognitive-runtime`; Linux memory 1/1, store_access 3/3, request_path 1/1, Clippy at `648e69f` | consumed by D04 |
| `P9-T03/D04` | `done` | acceptance mapping + closure; required CI `31476761080` on `64f89cd`; checkpoint written; PR #193 | consumed by completed P9-T03 task |
| `P2-T01/D01` | `done` | unchanged task acceptance plus Linux focused tests and required CI | consumed by P2-T02/D01 |
| `P2-T03/D01` | `done` | scheduler persistence, CAS lease and eligibility passed prior Linux/store validation | consumed by D02 |
| `P2-T03/D02` | `done` | durable authority ceilings and STOP-before-lease passed prior exact-Linux focused validation | consumed by D03 |
| `P2-T03/D03` | `done` | exact immutable Linux `7489f4c`: scheduler authority tests 11/11 and M5 durable task-binding reverse-index store chain 6/6 passed; missing, ambiguous, inconsistent and unknown-state bindings fail closed | consumed by D04 |
| `P2-T03/D04` | `done` | exact immutable Linux `b396d32`: scheduler authority closure/release tests 13/13, scheduler lease/recovery 6/6, Clippy/fmt and required Ubuntu/Windows CI passed; malformed and stale releases preserve fenced leases | consumed by D05 |
| `P2-T03/D05` | `done` | daemon-only candidate admission, sealed WIA issuance, exact scheduler lease-bound one-time consumption, startup recovery, and independently verified continuation entry are complete. Exact immutable `08932f7868d46f494aaa76835f4818fd7a1f2962` passed native Linux focused worker/recovery tests, `cargo fmt --all -- --check`, workspace build/test/Clippy, and required Ubuntu/Windows CI. Candidate WIA cannot authorize a second budget debit or `CONTINUE -> OBSERVE`; continuation entry atomically consumes only the verified authority under the exact active scheduler lease. No worker output becomes progress/evidence, and no Task acceptance or completion is claimed. | P2-T03 formal acceptance is complete; retain this boundary for P2-T07/P2-T06 consumers |
| `P2-T07/D01` | `done` | daemon-private fixed post-state, verification request/report, independent currentness validation, checkpoint, and continuation authority are complete at `08932f7868d46f494aaa76835f4818fd7a1f2962`. Exact native Linux focused recovery tests and full workspace fmt/build/test/Clippy passed, as did required Ubuntu/Windows CI. Its only permitted integration outcome is `ACT -> VERIFY -> CONTINUE -> OBSERVE`; it cannot create Task acceptance/completion or trust worker output as evidence. | select a separate P2-T07 Artifact/evidence/acceptance slice when implementation dependencies are ready |
| `P2-T04/D01` | `done` | The private scheduler-to-deterministic-Context-to-pinned-Pi candidate composition closes at `a8ef5c00654e1c05a4c30beb193b9c026654c2f1`. The daemon persists ContextView before Pi, revalidates durable authorization before each body access, suppresses duplicate Pi candidate retries, and owns admission/WIA. Real SQLite Context, candidate/WIA/fence, and Pi boundary negatives passed on exact native Linux; required Ubuntu/Windows CI passed. | completed; do not reopen without a new task/lease |
| `P2-T06/D01` | `done` | `NativeWorkspaceReadExecutor` closes its bounded pre-dispatch adapter exit at `20e0e9d86ee17999bc058a7c380f7816781bbd6a`: only validated, daemon-staged `WorkspaceRead` descriptors are accepted; stale sink epochs and mismatched/stale keys fail before filesystem access; one original key serializes I/O and yields only bounded/redacted retained output. Exact native Linux passed the focused executor suite and fmt; required Ubuntu/Windows CI passed in Draft PR #162. | consumed by D02; this adapter result is not a durable Effect dispatch, progress, evidence, verification, Task completion, Gate, release, or Profile claim |
| `P2-T06/D02` | `done` | At immutable `54bd1748e6a3611503242a0f458510eb84e52f67`, real SQLite fixtures prove durable Intent/Effect dispatch records `EXECUTING` before native filesystem I/O, leaves Task state unchanged, bounds/redacts output, and records `EXECUTED`. A lost post-I/O response enters `OUTCOME_UNKNOWN`; reconciliation queries the original idempotency key, closes the Effect, and proves no second read. Native Linux focused tests (12/12), Clippy, fmt, and required Ubuntu/Windows CI passed. | consumed by D03; no process execution, mutation, Task completion, Gate, release, or Profile claim |
| `P2-T06/D03` | `done` | D02's durable Effect and reconciliation boundary is stable at `54bd1748e6a3611503242a0f458510eb84e52f67`. At `f2af668909abb70634ee2462303d3f4bc41e5600`, the private ProcessCheck adapter covers bounded target validation, supervisor registration, timeout/orphan failure, stale fencing, idempotency, redaction, output limits, real SQLite `EXECUTING`-before-supervisor integration, unknown-outcome reconciliation through the original key with exactly one supervisor access and unchanged Task state, and before/mid fault-boundary negatives. Exact native Linux focused tests (21), Clippy/fmt, and required Ubuntu/Windows CI passed. | consumed by D04; no production process-management or Task-completion claim |
| `P2-T06/D04` | `done` | At `bfcc684db6685e1077050a4b3c82fcf84c524711`, the daemon-private `DaemonProcessSupervisor` enforces registered attempt identity, PID ownership, fencing epochs, orphan/recovery/shutdown lifecycle, timeout/output bounds, injected observation, and fail-closed default behavior. Exact native Linux focused tests (26), Clippy/fmt, and required Ubuntu/Windows CI passed. | consumed by completed P2-T06 task |
| `P2-T07/D02` | `done` | daemon-private independent verifier seam reloads the durable request and fixed post-state, checks currentness and verifier identity, validates content-addressed artifact evidence references, and appends only a verification report. The new fenced-writer and verifier-identity regression tests and targeted Clippy check passed in the exact remote Linux validation run at `df7d483282f3ef0a6bbb17bae3d29bb24f13e0f7`. | consumed by P2-T07 closure |
| `P3-T01/D01` | `done` | The real owner-local Context path is task-bound through immutable ContextRequest, resolves metadata-first with tenant/scope filtering before body access/ranking, reloads durable authorization/revocation currency for each body, persists the sealed ContextView, and only then transports bounded Context to candidate-only Pi. Exact native Linux passed `cargo test -p kernel-server` and Context store 9/9 at `0ad1ddb95f4e347d0c205597e69ad8818819948e`; required Ubuntu/Windows CI passed in PR #161. | P3-T01 formal acceptance is complete; P3-T02 may consume the stable source port; B03 remains `not-run` |
| `P3-T02/D01` | `done` | Daemon-owned System/Task fragments are derived from immutable ContextRequest/TaskContract data and required before private Pi can receive a resolved view. Semantic duplicate content is rejected before ranking/rendering with `DUPLICATE_CONTENT_DIGEST` and `omitted_duplicate_content`; required fragment budget overflow returns `CONTEXT_BUDGET_EXCEEDED`. Exact native Linux `76ced01` passed Context pipeline 8/8, focused scheduler tests, full kernel-server 102/102, and kernel-server Clippy. | consumed by P3-T02/D02; no B03/UCR-01/release/Profile claim |
| `P3-T02/D02` | `done` | Source-family filtering, role-specific freshness before body loading, and source-digest verification for excluded losses are complete on the real scheduler path. Exact native Linux focused stale-source negative and Clippy passed at `0d8f5628a897aea32ee4cb7929bac1320ccb2a96`; required Ubuntu/Windows CI passed in PR #166. | P3-T02 formal acceptance is complete; retain B03/UCR-01/release/Profile as separate non-claims |
| `P3-T03/D01` | `done` | At `f626d107232e7861148e5894be52547c4d24cc99`, the daemon-owned bounded filesystem CAS publishes digest-addressed bytes and immutable metadata atomically, rejects malformed/digest-mismatched references, rejects unauthorized reads, and removes only abandoned staging writes. Exact native Linux focused test/Clippy/fmt and required Ubuntu/Windows CI passed. | consumed by completed D02; no Artifact closure, Gate, release, or Profile claim |
| `P3-T03/D02` | `done` | At `87e436e22dae3722fb0ced6c8ceeb8f0f4deddc8`, verifier evidence references resolve through D01 before append-only report persistence. Missing or digest-invalid CAS evidence fails closed and produces no report; exact native Linux focused verifier/CAS validation and required Ubuntu/Windows CI passed. | P3-T03 formal acceptance is complete; retain B03, Task completion, Gate, release, and Profile as separate non-claims |
| `P3-T04/D01` | `done` | At `3097faa91439bcb3e9b348c8f474f6184d5277b5`, a daemon-private digest-only cache key binds governance, ContextRequest, TaskContract, source, renderer, and daemon-known Tool facts. Every cache consultation repeats discovery, freshness, authorization, and body/digest validation before comparing metadata; stable/revoked negatives, exact native Linux, and required Ubuntu/Windows CI passed. | task acceptance closed; no B03, Gate, release, or Profile claim |
| `P3-T04/D02` | `done` | At `128915e15d4f4b4b98f195f0b6a49a6de76f34f2`, bounded repeat/no-progress signatures combine daemon-issued action fingerprints, durable registered error classes, and canonical evidence digests. Repeat, retry/stagnation, malformed-fact, and changed-evidence negatives passed; scheduler admission fails closed for wait/switch/block absent a daemon-owned strategy. Exact native Linux and required Ubuntu/Windows CI passed. | P3-T04 acceptance closed; no Task completion, B03, Gate, release, or Profile claim |
| `P3-T05/D01` | `done` | At `72690e028c7f3fb3896782c1874f575f35ebe165`, the fixed UCR-01 runner requires all six families, digest-pinned fixture/trace/baseline facts, and finite bounded measurements. It rejects missing fixtures, baseline drift, invalid measurement values, and Gate/release/Profile/completion/pass claims. Exact native Linux and required Ubuntu/Windows CI passed. | P3-T05 acceptance closed; no B03/B06/B07, Gate, release, or Profile claim |
| `P3-T06/D01` | `done` | At `afffb24072c78dc2f93958bc14e164f2681aea95`, a deterministic non-claim evaluator accepts only complete authorized/current/required-source/no-false-completion observations and rejects authority-shaped fields. Exact native Linux validation passed at final branch revision `96f616fb3d337b6321cc818961bc48d69f94fda8`: 11/11 Node tests, tools build, and consistency. Required Ubuntu/Windows CI passed. | consumed by D02; D01 cannot set B03 state |
| `P3-T06/D02` | `done` | ADR-0040 fixes the MVP B03 denominator at 22 Rust authority-path tests plus 11 evaluator/tooling tests. All 33 checks passed; native Linux/Clippy, cleanup/redaction, owner review, and required Ubuntu/Windows CI run `31347323835` passed at `7ea39472899e8ac77f30e589da89b7b4e0b316a2`. PR #171 merged and the lease/branch/main closure completed. | consumed by completed P3-T06 task |
| `P5-T01/D01` | `done` | At immutable `39b0e06e804947f9bd6c4a67295ffd4f4ccd3231`, the authenticated official Pi acquisition path fixes the package/version/npm origin, verifies SRI, independent SHA-256 and domain digest, dependency-lock digest, Node compatibility, adapter/sandbox/compatibility pin equality, and signed-lock reference before atomically committing daemon-private evidence. Required Ubuntu/Windows CI run `31351532417` passed; local fmt, diff, and consistency passed. | enter D02: add versioned installation-root activation, upgrade, and rollback authority; no activation, capability, Effect, Task completion, B09, release, or Profile claim from D01 |
| `P5-T01/D02` | `done` | At immutable `f63a5c8916089ac8dbcbc0be741ebd92c4fc28ef`, versioned installation-root bindings and an active pointer are transactionally persisted from committed official locks, CAS-fenced against competing activation, and rollback rejects incomplete targets without changing the active pointer. Exact native Linux focused runtime tests 6/6, store tests 7/7, Clippy, and required Ubuntu/Windows CI run `31352954426` passed. | consumed by D03; no AgentInstance, sidecar session, process supervision, Effect, Task completion, B09, release, or Profile claim |
| `P5-T01/D03` | `done` | D03 completed the daemon-private uninstall quarantine boundary at `3413598e19746807674c31b12bc7814a848edcdf`; exact native Linux focused runtime/store validation, Clippy, and required Ubuntu/Windows CI passed. | P5-T01 is closed; retain the non-claim boundary and select work only through the active P7-T04 lease |
| `P7-T04/D01` | `done` | Exact native Linux at `c2aece50ae670463d5f7949e9b9c7aefe2690c28` passed bin tests 2/2, a bounded 5-sample hypothesis run covering Context/cache/CAS/scheduler/Memory FTS5/Intent-Effect/report serialization, and Clippy/fmt. Required Ubuntu/Windows CI run `31373702405` passed at documentation HEAD `e0c8c38265ac440e1043a4b03f0eba09625fac81`. Output remains hypothesis-only. | consumed by D02; no Gate/release/Profile claim |
| `P7-T04/D02` | `done` | At implementation revision `b60538617192d87e0277ced3d557e46581096209`, `GovernedPathStageCollector` times authorization, Context resolution, cache reuse, and Intent persistence on one daemon-owned path with warm/cold and omitted-stage negatives. Exact native Linux `perf::tests` 5/5 plus Clippy/fmt passed; required Ubuntu/Windows CI run `31375051991` passed at `7738471f4a2934facfff71a50d221194b991c4b5`. | consumed by D03; no Gate/release/Profile claim |
| `P7-T04/D03` | `done` | `buildB06B07ObservationReport` records stable/changed Context reductions versus full replay with complete denominator and zero critical/false-completion safety accounting; authority-shaped claims fail closed. Required Ubuntu/Windows CI run `31376436215` passed at `d4c42e998d8d87c9b539193b6658a90a9ea4e748`. | consumed by D04/D05; observations remain non-claim |
| `P7-T04/D04` | `done` | `evaluateModuleRegressionFloor` evaluates hypothesis-only module floors, records breaches, and rejects floating-CI release gates. Supported tools tests and required CI cover positive/negative fixtures. | consumed by D05; floating CI is not release hardware evidence |
| `P7-T04/D05` | `done` | Owner-preregistered fixed-native governance A/B non-inferiority campaign on `DEV-LINUX-NATIVE-01` at measurement revision `d4c42e998d8d87c9b539193b6658a90a9ea4e748` with environment digest `sha256:8822e490dbeee6e77157cbd6813073a406912eaf6727fa84ca0858a493afbbfb` and report digest `sha256:b90b8452e5d7b833ada423fb6d9d8e6ae5db92830c22ebd2363d435e4fc4aad9`. Complete denominator 6/6, zero critical/false completions. | P7-T04 formal acceptance complete; no significant-benefit, Gate, Profile, or GMVP-LINUX claim |
| `P4-T01/D01` | `done` | At `e4eb38ad9aaba13f04fb51657dfdc884af66cdc5`, the daemon-private Memory admission path persists source-bound candidates, immutable reason-coded decisions, and admitted objects only after deterministic policy and current Context-source revalidation. Exact native Linux focused kernel/store/server tests and Clippy passed; required Ubuntu/Windows CI passed. | P4-T01 formal acceptance closed; later FTS/retrieval and lifecycle work remain separate tasks |
| `P4-T03/D01` | `done` | At `e0b2c329a209fd9f92d4068a2e96226a0cd60d6d`, daemon-private forget tombstones append immutable audit data, atomically remove FTS rows, and exclude tombstoned Memory during search/rebuild. Exact native Linux focused lifecycle/migration/FTS validation, local fmt/diff/consistency, and required Ubuntu/Windows CI passed. | consumed by D02; no public API, B08, Gate, release, or Profile claim |
| `P4-T03/D02` | `done` | At `8f9250dcd4cbcd8f15867e7a0f45165032e26c9d`, the daemon-private retention-expiry lifecycle transition enforces the exact deadline boundary, rejects duplicate sweeps, and invalidates FTS rows. Native Linux focused validation and required Ubuntu/Windows CI passed. | consumed by D03; retain public Memory API and B08 as separate non-claims |
| `P4-T03/D03` | `done` | At `8f9250dcd4cbcd8f15867e7a0f45165032e26c9d`, immutable version lineage, expected-version CAS replacement, supersede audit, and atomic derived-index movement passed the focused stale-writer/non-resurrection test plus native Clippy, exact native Linux tests, and required Ubuntu/Windows CI. | P4-T03 formal acceptance is complete; final task closure is the only remaining action |
| `P4-T04/D01` | `done` | Immutable SQLite v21 package/revision/binding facts at `4c0b9ad8429e8342ec46d609ebbcc54ffc441105` passed local format/diff/consistency and required Ubuntu/Windows CI. Unsafe local provenance, incompatible revisions, and cross-workspace bindings fail closed. | consumed by D02; retain public API/projection and Context/Task consumption as separate work |
| `P4-T04/D02` | `done` | SQLite v22 appends immutable binding revocations without erasing explain history; revoked bindings are absent from daemon eligibility reads. Exact native Linux focused tests 10/10 and Clippy passed at `3b3d3fabb2345f9fdd24278d409d4c7ff2886975`; required CI is running for that revision. | consumed by D03; public API/projection and Context/Task consumption remain separate |
| `P4-T04/D03` | `done` | SQLite v23 same-package revision supersession preserves old exact pins and rejects competing lineage; import payloads are digest-bound; revoked bindings remain explainable but absent from eligibility reads; and task bearers are rejected before local import persistence. Exact native Linux passed the Skill store tests 3/3, migration tests 8/8, daemon authorization test 1/1, and Clippy for `kernel-server` and `cognitive-store`. Required Ubuntu/Windows CI passed at `883cd5fca9b14182cc5b5632948476b31b8744a3`. | P4-T04 complete; public API/projection, Context/Task consumption, B08, Gate, release, and Profile remain separate |
| `P4-T05/D01` | `done` | Task-bound `/task/resource/v1/{projection,watch}` routes require a Task-channel bearer and a nonempty `task_ref`; management bearers are rejected before projection handling. Management projection routes remain isolated. Exact Ubuntu and Windows CI passed in run `31331192587`; local fmt, diff, and consistency passed. The projection remains daemon-private and explicitly does not fabricate Memory/Skill rows or claim B08. | consumed by D02; public contract generation, lifecycle API, B08, Gate, release, and Profile remain separate |
| `P4-T05/D02` | `done` | Authority-backed management explain routes load immutable Memory objects and Skill binding explanations through `MemoryStore`/`SkillStore`; missing IDs and Task-channel crossing fail before mutation. Exact Ubuntu/Windows CI run `31332620195` passed; local fmt, diff, and consistency passed. | consumed by D03; lifecycle command mutation and public contract boundaries remain separate |
| `P4-T05/D03` | `done` | Daemon-admin Memory forget and Skill binding revoke routes call the existing append-only authority ports; malformed payloads and Task-channel attempts fail before persistence. Exact Ubuntu/Windows CI run `31333312264` passed; local fmt, diff, and consistency passed. | consumed by D04; remember/import/bind operations remain separate |
| `P4-T05/D04` | `done` | Daemon-private remember delegates to the existing Context-revalidating Memory admission service; Skill import and binding callers delegate to immutable SkillStore authority facts. The Memory admission test double now implements the complete MemoryStore port. Exact Ubuntu/Windows CI run `31335218082` passed; local fmt, diff, and consistency passed. | consumed by D05; public contract generation, B08, Gate, release, and Profile remain separate |
| `P4-T05/D05` | `done` | Closure checkpoint `20260810-personal-p4-t05-memory-skill-api-closure.md`; D01-D05 acceptance mapping and non-claim evidence are complete. Required Ubuntu/Windows CI run `31335218082` passed; local fmt, diff, and consistency passed. | consumed by P4-T06; B08, Gate, release, and Profile remain separate |
| `P4-T06/D01` | `done` | Task consumption requires a current daemon Task contract and scheduler Context policy; retrieval scope and purpose are daemon-derived, active Skill bindings are scope/task checked, and unknown tasks fail closed before Memory/Skill discovery. Required Ubuntu/Windows CI run `31338256107` passed; local fmt, diff, and consistency passed. | consumed by D02; no B08/Gate/release/Profile claim |
| `P4-T06/D02` | `done` | The private consumption trace binds selected Memory and Skill provenance to current TaskContract and ContextRequest authority facts; revoked/ineligible Skill and forgotten/expired/stale Memory remain excluded by existing authority reads. Required Ubuntu/Windows CI run `31338813801` passed; local fmt, diff, and consistency passed. | consumed by D03; B08 remains a separate Gate campaign |
| `P4-T06/D03` | `done` | Closure checkpoint `20260810-personal-p4-t06-memory-skill-correctness-closure.md` records acceptance and explicit non-claims. Exact native Linux focused test and Clippy passed at `f4b4d38`; required Ubuntu/Windows CI passed for code run `31338813801` and closure run `31339698492`. PR #177 merged at `main@f35c2e1dfd0f8a841e009bac4c1458b0dfcfde28`; lease and branch are closed. | P4-T06 formal task complete; B08 remains a separate Gate campaign |
| `P2-T02/D01` | `done` | exact immutable Linux `734cbce` focused daemon process test passed 1/1; required Ubuntu and Windows CI passed. The authenticated task channel uses generated bindings, server-owned governance/lease facts, preview digest and admission checks, and snapshot-first bounded process-lifetime watch delivery. | select a new non-overlapping P2-T02 slice for the remaining private projection and deterministic CLI/Shell sidecar parity exit |
| `P2-T02/D02` | `done` | exact immutable Linux `70f40a5` resource projection process test passed 1/1; required Ubuntu and Windows CI passed. Private versioned projection/watch is management-bound, family/cursor-scoped, and makes unavailable authority sources explicit. | select D03 for deterministic CLI parity without creating public DTOs or fabricating domain authority |
| `P2-T02/D03` | `done` | exact immutable Linux `af2f6c9` daemon-plus-CLI process test passed 1/1; required Ubuntu and Windows CI passed. Resource projection/watch uses management credentials and Task watch uses task credentials; commands are read-only and do not replay mutations. | claim D04 Shell sidecar parity using the same isolated daemon client semantics |
| `P2-T02/D04` | `done` | exact immutable Linux `ed01c27` sidecar-to-daemon read/watch path passed; local focused TypeScript build/test and required Ubuntu/Windows CI passed. Pi uses management credentials for resource projection/watch and a distinct Task bearer for Task watch; streams require snapshot-first responses and mutations remain absent. | assess full P2-T02 acceptance against D01-D04 evidence before changing the parent task status |
| `P8-T01/D01` | `done` | axioms document, governance-rule convergence, authoritative-ledger repair, Phase 8/9 registration, TEST-ENVIRONMENTS truncation/B01 text fix, PARALLEL-LANES closed-history archive, ADR-0008/prompt/docs-sync label corrections delivered on `personal/P8-T01-doc-restructure`; documentation-only | consumed by D02 |
| `P8-T01/D02` | `done` | whitepaper Personal alignment chapter, product/architecture extensions, headroom chapter, and ADR-0041+ series delivered; documentation-only | consumed by D03 |
| `P8-T01/D03` | `done` | plan.md/trace sync, acceptance mapping, closure checkpoint, lease-mismatch failure-injection repair, and required Ubuntu/Windows CI run `31383446541` on PR #180 at `cd08da7d69890e98f1736e93215440aa85c881dc`; documentation-only | P8-T01 formal acceptance is complete; retain closure evidence |
| `P5-T02/D01` | `done` | Daemon-private Agent registration from an active official Pi installation root. Exact native Linux and required Ubuntu/Windows CI run `31389291637` passed at `bf912ee28965201a24fc35183313e5ed3dbd432e`. | consumed by D02 |
| `P5-T02/D02` | `done` | Activate registered instance into epoch-fenced SidecarSession without OS process/capability/Effect/Task claims. Same Linux suite and CI run `31389291637` at `bf912ee`. | consumed by D03 |
| `P5-T02/D03` | `done` | Pause/resume/stop plus health observation and paused/stopped recover fencing, with management-session admin-cli callers and `process_bound=false`. Exact native Linux runtime 11/11, admin 1/1, Clippy, and required Ubuntu/Windows CI run `31391916831` passed at `58ff0a723a8eae0f7fc89d9a99e9fdd55406aa92`. | P5-T02 formal acceptance complete; PR #181 merged |
| `P2-T08/D01` | `done` | Non-claim Runtime Spine Gate suite harness for fixed B02/B04/B05/B12 observations; evaluator cannot set Gate state. Tools Node tests 26/26 and required Ubuntu/Windows CI run `31395283074` passed at `38b45bd64ee898438addeab10f04d96bc80e34d3`. | consumed by D02 |
| `P2-T08/D02` | `done` | ADR-0018 local-native Provider secret exception expired: run/evaluate/extension-load fail closed; daemon-candidate + daemon Provider proxy remain. Linux focused adapter tests/Clippy and required Ubuntu/Windows CI run `31400044908` passed at `e2cee441b60774326fa224aec3fd8f779584d6f6`. | consumed by D03 |
| `P2-T08/D03` | `done` | Named Runtime Spine authority-path negatives: shell close preserves authority, daemon close recovers without duplicate dispatch, OUTCOME_UNKNOWN original-key reconcile rejects blind retry, false-completion floor rejects passed-report-as-completion. Linux focused tests/Clippy and required Ubuntu/Windows CI run `31403738305` passed at `2875978b55b5351dc89d51abd9e5397d27ea650d`. | consumed by D04 |
| `P2-T08/D04` | `done` | ADR-0046 matrix + CI at `be7febb`; owner `affirm all`; B02/B04/B05/B12 MVP pass; closure checkpoint written | consumed by completed P2-T08 task |
| `P5-T05/D01` | `done` | Process-bound SidecarSession at `c9c2248`: schema v4; Linux p5_t05 4/4 + p5_t02 11/11 + migrations 8/8 + admin 1/1 + Clippy/fmt | consumed by D02 |
| `P5-T05/D02` | `done` | upgrade/rollback/uninstall refuse process-bound; pin/digest drift refuses activation at `ae332e9`; Linux upgrade 4/4 + process 4/4 + p5_t02 11/11 + p5_t01 10/10 + Clippy | consumed by D03 |
| `P5-T05/D03` | `done` | recover/orphan + identity-separation + install≠permission at `431db73`; Linux identity 3/3 + upgrade 4/4 + process 4/4 + Clippy | consumed by D04 |
| `P5-T05/D04` | `done` | ADR-0047 matrix + non-claim report at `548f138`; CI `31423464703`; owner `affirm B09`; B09 MVP pass; closure checkpoint written | consumed by completed P5-T05 task |

This queue is the only current slice-status view. The formal plan owns slice
definitions and exits. Handoffs and the chronological evidence detail below do
not override this table.

### Layer 3 — Gate and campaign progress

| Gate/campaign | Status | Accounted progress | Missing exit |
|---|---|---|---|
| B01 | `pass` | retained `001` remains `fail` under its historical N=20 rule (2 successes / 8 failures over 10 started attempts); ADR-0039 successor `002` completed its fixed denominator of 6 counted outcomes with 5 successes / 1 failure, zero critical safety failures, complete aggregate statistics, and affirmative independent verifier disposition (see the canonical B01 row in the Area table above) | none for B01 itself; B01 pass does not transfer to G1, GMVP-LINUX, release, or Profile |
| B02/B04/B05/B12 | `pass` (MVP, ADR-0046) | fixed authority-path/harness matrix at `be7febb` + CI `31407542786` + non-claim report + owner `affirm all`; see P2-T08 closure | none for MVP scope; does not pass GMVP-LINUX, B08, B09, release, or Profile |
| B03 | `pass` (MVP, ADR-0040) | fixed 33-check matrix (22 Rust authority-path + 11 evaluator/tooling tests) with native Linux/Clippy, cleanup/redaction, owner review, and required CI; see the P3-T06 row in the Area table for the exact revision and run | none for the MVP scope; B03 pass does not cover B06/B07, UCR-01 utility, GMVP-LINUX, release, or Profile |
| B06/B07 | non-claim observations | P7-T04/D03 recorded stable/changed Context raw observations against full replay with complete denominator and safety accounting | remain observations; they do not block GMVP-LINUX and create no benefit claim |
| B08/B09 | B08 `pass` (MVP, ADR-0048); B09 `pass` (MVP, ADR-0047) | B08 matrix at `65a736c` + CI `31479512940` + disposition; B09 matrix at `548f138` + CI `31423464703` | none for MVP scopes; neither alone passes GMVP-LINUX/release/Profile |
| B10 | `not-run` | ADR-0050 fixed matrix + tools harness registered under P5-T04; Linux/CI pending | complete P5-T04/D04 matrix + disposition |
| GMVP-LINUX | `pass` (MVP, ADR-0049) | fixed composition binder at `b3f4b88` + CI `31480604511` + prior Gate MVP dispositions; see P7-T08 closure | none for MVP scope; does not claim Profile or Windows B01-W |
| Profile | `implemented: 0` | non-claim | independent applicable-MUST conformance evidence |

B01 closed on 2026-08-09/10 through the ADR-0039 successor policy; the earlier
successor-verification blocker was resolved before the campaign completed, and
its history is preserved in the closed-lease ledger and the evidence journal.
Retained campaign `001` remains `fail`; no started attempt was deleted,
renumbered, or retried, and none of its artifacts transferred to `002`.

P2-T01 is now task-complete: the L5
task lifecycle entry point over the intent-chain kernel. `proposal` durably
fixes raw intent before interpretation, `preview` emits a canonical
digest-bound contract preview, `admit` rejects preview-digest mismatch before
any kernel mutation and mints the contract under epoch CAS, `control`
supersedes to a new epoch fencing old bindings, and `query` exposes the
read-only intent projection. Merged as `main@7f763c8` (PR #127). Linux-host
evidence: P2-T01 service tests 4/4, management lib 3/3, store
`m5_intent_chain` 6/6, clippy clean, required CI green. This is
`tested-supported-ci` implementation evidence and satisfies the unchanged
P2-T01 task acceptance. P2 product acceptance (B02/B04/B05/B12) remains
not-run, so this task closure does not create a Gate, release or Profile claim.

### Chronological implementation evidence detail

The entries below retain implementation chronology. Their old "next slice" or
"remaining work" wording cannot override the three current layers above.

The previous P2 slice added the durable scheduler persistence layer (P2-T03):
`scheduler_entries` (migration v2) with lease owner/epoch/expiry,
next-eligible, attempt count and cancel flag; `SchedulerRepository` in
`crates/cognitive-store/src/scheduler.rs` provides transactional CAS lease
acquire (duplicate/cancelled refused), owner-bound release that fails closed
on mismatch, and durable cancel. Merged as `main@f3bacbe` (PR #128).
Linux-host evidence: P2-T03 scheduler tests 4/4, `cognitive-store` full
suite (migration 7/7), clippy/fmt clean, required CI green. The
timer/clock-policy and budget-ceiling enforcement that consumes this
repository was the next slice; P2 acceptance (B02/B04/B05/B12) remains
not-run.

The completed P2-T03 scheduler-service slice adds deterministic eligibility over
that repository. `SchedulerService` clamps backwards canonical wall-clock
samples to a per-worker monotonic floor, calculates a positive lease TTL, and
calls the repository's conditional acquisition path. The store only grants a
runnable entry after `next_eligible`, or reclaims an expired lease under a
strictly higher epoch, so stale workers stay fenced. The failure-first focused
test initially failed because `SchedulerService` did not exist, then passed
5/5 on the Linux host; `cognitive-store` full tests, `cargo fmt --all --
--check`, and focused runtime/store Clippy also passed. This is
`experimental-local-only` / `tested-local` implementation evidence, not a
P2 Gate, release, or Profile claim. Deadline/retry/step/cost-ceiling authority
facts and BoundedHarness worker integration remain not-run.

The follow-up P2-T03 ceiling-admission slice adds inclusive deadline, retry,
step, and cost ceiling evaluation to `SchedulerService`. The service validates
the supplied authority-fact snapshot, compares parsed deadline instants against
its monotonic wall-clock floor, and returns the first reached stop reason before
a caller starts another dispatch. The failure-first test initially did not
compile because the ceiling types and evaluator did not exist; after `fb2baa8`,
the Linux-host focused suite passed 2/2 with workspace fmt and focused Clippy.
This is `experimental-local-only` / `tested-local` implementation evidence,
not a P2 Gate, release, or Profile claim. The caller still has to reload these
facts from durable TaskContract, progress, and budget authority records and
persist the resulting stop fact before worker integration; those paths remain
not-run.

The latest P2-T03 implementation-only slice tightens scheduler release fencing
so both lease owner and epoch must match the durable row. It also adds a
daemon-private post-admission Effect-closure boundary: a ceiling STOP skips the
closure callback, and an unresolved closure retains the exact fenced dispatch
for reconciliation rather than reporting scheduler or Task success. The new
stale-release, STOP-ordering and unresolved-closure regressions were written
before the implementation. `cargo test -p cognitive-runtime --test
p2_t03_scheduler_lease_timer` and `cargo test -p kernel-server
scheduler_authority::tests` were not completed because the Windows GNU linker
returned exit 121 while linking dependency build scripts; formatting and diff
checks passed. This is implementation-only work with no new evidence level,
Gate, release or Profile claim. Durable task-to-Effect lookup, concrete worker
closure/release wiring, BoundedHarness integration and all P2 Gates remain
not-run.

The P2-T03 contract and scheduler slices merged in PR #129 as `main@7ea1cde`.
They add finite TaskContract compatibility, deadline and immutable
task-to-loop/budget bindings, scheduler ceiling evaluation, daemon-side durable
fact loading, and registered ceiling STOP edges. P2-T03 remains `in-progress`:
worker dispatch, durable stop handling, Effect closure, BoundedHarness
integration and all P2 Gates remain `not-run`.

The follow-up P2-T03 daemon authority parsing slice centralizes the current
v0.2 TaskContract parse path used by `load_scheduler_ceiling_facts`, rejecting
v0.1 rows before execution-binding deserialization and rejecting incomplete
v0.2 rows as malformed. `git diff --check` and `cargo fmt --all -- --check`
passed. The local Windows GNU linker still exited 121 before this crate
compiled. After `8d7601d` was pushed, the qualified Linux host
`wuz@192.168.1.2` cloned a disposable Git worktree at that exact revision and
the focused `scheduler_authority::tests` suite passed 2/2. The old
`/home/wuz/agent-kernel` no-Git snapshot remains invalid as test input. This is
`implementation_evidence: tested-local`; durable STOP handling, worker dispatch, Effect closure,
BoundedHarness integration, Gates, release and Profile claims remain not-run.

The current P2-T03 kernel slice adds `LoopDriver::stop_for_ceiling`: a bounded
ceiling reason can now commit the already-registered `START|CONTINUE -> STOP`
edge only after the daemon-owned writer lease, current TaskContract, latest
same-epoch checkpoint and budget ledger have been reloaded. The checkpoint
inventory rejects unresolved effects; the transition records the required
contract/checkpoint/budget evidence and prevents the normal next-iteration
path. The new focused regression was failure-first but did not compile locally
because the Windows GNU linker exited 121 while building dependencies. Workspace
formatting and `git diff --check` passed. This is implementation work only,
with no new implementation-evidence level, Gate, release or Profile claim.

The latest P2-T03 implementation-only slice adds a daemon-private release
boundary after Effect closure. It forwards a closed Effect's exact
task/owner/epoch-fenced dispatch to the durable scheduler release operation,
while a STOP or pending reconciliation bypasses release and cannot imply
scheduler or Task success. The two regressions were added before the helper;
their local Rust test attempt was blocked before crate compilation by the
Windows GNU linker (exit 121). Formatting, diff and consistency checks passed.
Linux-host validation, Gates, release and Profile claims remain not-run; the
next implementation requirement is a durable task-to-Effect lookup and the
concrete worker integration. See
`20260803-personal-p2-t03-durable-dispatch-closure-handoff.md`.

The follow-up P2-T03 implementation-only slice adds the durable task-to-Effect
authority read required before concrete worker closure wiring. `ProtocolStore`
now lists immutable Intent rows for an exact `TaskBinding` in deterministic
identity order, and the SQLite adapter reads the persisted task/epoch columns
without a schema migration. The daemon resolver rejects missing or ambiguous
bindings, binding inconsistency, missing Effects and unknown states; only
durable reconciliation/verification terminal states permit `Closed`, while
all in-flight states retain `PendingReconciliation`. The focused storage and
classifier regressions were added before the implementation. `cargo fmt --all
-- --check`, `git diff --check`, and `pnpm run check:consistency` passed;
focused Rust tests did not reach crate compilation because the Windows GNU
linker returned exit 121. This is implementation-only work with no new
implementation-evidence level, Gate, release or Profile claim. Concrete worker
lookup/closure/release wiring, BoundedHarness integration and all P2 Gates
remain not-run. See
`20260803-personal-p2-t03-durable-task-effect-lookup-handoff.md`.

The intended Linux-native validation of pushed revision
`a74ad74856b4cef6d05668acf42832ea18351b8a` did not begin: SSH host-key
verification failed (exit 255) before the remote shell could clone, checkout,
or test the revision. This attempt adds no implementation evidence and does
not change existing P2-T03 `tested-local` evidence, any P2 Gate, release, or
Profile claim. Concrete worker lookup/closure/release wiring and BoundedHarness
integration remain not-run. See
`20260803-personal-p2-t03-linux-native-task-effect-validation-handoff.md`.

The P2-T03 concrete-effect-closure-release slice adds the daemon-private
worker boundary that requires a leased task to match its exact TaskBinding,
classifies the associated Effect only from durable authority state, and calls
the scheduler repository with the exact owner and epoch only after durable
closure. A closed Effect records scheduler completion without accepting the
Task; pending reconciliation and a ceiling STOP retain their leases. The
failure-first focused regression was blocked before crate compilation by the
Windows GNU linker (exit 121); `cargo fmt --all -- --check`, `git diff
--check`, and `pnpm run check:consistency` are recorded separately by this
closure. Linux-native validation is not run because approved SSH host-key
trust is unavailable. This is implementation-only work with no new
implementation-evidence level, Gate, release, or Profile claim. BoundedHarness
integration and all P2 Gates remain not-run. See
`20260803-personal-p2-t03-concrete-effect-closure-release-handoff.md`.

## Historical evidence journal

The entries below are preserved execution-time facts. Their old blocker and
next-action language cannot override the Current snapshot above.

The implementation-only provider-contract slice inspected the exact installed
Pi `0.81.1` declarations and composer on the qualified experimental host. Pi
composes provider-model definitions into a runtime model with a required
`baseUrl`, then receives the selected runtime model through `setModel`. The
Extension previously passed provider-only metadata without that loopback URL;
the focused regression failed first and passed after the repair. Campaign `.4`
from `main@c044f2f` passed its protected workflow, was independently verified
from a SHA-256-fixed source bundle after `git fsck` and exact checkout, and was
installed after documented stale-lock recovery. The bounded redacted route
reported `status:ok`, expected reply observed, response received, 4267 ms, and
`authority_side_effects:false`. It printed no Provider, SecretRef, SQLite,
model, request, response, Task, Effect, Verification, capability, or authority
material. This is `tested-local` implementation evidence only, not B01,
GMVP-LINUX, release, or Profile evidence. The route-probe lease is closed;
`blocked_paths`: clean-Linux B01 campaign design and runner paths;
`blocked_task_ids`: `P1-T09`; `blocked_gate_ids`: `B01`, `GMVP-LINUX`, and
Profile; owner: product owner for VM allocation and credential opt-in; next
action: allocate a new clean Linux VM and name the B01 operator and independent
verifier. The closed preregistration lease records the fixed campaign contract;
no attempt has started.

Campaign `.3`'s prior 2.9-second redacted nonzero route result is superseded
as the current route fact by the independently verified and installed `.4`
result above. The former 90-second timeout remains attributed to inherited
non-TTY stdin, with the runner binding Pi stdin to `/dev/null`; the missing
runtime-model loopback `baseUrl` was the subsequent provider-contract repair.
The current atomic slice adds a reusable Linux-native Pi observation probe. It
first imports the built ESM module, then passes a session-local wrapper through
the exact Pi's explicit `--extension <absolute-path>` flag. The wrapper only
writes a disposable marker before delegating to the actual default export;
that marker was observed, confirming Pi invoked the CognitiveOS Extension entry
point. The isolated child has no daemon endpoint or Provider material, and the
expected no-daemon print-mode execution timed out after 45 seconds (exit 124).
This is `tested-local` Extension-load evidence only, not a daemon, Provider,
conversation, command-output, B01, release, or Profile result.

The current Linux-native Provider-prerequisite slice corrects the SecretStore
executable probe for the host's `secret-tool` behavior, then rechecks the
native Secret Service, current daemon endpoint, and non-secret Provider state.
An operator entered a key only through the CLI's hidden-input prompt; the
resulting `cognitive init` output confirms a redacted native-secret binding and
the selected `deepseek-v4-flash` model. A one-shot daemon-owned Provider-proxy
request returned the expected bounded response without an authority side
effect. This is `tested-local` connectivity evidence only. The direct Pi smoke
timed out and Pi remains unconfigured, so it is not a claim that the product
Pi route, first conversation, B01, release, or Profile is ready.

The current atomic slice adds a loopback-only HTTPS Provider fixture process and
an additional-root test seam that preserves the production Rustls policy. Its
failure-first integration suite covers real discovery serialization, malformed
and unauthorized responses, non-chat capability, timeout, oversized response,
redirect refusal, selected-model persistence, deterministic request counts, and
secret redaction. The exact suite passed **3/3** within the supported Ubuntu
and Windows CI workspace tests for PR #117 (run 30513254161). Local focused
test and Clippy commands remain `not-run` to completion because the Windows GNU
linker exits 121 before tests start; that limitation neither invalidates the
supported-CI evidence nor creates a Gate, release, or Profile claim.

### Earlier historical entries

> **Fail-closed Pi launch preparation slice (2026-07-30):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. `cognitive pi launch` now admits only a daemon-owned,
> numeric loopback endpoint document and an authenticated ready Personal doctor
> projection before it reads the fixed non-secret `pi.json`. It requires all
> first-conversation components (including native SecretStore, Provider and
> digest-matched selected model) to be ready; corrupt/missing endpoint or
> configuration, relative/missing Pi paths, readiness failures, and exact Pi
> `0.81.1` version drift reject launch. The spawned client receives only the
> confirmed `--extension <absolute-path>` argument and a cleared,
> OS-execution allowlist environment; it receives no Provider or secret
> material. Focused `windows_wsl2_linux_guest` admin-cli Personal units
> **15/15**, Pi/readiness **1/1**, Personal readiness **1/1**, Provider-proxy
> **2/2**, and cognitive CLI **5/5** passed; strict changed-package Clippy
> passed. This is implementation and local-test evidence only: it does not
> demonstrate a real Pi Extension load, Provider conversation, native Secret
> Service, B01, Gate, release, or Profile claim.

> **Non-secret Pi configuration slice (2026-07-30):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. Trusted public upstream source at the reviewed Pi
> `0.81.1` commit confirms `--extension` / `-e <path>` as the exact Extension
> loading option. The Personal CLI now offers `cognitive pi configure`, which
> atomically writes only the existing non-secret `pi.json` fields after
> rejecting relative paths and all non-configuration flags (including Provider
> secret inputs). It does not start Pi, access Provider configuration,
> SecretRefs, SecretStore, SQLite, or authority state; the daemon still owns
> Pi file/version readiness observation. Focused `windows_wsl2_linux_guest`
> admin-cli Personal units **9/9** passed after a failure-first relative-path
> test. This is implementation and local-test evidence only: it does not
> provide a Pi launch, Pi Extension load, deterministic binary Provider
> fixture, first conversation, native Secret Service, B01, Gate, release, or
> Profile claim.

> **Readiness truth and installed XDG launch slice (2026-07-29):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. A Provider component now becomes `ready` only when
> its non-secret `provider.json` snapshot digest has a matching valid,
> chat-capable `selected-model.json`; missing, malformed, or digest-mismatched
> selected-model state blocks both aggregate and first-conversation readiness
> with redacted local error classes. `cognitive daemon start` now leaves
> `--runtime-root` absent for installed XDG launches, so `kernel-server`,
> `cognitive init`, and the Pi extension share the real user layout; an
> explicit hermetic root is still forwarded only for tests. The CLI default is
> aligned with the canonical loopback service at `127.0.0.1:48181`. Focused
> `windows_wsl2_linux_guest` evidence: kernel-server unit suite **32/32**,
> Pi/readiness integration **1/1** each, Provider-proxy regression **2/2**,
> admin-cli Personal units **6/6**, and cognitive CLI regression **5/5**
> passed. This is implementation and local-test evidence only: it does not
> provide a Pi configuration/launch command, actual Pi Extension loading, a
> deterministic binary Provider fixture, a real Provider conversation, native
> Secret Service evidence, B01, a Gate, release, or Profile claim.

> **Provider discovery and selected-model prerequisite (2026-07-29):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. A new shared `cognitive-provider-transport` adapter
> now owns the bounded Rustls-only Provider egress boundary used by both the
> daemon proxy (through its compatibility re-export) and `cognitive init`.
> The adapter preserves HTTPS-only URLs, no redirects, URL-user-info and
> header-injection rejection, timeout/cancellation behavior, and the 1 MiB
> response limit. When Provider flags are supplied, `cognitive init` now
> configures the SecretStore binding and runs `ProviderDiscoveryService`; a
> supplied `--model-id` is `ExactCatalog`, never a manual fallback. Only a
> chat-capable probe persists `selected-model.json` and the non-secret snapshot
> digest; a failed or missing catalog model clears stale selection and reports
> a redacted actionable error. Focused `windows_wsl2_linux_guest` evidence:
> private init discovery tests **2/2 passed**, shared transport tests **2/2
> passed**, daemon proxy regression **2/2 passed**, and hermetic cognitive CLI
> regression **5/5 passed**. Strict Clippy for the changed packages and
> formatter passed. This is implementation and local-test evidence only: no
> real Provider, Pi launch, first conversation, native campaign, B01, Gate,
> release, or Profile claim is made.

> **Install-to-first-conversation XDG/endpoint foundation (2026-07-29):** The
> current first-conversation work item is `in-progress`. `kernel-server
> --personal` now resolves the real user XDG layout when its explicit
> hermetic-only `--runtime-root` is absent; it no longer creates a PID-scoped
> temporary layout for the installed user service. After a successful loopback
> bind, the daemon atomically publishes its actual bound endpoint to the shared
> non-secret `state/cognitiveos/daemon-endpoint.json` document and removes it
> during orderly shutdown. `cognitive daemon start` no longer pre-publishes an
> endpoint: it waits for the lock, bootstrap secret, and daemon-owned endpoint
> before reporting success. Focused `windows_wsl2_linux_guest` evidence:
> kernel-server Personal daemon integration **5/5 passed**, CLI daemon lifecycle
> integration **1/1 passed**, and strict Clippy passed for both crates. This is
> implementation and local-test evidence only, not Provider discovery,
> selected-model persistence, Pi launch, first conversation, B01, a product
> Gate, or Profile conformance.

> **P1-T08 Linux-native closeout (2026-07-29):** P1-T08 is `done`.
> The experimental release-shaped campaign executed the inspected shell through
> the fixed production Rust adapter and `/usr/bin/systemctl --user` on the
> designated independent Linux-native host `personal-linux-native-01`.
> Evidence covers a clean install of `0.0.0-campaign.20260729.3`, a healthy
> upgrade to `.4`, a pre-pointer failure for `.5`, and a post-pointer final
> confirmation failure for `.6`. Both failure cases returned the stable
> installer failure boundary; after the runs, the canonical unit and service
> were active, the exact 48181 liveness endpoint was healthy, and the bounded
> non-secret `active-version` pointer was restored to `.4`. Immutable `.2`
> through `.6` campaign versions remained retained. Focused WSL implementation
> tests passed **19/19** (`linux_bundle_campaign_builder`,
> `linux_bundle_service_lifecycle`, and `linux_bundle_single_service`) and
> strict runtime Clippy passed. This provides Linux-native experimental test
> evidence for the P1-T08 installer transaction only; it is not a production
> release/signing, B01, product Gate, Profile, containment, uninstall, or
> first-conversation claim. P1-T09 remains `not-started`; the next active
> work item is the install-to-first-conversation route.

> **P1-T08 MVP single-service installer transaction (2026-07-29):** P1-T08
> was `in-progress` with `development_track: experimental-local-only` before
> the Linux-native closeout recorded above.
> The inspected shell now verifies a release-bound Rust installer digest and
> hands the complete downloaded bundle to `linux-bundle-installer`. The Rust
> path shares one offline verify → OS lease → private staging prefix, publishes
> immutable bytes, atomically publishes the fixed canonical user unit, runs
> fixed `systemctl --user` daemon-reload/restart actions, checks the exact
> 48181 liveness contract before and after active-pointer publication, and
> deterministically restores the previous pointer/unit/service or removes the
> first-install unit without issuing a receipt. The adapter runner now owns
> fixed bootstrap-fact parsing, release-version verification, and controller
> injection while the production binary still creates only the fixed
> `/usr/bin/systemctl` controller; its positive transaction test uses an
> isolated controller boundary. Focused tests executed in
> `windows_wsl2_linux_guest`: **50 passed, 0 failed, 1 ignored child
> entrypoint**; runtime strict Clippy, formatting, repository consistency, and
> diff whitespace checks passed. This is implementation and fixture evidence
> only. Linux-native user-systemd, release artifact/signing, B01, Gate,
> Profile, containment, uninstall and first-conversation evidence remain
> `not-run` or not provided.

> **Personal MVP-first route decision (2026-07-29):** ADR-0034 records the
> owner-approved first production path: one canonical user service,
> `cognitiveos-personal.service`, on `127.0.0.1:48181`, with bounded downtime
> during explicit Alpha upgrades. ADR-0032/0033 dual-service fixtures remain
> valid implementation-fixture evidence and an optional future upgrade design,
> but no longer block P1-T08/P1-T09. Existing task IDs remain stable;
> `P7-T08 / GMVP-LINUX` is added as a product-only convergence Gate after B01,
> P2 and P7-T01..T03. P1-T08 remains `in-progress`, P1-T09 and P7-T08 remain
> `not-started`, all Personal product Gates remain `not-run`, and Profile
> `implemented` remains 0. This planning decision provides no single-service,
> Linux-native, B01, release, containment or Profile evidence.

> **P1-T08 fake-systemctl controller fixture (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. ADR-0033
> specifies a private/injected unit-root controller boundary and fixed
> daemon-reload, candidate start/stop, and canonical active restart actions.
> The controller renders and atomically publishes the candidate unit before a
> fixed-argument daemon-reload and candidate start; a focused Unix fake harness
> records the exact action order and confirms candidate isolation from the
> canonical unit. Focused lifecycle tests executed in
> `windows_wsl2_linux_guest`: **10/10 passed**. This is
> implementation-fixture evidence only, not Linux-native systemd, B01, Gate,
> Profile, containment, or release evidence. PR
> [#115](https://github.com/agentkernel/cognitive-os/pull/115) merged as
> `main@aa09f6c`; supported Ubuntu/Windows-MSVC push and pull-request matrices
> passed in runs
> [30382894322](https://github.com/agentkernel/cognitive-os/actions/runs/30382894322)
> and
> [30382932475](https://github.com/agentkernel/cognitive-os/actions/runs/30382932475).
> Pointer/unit/service compensation fault injection, full redaction coverage,
> and a Linux-native campaign remain separate work.

> **P1-T08 rendered user-service foundation (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. ADR-0032
> fixes two product-owned user-unit identities, disjoint loopback liveness
> ports, staged-versus-active executable paths, and the candidate-stop before
> canonical-active-start ordering. `cognitive-runtime` now renders only fixed
> candidate/active unit content, atomically publishes a fixture unit through a
> private temporary file, and rejects unsafe version/path input. The existing
> service transaction stops a healthy candidate before activation and starts
> then confirms the canonical active service after the pointer changes; failed
> flows retain deterministic compensation and never issue a receipt. Focused
> service lifecycle tests executed in `windows_wsl2_linux_guest`: **9/9
> passed**. PR [#114](https://github.com/agentkernel/cognitive-os/pull/114)
> merged as `main@b151b54` after the initial Windows path-separator failure was
> corrected in `0a90033`; its supported Ubuntu/Windows-MSVC push and
> pull-request CI matrices passed in runs
> [30379506413](https://github.com/agentkernel/cognitive-os/actions/runs/30379506413)
> and
> [30379508772](https://github.com/agentkernel/cognitive-os/actions/runs/30379508772).
> This is implementation-fixture and supported-matrix evidence only, not
> Linux-native systemd, B01, Gate, Profile, containment, or release evidence.
> A production user-systemd installation path, daemon-reload fixture, and
> Linux-native systemd campaign remain separate work.

> **P1-T08 safe-extraction slice (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. ADR-0031
> specifies a bounded, fixed-layout `tar.gz` extraction boundary. The
> implementation verifies the existing signed artifact before any lease or
> deployment mutation, then re-hashes it under the existing per-root OS lease
> and extracts only into private staging. It rejects unsafe paths, links,
> special entries, non-executable or privileged entry modes, and layouts other
> than `bin/kernel-server`; only a fully validated candidate is atomically
> published as `staged/<version>`. Extraction failure leaves the active pointer
> unchanged and creates no receipt. Focused local tests executed in
> `windows_wsl2_linux_guest`: installation **12/12**, lifecycle **12/12** with
> one ignored child entrypoint, and service lifecycle **6/6**; strict feature
> Clippy, formatting, and consistency also passed. The successful fixture
> layout satisfies static controller preflight only: the checked-in user unit
> remains unrendered and the controller still makes no systemd action. This is
> neither Linux-native systemd, B01, Gate, Profile, containment, nor release
> evidence. PR [#113](https://github.com/agentkernel/cognitive-os/pull/113)
> merged as `main@d57efc1` after both push and pull-request CI matrices passed
> on Ubuntu and Windows/MSVC. That supported-matrix evidence remains distinct
> from Linux-native systemd, B01, Gate, Profile, containment, and release
> evidence. The merge-evidence documentation commit `main@6ee68a2` also
> passed post-merge Ubuntu and Windows/MSVC CI run
> [30367954074](https://github.com/agentkernel/cognitive-os/actions/runs/30367954074).

> **P1-T08 service-lifecycle slice (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`.
> Implementation commit `26bbf12` adds a separate service-aware transaction
> that retains the existing per-root OS lifecycle
> lease across verified staging, candidate controller calls, bounded liveness,
> pointer activation/final confirmation, and deterministic compensation. The
> checked-in systemd user-unit is intentionally unrendered; the production
> controller rejects that template and the absent safe extracted daemon layout
> before any systemd action. `/personal/health` is now a small stable liveness
> response and is explicitly not readiness. Focused fake-controller/loopback
> tests passed **6/6** locally in `windows_wsl2_linux_guest`; this is neither
> real Linux-native systemd evidence nor B01, Gate, Profile, containment, or
> release evidence. PR [#112](https://github.com/agentkernel/cognitive-os/pull/112)
> merged as `main@3fc6faf` after both push and pull-request Ubuntu and
> Windows/MSVC CI matrices passed. Its follow-up merge-evidence commit
> `main@8b51018` also passed post-merge CI run
> [30360532366](https://github.com/agentkernel/cognitive-os/actions/runs/30360532366)
> on Ubuntu and Windows/MSVC. Safe archive extraction/runnable layout, real unit
> rendering, production service campaign, uninstall, signing/release material,
> and all release claims remain absent.

> **P1-T08 inspectable bootstrap/download slice (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. The new
> `deploy/linux/install.sh` is an inspectable, unrendered source template that
> fails before network access until release rendering binds its fixed version,
> HTTPS object directory, redirect host, verifier SHA-256, public keyring and
> Pi pin. Its bounded `curl --disable` download path uses private temporary
> directories, partial files, one restricted HTTPS redirect, and cleanup traps.
> A digest-authenticated `linux-bundle-verifier` adapter delegates to the
> existing offline Rust verifier only; it does not stage, activate, invoke a
> health callback, start systemd, or create authority state. Focused shell
> behavior tests passed locally in `windows_wsl2_linux_guest`; supported
> Ubuntu and Windows/MSVC push/pull-request CI passed for PR
> [#111](https://github.com/agentkernel/cognitive-os/pull/111), merged as
> `main@35115d3`, and post-merge CI run
> [30350642356](https://github.com/agentkernel/cognitive-os/actions/runs/30350642356)
> also passed. This is not Linux-native evidence. Production keys/releases,
> service health/rollback, uninstall, campaign, B01, Gate, Profile,
> containment, and release claims remain absent.

> **P1-T08 installer lifecycle lease slice (2026-07-28):** P1-T08 remains
> `in-progress` on `lane/personal-p1-t08-installer-lease` with
> `development_track: experimental-local-only`. The official
> `install_linux_bundle` entry point now completes the full offline verifier
> before creating any lease or deployment state, then acquires a stable,
> product-owned OS file lock for the canonical deployment root before opening
> that root. Lock ownership depends only on the live descriptor and OS lock:
> there is no process-local mutex, TTL, owner metadata, or stale-file
> takeover. The fixed lifecycle remains verify -> lease -> deployment open ->
> previous-version read -> verified staging -> exactly one health callback ->
> atomic activation -> active-pointer re-read and confirmation -> non-secret
> receipt. Cross-process and deterministic interruption tests cover same-root
> and cross-version exclusion, different-root independence, normal/error/panic
> and child-termination release, verifier zero mutation, staging/health/
> activation failures, every exposed fault boundary, stale lock contents,
> untorn pointers, activation-completed-without-receipt, and lease-error
> redaction. Local WSL feature tests passed **14/14** with one child entrypoint
> ignored; the complete non-feature runtime surface passed **91/91** with one
> child entrypoint ignored. Strict feature Clippy, formatting, and consistency
> checks passed. PR [#110](https://github.com/agentkernel/cognitive-os/pull/110)
> merged as `main@8aa0031` after push and pull-request workflows passed on
> both supported Ubuntu and Windows/MSVC runners. Local test results remain
> `windows_wsl2_linux_guest` evidence and are not Linux-native evidence. No
> downloader,
> inspected shell installer, systemd service, uninstall, production signing
> key/trust root, release bundle, Linux-native campaign, B01, Gate, Profile,
> containment, or release claim is added.

> **P1-T08 offline attestation verifier merge (2026-07-28):** PR
> [#108](https://github.com/agentkernel/cognitive-os/pull/108) merged as
> `main@afa1d5d` after both push and pull-request Ubuntu/Windows-MSVC CI
> matrices passed. P1-T08 remains `in-progress`. ADR-0028 now
> fixes an offline Ed25519 detached-signature mechanism over an RFC 8785 JCS
> canonical, closed attestation statement. `cognitive-runtime::linux_bundle`
> accepts only an explicitly supplied product-owned versioned keyring; unknown,
> revoked, malformed, duplicate, or bundle-selected trust roots fail closed.
> The signed statement binds product, platform, version, artifact filename and
> digest, the caller-fixed Pi version/integrity, and a strict HTTPS provenance
> reference. Metadata reads are bounded; unsafe, colliding, non-regular, and
> symlink bundle files are rejected; staging re-hashes artifact bytes to reject
> post-verification tampering before candidate creation. Focused WSL tests
> passed **14/14**, the complete `cognitive-runtime` test surface passed, and
> strict runtime Clippy plus formatting passed before the supported CI
> matrices succeeded. No
> production signing key, release attestation, downloader, inspected installer,
> systemd user service, uninstall path, Linux-native campaign, B01, Gate,
> Profile, containment, or release claim exists.

> **P1-T08 first implementation slice (2026-07-27):** P1-T08 is now
> `in-progress` on `lane/personal-p1-t08-bundle-foundation`. The first
> failure-first foundation is a local, non-downloading Linux bundle manifest
> validator plus staged filesystem activation model. It rejects tampered
> artifacts, missing/unsupported attestation references, incorrect Pi pins,
> and vendored Node/Pi payloads; interrupted staging and failed health checks
> retain the prior active version and user data, while a successful check
> atomically replaces the version pointer and retains the prior version. WSL
> focused tests passed. This is local implementation evidence only: no release
> bundle, downloader, systemd user service, trusted attestation verifier,
> Linux-native Gate, B01, Profile, containment, or release claim exists yet.

> **P1-T07 closeout (2026-07-27):** PR
> [#105](https://github.com/agentkernel/cognitive-os/pull/105) merged as
> `main@9d4c3d9` after its Ubuntu and Windows/MSVC CI checks succeeded. The
> task is now **done**: the Pi extension registers exactly one daemon-projected
> model and sends a bounded one-shot `stream:false` completion only through the
> management-authenticated daemon proxy. The extension neither receives
> Provider configuration nor secret material; the daemon proxy remains
> HTTPS-only, redirect-free, bounded, and non-streaming. P1-T07 completion is
> implementation and test evidence only. It is **not** a G0/B01-B12, Profile,
> containment, Linux-native Gate, or release claim. P1-T08 is the next planned
> task; no installer or service claim has been made.

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-27（Personal P0-T06 已完成：在 `wuz@192.168.1.2` 上实际执行 `extension-load` probe，证据记录已脱敏并核对为 `extension_command_registered=true`、`session_start_hook_observed=true`、`status_command_observed=true`、`status=executed`、`raw_output_included=false`、`output_redacted=true`、`authority_committed=false`、`effects_created=false`、`task_transitions=0`、`capabilities_granted=0`；仍是 PoC / non-claim evidence，不构成 containment、Profile 或 release claim。P1-T07 已交付 Pi runtime observation 和 daemon-owned non-streaming Provider proxy：`POST /provider/v1/chat/completions` 只接受 management bearer，Provider material 只在 daemon 内解析并仅送至 outbound request；production transport 是 daemon composition root 的 `reqwest` + Rustls，HTTPS-only、no redirects、1 MiB response bound，且 `stream:true` 稳定拒绝。ADR-0027 记录不采用 subprocess 的原因。focused provider test 目前在 Windows GNU linker exit 121 前未能执行；本 WSL instance 无 `cargo`，必须由 CI 验证。当前 pinned Pi API mirror 尚无已验证 completion-provider hook，故 Pi 未接线至 proxy，P1-T07 仍 in-progress。Owner 已批准 ADR-0018 的**默认关闭、本机 Linux、P2 到期**开发例外：adapter 仅在精确显式开关和独立 Provider config 目录存在时，从 native Secret Store 解析已配置的 DeepSeek key 后传给初始 Pi 子进程；不读取 parent env，Windows/CI/无 native backend 一律 fail-closed。该例外不构成 Pi containment、G0/B01-B12/C0/C1/Profile 或 release claim。此前完整 Windows-native 基线验证保持通过；Pi 外部 Agent 的候选执行边界已交付：Pi 0.81.1 + DeepSeek 实际 5/5 无工具 smoke，观测模型 `deepseek-v4-flash`，p50/p95/p99 = 6081/6451/6451 ms；固定 **authority=0 / Effect=0 / uncontained_candidate_only**。Lane-KRN durable InstallationStore 已合入 `main`：SQLite WAL 暂存/提交、显式崩溃恢复和跨句柄原子可见性测试已提供。Lane-RUN 现通过 in-process `DurableInstallationManager` 消费该 store；验证先于 stage/commit，recovery 仅限 manager，会话不授予 capability。`admin-cli install` 现以已认证 management session 的 `principal://` 为唯一 Custom 确认操作者，显示固定风险提示、构建确定性 `file://` bundle、拒绝无 lockfile/浮动依赖、并仅执行 `npm ci --ignore-scripts --offline`；它记录并输出 bundle/lockfile/adapter/sandbox/compatibility digests 后再 durable commit。来源/确认的**耐久查询记录**尚无 KRN store carrier，因此本批不将该 CLI 输出冒充 release evidence。该确认不是上游签名、C0/C1、Profile 或 sandbox 声明；官方供应链 verifier、Linux-native OS sandbox、lifecycle/I/O adapter 与跨进程 lifecycle lease 仍待完成。见 [PI-AGENT-INTEGRATION-PLAN.md](PI-AGENT-INTEGRATION-PLAN.md)。P0-T01 已完成：`01ceb93` 的跨平台 CI 成功，且本机 Windows GNU linker failure 已如实记录为非支持基线；不构成 Personal 产品、G0、B01-B12 或 Profile 声明。）
> **P1-T07 verification correction (2026-07-27):** the preceding status text's
> statement that this WSL instance has no Cargo is obsolete. Cargo is available
> at `/root/.cargo/bin/cargo`; the WSL/Linux focused provider-proxy process test
> and `kernel-server` strict Clippy check passed. This is focused local Linux
> evidence only, not a supported-matrix, Gate, Profile, containment, or release
> claim. Windows GNU linker exit 121 remains a non-supported local limitation,
> and supported CI (including Windows/MSVC) remains required. The unresolved Pi
> completion-provider hook keeps P1-T07 `in-progress`.

> **P1-T07 post-merge update (2026-07-27):** PR [#104](https://github.com/agentkernel/cognitive-os/pull/104)
> merged after Ubuntu and Windows/MSVC CI both passed. In addition to the
> earlier focused WSL checks, `cargo test -p kernel-server --locked` also
> passed in the WSL guest. This does not upgrade the batch into any Gate,
> Profile, containment, or release claim; it only closes the provider-proxy
> implementation checkpoint. P1-T07 still remains `in-progress` because the
> pinned Pi completion/provider integration surface is not yet verified.

> **P1-T07 completion-bridge update (2026-07-27):** the daemon now persists a
> separate non-secret selected-model projection only after a minimally-ready
> discovery probe, clears it on lifecycle invalidation and unavailable probes,
> and exposes it through management-only `GET /provider/v1/selected-model`.
> The Pi adapter registers exactly one daemon-projected model and forwards one
> bounded `stream:false` completion through the authenticated daemon proxy;
> it never receives Provider configuration or secret material. Local focused
> Rust provider/projection and Pi bridge tests, complete `kernel-server` tests,
> strict Clippy, formatting, TypeScript build/tests, and static consistency
> checks passed. This is local implementation and test evidence only: it is not
> a Gate, Profile, containment, or release claim. P1-T07 remains `in-progress`
> pending supported CI and any remaining milestone evidence.

> **P1-T07 integration-surface investigation (2026-07-27):** the exact pinned
> Pi `0.81.1` source commit documents `ExtensionAPI.registerProvider(...)` and
> a complete custom-provider streaming API. This is a supported extension
> surface, not an approved direct-provider bypass. It cannot be wired safely by
> the current batch: the checked-in structural mirror intentionally omits that
> API, the daemon proxy deterministically rejects Pi's required `stream:true`
> requests, and no authenticated, non-secret daemon model projection exists.
> The legacy provider/interception hooks are insufficient because they could
> reintroduce Pi credential/config resolution. P1-T07 remains `in-progress`;
> no Pi-side Provider config, upstream credential, environment-key fallback,
> SQLite write, or direct Provider route was added. P1-T08 installer work is
> dependency-blocked until this completion-path boundary is safely closed.

> 2026-07-24 carrier 批：KRN 已为 existing installation staging/commit record 加入 Custom source acknowledgement evidence，并以同一 SQLite 事务持久化；RUN 的 manager-only query 与 CLI 输出均读取 committed evidence。该批仍须两平台 CI，且不改变官方 provenance、Linux sandbox、Pi adapter、恢复战役、PERF 或 Profile non-claim。
> 2026-07-24 Pi P4 pre-launch admission 批（`lane/run-pi-batch1`）：新增显式 Windows-native/WSL2 拒绝，且 Linux-host admission 必须精确绑定有效 policy、sandbox adapter、compatibility digest、healthy registered adapter 及 HTTPS DeepSeek egress proxy；permit 不携带 authority/capability/Effect/Task completion，仓库未提供可启动 Pi 的 permissive adapter。WSL2 guest 诊断 runtime tests **52/52** + runtime clippy passed；Windows 本机 MinGW linker error 121，未形成 Windows test pass。没有 Linux-native evidence、F-017 扩大声明、Profile claim 或 release GO。

> 2026-07-26 V01 cross-platform evidence repair：POSIX 与 Windows 编排器均尊重
> `CARGO_TARGET_DIR`，使用完整 Rust test path 执行 PERF-004（避免 `--exact` 命中 0
> 个测试仍 exit 0），并复用 conformance runner 生成的完整 schema-shaped builder
> report；两者都验证 release-candidate manifest 的本地 evidence graph，再复制
> `performance-report-v01-sample.json`。repo-tools 新增脚本对齐测试。WSL focused
> evidence：`pi-agent-adapter` **20 substantive tests passed**，PERF-004 exact
> unit **1 passed**，
> conformance runner **85 vectors: 60 pass / 25 not-run**；均为 local/builder
> evidence，不是 Profile、release 或 measured campaign。随后受支持的 WSL/POSIX
> `verify:local` 全流程以独立 Cargo target 完成：**exit 0 / L3 /
> stopped=false / release=non_claim_preserved**；manifest、pins、self-check、F-017
> freeze 与 PERF-004 均 `auto_pass`，`profile_implemented=0`，平台标签保持
> `windows_wsl2_linux_guest`。编排器还解析 PERF exact 日志确认真实
> `1 passed; 0 failed`；该结果仍不构成 Windows-native/Linux-native sandbox、
> Profile、release 或 campaign evidence。
> Pi real-load 预检随后确认当前 guest 为 WSL2，不能使用 Linux-native secret
> exception；adapter 已在选择/probe Secret Service 前显式拒绝 WSL、Windows 与
> enabled CI，更新后的 WSL suite **20 substantive tests passed**。当前 guest 无独立
> `pi` executable，因而未解析 credential、未启动 Pi、未生成 Extension load evidence。

> 2026-07-26 P2 卡扩写批：`plan.md` 的 P2-T01..P2-T08 压缩卡已按 §11.1 扩写为
> 完整强制字段集（范围/依赖/验收语义与任务状态零变更，仅补字段、仓库锚点与
> ADR-0026/0018 等既有决策引用）。该批原停留在工作树且 §15.2 全部 not-run
> （当时会话沙箱无 shell）；已于本日随下述工具链恢复批一并落盘并真实执行验证。
> Owner 待办一次性清单见
> [20260726-personal-p2-cards-expansion-handoff.md](../checkpoints/20260726-personal-p2-cards-expansion-handoff.md)
> §5（沙箱磁盘项已随环境恢复消解；既有：`wuz@192.168.1.2` SSH、Linux-native DeepSeek
> key、干净 Linux VM）。本批非 Gate/Profile/release 声明。

> 2026-07-26 本机 Linux 工具链恢复与工作树落盘批：此前多个窗口把"无 shell /
> Windows GNU linker exit 121"当作本机不可测试的既定条件。本窗口在 WSL2 guest
> 内安装了与 `rust-toolchain.toml` 一致的 **Linux-native Rust 1.97.1**、Node
> 22.14.0 与 pnpm 10.33.2，本机因此首次可以完整执行受支持的测试面。实测结果
> （均为 `windows_wsl2_linux_guest` 本机执行，非 CI、非 Linux-native Gate 证据）：
> `cargo test --workspace --locked` **358 passed / 0 failed（67 个 suite）**、
> `cargo clippy --workspace --all-targets --locked -- -D warnings` 通过、
> `cargo fmt --all -- --check` 通过、`pnpm -r build` 通过、`pnpm -r test` 通过、
> `pnpm run check:consistency` OK（273 REQ / 55 码 / 63 schema / 85 向量）。
> 该批只改变"本机能否执行测试"这一环境事实，**不改变** 任何 Gate、Profile、
> release、G0/B01-B12 结论，也不把 WSL2 结果升级为 Linux-native evidence。

> 2026-07-26 客户端文档域仓库拆分（owner 执行）：`clients/` 整体迁出至独立仓库
> [agentkernel/cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)
> （保留 subtree 历史，外仓根对应原 `clients/` 目录）。本仓已删除 155 个
> `clients/**` 文件，所有跨仓引用改为 `blob/main/<path>` URL，并修复由此产生的
> 9 条断链；`docs/clients/`、`docs/platforms/`、`apps/cognitiveos-console/` 兼容
> stub 保留。ADR-0007、CLIENTS-DEC-001 与 2026-07-20 Lane-CON 例外作为历史记录
> 注记保留而非删除。本批不改变任何 Gate、readiness、Profile 或 release 结论。

## 里程碑状态

> 2026-07-26 evidence foundation batch：POSIX V01 pins 已与实测 `85/60/25` 和
> self-check `41` 对齐；conformance runner 现在写出 schema-valid builder sample，
> manifest validator 验证本地 result/report 引用及 digest。该批仍是 builder/sample
> evidence，不是 measured campaign；`verify:local` 现在将 manifest/evidence graph
> 校验作为 L2 必需项；Personal 后续阶段可在本机或隔离环境以
> `experimental-local-only` 开发，不改变产品 Gate、Profile 或 release 状态。

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | **done** | [20260721-m5-milestone-review.md](../checkpoints/20260721-m5-milestone-review.md) | KRN+CTR+RUN 1–2b+TSC+CFR 已合入。行为向量当时 **52 pass / 32 not-run**；F-011 三负例行为闭合；D-018 仍 partially-implemented。**GO M6**（附带条件见评审 §7） |
| M6 安装与适配、v0.1 发布 | **实现已提供；测试已执行（局部）；出口 GO-with-explicit-non-claim** | [20260721-v01-rereview.md](../checkpoints/20260721-v01-rereview.md)（初评 [NO-GO](../checkpoints/20260721-m6-milestone-review.md)） | RUN/CFR M6 交付 + EXIT 声明集/F-017 digests；当前 runner pins **60/25**（85 vectors；self-check 41/41）；RC ≤ experimental；**implemented = 0**；durable install / PERF 战役 / D-018 / Win-native / WSL2 = explicit non-claim；计划：[M6-EXIT-PLAN.md](M6-EXIT-PLAN.md) |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | 客户端项目根迁移完成（ADR-0007）；Phase 0 文档收口；M5 出口已 GO，但 implementation-ready 仍 **no (blocked)**：缺五平台 PoC / 技术栈 ADR / 依赖组 1/2/7 完整交付与法务 gate；与 M6 核心可并行 tracking-only，不混入主线 PR；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行**（嵌套独立仓；**不入** Cos `origin/main`） | Next.js 静态/SSG；Vitest / Playwright / axe 证据以 **blog 仓** 为准 | 仅研究发布与展示层；不改变 REQ/向量/Profile。**唯一路径** `personal-blog/`；远程 [`agentkernel/blog`](https://github.com/agentkernel/blog)；纪律见 `.cursor/rules/19-personal-blog-boundary.mdc` |
| Personal 产品化计划 | **P1-T08 in-progress；P0-T01..T07 / P1-T01..T07 done；无产品 Gate/Profile 声明** | P1-T07 已交付 daemon-owned Provider proxy 与 Pi completion bridge；P1-T08 已有 verifier、lease、安全解包和 dual-service controller fixture，但 production single-service installer、native systemd、完整 XDG/Provider/Pi 首聊和 B01 均未提供或未执行。ADR-0034 将 single service/48181 定为首个生产路径，新增 P7-T08/GMVP-LINUX；所有 local/WSL/fake/CI 证据继续按其原始范围记账。Personal B01-B12/GMVP-LINUX 仍 `not-run`，Profile implemented 仍为 0。 | 正式台账：[PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)；[PERS-PR trace](personal-trace.yaml) 独立于 registry matrix。Personal task `done` 不代表 product Gate 或 Profile 已符合。 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **63**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **70**（matrix 实测非空 impl；shell channel + target resolution 两批各回填 2 条后的当前值） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 33 向量**（既有 32 + **ORDINARY-CORE-AUDIT-INSPECT-001**）+ workspace Rust 项 + tracer bullet；静态执行 27 向量；**均不构成 Profile 覆盖声明**；TS **85** 项（sdk-ts 72 / agent-shell 13） |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`；RC manifest ≤ `experimental`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-23 Ordinary Core AUDIT 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **85** |
| **pass** | **60** = 静态 27 + **行为 33**（既有 32 + **ORDINARY-CORE-AUDIT-INSPECT-001**） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run** | **25**（含 MGMT-FALLBACK 其余未执行范围、shell migration、delta-scope、store-degradation disk-full 等） |
| 错误实现自检 | **41/41 corrupted 向量全部翻 fail**（新增 audit-before-release / receipt mismatch anti-pattern）；CI 地板 ≥41 |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`；报告 sha256 由 runner 打印）。层 7/8 无专属 slug = D-004 已按文档化跨切片映射闭合（conformance/README + runner `CROSS_SLICE_HOSTED`）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 0（+1 证据性质） | F-001（证据缺口，随里程碑消解，不阻断） |
| P1 | **0**（+持续） | F-017 **closed-for-release-claim-set**；F-015 持续。**F-011 已于 CFR M5 行为批闭合**；F-014/F-023 已于 M4 闭合 |
| 漂移 | **0 open**（+3 deferred/design-materialized，+1 decided/partial） | **D-022 v0.2 design/registration blocker**（AUDIT owner-authorized security/audit/compliance review 分量完成但 provenance 受限；SIG independent review、四类 machine registration、OPS member closure 与 CA-0 GO pending；继续阻断 CA-1～CA-8）；**D-017 deferred-to-v0.2**；**D-018 partially-implemented**（组装器 + watch/shell 行为证据已有；治理对象端口仍缺）；**D-016 registration eligibility NO-GO**（八项 blocked；machine contracts 未登记）；D-019 已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **Ordinary Core AUDIT vector mapping registered in joint batch** | `lane/cfr-ctr-ordinary-core-audit-inspect` | `REQ-AUDIT-001` / `002` both map to `ORDINARY-CORE-AUDIT-INSPECT-001`; matrix is fresh; no schema/candidate semantics changed |
| Lane-CFR 符合性与工具 | **Ordinary Core AUDIT vector test executed** | `lane/cfr-ctr-ordinary-core-audit-inspect` | `ORDINARY-CORE-AUDIT-INSPECT-001` pass via audited public consumer; pins **60/25**; self-check **41/41**; non-Profile claim |
| Lane-KRN 内核主线 | **durable InstallationStore 原子批已合入**（PR #78） | `main` @ `7324227` | SQLite WAL staging→commit、显式 interrupted-staging recovery、跨句柄可见性及不可覆盖负例已提供；不新增 installation transition table（D-020）。Lane-RUN local authority consumption has passed targeted verification; cross-process lifecycle lease remains undecided. |
| Lane-KRN Personal P1-T01 | **XDG layout + dual-DB prepare done（PR #92 CI green）** | `lane/krn-personal-p1-t01-xdg-migrations` | CI run 30155053950 Ubuntu/Windows-MSVC success；不改 registry/schema/vector；非 G0/Profile claim |
| Personal P1-T02 | **Provider config + SecretStore binding done（PR #93 CI green）** | `lane/personal-p1-t02-secret-provider-config` | CI run 30156079691 Ubuntu/Windows-MSVC success；ADR-0020；非 G0/Profile claim |
| Personal P1-T03 | **Provider discovery + capability snapshot done（PR #94 CI green）** | `lane/personal-p1-t03-provider-discovery-probe` | PR #94 / `main@118d20a`；CI runs 30157577277 + 30157576277 Ubuntu/Windows-MSVC success；ADR-0021；非 G0/Profile claim |
| Personal P1-T04 | **bounded daemon + timeout/concurrency done（PR #96 CI green）** | `lane/personal-p1-t04-timeout-concurrency` | PR #96 CI runs 30162481713 + 30162477963 Ubuntu/Windows-MSVC success；ADR-0022；auth/size/timeout/concurrency/restart covered；非 G0/Profile claim |
| Personal P1-T05 | **readiness/status/doctor done（PR #97 CI green）** | `lane/personal-p1-t05-readiness-doctor` | CI runs 30164114878 + 30164113787 Ubuntu/Windows-MSVC success；ADR-0023；blocked/degraded/ready + auth；非 G0/Profile claim |
| Personal P1-T06 | **cognitive CLI done（PR #98 CI green）** | `main` @ `adbb0e5` | CI run 30167503487 Ubuntu/Windows-MSVC success；ADR-0024；非 G0/Profile claim |
| Personal P0-T03 | **License/platform/distribution decision done（PR #99 CI green）** | `main` @ `fd6ff6b` | CI runs 30180002937 + 30179991223 Ubuntu/Windows-MSVC success；ADR-0025；非 G0/Profile claim |
| Personal P0-T06 | **Pi Extension fixture + real local `extension-load` evidence done** | `main` @ `a6c99d6` | The pinned fixture rejects project trust and mutating tools, and its `extension-load` mode drove a real Pi RPC session. On 2026-07-27 the designated Linux-native local experimental host produced a redacted record with registered command/session hook/status command and no authority commit, Effects, Task transitions or capabilities. It remains PoC/non-claim evidence, not a Gate, Profile, containment or release claim. ADR-0018's exact-opt-in, Linux-only local-development secret exception still expires at P2; WSL2/Windows/CI fail closed. |
| Personal P1-T07 | **Pi Extension + readiness observation + daemon Provider proxy batch delivered (in-progress)** | `lane/personal-p1-t07-provider-proxy` | The extension remains default-deny/non-authority; `pi` readiness uses real non-secret runtime observation without changing ADR-0023 aggregation. The daemon route `POST /provider/v1/chat/completions` is management-channel authenticated and resolves Provider material only inside the daemon; the production transport is `reqwest` + Rustls, HTTPS-only, redirect-free, bounded and non-streaming. ADR-0027 records why a subprocess was rejected and why `stream:true` is refused. A synthetic focused service test verifies the credential reaches only daemon-to-transport traffic; route failures are covered. Windows GNU testing is blocked by known linker exit 121 and this WSL instance has no cargo; CI remains required. Pi lacks a verified completion-provider hook, so it is not wired to the proxy and P1-T07 remains in progress. No G0/B01-B12/Profile/containment/release claim. |
| Lane-TSC TS 客户端 | **M5 HTTP/SSE 已交付**（PR #28） | `lane/tsc` | proposal/preview/submit 完整 HTTP 面增量（计划标 P2）；channel isolation 已由 RUN+CFR 补 authority 证据 |
| Lane-RUN 运行时与管理面 | **Pi P4 fail-closed pre-launch admission merged (PR #83)** | `main` @ `937e727` | Custom CLI/durable evidence baseline remains; Pi P4 additionally refuses Windows-native/WSL2 and requires Linux host + valid exact policy/adapter/compatibility digests + healthy registered adapter + HTTPS DeepSeek proxy. No Pi process/authority/Effect/Task completion path exists. WSL2 guest tests 52/52 + clippy pass; Windows local linker blocked; Linux-native evidence, official provenance, lifecycle/I/O adapter and cross-process lease remain pending. |
| Lane-DOC 文档维护 | **ADR-0015 complexity boundary accepted** | `lane/doc-product-complexity-boundary` | Ordinary Core remains the default product range; strict independent AUDIT/SIG/TARGET work is High-Assurance deferred/tracking. This changes priority only, never factual D-016/D-022 or Profile gates. |
| Lane-CON Console | tracking-only（文档域已迁出本仓） | — | 客户端文档域 2026-07-26 迁至独立仓库 [cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)；本仓只余 `apps/cognitiveos-console/`、`docs/platforms/`、`docs/clients/` 兼容 stub。M5 GO 后可复评 gate；仍缺 PoC/ADR；implementation-ready blocked |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260726-personal-p1-t07-pi-extension-package-handoff.md](../checkpoints/20260726-personal-p1-t07-pi-extension-package-handoff.md)（Personal：P1-T07 第一个原子部分 Pi Extension 包；45 TS tests passed；任务仍 in-progress；非 Gate/Profile）
2. [20260726-toolchain-recovery-and-worktree-landing-handoff.md](../checkpoints/20260726-toolchain-recovery-and-worktree-landing-handoff.md)（本机 Linux 工具链恢复、工作树两批落盘、clients 仓库拆分收口；实测 358 Rust tests passed；非 Gate/Profile）
3. [20260726-personal-p2-cards-expansion-handoff.md](../checkpoints/20260726-personal-p2-cards-expansion-handoff.md)（Personal：P2 卡 §11.1 扩写 docs-only 批；已随本日批落盘；含 owner 待办清单；非 Gate/Profile）

## 客户端目录治理交付

> **2026-07-26 仓库拆分：** 客户端文档域已整体迁出至独立仓库
> [agentkernel/cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)；
> 本仓不再包含 `clients/` 目录，也不得重建。下表是**迁出前**的交付与结论记录，
> 其后续维护责任归外仓；本仓只保留兼容 stub 与跨仓指针。readiness 结论本身未变。

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](https://github.com/agentkernel/cognitiveos-clients/blob/main/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/READINESS.md)：PoC runbook/模板与技术栈比较草案已提供（非执行/非 ADR）；M5 出口已 GO，仍 blocked 于依赖组 1/2/7 完整交付、五平台 PoC 执行、技术栈 ADR、AGPL 法务评估（POC-LIC not-run）、Tier 1 runtime PoC |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
