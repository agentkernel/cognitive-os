# Personal project identity and development-rule refactor handoff

## Record metadata

- record_type: handoff
- project_id: cognitiveos-personal
- task_id: governance/project-identity-rules
- lease_id: `lease/personal/governance/project-identity-rules-20260730`
- status_at_handoff: done
- gate_status_at_handoff: not-applicable
- claim_scope_at_handoff: non-claim
- current_status_source: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` and
  `docs/plan/PROGRESS.md` Current snapshot
- supersedes: `20260730-governance-operating-model-handoff.md`
- superseded_by: none-known-at-write-time

## 1. Completed

1. Declared `cognitiveos-personal` as the repository's only active
   implementation project. CognitiveOS whitepapers, specifications,
   conformance assets, and reusable kernel code are now explicitly the
   architecture/contract foundation rather than a second product backlog.
2. Added human-readable `PROJECT-IDENTITY.md` and machine-readable
   `project-scope.yaml`, then aligned the root README, docs map, formal Personal
   plan, research plan, current progress snapshot, and workspace package name.
3. Reduced `AGENTS.md` to a Personal-first agent entry and added one thin,
   always-applied Cursor adapter rule. The tool-neutral Operating Model remains
   the workflow source of truth.
4. Unified the `in-progress` transition: pure research and uncommitted planning
   do not change status; the first real implementation or test slice does.
5. Added a forward-progress protocol requiring a vertical implementation,
   failure-first repair, verifiable governance correction, or bounded blocker.
6. Made `PARALLEL-LANES.md` the only active ownership ledger with stable lease
   IDs and moved closed work out of the active table. `PROGRESS.md` now only
   references active lease IDs.
7. Standardized handoff metadata and marked the P1-T09 Pi-launch handoff's old
   status as historical, with an explicit superseding handoff.
8. Extended `check-consistency.mjs` to verify project identity, canonical source
   paths, active lease status/reference integrity, and writable-path overlap.

## 2. Verification

- `pnpm --filter @cognitiveos/repo-tools run build` - passed.
- `pnpm --filter @cognitiveos/repo-tools run test` - 4/4 passed.
- `pnpm run check:consistency` - passed: 273 requirements, 55 error codes, 63
  schemas, 85 vectors, links, traceability, project identity, and leases.
- `git diff --check` - passed.
- IDE diagnostics for the changed checker/rule/governance paths - none.
- Failure-injection rehearsal - temporarily changed
  `active_project.id` to `injected-invalid-project`; consistency failed with
  `active_project.id must be cognitiveos-personal`. The canonical ID was then
  restored and the full check passed.

## 3. Classification and non-claims

- Classification: repository-governance structural refactor; CognitiveOS
  normative surface unchanged.
- No registry, schema, transition, public DTO, registered error, generated
  contract, or conformance vector semantics changed.
- P1-T09 remains `in-progress`; B01 and GMVP-LINUX remain `not-run`; Profile
  `implemented` remains `0`.
- This delivery makes no product-Gate, release, Profile, Pi-load, Provider
  conversation, containment, or platform-parity claim.

## 4. Next executable entry

Claim a new non-overlapping `lease/personal/P1-T09/<slice>` for exact Pi
`0.81.1` availability and the smallest real `--extension <absolute-path>` load.
Do not reopen the architecture milestone backlogs or use them as parallel task
sources. B01 remains separate and requires a pre-registered campaign.

## 5. Remote visibility

- Branch: `lane/doc-personal-project-identity`.
- Implementation commit: not created in this handoff snapshot.
- Remote visibility: not pushed in this handoff snapshot.
- Lease: closed after final local verification; future work must claim a new
  lease.
