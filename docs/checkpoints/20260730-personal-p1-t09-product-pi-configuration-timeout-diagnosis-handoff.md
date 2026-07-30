# Personal P1-T09 product Pi configuration timeout diagnosis handoff

## Record metadata

- record_type: historical-handoff
- project_id: cognitiveos-personal
- task_id: P1-T09
- lease_id: `lease/personal/P1-T09/product-pi-configuration-timeout-diagnosis`
  (closed)
- status_at_handoff: in-progress
- gate_status_at_handoff: B01 not-run
- claim_scope_at_handoff: non-claim
- current_status_source: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` and
  `docs/plan/PROGRESS.md` Current snapshot
- supersedes: `20260730-personal-p1-t09-real-provider-prerequisites-handoff.md`
- superseded_by: none-known-at-write-time

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Classification: corrective evidence; normative surface unchanged
- Branch: `lane/personal-p1-t09-product-pi-configuration-timeout-diagnosis`
- Remote visibility: pending atomic blocker-record push

## Bounded diagnosis

The SSH-qualified experimental Linux host remains Linux x86_64 with a native
active `cognitiveos-personal.service`. The exact Pi executable remains
available and reported `0.81.1` when invoked directly.

The required product route cannot yet be configured or exercised because the
same host has neither of the two non-secret product artifacts required by the
formal route:

1. a product `cognitive` CLI executable; and
2. a deployed built CognitiveOS Extension entry.

The product route therefore has no safe absolute paths for `cognitive pi
configure`. No replacement source-built CLI or temporary Extension was written
into the restored product layout, because that would not diagnose the product
bundle and could hide a daemon/bundle version mismatch.

## Verification

- Remote native user service activity -- passed.
- Remote exact Pi version probe -- passed: `0.81.1`.
- Product `cognitive` CLI availability -- absent.
- Deployed built CognitiveOS Extension entry availability -- absent.
- `cognitive pi configure` -- not-run: required product CLI and Extension
  paths are absent.
- `cognitive doctor` -- not-run: required product CLI is absent.
- `cognitive pi launch` -- not-run: required product CLI and configuration are
  absent.
- Bounded direct Pi first-response verification -- not-run: the product
  Extension entry is absent, so no formal product invocation can be made.
- Native Secret Service mutation, Provider configuration inspection, SecretRef
  read, bootstrap-secret read, SQLite inspection, selected-model digest read,
  key/token output, and model-response output -- not-run by design.
- Full workspace tests, B01, GMVP-LINUX, release, and Profile verification --
  not-run.

## Bounded blocker and non-claims

- `blocked_paths`: product bundle deployment paths on
  `personal-linux-native-01`.
- `blocked_task_ids`: `P1-T09`.
- `blocked_gate_ids`: `B01`, `GMVP-LINUX`, and Profile.
- owner: P1-T08 bundle/release artifact owner.
- next action: deploy a coherent product bundle that contains the `cognitive`
  CLI and the built CognitiveOS Extension entry, then use the formal
  non-secret configuration route with their absolute paths and rerun redacted
  doctor, launch, Pi diagnostics, and one bounded first-response check.

P1-T09 remains `in-progress`; B01, GMVP-LINUX, and Profile remain `not-run`.
This is only `experimental-local-only` bounded blocker evidence. It makes no
Pi-usability, first-conversation, Gate, release, or Profile claim. No Task,
Effect, Verification, capability, or authority side effect was created.
