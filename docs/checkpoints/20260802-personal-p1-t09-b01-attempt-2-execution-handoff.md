# P1-T09 B01 attempt 2 execution handoff

- Date: 2026-08-02
- Task: P1-T09 install-to-first-conversation route
- Gate: `B01-clean-linux-first-install-first-conversation-001`
- Change class: implementation-only (campaign execution record)
- Closed lease: `lease/personal/P1-T09/b01-attempt-2-execution`

## Outcome

The product owner confirmed the exact preregistered system-libvirt domain and
reset snapshot. The authorized no-secret preflight confirmed:

- domain: `B01-Desktop-Linux-002`;
- snapshot: `b01-platform-qualified-baseline`;
- guest address: `192.168.123.160`.

Attempt 2 began when the exact snapshot was reverted and the domain started.
Two bounded SSH readiness probes to `hal9001@192.168.123.160` timed out.
Because the clean-reset checkpoint had been crossed, this is attempt 2 and is
recorded as a failure in the immutable fixed-N ledger. No artifact, Pi,
installer, service, Provider, credential, prompt, route runner, Task, Effect,
or Verification operation occurred.

Cleanup reverted the exact snapshot again and confirmed the domain is `shut
off`. The failure is not a critical safety failure; it is a guest-network
readiness blocker.

## Checks

| Check | Result |
|---|---|
| system-libvirt domain/snapshot/address preflight | pass |
| snapshot revert and domain state after start | pass |
| bounded guest SSH readiness | fail: connection timed out |
| cleanup snapshot revert and domain state | pass: `shut off` |
| `git diff --check` | pass |
| `pnpm run check:consistency` | pass |

## Next action

Before any separately leased attempt 3, diagnose why the pristine baseline
guest does not accept SSH at its preregistered address. Keep the guest shut off
and do not alter the baseline. If remote diagnostics are authorized, use only
read-only system-libvirt inspection first; do not handle Provider credentials.
