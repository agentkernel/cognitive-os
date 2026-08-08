# P1-T09 B01 baseline Node.js/npm provisioning

- Date: 2026-08-08
- Task: P1-T09 install-to-first-conversation route
- Campaign: `B01-clean-linux-first-install-first-conversation-001`
- Lease: `lease/personal/P1-T09/b01-baseline-node-provisioning`
- Classification: baseline maintenance; not a B01 attempt

## Scope

The owner authorized provisioning only the Node.js/npm runtime required by the
preregistered exact Pi installation. This maintenance must not install Pi,
CognitiveOS, or any Provider; it must not enter, copy, or inspect credentials.

## Procedure

1. Start the registered B01 guest from the shutoff baseline.
2. Install the distribution-provided `nodejs` and `npm` packages using the
   authorized minimal package-manager path.
3. Verify only non-secret runtime versions and that no Pi state was created.
4. Shut the guest off and replace `b01-platform-qualified-baseline` with the
   repaired state under the same registered snapshot name.

## Exit criteria

- `node --version`: pass;
- `npm --version`: pass;
- `~/.pi` absent: pass;
- CognitiveOS product state absent: pass;
- guest shut off before snapshot replacement: required;
- replacement baseline snapshot: required.

## Completion

On 2026-08-08, the owner completed the approved interactive SSH package
installation. Non-secret verification passed with `node v18.19.1` and `npm
9.2.0`; `/home/hal9001/.pi` and `/home/hal9001/.local/share/cognitiveos` were
absent. The guest was then confirmed `shut off`, the prior
`b01-platform-qualified-baseline` snapshot was removed, and a replacement
snapshot with the same registered name was created successfully. The
replacement snapshot is internal, current, metadata-bearing, and has no
children or descendants.

No B01 attempt is created by this maintenance. The ledger remains 7 of 20:
1 success, 6 failures, and zero critical safety failures observed.
