# P3-T02 Context Builder closure

- Date: 2026-08-09
- Classification: `implementation-only`
- Task: `P3-T02`
- Slices: `P3-T02/D01`, `P3-T02/D02`
- Branch: `personal/P3-T02-context-builder`
- Lease: `lease/personal/P3-T02/context-builder`
- PR: [#166](https://github.com/agentkernel/cognitive-os/pull/166) (all required CI passed)
- Acceptance checkpoint: `0d8f5628a897aea32ee4cb7929bac1320ccb2a96`
- Upstream: `origin/personal/P3-T02-context-builder`

## Acceptance mapping

The builder derives required System and Task fragments only from the immutable
ContextRequest and TaskContract before it permits sealed ContextView transport
to private Pi. Required fragments that cannot fit the hard budget fail closed;
authorized duplicate bodies are omitted with an explicit loss declaration.

Workspace source discovery remains metadata-only. The scheduler admits only
declared Working, Evidence, or Shell source families, checks their immutable
governed-header creation time against role-specific request freshness before
loading any body, and persists explicit excluded-source loss/rejection facts.
Loaded ContextView entries retain their strong digest-bound source references;
excluded-source loss verification retains the immutable source digest. Current
authorization and revocation revalidation remains immediately before every
authorized body load.

## Validation

- Local formatting: `cargo fmt --all -- --check` -- pass
- Local diff hygiene: `git diff --check` -- pass
- Exact-revision native Linux: `cargo test -p kernel-server stale_workspace_source_is_excluded_before_body_loading_with_explicit_loss` -- pass at `0d8f5628a897aea32ee4cb7929bac1320ccb2a96`
- Exact-revision native Linux: `cargo clippy -p kernel-server --all-targets -- -D warnings` -- pass at `0d8f5628a897aea32ee4cb7929bac1320ccb2a96`
- Required Ubuntu CI -- pass for PR #166 run `31268250790`
- Required Windows CI -- pass for PR #166 run `31268250790`

## Non-claims and next action

This delivery does not pass B03, make a UCR-01 benefit claim, or create a
release or Profile claim. It does not authorize Pi, promote Context to Task
authority, or treat source retrieval as Task completion. The remaining closure
steps are to mark PR #166 ready, merge it, close the lease, and clean the task
branch.
