# PERSONAL-PERF-EVAL-010 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-010`
- Lease: `lease/personal/EVAL-010/c1-c2-paired-freeze`
- Date: 2026-08-18
- Frozen product source: `289eebade1432fdf224cfe16661fdc102874e416`
  (P2-T33 private-candidate host path; unmerged freeze). Docs commit
  `9dc40417` has the same product bytes; freeze source/binaries pin
  `289eebad`, not `origin/main` (`2a8d4d2f`) and not P2-T32 `fb85cfff`.
  Unmerged freeze is allowed.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted on this freeze (measurement-only)

This is a **new freeze**. It does not reopen EVAL-009 / EVAL-008 / EVAL-007 /
PR #238 / PR #239 / PR #240. It does not reuse EVAL-004/005/006/007/008/009
campaign roots, loopback ports `48286`/`48288`/`48290`/`48292`/`48294`/
`48296`/`48386`/`48388`/`48390`/`48392`/`48394`/`48396`, SecretStore items
`/12`–`/18`, broker, runner, corpus, oracle, or evidence denominator.

Owner 2026-08-18 authorized product changes after EVAL-009 close, then
continuing C1/C2 真机. P2-T33 linux-002 focused `p2_t33` 2/2 and Ubuntu
`32063236152` `verify` passed at `289eebad`. P2-T33 stub pass is still not
C1/C2 Agent-benefit. This freeze uses a **long unique root** so the
UNIX_PATH_MAX product fix is measured on the same class of path that skipped
EVAL-008.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval010-20260818` mode `0700` (long; UNIX_PATH_MAX product proof) | `e009`, `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `perfeval006-20260817`, `perfeval007-20260817`, `perfeval008-20260818`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48298` daemon; broker `127.0.0.1:48398` (P-arm only after O-arm is fairly measurable) | `48181`, `48282`, `48284`, `48286`, `48288`, `48290`, `48292`, `48294`, `48296`, `48383`, `48386`, `48388`, `48390`, `48392`, `48394`, `48396` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`); expected `/19` or next unused path | `/11`–`/18`; never `secret-tool search`/`lookup` |
| Source pin | `289eebad` (P2-T33 host path) | EVAL-009/008 pin `fb85cfff`; EVAL-007 pin `2a8d4d2f` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

Guest control: `virsh -c qemu:///system` on `hal9000` only. Do not start,
restore, or delete the B01 guest beyond this preregistration. The domain is
used as-is. Do not revert snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-009 remains closed | **pass** | do not reopen; do not reuse `/18` / `48296` / `e009` runtime |
| Evaluation lease claimed | **pass** | this document + Current snapshot row `PERSONAL-PERF-EVAL-010` **active** |
| Product source pin | **pass** | `289eebade1432fdf224cfe16661fdc102874e416` |
| Source archive + SHA-256 | `not-run` | after this claim is pushed |
| New campaign root/port | `not-run` | `/home/hal9001/perfeval010-20260818` / daemon `48298` |
| Exact-source daemon/CLI binaries | `not-run` | pin `289eebad`; real `pi-agent-adapter` |
| Campaign daemon on `48298` | `not-run` | public `cognitive daemon start` |
| New SecretStore entry | `not-run` | stdin import; D-Bus `SearchItems` paths only |
| Local Pi `0.81.1` pin | `not-run` | `--extension` absolute |
| C1/C2 paired B0 | `not-run` | WorkspaceSearch O-arm first; C1 stays search-only |
| C1/C2 paired B1/B2 | `not-run` | after B0 path/fairness |
| Cleanup / campaign close | `not-run` | stop `48298`; clear `/19`; leave `48181`/`48284`/`48383` and prior EVAL roots |

## Unique next action

Push this claim. Freeze exact `289eebad` binaries onto
`/home/hal9001/perfeval010-20260818` with daemon `127.0.0.1:48298`, import a
new SecretStore item without search/lookup, then run B0 C1 WorkspaceSearch
O-arm with the real adapter.

Claim ceiling `hypothesis`; `not_reviewed`. No Gate, release, Profile, B01,
or Agent-benefit promotion.
