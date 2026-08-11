# ADR-0049: Personal GMVP-LINUX MVP Composition Validation Policy

- Status: Accepted (owner session standing continuous-delivery direction
  2026-08-11: ADR-0040-class fixed composition for P7-T08/D03–D04
  GMVP-LINUX MVP)
- Date: 2026-08-11
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P7-T08, GMVP-LINUX, ADR-0040, ADR-0046, ADR-0047, ADR-0048,
  B01–B05/B08/B09/B12, UCR-01, P7-T01..T03
- Supersedes: a live multi-Gate statistical re-campaign for **P7-T08 MVP
  GMVP-LINUX disposition only**

## Context

Promotion composition for Personal Linux 1.0 is exactly
`B01+B02+B03+B04+B05+B08+B09+B12`. Each Gate already has an ADR-0040-class MVP
disposition path (or successor B01 policy). P7-T01..T03 deliver release
manifest, backup/restore, and doctor operability. UCR-01 fixed-scenario
assertions are plan-owned acceptance checks for P7-T08 and must be bound as
composition observations without inventing a second release Gate.

A live re-run of every Gate statistical campaign would not strengthen the
composition signal beyond the already recorded MVP dispositions and
authority-path evidence.

## Decision

For the P7-T08 MVP disposition of `GMVP-LINUX`, the fixed validation
denominator is the complete composition matrix below, executed as a
non-claim binder at one exact reviewed revision, plus required
Ubuntu/Windows CI:

| Class | Required observations |
|---|---|
| Gate composition | `b01_mvp_pass`, `b02_mvp_pass`, `b03_mvp_pass`, `b04_mvp_pass`, `b05_mvp_pass`, `b08_mvp_pass`, `b09_mvp_pass`, `b12_mvp_pass` |
| UCR-01 assertions | `required_recall`, `no_unauthorized_stale_exposure`, `skill_reuse`, `no_duplicate_effect`, `no_false_completion`, `stale_epoch_rejected`, `stable_changed_context_token_reduction` |
| Operability rollup | `six_resource_release_manifest`, `sbom_attestation_digest_bound`, `lifecycle_backup_restore`, `six_resource_doctor`, `headless_vault_doctor`, `desktop_or_headless_secretstore_path`, `pi_sidecar_b09_pins` |
| Non-claim suite harness | `tools` Node tests for `gmvp-linux-gate` |

UCR-01 assertion rows are satisfied for MVP by binding the existing
authority-path / fixed-runner evidence already delivered under P2-T08,
P3-T05/T06, P4-T06, and P7-T04 non-claim observations; they do not require a
new live Provider statistical UCR campaign for MVP disposition.

MVP pass conditions for GMVP-LINUX are all of the following:

1. every composition observation is explicitly true in a non-claim report;
2. B08 MVP disposition under ADR-0048 is recorded `pass` (and the other Gate
   MVP dispositions remain `pass`);
3. required Ubuntu and Windows CI pass for the review revision;
4. the non-claim GMVP suite report is generated (`claim_scope: non-claim`;
   evaluator cannot set Gate state); and
5. an affirmative or rejecting disposition for GMVP-LINUX is recorded against
   that bounded evidence. Under standing continuous-delivery authorization,
   the agent may record that disposition for this ADR-0049-class MVP path
   when items 1–4 are complete; the product owner may override. Unresolved
   thresholds, live statistical re-campaigns, Profile promotion, Windows
   B01-W parity, and other Operating Model §2.4 boundaries still require
   explicit owner confirmation.

## Consequences

- P7-T08/D03–D04 can close after the composition binder, required CI,
  non-claim report, recorded GMVP-LINUX disposition, and normal PR/lease
  closure.
- The GMVP evaluator remains non-authoritative.
- B06/B07/B10/B11 remain non-blocking and must not appear as required
  composition observations.
- This decision does not claim Profile conformance or Windows install parity.

## Non-goals and non-claims

This ADR does not claim Profile, Windows B01-W, B06/B07 benefit, B10/B11
enablement, or a second release Gate.
