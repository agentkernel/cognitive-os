# Personal P1-T09 Linux real Provider prerequisites handoff

## Record metadata

- record_type: historical-handoff
- project_id: cognitiveos-personal
- task_id: P1-T09
- lease_id: `lease/personal/P1-T09/linux-real-provider-prerequisites` (closed)
- status_at_handoff: in-progress
- gate_status_at_handoff: B01 not-run
- claim_scope_at_handoff: non-claim
- current_status_source: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` and
  `docs/plan/PROGRESS.md` Current snapshot
- supersedes: `20260730-personal-p1-t09-exact-pi-extension-load-handoff.md`
- superseded_by: none-known-at-write-time

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Classification: corrective and implementation-only; normative surface unchanged
- Branch: `lane/personal-p1-t09-real-provider-prerequisites`
- Remote visibility: pending atomic closure push

## Configuration and connectivity facts

1. `personal-linux-native-01` has a native user D-Bus, reachable FreeDesktop
   Secret Service, and `/usr/bin/secret-tool`. Its local `secret-tool` rejects
   both `--version` and `--help` with usage exit status `2`; production backend
   selection consequently failed even though the secret service was reachable.
2. `LinuxSecretToolStore` now checks that a non-sensitive `secret-tool lookup`
   process can be started, rather than treating unsupported version/help flags
   as a backend failure. Real `store` remains the authoritative fail-closed
   write operation.
3. After the correction, an operator supplied a DeepSeek key only at the
   product CLI hidden-input prompt. `cognitive init` reported redacted native
   `linux-secret-tool` storage, `deepseek` configuration, and selected model
   `deepseek-v4-flash`. No secret, token, SecretRef, or config file contents
   were recorded here.
4. A current-source temporary daemon published its normal loopback endpoint
   and local bootstrap, then a bounded probe using the same
   `PersonalDaemonClient` as the Pi Extension completed one daemon-owned
   Provider-proxy request. It observed `finish_reason: stop`, the expected
   short reply marker, and `authority_side_effects: false`. The probe does not
   display the model response.
5. The original user service was restored after the smoke; the temporary
   source-built daemon was stopped. The long-lived credential remains in native
   Secret Service, while an updated product bundle/daemon is still required for
   the complete current P1-T09 route.

## Verification

- Remote `cargo test -p cognitive-secret --test p1_t02_provider_secret --locked` — passed, 10/10.
- Remote current-source builds — passed: `admin-cli` `cognitive` and
  `kernel-server`.
- `cognitive doctor` before Pi configuration — secret and provider components
  ready; Pi remains `not_configured` and `first_conversation_ready: false`.
- Native SecretStore initialization/discovery — passed with redacted success
  output and `deepseek-v4-flash` selected.
- Bounded daemon-owned Provider-proxy smoke — passed: response received,
  expected marker observed, `finish_reason: stop`, and no authority side effect.
- Direct exact Pi `0.81.1` explicit-Extension `--print` smoke — failed to
  complete within 90 seconds; zero captured response bytes. It is not accepted
  as Pi usability or first-conversation evidence.
- Original campaign user service restoration and loopback health — passed.
- Full workspace test, secret scan, formal Pi configuration/launch, B01,
  release, and Profile verification — not-run.

## Status and explicit non-claims

- `P1-T09`: `in-progress`; this adds `tested-local` native-secret and
  daemon-owned real Provider connectivity evidence.
- The configured credential is persistent in the native Secret Service and is
  not an environment variable, command-line value, normal config field, SQLite
  value, log, or evidence artifact.
- Pi remains unconfigured and its direct real smoke timed out. Therefore no
  Pi usability, product first conversation, B01, release, GMVP-LINUX, or
  Profile claim is made.
- The probe created no Task, Effect, Verification, capability, or authority
  state side effect.

## Next action

This lease is closed with the corrected SecretStore probe and the redacted
Provider-proxy connectivity record. Next, claim a non-overlapping P1-T09 slice
to configure the product Pi executable and Extension entry through the
non-secret `cognitive pi configure` route, diagnose the direct Pi timeout, and
rerun one bounded, redacted real Pi response.
