# P1-T09 B01 clean-Linux campaign preregistration

- Date: 2026-07-31
- Task: P1-T09 install-to-first-conversation route
- Gate: B01 first-install/first-conversation
- Lease: `lease/personal/P1-T09/b01-preregistration`
- Change class: implementation-only documentation closure
- Task status: `in-progress`
- Development track: `experimental-local-only` for prior route evidence only
- Implementation evidence: `tested-local` for campaign `.4` route evidence only
- Gate status: `not-run`
- Claim scope: B01, GMVP-LINUX, release, and Profile remain non-claim

## Purpose and non-substitution rule

This record pre-registers what must be fixed before a B01 attempt may start.
It does not start an attempt, reserve an environment, select a Provider,
configure a secret, install an artifact, or run Pi. In particular,
`personal-linux-native-01` and campaign `.4` are experimental implementation
evidence; they are explicitly excluded from B01 evidence.

The first response observed in the campaign `.4` experimental route is an
implementation prerequisite for B01, not a B01 result. Rust daemon authority,
SecretStore ownership, persist-before-dispatch, and independent completion
semantics remain unchanged.

## B01 campaign contract

| Field | Pre-registered value | Status |
|---|---|---|
| Campaign identity | `B01-clean-linux-first-install-first-conversation-001` | reserved; not started |
| Product scope | One clean Linux x86_64 first install through one bounded first response | fixed |
| Environment | A new, dedicated clean Linux x86_64 VM; non-WSL; native user-systemd; no prior CognitiveOS Personal deployment or Pi runtime state | unallocated |
| Artifact | A future immutable artifact produced from a reviewed `main` commit, with exact source commit, SHA-256, signature, trusted-key/version, and Pi pin captured before attempt start | unselected |
| Pi | `@earendil-works/pi-coding-agent@0.81.1`, exact executable version observed after clean installation | required; not run |
| Secret Service | Native supported Secret Service, probed before opt-in; Provider/user credential enters only via approved SecretStore hidden-input flow | required; not run |
| Provider opt-in | A designated operator enters a real credential only through the approved hidden-input flow on the B01 VM; no credential is copied to chat, shell argv, ordinary config, logs, evidence, or Git | operator action required |
| Workload | One fixed, non-sensitive prompt requesting the configured non-secret expected marker; prompt text and response body are not retained | fixed |
| Route runner | Reviewed `tools/personal/p1-t09-product-route-smoke.sh` invoked with absolute installed paths, explicit `--extension`, bounded timeout, and closed stdin | required; not run |
| Attempt accounting | Every invocation after the clean-reset checkpoint is an attempt. A timeout, nonzero exit, missing marker, readiness failure, setup fault, or cleanup failure is recorded as a failure; retries receive a new attempt number | fixed |
| Success threshold | One started attempt, one expected marker, one bounded response, zero authority side effects, and no unredacted secret/internal material; no retries are permitted for a passing B01 claim | fixed |
| Redaction | Retain only attempt number, phase, fixed error class, exit category, bounded duration, boolean marker/response/authority fields, artifact identity, and cleanup result | fixed |
| Cleanup | Remove the test installation, user service, campaign artifact copies, Pi runtime/config state introduced by the attempt, temporary files, and operator-entered secret through the approved SecretStore deletion flow; record booleans only | required; not run |

## Start gate checklist

All items must be recorded as `pass` before attempt 1 may start:

1. Owner allocates a new clean VM and provides only its non-secret access
   endpoint and a declaration that it is not `personal-linux-native-01`.
2. Runner records Linux x86_64, non-WSL, native user-systemd, supported native
   Secret Service, clean state, and a disposable directory.
3. A reviewed `main` artifact is selected; its immutable version, source commit,
   SHA-256, signature, trusted key metadata, and expected Pi compatibility are
   independently verified before installation.
4. The campaign record names the operator who will perform the hidden-input
   Provider credential opt-in and the independent verifier for the B01 result.
5. The clean/reset snapshot, workload, timeout, attempt ledger location,
   redacted evidence collector, and cleanup script/procedure are recorded.
6. The B01 runner is reviewed as a formal Gate runner rather than reusing the
   experimental route runner result or its host.

## Current blocker record

- `blocked_paths`: B01 dedicated clean-VM allocation, formal campaign runner,
  artifact selection, reset procedure, independent verifier assignment, and
  operator-owned SecretStore opt-in.
- `blocked_task_ids`: `P1-T09`.
- `blocked_gate_ids`: `B01`, `GMVP-LINUX`, and Profile.
- Owner: product owner for VM allocation and credential opt-in; P1-T09 runner
  owner for formal runner and evidence collection.
- Next action: the product owner must allocate a new clean Linux VM and name
  the B01 operator and independent verifier. After that, the runner owner can
  claim a dedicated B01 execution lease, register immutable artifact identity,
  and run the start-gate checklist before attempt 1.

## Checks for this preregistration slice

| Check | Result |
|---|---|
| `pnpm run check:consistency` | pass |
| `git diff --check` | pass |
| B01 environment allocation | not-run; requires owner action |
| B01 attempt | not-run; start gate is intentionally incomplete |
