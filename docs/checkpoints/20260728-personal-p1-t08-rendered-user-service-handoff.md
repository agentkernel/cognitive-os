# Personal P1-T08 Rendered User-Service Foundation Handoff

**Date:** 2026-07-28
**Base:** `main@f3eec66` (local continuation baseline)
**Branch:** `lane/personal-p1-t08-rendered-user-service`
**Implementation commit:** `4761905`
**Pull request:** [#114](https://github.com/agentkernel/cognitive-os/pull/114)
**Merge commit:** `b151b5403de6a373d5c4af59af73f72d10e1245e`

## Delivered boundary

ADR-0032 records the narrow candidate-to-active model: two product-owned unit
names, separate loopback ports, `staged/<version>` candidate executable path,
`versions/<version>` canonical executable path, and candidate stop before
pointer activation/canonical start.

`cognitive-runtime` renders fixed unit contents without accepting a manifest,
bundle, keyring, health URL, arbitrary command, arbitrary unit name, or
arbitrary bind address. The renderer rejects unsafe version and path input.
Fixture unit publication creates a private temporary file, syncs it, assigns
explicit private permissions where supported, and atomically renames it to a
fixed candidate/active unit filename. The generic lease-held transaction now
stops the candidate before active pointer promotion, then starts and confirms
the canonical service after promotion. Compensation continues to return no
receipt when incomplete.

The checked-in source templates remain unrendered. No production user-systemd
directory selection, daemon-reload fixture, release renderer, GitHub Release,
uninstall, or Linux-native campaign is provided by this atomic foundation.

## Failure-first evidence

The first new focused test imported the missing rendered-unit API and asserted
the required candidate/active fixed paths and ports; before implementation it
failed to compile because `PersonalUserServiceUnitKind` and
`render_personal_user_service_unit` did not exist. A second focused test
required candidate stop before canonical active start/confirmation; the former
transaction skipped those actions. Neither red state was committed.

## Executed verification

Executed in `windows_wsl2_linux_guest`:

```text
wsl bash -lc 'cd /mnt/d/agent-kernel; /root/.cargo/bin/cargo test \
  -p cognitive-runtime --test linux_bundle_service_lifecycle --locked --offline'
# 9 passed; 0 failed

wsl bash -lc 'cd /mnt/d/agent-kernel; /root/.cargo/bin/cargo fmt \
  --all -- --check'
# passed
```

This is local implementation-fixture evidence only. Ubuntu/Windows-MSVC CI is
not Linux-native systemd evidence. After a Windows path-separator correction
(`0a90033`), both supported push and pull-request matrices passed on Ubuntu
and Windows/MSVC: push run
[30379506413](https://github.com/agentkernel/cognitive-os/actions/runs/30379506413)
and pull-request run
[30379508772](https://github.com/agentkernel/cognitive-os/actions/runs/30379508772).
Linux-native systemd is `not-run`; WSL2 and supported CI must not be reported
as Linux-native evidence.

## Completed and remaining scope

- **specified:** ADR-0032 fixes the implementation-local two-unit state
  machine and non-claim boundary.
- **implementation provided:** fixed rendering, fixture atomic publication,
  candidate-to-active ordering, and focused fake-controller assertions.
- **tests executed:** 9 focused service lifecycle tests and formatting in WSL2.
- **Profile conformant:** not claimed.

P1-T08 remains `in-progress`; P1-T09 remains `not-started`. Remaining P1-T08
work includes production user-systemd root hardening, daemon-reload/controller
fixture coverage, complete failure-injection and redaction matrix, a separate
Linux-native fixture campaign decision, uninstall/user-data retention,
production signing/release material, and every B01/Gate/Profile/release claim.

## Next safe atomic action

Complete a dedicated fake-systemctl controller batch that owns a fixed private
unit root, daemon-reload/start/stop/restart action sequence, timeout/output
caps, and pointer/unit/service compensation assertions. Do not start a real
Linux-native systemd campaign until that batch and its focused tests are green.
