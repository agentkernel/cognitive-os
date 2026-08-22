# P8-T12 Resource Manager — task handoff

- Task: `P8-T12` (`done`)
- Slice: `P8-T12/D04`
- Branch: `personal/P8-T12-resource-manager`
- Product evidence revision: `1adbdd13b517f50e9793c78e80429006677536d0`
- Draft/ready PR: [#258](https://github.com/agentkernel/cognitive-os/pull/258)
- Lease: closed `lease/personal/P8-T12/resource-manager`
- Change class: `implementation-only`; `normative surface unchanged`
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Implemented

Management Resource Manager envelope over existing six-family stores:
`list`/`inspect`/`bind`/`unbind`/`enable`/`disable`/`revoke`. Generic
`create`/`install`/`execute`/`complete` and the same paths on the task channel
fail closed. Watch stays on `GET /resource/v1/watch`. `cognitive resource`
exposes the same verbs. No public generic Resource DTO.

## Validation

- Local Windows: `cargo fmt --all -- --check`, handbook checker/generator,
  consistency, docs-sync **pass**. GNU Rust linking `not-run`.
- Exact-revision `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, `hal9000`, rustc
  1.97.1) at `1adbdd13`: `p8_t12_resource_manager` **3/3**, admin-cli parse
  **1/1**, Clippy `-D warnings` for `kernel-server` and `admin-cli`, fmt
  **pass**. Not B01.
- Required CI `32561124182` at `1adbdd13` **pass** Ubuntu, Windows, and
  `required-ci`.

Running report: [P8-T12 report](./20260822-personal-p8-t12-resource-manager-report.md).

## Remaining

Ready/merge PR [#258](https://github.com/agentkernel/cognitive-os/pull/258) after
docs-head required CI, then delete the task branch and fast-forward `main`.
Owner asked only for this Resource Manager delivery. Do not auto-claim P6-T01..T04,
P7-T05, P7-T06, or blocked P7-T07.

Untracked user handbook pages (`getting-started.md`, `system-overview.md`) were
left untouched.
