# P1-T09 B01 network recovery blocker

- Date: 2026-08-08
- Classification: `implementation-only`
- Task: `P1-T09`
- Gate: `B01-clean-linux-first-install-first-conversation-001`
- Branch: `personal/P1-T09-b01-network-recovery`
- Revision: `1ea2de944cc684619bab3eb7e7403f7bdd30fc56`
- Draft PR: [#165](https://github.com/agentkernel/cognitive-os/pull/165)
- Lease: `lease/personal/P1-T09/b01-network-recovery`

## Read-only diagnosis

The authorized system-libvirt inspection of the shut-off
`B01-Desktop-Linux-002` guest confirmed that the registered domain and its
`b01-platform-qualified-baseline` snapshot still exist. The guest interface is
attached to the active `default` NAT network with the preregistered MAC. That
network is configured as `192.168.123.0/24`, so the attempt-2 target address
`192.168.123.160` is in the configured subnet. The DHCP lease table is empty,
which is expected while the guest is shut off.

The inspection was read-only: it did not start, reset, alter, snapshot, mount,
install on, or otherwise mutate the guest or campaign baseline. It therefore
does not start attempt 3 and does not change the fixed-N ledger (2 of 20,
1 success and 1 readiness failure).

## Blocker and recovery action

The host-level network configuration does not explain attempt 2. Establishing
whether the baseline guest now accepts SSH requires a fresh baseline reset and
guest start, which crosses the clean-reset checkpoint and is attempt 3. An
attempt that reaches readiness also requires the designated desktop operator's
hidden-input Provider credential opt-in before it can be validly completed;
the credential is intentionally unavailable to this agent.

- `blocked_paths`: `B01-Desktop-Linux-002`, its registered baseline snapshot,
  and the operator-controlled hidden-input SecretStore opt-in.
- `blocked_task_ids`: `P1-T09`.
- `blocked_gate_ids`: `B01`, `G1`, `GMVP-LINUX`.
- owner: designated B01 desktop operator / independent verifier.
- next action: schedule a fresh, fully staffed attempt 3. First restore
  `b01-platform-qualified-baseline`, start the guest, and record its bounded
  readiness result in the fixed-N ledger. If ready, the designated operator
  completes the approved hidden-input opt-in and the independent verifier
  reviews the redacted result. Do not run a readiness-only probe or treat it
  as a retry.

## Validation

- `pnpm run check:consistency` -- pass before the diagnostic record.
- `git diff --check` -- pass before the diagnostic record.
- Authorized read-only system-libvirt domain, snapshot, interface, network,
  DHCP, and XML inspection -- pass.
- Guest start/reset, SSH readiness after a fresh reset, artifact installation,
  Pi execution, Provider opt-in, first response, cleanup, and independent
  verifier disposition -- not run.

## Non-claims

This record does not add a B01 attempt, establish guest SSH readiness, alter
the B01 result, or support B01, G1, release, GMVP-LINUX, or Profile claims.
