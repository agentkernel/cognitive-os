# P7-T08/D03 GMVP-LINUX composition evidence (ADR-0049)

- Task: `P7-T08`
- Slice: `P7-T08/D03`
- Campaign: `GMVP-LINUX-composition/1`
- Policy: ADR-0049 MVP fixed composition
- Binder revision (this checkpoint commit): pending push after docs sync
- B08 matrix revision: `65a736cd00a4b3da39a96be799ed6b60e434eeac`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/194
- Date: 2026-08-11

## Gate composition bindings

| Observation | Bound evidence |
|---|---|
| b01_mvp_pass | ADR-0039 successor `002` / P1-T09 closure |
| b02_mvp_pass | ADR-0046 / P2-T08 closure |
| b03_mvp_pass | ADR-0040 / P3-T06 closure |
| b04_mvp_pass | ADR-0046 / P2-T08 closure |
| b05_mvp_pass | ADR-0046 / P2-T08 closure |
| b08_mvp_pass | ADR-0048 / P7-T08/D02 disposition |
| b09_mvp_pass | ADR-0047 / P5-T05 closure |
| b12_mvp_pass | ADR-0046 / P2-T08 closure |

## UCR-01 assertion bindings (MVP; non-claim)

| Observation | Bound evidence |
|---|---|
| required_recall | P4-T06 same-Task Memory/Skill consumption + P3 Context authority path |
| no_unauthorized_stale_exposure | B03 MVP matrix + Memory FTS authority filter |
| skill_reuse | P4-T04/P4-T06 Skill binding/consumption |
| no_duplicate_effect | Runtime Spine B05/B12 original-key reconcile |
| no_false_completion | Runtime Spine false-completion floor |
| stale_epoch_rejected | B09 fencing + Runtime Spine epoch negatives |
| stable_changed_context_token_reduction | P7-T04/D03 B06/B07 non-claim observation binder (does not create a new benefit claim) |

## Operability rollup bindings

| Observation | Bound evidence |
|---|---|
| six_resource_release_manifest | P7-T01 release_manifest |
| sbom_attestation_digest_bound | P7-T01 D02 |
| lifecycle_backup_restore | P7-T02 personal_backup |
| six_resource_doctor | P7-T03 six_resource_doctor |
| headless_vault_doctor | P7-T03 headless_vault_doctor |
| desktop_or_headless_secretstore_path | P1-T02 / P7-T03 doctor paths |
| pi_sidecar_b09_pins | P5-T05 / ADR-0047 |

## Non-claim report digests

- `suite_digest`: `sha256:b633266e8b952c8a310cea04e316b3bea3f2c23ad3c718bfd515da69528a7a21`
- `trace_digest`: `sha256:4edf1fab99b9bd8aa39c5c7255a4be71cc3782333688405589e7fbe505353df3`
- `report_digest`: `sha256:c7cbc45f78f8e214f69a2d1c42492175ac698504440aa05558528510445806d7`
- `claim_scope`: `non-claim`

## Harness validation

- Local `pnpm -C tools test` includes `gmvp-linux-gate` 2/2
- Required CI for the composition binder commit: pending after push

## Non-claims

Evaluator does not set Gate state. No Profile, Windows B01-W, or B06/B07/B10/B11
benefit claim.
