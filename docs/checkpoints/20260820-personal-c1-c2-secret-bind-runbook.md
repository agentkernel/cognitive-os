# P9-T08 package 11 — Secret / doctor bind runbook

- Status: **runbook**. Live doctor on a new EVAL root is the package-15 start
  gate, not a P9-T08 sample.
- Claim ceiling: `hypothesis` / non-claim.

## Allowed bind (only)

On a **new isolated runtime root**, after an opaque SecretRef already exists
in the approved Linux Secret Service:

```text
cognitive init --runtime-root <new-root> --reuse-existing-secret-binding
```

Required redacted facts:

- `action: bound_existing_secret_ref`
- `secret_material_written: false`
- `secret_ref_redacted: true`

Then redacted doctor (same root):

```text
cognitive doctor --runtime-root <new-root>
```

Required redacted facts:

- Provider ready
- `secret_ref_resolves: true`
- selected-model digest match
- `first_conversation_ready: true`

`first_conversation_ready` is conversation-shell readiness, not C1/C2 Task
completion.

## Forbidden

- File-copy or symlink of `provider.json` between roots
- `secret-tool search` or `secret-tool lookup`
- Printing secret material
- Key on argv, environment, ordinary config, logs, evidence, Git, or chat
- Recapture from a keyfile
- Reuse of SecretStore items `/12`–`/19`

## If no SecretRef is present

Record `not-run` with recovery: owner graphical hidden-input import into the
**new** EVAL SecretStore item (planned `/20`), using product stdin
(`cognitive init --api-key-file -`). Do not invent a product TODO. Do not
import onto a closed EVAL item.

## P-arm broker credential

Arm `P` loads the campaign probe or campaign item through
`tools/personal/c1-c2-paired/linux-secret-service.mjs` (D-Bus `GetSecret`).
Pi sees only `campaign-broker-nonsecret-token`. Cleanup stops the broker
process; it does not delete the owner Provider key.

## Non-claims

A doctor-ready fact is not B0, not paired performance, and not Agent-benefit.
