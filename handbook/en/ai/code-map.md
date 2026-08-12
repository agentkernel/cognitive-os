---
doc_id: ai.code-map
locale: en
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: Cargo.toml
  - path: pnpm-workspace.yaml
fingerprint: "sha256:793f292954380524a0ebe0e71ee3ae9e49c7955cbbb750778f349e4136ae2c93"
non_claims:
  - Component presence is not a Gate, release, or Profile claim; wiring status is tracked in the developer execution-chain page.
---

# Code map

Ten Rust crates, three Rust apps, four TypeScript packages/apps. Dependency direction:
`contracts → domain → kernel → store/runtime/management → apps`. The deterministic
core (`cognitive-kernel`) has no HTTP, SQLite, or model SDK dependency by design.

| Unit | Role | Load-bearing entry points |
|---|---|---|
| `crates/cognitive-contracts` | canonical JSON/digest, schema codegen, generated Rust bindings (53 modules), error registry (55 codes), golden parity | `canonical.rs`, `bin/contracts-codegen.rs`, `generated/mod.rs` |
| `crates/cognitive-domain` | IDs, capability arithmetic, embedded transition tables, versions | `transitions.rs` (`table`, `find_edge`), `capability.rs` (`intersect_chain`) |
| `crates/cognitive-kernel` | the deterministic authority core: 10-step transition gate, intent chain, context pipeline/caches, Effect protocol, loop/WIA/continuation, recovery, tool registry, ports | `engine.rs` (`TransitionEngine`), `intent_chain.rs`, `effects.rs` (`EffectProtocol`), `harness.rs` (`LoopDriver`), `ports.rs` |
| `crates/cognitive-store` | SQLite WAL adapter: migrations v1–v23 (+installation v1–v4), scheduler leases, Memory/Skill/Context/Artifact stores, backup planning | `sqlite/`, `migration.rs`, `personal_db.rs` (`prepare_personal_databases`), `scheduler.rs` |
| `crates/cognitive-runtime` | execution layer: Linux bundle verify/install/service, Pi acquisition/registration/lifecycle, adapters/hooks/compaction/learning planners, perf surfaces | `installer.rs`, `linux_bundle*.rs`, `agent_registration.rs`, `scheduler_service.rs`, `perf.rs` |
| `crates/cognitive-management` | deterministic management plane (inspect/stop/revoke/reconcile), privileged sessions, R1 approvals, audit port, TaskApplicationService | `plane.rs` (`ManagementPlane`), `session.rs`, `task_application.rs` |
| `crates/cognitive-secret` | SecretStore backends (Linux Secret Service; fail-closed elsewhere), Provider config/discovery/transport | `store.rs` (`SecretStore`), `provider_service.rs`, `provider_transport.rs` |
| `crates/cognitive-provider-transport` | loopback TLS Provider fixture for deterministic tests | `bin/p1_t09_provider_fixture.rs` |
| `crates/cognitive-akp` | AKP 0.2 envelope parsing/digests, in-memory watch log | `lib.rs` (`parse_request`, `WatchLog`) |
| `crates/cognitive-conformance` | conformance runner: 89 vectors, five-state report, 41-flip self-check | `src/main.rs`, `src/exec/` |
| `apps/kernel-server` | the Personal daemon (`--personal`): loopback HTTP, auth channels, readiness/doctor, Provider proxy, scheduler authority, tool executor, verification executor | `src/personal/server.rs` (`serve_personal_loopback`), `scheduler_authority/`, `task_api.rs` |
| `apps/admin-cli` | two binaries: `cognitive` (product CLI) and `admin-cli` (management fallback) | `src/cognitive_main.rs`, `src/main.rs`, `src/personal_cli/` |
| `apps/pi-agent-adapter` | pinned Pi child-process adapter; only `daemon-candidate` is operational | `src/main.rs`, `src/lib.rs` |
| `packages/pi-cognitiveos` | Pi extension: daemon discovery/client, provider bridge, default-deny tools | `src/extension.ts` (`registerCognitiveOsExtension`), `src/daemon-client.ts` |
| `packages/sdk-ts` | channel-isolated AKP client SDK: envelopes, transports, watch consumer | `src/client.ts`, `src/channel.ts`, `src/watch.ts` |
| `packages/contracts-ts` | canonical JSON/digest twin + 55 generated TS modules + golden emitters | `src/canonical.ts`, `src/generated/` |
| `apps/agent-shell` | reusable Shell session library (preview → submit → attach/cancel); no TUI | `src/session.ts` (`ShellSession`) |

Key real call chains (details in the [developer guide](../developer/README.md)):

- CLI init: `cognitive init` → `prepare_personal_databases` → `SecretStore` → `ProviderDiscoveryService` → snapshot persistence.
- Daemon start: `serve_personal_loopback` → migrations → recovery → one private scheduler tick → bind → endpoint publication.
- Task admission: `POST /task/*` → `TaskApi` → `KernelTaskApplicationService` → `cognitive_kernel::intent_chain` → SQLite.
- Pi conversation: Pi extension → `POST /provider/v1/chat/completions` → daemon-owned SecretStore + `RustlsProviderTransport`.
- Install: `deploy/linux/install.sh` → `linux_bundle_installer` → verify → stage → health → activate (single-service transaction).

Execution wiring gaps you must not paper over are listed in
[execution-chain status](../developer/execution-chain-status.md).
