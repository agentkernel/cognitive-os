# CognitiveOS Personal — Support Matrix (P0-T03)

> **Status:** owner-accepted 2026-07-26 (ADR-0025)
> **MVP-first update:** owner-accepted 2026-07-29 (ADR-0034). Linux is the
> first public MVP train; Windows remains a product target with an independent
> B01-W claim gate.
> **Not:** B01-B12 evidence, `GMVP-LINUX`, Profile claim, or release GO.

## 1. Product platforms (first ship)

| Platform | Arch | Product status | Install surface (planned) | Secret backend direction | Evidence host today |
|---|---|---|---|---|---|
| Linux | x86_64 | **First public MVP platform** | GitHub Release checkable bundle + one canonical systemd **user** service on loopback port 48181 (P1-T08/P7-T08) | FreeDesktop Secret Service / `secret-tool` path (ADR-0018/0020) | CI Ubuntu build/test; Linux-native product evidence not-run |
| Windows | x86_64 | **Product target; install parity independent** | Daemon/CLI product path; native installer/service and B01-W in P7-T07 | Platform credential store direction (same fail-closed boundary) | CI Windows/MSVC build/test; B01-W not-run |

## 2. Engineering and non-product hosts

| Host | Status | Notes |
|---|---|---|
| CI Ubuntu | Supported Linux build/test matrix evidence | Required green for code merges; not Linux-native systemd, B01, `GMVP-LINUX`, or release evidence |
| CI Windows/MSVC | Supported Windows build/test matrix evidence | Required green for code merges; not B01-W or install-parity evidence |
| `personal-linux-native-01` (`wuz@192.168.1.2`) | Designated local Linux-native experimental host | Use for explicitly authorized `experimental-local-only` / `tested-local` Personal and Pi validation; local-only evidence here is not CI evidence and does not by itself create a product, Gate, Profile, containment, or release claim |
| Local Windows GNU / MinGW | **Non-supported** | P0-T01 linker exit 121; must not block CI-green work |
| WSL2 as product runtime | **Not first-ship product** | Existing Pi admission may refuse WSL2; do not market as Personal product host without a later ADR |
| Linux aarch64 / macOS | Deferred | Out of first-ship matrix |

## 3. Distribution

| Channel | Decision |
|---|---|
| GitHub Releases | **Yes** — public checkable artifacts |
| First public artifact | Linux x86_64 single-service bundle (`P1-T08` foundation; `P7-T01..T03` operability; `P7-T08 / GMVP-LINUX` release gate) |
| Vendor Pi in bundle | **No** — user-local pin (P0-T06) |
| Vendor Node in bundle | **No** |
| crates.io publish | **No** (until later P7 decision) |
| npm public publish | **No** (until later P7 decision) |

## 4. Gate mapping (honest)

| Gate | Relation to this matrix |
|---|---|
| G0 | Phase 0 task-level baseline is complete; this matrix alone neither proves nor reopens it. |
| B01 | Clean Linux VM install-to-first-dialogue campaign; WSL, fake systemd and ordinary CI do not substitute. |
| GMVP-LINUX / P7-T08 | Scoped public Linux MVP after B01, governed Task MVP and release-operability evidence; not a Profile Gate. |
| B01-W / P7-T07 | Independent Windows install-parity Gate; it does not block Linux MVP. |
| RC / P7-T06 | Full declared-scope support claims only with executed evidence; Multi-Agent may remain disabled after a valid no-go. |

## 5. References

- [ADR-0025](../adr/0025-personal-license-platform-distribution.md)
- [ADR-0034](../adr/0034-personal-mvp-first-single-service-release-train.md)
- [THIRD-PARTY-NOTICES.md](../legal/THIRD-PARTY-NOTICES.md)
- [PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)
