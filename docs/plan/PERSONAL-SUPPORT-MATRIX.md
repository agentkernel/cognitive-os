# CognitiveOS Personal — Support Matrix (P0-T03)

> **Status:** owner-accepted 2026-07-26 (ADR-0025)
> **Not:** G0 pass, B01-B12 evidence, Profile claim, or release GO.

## 1. Product platforms (first ship)

| Platform | Arch | Product status | Install surface (planned) | Secret backend direction | Evidence host today |
|---|---|---|---|---|---|
| Linux | x86_64 | **First product platform** | GitHub Release checkable bundle + systemd **user** service (P1-T08) | FreeDesktop Secret Service / `secret-tool` path (ADR-0018/0020) | CI Ubuntu |
| Windows | x86_64 | **First product platform** | Daemon/CLI product path; **native installer/service later** (not P1-T08) | Platform credential store direction (same fail-closed boundary) | CI Windows/MSVC |

## 2. Engineering and non-product hosts

| Host | Status | Notes |
|---|---|---|
| CI Ubuntu | Authoritative Linux evidence | Required green for code merges |
| CI Windows/MSVC | Authoritative Windows evidence | Required green for code merges |
| Local Windows GNU / MinGW | **Non-supported** | P0-T01 linker exit 121; must not block CI-green work |
| WSL2 as product runtime | **Not first-ship product** | Existing Pi admission may refuse WSL2; do not market as Personal product host without a later ADR |
| Linux aarch64 / macOS | Deferred | Out of first-ship matrix |

## 3. Distribution

| Channel | Decision |
|---|---|
| GitHub Releases | **Yes** — public checkable artifacts |
| First public artifact | Linux x86_64 bundle (P1-T08 implement; P7-T01 SBOM/attest) |
| Vendor Pi in bundle | **No** — user-local pin (P0-T06) |
| Vendor Node in bundle | **No** |
| crates.io publish | **No** (until later P7 decision) |
| npm public publish | **No** (until later P7 decision) |

## 4. Gate mapping (honest)

| Gate | Relation to this matrix |
|---|---|
| G0 | Needs this matrix **and** remaining Phase 0 items (notably P0-T06). Matrix alone is not G0 pass. |
| B01 | Clean Linux VM install to first dialogue runs; Windows B01 parity requires a future written Gate if claimed. |
| RC / P7-T06 | Full multi-platform support claims only with executed evidence. |

## 5. References

- [ADR-0025](../adr/0025-personal-license-platform-distribution.md)
- [THIRD-PARTY-NOTICES.md](../legal/THIRD-PARTY-NOTICES.md)
- [PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)
