# PERSONAL-PERF-EVAL-004 full OS-only scope amendment

- Campaign: `PERSONAL-PERF-EVAL-004`
- Amendment date: 2026-08-15
- Authorized lease: `lease/personal/EVAL-20260815/full-os-only-assessment`
- Frozen product source: `origin/main@93dde21da1635329bd11949b265f205ead46186b` (2026-08-15 freeze, historical)
- Re-freeze product source: `origin/main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd` (2026-08-16; archive `sha256:a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8`)
- Target: `B01-DESKTOP-002` / `B01-Desktop-Linux-002`
- Status: **closed 2026-08-17**. Re-freeze at `main@1e71344a` completed measurement. Final: [personal-performance-assessment-20260817-full-os-only.md](personal-performance-assessment-20260817-full-os-only.md). Independent reviewer `not_reviewed`. Do not reuse `/home/hal9001/perfeval004`, `48284`, or the 2026-08-15 SecretStore item.
- Claim ceiling: `hypothesis` / non-claim; independent reviewer: `not_reviewed`

This owner-authorized amendment expands the existing EVAL-004 scope; it does
not create a second campaign, lease, denominator, or conclusion for
`PERSONAL-PERF-EVAL-002`. The original parent plan remains the statistical and
safety contract. The C1/C2 addendum remains applicable and is extended by this
document.

## 1. Measurement-only boundary

The campaign may create and run only campaign fixtures, brokers, runners,
oracles, evidence, redactors, and reports. It must invoke existing public
product surfaces. It must not modify product code, contracts, schemas,
production negatives, product tests, generated handbook sources, or authority
semantics.

No paired `P/O` delta is calculated if the arms differ in tools, workspace,
oracle, or production caller; if pure Pi uses the daemon; if only one arm ran;
or if acceptance authority did not complete the Task. Started samples,
including timeouts, unknown outcomes, cleanup failures, and instrumentation
errors, remain in the denominator.

The campaign must not access, hash, copy, name in a command, or otherwise
process `C:\Users\wuron\Desktop\deepseek.txt`. Provider credentials may enter
only through guest graphical SecretStore input or a separately owner-approved
guest stdin-import flow. They must never enter argv, environment, ordinary
configuration, logs, evidence, raw responses, reports, or chat.

## 2. B0 capability-gate result: source and host facts

The exact requested source exists at `origin/main`. The local checkout is at a
later documentation-only campaign-registration revision and therefore is not
the campaign source. On 2026-08-15, the target route was qualified with a
non-interactive, strict-host-key, no-secret probe: local ->
`wuz@192.168.1.2` (libvirt host `hal9000`) -> ProxyJump ->
`hal9001@192.168.123.160`. Host system libvirt reports the target domain
`B01-Desktop-Linux-002` running (UUID `f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0),
with 2 vCPU and 4 GiB configured memory. Guest identity is
`hal9001-Standard-PC-Q35-ICH9-2009`, Linux `7.0.0-28-generic`, glibc `2.39`.
`B01-Clean-Linux-001` was observed only in the host list as shut off; it was
not contacted or operated. The later owner-operated hidden-input flow is
recorded in the preregistration; no secret material was exposed. No sample has
started.

Before a target contact, B0 must record guest identity, image/kernel/glibc,
vCPU/RAM, background processes, exact deployed revision, isolated campaign
root and loopback port, workspace/reset digests, provider/model snapshot,
timeouts/retry/max-turn, broker qualification, runner/corpus/oracle digests,
redaction scanner, hard counters, and cleanup procedure. Only
`virsh -c qemu:///system` may control the target domain; `B01-Clean-Linux-001`
is out of bounds. Snapshot revert/delete, P9-T04 daemon interruption,
`~/p9t04` mutation, owner-secret access, and system-level guest changes are
prohibited.

The isolated root, reserved port, frozen source archive, exact-source
`kernel-server` and `cognitive` CLI binaries, local Pi `0.81.1` runtime, and
extension are now qualified and fully listed in the EVAL-004 preregistration.
The campaign daemon is managed exclusively by its public `cognitive daemon
start` caller and is bound only to `127.0.0.1:48284`. Its public status is
honest post-entry readiness (`provider` and Pi ready,
`first_conversation_ready: true`); no Task, Tool, Effect, verification, or
sample has started. B0 remains partial pending the frozen pure-Pi broker and
equivalent fixture/oracle/runner/redactor assets.

## 2.1 B0 product-gap disposition

Public-contract inspection establishes that the configured daemon-private
candidate carrier persists only `parameters_digest`, while the production Tool
router requires persisted canonical Intent parameters: `query` for
WorkspaceSearch and `input_b64` plus `preimage` for WorkspaceWrite/Patch. No
public, schema-bound authority-safe route connects those parameter bytes to
the admitted Intent. The campaign may not substitute SQLite injection, private
transport injection, or test helpers.

Accordingly, positive C1 Search and C2a Write/Patch are `blocked/not-run`; real
C2b consumption and C2c reconcile remain `not-run`; C2d/O6 remains
`partial/not-run` without public terminal acceptance evidence. At most,
WorkspaceRead can later provide a limited caller-reachability proof, not a
completion claim. This is a product gap, so the campaign is paused before B1;
it requires an independent product task, branch, Draft PR, product lease,
supported validation, merge, and a newly frozen EVAL-004 revision.

## 3. Current capability and disposition matrix

Statuses below are current B0 classifications, not results. `qualified` means
eligible for B0 only; it never means the cell has executed.

| Cell | Historical status | Current product / production caller | Compliant observation | Campaign asset | Product work | Disposition |
|---|---|---|---|---|---|---|
| C1; T4 | `not-run` | Read reaches scheduler; Search needs missing persisted query | Task/resource boundary | read-only workspace, equivalent Pi adapter, oracle | yes | Read-only limited proof possible; Search `blocked/not-run` |
| C2a; A3; T5 | `not-run` | router supports preimage/atomic publish but public carrier lacks payload/preimage | task boundary, fixture oracle | resettable workspace, hidden diff/test oracle | yes | positive execution `blocked/not-run` |
| C2b; A6; S4; S8 | `not-run` | governed consumer and exact pins exist but run privately | lifecycle and Task boundaries | two-session reuse corpus | observation | real consumption `not-run` |
| C2c; A7; T9 | `not-run` | durable workspace reconcile exists; generic external closure is not qualified | no public reconcile trace | original-key fault fixture | observation / possibly yes | `partial`; general case `not-run` |
| C2d | `not-run` | verifier/CAS/acceptance path exists privately | no durable public terminal evidence | mechanical oracle, completion collector | observation | `partial/not-run` |
| O2 | `not_available` | authorization and reauthorization exist | no public redacted decision surface | future probe corpus | observation | `not_available` |
| O3 | `not_available` | cache/compaction implementation exists | no cache/compaction observation surface | future cache corpus | observation | `not_available` |
| O4 | `not-run` | scheduler, budget, lease, and fencing have production paths | no fairness, queue, or fence telemetry | multi-runnable workload | observation | capability `partial`; measurement `not_available` |
| O5 | `not-run` | durable Intent/Effect applies to supported tools | no public Effect history | mutation/fault oracle | observation | C2a-limited `partial` |
| O6 | `not-run` | independent verification and acceptance are wired for C1/RegisteredCheck | terminal outcome | verifier/CAS collector | no | C1-supported; general `partial` |
| O10 | historical only | management lifecycle exists | routine management surface | preregistered lifecycle procedure | no | `not-run` pending procedure |
| O11; UJ3 | historical `pass` | six-resource projection and bounded replay exist | public bounded surface | projection/replay oracle | no | qualified public smoke |
| O12 | historical hard-counter pass | SecretStore fail-closed path exists | redaction scan and management status | non-logging entry and cleanup | no | B0 hard condition |
| O13 | historical partial/pass | bounded public replay exists; full audit chain is internal | public bounded replay only | replay/cursor oracle | observation | `partial` |
| O14 | `not_available` | backup planning/migration facilities exist, but no user restore path | no user CLI/API | none permitted | yes | `not_available` |
| T3 | `not-run` | candidate tool-selection path exists | catalog/projection | frozen selection pilot | no | optional B1, otherwise `not-run` |
| T6 | `not-run` | ProcessCheck is production-carried but no supervisor registry supports success | refusal only | negative fixture | yes for positive scope | positive `not-run`; negatives eligible |
| T7 | `not-run` | HTTP executor is production-carried but origin allowlist is empty | refusal only | negative fixture | yes for positive scope | positive `not-run`; negatives eligible |
| T8 | `not-run` | descriptor drift denial is available | refusal boundary | drift negative fixture | no | B3 eligible after B0 |
| B3 | `partial` | fault eligibility depends on each qualified fixture | public refusal/status | fault manifest | no, except blocked positives | per-fault denominators |
| B4 | historical `pass` | public management/resource paths exist | public surfaces | mixed-workload sampler | no | qualified after C1/C2 gates |
| B5 | 1 h/8 h historical pass; 24 h `not-run` | no new result | public local observations | soak schedule | no | 1 h, then 8 h; 24 h conditional |
| UJ2 | `partial` | route exists; cold instrumentation was historically defective | lifecycle/route | cold/warm runner | no | B0 instrument qualification |
| UJ4 | historical `pass` | admission plus C1 same-process completion path exists | Task admission/watch | completion and restart-split oracle | no | split dispositions |
| UJ6 | register only | capability truth is documentable | final register | matrix and evidence manifest | no | final coverage register |

O2 and O3 cannot use raw SQLite, test helpers, or internal diagnostics as a
substitute. O4 is no longer described as an absent scheduler, but its required
global-picker/fairness evidence has no compliant public surface. O14 remains a
product gap because P7-T02's internal/admin facilities are not a user-reachable
archive-and-restore journey. Positive T6 and T7 are outside supported scope;
the campaign may only exercise their fail-closed negatives.

## 4. Gate and execution sequence

For each cell, B0 classifies: capability and observation present -> execute;
capability present but observation absent -> `not_available`; capability absent
-> `blocked`/`deferred` and formal product task; partial -> execute only the
provable portion. A discovered product gap pauses affected cells. Product work
must use a separate task, branch, Draft PR, lease, supported validation, merge,
and a newly frozen campaign revision before rerun.

After B0: freeze campaign-only assets; run B1 pilots without selecting seeds or
changing the oracle; then B2 with 30 held-out seeds for each applicable C1,
C2a, C2b, C2c, and C2d class. Run B3 fault/recovery only with controlled
fixtures; B4 at concurrency 1/8/16; B5 for 1 h, then 8 h only if clean, and
24 h only on an unresolved 8 h slope with owner budget. Every sample records
the identifiers, digests, arm, timestamps, outcome, retries, tool/effect keys,
verification, acceptance, and cleanup specified by the parent plan.

The final assessment is
`docs/evaluation/personal-performance-assessment-20260815-full-os-only.md`.
It will separately retain all completion/failure/unknown/timeout denominators,
`not-run`, `not_available`, `partial`, and `instrument_error` records. It may
state only hypothesis/non-claim findings pending independent review.
