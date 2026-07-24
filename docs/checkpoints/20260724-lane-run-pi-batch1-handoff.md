# Lane-RUN Pi Batch 1 handoff

- Date: 2026-07-24
- Base: `3c22a4243254d12a36916a8b4624c07b1ffa6ab5` (`origin/main`, CI success)
- Branch: `lane/run-pi-batch1`
- Merged: PR #83, `937e7271af9edda73dad3da1a6841678fc18b1a2`
- Scope: P4 fail-closed pre-launch admission only

## Completed

- Added `pi_launcher` with `admit_pi_launch` and opaque `PiLaunchPermit`.
- Windows-native returns `WindowsNativeUnsupported`; WSL2 returns
  `Wsl2SeparatePlatform`; neither can gain a Linux-native claim.
- Linux admission requires the actual host to be Linux plus exact policy,
  sandbox-adapter and compatibility digests, a healthy registered adapter, an
  HTTPS model egress proxy and the exact registered DeepSeek endpoint.
- The permit neither grants authority/capability nor creates an Effect or Task
  completion. No concrete permissive adapter or Pi process launch is present.

## Test-first and verification evidence

1. The first test attempt on Windows was blocked before project compilation by
   MinGW linker `ld` exit 121. This is not a pass.
2. WSL2 Ubuntu guest (explicitly `windows_wsl2_linux_guest`, not Linux-native)
   ran `cargo test -p cognitive-runtime --offline`: 45 unit + 2 existing
   integration + 5 new launcher tests = **52 passed / 0 failed**.
3. The same guest ran `cargo clippy -p cognitive-runtime --all-targets -- -D warnings`: pass.
4. Windows `cargo fmt --check` and `git diff --check`: pass.

## Status boundary

- 规范已登记：existing agent/sandbox requirements and error code
  `AGENT_ADAPTER_BYPASS_DETECTED`; no new REQ/schema/vector/error code.
- 实现已提供：fail-closed pre-launch admission only.
- 测试已执行：WSL2 guest diagnostics only; Windows local test execution is blocked.
- Profile 已符合：**0**. There is no actual Linux-native sandbox/proxy behavior
  evidence, no F-017 expanded claim, and no governed Pi execution claim.

## Next entry

Provision a real Linux-native reference host and a registered sandbox adapter
with model egress proxy. Then execute per-channel allow/undeclared-deny/fault
behavior tests with reproducible environment and digest evidence before any
Pi process launch or F-017 claim update. Official provenance remains blocked
without a real trusted attestation.
