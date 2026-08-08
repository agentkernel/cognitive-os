# P1-T09 B01 exact supported Node.js runtime maintenance

- Date: 2026-08-08
- Task: P1-T09 install-to-first-conversation route
- Campaign: `B01-clean-linux-first-install-first-conversation-001`
- Lease: `lease/personal/P1-T09/b01-node-runtime-maintenance`
- Classification: baseline maintenance after counted Attempt 8; not a retry

## Reason

Attempt 8 verified that the installed CognitiveOS service remained active, but
the preregistered exact Pi package `@earendil-works/pi-coding-agent@0.81.1`
requires Node.js `>=22.19.0`. The previous baseline's Node.js `v18.19.1` did
not satisfy that engine requirement.

## Completion

The owner used an interactive SSH sudo prompt to install the runtime from a
locally staged, SHA-256-verified Node.js `v22.23.2` Linux x64 archive. The
guest reported Node.js `v22.23.2` and npm `10.9.8`. A non-secret npm metadata
probe against `https://registry.npmmirror.com` returned the exact Pi `0.81.1`,
its `>=22.19.0` engine requirement, and the registered SRI value. No Pi or
CognitiveOS state was present. The temporary runtime staging directory was
removed, the guest was shut off, and the registered
`b01-platform-qualified-baseline` snapshot was replaced successfully.

No B01 attempt is created by this maintenance. The ledger remains 8 of 20:
1 success, 7 failures, and zero critical safety failures observed.
