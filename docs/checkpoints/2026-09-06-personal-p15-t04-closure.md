# P15-T04 Knowledge vs v9 — closure

- Task: `P15-T04` **done** / slices `P15-T04/D01` **done** + `P15-T04/D02` **done**
- Change class: `implementation-only` (Owner-facing Knowledge files / why / import vs frozen v9). P14-T08 Vault authority stays. Files are not Project authority. No Obsidian. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Lease: `lease/personal/P15-T04/knowledge-v9` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P15-T04-knowledge-v9` → Draft PR [#339](https://github.com/agentkernel/cognitive-os/pull/339) (ready/merge in this close)
- Dual Track / guest pin: `991408f8da99f2d15d8ffdda25be9471b3adb8e0` walk
- Report HEAD required CI [34663663188](https://github.com/agentkernel/cognitive-os/actions/runs/34663663188) **SUCCESS** at `1cfa5f69` (ubuntu 4m15s, windows 15m2s, required-ci 4s). This closure commit re-runs required CI before ready/merge.
- Guest `/ui/` pin: `991408f8` / SPA `index-3s2lrA_c.js` + `index-C9SPnPVx.css`
- Running report: [P15-T04 report](2026-09-06-personal-p15-t04-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Knowledge Owner copy toward v9" only). Not Gate / release / Profile / B01. Windows native daemon chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. P14-T08 remains **done**. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not implement T05–T06 from this close.

## 1. Acceptance mapping (formal plan P15-T04 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Knowledge vs v9 files / Why this fragment / import Owner copy | `KnowledgePage` title `当前项目资料` / locked `知识已锁定`; tabs 项目资料 / 导入 / 为什么用这段 / 记忆; lede 不必另装笔记应用 | Dual Track N2 HTTP-paste / English `Knowledge` / `Import` | Dual Track N2 **fail** then **pass**; guest J5 unlocked title `当前项目资料`; tabs match; Import `input[name=vault-files]`; no `textarea[name=vault-body]` |
| files ≠ Project authority; secret ingest refused | Import refuses secret-shaped payload; no `vault.import` on that path; honesty 资料不是项目权威 | Dual Track N3 file-as-authority / secret ingest | N3 **fail** then **pass**; guest Import has file input only; no HTTP-paste wall |
| Unauthenticated fail-closed; no fake Activate; no Twitter/X P0 | SessionGate; 0 Activate; twitter=false | Dual Track N1 | N1 **pass**; guest J0 Session denied / Open Session; no `opc-knowledge` |
| `JOURNEY-BROWSER-SYNC-01` J5 + regression J0/J2/J10 | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite origin; `:48181` replace | J5 **pass**; J0/J2/J3/J10 **pass**. Left `:48181` untouched (PID 166715) |

Formal-plan 关闭门: files / why / import 视觉对 v9 — **true**; Vault 权威不变 — **true**.

Drift negatives: HTTP paste 冒充 v9 — Dual Track N2 + guest no `vault-body` textarea. file-as-authority / secret ingest — Dual Track N3. Obsidian — Dual Track + guest text has no Obsidian. Vite as product origin — walk used daemon `/ui/` only. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T08 functional card not reopened.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | Focused Knowledge + opcIa **46/46** at `991408f8`. Earlier D01 full suite **549/549** (T03 later raised web suite to **553/553**). Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T04. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | Closing HEAD `1cfa5f69` required CI [34663663188](https://github.com/agentkernel/cognitive-os/actions/runs/34663663188) **SUCCESS**. Ancestor fold `991408f8` CI [34663102994](https://github.com/agentkernel/cognitive-os/actions/runs/34663102994) superseded by this HEAD. |
| `DEV-LINUX-NATIVE-01` | Exact-revision UI dist `index-3s2lrA_c.js` at `991408f8`. No kernel-server rebuild (UI-only). |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID **2740931** pin `991408f8`. Left `:48181` untouched (PID **166715**). `cognitive dsh web` `:3080` **not-run**. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used host Chrome 140 against forwarded guest `/ui/`) |

## 3. Non-claims

Not T05 Settings vs v9. Not T06 layout tokens. Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T08 stays **done**. SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334) not merged. Obsidian not bundled.

## 4. Unique next

Ready/merge PR [#339](https://github.com/agentkernel/cognitive-os/pull/339) after this closure is on the same required-CI HEAD or a follow-up docs HEAD that re-runs required CI. Unique next after T04 close: **claim / continue `P15-T05`** (Settings / Model Connections vs v9). Worktree `D:\agent-kernel-wt-P15-T05` already has uncommitted Settings files — adopt, do not overwrite (A8). Do not claim T06 from this close. Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.
