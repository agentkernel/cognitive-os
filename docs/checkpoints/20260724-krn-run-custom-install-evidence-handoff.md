# KRN/RUN Custom installation evidence carrier handoff

## Completed

- `InstallationEvidence` is committed atomically with the existing package,
  adapter, sandbox and compatibility digests. Custom evidence binds the
  authenticated `principal://` operator, immutable `file://` bundle, lockfile
  digest and `custom_acknowledgement_bound` result; no extra approval, REQ,
  error code or installation state table was introduced.
- Existing SQLite databases are upgraded in place by adding nullable evidence
  columns to the two existing installation tables. Legacy commits remain
  readable but have no Custom evidence and cannot be presented as confirmed
  Custom installs.
- `DurableInstallationManager` exposes the committed record query; the CLI now
  reads that durable record before emitting Custom source data.

## Test evidence

- Test-first KRN acceptance initially failed because `InstallationEvidence`,
  `new_with_evidence`, and `evidence()` did not exist. It now proves staging is
  invisible, commit is atomic, and Custom evidence survives a reopen.
- Local pass: cognitive-store acceptance 5/5; cognitive-runtime 45/45;
  admin-cli Custom integration 2/2; strict clippy for KRN/RUN/CLI, fmt, and
  diff checks.
- Conformance request fixture was updated only for the added runtime input;
  vectors and pins are unchanged. No Profile claim.

## Remaining

- Push and wait for Windows/Ubuntu CI. After merge, move to actual Linux-native
  Pi containment, then six adapter mappings, OOB/recovery behavior execution and
  the fixed-platform PERF-004 campaign. Official provenance remains blocked until
  a real attestation verifier exists.
