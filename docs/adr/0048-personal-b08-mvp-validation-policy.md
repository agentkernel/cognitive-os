# ADR-0048: Personal B08 Memory/Skill MVP Validation Policy

- Status: Accepted (owner session standing continuous-delivery direction
  2026-08-11: ADR-0040/ADR-0046/ADR-0047-class fixed denominator for
  P7-T08/D01–D02 B08 MVP)
- Date: 2026-08-11
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P7-T08, B08, P4-T01..T06, ADR-0040, ADR-0046, ADR-0047, GMVP-LINUX
- Supersedes: a live Provider/UCR statistical campaign for **P7-T08 MVP B08
  disposition only**

## Context

P4-T01..T06 already deliver daemon-private Memory admission, FTS authority
filtering, forget/expiry/supersede lifecycle, Skill package/revision/binding
revoke and supersede, and same-Task consumption channel isolation with
failure-first negatives. P4-T06 closed as implementation evidence and left
B08 as an independent Gate campaign. A live UCR statistical campaign would
add Provider/credential ceremony without strengthening the MVP
authority-path signal already covered by those focused tests.

Owner standing direction for Gate/campaign slices: prefer ADR-0040-class fixed
denominators (authority-path / fixture / non-claim report) unless formal
acceptance explicitly forbids the MVP path. P7-T08 acceptance requires B08 as
part of the GMVP-LINUX promotion composition and does not require a live
Provider statistical suite for the B08 MVP disposition itself.

## Decision

For the P7-T08 MVP disposition of B08, the fixed validation denominator is the
complete authority-path matrix below, executed at one exact reviewed revision
on `DEV-LINUX-NATIVE-01`, plus required Ubuntu/Windows CI and a non-claim
report:

| Required observation | Fixed evidence |
|---|---|
| `memory_admission_current_source` | `cognitive-store` test `admitted_current_source_creates_an_immutable_memory_object` |
| `memory_stale_source_rejects` | `cognitive-store` test `stale_or_unknown_source_binding_rejects_without_memory_object` |
| `memory_reject_decision_no_object` | `cognitive-store` test `reject_decision_cannot_create_memory_object` |
| `memory_search_authority_filter` | `cognitive-store` test `search_filters_authoritative_scope_purpose_and_retention_before_fts_ranking` |
| `memory_forget_no_resurrection` | `cognitive-store` test `forget_appends_a_tombstone_and_prevents_fts_resurrection` |
| `memory_expiry_boundary` | `cognitive-store` test `expiry_requires_reached_retention_boundary_and_invalidates_fts` |
| `memory_version_cas_supersede` | `cognitive-store` test `versioned_update_uses_durable_cas_and_supersedes_the_previous_search_row` |
| `skill_workspace_binding` | `cognitive-store` test `compatible_local_revision_binds_only_inside_its_workspace` |
| `skill_unsafe_revoke_fail_closed` | `cognitive-store` test `unsafe_import_and_incompatible_or_revoked_bindings_fail_closed` |
| `skill_supersede_exact_pins` | `cognitive-store` test `revision_supersede_preserves_exact_pins_and_rejects_competing_lineage` |
| `task_consumption_channel_isolation` | `kernel-server` test `task_projection_requires_task_reference_and_management_cannot_cross_task_boundary` |
| Non-claim suite harness | `tools` Node tests for `b08-memory-skill-gate` (incomplete observation and authority-shaped claim negatives) |

MVP pass conditions for B08 are all of the following:

1. every row in the matrix passes at one exact reviewed revision;
2. focused Rust checks run on qualified native Linux and pass Clippy with
   warnings denied for the exercised packages;
3. required Ubuntu and Windows CI pass for the review revision;
4. a non-claim B08 suite report is generated (`claim_scope: non-claim`;
   evaluator cannot set Gate state); and
5. an affirmative or rejecting disposition for B08 is recorded against that
   bounded evidence. Under standing continuous-delivery authorization, the
   agent may record that disposition for this ADR-0048-class MVP path (and
   equivalent ADR-0040/0046/0047-class fixed-denominator Gate MVPs) when items
   1–4 are complete; the product owner may override. Unresolved Gate
   thresholds, live statistical campaigns, release/Profile promotion, and
   other Operating Model §2.4 boundaries still require explicit owner
   confirmation.

Live Provider/UCR statistical campaigns remain available for later promotion
work when additional signal is needed. They are not a P7-T08 MVP B08
completion mutex. Full GMVP-LINUX composition (UCR-01 fixed-scenario
assertions, six-resource operability rollup) remains later P7-T08 slices.

## Consequences

- P7-T08/D01–D02 can close B08 after the fixed matrix, native Linux/Clippy,
  required CI, non-claim report, recorded B08 disposition, and docs sync.
- The B08 evaluator remains non-authoritative: reports cannot mutate Gate
  state; the documented product decision owns Gate status.
- Provenance/freshness/conflict/forget, Skill package/revision/binding, and
  same-Task consumption channel isolation stay mandatory observations inside
  the fixed matrix.
- This decision does not transfer to GMVP-LINUX, release, or Profile by itself.

## Non-goals and non-claims

This ADR does not claim embedding/vector/graph retrieval, public Memory/Skill
schema authority, live UCR statistical benefit, GMVP-LINUX, release, or
Profile.
