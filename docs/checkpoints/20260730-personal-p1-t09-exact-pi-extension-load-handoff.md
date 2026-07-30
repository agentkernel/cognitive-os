# Personal P1-T09 exact Pi Extension-load handoff

## Record metadata

- record_type: historical-handoff
- project_id: cognitiveos-personal
- task_id: P1-T09
- lease_id: `lease/personal/P1-T09/exact-pi-extension-load` (closed)
- status_at_handoff: in-progress
- gate_status_at_handoff: B01 not-run
- claim_scope_at_handoff: non-claim
- current_status_source: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` and
  `docs/plan/PROGRESS.md` Current snapshot
- supersedes: `20260730-personal-p1-t09-provider-fixture-handoff.md`
- superseded_by: none-known-at-write-time

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Classification: implementation-only; normative surface unchanged
- Branch: `lane/personal-p1-t09-exact-pi-extension-load`
- Remote visibility: pending atomic closure push

## Completed local observation

1. Reconfirmed the disposable `personal-linux-native-01` host is Linux x86_64,
   non-WSL, has native user-systemd, Rust `1.97.1`, and Node `22.19.0`.
2. Used an uncredentialed exact-package invocation to execute actual
   `pi --version`, which emitted exact `0.81.1`.
3. Built `@cognitiveos/pi-cognitiveos` with
   `pnpm --filter @cognitiveos/pi-cognitiveos build`, staged the complete
   `dist/` directory in a disposable remote `/tmp` directory, and confirmed its
   ESM entry point imported with default export
   `registerCognitiveOsExtension`.
4. Ran exact Pi `0.81.1` with explicit `--extension <absolute-path>`. A
   session-local wrapper was the extension argument; it writes a 0600 marker
   and immediately delegates to the actual built CognitiveOS default export.
   The observed marker `PI_EXTENSION_DEFAULT_EXPORT_INVOKED` confirms that Pi
   invoked the production Extension entry point. The production function
   synchronously registers its `project_trust`, `tool_call`, `session_start`,
   and `/cognitive-status` surfaces before awaiting the daemon-selected Provider.

## Isolation and outcome

The child used empty disposable XDG roots, a disposable runtime directory,
`PI_OFFLINE=1`, no extension discovery, no skills, no prompt templates, no
themes, no context files, no session persistence, and no tools. It received no
Provider configuration, Provider/user secret, SecretRef, SQLite path,
selected-model material, or authority state. The wrapper's marker was the only
session-local file observation and was deleted at cleanup.

Because the isolated child intentionally had no daemon endpoint or bootstrap
material, print mode did not establish a conversation and reached its 45-second
timeout (Pi exit `124`). That outcome is explicitly not a daemon-readiness,
Provider, session, command-output, or first-response success. No Task, Effect,
Verification, capability, or authority side effect was created.

## Verification

- `pnpm --filter @cognitiveos/pi-cognitiveos build` — passed.
- Exact Pi command: `npm exec --yes --package=@earendil-works/pi-coding-agent@0.81.1 -- pi --version` — passed; output `0.81.1`.
- Linux-native observation probe — passed its import and Pi-invocation criteria;
  actual print-mode subcommand intentionally timed out (exit `124`) without a
  daemon, as recorded above.
- IDE diagnostics for `tools/personal/p1-t09-observe-extension-load.sh` — none.
- `pnpm run check:consistency` — passed: 273 requirements, 55 error codes, 63
  schemas, 85 vectors, links, traceability, project identity, and leases.
- Full workspace test, secret scan, real Provider conversation, native Secret
  Service smoke, B01, release and Profile verification — not-run.

## Status and explicit non-claims

- `P1-T09`: `in-progress`; `development_track: experimental-local-only`.
- This atomic slice supplies `tested-local` exact-Pi and real Extension default
  export invocation evidence only. The prior deterministic Provider fixture
  remains `tested-supported-ci` evidence.
- `B01`: `not-run`; `GMVP-LINUX`: `not-run`; Profile `implemented`: `0`.
- This does not claim a real daemon session, Provider conversation, first
  response, native Secret Service result, clean Linux VM campaign, Gate,
  release, or Profile conformance.

## Lease closure and next action

- This lease is closed with the evidence records in the formal plan and Current
  snapshot. Normative assets were not touched.
- Next action: claim a non-overlapping P1-T09 route slice for a real first
  response plus native Secret Service smoke, preserving the daemon-owned
  SecretStore and Provider boundaries. Do not pre-register or execute B01 until
  those route prerequisites are complete.
