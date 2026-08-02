# 20260802 Personal 1.0 Documentation Restructure Handoff

## Record metadata

- record_type: handoff
- project_id: cognitiveos-personal
- task_id: governance/personal-1-0-docs
- lease_id: lease/personal/governance/personal-1-0-docs
- status_at_handoff: done
- development_track_at_handoff: not-applicable
- implementation_evidence_at_handoff: tested-local
- gate_status_at_handoff: not-applicable
- claim_scope_at_handoff: non-claim
- task_definition_source: docs/plan/PERSONAL-DEVELOPMENT-PLAN.md
- current_status_source: docs/plan/PROGRESS.md Current snapshot
- blocked_paths: none
- blocked_task_ids: none
- blocked_gate_ids: none
- blocker_owner: none
- next_executable_action: review the uncommitted branch diff, create a commit/PR only when authorized, then select the next non-overlapping Personal task from the formal plan
- supersedes: none
- superseded_by: none-known-at-write-time

## 1. 本次会话完成

- Declared the atomic delivery as `product-semantic` plus `structural` and
  `corrective` documentation/tooling synchronization. The CognitiveOS public
  registry/schema/transition/vector surface is unchanged.
- Added ADR-0035 to separate the Pi-hosted Agent Shell from managed Pi package,
  installation, instance, execution, process, conversation and Task identities.
- Added ADR-0036 to define `GMVP-LINUX` as Personal `1.0.0`, qualify Pi as the
  only Linux 1.0 Agent, and select exact official-npm acquisition without
  bundling Pi or Node.
- Added canonical Personal product and architecture directories covering the
  cognitive resource model, user journeys, Linux 1.0 scope, layered system,
  Agent lifecycle, authority, data and recovery.
- Reworked the formal plan, PERS-PR trace, support matrix, root task cards and
  Pi integration map around typed dependencies, two independent Pi tracks and
  separate B09 managed-Pi/B10 Tool-MCP evidence.
- Added `PERSONAL-TEST-ENVIRONMENTS.md` with known Windows, Linux, WSL, CI,
  fixture, contract-runner and B01 environment pins and claim limits.
- Corrected the B01 campaign-level status to `running`: attempt 1 remains valid,
  while the minimum 20-attempt denominator, success-rate calculation, zero
  critical-failure closure, aggregate statistics and final verifier remain open.
- Tightened development governance, lease coordination, handoff fields,
  project-scope ownership and the docs-sync contract. Legacy prompts are now
  non-executable references rather than Personal task sources.
- Corrected stale Pi Extension and Agent Shell package documentation without
  changing their runtime behavior.
- Extended `check-consistency.mjs` to reject duplicate Personal task rows,
  summary drift, parallel trace snapshots, missing trace/task/Gate/design
  sources, premature B01 pass, executable legacy prompts, invalid lease
  metadata, broad protected-tree leases and lease-ledger self-ownership.
- Added override-based failure injection that exercises those governance
  failures without corrupting the working tree.

Related product planning entries include PERS-PR-005, PERS-PR-013,
PERS-PR-014, PERS-PR-021 and PERS-PR-024 through PERS-PR-027, plus P2-T02,
P5-T01, P5-T02, P5-T05 and P7-T08. No new REQ, F or IMP identifier was created.

## 2. 未完成 / 进行中

- No Personal runtime implementation was added. P2-T02, managed-Pi
  P5-T01/P5-T02, B09 and P7 Linux 1.0 production-operability work remain at
  the status recorded in the formal plan and Current snapshot.
- B01 remains `running`, not `pass`. Before attempt 2, record a superseding
  campaign addendum using the formal minimum; do not edit, delete or rerun
  attempt 1 or rewrite its historical record.
- This branch is intentionally uncommitted and not visible remotely because no
  commit, push or PR was requested in this session.

## 3. 测试与证据状态

- `pnpm run check:consistency`: **pass**, local Windows; all registry,
  schema/vector, link, traceability, Personal plan/Gate, design-source, prompt
  and lease checks passed.
- `pnpm --filter "@cognitiveos/repo-tools" run build`: **pass**, local Windows.
- `pnpm --filter "@cognitiveos/repo-tools" run test`: **pass**, 5/5 including
  the Personal governance failure-injection rehearsal.
- `pnpm --filter "@cognitiveos/agent-shell" run build`: **pass**, local Windows.
- `pnpm --filter "@cognitiveos/agent-shell" run test`: **pass**, 13/13.
- `pnpm --filter "@cognitiveos/pi-cognitiveos" run build`: **pass**, local Windows.
- `pnpm --filter "@cognitiveos/pi-cognitiveos" run test`: **pass**, local Windows;
  all focused Extension/provider/safety tests passed.
- `git diff --check`: **pass** after removing two Markdown trailing-space
  violations found by the first run.
- Rust build/test/Clippy: **not-run**; this delivery changes no Rust source or
  CognitiveOS public machine/behavior contract.
- Remote CI: **not-run**; no commit or PR exists.
- Conformance vectors/Profile runner: **not-run**; no vector or Profile claim
  changed. Vector pass counts and Profile status are unchanged.
- Evidence: command output only, non-claim. No release or Gate artifact was
  produced and no immutable implementation digest applies.

## 4. 未决风险与漂移

- The old B01 preregistration and attempt ledger preserve execution-time
  wording that cannot override the formal 20-attempt threshold or Current
  snapshot. A superseding addendum is required before more attempts.
- Official Pi acquisition, production-signed acquisition locks and managed Pi
  lifecycle are design/plan only until P5-T01/P5-T02 and B09 execute.
- Linux 1.0 remains `not-run`; product documents use target/planned language
  until `GMVP-LINUX` passes.
- No normative drift was introduced, so findings-ledger was not changed.

## 5. 下一步入口

- 正式任务: select only from `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`; likely
  executable slices include the remaining P1-T09 campaign work, P2-T02 Shell
  composition or a non-overlapping P2-T03 continuation after checking current
  facts and leases.
- matching handoff 规则: `project_id=cognitiveos-personal` +
  `task_id=governance/personal-1-0-docs` +
  `lease_id=lease/personal/governance/personal-1-0-docs` + `20260802`.
- 工作分支: `lane/doc-personal-1-0-restructure`.
- 第一个动作: run `git status --short --branch`, review this branch's diff,
  and obtain explicit authorization before committing or pushing. Any follow-up
  edit must first claim a new exact-path lease.

## 6. 快照

- PROGRESS 已更新: 是；B01 is `running`, Linux 1.0 is `not-run`, and the
  documentation lease is removed at closure.
- active lease: closed in the same documentation delivery.
- 本次提交列表: none.
- immutable implementation commit: not-applicable.
- remote visibility: pending; uncommitted local branch only.
