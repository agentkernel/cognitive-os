# P9-T04 comprehensive performance campaign preregistration

- Status: preregistered; execution not started
- Task: `P9-T04`
- Campaign ID: `P9-T04-comprehensive-performance-001`
- Campaign lease: `lease/personal/P9-T04/comprehensive-performance-campaign`
- Branch: `personal/P9-T04-comprehensive-performance-campaign`
- Source revision: `9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c`
- Environment: `B01-Desktop-Linux-002`
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

## Abort and recovery

Secret exposure, duplicate external Effect, false completion, stale-epoch
commit, authority-writer bypass, unreconciled mutation, incomplete denominator,
manifest mismatch, sensitive evidence, direct authority write by the runner,
or B01 procedure violation stops claim promotion. The operator preserves
redacted facts, performs the defined safety recovery and cleanup, and records
the result rather than deleting samples.

