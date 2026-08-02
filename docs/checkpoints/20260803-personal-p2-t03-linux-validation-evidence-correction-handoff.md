# P2-T03 Linux validation evidence correction handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/linux-validation-evidence-correction` (closed)
- Branch: `main`
- Change class: corrective
- Task status: `in-progress`
- Implementation evidence: unchanged
- Normative surface: unchanged

## Correction

The prior validation closure incorrectly stated that the qualified Linux host
had cloned exact revision `a74ad74856b4cef6d05668acf42832ea18351b8a` and run
two focused tests. The authoritative terminal record instead shows SSH host-key
verification failure (exit 255) before the remote shell began.

Accordingly, the affected plan, current snapshot, lease ledger, and validation
handoff now state that no clone, exact-revision checkout, test, or temporary
worktree cleanup executed. This correction does not change P2-T03 task status,
existing prior `tested-local` evidence, Gates, release, or Profile claims.

## Remaining blocker and next action

- `blocked_paths`: approved SSH host-key trust configuration for the qualified
  Linux host.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12 and GMVP-LINUX.
- owner: product owner for host-key trust configuration.
- next action: after approved trust is available, run a new non-interactive,
  no-secret exact-revision Git-worktree validation; do not substitute a stale
  source snapshot.
