# P1-T09 B01 successor artifact verification blocker

- Date: 2026-08-09
- Classification: `corrective`
- Task: `P1-T09`
- Gate: `B01`
- Branch: `personal/P1-T09-b01-campaign-continuation`
- Draft PR: [#167](https://github.com/agentkernel/cognitive-os/pull/167)
- Lease: `lease/personal/P1-T09/b01-campaign-continuation` (closed)

## Completed non-claim artifact work

The owner retained failed campaign `001` and authorized separately
preregistered successor `B01-clean-linux-first-install-first-conversation-002`.
No successor attempt has started. The protected workflow completed successfully:

| Field | Recorded fact |
|---|---|
| Workflow run | `31284948257` |
| Source revision | GitHub `main@4ea42c0c8f856aa22e2a360bd42005c8dbec400f` |
| Artifact ID/name | `9029551950` / `personal-experimental-linux-campaign-0.0.0-campaign.20260809.1` |
| GitHub artifact digest | `sha256:ceabde055be969f4e81d983e275438019d2fb71e34120e699931e4c85714eaca` |
| Bundle digest from downloaded manifest | `sha256:fb5d006fa25a6545d46b3089e53a8fe08fe9df0d113165c799752154b981abf4` |
| Signature boundary | `p1-t09-experimental` / `p1-t09-experimental-20260730`; independently unverified |

The workflow's reviewed-input validation and signed artifact build passed. This
is not standalone offline signature verification and creates no B01 result.

## Blocker

- `blocked_paths`: `personal-linux-native-01` GitHub HTTPS access and its
  disposable exact-revision verifier worktree
- `blocked_task_ids`: `P1-T09`
- `blocked_gate_ids`: `B01`, `G1`, `GMVP-LINUX`
- owner: native-host network operator and independent verifier
- evidence: non-interactive SSH succeeded, while native `git clone` and
  `git ls-remote` against GitHub HTTPS timed out. No local source was copied to
  the host and no stale or non-Git snapshot was substituted.
- single next action: restore GitHub HTTPS access on `personal-linux-native-01`,
  provision a disposable Git worktree at
  `4ea42c0c8f856aa22e2a360bd42005c8dbec400f`, run the standalone
  `linux_bundle_verifier` over the downloaded artifact, and record its outcome
  before any B01 guest reset.

`B01-Desktop-Linux-002` was not started, reset, installed on, or otherwise
changed. The ignored `/artifacts/` directory was not used as a successor input.
The local workflow-artifact verification directory is disposable staging outside
the repository and is not evidence or a replacement for native verification.

## Recovery workflow non-claim

Commit `4df9235` adds a GitHub-hosted verifier workflow as a recovery aid. It
cannot yet be dispatched because GitHub registers `workflow_dispatch` workflows
only from the default branch; merging it would be premature while P1-T09 is
blocked and PR #167 remains Draft. It does not replace the exact-revision native
verifier route.
