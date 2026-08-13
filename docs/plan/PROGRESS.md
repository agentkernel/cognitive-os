# PROGRESS — 单页进度仪表

## Current snapshot (2026-08-12)

This section is the authoritative current view. Entries below `Historical
evidence journal` preserve execution-time facts and cannot override it.

Repository identity: `cognitiveos-personal` is the only active implementation
project. CognitiveOS design, specifications, conformance assets, and reusable
kernel code are the architecture/contract foundation for Personal, not a
second product backlog. See [PROJECT-IDENTITY.md](../governance/PROJECT-IDENTITY.md).

| Area | Current status | Evidence boundary | Next actionable step |
|---|---|---|---|
| Project focus | `cognitiveos-personal`: active and sole implementation project | CognitiveOS architecture assets remain reference/contract inputs; no second product backlog | owner-directed blocker removal continues in canonical dependency order. P2-T10 executor parity, P2-T11 public correctness, and P2-T12 admitted-Task production worker loop are **done**. A `C1` WorkspaceRead can now reach durable Effect execution/reconciliation through the periodic daemon path, but it cannot yet reach genuine independent verified completion because P2-T13's production verification caller remains absent. `C2` parameter-bearing mutation request carriers remain outside P2-T12. Next is P2-T13 (`ACT -> VERIFY -> CONTINUE -> OBSERVE`); P2-T14 remains a separate task blocked on the owner/Lane-CTR `acceptance_decision` and acceptance-authority contract decision. P8-T08 is closed and P7-T07 remains blocked on B01-W prerequisites |
| Owner-directed campaign | `PERSONAL-PERF-EVAL-002` **closed and superseded; not active** — phase 3 was claimed but **no phase-3 cell executed** before the owner replaced it with a delivery instruction to remove the structural blockers the campaign found (Operating Model §2.5.4). Evaluation routing is off: continuation events resume development, not measurement | Owner scope decision 2026-08-12: evaluate the current implementation on `B01-Desktop-Linux-002`; no development tasks. Execution plan: [personal-performance-benchmark-execution-plan.md](../evaluation/personal-performance-benchmark-execution-plan.md) (revised to v1.1 on 2026-08-12 by owner instruction before any execution: overlapping families merged G 12→6 with absorbed variants, A2/A8 folded into A1/G3, per-item S/T/O authority cells consolidated into `MS-AUTH`/`T-GOV` smokes, UJ5 merged into B3, diagnostic sample sizes aligned to plan §7.1, 24 h soak made conditional; confirmatory pairs 480→270, ≤1620 paired runs, ≈55–60% of the v1.0 Provider budget; fairness/secret/safety/claim boundaries and the not-run capability register unchanged); final report target `docs/evaluation/personal-performance-assessment-<date>.md`. While this row is active, continuation events (new window, `继续`, context compression) resume this campaign per Operating Model §2.5; `CAMPAIGN-BACKLOG-CONTINUATION-01` auto-claim of `P*-T*` implementation tasks is suspended. The campaign is measurement-only: capability/runner/credential gaps are recorded `not-run`/`not_available`, never implemented mid-campaign; axioms, secret rules, guest isolation, and claim ceilings (`hypothesis`, non-claim) are unchanged | evaluation lease claimed 2026-08-12 and released at closure (writable: `docs/evaluation/`, `docs/checkpoints/`, `docs/plan/PROGRESS.md` only). Freeze + execution record: [20260812-personal-perf-eval-002-preregistration.md](../checkpoints/20260812-personal-perf-eval-002-preregistration.md). Frozen source `4cbec8470bc7a19f23f978e8754ed20133122eb1` (product code identical to the plan's reading baseline; only `tools/` differs), built on `DEV-LINUX-NATIVE-01` and digest-verified (66/66) into guest campaign root `~/perfeval002`, campaign daemon on `127.0.0.1:48282`. `B0` **pass**. `P` arm and every `O vs P` paired cell are **`not-run`**: the pure-Pi credential broker and the paired runner do not exist and Operating Model §2.5 forbids implementing them mid-campaign. `D1`/`D2` **partial**: 80/80 samples retained, all refused with `PERSONAL_PROVIDER_SECRET_UNAVAILABLE`, because Secret Service `SearchItems` (paths only, no value read) shows the Provider key item is **absent** — P9-T04 cleanup removed it while leaving `provider.json` referencing it; restoring it is owner-only graphical hidden input. Executed since: `UJ4` **pass** (30/30 admitted, 0 verified completions, admission 68.78 ms p50), `UJ3` **pass** (in-daemon reads <1 ms; `cognitive status` 70.77 ms p50; channel isolation 401/403/200 as designed), `UJ3b` **pass** (readiness attribution: `secret` probe 57.5 ms median of ~65 ms, all other components 0 ms; CLI spawn only ~6 ms), `T-GOV` **partial** (projection honest — 6 registered / 2 execution-ready; dynamic lifecycle `not_available`), `MS-AUTH` **partial** (6/6 negatives fail closed with precise codes; 0/40 positive writes admitted, all generic 409; **`skill/binding/revoke` route is shadowed by the `skill/bind` prefix match and is unreachable**), `B3` **partial** (model mismatch 10/10 denied pre-dispatch; 10/10 restart cycles with zero orphan/lock/endpoint residue; daemon-unavailable 0.23 ms refusal), `B4` **pass** (832 requests, zero errors through 33 concurrent connections; single-threaded daemon flat at 1.1–1.4 k rps; concurrency converts to queueing latency; instant recovery). `B5` 1 h soak **pass** (1620/1620, RSS +40 kB/h, zero extra writes, clean cold restart), added cell `O-LAUNCH` **pass** (Pi 0.81.1 launch to refusal 18 067 ms p50 against a 1682.5 ms bare `pi --version` and a 44.5 ms Node floor; the ~16.4 s remainder is recorded unattributed per plan §5.2), cleanup + secret scan **pass** (0 key-shaped hits in evidence/runtime; 10 repo-wide hits triaged to digit-free vendored identifiers; P9-T04 residue and the guest snapshot untouched; 0 scenario boundary violations). 3853 retained samples, no started sample discarded. Final report: [personal-performance-assessment-20260812.md](../evaluation/personal-performance-assessment-20260812.md). Top evidence-ranked findings: (1) `status`/`doctor` report all components `ready` **and `first_conversation_ready: true`** with no Provider key present, then every conversation fails; (2) `POST .../skill/binding/revoke` is unreachable — the `skill/bind` prefix match shadows it, confirmed differentially; (3) the Secret Service probe is 57.5 ms of the ~65 ms readiness evaluation while every other component is 0 ms; (4) Pi start costs 1.68 s before doing any work. Claim ceiling `hypothesis`, verifier `not_reviewed`, no Gate/release/Profile/B01/Agent-benefit promotion **Phase 2 (owner scope change, 2026-08-12):** the owner granted highest authorization to act on the phase-1 recommendations and complete all tests, which lifts the two phase-1 blockers. Lease `lease/personal/EVAL-20260812/performance-evaluation-002-phase2` claimed; phase-2 record appended to the same preregistration. The Provider credential was imported from the owner's own designated file (`~/下载/deepseek.txt`, whose lines 2–6 are the owner's saved `cognitive init` invocation fixing provider `deepseek`, base URL `https://api.deepseek.com/v1`, model `deepseek-v4-flash`) through the product's stdin path per Operating Model §2.3 — key never in argv, env, config, log, evidence or chat, and the source file is untouched. `D1`/`D2` re-ran green: 30/30 and 50/50 `complete_response`, marker and usage 100 %, Provider network 985.81/922.91 ms p50, local governance residual **127.3/128.48 ms p50** (reproducing P9-T04's 126.5/128.5 ms at a different revision), residual share 11.5–12.2 % `O1` **pass** (Pi first response 4625 ms p50 / 5006 ms p95, 30/30 — reproducing P9-T04 exactly). Campaign-only instruments built in ignored artifact roots and digest-pinned (pure-Pi broker `d29bc159…`, corpus `38e282d4…`, paired runner `6b398952…`); no product code, contract, negative, test or handbook source touched. **`B1` pilot pass** (90 pairs) and **`B2` confirmatory pass** (270 held-out pairs, 540 runs, all retained): completion `P` 240/270 = 88.9 % vs `O` 242/270 = 89.6 % (delta **+0.7 pp**, 95 % clustered-bootstrap CI [−2.22, +3.70] pp, McNemar exact p = 0.8145), paired wall delta **+1828.5 ms** (CI [1753.6, 1893.9] ms, +44.2 %), identical output size (237.5 vs 235.0 chars). Attribution ablations **exclude** Extension load (4.5 ms), streaming mode (−38.7 ms), daemon residual scaling (flat 128.1 ms on real payloads) and extra model work; ~2 s remains in the Extension's per-request path and needs nested per-run timing to confirm. Cleanup **pass**: campaign SecretStore entry cleared, broker stopped, 0 key-shaped hits anywhere, P9-T04 residue and guest snapshot untouched, 0 boundary violations. 4739 retained samples across both phases | campaign closed. The phase-1 and phase-2 leases were released on 2026-08-12; the phase-3 lease is closed in the [Parallel Lanes](PARALLEL-LANES.md) archive **with every phase-3 cell recorded `not-run`**, because the owner superseded it on 2026-08-12 with an explicit delivery instruction: remove the structural blockers that make the `C1`/`C2` capability classes unreachable (§14 capability truth, §16 Priority 6). That programme is registered as P2-T10 plus its follow-on tasks and is executed as ordinary development, not as measurement. Outstanding measurement follow-ups for the owner, unchanged: nested per-stage Pi timing (needed to close the ~2 s gap), per-request usage exposure on the `O` arm (would turn token/cost from `not_available` into measured), and the owner's Provider key sitting in plaintext at `~/下载/deepseek.txt` on the guest |
| Owner-directed campaign recovery | `PERSONAL-PERF-EVAL-002` **phase-3 recovery complete; campaign closed 2026-08-13 and not active**. Evaluation routing is off again; development resumes only on a fresh owner delivery instruction | Retained guest evidence proved the earlier “no phase-3 cell executed” closure false; the append-only recovery record is preregistration §12. At exact post-P2-T11-only revision `158e9276e49573db84aeb6ab55012d314368a76c`: recovered `B6` completed 270/270 blocks (540/540 runs) with B2→B6 completion-delta change +0.37 pp (CI [−3.33,+4.07]) and median-overhead change −91.5 ms (CI [−219.9,+31.2]) — no measurable optimization; B3/UJ2/T3/B4 cells executed or honestly dispositioned (UJ2 cold reclassified `instrument_error` for a missing `--bind`); `B5` 8 h **pass** — 480/480 minutes, 12 960/12 960 local ops with 0 non-OK, 8/8 clean hourly restarts, no cross-segment leak signature, 48/48 held-out paired blocks (96/96 runs, `P` 43/48 vs `O` 44/48, delta +2217.7 ms descriptive); `B5` 24 h `not-run` (trigger not met). Final totals: **18 735 retained samples**; cleanup **pass** (campaign daemon/broker/observer stopped, SecretStore entry cleared, 0 key-shaped hits across 18 972 files, P9-T04 residue and snapshot untouched, 159 evidence files ~1.1 MiB retained). Claim ceiling `hypothesis`, verifier `not_reviewed`, no Gate/release/Profile/B01/Agent-benefit promotion | none for this campaign — the recovery lease is closed and the final report/preregistration are reconciled; do not resume the development backlog without a fresh owner instruction |
| P2-T11 Public-surface correctness defects | **done** | Owner reauthorized development on 2026-08-12; this is the first task and it delivers evidence-ranked Priority 1 and Priority 2 from the `PERSONAL-PERF-EVAL-002` assessment. Implemented on `personal/P2-T11-surface-correctness`: `provider` readiness now derives `secret_ref_resolves` from a real resolution attempt (material dropped immediately, never in a fact), returns `provider_secret_unresolvable` / `provider_secret_store_unavailable` and blocks `overall`/`first_conversation_ready`; `skill/binding/revoke` is matched before the shadowing `skill/bind` prefix. Three failure-first negatives added: dangling ref never reads ready, unreachable backend blocks rather than assumes, and a discriminating route probe asserting `RESOURCE_SKILL_REVOCATION_ID_INVALID`. `cargo fmt --all` clean; Rust build/test is not runnable locally per `RUST-LINK-DEV-WIN-GNU-01`. **The Rust change is written but deliberately left uncommitted in the worktree on the task branch**: `docs-sync-gate --staged` correctly refuses it because the changed paths route through `handbook/_meta/source-map.json` to the `daemon-http` group, and using the `DOCS_IMPACT_NONE` escape would have been a false claim — this genuinely changes documented behaviour (a new `secret_ref_resolves` readiness fact, two new `provider_*` error classes, and a management route that now reaches a different handler). The gate reported 16 `HB008` fingerprint-drift violations across 8 pages × 2 locales: `developer/daemon-and-http`, `developer/memory-and-skill`, `reference/capability-status`, `reference/http-api`, `user/known-limitations`, `user/operations-and-recovery`, `user/what-is-personal`, `ai/validation-commands` | next, as one change set: regenerate the generated reference pages with `node tools/src/generate-handbook.mjs`, hand-update the authored pages bilingually for the new readiness fact/error classes and the now-reachable revoke route, refresh fingerprints, then commit the Rust change **together with** the handbook sync (never via `DOCS_IMPACT_NONE`, never via `--no-verify`); **Validation log (per `TEST-REPORT-INCREMENTAL-01`, appended as each unit finished):** (1) local `cargo fmt --all -- --check` **pass**; (2) `check-handbook` **pass** (54 docs × 2 locales) and `generate-handbook --check` **pass** (18 pages byte-identical) after the handbook sync; (3) `check:consistency` **pass**; (4) `docs-sync-gate --staged` **pass** legitimately (no false `DOCS_IMPACT_NONE`); (5) exact-revision native Linux `DEV-LINUX-NATIVE-01` at `7c25863323b059d2821c93bff2934393f1312d88`: `personal::readiness` **10/10 pass** including both new negatives, `p4_t05_resource_api` **1/1 pass**, `cargo clippy -p kernel-server --all-targets` **pass**, `cargo fmt --all -- --check` **pass**; (6) Draft PR [#206](https://github.com/agentkernel/cognitive-os/pull/206) opened; (7) required CI run `31609417500` **ubuntu-latest FAIL**, windows pending. Failure is **not** in the Rust change: `tools/test/check.test.mjs:226` (`owner-directed evaluation lease is accepted without a formal task slice`) injects a temporary `EVAL` lease into a copy of `PARALLEL-LANES.md` and expects `check-consistency` to pass, but the injected lease now overlaps the real active `lease/personal/P2-T11/surface-correctness` on `docs/plan/PROGRESS.md`, producing `overlapping active writable paths`. The fixture couples to live repository lease state instead of being self-contained; (8) fixture repaired by replacing the active-lease table body in its temporary copy instead of appending to the live one — the assertion is unchanged and still proves an `EVAL` lease is accepted without a formal task slice; (9) `tools/test/check.test.mjs` **10/10 pass** and the full tools suite **58/58 pass** locally | next: push and confirm both required CI jobs green on the new head, then flip PR [#206](https://github.com/agentkernel/cognitive-os/pull/206) out of Draft, merge, and run the ready/merge/lease/branch/main closure for P2-T11 |
| Active task lease | `lease/personal/P2-T13/production-verification-loop` — `P2-T13/D01` on `personal/P2-T13-verification-loop` | Claimed from clean merged `main@8918ee1182d09585f07aa81be8bb5bb23d62158a` after P2-T12 PR #209 and branch/lease reconciliation. Scope is the production `ACT -> VERIFY -> CONTINUE -> OBSERVE` path only; P2-T14 acceptance authority remains separate | add failure-first ArtifactStore composition and fixed-post-state/request/`ACT -> VERIFY` production tests, then implement D01 |
| P8-T08 Docs-sync enforcement | **done** | Closed 2026-08-12 via PR #203 (`personal/P8-T08-docs-sync-enforcement`). Owner-directed upgrade of documentation-system synchronization from guidance to an enforced obligation before every commit/push/merge. Delivered: conditional `tools/src/docs-sync-gate.mjs` (`--staged`/`--push`/`--range`; source-map routing; handbook check set on any hit; fail-closed on mapped-source changes without a handbook update; sole escape `DOCS_IMPACT_NONE="<concrete reason>"` echoed for the commit/PR record, blank/trivial reasons rejected), repo `.githooks/` pre-commit/pre-push (opt-in once per clone via `pnpm run hooks:install`), `check:docs-sync` script, docs-sync-contract v0.2 (§2 all-class handbook obligations, §5 items 16–17 machine gates, §6 checklist item), rule 10/20 + AGENTS.md checkpoint/closure wording, and bilingual handbook page sync (sync-policy, ai docs-impact/validation-commands, contributing workflow, repository map, coverage/source-map/allowlist). Validation: gate routing/negative tests 7/7 (tools suite 56/56), gate self-run green on its own staged change set, `check-handbook` + generator `--check` + `check:consistency` + `git diff --check` green; required CI on PR #203. Obligation semantics owned by docs-sync-contract; no second workflow; no Gate/contract/release surface. Closure: `docs/checkpoints/20260812-personal-p8-t08-docs-sync-enforcement-closure.md`. | retain closure evidence; enable hooks per clone with `pnpm run hooks:install` |
| P8-T07 Independent bilingual handbook | **done** | Closed 2026-08-12 via PR #202 (`personal/P8-T07-independent-handbook`, content head `f17f558`). Delivered: bilingual `handbook/` (54 documents × en/zh-CN: user 11, developer 19, reference 12, AI 6, navigation 2, plus 5 locale-neutral meta), machine model (manifest, frontmatter schema 2020-12, source-map, total source coverage, per-page fingerprints, source-set record), 9 generated reference families (CLI ×2, HTTP routes, 55 errors, config files, env vars, 5 transition domains, 74 schemas, tool catalog) with bidirectional annotation anti-rot, standalone `check-handbook` (HB001–HB015 + `--diff-base` legacy integrity) with 15/15 focused negatives, generator `--check` byte gate, fingerprint filler, `.cursor/rules/20` sync adapter, root `llms.txt` + `check:handbook` script + CI step, one AGENTS.md pointer. Reading ledger: baseline `9fbd390` full-tree classification plus the `9fbd390..6b637a5` and `6b637a5..5dd7003` increments (P9-T04, P7-T07, P2-T09 merges read before closure; source-set re-recorded at `5dd7003`). Local validation green: handbook checker, generator `--check`, diff-base proof, tools tests 49/49, `check:consistency`, `git diff --check`. Content-head CI run `31572262460` was green on every pre-existing step; the new handbook step failed only on shallow-clone HB013 (`git ls-tree` on the absent baseline object); the closure head carries the fix and its dispatch run `31573503885` passed both platforms; merge gated by PR #202 checks. Closure: `docs/checkpoints/20260812-personal-p8-t07-independent-handbook-closure.md`. The handbook is informative-only: no Gate/contract/release/Profile surface, no dynamic-status copies. | retain closure evidence; future source changes follow the rule-20 sync adapter and `handbook/_meta/sync-policy.md` |
| P7-T07 Windows install surface | **blocked** | D01-D03 are complete and validated: Windows Credential Manager production backend (fixed audited PowerShell helper from the absolute system path, secrets only on helper stdin/stdout, local-only persistence, 2560-byte blob ceiling, fail-closed selection, no plaintext fallback) with failure-first negatives including the real Credential Manager roundtrip; inspectable bootstrap installer + least-privilege per-user scheduled-task templates with behavioral fail-closed negatives; and the authored B01-W gate (ADR-0052, preregistration `20260812-personal-p7-t07-b01-w-preregistration.md`, required-not-provisioned `B01-W-DESKTOP-001`). Required CI `31570126985` passed both platforms at `13e772a` on Draft PR #200. `blocked_gate_ids: B01-W`; prerequisites: Windows release artifacts (release-pipeline scope decision), `B01-W-DESKTOP-001` provisioning, operator for graphical hidden-input credential entry. Blocked-closure record `20260812-personal-p7-t07-windows-install-closure.md`. No install parity, B01-W, Gate, release, or Profile claim | owner: provide the three prerequisites or an explicit B01-W scope disposition, then execute the preregistered campaign and close D04 |
| P2-T09 Tool execution readiness projection | **done** | Optimization-proposal INV-TASK-1. `tool_execution_readiness` derives readiness from the assembled executor set (`WorkspaceRead`, `ProcessCheck`) instead of registry availability, and the daemon-private projection reports both facts separately. Readiness stays out of the immutable descriptor and its digest, proven by a dedicated negative. Exact native Linux `cbb830a2dc64855f51c4b5ffdf04ae4f6fba4e4c`: tool_registry 11/11 with four new negatives, Clippy for `cognitive-kernel` and `kernel-server`, tool_executor unchanged at 27/27. Required Ubuntu/Windows CI passed at `0227b97`; PR #201. No new Tool family, no Gate/release/Profile claim, no P9-T04 evidence inherited. | retain closure evidence; executor parity is P2-T10 |
| P2-T10 Registered Tool family executor parity | **done** | Closed 2026-08-12 via PR [#207](https://github.com/agentkernel/cognitive-os/pull/207), merged at `main@3f766020c4d822556887ff8af59d41ed0cb92d75`. Removes gap 3 of the four wiring gaps on `handbook/*/developer/execution-chain-status.md`: the four families the campaign measured as `registered_only` now have real sinks. `WorkspaceSearch` enforces containment twice (canonicalized-root check plus a per-entry `symlink_metadata` skip) under fixed visit/match/file-size/line ceilings. The `WorkspaceWrite`/`WorkspacePatch` sink is an expected-preimage compare-and-swap — validation refuses a mutation that does not declare what it replaces — publishing through a staging file, `sync_all` and rename, with `query_outcome` re-reading the target so reconciliation comes from durable filesystem state rather than process memory; the patch applier fails closed on drifted context, unknown prefixes, overlapping or out-of-order hunks and self-contradicting headers. `HttpFetchReadOnly` runs over the repository's single audited Rustls boundary, kept separate from Provider egress so it cannot inherit that authorization: no caller headers, no body, no redirects, no inherited proxy, bounded deadline, oversize refused rather than truncated. `ASSEMBLED_EXECUTOR_FAMILIES` now lists all six and a parity negative fails the moment it names a family no sink accepts. **Exact native Linux `8979749e9efa2fcb9bef8f6be47f0ea76e5d1751`: `tool_executor` 54/54 (was 27), full `kernel-server` package integration tests pass, `cognitive-provider-transport` 10/10 including four real loopback-TLS negatives, `tool_registry` 11/11, fmt and Clippy clean for all three packages. Required Ubuntu/Windows CI passed on closure head `effae9a` (run `31617885666`); the first Windows attempt failed and was confirmed flaky by re-running the same revision — `p1_t05_personal_readiness` allows the daemon only 2 s to publish its bootstrap secret and the runner took ~2.2 s.** `execution_ready` means only that this binary contains a sink for the family; it is **not** a claim that an Agent can reach one. Gaps 1, 2 and 4 are unchanged and are registered as P2-T12 and P2-T13. Closure: [20260812-personal-p2-t10-tool-executor-parity-closure.md](../checkpoints/20260812-personal-p2-t10-tool-executor-parity-closure.md). No Gate/release/Profile claim | retain closure evidence; `C1`/`C2` remain unexecutable until P2-T12 and P2-T13 land |
| P2-T12 Admitted Task reaches the worker loop | **done** | D01-D05 satisfy the unchanged acceptance at exact post-review `06c6a6f2cff96a33b8ac9e9f1be5c8a7a58f0758`. Admission/startup repair publish the scheduler/Loop/Budget bootstrap; zero-Intent rows reach candidate admission with row isolation; the post-bind worker is single-owner/non-reentrant/cancellable; and production WorkspaceRead reloads persisted descriptors/current authorization, stages before WIA consumption, persists `EXECUTING` before I/O, reconciles the original key, and releases only the exact lease. Native scheduler authority 50/50, server 9/9 and Clippy passed; required Ubuntu/Windows CI `31687579721` passed. No Task completion, verification, Gate, release, Profile, or B01 claim | merged delivery unblocks P2-T13 independent verification production caller |
| P2-T13 Independent verification runs in production | **in-progress** | Claimed 2026-08-13 after P2-T12 merged. D01 targets daemon ArtifactStore composition plus production persistence of FixedPostState/VerificationRequest and Loop `ACT -> VERIFY`. Existing consumption of verified continuation remains reused; no Task completion or P2-T14 authority is in scope. Local Rust execution is `not-run` under the registered Windows GNU restriction | author failure-first production D01 tests, push exact red, then implement the ArtifactStore and `ACT -> VERIFY` caller |
| P2-T14 Verified Task completion and acceptance authority | **not-started** | Registered 2026-08-12, split out of P2-T13 because it is not wiring work. `ACTIVE -> CANDIDATE_COMPLETE` and `CANDIDATE_COMPLETE -> COMPLETED` have **no production implementation anywhere** in `crates/` or `apps/`; only tests and the conformance runner construct them. Engine guards are caller-supplied strings (`engine.rs:189`/`:586`), so the three acceptance guards must be genuinely derived — one of them, `verification_passed_and_current`, is annotated `// underivable` in an existing test. Most importantly the transition requires `acceptance_decision` evidence for which **no Rust type, schema binding, table or row exists**, and there is no acceptance-authority principal concept; both are owner + Lane-CTR normative decisions, so the task will sit `blocked` on that decision rather than substituting a private structure. **Governance observation for the owner, not acted on:** P2-T07's plan card requires per-criterion `pass/fail/unknown` with evidence digests, but the delivered `VerificationReportRow` carries a single whole-report `status` and a flat evidence array; whether to reconcile that is the owner's call and P2-T07's `done` status is left untouched | owner decision on the `acceptance_decision` contract and the acceptance-authority principal; then `P2-T14/D01` |
| P9-T04 Comprehensive performance and real Task campaign | **done** | Closed 2026-08-12 as a non-claim report: `docs/checkpoints/20260812-personal-p9-t04-performance-campaign-closure.md`. `L0`–`L3` executed in full (160 retained Provider samples across six cells on `B01-Desktop-Linux-002` against real DeepSeek), `L4` `T1` admission partial at 10 retained / 8 admitted, `L5` closed `not-run` by owner disposition. All eight safety counters zero; claim ceiling `hypothesis`; `verifier=not_reviewed`; no Gate, release, Profile, B01 or Agent-benefit claim. Headline hypothesis-only findings: governance plus loopback overhead is stable at 126.5–128.5 ms p50 while Provider latency swings 898.9–1016.1 ms p50, and Pi process spawn adds ~3.5 s over the same call made through the daemon client. Historical detail: ADR-0051 and preregistration `20260812-personal-p9-t04-performance-campaign-preregistration.md` are registered. D01's exact native Linux test passed 4/4 at `47629c40217ad039aededcb6a91207fe83f15b7a`; local fmt, consistency, and Pi client build/test passed. Provider-network timing integration is on `c40016042035af27f5a1c16d59f34100d535207e`; required Ubuntu/Windows CI `31551730435` passed. Correlation caller checkpoint `b67f8e4` adds a generated `campaign-` correlation ID to the real Pi-to-daemon Provider request and keeps it out of the request body; required Ubuntu/Windows CI `31553062959` passed at `4074c0a51eb5fede730d74559e5d35bc659941d3`. D02 adds the executable L0/L1 runner whose admission refuses secret-shaped process input and whose report rejects unredacted or self-promoted observations; its negatives passed 12/12 with Clippy on exact native Linux `ad800ca` and required Ubuntu/Windows CI passed at the same revision. D03 decomposes the real loopback front door into disjoint transport stages that explicitly disclaim `effect_persistence`, `provider_network`, `pi_process_launch`, and `scheduler_wait`; transport 7/7, kernel-server front door 7/7, and Clippy passed on exact native Linux `43eb434`. D04 adds the bounded resource sampler that never opens `cmdline`/`environ`, never resolves a descriptor target, and treats a decreasing cumulative counter as PID reuse; 10/10 plus Clippy passed on exact native Linux `ad3c017`. D05 makes the ADR-0051 campaign policy executable: full L0–L5 coverage, retained denominators, hard safety accounting, cleanup facts, and a claim ceiling that a safety failure or missing verification forces back to `non-claim`; 10/10 negatives, Clippy, and the full 148/148 runtime suite passed on exact native Linux `40c5d82`. D06 executed one L0–L2 campaign on `DEV-LINUX-NATIVE-01` at exact pushed `ba14183` with report digest `sha256:c18a63db...e83be`: 1/200/52 started and retained samples for L0/L1/L2, `L3`–`L5` `not_run`, all seven safety counters zero, complete cleanup, `hypothesis` claim and `benefit_claimed=false`. D07 adds the executable L3 route policy (`retry=0`, complete outcome denominator, no fabricated TTFT or cost) with 9/9 negatives, Clippy, and required CI `31558942939` at `5e04b1c`. D08 and D09 add the L4 governed-Task scenario harness: a frozen oracle plus independent acceptance decides each outcome, a read-only scenario that mutated anything is a boundary violation that outranks every other result, and all seven hard safety counters are reachable from a scenario judgment. Runtime lib suite 168/168, Clippy, and required CI `31559903103`/`31560590193` passed at `099a079` and `c8d047f`. The next code slice D10 defines the `T2`-`T8` scenarios. L3/L4 execution remains owner-gated on the B01 start gate plus a graphical hidden-input Provider import, and L5 remains blocked on the A-arm baseline decision recorded in the execution plan. Provider usage/cost remain fail-closed (`not_available` unless complete counters exist; cost unavailable), and loopback plus daemon Provider-network durations remain separate. B01 guest remains shut off and no Provider import, guest mutation, benchmark execution, Gate, release, Profile, or Agent-benefit claim has occurred. | validate the D02 runner negatives on exact native Linux and required CI, then decompose transport stages out of Effect persistence |
| P9-T01 Async event foundation decision gate | **done** | D01's exact native Linux observation at `826745c` passed `perf::tests` 5/5 and selected hypothesis-only `conservative-no-migration`. Recovery added bounded socket reads and serialized loopback-daemon test execution after a Windows stall. Final required Ubuntu/Windows CI `31540218963` passed on merged PR #197 head `498852f`; merge commit `af64b85`. No async migration, Gate, release, or Profile claim. | retain closure evidence; any reconsideration needs separately measured HTTP/watch/sidecar transport evidence |
| P5-T04 Post-1.0 dynamic Tool + B10 | **done** | D01–D04 closed under ADR-0050. Linux 4/4 + Clippy at `b49d274`; required CI `31486478177` on `992dfe3`; B10 MVP `pass`. Closure `20260811-personal-p5-t04-dynamic-tool-closure.md`. | retain closure evidence; no GMVP-LINUX/release/Profile claim |
| P5-T03 Post-1.0 MCP Tool adapter | **done** | D01–D04 closed; Linux 4/4 + Clippy at `a83bdb8`; required CI `31482773002` on `4c06161`. Closure checkpoint `20260811-personal-p5-t03-mcp-tool-adapter-closure.md`. | retain closure evidence; no B10/Gate/release/Profile claim |
| P7-T08 Public Linux 1.0 Gate (`GMVP-LINUX`) | **done** | D01–D04 closed; B08 MVP `pass` (ADR-0048) at `65a736c` + CI `31479512940`; GMVP-LINUX MVP `pass` (ADR-0049) at `b3f4b88` + CI `31480604511`. Closure checkpoint `20260811-personal-p7-t08-gmvp-linux-closure.md`. | retain closure evidence; no Profile/Windows B01-W claim |
| P2-T15 Independent-review executor hardening | **done** | Owner-directed corrective delivery after merged P2-T10/P2-T11. D01-D03 close all confirmed review defects with failure-first Linux/Windows negatives, durable original-key state and receipt isolation, exact catalog equality, handle-relative no-follow workspace I/O, final-window CAS, bounded enumeration/preimages, orphan recovery, final-newline patch semantics and one-snapshot readiness. Required CI `31666003044` passed at exact `580c0a06d39ee3d6fb460e23be9c7ac0939a4b63`; exact native Linux passed Tool executor 76/76, readiness 11/11, full kernel-server 179/179 plus integrations, full workspace Rust tests, Clippy and fmt; final review reports no findings | closure: `docs/checkpoints/20260813-personal-p2-t15-executor-hardening-closure.md`; merge PR [#208](https://github.com/agentkernel/cognitive-os/pull/208) normally and retain non-claims |
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
| 72 | 63 | 1 | 1 | 7 | 9 |

`P9-T04` is `done` (PR #199), closed as a non-claim report. Formal task
completion remains independent from GMVP-LINUX, release, Profile, and Windows
B01-W claims, and this campaign adds no Gate, B01, or Agent-benefit claim.
`P8-T07` (independent bilingual handbook, PR #202) is `done`: documentation and
tooling only, no Gate, contract, or release surface. `P8-T08` (docs-sync
enforcement, PR #203) is `done`: governance/tooling only — documentation
synchronization is now an enforced pre-commit/pre-push/pre-merge obligation.

### Layer 2 — Current Delivery Slice queue

| Slice | Status | Actual evidence boundary | Executable next action |
|---|---|---|---|
| `P2-T10/D01` | `done` | `NativeWorkspaceSearchExecutor` assembled with double containment (canonicalized root check plus per-entry `symlink_metadata` skip) and fixed visit/match/file-size/line ceilings. Eight new focused negatives: unstaged and digest-drifted dispatch never scan, empty and oversized queries are refused at the sink, a symlink planted in the tree is never traversed and an escaping search root is refused after canonicalization, ceilings truncate and redact before retention, a stale fencing epoch refuses before I/O, `EXECUTING` is durably recorded before the first filesystem read without advancing the Task, and an OUTCOME_UNKNOWN reconcile resolves through the original key with `scan_count` still 1. **Exact native Linux at `ed0bea98792fa85a46183d26a22959b83127cde1`: `cargo test -p kernel-server tool_executor` 35/35 pass (27 → 35), `cargo test -p cognitive-kernel tool_registry` 11/11 pass, `cargo fmt --all -- --check` pass.** Readiness deliberately unchanged, so `workspace_search` still projects `registered_only` until D04 | consumed by D02 |
| `P2-T10/D02` | `done` | `NativeWorkspaceMutationExecutor` covers `WorkspaceWrite` and `WorkspacePatch` behind one boundary: validation now refuses a mutation that does not declare its expected preimage, the sink verifies that preimage before building the postimage and again immediately before the rename, publication is staging-file + `sync_all` + rename, and `query_outcome` re-reads the target so reconciliation works from durable filesystem state rather than process memory. The patch applier fails closed on drifted context, unknown prefixes, overlapping or out-of-order hunks and headers that miscount their own body. Retained output is a bounded receipt (target, action, byte count, postimage digest) and never file content. Ten new focused negatives: preimage mismatch leaves the target and directory untouched, the atomic-publish hook observes the postimage staged while the target still holds the preimage, a concurrent writer that wins the pre-rename race is not clobbered, a symlinked target and an escaping parent are both refused, duplicate dispatch publishes exactly once, three patch-context failures, a fresh executor resolves the original key from the file alone, stale fencing refuses before any write, and a real-SQLite `EXECUTING`-before-mutation plus OUTCOME_UNKNOWN reconcile with `publish_count` still 1. **Exact native Linux at `be4aaa471338a5c07ed1de7c28d8747e2c0569a9`: `cargo test -p kernel-server tool_executor` 45/45 pass (35 → 45), `cargo test -p cognitive-kernel tool_registry` 11/11 pass, `cargo fmt --all -- --check` pass, `cargo clippy -p cognitive-kernel --all-targets` clean; `cargo clippy -p kernel-server --all-targets` raised one `type_complexity` warning on the test hook, fixed by a type alias in the next revision** | consumed by D03 |
| `P2-T10/D03` | `done` | `cognitive-provider-transport` gains a read-only fetch boundary beside the Provider one — no caller headers, no body, no redirects, no inherited proxy, bounded deadline, bounded response refused rather than truncated — kept separate so a Tool fetch can never inherit Provider authorization. `NativeHttpFetchReadOnlyExecutor` restates no network policy: staging calls the registered `validate_read_only_http_fetch` validator, so the origin allowlist, HTTPS requirement, absent userinfo/query/fragment and timeout ceiling come from one source. Outcome classification is deliberate: policy refusal is `NotExecuted` (never left the process), timeout and transport fault are `Unknown` (may have reached the origin), oversize is `NotExecuted` with nothing retained. GET only in this MVP; `HEAD` is supported by the transport but has no registered parameter channel and no caller. Seven executor negatives plus four real loopback-TLS negatives. **Exact native Linux at `11932359911e822b5ebf95fed1c7c68994f4ce88`: `cargo test -p kernel-server tool_executor` 52/52 pass (45 → 52), `cargo test -p cognitive-provider-transport` 3 unit + 3 `p1_t09` + 4 new `p2_t10_read_only_fetch` all pass (real HTTPS GET with `authorization=absent`, 302 not followed, oversize refused, deadline honoured), `cargo test -p cognitive-kernel tool_registry` 11/11 pass, `cargo fmt --all -- --check` and Clippy for all three packages clean** | consumed by D04 |
| `P2-T15/D01` | `done` | All workspace/native-sink hardening is present behind focused failure-first negatives: handle-relative no-follow search/mutation, opened-handle reparse/type checks, active file/directory/parent swaps, per-target CAS lock plus exact final-window recheck, every immutable descriptor field drifted across all six families, enumeration-time visit ceiling, streamed write preimages, explicit patch preimage ceiling, cleanup-failure/orphan recovery and old/new no-final-newline semantics. Required Ubuntu/Windows CI is green through `5da160b` | consumed by D02 |
| `P2-T15/D02` | `done` | Durable mutation/HTTP state binds original keys across restart, same-postimage competition, later reversion and state loss; stable seen-key witnesses prevent missing records being recreated as fresh non-execution; mutation state is workspace-external and state-root creation is no-follow; readiness resolves the secret from one loaded config snapshot. Bilingual mapped docs and final no-findings review are complete; required CI `31664427142` passed at `5da160b` | consumed by D03 |
| `P2-T15/D03` | `done` | Required CI `31666003044` passed at exact `580c0a0`; exact `DEV-LINUX-NATIVE-01` at the same revision passed Tool executor 76/76, readiness 11/11, full kernel-server 179/179 plus integrations, full workspace Rust tests, kernel-server Clippy and fmt; disposable worktree removed; closure record and final no-findings review complete | task accepted; merge PR #208 and complete branch/main cleanup |
| `P2-T12/D01` | `done` | At exact pushed `0e7c76b933c46a5d9dc649bd616cecc04b4b6d58`, native admission **8/8**, startup-repair **2/2**, scheduler-authority **41/41**, and five-package all-target Clippy pass; required Ubuntu/Windows CI run `31672335888` passed both jobs. Production admission atomically publishes TaskContract/event + contract-named `START` Loop/event + hard Budget + current-epoch `Runnable` scheduler row under fencing and epoch CAS. Startup recovery idempotently repairs only missing current-contract Loop/Budget/scheduler members without replacing existing authority. Negatives cover unadmitted absence, crash/reopen, duplicate admission/recovery, late-member rollback, and missing-only repair. Local Rust is `not-run` by the registered Windows GNU restriction | consumed by D02; no execution, verification, completion, Gate, release, or Profile claim |
| `P2-T12/D02` | `done` | Zero-Intent work now resolves to pre-admission, persists and admits the candidate, and leaves its WIA for a later pass; row-local failures are logged/skipped without aborting later rows. Exact native `01ef6c831186a0c815a06f498f2bf348508a25a9` passed scheduler authority 43/43 plus kernel-server all-target Clippy; required Ubuntu/Windows CI run `31675250046` passed both jobs | consumed by D03; no executor I/O, verification, completion, Gate, release, or Profile claim |
| `P2-T12/D03` | `done` | Post-bind fixed-delay worker owns the scheduler repository, rejects self-reentry, continues after pass errors, isolates row failures, and cancels/joins on orderly exit. Exact native `4f0d625b5e4476cfc1956088134b9a4e1afd3e08`: server 9/9, scheduler authority 43/43, kernel-server Clippy; required Ubuntu/Windows CI `31677480018` passed | consumed by D04; no Tool I/O, verification, completion, Gate, release, or Profile claim |
| `P2-T12/D04` | `done` | The daemon publishes stable immutable native descriptor IDs, exposes only contract-allowed IDs to Pi, derives candidate and dispatch authorization from current facts, and production-dispatches WorkspaceRead through the existing Effect protocol. Exact `06c6a6f2cff96a33b8ac9e9f1be5c8a7a58f0758` passed native scheduler authority 50/50, server 9/9, and Clippy. Negatives cover unassembled/unsupported family before WIA, stale/replaced lease, `EXECUTING` before I/O, same-process original-key reconcile, process-restart zero redispatch, STOP/budget ordering, and no Task completion | consumed by D05; final required CI remains task-closure evidence |
| `P2-T12/D05` | `done` | Formal acceptance maps to D01-D04 implementation and negatives; final exact native `06c6a6f2cff96a33b8ac9e9f1be5c8a7a58f0758` passed scheduler authority 50/50, server 9/9 and Clippy; required Ubuntu/Windows CI `31687579721` passed; bilingual handbook, report, and closed lease archive are synchronized | task accepted; ready/merge PR #209 and reconcile branch/main |
| `P2-T13/D01` | `in-progress` | Failure-first production composition begins on merged P2-T12 main with the existing verifier/store seams unchanged | compose `ArtifactStore`, pin `FixedPostStateRow` + `VerificationRequestRow`, and commit Loop `ACT -> VERIFY` |
| `P2-T13/D02` | `ready` | not started | after D01: derive criteria from the contract's `Acceptance` conditions, add a production `IndependentVerifier` and its `verifier_ref` registry, and call `record_independent_verification` |
| `P2-T13/D03` | `ready` | not started | after D02: emit the loop checkpoint and issue the continuation authorization, closing the loop through the already-wired consumption path |
| `P2-T13/D04` | `ready` | not started | after D03: acceptance mapping and closure, recording that zero Tasks complete until P2-T14 |
| `P2-T14/D01` | `ready` | not started | obtain the owner and Lane-CTR decision on the `acceptance_decision` evidence object and the acceptance-authority principal, then register the resulting contract |
| `P2-T14/D02` | `ready` | not started | after D01: implement both acceptance transitions with the three guards genuinely derived rather than asserted |
| `P2-T10/D04` | `done` | `ASSEMBLED_EXECUTOR_FAMILIES` now lists all six registered families, so the daemon-private projection reports each as `execution_ready`. Two new negatives keep that honest: a parity test stages a real request of every listed family through whichever sink owns it and fails the moment the list names a family no sink accepts, and a derivation test re-proves that dropping the assembled set makes every enabled descriptor read `registered_only` again while every descriptor digest is unchanged. The code comment, the tests and both handbook locales now state the same narrow meaning: readiness says this binary contains a sink, not that an Agent can reach one. **Exact native Linux at `8979749e9efa2fcb9bef8f6be47f0ea76e5d1751`: `cargo test -p kernel-server tool_executor` 54/54 pass (52 → 54), full `cargo test -p kernel-server` package integration tests pass, `cargo test -p cognitive-provider-transport` 10/10 pass, `cargo test -p cognitive-kernel tool_registry` 11/11 pass, `cargo fmt --all -- --check` pass, Clippy clean for all three packages.** Acceptance mapping and the honest non-claims are in [20260812-personal-p2-t10-tool-executor-parity-closure.md](../checkpoints/20260812-personal-p2-t10-tool-executor-parity-closure.md); PR [#207](https://github.com/agentkernel/cognitive-os/pull/207), merged at `main@3f76602`; required Ubuntu/Windows CI passed on run `31617885666` | consumed by the completed P2-T10 task |
| `P2-T11/D01` | `done` | Both `PERSONAL-PERF-EVAL-002` Priority 1/2 defects fixed on `personal/P2-T11-surface-correctness`: `provider` readiness derives `secret_ref_resolves` from a real resolution attempt and blocks on `provider_secret_unresolvable` / `provider_secret_store_unavailable`; `skill/binding/revoke` is matched before the shadowing `skill/bind` prefix. Three failure-first negatives added (dangling ref never ready, unreachable backend blocks rather than assumes, discriminating revoke-route probe). `cargo fmt --all` clean; Rust build/test not runnable locally per `RUST-LINK-DEV-WIN-GNU-01` | push the branch and run the focused Rust validation on exact-revision `DEV-LINUX-NATIVE-01`, then open the Draft PR and required CI |
| `P8-T08/D01` | `done` | gate landed at `539922b`: `--staged`/`--push`/`--range` modes, source-map routing, conditional handbook check set, fail-closed unsynced-mapped-change verdict with explicit-reason escape; `.githooks` pre-commit/pre-push (mode 100755), `hooks:install` + `check:docs-sync` scripts; routing/negative tests 7/7 (tools suite 56/56); real-repo smokes: skip on unrelated range, full check set on handbook range, staged self-run green | consumed by D02-D03 |
| `P8-T08/D02` | `done` | docs-sync-contract v0.2 (§2 all-class handbook obligations with enforced timing, §5 items 16-17, §6 checklist); rule 10 checkpoint obligation bullet, rule 20 mandatory-timing items, AGENTS.md checkpoint + closure-protocol wording; handbook synced bilingually (sync-policy layers, ai docs-impact/validation-commands, contributing workflow, repository map, `.githooks` coverage rule, `docs-sync-enforcement` source-map rule, allowlist) with fingerprints refreshed | consumed by D03 |
| `P8-T08/D03` | `done` | acceptance mapping + validation record in `docs/checkpoints/20260812-personal-p8-t08-docs-sync-enforcement-closure.md`; local check set green; required CI via PR #203 checks at the merge head | task closed; ready/merge/lease/branch/main reconciliation |
| `P8-T07/D01` | `done` | machine core landed at `f17f558`: `handbook/_meta` model files, `check-handbook` (HB001–HB015), generator, fingerprint filler; focused negatives 15/15 green; full-tree coverage check passes with `git ls-files -z` non-ASCII handling and BOM-tolerant frontmatter parsing | consumed by D02–D04 |
| `P8-T07/D02` | `done` | bilingual user tree (11 docs × 2) plus 9 generated reference families (18 pages) produced from implementation/machine contracts at `6b637a5`; generator `--check` byte-identical; annotation drift (task-resource prefix rewrite routes, `USERPROFILE`, helper/bracket env reads) reconciled against real sources | consumed by D04 |
| `P8-T07/D03` | `done` | bilingual developer (19 docs × 2) and AI (6 docs × 2) trees; symbol/test references machine-verified (HB006/007 caught and fixed 8 name-drift groups); sync surfaces landed: rule 20, `llms.txt`, AGENTS pointer, `check:handbook` script, named CI step | consumed by D04 |
| `P8-T07/D04` | `done` | acceptance mapping + validation record in `docs/checkpoints/20260812-personal-p8-t07-independent-handbook-closure.md`; `--diff-base 6b637a5` legacy-integrity proof green against the recorded allowlist; local check set green (handbook checker, generator `--check`, tools 49/49, consistency, `git diff --check`); content-head run `31572262460` green except shallow-clone HB013 (fixed in the closure head); required CI on the closure head via PR #202 checks | task closed; ready/merge/lease/branch/main reconciliation |
| `P7-T07/D01` | `done` | Windows Credential Manager backend, production selection, admin-cli arm, and failure-first negatives (including the real Credential Manager roundtrip/rotate/delete on `CI-WINDOWS-MSVC-01` and non-Windows fail-closed negatives on `CI-UBUNTU-01`) validated by required CI `31570126985` at `13e772a` | consumed by D04 |
| `P7-T07/D02` | `done` | inspectable installer/scheduled-task templates with static required/forbidden-fragment checks plus behavioral unrendered/version-mismatch/malformed-digest/non-HTTPS/extra-argument rejections and least-privilege task-XML parse; same required CI run at `13e772a` | consumed by D04 |
| `P7-T07/D03` | `done` | ADR-0052 §2/§3, B01-W preregistration `20260812-personal-p7-t07-b01-w-preregistration.md`, and the required-not-provisioned `B01-W-DESKTOP-001` registration; consistency validated locally and in the same CI run | consumed by D04 |
| `P7-T07/D04` | `blocked` | acceptance mapping recorded in `20260812-personal-p7-t07-windows-install-closure.md`: three of four acceptance items are complete; B01-W execution is blocked on Windows release artifacts, `B01-W-DESKTOP-001` provisioning, and an operator | owner recovery per the blocked-closure record, then execute the preregistered B01-W campaign and close the task |
| `P9-T04/D10` | `done` | `L3` executed on `B01-Desktop-Linux-002` at pushed `76d3d943b3ac8b06076f7122ab204e70dfdbb37d`. The owner performed the graphical hidden-input Provider import and authorized a campaign-scoped SSH path; both are recorded baseline changes. Scenario `R1` retained 30/30 started requests, all `complete_response`, marker 30/30, Provider usage `measured` 30/30, `retry=0`. Provider network p50 898.9 ms / p95 1224.6 ms; local governance + loopback overhead p50 126.5 ms / p95 147.6 ms, an 11.76 % share of the loopback exchange. No TTFT and no cost reported. Report digest `sha256:e6a62b91a2c42c44f60f908ff88372c646b71f97cb9fbeb170cb73a347533094`. `R5-selected-model-mismatch` also completed at `c45ed243033bb697d44ab1b361cd35d38190dddc`: 20/20 `denied_before_dispatch` with registered code `PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH`, zero Provider dispatches, `not_available` usage, denial latency 35.1 ms p50 / 44.0 ms max; digest `sha256:d859c7531ccac6ffb171c1f0314366ca896e89aa8dada4536ae0113b7af15982`. `R4-warm-repeated-conversation` completed 50/50 `complete_response` with marker and `measured` usage 50/50, Provider network p50 1016.1 ms / p95 1400.2 ms, local overhead p50 128.5 ms / p95 158.1 ms, 0.85 serial requests per second; digest `sha256:1b3f45601ff0ba4ac7df93b87431fbd3cba1c7f6c40443472b7d6fcec042046e`. Local overhead is stable across cells (126.5 ms and 128.5 ms p50) while Provider latency varies widely, which is why the two are measured apart. `R3-cold-daemon-first-response` completed 20/20 ready and 20/20 complete responses, startup-to-ready 182.6 ms p50, total cold journey 2069.2 ms p50 / 3012.8 ms p95, digest `sha256:11fb69dc1686d5c67e79fc2ab7448b3d61bfb6909ee91fb1ca958ecf7ab4fa7d`. `R6-bounded-timeout` retained 10/10 under a 120 ms client deadline with the bound holding at 122.9 ms p50 / 131.5 ms max, digest `sha256:68d76dffef701cf27af2b4c3b28ce1615b3fdcbf79ab749b77735b293660b335`; these stay `outcome_unknown` rather than `timeout` because `PI_EXTENSION_DAEMON_UNREACHABLE` cannot separate a deadline expiry from an absent daemon. Rate-limit stays not-run because inducing 429 would mean hammering a third-party Provider. Owner dropped `L5` on 2026-08-12, so the campaign closes with an `L0`-`L4` non-claim report. `R2-pi-first-response` completed 30/30 at `a9a2304175becf3d8ad06a78f9364f8da8f31f73` with Pi `0.81.1` acquired from the official npm origin (integrity `sha512-r6ovAsZO...N/P8A==`): marker 30/30, `authority_side_effects=false` 30/30, first response 4625 ms p50 / 5004 ms p95, digest `sha256:f6c6ac44a360d82e79b37f4d25f8f9b728c9b274b88f6b32e1780628585b4484`. Against `R1`'s ~1.0 s the Pi-hosted path costs ~3.5 s more, so Pi process spawn and agent init — not the governed path or the Provider — dominate first-response time. Reaching this required fixing the route probe, whose `env -i` allowlist omitted `XDG_STATE_HOME`, the one root the daemon publishes its endpoint file into. **`L3` is complete** (rate-limit sub-class deliberately not-run) | consumed by the completed P9-T04 task; closure checkpoint `20260812-personal-p9-t04-performance-campaign-closure.md`. The 11.76 % governance share is a single-arm observation, not a `B`-versus-`A` threshold result |
| `P2-T09/D01` | `done` | readiness derived from the assembled executor set, not registry availability; four failure-first negatives including "readiness changes no descriptor digest". Exact native Linux `cbb830a2dc64855f51c4b5ffdf04ae4f6fba4e4c` tool_registry 11/11 + Clippy, tool_executor unchanged 27/27; required CI passed at `0227b97` | consumed by completed P2-T09 task |
| `P9-T04/D09` | `done` | scenario safety-coverage negatives passed 10/10 with the full `cognitive-runtime` lib suite 168/168 and Clippy on exact native Linux `c8d047f2261868e9c4dcc21bcede86e45d4b5fd6`; required Ubuntu/Windows CI run `31560590193` passed at the same revision. All seven hard safety counters are now producible from a scenario judgment, and observed counts survive an otherwise verified run | consumed by the remaining scenario definitions |
| `P9-T04/D08` | `done` | scenario-harness negatives passed with the full `cognitive-runtime` lib suite 166/166 and Clippy on exact native Linux `099a079e76ca5b39c203e656a913db7be58c1275`; required Ubuntu/Windows CI run `31559903103` passed at the same revision. A frozen oracle decides each outcome, a mutation-boundary violation outranks every other result, and a weakened oracle or dropped run fails closed | consumed by D09 |
| `P9-T04/D07` | `done` | route-policy negatives passed 9/9 with Clippy on exact native Linux `7a2e3e1ce06b626dce7519f3c0eb42b44f15a907`; `retry=0` is fixed, every started request stays a classified outcome, and fabricated TTFT, fabricated cost, network time without dispatch, and measured usage on a failed request all fail closed. Required Ubuntu/Windows CI run `31558942939` passed at `5e04b1c070d4f387cac09b344cc94463a4f98c4f` | L3 execution needs the B01 start gate plus an operator graphical hidden-input Provider import; no code dependency remains |
| `P9-T04/D06` | `done` | one L0–L2 campaign executed on `DEV-LINUX-NATIVE-01` at exact pushed `ba141838c4949a6a16a95aa581ac6e3129a6cdb2`; report digest `sha256:c18a63db97400df8c254e2fe5ee6195826203b8183187501075505bb528e83be` records 1/200/52 started and retained samples for L0/L1/L2, `L3`–`L5` `not_run`, all seven safety counters zero, complete cleanup, `hypothesis` claim, `benefit_claimed=false`, `verifier=not_reviewed`. The runner refused a `B01-DESKTOP-002` label with `UnregisteredEnvironment`; runtime lib suite 149/149 and Clippy passed at the same revision | consumed by the remaining Provider-dependent layers |
| `P9-T04/D05` | `done` | campaign report-policy negatives passed 10/10 with Clippy and the full `cognitive-runtime` lib suite 148/148 on exact native Linux `40c5d8205df586fef405aa8839e57d24c932f045`; a dropped sample, a safety failure with a claim, an unverified or uncleaned promotion, and a benefit claim without a completed L5 all fail closed | consumed by D06 |
| `P9-T04/D04` | `done` | resource sampler negatives passed 10/10 with Clippy on exact native Linux `ad3c01751647af2e870c0559cb914a935392aed2`, including a real `/proc` parse of the running test process; command-line, environment, and descriptor-target sentinels never reach a sample, and a decreasing cumulative counter is rejected as PID reuse | consumed by D05 |
| `P9-T04/D03` | `done` | transport unit tests passed 7/7 and the real credential-header loopback request passed inside kernel-server front-door tests 7/7 with Clippy on exact native Linux `43eb4341bddde57f99d35530d1c202ff15ef33f7`; the observation disclaims `effect_persistence`, `provider_network`, `pi_process_launch`, and `scheduler_wait` and retains no header or bearer | consumed by D04 |
| `P9-T04/D02` | `done` | executable L0/L1 runner negatives passed 12/12 with Clippy on exact native Linux `ad800ca8a43196b4d9d7bcdab4481bdfa8245012`; the built runner emitted a redacted 7-benchmark report and failed closed on inherited secret-shaped environment, unregistered credential-shaped argument, and abbreviated revision without echoing any value; required Ubuntu/Windows CI passed at the same revision | consumed by D03 |
| `P9-T04/D01` | `done` | secret-free correlation/timing/Provider-usage envelope negatives passed 4/4 on exact native Linux `47629c40217ad039aededcb6a91207fe83f15b7a`; the real Pi-to-daemon correlation caller passed required Ubuntu/Windows CI `31553062959` at `4074c0a51eb5fede730d74559e5d35bc659941d3` | consumed by D02 |
| `P5-T04/D01` | `done` | dynamic package bind + disabled discovery; Linux 4/4 + Clippy at `b49d274` | consumed by D02 |
| `P5-T04/D02` | `done` | enable/disable/quarantine + TaskContract exposure; covered in same Linux 4/4 | consumed by D03 |
| `P5-T04/D03` | `done` | reconcile/composite/cache/bypass; covered in same Linux 4/4 | consumed by D04 |
| `P5-T04/D04` | `done` | ADR-0050 B10 MVP pass; required CI `31486478177` on `992dfe3`; PR #196 | consumed by completed P5-T04 task |
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
| `P9-T01/D01` | `done` | exact native Linux `perf::tests` 5/5 at `826745c`; aggregate authority-stage observation selected hypothesis-only `conservative-no-migration`; bounded socket reads and serial loopback-daemon tests restored required CI `31516749535` on `195510c` | consumed by completed P9-T01 task; no migration or performance claim |
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
| B10 | `pass` (MVP, ADR-0050) | fixed matrix at `b49d274` + CI `31486478177` on `992dfe3` + non-claim report + §2.3 disposition | none for MVP scope; does not pass GMVP-LINUX/release/Profile |
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
