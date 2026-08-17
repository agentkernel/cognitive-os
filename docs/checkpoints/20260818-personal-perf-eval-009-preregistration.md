# PERSONAL-PERF-EVAL-009 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-009`
- Lease: `lease/personal/EVAL-009/c1-c2-paired-freeze`
- Date: 2026-08-18
- Frozen product source: `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630`
  (same unmerged P2-T32 public launcher pin as EVAL-008). Docs commit
  `a653dc7b` has the same product bytes; freeze source/binaries pin
  `fb85cfff`, not `origin/main` (`2a8d4d2f`). Unmerged freeze is allowed.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reopen EVAL-008 / EVAL-007 / PR #238 /
PR #239. It does not reuse EVAL-004/005/006/007/008 campaign roots, loopback
ports `48286`/`48288`/`48290`/`48292`/`48294`/`48386`/`48388`/`48390`/
`48392`/`48394`, SecretStore items `/12`–`/17`, broker, runner, corpus,
oracle, or evidence denominator.

Owner 2026-08-18 authorized continuing C1/C2 and remaining execution-plan
真机 cells after EVAL-008 close, and authorized solving recoverable blockers
without waiting. EVAL-008 skip class
`private_completion_socket_could_not_be_created` is hypothesized as Linux
`UNIX_PATH_MAX` (108) overflow on

`{root}/runtime/config/cognitiveos/private-completions/candidate-{pid}-{seq}/completion.sock`

under the long EVAL-008 root. This freeze uses a **short unique root** so the
same pin can be measured. That is a campaign isolation parameter, not a
product patch and not a claim that long roots work. P2-T32 stub pass is still
not C1/C2 Agent-benefit.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/e009` mode `0700` (short; UNIX_PATH_MAX hypothesis) | `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `perfeval006-20260817`, `perfeval007-20260817`, `perfeval008-20260818`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48296` daemon; broker `127.0.0.1:48396` (P-arm only after O-arm is fairly measurable) | `48181`, `48282`, `48284`, `48286`, `48288`, `48290`, `48292`, `48294`, `48383`, `48386`, `48388`, `48390`, `48392`, `48394` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`); expected `/18` or next unused path | `/11`–`/17`; never `secret-tool search`/`lookup` |
| Source pin | `fb85cfff` (P2-T32 public launcher) | EVAL-007 pin `2a8d4d2f`; EVAL-006 pin `103fe776` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

Guest control: `virsh -c qemu:///system` on `hal9000` only. Do not start,
restore, or delete the B01 guest beyond this preregistration. The domain is
used as-is. Do not revert snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`. Guest `ldd` on
campaign binaries may resolve only glibc/`libgcc`/`libm`.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-008 remains closed | **pass** | do not reopen; do not reuse `/17` / `48294` / `perfeval008-20260818` runtime |
| Evaluation lease claimed | **pass** | this document + Current snapshot row `PERSONAL-PERF-EVAL-009` **active** |
| Product source pin | pending | `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` |
| Source archive + SHA-256 | pending | reuse EVAL-008 archive digest `202384ee…` after re-hash; 0 `.git/` members |
| New campaign root/port | pending | `/home/hal9001/e009` mode `0700`; daemon `127.0.0.1:48296` |
| Exact-source daemon/CLI binaries | pending | same SHA-256 as EVAL-008 freeze; `ldd` glibc/`libgcc`/`libm` only |
| Campaign daemon on `48296` | pending | public `cognitive daemon start`; `log_path` mode `0600` |
| New SecretStore entry | pending | stdin import; D-Bus `SearchItems` paths only |
| Local Pi `0.81.1` pin | pending | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | pending | real adapter, not the P2-T32 stub |
| `cognitive doctor` | pending | readiness only |
| C1/C2 paired B0 | pending | C1 WorkspaceSearch O-arm first |
| C1/C2 paired B1/B2 | pending | only after B0 path/fairness |
| Cleanup / campaign close | pending | stop `48296`; clear this campaign SecretStore item; leave `48181`/`48284`/`48383` and EVAL-004/005/006/007/008 roots |

## Unique next action

Freeze this short-root pin, then B0 C1 WorkspaceSearch O-arm. If the Task
leaves `DRAFT` with `lease_acquired` ≥ 1, continue remaining C1/C2 families
then P-arm. If the same skip class returns, retain the sample and do not
open Provider spend. Never `secret-tool search`/`lookup`. Do not patch
product code in this campaign.

## Non-claims

This campaign creates no Gate, release, Profile, B01, or Agent-benefit
promotion. `retry=0` for Provider cells. Every started sample is retained.
Each finished cell is appended to the running report immediately.
WorkspaceRead is still not advertised as a Pi tool; C1 uses WorkspaceSearch
only. Short root is not a product fix for long XDG paths.
