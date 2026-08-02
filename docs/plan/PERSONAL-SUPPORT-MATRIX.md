# CognitiveOS Personal Support Matrix

- Status: owner-accepted product support policy
- Original decision: ADR-0025, 2026-07-26
- Linux 1.0 updates: ADR-0034 and ADR-0036
- Current Gate status: [PROGRESS.md](PROGRESS.md) `Current snapshot`
- Environment evidence: [PERSONAL-TEST-ENVIRONMENTS.md](PERSONAL-TEST-ENVIRONMENTS.md)

This file owns supported/deferred platform and distribution policy. It does not
copy current campaign results and is not B01-B12, `GMVP-LINUX`, release or
Profile evidence.

## 1. Product platforms

| Platform | Architecture | Product policy | Installation/secret boundary |
|---|---|---|---|
| Linux | x86_64 | **Personal 1.0 target and first public release** | GitHub Release checkable bundle; one `cognitiveos-personal.service` user unit; loopback 48181; FreeDesktop Secret Service; `GMVP-LINUX` required |
| Windows | x86_64 | product target after Linux 1.0; no install-parity claim yet | P7-T07 owns native credential backend, installer/service and independent B01-W |
| WSL2 | x86_64 guest | engineering environment only, not a 1.0 product runtime | cannot substitute for native Linux/Windows product evidence |
| Linux aarch64, macOS, mobile | various | deferred | requires later product decision and independent qualification |

## 2. Agent support

| Agent | Linux 1.0 support policy |
|---|---|
| Pi `0.81.1` | only planned product-qualified Agent; also hosts the Shell under a separate client identity; requires B09 and release manifest inclusion |
| OpenClaw, Hermes, Codex, WorkBuddy, others | not supported by Linux 1.0; each requires exact adapter/package/protocol pins, independent campaigns and explicit release inclusion |

Installing several Agent packages does not imply Multi-Agent orchestration.
Pi Shell evidence and Pi managed-Agent evidence are separate, and neither can
qualify another adapter.

## 3. Distribution and acquisition

| Surface | Decision |
|---|---|
| CognitiveOS product | public, verifiable Linux x86_64 artifacts through GitHub Releases after `GMVP-LINUX` |
| Pi in product archive | **No**; Pi is not vendored or redistributed in the CognitiveOS bundle |
| Pi default acquisition | Personal fetches exact `@earendil-works/pi-coding-agent@0.81.1` from the fixed official npm origin after user preview, verifies identity/SRI/digests and commits a production-signed acquisition lock |
| Node in product archive | **No**; a compatible user/system Node is an explicit prerequisite |
| crates.io/npm publication of CognitiveOS packages | disabled until a later owner decision |
| Provider/user secret | native approved Secret Store only; never product/Agent archive, argv, ordinary config, SQLite, logs or evidence |

Npm SRI is an integrity input, not publisher provenance. The acquisition lock
means CognitiveOS admitted exact upstream bytes; it does not claim upstream
authorship.

## 4. Claim boundaries

- Linux 1.0 requires formal B01, P2 B02/B04/B05/B12, managed-Pi B09 and P7
  production-operability evidence.
- B10/MCP, Memory, Multi-Agent, Web UI and Windows B01-W do not block Linux
  1.0 and cannot appear in its release claim.
- Ordinary Ubuntu/Windows CI, WSL, fake-systemd fixtures and
  `personal-linux-native-01` provide only their registered implementation
  evidence unless a formal campaign explicitly includes them.
- Product release does not imply CognitiveOS Core Profile `implemented`.

## 5. References

- [Linux 1.0 scope](../product/personal/linux-1.0-scope.md)
- [ADR-0025](../adr/0025-personal-license-platform-distribution.md)
- [ADR-0034](../adr/0034-personal-mvp-first-single-service-release-train.md)
- [ADR-0035](../adr/0035-personal-pi-shell-and-managed-agent-role-separation.md)
- [ADR-0036](../adr/0036-personal-linux-1-0-and-official-pi-acquisition.md)
- [THIRD-PARTY-NOTICES.md](../legal/THIRD-PARTY-NOTICES.md)
- [PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)
