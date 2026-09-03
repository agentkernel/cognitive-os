# P0-T09 — closure (machine drift checks; Slice P0-T09/D01)

- Task: `P0-T09` 计划/规则漂移的机械校验 (Phase 0) — status `done` with this delivery; Slice `P0-T09/D01` `done`
- Change class: **implementation-only** (tool surface `tools/**`, handbook `_meta` + mapped pages); normative surface unchanged; formal plan build order not edited
- Lease: `lease/personal/P0-T09/drift-checks` → closed in the merge-closure commit on `main` (PARALLEL-LANES §3.1)
- Branch: `personal/P0-T09-drift-checks`; content head `64732c61`, ledger head `8f0d83b1`, closure head recorded in the PR
- PR: [#312](https://github.com/agentkernel/cognitive-os/pull/312) — Draft → ready → merged (merge commit recorded in PROGRESS)
- Required CI: [33678438650](https://github.com/agentkernel/cognitive-os/actions/runs/33678438650) **SUCCESS** at `8f0d83b1` (resolve 4s; verify ubuntu-latest 3m49s; verify windows-latest 18m37s; required-ci 3s)
- Running report: [2026-09-03-personal-p0-t09-drift-checks-report.md](2026-09-03-personal-p0-t09-drift-checks-report.md)

## Acceptance mapping (Phase 0 table row + `P0-T09/D01` slice row + plan.md 关闭门)

| Acceptance item | Implementation | Negative fixture / evidence |
|---|---|---|
| `check-consistency` 与 `check-agent-rules` 的链接/路径检查基于 `git ls-files`，已提交文档链接未跟踪文件在本机即红 | `tools/src/lib.mjs` `loadTrackedPaths`/`isTrackedPath`; `check-consistency.mjs` (links, `owner_spec`, matrix, trace, project-scope; fail-closed `TRACKED_PATHS_UNAVAILABLE`); `check-agent-rules.mjs` (tracked-only; local-only assets filesystem; labelled fallback) | report F1–F4, U7 drill (`(exists locally but is not tracked by Git)`), U10 clean-tree drill |
| 正式计划 Phase 13 mermaid 边集合与 dev-prep index「Phase 13 build order」边集合被机械比对 | `check-consistency.mjs` §7 (`extractSectionMermaid`, `parseBuildOrderEdges`; solid/dashed kind; ids normalized) | report F1–F2 (missing/extra/kind/graph-missing tests), U7 drill (`BUILD_ORDER_EDGE_MISSING` / `_EXTRA`) |
| `installer.rs` `OFFICIAL_PI_PACKAGE` 变动经 source-map 触发 `ref.compatibility` / `dev.agent-pi-lifecycle` 复查（symbols 钉住） | `source-map.json` rule `pi-official-package-pin`; page frontmatter `sources` pin `OFFICIAL_PI_PACKAGE`/`OFFICIAL_PI_VERSION` (4 files); new HB016 in `handbook-lib.mjs`; `ref.compatibility` fingerprint now covers `installer.rs` | report F5–F6 (HB016 one-locale drop, HB007 rename), `docs-sync-gate.test.mjs` live-route test (fails closed without a handbook update) |
| 每项有 focused negative fixture 且注入演练输出附于 PR | 9 new tests across the four test files; §5 drill pasted in the PR body and report §4 | repo-tools 124/124 locally, in the clean worktree, and in CI |
| Lane-CFR 工具面 + handbook `_meta`; mapped handbook pages bilingual + fingerprints | `dev.conformance-testing`, `ai.validation-commands`, `meta.sync-policy`, `ai.docs-impact`, `ref.compatibility`, `dev.agent-pi-lifecycle` | `check-handbook` OK (58×2), `generate-handbook --check` OK |
| 漂移检测负例：校验只在 CI 生效而本机放行；文件系统存在性冒充 tracked；边集合写死不解析；为通过校验反向改正式计划 | none of these occurred: local == CI verdict (U10); `git ls-files`, not `existsSync`, decides; edges are parsed from both mermaid blocks; the formal plan graph is unchanged | report §1 observation: no tracked living-doc link currently targets an untracked file, so the check is preventive today and proven by fixture |

## Non-claims

Claim ceiling `hypothesis`. Node tooling / handbook / ordinary-CI evidence only. No Gate, release, Profile, T15, Windows-support or Agent-benefit claim. Rust build/test/Clippy `not-run` on `DEV-WIN-GNU-01` (no Rust surface changed; CI ran them). The process incident (unrelated stash applied and reverted; no stash dropped) is recorded in report §2.

## Next unique action

Claim `P0-T01/D02` (local Rust toolchain repair) with lease `lease/personal/P0-T01/toolchain-repair` on branch `personal/P0-T01-D02-toolchain` from fresh `origin/main`; owner has chosen option (a) local-only override.
