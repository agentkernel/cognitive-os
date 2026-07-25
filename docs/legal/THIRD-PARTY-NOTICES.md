# Third-Party Notices and Redistribution Inventory (P0-T03)

> **Status:** decision package complete (owner GO 2026-07-26)
> **Classification:** Personal product legal/distribution inventory. Not a
> registry REQ, schema, vector, Profile claim, G0 claim, or B01-B12 claim.
> **Authority:** [ADR-0025](../adr/0025-personal-license-platform-distribution.md)

## 1. First-party license

| Component | License | Location |
|---|---|---|
| CognitiveOS reference implementation (this repository) | Apache-2.0 | Root `LICENSE`, `NOTICE` |
| Rust workspace members | Apache-2.0 (`license = "Apache-2.0"`, `publish = false`) | Root `Cargo.toml` + member crates |
| TypeScript workspace packages | Apache-2.0 (`"license": "Apache-2.0"`, `"private": true`) | Root and package `package.json` |

Copyright holder for first-party notices: **The CognitiveOS Authors**.

## 2. Redistribution policy (owner decision)

| Asset class | Redistribute in Personal public release? | Notes |
|---|---|---|
| CognitiveOS daemon / CLI / store / secret crates | **Yes** (Apache-2.0) | GitHub Release checkable Linux bundle is the first public artifact path (P1-T08 / P7-T01) |
| TypeScript packages (`contracts-ts`, `sdk-ts`, `agent-shell`) | **Source available under Apache-2.0**; npm publish remains **off** until a later P7 decision | Keep `"private": true` |
| crates.io publish | **Off** until a later P7 decision | Keep `publish = false` |
| **Pi** (external Agent shell) | **No** | MIT-licensed upstream; users install a pinned compliant Pi themselves. Personal never vendors Pi into the release bundle (P0-T06 / P1-T07 pin integrity only) |
| **Node.js / npm / pnpm** | **No** | Host toolchain; installer verifies presence, does not ship Node |
| Provider API keys / secrets | **Never** | Native Secret Store only (ADR-0018 / ADR-0020) |
| User SQLite data / runtime tokens | **Never** in artifacts or support bundles without redaction | Doctor/support redaction is P7-T03 |

## 3. Direct Rust workspace dependencies (curated)

Licenses below are the commonly published SPDX identifiers for the pinned
direct dependencies used by this workspace. They are **informative inventory
for redistribution planning**, not a substitute for a release SBOM.

| Crate | Role | Typical SPDX |
|---|---|---|
| `serde` / `serde_json` | serialization | MIT OR Apache-2.0 |
| `serde_json_canonicalizer` | RFC 8785 helper | MIT OR Apache-2.0 |
| `sha2` | digests | MIT OR Apache-2.0 |
| `thiserror` | error types | MIT OR Apache-2.0 |
| `rusqlite` (+ bundled SQLite) | authority/installation store | MIT; bundled SQLite is public-domain / blessing style — verify at release |
| `uuid` | UUIDv7 IDs | MIT OR Apache-2.0 |
| `tempfile` | tests | MIT OR Apache-2.0 |

Process for release (P7-T01):

1. Generate a locked SBOM from `Cargo.lock` and `pnpm-lock.yaml`.
2. Diff against this inventory; fail release if any **copyleft that would force
   product relicensing** or unknown license appears without owner waiver.
3. Attach SPDX/CycloneDX + attestation to the GitHub Release asset set.
4. Refresh this file's "last verified" stamp in the same release PR.

**Last curated review:** 2026-07-26 (P0-T03 decision package; no SBOM generated).

## 4. Direct TypeScript workspace dependencies (curated)

| Package | Role | Typical SPDX |
|---|---|---|
| `typescript` | build | Apache-2.0 |
| `@types/node` | types | MIT |
| `ajv` / `ajv-formats` | schema validation in tools/tests | MIT |

Root and packages remain `"private": true`. No npm public publish is authorized
by this decision package.

## 5. External runtime components (not redistributed)

| Component | License (upstream) | Personal obligation |
|---|---|---|
| Pi (earendil-works / npm package family) | MIT | Pin version + integrity in P0-T06; user-local install; no vendor tarball in CognitiveOS release |
| Node.js | various (Node license) | Document minimum engine; do not ship Node in the Linux bundle |
| Linux Secret Service / `secret-tool` / platform credential stores | OS / distro | Probe/use only; do not reimplement plaintext fallbacks |
| Windows credential APIs (future Windows product path) | OS | Same secret boundary as ADR-0018/0020 |

## 6. Notices completeness checklist (P0-T03 acceptance)

- [x] Root `LICENSE` is Apache-2.0
- [x] Root `NOTICE` points to first-party copyright and this inventory
- [x] Owner GO recorded for license, first platforms, and distribution channel
- [x] Pi/Node explicitly **not** redistributed
- [x] crates.io / npm public publish remain **off** (no silent enable)
- [ ] Machine SBOM + release attestation — **deferred to P7-T01** (explicit non-claim)

## 7. Non-claims

Completing this inventory does **not** mean:

- G0 is passed (still requires P0-T06 and remaining Phase 0 closure evidence)
- B01-B12 or RC evidence exists
- Profile `implemented`
- A public release has been cut
- Windows installer/service exists (Windows is a first-class **product platform**
  decision; the first **public checkable bundle** remains Linux per P1-T08)
