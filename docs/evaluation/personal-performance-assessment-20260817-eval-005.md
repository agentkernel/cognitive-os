# PERSONAL-PERF-EVAL-005 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-005`
- Frozen source: `main@b16d2955` (P2-T29 closed)
- Lease: `lease/personal/EVAL-005/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

## Cells

| Cell | Status | Note |
|---|---|---|
| Freeze (archive/binaries/root/port) | **pass** | archive `sha256:af2836dd…`; guest `/home/hal9001/perfeval005-20260817`; daemon pid 267060 on `127.0.0.1:48288` |
| SecretStore import | **pass** | new item `/14`; stdin only; D-Bus paths only; never search/lookup; `/12`/`/13` unused |
| Pi 0.81.1 pin | **pass** | tarball `sha256:420113c0…`; `--extension` absolute; doctor ready; `first_conversation_ready: true` |
| Exact-source `pi-agent-adapter` | **pass** | `sha256:5a082cb5…`; candidate paths in `pi.json`; campaign wrapper ESM repaired |
| B0 C1/C2 paired | **partial** | two O-arm C1-search Tasks started+retained; both still `DRAFT`; no Intent/Effect; P-arm not-run |
| B1 C1/C2 paired | **not-run** | |
| B2 C1/C2 paired | **not-run** | |
| Dependent O5/O6, T4–T5/T8/T9, B3 stale, MS-AUTH Memory positives | **not-run** | after C1/C2 paired B0 |
| B4 mixed / B5 paired soak | **not-run** | only if Provider budget remains |

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
