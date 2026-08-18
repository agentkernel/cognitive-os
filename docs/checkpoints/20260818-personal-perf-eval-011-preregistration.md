# PERSONAL-PERF-EVAL-011 C1/C2 B0 preregistration

- Campaign: `PERSONAL-PERF-EVAL-011`
- Lease: `lease/personal/EVAL-011/c1-c2-b0-qualification`
- Date: 2026-08-18
- Execution plan: [Personal performance benchmark execution plan](../evaluation/personal-performance-benchmark-execution-plan.md)
- Frozen product source: `979e52e4c6681d0fc8c6431c965e3267a7a0d917`
  (`personal/P2-T36-c1-public-production-path`, already pushed)
- Target: `B01-Desktop-Linux-002` through the registered host route
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: prohibited; this campaign is measurement-only

## 1. Scope and owner authorization

The owner explicitly activated this new, isolated B01 C1/C2 campaign on
2026-08-18. It does not reopen, amend, extend, or reuse `PERSONAL-PERF-EVAL-004`
through `PERSONAL-PERF-EVAL-010`. It is limited to B0 O-arm qualification for
the public `cognitive daemon start` composition at the frozen source revision.

The campaign may observe whether a real Pi candidate reaches the daemon-owned
candidate validation, scheduler lease, Tool executor, independent verifier, and
daemon acceptance path for WorkspaceRead and WorkspaceSearch. It must retain
every started sample and record `pass`, `fail`, `partial`, or `not-run`
immediately in the single running report. Provider cells use `retry=0`.

No B1/B2 paired sample, P-arm broker, corpus/oracle change, performance result,
Gate, release, Profile, B01, or Agent-benefit claim is authorized by this
preregistration. If B0 is incomplete or unfair, all dependent cells remain
`not-run`.

## 2. Isolation and immutable campaign parameters

| Item | EVAL-011 allocation | Explicit exclusions |
|---|---|---|
| Guest root | `/home/hal9001/perfeval011-20260818`, mode `0700` | EVAL-004 through EVAL-010 roots, `cos-current`, `~/p9t04`, and every prior campaign evidence root |
| Daemon endpoint | `127.0.0.1:48300` | `48181`, `48282`, `48284`, `48286`, `48288`, `48290`, `48292`, `48294`, `48296`, `48298`, `48383`, `48386`, `48388`, `48390`, `48392`, `48394`, `48396`, `48398` |
| P-arm broker reservation | `127.0.0.1:48400`; not started unless a later owner-approved amendment qualifies it | every prior broker, including `48398`; no broker in B0 |
| SecretStore | one new item imported only by the product CLI's hidden stdin prompt; expected next unused item after `/19` | items `/12` through `/19`; `secret-tool search` and `secret-tool lookup` are prohibited |
| Source | GitHub archive and binaries from exact `979e52e4c6681d0fc8c6431c965e3267a7a0d917` | copied local source, an uncommitted tree, or a source snapshot without Git provenance |
| Pi | pinned `@earendil-works/pi-coding-agent@0.81.1`, exact absolute extension path | Pi built-in filesystem and shell tools; no tool-policy relaxation |

`B01-Clean-Linux-001` is never contacted. This procedure authorizes no guest
snapshot create, restore, revert, deletion, power action, or external system
configuration change. The B01 guest is used as-is. The campaign ends with
public daemon stop, new SecretStore item cleanup through the allowed product
attribute triple, and no reuse of its root, port, or item.

## 3. Baseline and first attempt allowlist

Before any guest mutation, record only these non-secret baseline facts:

1. host and guest identity, current guest state, and current snapshot list via
   `virsh -c qemu:///system` on the registered host;
2. absence of the new campaign root and no listener on `48300` or `48400`;
3. exact source archive digest, binary digests, and pinned Pi version;
4. D-Bus Secret Service item paths/count only, never item attributes or values.

The first B0 attempt may then: create the new root, transfer the frozen source
archive and exact-source binaries, use the public CLI to configure Pi and start
the daemon at `48300`, prompt the owner directly on the guest terminal for the
Provider key, query redacted doctor/status, and execute at most one bounded
WorkspaceRead O-arm qualification sample followed by at most one independent
WorkspaceSearch O-arm qualification sample. The second sample does not start if
the first cannot reach a scheduler lease fairly.

## 4. Acceptance and stop rules

For each B0 O-arm sample, retain the public Task lifecycle and bounded
observation of candidate validation, scheduler lease, dispatch, verification,
and acceptance. A `DRAFT`, zero-lease, missing credential, candidate rejection,
or unavailable runner is a truthful non-pass result, not a product patch target.
No private transport injection, test-only caller, mock authority, manual SQLite
edit, or hand-built runner is permitted.

Stop the campaign for owner pause/scope change, an unlisted baseline/snapshot
change, secret-exposure risk, unknown guest concurrent modification, or after
the B0 qualification disposition. On any stop, append the result immediately,
clean campaign-owned runtime state, reconcile the active campaign row and EVAL
lease, and do not resume P2-T36 without a fresh owner delivery instruction.

# PERSONAL-PERF-EVAL-011 running report

- Campaign: `PERSONAL-PERF-EVAL-011`
- Lease: `lease/personal/EVAL-011/c1-c2-b0-qualification`
- Frozen source: `979e52e4c6681d0fc8c6431c965e3267a7a0d917`
- Target: `B01-Desktop-Linux-002`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Status: active B0 qualification; measurement-only

Results are appended immediately after every completed campaign unit.

| Cell | Status | Evidence |
|---|---|---|
| Owner activation and EVAL lease | pass | Owner explicitly confirmed a new isolated B01 C1/C2 campaign. EVAL-004 through EVAL-010 remain closed and excluded. The frozen product source is pushed revision `979e52e4c6681d0fc8c6431c965e3267a7a0d917`; no product code changes are permitted. |
| B01 baseline / snapshot observation | not-run | Next preregistered cell. Record guest state and snapshot list only; do not create, revert, delete, start, stop, or otherwise mutate snapshots. |
| Exact-source archive and binary freeze | not-run | Must originate from GitHub revision `979e52e4c6681d0fc8c6431c965e3267a7a0d917`, with digests recorded before guest execution. |
| New root, port, and SecretStore allocation | not-run | Allocate only `/home/hal9001/perfeval011-20260818`, port `48300`, reserved broker `48400`, and one new SecretStore item. Never reuse EVAL-004 through EVAL-010 assets. |
| Public daemon readiness | not-run | Public doctor/status is readiness evidence only, not C1/C2 completion. |
| B0 C1 WorkspaceRead O-arm | not-run | `retry=0`; at most one bounded sample after readiness. Record candidate validation, lease, dispatch, verification, and acceptance separately. |
| B0 C1 WorkspaceSearch O-arm | not-run | Starts only if WorkspaceRead reaches a fair scheduler-lease observation. |
| B0 P-arm / paired B1/B2 / C2a-C2d | not-run | Not authorized until B0 O-arm is complete, fair, and a future approved amendment freezes the missing assets. |
| Campaign cleanup | not-run | Stop only campaign-owned processes and clean only the new item/root as preregistered. |

## Unique next action

Record the read-only B01 guest baseline and snapshot list through the registered
host route before creating any EVAL-011 guest resource.

