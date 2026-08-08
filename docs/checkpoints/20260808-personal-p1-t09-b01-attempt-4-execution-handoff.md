# P1-T09 B01 Attempt 4 artifact-availability failure

- Date: 2026-08-08
- Task: P1-T09 install-to-first-conversation route
- Campaign: `B01-clean-linux-first-install-first-conversation-001`
- Lease: `lease/personal/P1-T09/b01-attempt-4-execution`
- Classification: implementation-only attempt record

## Result

With owner authorization, Attempt 4 restored the repaired
`b01-platform-qualified-baseline` and started `B01-Desktop-Linux-002` through
the authorized system-libvirt control path. The bounded local-key ProxyJump SSH
readiness check for `hal9001@192.168.123.160` passed.

Before installation, the runner queried the preregistered workflow run
`30687541828`. GitHub reports its required
`personal-experimental-linux-campaign-0.0.0-campaign.20260801.1` artifact as
expired. The only candidate retained on the authorized host was an older,
non-registered campaign artifact, so it was not substituted. This is a
counted Attempt 4 failure under the fixed-N contract, not a retry.

No artifact was copied or installed. No Pi state, product service, Provider
configuration or request, credential, prompt, expected marker, response,
authority side effect, Task, Effect, or Verification was created. The exact
baseline was restored and the domain was confirmed `shut off`.

## Next action

Produce and independently verify a newly available reviewed-main campaign
artifact, then preregister its immutable identity, signature, trusted-key
metadata, and Pi pin before requesting authorization for a fresh counted
Attempt 5. The owner-specified DeepSeek provider and model must be confirmed
against that release's non-secret supported configuration surface before
Operator A receives a hidden-input prompt. No credential must be supplied in
chat, command arguments, ordinary files, logs, or evidence.

## Non-claims

This result does not pass B01, GMVP-LINUX, release, or Profile. The campaign
ledger is 4 of 20 attempts: 1 success, 3 failures, and no critical safety
failure observed.
