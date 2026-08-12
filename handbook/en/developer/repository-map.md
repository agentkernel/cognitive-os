---
doc_id: dev.repo-map
locale: en
kind: reference
audience: [developer, ai]
status: implemented
generated: false
sources:
  - path: Cargo.toml
  - path: pnpm-workspace.yaml
  - path: package.json
fingerprint: "sha256:46b917bfe579dbf7c271b70078934ab94e4005fe0d05477789e247eb1a021ffe"
non_claims:
  - Directory presence is not implementation or Gate evidence; wiring status lives in execution-chain-status.
---

# Repository map

| Tree | Content | Change discipline |
|---|---|---|
| `crates/` | ten Rust crates (contracts, domain, kernel, store, runtime, management, secret, provider-transport, akp, conformance) | implementation surface; kernel stays free of HTTP/SQLite/model SDKs |
| `apps/` | `kernel-server` (daemon), `admin-cli` (two binaries), `pi-agent-adapter`, `agent-shell` (TS lib), `cognitiveos-console` (deprecated stubs) | implementation surface |
| `packages/` | `pi-cognitiveos` (Pi extension), `sdk-ts`, `contracts-ts` | implementation surface; `*/src/generated/` is generator-owned |
| `specs/` | requirement/error/state-domain registries, 74 JSON schemas, 5 transition tables, narrative companions | architecture contracts — Lane-CTR only; never bent to fit code |
| `conformance/` | 89 vectors + README | contract assets — same protection |
| `tests/` | baseline/e2e/faults/security indexes + `tests/golden/` cross-language fixtures | golden JSONs are generated |
| `tools/` | Node checkers/generators (consistency, traceability, gate evaluators, handbook, docs-sync gate) | syntax-checked and tested by `@cognitiveos/repo-tools` |
| `.githooks/` | repo-tracked pre-commit/pre-push docs-sync hooks (opt-in: `pnpm run hooks:install`) | thin `sh` wrappers over `tools/src/docs-sync-gate.mjs` |
| `docs/` | governance, formal plan + current snapshot + lease ledger, product/architecture design, ADRs, standards, checkpoints, prompts | canonical documentation system; the handbook links to it and never edits it |
| `handbook/` | this bilingual derived documentation system | validated by `tools/src/check-handbook.mjs` |
| `deploy/` | inspected installer template + systemd unit templates | rendered by the campaign builder |
| `scripts/` | V01 auto-run orchestrators (stale pins; not a current gate) | historical |
| `History/` | frozen archive | never read or cite |

Root files: `Cargo.toml` (workspace + shared lints), `package.json` (pnpm scripts),
`pnpm-workspace.yaml`, `rust-toolchain.toml` (pinned 1.97.1), `AGENTS.md` (agent
entry), `plan.md` (research detail), whitepaper + reviews + RFC-0001 (informative /
frozen), `LICENSE`/`NOTICE` (Apache-2.0; Pi and Node are not redistributed),
`llms.txt` (AI pointer).

Dependency direction (enforced by crate manifests):
`contracts → domain → kernel → {store, management, runtime} → apps`, with
`secret`/`provider-transport`/`akp` as leaf utilities and `conformance` consuming
everything for behavioral gates.
