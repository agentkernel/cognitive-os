# P2-T37 C2a public mutation path running report

Task: `P2-T37`
Branch: `personal/P2-T37-c2a-public-mutation-path`
Lease: `lease/personal/P2-T37/c2a-public-mutation-path`
Scope: `P2-T37/D01` and the beginning of `P2-T37/D02` only

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

- Complete D02 with exact pushed native-Linux public WorkspaceWrite and
  WorkspacePatch lifecycle validation as separate fresh Tasks.
- Run the daemon/API mutation negatives for malformed base64, preimage,
  descriptor/digest/epoch drift, duplicate or mixed candidates, preimage
  mismatch, and no mutation before daemon admission.
- Complete D03 with supported CI, handbook/docs synchronization if required,
  acceptance mapping, and deterministic task/lease/branch closure.
