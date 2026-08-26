---
doc_id: user.rc-and-support
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: tools/src/personal-rc-gate.mjs
    symbols: ["buildPersonalRcDeclarationReport"]
  - path: docs/plan/PERSONAL-SUPPORT-MATRIX.md
  - path: personal/deploy/linux/install.sh
  - path: personal/crates/cognitive-store/src/personal_backup.rs
    symbols: ["plan_personal_lifecycle"]
  - path: personal/apps/admin-cli/src/personal_cli/init.rs
    symbols: ["run_init"]
  - path: personal/apps/admin-cli/src/personal_cli/daemon.rs
tests:
  - tools/test/personal-rc-gate.test.mjs
fingerprint: "sha256:82c24ea70174d3b3d053f0a1abfc51f9fde44805f7fac8330b79ffc5a6a7e416"
non_claims:
  - This page is an operator map, not a Gate disposition, Profile result, or production GitHub Release.
  - There is no public `cognitive uninstall` or `cognitive update` verb; do not invent one.
  - Multi-Agent, Web UI, dsh Path B, B10/MCP, and Windows install parity are not in the Linux RC claim.
---

# Linux RC operator map

`partial`: every step below is implemented in code, but there is **no public
production release artifact**. Bundles so far are experimentally signed campaign
builds. Current Gate and task facts live in
[`docs/plan/PROGRESS.md`](../../../../docs/plan/PROGRESS.md) and are not copied
here. Platform and claim policy live in
[`PERSONAL-SUPPORT-MATRIX.md`](../../../../docs/plan/PERSONAL-SUPPORT-MATRIX.md).

The Personal Linux RC declaration is a **digest-bound composition** of existing
evidence. It does not run a new clean-VM campaign and does not mutate an isolated
B01 guest.

## 1. Install

Follow [Install and reach the first conversation](install-and-first-conversation.md)
§1: run the inspected `install.sh` from a signed bundle. The installer verifies
the bundle, stages immutable bytes, installs `cognitiveos-personal.service` on
loopback `127.0.0.1:48181`, and only then flips `active-version`. Failure
compensates: the previous version, unit, and pointer are restored.

## 2. Init

Follow the same guide §2 and [Getting started](getting-started.md):

```text
cognitive init --provider <id> --base-url <https-url> --api-key-file -
```

The Provider key enters the approved Secret Store only. There is no plaintext
fallback.

## 3. Provider

After the daemon is running, named accounts, keys, bindings, and usage live on
[Provider Control Plane](provider-control-plane.md) (CLI and same-origin
`GET /ui/`). Keys never enter SQLite, argv, or browser storage. The Control Plane
Web UI is a non-blocking surface and is **not** part of the Linux RC product
claim.

## 4. Pi

The only Linux 1.0 RC product-qualified Agent is pinned Pi plus its per-Agent
sidecar. Use [The Pi shell](pi-shell.md). DeepSeek Harness (dsh) Path B is
implemented later and does not inherit Pi evidence.

## 5. Task

[Tasks and execution](tasks-and-execution.md) is the public Task path. A first
conversation or a Provider response is not Task completion. Independent
verification is required before a Task may complete.

## 6. Recovery

[Operations and recovery](operations-and-recovery.md) covers status/doctor,
crash/unknown-outcome recovery, backup/restore, and database safety. Backup never
copies secrets or `authority.sqlite`.

## 7. Update and rollback

There is no public `cognitive update`. Update means re-running the inspected
signed `install.sh` from a **newer** bundle on the same host. The previous
version stays on disk until the new activation succeeds. Any failure restores
the previous version, unit, and `active-version` pointer and issues no success
receipt. See [Installer and service](../developer/installer-and-service.md).

Authority-path planning (`plan_personal_lifecycle` Update/Rollback) records the
intent; it does not replace the installer compensation.

## 8. Uninstall

There is no public `cognitive uninstall`. The supported operator sequence is:

```text
cognitive daemon stop
systemctl --user disable --now cognitiveos-personal.service
```

That stops and disables the user unit. It does not delete Secret Store items,
`authority.sqlite`, or XDG data. `plan_personal_lifecycle` Uninstall refuses a
Secret target and refuses Data unless deletion is explicitly confirmed; committing
that plan still does not delete host files. Managed Pi uninstall remains the
`admin-cli` lifecycle on [Operations and recovery](operations-and-recovery.md).

## What this RC does not include

- Multi-Agent / B11 (recorded disabled-NO-GO for this RC; Phase 6 stays default-disabled)
- B10 / MCP / dynamic Tool marketplace
- Web UI / Control Plane in the Linux RC product claim
- Windows installer/service / B01-W
- CognitiveOS Core Profile `implemented`
- A production GitHub Release or production signing ceremony
