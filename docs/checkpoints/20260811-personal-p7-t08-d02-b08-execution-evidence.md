# P7-T08/D02 B08 execution evidence (ADR-0048)

- Task: `P7-T08`
- Slice: `P7-T08/D02`
- Campaign: `B08-memory-skill-consumption/1`
- Policy: ADR-0048 MVP fixed denominator
- Exact revision: `65a736cd00a4b3da39a96be799ed6b60e434eeac`
- Environment: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`) via
  `git clone --depth 20 --branch personal/P7-T08-gmvp-linux`
  into `/home/wuz/agent-kernel-worktrees/p7-t08-gmvp-linux`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/194
- Date: 2026-08-11

## Matrix results (all pass)

| Observation | Fixed evidence | Result |
|---|---|---|
| memory_admission_current_source | `admitted_current_source_creates_an_immutable_memory_object` | pass |
| memory_stale_source_rejects | `stale_or_unknown_source_binding_rejects_without_memory_object` | pass |
| memory_reject_decision_no_object | `reject_decision_cannot_create_memory_object` | pass |
| memory_search_authority_filter | `search_filters_authoritative_scope_purpose_and_retention_before_fts_ranking` | pass |
| memory_forget_no_resurrection | `forget_appends_a_tombstone_and_prevents_fts_resurrection` | pass |
| memory_expiry_boundary | `expiry_requires_reached_retention_boundary_and_invalidates_fts` | pass |
| memory_version_cas_supersede | `versioned_update_uses_durable_cas_and_supersedes_the_previous_search_row` | pass |
| skill_workspace_binding | `compatible_local_revision_binds_only_inside_its_workspace` | pass |
| skill_unsafe_revoke_fail_closed | `unsafe_import_and_incompatible_or_revoked_bindings_fail_closed` | pass |
| skill_supersede_exact_pins | `revision_supersede_preserves_exact_pins_and_rejects_competing_lineage` | pass |
| task_consumption_channel_isolation | `task_projection_requires_task_reference_and_management_cannot_cross_task_boundary` | pass |
| Non-claim harness | `tools/test/b08-memory-skill-gate.test.mjs` | pass 2/2 |

Focused Linux commands at `65a736c`:

- `cargo test -p cognitive-store --test p4_t01_memory_store --test p4_t02_memory_search --test p4_t04_skill_store --locked` → 14/14
- `cargo test -p kernel-server --test p4_t05_resource_api --locked` → 1/1
- `cargo clippy -p cognitive-store -p kernel-server --all-targets -- -D warnings` → pass
- `node --test tools/test/b08-memory-skill-gate.test.mjs` → 2/2

## Non-claim report digests

- `suite_digest`: `sha256:51ab75903db8dde2a27c8ae99db3c0c069a197ec08fadd2e023308028db41477`
- `trace_digest`: `sha256:670c6fbf584c8ff4a1cfc31d335e5a623ab1377d6967ed828f672d993a20a7c6`
- `report_digest`: `sha256:976441665b4583fa72a7f4bc93cbac4ed669268a7fd3a518547f365deb424927`
- `claim_scope`: `non-claim` (evaluator cannot set Gate state)

## Required CI

- Required Ubuntu/Windows CI run `31479512940` SUCCESS for PR #194 head
  `65a736cd00a4b3da39a96be799ed6b60e434eeac`.

## Disposition

Recorded under §2.3 / ADR-0048 in
`docs/checkpoints/20260811-personal-p7-t08-d02-b08-disposition.md`:
B08 MVP `pass`. No GMVP-LINUX/release/Profile claim from this slice alone.

## Non-claims

No embedding/vector/graph retrieval, public Memory/Skill schema authority,
live UCR statistical benefit, GMVP-LINUX, release, or Profile claim.
