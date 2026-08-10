# P5-T01 Agent and sidecar package acquisition/install lifecycle closure

## Task boundary

- Task: `P5-T01`
- Slices: `D01-D03`
- Branch: `personal/P5-T01-pi-acquisition`
- Lease: `lease/personal/P5-T01/pi-acquisition`
- PR: `#178`
- Scope: daemon-private official Pi acquisition, versioned activation, rollback,
  and stopped/absent uninstall quarantine only

## Acceptance mapping

- D01 authenticates the fixed official Pi identity `@earendil-works/pi-coding-agent`
  at `0.81.1`, fixed npm origin, npm SRI, independently computed SHA-256,
  dependency-lock digest, Node `>=22.19.0`, adapter/sandbox/compatibility pins,
  and signed acquisition-lock reference before atomic durable commit.
- D02 persists immutable versioned installation-root bindings and an active
  pointer from committed official locks only, with expected-version fencing,
  failed-health/compatibility rejection, upgrade preservation, and incomplete
  rollback rejection.
- D03 requires an explicit stopped or absent lifecycle observation, clears only
  the fenced active pointer into quarantine, and preserves immutable acquisition
  evidence, user data, secrets, and unrelated installation references.
- The authenticated admin caller remains management-session authorized and the
  official path grants zero capabilities, creates zero Effects, and completes
  zero Tasks.

## Failure-first coverage

- wrong identity, version, origin/redirect, SRI, independent digest,
  dependency-lock digest, Node compatibility, and signed-lock reference;
- uncommitted/non-official lock activation;
- compatibility/health rejection before pointer publication;
- competing activation CAS conflict;
- failed upgrade preserving the complete active binding;
- incomplete rollback target without a success receipt;
- missing pointer, wrong root, active lifecycle, stale uninstall fence, and
  partial-uninstall/no-success-receipt negatives;
- evidence/data/secret/unrelated-installation preservation after quarantine.

## Validation

- `cargo fmt --all`: passed locally.
- `git diff --check`: passed locally.
- `pnpm run check:consistency`: passed locally.
- Exact native Linux validation at `3413598e19746807674c31b12bc7814a848edcdf`:
  runtime focused suite 10/10, installation-store suite 9/9, admin-install
  integration suite, and Clippy for runtime/store/admin CLI passed.
- Required Ubuntu and Windows CI for final revision `3413598e19746807674c31b12bc7814a848edcdf`:
  passed in run `31355388291`.
- Local Windows Rust build/test/Clippy were not run because the registered GNU
  linker failure makes that environment unsupported for Rust validation.

## Explicit non-claims

This closure does not claim AgentInstance registration, sidecar sessions,
process supervision, provider execution, Effects, Task completion, B09, Gate,
release, Profile, production publisher-signature verification, or public
installation lifecycle contracts. D01's accepting verifier remains a test
fixture seam, not publisher provenance proof.
