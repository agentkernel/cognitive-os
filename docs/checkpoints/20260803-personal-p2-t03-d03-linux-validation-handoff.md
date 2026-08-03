# P2-T03 D03 exact Linux validation handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Slice: `P2-T03/D03`
- Revision: `7489f4c`
- Change class: focused regression coverage and supported validation
- Task status: `in-progress`
- Slice status: `done`
- Normative surface: unchanged

## Delivered evidence

The daemon-private resolver follows the immutable `TaskBinding` reverse index
to exactly one durable Intent and then its governed Effect. The focused
regression coverage proves that no binding, multiple bindings, and a row whose
stored binding disagrees with the reverse-index query each fail closed. Existing
classification coverage retains the unknown-state fail-closed behavior.

No caller-supplied Effect state, process receipt, scheduler release, Task
acceptance, verification, capability minting, or client authority was added.

## Exact-revision Linux validation

An archive of immutable revision `7489f4c` was transferred to a disposable
Linux worktree at `wuz@192.168.1.2`; no Provider, secret, Pi installation,
service-manager action, B01 guest, or external operation was used.

- `cargo test -p kernel-server scheduler_authority --locked`: passed, 11/11.
- `cargo test -p cognitive-store --test m5_intent_chain --locked`: passed,
  6/6.
- `cargo fmt --all`: passed locally.
- `git diff --check`: passed locally before commit.

## Remaining work and non-claims

`P2-T03/D04` remains blocked pending its immutable Linux runtime/kernel
closure-and-release validation and required CI. D05 remains blocked behind
D04. B02, B04, B05, B12, release, GMVP-LINUX, and Profile remain not-run or
incomplete.
