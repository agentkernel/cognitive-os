# P1-T09 B01 baseline SSH provisioning closure

- Date: 2026-08-08
- Classification: `implementation-only`
- Task: `P1-T09`
- Lease: `lease/personal/P1-T09/b01-baseline-ssh-provisioning`
- Branch: `personal/P1-T09-b01-network-recovery`

## Provisioning outcome

With owner authorization, the manually started `B01-Desktop-Linux-002` guest
was shut down gracefully. Its offline root disk was mounted through qemu-nbd
while the guest was shut off. The existing `hal9001` authorized-keys entry was
retained and a dedicated non-secret Ed25519 public key was added with owner
UID/GID `1000:1000` and mode `0600`. No private key, password, Provider
credential, or SecretStore material was copied to the guest or evidence.

The repaired guest was booted and a bounded SSH check through the authorized
host path succeeded. The guest was then shut off and the old
`b01-platform-qualified-baseline` snapshot was replaced with a snapshot of the
repaired state under the same registered name. The replacement snapshot is
shut off and is now the reset point for a future fresh attempt 4.

This provisioning and snapshot replacement is baseline maintenance, not a B01
attempt. The fixed-N ledger remains 3 of 20: 1 success and 2 readiness
failures.

## Validation

- owner-authorized graceful guest shutdown: pass;
- offline public-key provisioning: pass;
- guest SSH check after provisioning: pass;
- guest shutdown before snapshot replacement: pass;
- replacement snapshot `b01-platform-qualified-baseline`: pass, shutoff;
- B01 attempt 4, artifact, Pi, Provider opt-in, first response, cleanup, and
  independent verifier disposition: not run.

## Next action and non-claims

Close this provisioning lease. Before attempting the formal campaign again,
claim a fresh P1-T09 attempt-execution lease and obtain owner confirmation for
the complete attempt 4, including the designated desktop operator's
hidden-input Provider opt-in and independent verifier review. This record does
not change B01 Gate status or create release, GMVP-LINUX, or Profile evidence.
