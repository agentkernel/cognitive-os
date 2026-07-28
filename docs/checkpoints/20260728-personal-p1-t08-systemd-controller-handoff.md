# Personal P1-T08 Fake-Systemctl Controller Handoff

**Date:** 2026-07-28
**Base:** `main@bc77beb`
**Branch:** `lane/personal-p1-t08-systemd-controller`
**Implementation commit:** pending

## Delivered boundary

ADR-0033 fixes the controller-fixture decision left open by ADR-0032. The
fixture constructor receives only an isolated deployment root, private unit
root, fake manager binary, and the fixed active loopback address. Candidate and
active unit identities, executable layouts, runtime roots, ports, and manager
argument prefixes remain product-owned.

Before candidate start, the controller publishes the fixed candidate unit and
executes fixed `systemctl --user --no-ask-password --no-pager daemon-reload`.
It then runs the fixed candidate start action. A Unix fake-systemctl harness
records these exact action boundaries and verifies that no canonical active
unit is published during candidate work.

## Failure-first evidence

The new fixture-focused test required a constructor with an isolated unit root
and fake binary plus publication/reload before start. The earlier controller
had neither fixture injection nor daemon-reload/publication behavior, so the
test could not compile before implementation. The red state was not committed.

## Executed verification

Executed in `windows_wsl2_linux_guest`:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-controller \
  /root/.cargo/bin/cargo test -p cognitive-runtime \
  --test linux_bundle_service_lifecycle --locked --offline
# 10 passed; 0 failed
```

This is implementation-fixture evidence only. It is not Linux-native systemd,
B01, Gate, Profile, containment, RC, or release evidence.

## Remaining scope

P1-T08 remains `in-progress` / `experimental-local-only`; P1-T09 remains
`not-started`. Follow-up work must extend the fixture to the complete
pointer/unit/service compensation, timeout/output-cap, unsafe-parent, and
redaction fault-injection matrix before any separately authorized native
systemd campaign is considered.
