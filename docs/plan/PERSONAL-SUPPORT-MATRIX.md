# CognitiveOS Personal Support Matrix

- Status: owner-accepted product support policy
- Original decision: ADR-0025, 2026-07-26
- Linux 1.0 updates: ADR-0034, ADR-0036, ADR-0037 and ADR-0038
- Personal 2.0 adopted targets: ADR-0056 and ADR-0057
- Current Gate status: [PROGRESS.md](PROGRESS.md) `Current snapshot`
- Environment evidence: [PERSONAL-TEST-ENVIRONMENTS.md](PERSONAL-TEST-ENVIRONMENTS.md)

This file owns supported/deferred platform and distribution policy. It does not
copy current campaign results and is not B01-B12, `GMVP-LINUX`, release or
Profile evidence.

## 1. Product platforms

| Platform | Architecture | Product policy | Installation/secret boundary |
|---|---|---|---|
| Linux | x86_64 | **Personal 1.0 unified cognitive-resource target and first public release** | GitHub Release checkable bundle; one `cognitiveos-personal.service` user unit; loopback 48181; desktop Secret Service or approved headless encrypted vault; bounded Extended Home; headless/foreground operation; `GMVP-LINUX` required |
| Windows | x86_64 | product target after Linux 1.0; no install-parity claim yet | P7-T07 owns native credential backend, installer/service and independent B01-W |
| WSL2 | x86_64 guest | engineering environment only, not a 1.0 product runtime | cannot substitute for native Linux/Windows product evidence |
| Linux aarch64, macOS, mobile | various | deferred | requires later product decision and independent qualification |

## 2. Linux 1.0 cognitive-resource support

| Resource | Linux 1.0 supported baseline | Claim boundary |
|---|---|---|
| Memory | SQLite source of record, deterministic admission/policy, FTS5 + metadata filter, lifecycle/forget | no embedding/vector/graph or automatic cross-workspace recall claim |
| Skill | local package, immutable revision/digest, explicit import and Agent/Task/workspace binding | local Skill is not self-authorizing executable authority |
| Tool | native versioned Tool Registry with workspace read/search/write/patch, bounded process/check and read-only HTTP fetch operations | no MCP, dynamic marketplace or arbitrary generic shell claim |
| Context | real workspace/task/evidence sources, minimum Context Builder, unique Artifact CAS, delta/stable-prefix/cache correctness | B03 correctness is required; B06/B07 optimization benefit does not block 1.0 |
| Task | real daemon Resource + Task API/watch, server-owned preview/admission and private versioned unified projection | Shell/CLI/sidecar remain clients; no second Task state machine |
| Runtime | daemon-owned scheduler, bounded worker, Tool/process executor, cursor, fault/reconcile, checkpoint/evidence/verifier | process exit, Tool receipt, Provider success or `agent_end` does not complete a Task |

The six resources share a Personal projection and one UCR-01 Task trace without
sharing one giant schema or authority lifecycle. A public `ResourceSummary` is
not part of the 1.0 support claim.

## 3. Agent support

| Agent | Linux 1.0 support policy |
|---|---|
| Pi `0.81.1` + pinned per-Agent sidecar | **only Linux 1.0 RC product-qualified Agent integration**; Pi also hosts the Shell under a separate client identity; exact Agent/sidecar package, protocol, adapter, instance and process pins require B09 and release-manifest inclusion |
| DeepSeek Harness (dsh) Path B | implemented on a later post-1.0 train; **not** in the Linux 1.0 RC Agent claim; does not inherit Pi/B09 evidence |
| OpenClaw, Hermes, Codex, WorkBuddy, others | not supported by Linux 1.0; each requires exact Agent/sidecar package/protocol/adapter pins, independent campaigns and explicit release inclusion |

Installing several Agent packages does not imply Multi-Agent orchestration.
Pi Shell evidence and Pi managed-Agent + sidecar evidence are separate, and
neither can qualify another adapter. The sidecar is a client boundary, never an
authority writer or daemon bootstrap/management authority.

## 4. Operating and product modes

| Surface | Linux 1.0 policy |
|---|---|
| Product envelope | Standard Workspace plus Extended Home selected document/project roots and optional ordinary outbound network; Secret/SSH/GPG/browser credentials, authority/bootstrap data, Docker/system sockets, system directories and privilege management remain denied |
| Headless | supported target with native user-systemd and an approved encrypted vault; locked diagnostic start and SSH TTY unlock are required, optional unattended unlock may use only systemd encrypted credential material |
| Foreground | supported target for inspectable development/operator execution using the same daemon authority and sidecar contracts |
| Shell path | Pi-hosted Shell -> pinned Pi sidecar -> daemon application services |
| Deterministic path | CLI -> the same daemon application services; no policy or projection bypass |

## 5. Deferred and unsupported for Linux 1.0

| Capability | Policy |
|---|---|
| Embedding, semantic retrieval, vector or graph stores | post-1.0; SQLite FTS5 + metadata filtering is the supported baseline |
| MCP and dynamic Tool marketplace/ecosystem | post-1.0 B10 capability train; native Tool operations remain in 1.0; B10 MVP evidence exists but is **not** in the Linux RC claim |
| Multi-Agent | **disabled-NO-GO for this RC**; post-1.0 `P6-T01`..`T04` / B11 stay not-started and are not claimed |
| Web UI / Control Plane | delivered as a non-blocking post-1.0 surface (`clients/pc/web`); **not** in the Linux RC claim |
| Windows installer/service | post-Linux-1.0 P7-T07/B01-W; no install parity before independent evidence |
| non-Pi Agents | independent Agent + sidecar qualification and release inclusion required; dsh Path B is in this row, not in the Pi 1.0 claim |

## 6. Distribution and acquisition

| Surface | Decision |
|---|---|
| CognitiveOS product | verifiable Linux x86_64 bundle shape after `GMVP-LINUX`; a production GitHub Release and production signing ceremony remain owner-operated and are **not** claimed by the P7-T06 evidence package |
| Pi in product archive | **No**; Pi is not vendored or redistributed in the CognitiveOS bundle |
| Pi default acquisition | Personal fetches exact `@earendil-works/pi-coding-agent@0.81.1` from the fixed official npm origin after user preview, verifies identity/SRI/digests and commits a production-signed acquisition lock |
| Node in product archive | **No**; a compatible user/system Node is an explicit prerequisite |
| crates.io/npm publication of CognitiveOS packages | disabled until a later owner decision |
| Provider/user secret | approved `SecretStore` backend only: desktop Secret Service or headless encrypted vault; never product/Agent archive, service unit/credential payload, environment, argv, ordinary config, SQLite, logs or evidence |

Npm SRI is an integrity input, not publisher provenance. The acquisition lock
means CognitiveOS admitted exact upstream bytes; it does not claim upstream
authorship.

## 7. Claim boundaries

- `GMVP-LINUX` benchmark composition is exactly B01, B02, B03, B04, B05, B08,
  B09 and B12. P7-T08 separately requires the six-resource release manifest,
  production trust, native service, desktop/headless secret-backend evidence,
  lifecycle, backup/restore and doctor evidence.
- B06/B07 benefit, B10/MCP/dynamic Tool, B11/Multi-Agent, embedding/vector/graph,
  non-Pi Agents, dsh Path B, Web UI and Windows B01-W do not block Linux 1.0 and
  must not appear in this RC claim. P6 is recorded `disabled-nogo` for this RC.
- Ordinary Ubuntu/Windows CI, WSL, fake-systemd fixtures and
  `personal-linux-native-01` provide only their registered implementation
  evidence unless a formal campaign explicitly includes them.
- A Personal Linux RC declaration does not imply CognitiveOS Core Profile
  `implemented`. `implemented` counts only applicable MUST evidence.

## 8. Personal 2.0 adopted targets — not yet supported

The following are owner-accepted Personal 2.0 product targets. They are not
current support rows, implementation evidence, or Gate/release/Profile claims.
Existing Linux 1.0 support and Gate boundaries above remain unchanged.

| Personal 2.0 target | Adopted policy | Current support boundary |
|---|---|---|
| Desktop Control Plane | Primary entry and supervision center; target IA is Home / Agents / Work / Library / Activity / Settings, with Providers and System under Settings | **not yet supported as a Personal 2.0 target**; existing daemon-served Web UI evidence does not establish desktop packaging, target IA, or 2.0 qualification |
| Global Agent Shell | Advises, explains, navigates, and proposes daemon previews; remains candidate-only and never writes authority or claims Task completion | **not yet supported**; no Personal 2.0 desktop Shell implementation or qualification |
| Installed-Agent conversations | Vendor-specific adapters behind a common internal projection and honest capability matrix; native Agent applications remain usable | **not yet supported**; existing Pi/dsh/Codex adapter evidence does not establish a common conversation surface or transfer qualification |
| MCP seventh family | Server/package/connection/capability/binding/health/quarantine identities and lifecycle; advertised tools remain Tool candidates; resources/prompts route through Context/Skill admission; no generic `Resource` schema | **not yet supported**; P5-T03/P5-T04/B10 evidence remains bounded to its MCP Tool adapter/dynamic Tool scope and is not seventh-family implementation |
| Credential import | ADR-0055 user-initiated per-source consent; daemon-only read; approved SecretStore target; raw material never reaches UI or Agent | **`Requires-backend`** for every concrete source until a formal implementation task and focused negatives exist |

No Personal 2.0 Gate is created by this adopted-target table.

## 9. References

- [Linux 1.0 scope](../../personal/docs/product/linux-1.0-scope.md)
- [ADR-0025](../adr/0025-personal-license-platform-distribution.md)
- [ADR-0034](../adr/0034-personal-mvp-first-single-service-release-train.md)
- [ADR-0035](../adr/0035-personal-pi-shell-and-managed-agent-role-separation.md)
- [ADR-0036](../adr/0036-personal-linux-1-0-and-official-pi-acquisition.md)
- [ADR-0037](../adr/0037-personal-unified-cognitive-resource-substrate.md)
- [ADR-0038](../adr/0038-personal-agent-sidecar-linux-evolution-boundary.md)
- [ADR-0055](../adr/0055-personal-credential-import-boundary-and-a5-revision.md)
- [ADR-0056](../adr/0056-personal-2-0-desktop-control-plane.md)
- [ADR-0057](../adr/0057-personal-2-0-mcp-resource-family.md)
- [THIRD-PARTY-NOTICES.md](../legal/THIRD-PARTY-NOTICES.md)
- [PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)
