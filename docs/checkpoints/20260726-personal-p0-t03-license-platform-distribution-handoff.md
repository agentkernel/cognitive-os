# 20260726 Personal P0-T03 License / Platform / Distribution Handoff

## 1. Task Snapshot

- Task: `P0-T03` — License、首发平台与分发决策
- Date: 2026-07-26
- Branch: `lane/personal-p0-t03-license-platform-distribution`
- Base: `main@cc29335`
- Lane: Personal product legal/distribution decision (docs + package metadata; no authority runtime change)
- Status: **done** for decision package acceptance. Not G0, B01-B12, Profile, or release claim.

## 2. Owner decisions recorded

| Topic | Decision |
|---|---|
| License | Apache-2.0 single license |
| First product platforms | Linux x86_64 + Windows x86_64 |
| Distribution | GitHub Release checkable bundle; **do not** vendor Pi/Node; user-local Pi install |
| crates.io / npm publish | Still **off** (`publish=false` / `private: true`) |

## 3. Completed in this atomic batch

- Root `LICENSE` (Apache-2.0 official text)
- Root `NOTICE` (first-party attribution + pointer to third-party inventory)
- `docs/adr/0025-personal-license-platform-distribution.md`
- `docs/legal/THIRD-PARTY-NOTICES.md` (curated inventory; SBOM deferred to P7-T01)
- `docs/plan/PERSONAL-SUPPORT-MATRIX.md`
- Workspace license metadata: Cargo `license = "Apache-2.0"` + member `license.workspace = true`; TS `"license": "Apache-2.0"`
- Formal ledger / PROGRESS / DEVELOPMENT-PLAN pending items / `plan.md` P0-T03 card / docs map updated

## 4. Not completed / out of scope

- P0-T06 Pi version/Extension/RPC PoC (now unblocked by dependency)
- P1-T08 Linux installer / systemd user service implementation
- Windows installer/service implementation
- Machine SBOM / artifact attestation (P7-T01)
- Enabling crates.io or npm publish
- G0 closure (still requires P0-T06)
- B01-B12 / Profile claims

## 5. Tests and evidence

| Check | Status | Result |
|---|---|---|
| `pnpm run check:consistency` | executed | OK (273 requirements, 55 error codes, 63 schemas, 85 vectors) |
| `git diff --check` | executed | clean (CRLF→LF normalize warnings only on metadata files) |
| Rust/TS behavioral product tests | not applicable | no runtime behavior change |
| Personal Gates / B01-B12 / Profile | not-run | no claim |
| Local Windows GNU cargo | non-supported host | not required for this docs/metadata batch |

## 6. Design and safety boundaries

- Pi/Node must not be redistributed in CognitiveOS release artifacts
- Secrets remain out of config/SQLite/env/argv/logs/evidence
- Dual first platforms do **not** auto-claim Windows B01 parity; B01 remains Linux-oriented until a future Gate is written
- First public checkable artifact remains Linux x86_64 bundle (P1-T08)

## 7. Next entry

1. **P0-T06** — Pi version, Extension, and RPC compatibility PoC (deps: P0-T03 done).
2. After P0-T06: G0 evidence package; then P1-T07 Pi package/extension product path.
3. Suggested prompt: implement P0-T06 with pinned Pi integrity fixtures and fail-closed non-authority Extension surface; do not vendor Pi into the repo release bundle.

## 8. Snapshot

- PROGRESS updated: yes (P0-T03 done; G0 still waits on P0-T06; no Profile claim)
- Formal Personal ledger updated: yes (`done`, 12/51)
- PR / CI: pending push
