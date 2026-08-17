# PERSONAL-PERF-EVAL-006 assessment (final)

- Campaign: `PERSONAL-PERF-EVAL-006`
- Frozen source target: `main@103fe776` (P2-T30 closed)
- Lease: `lease/personal/EVAL-006/c1-c2-paired-freeze` (closed 2026-08-17)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: owner 2026-08-17 authorized a new C1/C2 paired freeze after
  P2-T30. Campaign **closed**. Measurement-only. Evaluation routing OFF.

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

## Cells

| Cell | Status | Note |
|---|---|---|
| Freeze (archive/binaries/root/port) | **pass** | guest `/home/hal9001/perfeval006-20260817`; daemon `127.0.0.1:48290` pid 273829; archive `sha256:d322be1555…`; kernel-server `sha256:47513386ae…` |
| SecretStore import | **pass** | new item `/15` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor package/pinned/observed `0.81.1`; `first_conversation_ready: true` |
| Exact-source `pi-agent-adapter` | **pass** | same `103fe776` archive; `sha256:816856b496…`; `o-arm-candidate.mjs` `sha256:29870821…` |
| B0 C1/C2 paired | **partial** | one retained O-arm C1-search Task; skip class `scheduler_row_skip_before_lease`; P-arm not-run |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| Cleanup | **pass** | daemon 48290 already stopped; broker 48390 absent; SecretStore `/15` cleared; 48181/48284/48383 and EVAL-004/005 roots untouched; redactor 16/0 |

## Freeze (2026-08-17) — pass

Exact source `main@103fe776`. Guest root mode `0700`. Listeners `48181` /
`48284` / `48383` untouched. SecretStore item `/15` is new (not `/12` /
`/13` / `/14`). Public doctor: all required components `ready`, Pi
`0.81.1`, `first_conversation_ready: true`. That is conversation
readiness, not a C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release,
Profile, B01, or Agent-benefit claim.

## B0 C1-search O-arm (2026-08-17) — partial; retained

One O-arm C1-search qualification Task was started with `retry=0` against
the public Task admit surface. It is retained. It did not leave `DRAFT`.
No Intent, Effect, verification, or acceptance row exists. WorkspaceRead
is still not advertised; this cell used WorkspaceSearch only.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-0` | `task://local/eval006-b0-C1-search-b0-0-071b35428873` | 200 (record 19 ms, interpret 9 ms, preview 4 ms, admit 29 ms) | wall 179547 ms; `acceptance_ref` absent | `DRAFT`; minted `2026-08-17T11:58:33.49Z` |

Guest evidence file:
`/home/hal9001/perfeval006-20260817/evidence/b0-oarm-C1-search-b0-0.json`
`sha256:185842821d1849aeeb28ad92e06f0d0dcbf7e0497dc544c2c70aed236f85111d`.
Instrument `eval006-b0-c1-search.py`
`sha256:2d029e45ea02740af98ece7c1dbfe80fc4507d4471003c73b3bbf3d94b1a6368`.
Public `cognitive task evidence`: `lifecycle.current_state=DRAFT`;
`intent_refs` / `effect_refs` empty; `latest_verification` /
`latest_acceptance` null. Bounded O4: `runnable_count` 32/32,
`lease_acquired` 0/0. O5 named zero. No `pi-agent-adapter` or
`pi-coding-agent` child. Campaign kernel-server pid 273829. Listeners
`48181` / `48284` / `48383` untouched. P-arm / broker `48390` not started.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Private-candidate skip (2026-08-17) — `scheduler_row_skip_before_lease`

Campaign-only observation. No product change. Public `cognitive status` /
`doctor` were ready (`first_conversation_ready: true`) before B0; that is
not C1/C2. The admitted Task stayed `DRAFT` for the full 180 s probe.
O4 `lease_acquired` 0/0. No Pi child. Freeze assets required for a spawn
were present (`pi.json` candidate paths, adapter binary, selected model).
This is therefore **not** a missing-freeze-asset `not_available`.

P2-T30's focused test drives `TaskApi::handle` then
`run_production_chain_tick` with a `DeterministicProductionChainProposer`.
This cell used the live daemon scheduler on `127.0.0.1:48290`. The public
skip class is unchanged from EVAL-005. The per-row skip string is only on
stderr, which public `cognitive daemon start` sends to `/dev/null`, so the
exact Context/Loop error is **not a public fact**.

This is not a real public C1/C2 caller. Remaining paired B0 classes,
B1/B2, and P-arm stay `not-run`. Do not open Provider spend on a path
that never leaves `DRAFT`. The skip is a product mutex for a new formal
task after this close (do not collide with P2-T29/T30). That follow-on
task must exercise the **live daemon HTTP** admit path, not only the
in-process fixture tick.

## C1/C2 paired remainder (2026-08-17) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes | `not-run` | same skip; one C1-search O-arm sample was started and is retained |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48390` | `not-run` | equivalent Pi adapter not started; fairness not measurable |
| O5/O6 as C1/C2 dependents | `not-run` | no Intent/Effect |
| T4–T5/T8/T9 | `not-run` | T8 is invocation-time descriptor drift; no public dispatch |
| B3 stale Task/epoch | `not-run` | no mutation path |

## Cleanup (2026-08-17) — pass

Owner 2026-08-17 authorized campaign close after B0 path/fairness failed
(measurement closure only; no product patch in this campaign). Guest route
unchanged: `wuz@192.168.1.2` ProxyJump `hal9001@192.168.123.160`.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48290` pid 273829 | already absent at cleanup; product `cognitive daemon stop` reported `already_stopped`; lock absent; listener absent |
| campaign broker `127.0.0.1:48390` | never started; listener absent |
| listeners `48181` / `48284` / `48383` | untouched (`cos-current` / EVAL-004 residue / EVAL-002 broker) |
| EVAL-004/005 roots | untouched |
| SecretStore item `/15` | `secret-tool clear` on the product attribute triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); post-clear D-Bus `SearchItems` unlocked/locked 0; `login` collection `Items` 0; never `secret-tool search`/`lookup` |
| redactor `evidence/` | 1 file, `key_shaped_hits=0` |
| redactor `runtime/` | 15 files, `key_shaped_hits=0` |
| campaign root | retained `0700` at `/home/hal9001/perfeval006-20260817` (evidence only) |

**Campaign closed.** C1/C2 paired remains `partial`/`not-run` on skip class
`scheduler_row_skip_before_lease` (Task stayed `DRAFT`; `lease_acquired`
0/0; no Intent/Effect; no Pi child). That skip is a product mutex for a
new formal task after this close. Claim ceiling `hypothesis`, verifier
`not_reviewed`. No Gate, release, Profile, B01, or Agent-benefit
promotion.

The EVAL-004 `secret-tool search` leak is unchanged: the owner must rotate
the Provider key. This session did not print secret material.

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Owner 2026-08-17 authorized backlog continuation
after this close; evaluation routing is OFF.
