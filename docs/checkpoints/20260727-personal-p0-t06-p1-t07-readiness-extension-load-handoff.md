# 20260727 Personal P0-T06 / P1-T07 Readiness + Extension-Load Handoff

## 1. Session snapshot

- Date: 2026-07-27.
- Branch: `lane/personal-p1-t07-pi-readiness`.
- Scope: close out P0-T06 with real Linux-native extension-load evidence; complete the P1-T07 `readiness.rs` `pi` component flip.
- Status: P0-T06 is now **done**; P1-T07 remains **in-progress** because provider proxy / production `ProviderTransport` is still pending.

## 2. Completed work

### P0-T06

- Ran the real `extension-load` probe on the Linux-native host `wuz@192.168.1.2`.
- Retrieved and verified the redacted evidence record.
- Evidence fields confirmed:
  - `extension_command_registered=true`
  - `session_start_hook_observed=true`
  - `status_command_observed=true`
  - `status=executed`
  - `raw_output_included=false`
  - `output_redacted=true`
  - `authority_committed=false`
  - `effects_created=false`
  - `task_transitions=0`
  - `capabilities_granted=0`
- This remains PoC / non-claim evidence only.

### P1-T07

- Flipped `apps/kernel-server/src/personal/readiness.rs` so the `pi` component now reads a real runtime observation from `pi.json`.
- Added/updated focused integration tests in `apps/kernel-server/tests/p1_t07_pi_readiness.rs`.
- Kept ADR-0023 aggregation unchanged; only the `pi` component’s observation changed.

## 3. Verification

- `cargo test -p kernel-server --test p1_t07_pi_readiness --locked` ✅
- `cargo test -p kernel-server --locked` ✅
- `pnpm run check:consistency` ✅
- `git diff --check` ✅

## 4. Evidence boundary

- No containment, Profile, release, or Gate claim.
- No raw transcript stored in the repository.
- WSL2 results remain local-only and are not Linux-native evidence.

## 5. Remaining work

1. P1-T07 daemon-side provider proxy route and production `ProviderTransport`.
2. P1-T08 Linux bundle installer / user service path.
3. Continue critical path with the next available task once P1-T07 is either completed or explicitly blocked.

## 6. Next prompt

"Continue from `20260727-personal-p0-t06-p1-t07-readiness-extension-load-handoff.md`: finish the remaining P1-T07 provider proxy decision/batch, then advance to the next unblocked Personal task on the critical path."
