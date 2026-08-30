# P11-T11 Memory admission, privacy, forget — closure

- Task: `P11-T11` / slice `P11-T11/D01` (full Phase 11 T11 acceptance)
- Change class: `implementation-only` (scoped episodic recall + privacy screens on the existing Memory store/HTTP; no `core/specs`, no Lane-CTR, no `/ui/` IA, no Letta/Mem0 write path)
- Branch: `personal/P11-T11-memory`
- Linux native focused HEAD: `f1dca3e038104c0f03879a2a368a635d4b876a2c`
- Required-CI / PR head: `60844f511a58d537fc68cb4620542cddeac21994`
- Merge revision: `main@b5084e06ce690919e727fd49a598c06e9996cac5`
- Pull request: [#289](https://github.com/agentkernel/cognitive-os/pull/289) (merged 2026-08-31)
- Lease: `lease/personal/P11-T11/memory` (closed into PARALLEL-LANES §3.1 by this ledger)
- Required CI on `60844f51`: **SUCCESS** — run [33327844743](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743): `resolve validation route` **SUCCESS** [99301045623](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99301045623), `verify (ubuntu-latest)` **SUCCESS** [99301056095](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99301056095) 3m35s, `verify (windows-latest)` **SUCCESS** [99301056084](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99301056084) 13m37s, `required-ci` **SUCCESS** [99302780648](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99302780648). Incremental log: [report](2026-08-31-personal-p11-t11-memory-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers full Phase 11 T11 close gate. Host privacy/rebuild E2E, B01, Windows OPC Memory E2E, and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux store **4/4**, HTTP **2/2** at `f1dca3e0` are **pass**. Workspace `required-ci` on `60844f51` is **SUCCESS**. Vault files still cannot enter Memory as authority (T10 N5 retained). Forget then index rebuild cannot resurrect scoped Memory.

| Acceptance item | Evidence |
|---|---|
| Memory requires admission; scoped episodic recall | store N1 `p11_t11_cross_scope_episodic_recall_is_rejected`; HTTP recall 403. Linux store **4/4** at `f1dca3e0` |
| Secret/PII-shaped candidate denied | store N2; HTTP remember 422 |
| Agent/self and Letta/Mem0-style direct write rejected | store N3; HTTP remember 422 |
| Forget → index/cache rebuild → no resurrection | store N4 scoped forget + `index.rebuild`; existing `p4_t02` FTS tombstone retained |
| Vault file still cannot enter Memory as authority | T10 `p11_t10_memory_admission_cannot_swallow_vault_files` (N5 retained) |
| Task-channel Memory mutation fail-closed | HTTP `POST /task/resource/v1/memory/{remember,forget,recall,correct,index.rebuild,review}` 403 (N6) |
| Management `correct` fail-closed | HTTP secret-shaped 422 + cross-scope 403 |
| Linux store T11 focused negatives | **pass** **4/4** at `f1dca3e0` (`DEV-LINUX-NATIVE-01`) |
| Linux HTTP scoped recall / privacy / forget / N6 / correct | **pass** **2/2** at `f1dca3e0` |
| Host privacy / rebuild E2E | **not-run** (card allows until the host route is qualified) |
| B01 campaign guest | **not_available** / **not-run** (evaluation routing OFF) |
| Windows OPC Memory E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01`) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `60844f51` | **SUCCESS** run [33327844743](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store N1–N4 (cross-scope, secret/PII, Agent/Letta-Mem0 write, forget non-resurrection) | **pass** 4/4 | `DEV-LINUX-NATIVE-01` | `f1dca3e038104c0f03879a2a368a635d4b876a2c` |
| kernel-server `p11_t11` (N6 + management correct + scoped forget) | **pass** 2/2 | `DEV-LINUX-NATIVE-01` | `f1dca3e038104c0f03879a2a368a635d4b876a2c` |
| Host privacy / rebuild E2E | **not-run** | unqualified | `f1dca3e0` |
| B01 guest | **not-run** | evaluation routing OFF | `f1dca3e0` |
| Windows OPC Memory E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | `f1dca3e0` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `60844f51` |
| `verify (ubuntu-latest)` on `60844f51` | **SUCCESS** [99301056095](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99301056095) | `CI-UBUNTU-01` | `60844f511a58d537fc68cb4620542cddeac21994` |
| `verify (windows-latest)` on `60844f51` | **SUCCESS** [99301056084](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99301056084) | `CI-WINDOWS-MSVC-01` | `60844f511a58d537fc68cb4620542cddeac21994` |
| `required-ci` on PR head `60844f51` | **SUCCESS** [99302780648](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743/job/99302780648) | GitHub Actions run [33327844743](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743) | `60844f511a58d537fc68cb4620542cddeac21994` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T13 `/ui/` IA. No Letta/Mem0 write path. No Agent self-admission. Vault≠Memory. Not T02 (Windows host). Not T08 Routine/Trigger. Do not auto-claim T13. Do not unpark T14/T15. Privacy/rebuild host E2E **not-run**. Evaluation routing OFF. Live `/ui/` remains Linux 1.0 six-family.

## Deterministic closure

1. Linux native focused **pass** at `f1dca3e0` (store 4/4, HTTP 2/2);
2. required CI [33327844743](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743) **SUCCESS** on `60844f51`;
3. PR [#289](https://github.com/agentkernel/cognitive-os/pull/289) merged as `main@b5084e06` on 2026-08-31;
4. lease `lease/personal/P11-T11/memory` moved to §3.1;
5. remote `personal/P11-T11-memory` deleted when GitHub already did after merge; local task branch deleted when safe; local `main` fast-forwarded to the merge plus this status/closure commit.

Unique next: claim `P11-T08/D01`. This file does **not** claim `lease/personal/P11-T08/routine`. Do not auto-claim `P11-T02`/`T13`. Do not unpark `P11-T14`/`T15`.
