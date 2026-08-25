---
doc_id: dev.management-plane
locale: en
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: crates/cognitive-management/src/plane.rs
    symbols: ["ManagementPlane", "reconcile"]
  - path: crates/cognitive-management/src/session.rs
    symbols: ["PrivilegedManagementSession"]
  - path: crates/cognitive-management/src/approval.rs
    symbols: ["ApprovalGate"]
  - path: crates/cognitive-management/src/audit.rs
    symbols: ["FileManagementAuditLog", "ResultReleaseGate"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
  - path: apps/admin-cli/src/main.rs
tests:
  - crates/cognitive-management/tests/m5_session_approval.rs
  - apps/admin-cli/tests/m5_deterministic_fallback.rs
  - apps/admin-cli/tests/p2_t27_pi_lifecycle.rs
fingerprint: "sha256:7f3c8b16d4b03bce656e63d84fb00f38d28e37a7556cbdb9db6c5b535b97f156"
non_claims:
  - R0/R2/R3 approval flows and a durable governance-ledger production path are not implemented; only what is listed here exists.
---

# Management plane

The deterministic fallback: when models, Pi, or the conversational path are
unavailable, `admin-cli` + `cognitive-management` still inspect, stop, revoke,
and reconcile against the same authority store — no model SDK anywhere on this
path.

## Sessions and risk tiers

Every verb demands a `PrivilegedManagementSession` JSON document: schema-valid,
purpose-bound, risk-tiered (R0–R3), lifecycle-managed (issue/renew/revoke,
absolute + idle expiry). `inspect` needs R1+, mutations need R2+, plus
per-action approval records for R1-class proposals
(`ApprovalGate`: independent structured confirmation, fatigue
aggregation, no blanket approvals). R0/R2/R3 structured flows beyond tier checks
are not implemented — hence `partial`.

## Verbs

- `inspect_with_audit`: privileged reads released only through
  `ResultReleaseGate` after a canonical-JSON-lines audit record is durably
  appended (`FileManagementAuditLog` enforces sequence/epoch/hash-chain shape).
- `stop`: fences the writer epoch, cancels scheduler work, classifies pending
  Effects (conservatively counting `RECONCILED/VERIFIED/VERIFY_FAILED` as
  pending), and reports what remains.
- `revoke`: appends capability revocations and advances the revocation epoch
  that context/authorization currency checks consume.
- `reconcile`: drives the kernel recovery sequence; with no executor configured,
  still-unknown outcomes quarantine (fail-safe) rather than resolve.

## Agent lifecycle verbs

`admin-cli install/register/activate/activate-root/rollback/agent-pause/agent-resume/agent-stop/
agent-recover/agent-health/uninstall` call the runtime lifecycle described in
[Agent and Pi lifecycle](./agent-and-pi-lifecycle.md), all session-gated.

## Task admission

`KernelTaskApplicationService::admit` remains the deterministic
digest/authority/epoch gate. Its production mint now returns success only after
the same authority transaction has published the TaskContract plus its
contract-named `START` Loop, hard Budget, and current-epoch runnable scheduler
row. This is scheduler bootstrap only: it creates no worker Intent/Effect,
performs no Tool I/O, and cannot complete a Task.

Honest gaps: the usage text omits `--package-id` for official installs; the
governance ledger (`revocation_epoch`/`capability_set_version` persistence) has a
file implementation consumed in tests, with daemon-side wiring still partial.
