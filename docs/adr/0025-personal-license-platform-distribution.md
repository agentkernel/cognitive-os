# ADR-0025: Personal License, First Platforms, and Distribution (P0-T03)

- Status: **Accepted** (owner GO 2026-07-26)
- Date: 2026-07-26
- Decision owners: CognitiveOS repository owner (interactive decision session)
- Classification: Personal product legal/platform/distribution decision. Not a
  CognitiveOS specification requirement, registry REQ, schema, transition,
  vector, Profile claim, G0 claim, or B01-B12 claim.
- Supersedes: open items P-1 / partial P-2 in
  `docs/plan/archive/DEVELOPMENT-PLAN.md` section 6 for Personal scope
- Related: P0-T01 baseline, ADR-0018/0020 secret boundary, P0-T06 Pi pin,
  P1-T08 Linux installer, P7-T01 release/SBOM
- Updated by: ADR-0036. The no-vendoring/no-Node-bundle decision remains in
  force; the former user-manual Pi installation requirement is superseded for
  the Linux 1.0 default path by product-owned acquisition from the fixed
  official npm source.

## Context

Personal Phase 0 required an owner GO/NO-GO on:

1. Root repository license (no `LICENSE` existed; Cargo `publish = false`;
   npm `"private": true`).
2. First-ship product platforms versus engineering CI hosts.
3. Public distribution channel and whether Pi/Node may be vendored.

Without these decisions, P0-T06 (Pi PoC), P1-T08 (installer), and P7-T01
(release pipeline) cannot legally proceed.

Owner answers (2026-07-26 interactive session):

| Topic | Choice |
|---|---|
| License | **Apache-2.0** single license |
| First product platforms | **Linux x86_64 + Windows x86_64** |
| Distribution | **GitHub Release checkable bundle**; **do not vendor Pi/Node**; user installs compliant Pi locally |

## Decision

### 1. License

1. First-party CognitiveOS code in this repository is licensed under
   **Apache License, Version 2.0**.
2. Root `LICENSE` holds the full license text; root `NOTICE` holds first-party
   attribution and a pointer to `docs/legal/THIRD-PARTY-NOTICES.md`.
3. Rust workspace declares `license = "Apache-2.0"` while keeping
   `publish = false`.
4. TypeScript packages declare `"license": "Apache-2.0"` while keeping
   `"private": true`.
5. Contributions are accepted under Apache-2.0 unless a separate written
   agreement states otherwise.

### 2. First product platforms and support matrix

| Class | Platforms | Meaning |
|---|---|---|
| **Product first platforms** | Linux x86_64, Windows x86_64 | Personal product targets for daemon/CLI/docs; both appear in the support matrix |
| **Authoritative CI evidence hosts** | Ubuntu (Linux) + Windows/MSVC | P0-T01 baseline; required green for code merges |
| **Non-supported engineering hosts** | Windows GNU/MinGW (linker exit 121), other unlisted hosts | May fail locally; never block CI-green merges; not product targets |
| **Explicitly deferred product platforms** | Linux aarch64, macOS, mobile, WSL2-as-product | Not first-ship; may be tracked later without rewriting this ADR |

Platform-specific install surfaces:

| Platform | First public install surface | Notes |
|---|---|---|
| Linux x86_64 | Checkable GitHub Release bundle + **systemd user service** (P1-T08) | B01 clean-run evidence remains Linux VM oriented |
| Windows x86_64 | Product platform for daemon/CLI; **Windows service/installer is a later task** under the support matrix | Must not claim B01 Windows parity until a dedicated installer Gate is written and executed |

Secret and transport boundaries (ADR-0018/0019/0020/0022) apply on both
product platforms. Platform credential backends may differ; plaintext
fallback remains forbidden.

### 3. Distribution and redistribution

1. **Public distribution channel:** GitHub Releases for verifiable artifacts.
2. **First public artifact class:** Linux x86_64 checkable bundle (digest,
   verifier, interruption/rollback tests in P1-T08; SBOM/attestation in P7-T01).
3. **Do not redistribute:** Pi binaries/packages, Node.js runtimes, Provider
   keys, user data, or unredacted secrets in any release or support artifact.
4. **Pi obligation (updated by ADR-0036):** pin version + integrity in P0-T06;
   the Linux 1.0 default path performs product-owned acquisition of the exact
   approved package from the fixed official npm origin. Pi remains outside the
   release bundle, and Extension/RPC remain non-authority clients. Manual
   user-local Pi paths are development/import modes, not the default 1.0 path.
5. **crates.io / npm public publish:** remain **disabled**. Enabling publish is
   a separate P7 owner decision and is not authorized by this ADR.
6. **Release manifest fields (define only; implement in P7-T01):**
   - `product_id`, `version`, `target_triple`, `artifact_digest`
   - `license_spdx` (`Apache-2.0`), `notice_ref`, `third_party_inventory_ref`
   - `sbom_digest` (P7-T01), `attestation_ref` (P7-T01)
   - `pi_pin` / `pi_integrity` (from P0-T06; reference only, not embedded binary)
   - `profile_claim` / `gate_claim` always explicit non-claim unless RC evidence exists

### 4. GO / NO-GO summary

| Gate question | Result |
|---|---|
| May Personal development continue under Apache-2.0? | **GO** |
| May public GitHub Release Linux bundles be planned? | **GO** (implementation still P1-T08/P7-T01) |
| May Pi/Node be vendored into the bundle? | **NO-GO** |
| May crates.io/npm publish be enabled now? | **NO-GO** |
| Is G0 complete? | **No** — P0-T06 still open; G0 requires remaining Phase 0 evidence |
| Is B01 or Profile claimed? | **No** |

## Consequences

- P0-T06 may start (Pi pin/Extension/RPC PoC) without inventing license terms.
- P1-T08 implements the Linux checkable installer + user service against this
  redistribution policy.
- Windows product work may proceed for daemon/CLI parity, but must not invent a
  silent second license or vendor Pi.
- P7-T01 must generate SBOM/attestation; this ADR's curated inventory is not a
  substitute SBOM.
- `docs/plan/archive/DEVELOPMENT-PLAN.md` pending P-1 is closed for Personal; P-2 is
  partially closed (GitHub Release yes; registry publish still no).

## Rejected alternatives

1. **MIT-only** — weaker patent grant than Apache-2.0 for this product.
2. **MIT OR Apache-2.0 dual** — acceptable ecosystem norm, but owner selected
   single Apache-2.0 for simpler notices.
3. **Source-available / no public license** — would block public GitHub Release
   redistribution clarity.
4. **Vendor Pi into the release bundle** — expands supply-chain and notice
   surface; rejected by owner.
5. **Enable crates.io/npm publish now** — out of Personal G0 minimal scope.

## Non-claims

Accepting this ADR does **not** mean Profile `implemented`, G0 passed,
B01-B12 executed, RC ready, or that Windows installer/service already exists.
