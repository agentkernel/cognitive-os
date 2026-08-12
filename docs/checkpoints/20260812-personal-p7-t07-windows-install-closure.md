# 2026-08-12 — P7-T07 Windows install surface: D01-D03 complete, task blocked on B01-W execution prerequisites

- Task: P7-T07 (Windows 安装面：credential 后端、installer/service 与 B01-W Gate)
- Lease: `lease/personal/P7-T07/windows-install-surface` (closed by this record;
  branch and Draft PR retained for resumption)
- Branch / PR: `personal/P7-T07-windows-install-surface` /
  [PR #200](https://github.com/agentkernel/cognitive-os/pull/200) — on
  2026-08-12 the owner directed landing the validated D01-D03 implementation
  and this blocked accounting on `main`; merging the code does not change the
  task status, satisfy the B01-W acceptance item, or create any claim
- Validated revision: merge head `13e772a` (includes D01 `fe11da8`,
  D02+ledger `2f1a8e9`, D03 `eb15327`, and the origin/main merge after the
  P9-T04 closure landed)
- Required CI: run `31570126985` **passed** on `CI-UBUNTU-01` (2m38s) and
  `CI-WINDOWS-MSVC-01` (6m52s) for `13e772a`; local `cargo fmt --all`,
  `git diff --check`, and `check:consistency` passed
- Task status: **blocked** (see §3); this record is the single blocked-closure
  handoff required for a genuine external prerequisite

## 1. Acceptance mapping

| Acceptance item | Status | Evidence |
|---|---|---|
| Windows credential store 后端（同 fail-closed 边界，无明文 fallback） | **implemented + CI-validated** | `WindowsCredentialManagerStore` (ADR-0052 §1): fixed audited PowerShell helper from the absolute system path, secrets only on helper stdin/stdout as hex, `CRED_PERSIST_LOCAL_MACHINE` only, 2560-byte blob ceiling enforced pre-write, fail-closed selection preserving the frozen P1-T02 contract, admin-cli init arm. Failure-first negatives: non-Windows fail-closed (Ubuntu CI), real Credential Manager roundtrip/rotate/delete, oversized-rejection-with-nothing-stored, foreign/absent-ref `NotFound`, redaction, script-embedding rejection, hex/base64 encoder negatives (`crates/cognitive-secret/tests/p7_t07_windows_credential_store.rs`, unit tests). CI run `31570126985`. Product-native qualification remains with B01-W. |
| 可检查 installer/service | **implemented + CI-validated** | `deploy/windows/install.ps1` bootstrap template and `deploy/windows/cognitiveos-personal-task.xml` per-user least-privilege scheduled-task template (ADR-0052 §2), mirroring the Linux install.sh/user-service contract and placeholders. Static required/forbidden-fragment checks on every platform; behavioral unrendered/version-mismatch/malformed-digest/non-HTTPS/extra-argument rejections and least-privilege task-XML parse on Windows CI (`crates/cognitive-runtime/tests/p7_t07_windows_install_surface.rs`). Download/delegation behavior stays inspectable-only until real rendered artifacts exist. |
| 专门 B01-W Gate 编写 | **authored** | ADR-0052 §3 policy (fixed N=6, >=5/6, zero critical, aggregate + independent verifier) plus preregistration `20260812-personal-p7-t07-b01-w-preregistration.md` and the required-not-provisioned `B01-W-DESKTOP-001` registration in `PERSONAL-TEST-ENVIRONMENTS.md` §11. |
| 专门 B01-W Gate 执行 | **blocked — not executed** | See §3. No attempt, denominator entry, or claim exists. |
| 不阻塞 Linux RC | satisfied | All work is additive to the Windows surface; no Linux path, Gate, or release semantics changed. |
| 未执行前不得声称 Windows install parity（ADR-0025） | satisfied | No parity, B01-W, Gate, release, or Profile claim anywhere in this delivery; CI evidence is implementation evidence only. |

## 2. Delivered slices

- `P7-T07/D01` done — Windows Credential Manager production backend + selection
  + admin-cli arm + failure-first negatives (implemented at `fe11da8`,
  validated at `13e772a` by CI `31570126985`).
- `P7-T07/D02` done — inspectable bootstrap installer + scheduled-task
  templates + static/behavioral negatives (implemented at `2f1a8e9`, validated
  at `13e772a` by the same run).
- `P7-T07/D03` done — B01-W gate authoring: ADR-0052 §2/§3, preregistration,
  environment requirement registration (at `eb15327`, consistency validated
  locally and in the same CI run).
- `P7-T07/D04` blocked — this acceptance mapping plus the blocked record; the
  remaining acceptance item cannot be satisfied autonomously.

## 3. Blocked record

- `blocked_task_ids`: `P7-T07`
- `blocked_gate_ids`: `B01-W`
  (campaign `B01-W-clean-windows-first-install-first-conversation-001`,
  preregistered, not started)
- Blocking prerequisites (all outside this task's autonomous authority):
  1. **Windows release artifacts** — the release pipeline produces and signs
     Linux bundles only; `cognitiveos-windows-x86_64.zip`,
     `cognitiveos-windows-bundle-installer.exe`, and their
     manifest/SBOM/attestation signing path do not exist. Extending the
     release pipeline is new scope beyond the closed P7-T01 acceptance and
     needs an owner scope decision.
  2. **`B01-W-DESKTOP-001` provisioning** — no clean Windows campaign VM
     exists; creating one (Windows image and licensing included) is an owner
     infrastructure decision that standing authorization does not cover.
  3. **Operator availability** — the campaign requires graphical hidden-input
     Provider credential entry and an independent verifier disposition.
- Owner: repository owner.
- Single recovery action: provide prerequisites 1-2 (or an explicit owner
  disposition changing the B01-W scope), then execute the preregistered
  campaign on an exact pushed revision of this branch and complete D04
  acceptance mapping to `done`.
- The implementation and this record are merged to `main` by owner directive;
  the lease is closed and the task branch is deleted after the merge.
  Resumption starts from `main` under a new task lease once the prerequisites
  exist; the merge itself changes no acceptance, Gate, or claim state.

## 4. Non-claims

No Windows install parity, B01-W, B01, Gate, release, Profile, or containment
claim. CI evidence is `tested-supported-ci` implementation evidence only. The
`B01-Desktop-Linux-002` guest, `DEV-LINUX-NATIVE-01`, and every P9-T04 leased
path were never touched by this task.
