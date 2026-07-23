# Lane-CTR Ordinary Core AUDIT machine-registration handoff

- Date: 2026-07-23
- Branch: `lane/ctr-v02-ordinary-core-audit-machine-registration`
- Scope: only Ordinary Core `status.inspect` minimal AUDIT registration.

## Candidate preservation and mapping

Candidate verification remains `cargo test -p cognitive-contracts --test
ordinary_core_audit_candidate`; the candidate JSON payloads were not edited.

| Candidate asset | Registered asset | Exact permitted differences | Business fields / constraints |
|---|---|---|---|
| `privileged-read-decision.candidate.schema.json` | `specs/schemas/privileged-read-decision.schema.json` | `$id`, formal title | mechanically equal `required`, `properties`, `allOf`, `additionalProperties` |
| `audit-commit-receipt.candidate.schema.json` | `specs/schemas/audit-commit-receipt.schema.json` | `$id`, formal title | mechanically equal `required`, `properties`, `additionalProperties` |
| `commit-privileged-read-decision.candidate.operation.json` | `specs/core/ordinary-core-audit.md` | formal companion path/status | same minimal port responsibility and failure ordering |
| `digest-rules.candidate.json` | `specs/core/ordinary-core-audit.md` | formal companion path/status | same domains, projections, exclusions, canonicalization and SHA-256 |

## Registration result matrix

| Item | Status |
|---|---|
| formal decision schema | registered |
| formal receipt schema | registered |
| digest domains/projections/exclusions | registered in normative companion |
| minimal port responsibility | registered in normative companion; runtime obligations explicitly not represented as schema proof |
| Rust/TS bindings and schema digest constants | generated |
| implementation binding consumption | pending Lane-RUN |
| conformance vectors/behavior | pending Lane-CFR |
| CA-0 GO | no |
| High-Assurance AUDIT | deferred |
| Profile implemented | 0 |

No full AUDIT record/stream/checkpoint/export family, signature, independent AUDIT
service, multi-approval, or consensus capability was registered.

## 2026-07-23 verification and delivery closure

- Start branch / HEAD: `lane/ctr-v02-ordinary-core-audit-machine-registration`
  at `11fbe4b30608cd916d18a3a20af5cbe91d7d8937` (equal to `origin/main` at
  verification start; no local commits).
- Intended registration paths are limited to `crates/cognitive-contracts/`
  codegen/generated modules/tests, `packages/contracts-ts/` generated bindings
  and tests, the two `specs/schemas/` assets, `specs/core/ordinary-core-audit.md`,
  `docs/traceability/matrix.yaml`, `docs/plan/{PROGRESS,PARALLEL-LANES}.md`, and
  this handoff.
- Unrelated untracked `.cursor/skills/**`, `*gen**_xlsx.py`,
  `artifacts/_local/**`, and Chinese-named `.xlsx`/`.md` paths were recorded only,
  left in place, and excluded. They were unchanged after verification; no
  `personal-blog/**` path was touched.

| Command | Result | Duration | Evidence |
|---|---|---:|---|
| `cargo build --workspace` | fail (environment) | 6.9s | GNU linker cannot find `-lgcc_eh` / `-lgcc` |
| `cargo test --workspace` | fail (environment) | 5.2s | same third-party build-script linker failure; no test executed |
| `cargo clippy --workspace --all-targets` | fail (environment) | 6.0s | same third-party build-script linker failure |
| `cargo fmt --check` | pass | 4.4s | formatting check clean |
| `cargo test -p cognitive-contracts --test ordinary_core_audit_registration` | fail (environment) | 4.5s | same linker failure before test execution |
| `cargo test -p cognitive-contracts --test ordinary_core_audit_candidate` | fail (environment) | 4.6s | same linker failure before test execution |
| `cargo test -p cognitive-contracts --test generated_types` | fail (environment) | 4.5s | same linker failure before test execution |
| `pnpm --filter @cognitiveos/contracts-ts build` | pass | 8.7s | TypeScript compilation passed |
| `pnpm --filter @cognitiveos/contracts-ts test` | pass | 7.0s | 39 pass, 0 fail |
| `pnpm run check:consistency` | pass | 7.8s | 273 requirements, 55 errors, 63 schemas, 84 vectors |
| `node tools/src/gen-matrix.mjs --check` | pass | 2.5s | matrix is up to date |
| `git diff --check` | pass | 1.7s | no whitespace errors |

No command timed out. The missing GNU runtime libraries are a local toolchain
blocker, not a Lane-CTR implementation failure; no corrective code change was
made. Because the Rust gates did not obtain pass, there is no staging, commit,
push, PR, or CI run. Restore a complete MinGW runtime (`libgcc_eh` and `libgcc`)
and rerun the Rust gates before delivery.

Remaining non-claims: Lane-RUN binding consumption pending; Lane-CFR conformance
behavior pending; CA-0 GO = no; High-Assurance AUDIT deferred; Profile
implemented = 0.

## GNU recovery verification supersedes the local blocker

The previous GNU linker blocker is superseded for this session. MSYS2 GCC
16.1.0 at `D:\toolchains\msys64\mingw64\bin\x86_64-w64-mingw32-gcc.exe`
was selected only through session `PATH` and
`CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER`; `CARGO_TARGET_DIR` was
`D:\toolchains-temp\cognitiveos-rust-target`. No repository config, Cargo
manifest, Rust toolchain file, or permanent PATH setting was changed. The
driver is `x86_64-w64-mingw32` and supplies both `libgcc.a` and `libgcc_eh.a`.

| Command | Result | Duration |
|---|---|---:|
| `cargo build --workspace` | pass | 75.8s |
| `cargo test --workspace` | pass; includes registration, candidate, and generated-types suites | 229.6s |
| `cargo clippy --workspace --all-targets` | pass | 86.2s |
| `cargo fmt --check` | pass | 4.4s |
| `cargo test -p cognitive-contracts --test ordinary_core_audit_registration` | pass, 3/3 | 102.9s |
| `cargo test -p cognitive-contracts --test ordinary_core_audit_candidate` | pass, 2/2 | 11.4s |
| `cargo test -p cognitive-contracts --test generated_types` | pass, 9/9 | 12.5s |
| `pnpm --filter @cognitiveos/contracts-ts build` | pass | 9.2s |
| `pnpm --filter @cognitiveos/contracts-ts test` | pass, 39/39 | 6.9s |
| `pnpm run check:consistency` | pass, 273 requirements / 55 errors / 63 schemas / 84 vectors | 8.3s |
| `node tools/src/gen-matrix.mjs --check` | pass | 2.8s |
| `git diff --check` | pass | included in 2.8s static check invocation |

No command timed out and no code repair was required. Frozen candidate files and
their reviewed bytes/digests were not edited. The unrelated untracked paths
remain excluded and untouched; `personal-blog/**` was not touched. This evidence
only validates Lane-CTR machine assets and local gates: Lane-RUN consumption and
Lane-CFR behavior remain pending, CA-0 GO remains no, and Profile implemented
remains 0.
