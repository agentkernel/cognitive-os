# PERSONAL-PERF-EVAL-005 assessment (closed)

- Campaign: `PERSONAL-PERF-EVAL-005`
- Frozen source: `main@b16d2955` (P2-T29 closed)
- Lease: `lease/personal/EVAL-005/c1-c2-paired-freeze` (closed 2026-08-17)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: owner 2026-08-17 authorized measurement closure so product
  work can start. Campaign **closed 2026-08-17**. Evaluation routing OFF.

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

## Cells

| Cell | Status | Note |
|---|---|---|
| Freeze (archive/binaries/root/port) | **pass** | archive `sha256:af2836dd…`; guest `/home/hal9001/perfeval005-20260817`; daemon pid 267060 on `127.0.0.1:48288` |
| SecretStore import | **pass** | new item `/14`; stdin only; D-Bus paths only; never search/lookup; `/12`/`/13` unused |
| Pi 0.81.1 pin | **pass** | tarball `sha256:420113c0…`; `--extension` absolute; doctor ready; `first_conversation_ready: true` |
| Exact-source `pi-agent-adapter` | **pass** | `sha256:5a082cb5…`; candidate paths in `pi.json`; campaign wrapper ESM repaired |
| B0 C1/C2 paired | **partial** | two retained O-arm C1-search Tasks; skip class `scheduler_row_skip_before_lease`; P-arm not-run |
| B1 C1/C2 paired | **not-run** | B0 never left `DRAFT`; no Provider spend |
| B2 C1/C2 paired | **not-run** | same cause |
| O5/O6 | **not-run** | O5 named zero on retained DRAFT Tasks is not a C1/C2 result; O6 HTTP 400 |
| T4–T5/T8/T9 | **not-run** | no public Tool invocation/dispatch |
| B3 stale | **not-run** | no mutation path |
| MS-AUTH Memory positives | **pass** | 10/10 unsealed remember→review→forget→second-forget 409; caller-header 400 |
| B4 mixed / B5 paired soak | **not-run** | no paired C1/C2 path |
| Cleanup | **pass** | daemon `48288` already stopped; broker `48388` absent; SecretStore `/14` absent; 48181/48284/48383 and EVAL-004 roots untouched; redactor 22/0 |

No Gate, release, Profile, B01, or Agent-benefit claim.

## Freeze, SecretStore, Pi pin (2026-08-17)

Isolated guest root `/home/hal9001/perfeval005-20260817` (mode `0700`) on
`B01-Desktop-Linux-002` via `wuz@192.168.1.2` ProxyJump
`hal9001@192.168.123.160`. Frozen archive of `main@b16d2955`:
`sha256:af2836ddd807f592110387e3e60eca5f2105b2464a22fcbc534ab08e98f6922a`.
Exact-source `kernel-server`
`sha256:00b8963ce991e782f180b315ba731dcac6001581201547b8dd5dc9b97916410a` and
`cognitive`
`sha256:760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5`.
Campaign daemon pid 267060 on `127.0.0.1:48288`. Listeners `48181` /
`48284` / `48383` untouched; `48286` / `48386` absent.

SecretStore: new item `/14` via product stdin; D-Bus `SearchItems` paths
only; owner key file mtime unchanged; never `secret-tool search`/`lookup`.
Pi `0.81.1` pinned with `--extension` absolute path; doctor
`first_conversation_ready: true`. Public Extension `dist/extension.js`
registers WorkspaceSearch/Write/Patch; WorkspaceRead is not advertised.

## Adapter freeze (2026-08-17) — pass

Exact-source `pi-agent-adapter`
`sha256:5a082cb5ee5fac056c67632c729ab7fec0cabaccd9fc2db4389ebf58dc14ee49`
copied to the guest from the same `b16d2955` archive. `pi.json` now has
`--candidate-adapter` and `--candidate-extension`. Campaign wrapper
`o-arm-candidate.mjs` (`sha256:3d9c03db…`) loads the frozen private-candidate
Provider plus frozen `daemonGovernedWorkspaceTools()`. An earlier unquoted
copy failed `node --check`; it was replaced in the guest root only.

## B0 C1-search O-arm (2026-08-17) — partial; retained

Started samples (`retry=0`):

- `task://local/eval005-b0-C1-search-b0-0-757d5b66ffae` minted
  `2026-08-17T07:21:17.175Z`
- `task://local/eval005-b0-C1-search-b0-0-ab6c3c389d2d` minted
  `2026-08-17T07:21:44.869Z`; probe wall 180825 ms

Public `cognitive task evidence`: both remain `DRAFT`; `intent_refs` and
`effect_refs` empty; no verification or acceptance. Scheduler O4
`runnable_count` continues. No `pi-agent-adapter` child observed. Campaign
daemon stdio is `/dev/null`. P-arm / broker `48388` not started. B1/B2 not
opened. Listeners `48181` / `48284` / `48383` untouched.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Private-candidate skip (2026-08-17) — `scheduler_row_skip_before_lease`

Campaign-only observation. No product change. Instrument
`observe_skip.py` `sha256:6a728cb0…`. Evidence
`b0-private-candidate-skip.json`
`sha256:0944e8b0cac0e7fb9bfe655d9511e0a3872414744b8f490d297e3d3ac8f2ca61`.

Public `cognitive status` / `doctor`: all required components `ready`,
`first_conversation_ready: true`, Pi `0.81.1`. That is conversation
readiness, not a C1/C2 Task→Workspace* caller. Provider
`secret_ref_resolves=true` (redacted). Campaign daemon pid **267060** still
bound to `127.0.0.1:48288`; stdout/stderr both `/dev/null`. Listeners
`48181` / `48284` / `48383` untouched; `48286` / `48386` / `48388` absent.

Both retained Tasks are still `DRAFT`. The only lifecycle event is
`cognitiveos.task-contract.minted`. `intent_refs` / `effect_refs` empty;
acceptance and verification absent. O4 on each ref (bounded window 32/32):
`runnable_count` 32/32, `queue_wait` 0/32, **`lease_acquired` 0/0**,
`budget_stop`/`fairness`/`stale_fence_denial` 0/0. Recent samples are only
`runnable_count` / `queue_wait`. O5 is a named zero (`observed_zero`). O6
HTTP 400. Eight 250 ms child samples of kernel-server: 0 children; 0
`pi-agent-adapter` / `pi-coding-agent` processes.

Freeze assets that would be required for a spawn are present: `pi.json`
has both candidate paths and those files exist; `selected-model.json`
`deepseek-v4-flash`; adapter 1,125,592 bytes. The skip class is therefore
**not** a missing-freeze-asset `not_available`. The public fact is that
the scheduler keeps the admitted rows runnable and never leases, never
emits Intent/Effect, and never leaves a Pi child. The per-row skip string
(`kernel-server personal scheduler tick: skip row …`) is only on stderr,
which this public `cognitive daemon start` sends to `/dev/null`, so the
exact `PrivatePiProposal` / Context error is **not a public fact**.

This is not a real public C1/C2 caller. Remaining paired B0 classes,
B1/B2, and P-arm stay `not-run`. Do not open Provider spend on a path
that never leaves `DRAFT`.

## C1/C2 paired remainder (2026-08-17) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes | `not-run` | same skip; only two C1-search O-arm samples were started and are retained |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48388` | `not-run` | equivalent Pi adapter not started; fairness not measurable |
| O5/O6 as C1/C2 dependents | `not-run` | no Intent/Effect |
| T4–T5/T8/T9 | `not-run` | T8 is invocation-time descriptor drift; no public dispatch |
| B3 stale Task/epoch | `not-run` | no mutation path |

## MS-AUTH Memory positives (2026-08-17) — pass

P2-T29 unsealed remember with daemon-composed `GovernanceSeed` headers is
on this freeze. Instrument `msauth_memory.py`
`sha256:023a5292…`. Evidence
`msauth-memory-positives.json`
`sha256:da0e0b25d0f58ab1c736cb95ac7fcc1073ae4f6b0b6411828ca07189e1b0728b`.
Management-channel public API only. `retry=0`. Elapsed 236 ms.

| Sub-cell | Started | Retained | Result |
|---|---:|---:|---|
| caller-minted header on unsealed remember | 1 | 1 | HTTP **400** `RESOURCE_MEMORY_PAYLOAD_INVALID` |
| remember→review→forget→second-forget | 10 | 10 | **10/10**: remember 201, review 200, forget 201, second forget **409** `RESOURCE_MEMORY_CONFLICT` |

Round wall 16–22 ms. This is authority-API smoke, not Agent Memory
consumption and not a paired C2b result. Skill lifecycle on this freeze
was not started. New evidence files had 0 key-shaped hits; the earlier
retained B0 probe file still has the previously classified non-secret
match. `secret-tool search`/`lookup` were not used.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Cleanup (2026-08-17) — pass

Owner 2026-08-17 authorized campaign close (measurement closure only; no
product patch in this campaign). Guest route unchanged:
`wuz@192.168.1.2` ProxyJump `hal9001@192.168.123.160`.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48288` pid 267060 | already absent; product `cognitive daemon stop` reported `already_stopped`; lock absent; listener absent |
| campaign broker `127.0.0.1:48388` | never started; listener absent |
| listeners `48181` / `48284` / `48383` | untouched (`cos-current` / EVAL-004 residue / EVAL-002 broker) |
| EVAL-004 roots `perfeval004` / `perfeval004-20260816` | untouched |
| SecretStore item `/14` | unlocked `login` collection `item_count 0`; direct path `/login/14` absent; `secret-tool clear` on the product attribute triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); D-Bus `SearchItems` and collection `Items` paths only; never `secret-tool search`/`lookup` |
| redactor `evidence/` | 4 files, `key_shaped_hits=0` |
| redactor `runtime/` | 18 files, `key_shaped_hits=0` |
| campaign root | retained `0700` at `/home/hal9001/perfeval005-20260817` (evidence only) |

**Campaign closed.** C1/C2 paired remains `partial`/`not-run` on skip class
`scheduler_row_skip_before_lease` (Tasks stayed `DRAFT`; `lease_acquired`
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
