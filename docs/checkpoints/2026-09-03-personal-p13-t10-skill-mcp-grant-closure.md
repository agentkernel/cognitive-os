# P13-T10 Skill/MCP security-reviewed acquire + scoped grant — closure

- Task: `P13-T10` / slice `P13-T10/D01`
- Change class: `implementation-only` (reuses v27 InstallFact / v30 grant-expansion; no new migration)
- Lease: `lease/personal/P13-T10/skill-mcp-grant` (closed this delivery → PARALLEL-LANES §3.1)
- Branch: `personal/P13-T10-skill-mcp-grant` (PR [#318](https://github.com/agentkernel/cognitive-os/pull/318))
- Fold HEAD: `abfb9ca2` (parents `83c603b4` + `origin/main@ef9baab2`)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## 1. Acceptance

Install ≠ grant. Chrome is **Request acquire preview** only. **No Activate**. Discovery → structured SecurityReview → exact Owner canvas preview → version-pinned InstallFact → scoped grant → update / compat / rollback. Failure-first: install-is-authorize, unreviewed install, hidden/injection, marketplace/engine-store, chat Approve, ambient grant, silent grant on update/rollback.

Evidence: store 6/6 + HTTP 1/1 + clippy on `DEV-LINUX-NATIVE-01` at `d861d341`; required CI [33747031610](https://github.com/agentkernel/cognitive-os/actions/runs/33747031610) **SUCCESS** at `d861d341`; Dual Track 23/23. Folded HEAD must have required CI green before merge.

## 2. Non-claims

Not T06/T07/T08/T09/T11/T13. No marketplace / engine store / second grant table / Activate. No Gate / release / Profile / B01. `PERS-PR-050` stays `not-run` until T13.

## 3. Unique next

After PR #318 merges: claim **P13-T09** if unclaimed. Do not claim sibling-owned T07/T08.
