# DOC-LOCAL-RUNTIME-HOST — running report (local runtime host designation)

- Activity: owner-directed documentation delivery `DOC-LOCAL-RUNTIME-HOST`
- Lease: `lease/personal/DOC-LOCAL-RUNTIME-HOST/plan-env`
- Branch: `personal/DOC-LOCAL-RUNTIME-HOST` (worktree `D:\agent-kernel`, base `origin/main@88c2948e`)
- Environment for every local unit: `DEV-WIN-GNU-01` (Windows PowerShell 5.1; Node tooling only)
- Claim ceiling: `hypothesis`. Documentation / environment-registry evidence only — no Gate,
  release, Profile, T15, B01-W, or Windows-support claim. Designation is not qualification.
  `not-run` is never pass.
- Reporting rule: `TEST-REPORT-INCREMENTAL-01` — each unit appended on completion; append-only.

## 1. Owner instruction (2026-09-05)

Two parts, executed in this delivery:

1. Delete remaining Pchat Sand Cursor-file backups (not Git).
2. Change the formal plan and related documents so project runtime testing
   happens on this local host; do not require Windows 11 as a provision gate.

This DOC does **not** implement `P13-T13`. It does **not** mark native E2E cells
pass. It does **not** invent a new environment ID.

## 2. Pchat backup deletion (host hygiene; not in Git)

| Path | Action | Result |
|---|---|---|
| `C:\Users\wuron\AppData\Local\SandClientMode\sand-client-cli\backups` | `Remove-Item -Recurse -Force` | removed |
| `C:\Users\wuron\AppData\Local\SandClientModeStream\sand-client-cli\backups` | `Remove-Item -Recurse -Force` | removed |

Earlier same-day hygiene had left four rollback snapshots (~354 MB). Owner
instruction deleted the rest. `proxy.json` was not copied. Manifest inventory
remains at `D:\archive\dev-host-hygiene-2026-09-05\` (outside the repository).
After deletion `C:` free space ≈ 14.5 GB.

## 3. Designation (plan / registry)

| Item | After this DOC |
|---|---|
| Environment ID | `DEV-WINDOWS-NATIVE-OPC-01` (unchanged; no new ID) |
| Physical host | Same machine as `DEV-WIN-GNU-01` (`D:\agent-kernel`) |
| Recorded OS | Windows 10 Pro 10.0.19045, x86_64 (fact, **not** a gate) |
| Status | **designated**; not qualified |
| `P13-T13` | `not-started` / unclaimed; `P13-T13/D01` **ready** (was `blocked` on Win11) |
| What T13 still owns | unsigned development install + hung native E2E backfill + pin write-back |
| Cargo / Dual Track TS on this host | development evidence only |
| B01-W / signing / release / Profile | unchanged |

Supersedes the 2026-09-04 exclusion of Windows-host test cells and the
disk-full “consult owner before local-disk-heavy testing” pause for this
designation. Disk facts in `PERSONAL-TEST-ENVIRONMENTS.md` §3 were refreshed
(C: ~14 GB / D: ~25 GB after hygiene; `TEMP`/`TMP` → `D:\tmp\rust-link`).

## 4. Validation units

| Unit | Command / check | Result | Environment | Notes |
|---|---|---|---|---|
| U1 backup deletion | PowerShell `Remove-Item` on both `backups` trees | pass | host FS | not a product test |
| U2 docs-sync (staged) | `node tools/src/docs-sync-gate.mjs --staged` | pass | `DEV-WIN-GNU-01` | hook on `9a000fc0` |
| U3 consistency | `pnpm run check:consistency` | pass | `DEV-WIN-GNU-01` | 275 requirements; Phase 13 edge set verified |
| U4 handbook | `node tools/src/check-handbook.mjs` | pass | `DEV-WIN-GNU-01` | 58×2 locales |
| U5 generator `--check` | `node tools/src/generate-handbook.mjs --check` | pass | `DEV-WIN-GNU-01` | 18 pages byte-identical |
| U6 rules | `pnpm run check:rules` | pass | `DEV-WIN-GNU-01` | 0 failures |
| U7 diff --check | `git diff --check` | pass | `DEV-WIN-GNU-01` | content + closure HEADs |
| U8 required CI | GitHub Actions run [33955909125](https://github.com/agentkernel/cognitive-os/actions/runs/33955909125) | pass | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | content HEAD `9a000fc0`: resolve 3s SUCCESS; ubuntu 4m39s SUCCESS; windows 16m20s SUCCESS; `required-ci` 4s SUCCESS |
| U9 Rust workspace | cargo build/test/clippy | not-run | — | documentation-only; no product code |
| U10 native E2E | install / tray / sleep / SecretStore | not-run | `DEV-WINDOWS-NATIVE-OPC-01` | owned by `P13-T13` |

## 5. Non-claims

- Not `P13-T13` done.
- Not Gate / release / Profile / B01-W / T15.
- Not a product-support claim that Windows 10 is the supported matrix.
- Not cargo-as-native-E2E.
- Historical closed-task reports that said “not provisioned until P13-T13”
  remain historical facts.

## 6. Unique next

This DOC is ready to merge (PR [#323](https://github.com/agentkernel/cognitive-os/pull/323); required CI [33955909125](https://github.com/agentkernel/cognitive-os/actions/runs/33955909125) **SUCCESS** at `9a000fc0`). After merge: claim `P13-T13`. Do not claim `P11-T15`.
