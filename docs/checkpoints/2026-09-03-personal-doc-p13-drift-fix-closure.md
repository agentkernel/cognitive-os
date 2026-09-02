# DOC-P13-DRIFT-FIX — closure (owner-directed documentation drift alignment)

- Delivery: `DOC-P13-DRIFT-FIX` (PERSONAL-DEVELOPMENT-PLAN Phase 13「配套维护交付」row); change class **corrective**, documentation only; normative surface unchanged
- Lease: `lease/personal/DOC-P13-DRIFT-FIX/build-order-and-pi-package` → closed in the merge-closure commit on `main` (moved to PARALLEL-LANES §3.1)
- Branch: `personal/DOC-P13-DRIFT-FIX`; content head `12e84b7c` (validated), closure head recorded in the PR
- PR: [#309](https://github.com/agentkernel/cognitive-os/pull/309) — Draft → ready → merged (merge commit recorded in PROGRESS)
- Required CI: [33670770754](https://github.com/agentkernel/cognitive-os/actions/runs/33670770754) **SUCCESS** at `12e84b7c` (resolve 3s; verify ubuntu-latest 3m30s; verify windows-latest 10m7s; required-ci 3s)
- Running report: [2026-09-03-personal-doc-p13-drift-fix-report.md](2026-09-03-personal-doc-p13-drift-fix-report.md)

## Acceptance mapping (plan row「关闭门」)

| Gate | Evidence |
|---|---|
| 两处建造顺序边集合逐条相等 | report §2: 27/27 edges equal (24 solid + 3 dashed); 6 edges added to the index; formal plan untouched |
| 四页 handbook 包名 == 代码常量 | `reference/compatibility.md` + `developer/agent-and-pi-lifecycle.md` (en, zh-CN) now state `@earendil-works/pi-coding-agent` = `OFFICIAL_PI_PACKAGE`; authority table in report §1 (code, environment registry §1, plan.md PI-02 agree) |
| autocrlf 说明双语存在 | `developer/development-environments.md` (en, zh-CN): local `core.autocrlf=true` is overridden by `.gitattributes` `* text=auto eol=lf`; no local config change |
| 全部门禁绿 | report U5–U10 (local, `DEV-WIN-GNU-01`) and U13 (required CI) |

Drift-detection negatives from the plan row, checked: only one graph changed — no (both compared, only the index changed); handbook used to rewrite the code constant — no (code is authority); PROGRESS dynamic status copied into the handbook — no.

## Additional surface (flagged for transparency)

To register the delivery's own running report / closure files on a `DOC-<id>` lease, the `check-consistency` DOC-lease allowlist was extended to accept exact `docs/checkpoints/` files (never the directory), with fixture coverage in `tools/test/check.test.mjs`; PARALLEL-LANES §2.3 wording aligned in the same commit. This is the "checker surface needed for lease registration" that §2.3 already permits; it grants no product-code ownership.

## Non-claims

Claim ceiling `hypothesis`. Documentation/static-consistency and ordinary-CI evidence only. No Gate, release, Profile, T15, Windows-support, Agent-benefit or implementation claim. `P0-T09` (machine enforcement of these alignments) is the next, separate formal-task delivery.

## Next unique action

Claim `P0-T09/D01` with a formal task lease `lease/personal/P0-T09/drift-checks` on branch `personal/P0-T09-drift-checks` from fresh `origin/main`.
