# F-017 Platform / Channel Sandbox Matrix (M6)

> Status: **open** (exit blocker for v0.1). Linux unit negatives exist; full
> platform CI evidence digests are incomplete. Do **not** merge WSL2 guest
> results into a Windows-native claim.

## Claim discipline

| Platform row | Meaning | Merge rule |
|---|---|---|
| `linux_native` | Reference Linux containment | May claim deny/degrade only with evidence digest |
| `windows_wsl2_linux_guest` | Windows host running Linux guest via WSL2 | Represents **Linux guest** only |
| `windows_native` | Windows-native containment backend | Without native evidence → `unsupported` / `not-tested` |

## Current measured rows (unit / in-process)

Evidence grounding: `cognitive-runtime::SandboxGate` crate tests + CFR
`AGENT-BYPASS-002` behavior mode (Linux native negatives).

| Platform | Channel | Claim | Evidence |
|---|---|---|---|
| linux_native | network (unmediated) | denied_with_evidence | runtime unit + AGENT-BYPASS-002 |
| linux_native | secrets (`host://`) | denied_with_evidence | runtime unit + AGENT-BYPASS-002 |
| linux_native | tool_proxy (unregistered) | denied_with_evidence | runtime unit + AGENT-BYPASS-002 |
| linux_native | remaining channels | not_tested | — |
| windows_wsl2_linux_guest | * | not_tested | no separate guest CI job yet |
| windows_native | * | unsupported | no Windows-native containment backend |

## Closure criteria (from M6-PLAN `M6-F017`)

1. Every claimed deny/degrade row has a reproducible command + digest.
2. Linux native reference negatives cover declared matrix dimensions used in release claims.
3. WSL2 rows explicitly labeled Linux guest.
4. Windows native remains `unsupported` until a native backend exists.
5. Cross-platform merge refused (`refuse_cross_platform_merge`).

Until (1)–(5) hold for every release claim, F-017 stays **open** and v0.1
milestone review is **NO-GO** on this gate.
