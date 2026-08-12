# P9-T04 comprehensive performance campaign preregistration

- Status: preregistered; offline layers executed, Provider-dependent layers not
  started
- Task: `P9-T04`
- Campaign ID: `P9-T04-comprehensive-performance-001`
- Campaign lease: `lease/personal/P9-T04/comprehensive-performance-campaign`
- Branch: `personal/P9-T04-comprehensive-performance-campaign`
- Source revision: `9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c`
- Environment for Provider-dependent layers (`L3`-`L5`): `B01-Desktop-Linux-002`
- Environment for offline layers (`L0`-`L2`): `DEV-LINUX-NATIVE-01`, per the
  execution plan's main performance environment and its prohibition on using
  the B01 campaign guest as an ordinary benchmark host. The offline runner
  cannot declare a B01 environment, so an offline result can never be relabelled
  as B01 evidence.
- Operator: standing owner-authorized campaign operator
- Independent verifier: separate redacted-evidence review before disposition
- Claim ceiling: `hypothesis` until all applicable report-policy conditions
  are independently verified; no Gate, release, or Profile claim

## Start gate

The guest was observed shut off before registration. No snapshot was restored,
guest state changed, artifact installed, Provider configured, service started,
or benchmark executed by this preregistration.

Before the first campaign action, the operator must record all of the following
in the ignored campaign artifact root and redacted report:

1. guest baseline snapshot name and reset command outcome;
2. exact source revision and independently checked artifact digest;
3. OS, kernel, glibc, CPU, RAM, disk, filesystem, governor, thermal, and
   background-load observations;
4. Rust, Node, pnpm, Pi, adapter, Extension, Tool, Skill, Provider/model, and
   selected-model pins or their explicit `not_available` values;
5. disposable Git worktree path with `git rev-parse HEAD` equal to the source
   revision;
6. campaign directory, redaction collector, cleanup plan, and no-secret scan;
7. approved SecretStore availability and the graphical hidden-input procedure;
8. fixed denominator, warmup exclusions, timeout/retry policy (`retry=0` for
   Provider requests), fault profile, budget, and randomized run order.

Any missing start-gate fact blocks B01 execution. It does not permit fallback
to an ordinary development host while labelling the result B01 evidence.

## Denominators and execution order

The campaign retains every started sample, including timeout, denial,
rate-limit, unknown outcome, quarantine, manual intervention, and environment
invalidity. Warmups are excluded before a cell begins and are not later
reclassified. Each deterministic cell has at least the preregistered number of
independent runs and samples from the execution plan; L3-L5 formal sample
counts are those declared in the frozen manifest.

Order: D01 measurement runner and negatives; L1/L2 baseline; L3 Provider/Pi;
L4 governed scenarios and 1 h then 8 h then eligible 24 h soak; L5 W1/W2
A/B/C/D only after the A-arm secret boundary, task set, manifest, and power
analysis are independently reviewed.

## Secret, evidence, and cleanup boundaries

The DeepSeek key is imported only by an operator through graphical hidden
input into the guest approved SecretStore. It is never read, copied, hashed,
passed in argv or environment, emitted in logs, or included in evidence.
Provider traffic, prompts, responses, headers, SecretRefs, and sensitive
SQLite contents are excluded from the collector.

Raw payloads are confined to ignored `artifacts/performance/<run-id>/` or an
approved external store. Git records only redacted facts, digests, attestation
references, reports, and non-claims. Cleanup stops campaign processes, removes
campaign state, temporary secret carriers and the campaign SecretStore entry,
checks for orphan processes/sockets/locks, and restores the guest baseline
shut off. It never deletes the owner-local Desktop source file.

## Execution record

### Offline layers, 2026-08-12

One `L0`-`L2` execution completed on `DEV-LINUX-NATIVE-01` at exact pushed
revision `ba141838c4949a6a16a95aa581ac6e3129a6cdb2` in a disposable Git
worktree. The redacted report is
`sha256:c18a63db97400df8c254e2fe5ee6195826203b8183187501075505bb528e83be`; the
payload stays outside Git in the operator's ignored campaign artifact root.

| Layer | Disposition | Started | Retained | Warmups excluded |
|---|---|---:|---:|---:|
| `L0` eligibility | completed | 1 | 1 | 0 |
| `L1` module benchmark | completed | 200 | 200 | 3 |
| `L2` governed path and store access | completed | 52 | 52 | 0 |
| `L3` Provider route | not-run | 0 | 0 | 0 |
| `L4` governed Task scenarios | not-run | 0 | 0 | 0 |
| `L5` benefit campaign | not-run | 0 | 0 | 0 |

All seven hard safety counters are zero, cleanup reported no orphan process,
socket, or stale lock, and the owner's Provider source file was never read or
touched. Claim level is `hypothesis`, `benefit_claimed` is false, and the
independent verifier disposition is `not_reviewed`.

Hypothesis-only observations worth carrying into the remaining layers: governed
`effect_persistence` dominated the path at roughly 265 ms cold and 242 ms warm,
consistent with the `P9-T01` finding that the stage aggregates SQLite open,
admission, persist, and reload rather than transport; and a per-open store read
cost roughly 121 ms against roughly 0.7 ms for the long-lived handle over 50
iterations, consistent with the `P9-T03` composition change. Neither is a
release, Gate, Profile, or Agent-benefit claim.

### B01 guest access finding, 2026-08-12

Under this preregistration and the active campaign lease, the guest was
reverted to `b01-platform-qualified-baseline`, started, observed at its NAT
address, and returned to the recorded shut-off baseline. No product was
installed, no Provider was configured, no benchmark ran, and no guest state
was left changed.

The observation that matters for planning: the guest exposes no non-interactive
access path. SSH is refused for the host account and the QEMU guest agent is
not connected, so an automated session cannot drive the guest at all. A B01
`L3`/`L4` attempt is therefore an operator-driven procedure at the graphical
console from start to finish — install, daemon start, Provider import, run, and
cleanup — not an automated run with one manual credential step. Establishing a
campaign-scoped access path inside the guest would itself change the qualified
baseline and needs its own owner decision.

### Provider-dependent layers

`L3`-`L5` remain not-run. `L3` and `L4` require the preregistered B01 start-gate
facts and an operator-performed graphical hidden-input Provider import into the
guest approved SecretStore, which a non-interactive session cannot perform.
`L5` additionally requires the A-arm baseline decision recorded in the execution
plan: an isolated non-product approved secret broker, or an owner-designated
native Agent baseline that satisfies the SecretStore boundary. Until that
decision exists, `L5` stays `blocked/not-run` and the daemon Provider proxy is
never presented as an "A arm without CognitiveOS".

## Abort and recovery

Secret exposure, duplicate external Effect, false completion, stale-epoch
commit, authority-writer bypass, unreconciled mutation, incomplete denominator,
manifest mismatch, sensitive evidence, direct authority write by the runner,
or B01 procedure violation stops claim promotion. The operator preserves
redacted facts, performs the defined safety recovery and cleanup, and records
the result rather than deleting samples.

