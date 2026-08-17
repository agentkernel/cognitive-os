# PERSONAL-PERF-EVAL-010 assessment (final)

- Campaign: `PERSONAL-PERF-EVAL-010`
- Frozen source target: `289eebade1432fdf224cfe16661fdc102874e416` (P2-T33
  private-candidate host path; unmerged freeze)
- Lease: `lease/personal/EVAL-010/c1-c2-paired-freeze` (closed 2026-08-18)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

This is the campaign's single report (`TEST-REPORT-INCREMENTAL-01`). Cells
were appended as they finished. The campaign is closed; do not append more
cells on this freeze.

Owner 2026-08-18 authorized product changes after EVAL-009, then continuing
C1/C2 真机. EVAL-009 remains **closed**. This freeze measures P2-T33 pin
`289eebad` on a **long unique root**. P2-T33 stub pass is not C1/C2
Agent-benefit.

## Cells

| Cell | Status | Note |
|---|---|---|
| EVAL-009 remains closed (coordination) | **pass** | do not reuse `48296` / `/18` / `e009` runtime |
| Evaluation lease claimed | **pass** | claimed then closed 2026-08-18; Current snapshot row `PERSONAL-PERF-EVAL-010` **closed**; routing OFF |
| Freeze (archive/binaries/root/port) | **pass** | pin `289eebad`; root `/home/hal9001/perfeval010-20260818` mode `0700`; daemon `127.0.0.1:48298` pid 287493; `log_path` mode `0600` |
| SecretStore import | **pass** | new item `/org/freedesktop/secrets/collection/login/19` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | **pass** | real adapter, not the P2-T33 stub; SHA-256 `70ba7f05d3b743737334186c4b8b3155047cfa5856c4b0e28c45924866095cdb` |
| B0 C1 WorkspaceSearch O-arm | **partial** | one retained sample; admit 200; lifecycle `DRAFT`; `lease_acquired` 0; skip class `candidate_has_missing_fields_or_an_invalid_parameters_digest`; short completion socket created; real adapter spawned |
| B0 remaining C1/C2 families | `not-run` | C1-search did not leave `DRAFT` with `lease_acquired` ≥ 1 |
| B0 P-arm / broker `48398` | `not-run` | O-arm path not fairly measurable |
| B1/B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| Cleanup | **pass** | daemon 48298 stopped; broker 48398 absent; SecretStore `/19` cleared; 48181/48284/48383 and prior EVAL roots untouched |

## Freeze (2026-08-18) — pass

Exact source `289eebade1432fdf224cfe16661fdc102874e416` (P2-T33 private-candidate
host path; unmerged). Guest root mode `0700`. Listeners `48181` / `48284` /
`48383` untouched. SecretStore item `/19` is new (not `/12`–`/18`). Public
doctor was ready (`first_conversation_ready: true`). That is conversation
readiness, not a C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release,
Profile, B01, or Agent-benefit claim.

Host `DEV-LINUX-NATIVE-01` built release binaries from the extracted `289eebad`
archive with a dedicated `CARGO_TARGET_DIR` (`CARGO_NET_OFFLINE=true`, 1m 37s,
Rust 1.97.1). Windows GNU Rust build remains `not-run`
(`RUST-LINK-DEV-WIN-GNU-01`). Copies used `scp`.

Public start: endpoint `127.0.0.1:48298`, pid `287493`, `log_path`
`/home/hal9001/perfeval010-20260818/runtime/state/cognitiveos/daemon.log`
(mode `0600`). The campaign root is **long** so nested `completion.sock` under
the runtime tree would exceed Linux `UNIX_PATH_MAX` (108); P2-T33 binds the
socket under `$XDG_RUNTIME_DIR` / `temp_dir()` / `/tmp/cognitiveos` instead.

| Asset | Value |
|---|---|
| source archive | 14,735,360 bytes; 1544 entries; 0 `.git/` members; SHA-256 `ccf7e6a1ecba22a55e3a5fe50831f6a182bed3a21b84192d22c5ac7efaac769f` |
| `kernel-server` | 16,573,216 bytes; SHA-256 `a60e1166fa81e09b2b6b2e95892e9daccfc28fd98806f874e01d34502aedf1c5` |
| `cognitive` | 10,313,736 bytes; SHA-256 `6917dca3a0f294c34d1f177dd5ebd3e1a36fff1c71de7661094049b30741a65f` (CLI digest equal to EVAL-008/009; launcher tree unchanged) |
| `pi-agent-adapter` | 1,128,568 bytes; SHA-256 `70ba7f05d3b743737334186c4b8b3155047cfa5856c4b0e28c45924866095cdb` (differs from EVAL-008/009 `816856b4…`) |
| `o-arm-candidate.mjs` | SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `private_candidate_provider.mjs` | SHA-256 `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` |
| Pi tarball | SHA-256 `420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c` |

SecretStore import (stdin; D-Bus paths only): item
`/org/freedesktop/secrets/collection/login/19` (1 unlocked, 0 locked).
`login` collection `Items` contained only `/19` during the freeze.
Product report: `secret_backend=linux-secret-tool`,
`secret_material_written=true`, `secret_ref_redacted=true`,
`selected_model=deepseek-v4-flash`, `snapshot_digest=fnv1a64:c58ce6f2f7521544`.
Doctor after configure: provider `secret_ref_resolves=true`,
`secret_material_exposed=false`, Pi `package_status=ready` /
`pinned_version=0.81.1` / `observed_version=0.81.1`,
`first_conversation_ready: true`. Daemon pid 287493 still bound to
`127.0.0.1:48298`.

No Gate, release, Profile, B01, or Agent-benefit claim.

## B0 C1-search O-arm (2026-08-18) — partial; retained

One O-arm C1-search qualification Task was started with `retry=0` against
the public Task admit surface on the public `cognitive daemon start`
launcher. It is retained. It did not leave `DRAFT`. No Intent, Effect,
verification, or acceptance row exists. WorkspaceRead is still not
advertised; this cell used WorkspaceSearch only.

P2-T33 UNIX_PATH_MAX product fix: **confirmed on this long root.** During
the probe, `$XDG_RUNTIME_DIR/cognitiveos/pc-287493-3.sock` existed
(`srw-------`). Nested `completion.sock` under
`/home/hal9001/perfeval010-20260818` was not required. This is therefore
**not** EVAL-008's `private_completion_socket_could_not_be_created`.

EVAL-009 adapter-reject skip: **not this cell's skip.** The real adapter
spawned (see `/proc` note below) and returned a structurally parsed
candidate; the scheduler then skipped at
`validate_untrusted_pi_candidate`.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-0` | `task://local/eval010-b0-C1-search-b0-0-6a6d84c33d70` | 200 (record 12 ms, interpret 5 ms, preview 1 ms, admit 25 ms) | wall 178793 ms; 88 probes; `acceptance_ref` absent | `DRAFT`; minted `2026-08-17T20:23:41.343Z` |

Guest evidence file:
`/home/hal9001/perfeval010-20260818/evidence/b0-oarm-C1-search-b0-0.json`
SHA-256 `159204217de9201742d35fe2cae70ddb2f6f718649bc0a4f5e7c747a16a42ce9`.
Instrument `eval010-b0-c1-search.py`
SHA-256 `28a30666449ba94981f625ca53bcc1ded4b21f36916f5887ee17cd5c597a4f50`.
Public `cognitive task evidence`: `lifecycle.current_state=DRAFT`;
`intent_refs` / `effect_refs` empty; `latest_verification` /
`latest_acceptance` null. Bounded O4 last probe: `runnable_count` 4,
`lease_acquired` 0 (`observed_zero` true). O5 named zero.

Public skip rows on `daemon.log` (mode `0600`):

`kernel-server personal scheduler tick: skip row
task://local/eval010-b0-C1-search-b0-0-6a6d84c33d70 at epoch 1: scheduler
private Pi candidate proposal failed: candidate has missing fields or an
invalid parameters digest`

Skip class: `candidate_has_missing_fields_or_an_invalid_parameters_digest`.
That is `validate_untrusted_pi_candidate` after a successful private
adapter round-trip. Empty `tool_ref`/`action`/`target`,
`expected_state_version < 1`, or a `parameters_digest` that is not
`sha256:` plus 64 lowercase hex all share this class. This freeze does
not further split the class.

The instrument's `adapter_or_pi_count` stayed 0, which is a **false
negative**: `/proc` `Name` truncates to 15 characters (`pi-agent-adapte`).
Last probe `kernel_children` listed pid 288358 `pi-agent-adapte` with
ppid 287493 (campaign `kernel-server`). Real `pi-agent-adapter` **did**
spawn.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim. Scheduler ticks after the retained sample may have reached the
Provider proxy; this campaign does not claim zero Provider spend. Do not
start additional Provider cells on this freeze.

## C1/C2 paired remainder and later batches (2026-08-18) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes (C2a/C2b/C2c/C2d) | `not-run` | C1-search stayed `DRAFT` with `lease_acquired` 0 |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48398` | `not-run` | O-arm not fairly measurable |
| O5/O6 as C1/C2 dependents | `not-run` | no Intent/Effect |
| T4–T5/T8/T9 | `not-run` | no public dispatch |
| B3 faults | `not-run` | no frozen fault runner; do not cobble |
| B4 concurrency | `not-run` | B0 path/fairness incomplete |
| B5 1 h / 8 h / 24 h | `not-run` | no frozen soak runner; 1 h trigger not met |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | broker/runner not qualified; do not cobble a paired shell |

## Cleanup (2026-08-18) — pass

Guest route unchanged: `wuz@192.168.1.2` ProxyJump
`hal9001@192.168.123.160`. `B01-Desktop-Linux-002` snapshot was not
reverted. `B01-Clean-Linux-001` was not contacted.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48298` pid 287493 | product `cognitive daemon stop` `action=stopped` (`stale_lock_removed=true`); lock absent; listener absent |
| campaign broker `127.0.0.1:48398` | never started; listener absent |
| listeners `48181` / `48284` / `48383` | untouched (`cos-current` / EVAL-004 residue / EVAL-002 broker) |
| EVAL-004/005/006/007/008/009 roots | untouched |
| SecretStore item `/19` | `secret-tool clear` on the product attribute triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); post-clear D-Bus `SearchItems` unlocked/locked 0; `login` collection `Items` 0; never `secret-tool search`/`lookup` |
| redactor `evidence/` | 1 file, `key_shaped_hits=0` |
| redactor `runtime/` | 19 files, `key_shaped_hits=11` (same class as EVAL-009: binary `sk-` substrings in SQLite/WAL; not decoded; not treated as leaked Provider material) |
| campaign root | retained `0700` at `/home/hal9001/perfeval010-20260818` (evidence only) |

## Capability matrix (hypothesis / non-claim)

| Class | This freeze | Note |
|---|---|---|
| Public doctor / first conversation | ready | not a C1/C2 Task |
| Private completion socket on a long root | **pass** (host path) | `$XDG_RUNTIME_DIR/cognitiveos/pc-287493-3.sock`; not EVAL-008's skip |
| C1 WorkspaceSearch O-arm | **partial** | admit 200; stayed `DRAFT`; `lease_acquired` 0; real adapter spawned; skip `candidate_has_missing_fields_or_an_invalid_parameters_digest` |
| C2a mutation / C2b Memory-Skill / C2c Effect recovery / C2d verified completion | `not-run` | same skip; do not open more Provider spend |
| C1/C2 P-arm | `not-run` | broker `48398` never started |
| C0 paired G/A families | `not-run` | paired runner/broker not qualified |
| B3 / B4 / B5 | `not-run` | no frozen runners; 1 h trigger not met |

## Evidence-ranked optimization priorities (hypothesis only)

1. **Public `cognitive daemon start` + real `pi-agent-adapter`:** scheduler skip class `candidate_has_missing_fields_or_an_invalid_parameters_digest` is now a public `daemon.log` fact after a successful adapter round-trip. P2-T33 stub Workspace* tests and the UNIX_PATH_MAX bind are a different composition and are not this cell.
2. Adapter JSON-fallback `parameters_digest` is not recomputed from `parameters`; a model-supplied non-`sha256:` digest fails `validate_untrusted_pi_candidate` after spawn. Workspace* tool-event path already recomputes the digest.
3. Keep `/proc` `Name` 15-character truncation in mind (`pi-agent-adapte`). Do not treat `adapter_or_pi_count=0` as "no adapter".
4. WorkspaceRead is still not advertised as a Pi tool; C1 remains WorkspaceSearch-only until a later freeze says otherwise.

This campaign does not register a `P*-T*` task. Closure does not resume
development by itself. Owner 2026-08-18 already authorized product changes
after EVAL-009; the digest/fields skip is the next product follow-up, on a
new task/freeze.

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Never `secret-tool search`/`lookup`. Do not print
Provider keys. Do not treat P2-T33 stub pass, UNIX_PATH_MAX bind, or
adapter spawn as C1/C2 Agent benefit.

**Campaign closed.** Evaluation routing is OFF. **Rotate the Provider key**
leaked by the earlier EVAL-004 `secret-tool search` incident if that
material is still in use.
