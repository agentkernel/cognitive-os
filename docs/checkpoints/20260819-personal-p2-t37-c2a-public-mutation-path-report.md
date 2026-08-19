# P2-T37 C2a public mutation path running report

Task: `P2-T37`
Branch: `personal/P2-T37-c2a-public-mutation-path`
Lease: `lease/personal/P2-T37/c2a-public-mutation-path`
Scope: `P2-T37/D01` complete; `P2-T37/D02` public WorkspaceWrite and
WorkspacePatch complete; `P2-T37/D03` supported CI/docs/closure remaining

This is an implementation report, not a C2, Gate, release, Profile, B01,
evaluation, or Agent-benefit claim.

## Incremental validation log

1. **Formal registration — pass.** `pnpm run check:consistency` verified the
   P2-T37 task, D01-D03 slice register, Current snapshot counts, and active
   lease. The checker reported 275 requirements, 55 error codes, 74 schemas,
   89 vectors, and valid Personal plan/lease traceability.
2. **Public Extension regression and failure-first mapping negatives — pass.**
   `pnpm --filter @cognitiveos/pi-cognitiveos build` completed successfully,
   and the package test suite passed **99/99**. The focused additions cover
   the four-tool public governed surface, bounded WorkspaceWrite/Patch
   candidate fields, operation descriptors, canonical base64/preimage
   validation, and refusal when mutation input or expected preimage fields are
   absent. Pi-native bash/write/edit and unknown tools remain denied.
3. **Rust formatting — pass.** `cargo fmt --all -- --check` completed on the
   Windows GNU host without compiling or linking Rust.
4. **Diff hygiene — pass.** `git diff --check` reported no whitespace errors.
5. **Focused Rust test — environment-blocked.** `cargo test -p admin-cli
   personal_cli::pi` reached the Windows GNU linker and failed with the known
   `x86_64-w64-mingw32-gcc` exit 121 limitation. No product assertion failed;
   Rust execution remains routed to supported Linux/CI validation.
6. **Required CI `32272661185` — fail (stale allowlist assertion).** Ubuntu and
   Windows `verify` both failed only in
   `launch_preparation_disables_pi_native_tools_and_preserves_print_mode`, which
   still expected `--tools WorkspaceRead,WorkspaceSearch` after the public
   launcher correctly exposed Write/Patch. `required-ci` failed because verify
   failed. No product assertion other than that stale expectation failed. The
   follow-up repair updates that assertion to the four-tool allowlist and adds a
   daemon canonicalize negative for malformed/non-canonical mutation base64.
7. **Linux focused Rust at `b043d430` — pass.** Exact `DEV-LINUX-NATIVE-01`
   (`wuz@192.168.1.2`) ran `cargo fmt --all -- --check`,
   `cargo test -p admin-cli --lib personal_cli::pi` **9/9** including the
   repaired four-tool print-mode allowlist, and
   `cargo test -p kernel-server --bin kernel-server candidate_mutation` **3/3**.
   Local Windows GNU Rust remains `not-run` per `RUST-LINK-DEV-WIN-GNU-01`.
8. **D02 public WorkspaceWrite lifecycle — pass.** Fresh non-B01 runtime
   `/home/wuz/p2-t37-c2a-write-20260820` on loopback `127.0.0.1:48445` used the
   rebuilt `b043d430` daemon/CLI/adapter binaries and the public stdin-fed
   `cognitive pi launch --print --task-ref task://personal/p2-t37-public-write`
   caller. Authenticated admit, real Pi `WorkspaceWrite` candidate submission,
   scheduler `lease_acquired: 1`, durable Intent/Effect, passed current
   independent verification, and daemon `ACCEPTANCE_GRANTED` closed the Task at
   `COMPLETED` with `reconcile_class: closed`. The workspace file
   `c2a-write.txt` is 10 bytes with SHA-256
   `1f4f0de2f82b076bf2833308146c57e4b8a517731499c800ef1546027088a3f4`. No Pi-native
   filesystem or shell tool was used. This does not prove WorkspacePatch, C2,
   Gate, release, Profile, B01, or Agent-benefit.
9. **Linux Clippy `-D warnings` — pass.** Exact `DEV-LINUX-NATIVE-01` at
   `a5da7329` passed `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`
   and `cargo clippy -p admin-cli --all-targets --locked -- -D warnings`.
10. **D02 public WorkspacePatch runtime — partial / not-run.** A separate
    non-B01 root `/home/wuz/p2-t37-c2a-patch-20260820` on loopback
    `127.0.0.1:48446` was initialized, public Pi configured, and
    `c2a-patch.txt` seeded (13 bytes, SHA-256
    `7c3a0a01304ba98bbe58fe47fbc45565b4477d046335828375216c739e4bdffa`). Public
    doctor reports system/database/secret/daemon/Pi ready and Provider
    `provider_config_missing` / overall `blocked` / `first_conversation_ready:
    false`. File-copy/link of `provider.json` is refused. The product recovery
    is `cognitive init --reuse-existing-secret-binding` (opaque SecretRef only;
    no key recapture). Patch admit/launch remains `not-run`.
11. **Required CI `32282577281` on `e53f72ad` — pass.** Ubuntu verify, Windows
    verify, `resolve validation route`, and `required-ci` all completed
    successfully. This is supporting CI for the current documentation head; it
    does not close D02 while public WorkspacePatch remains `not-run`.
12. **Public `cognitive init --reuse-existing-secret-binding` — pass.** Exact
    `1f257b1b` CLI on `DEV-LINUX-NATIVE-01` bound Patch root
    `/home/wuz/p2-t37-c2a-patch-20260820` with `action:
    bound_existing_secret_ref`, `secret_material_written: false`, and
    `secret_ref_redacted: true`. Doctor then reported Provider ready,
    `secret_ref_resolves: true`, `selected_model_digest_matches: true`, overall
    `ready`, and `first_conversation_ready: true`. No key material was
    recaptured, copied, printed, or placed on argv.
13. **D02 public WorkspacePatch lifecycle — fail / not closed.** Two independent
    public Tasks (`task://personal/p2-t37-public-patch`,
    `task://personal/p2-t37-public-patch-retry`, and
    `task://personal/p2-t37-public-patch-strict`) each acquired
    `lease_acquired: 1` after `cognitive pi launch --print --task-ref ...` with
    the repo Patch fixture, then persisted Effect `NOT_EXECUTED` /
    `reconcile_class: must_reconcile`. The workspace file remained 13 bytes,
    SHA-256 `7c3a0a01304ba98bbe58fe47fbc45565b4477d046335828375216c739e4bdffa`.
    Verification and acceptance stayed absent; lifecycle stayed `ACTIVE`. The
    three Intents reused the exact fixture `parameters_digest`
    `sha256:015686ed221b3962c6e0a273ff969843df6f419f4f7640d2e6a7e7b172c66689`
    (Pi copied the governed parameters). The workspace file was 13 bytes of
    `c2a-patch-v1` plus a literal `n`, not `c2a-patch-v1` plus LF, so the
    executor correctly refused. Required CI `32288802253` Ubuntu
    `c2a_public_patch_fixture_reaches_production_sink` **ok**; Clippy failed only
    on `clippy::expect_used` in that test. This does not prove WorkspacePatch,
    C2, Gate, release, Profile, B01, or Agent-benefit.

14. **D02 public WorkspacePatch lifecycle — pass.** After reseeding
    `c2a-patch.txt` to the exact bytes `c2a-patch-v1` plus LF (`63 … 31 0a`,
    SHA-256 `cb4ff53fe48499826134116581f605c9ed95cc37cfb3d0e42aac028b87c99c0f`),
    a fresh Task `task://personal/p2-t37-public-patch-reseed` was admitted with
    `native.workspace.patch` and launched via public
    `cognitive pi launch --print --task-ref ...` using
    `apps/kernel-server/tests/p2_t37_public_patch_prompt.txt`. Observation:
    `COMPLETED`, `reconcile_class: closed`, `lease_acquired: 1`, verification
    `passed`/`current`, `acceptance_current: true`, Effect `RECONCILED` /
    `executed`. The workspace file became `c2a-patch-v2` plus LF
    (`63 … 32 0a`, SHA-256
    `2d0cc7d5defec924af77da9ea56bd991f6467fe49a80b80bfc02d3c42d0d95ac`).
    No Pi-native filesystem or shell tool was used. This does not prove C2,
    Gate, release, Profile, B01, or Agent-benefit.
15. **Patch daemon stop — pass.** Public `cognitive daemon stop --runtime-root
    /home/wuz/p2-t37-c2a-patch-20260820` reported `action: stopped`,
    `stale_lock_removed: true`; `127.0.0.1:48446` is no longer listening.

## Implemented boundary

- Public `cognitive pi launch` now selects only the four daemon-governed
  Workspace tools: Read, Search, Write, and Patch.
- The Extension remains I/O-free. Public Write/Patch calls submit only target,
  family, canonical input field, and expected preimage as untrusted candidate
  parameters; authority references, WIA, Effect, lease, lifecycle, and other
  authority-shaped fields are not accepted from Pi.
- The existing daemon candidate admission remains responsible for canonical
  digest recomputation, descriptor/action/target validation, epoch/fencing,
  persistence, dispatch, expected-preimage mutation, independent verification,
  and acceptance. No direct filesystem or SQLite path was added.

## Remaining

- Complete D03: Clippy `expect_used` repair on the production-sink fixture test,
  required CI on the merge HEAD, handbook/docs synchronization if required,
  acceptance mapping, and deterministic task/lease/branch closure.
- Failure-first mutation negatives (malformed base64/preimage, descriptor/digest
  /epoch drift, duplicate/mixed candidates, preimage mismatch) remain covered by
  existing hermetic tests; Ubuntu `c2a_public_patch_fixture_reaches_production_sink`
  **ok** at `40787044`. Windows GNU Rust remains `not-run`.
