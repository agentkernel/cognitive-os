# P1-T09 protected experimental signing workflow handoff

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Lease: `lease/personal/P1-T09/protected-experimental-signing-workflow`
- Branch: `lane/personal-p1-t09-coherent-bundle-delivery`
- Change class: implementation-only; normative surface unchanged
- Development track: `experimental-local-only`

## Delivered

`.github/workflows/personal-experimental-linux-campaign.yml` defines a manual,
least-privilege Ubuntu workflow for an experimental P1-T09 campaign. It:

- has only `contents: read` permission and creates no GitHub Release;
- uses the fixed `personal-linux-experimental-campaign` Environment rather
  than accepting an Environment name as an input;
- accepts only a validated immutable campaign version;
- rebuilds/tests the reviewed Rust and TypeScript sources, then builds the
  daemon, product CLI, installer, campaign builder, and full Pi Extension;
- uses the existing builder to produce and offline-verify the coherent bundle;
- receives the signing seed only as an Environment secret, decodes it directly
  into a private temporary file, checks its length, and removes it in an
  always-running cleanup trap; and
- uploads only the signed bundle and non-secret offline installation facts for
  seven days.

The workflow does not expose the seed in an argument, environment variable,
output, artifact, diagnostic, ordinary repository file, or deployment host.
It does not run bootstrap `install.sh`; experimental deployment must use the
downloaded offline artifact and verified installer in a separate lease.

## Executed checks

| Check | Result |
|---|---|
| Python YAML parse and policy-shape assertions | pass |
| `pnpm run check:consistency` before handoff creation | fail; correctly identified the missing linked handoff |
| `pnpm run check:consistency` after handoff creation | pass |
| Workflow dispatch | not-run; required Environment and signing secret are absent |
| Artifact build, signature, offline verifier, deployment, route runner | not-run; workflow dispatch prerequisite absent |
| B01 / GMVP-LINUX / release / Profile | not-run or non-claim |

## Required administrator action

Before dispatching the workflow from the reviewed protected branch, configure
the repository Environment named exactly:

```text
personal-linux-experimental-campaign
```

It must restrict deployment to the protected branch and require the
repository's designated release/campaign reviewer. In that Environment, create
the secret named exactly:

```text
PERSONAL_EXPERIMENTAL_CAMPAIGN_SIGNING_SEED_BASE64
```

Its value must be an authorized existing Ed25519 seed encoded as standard
base64 and decode to exactly 32 bytes. Enter it through GitHub's protected
secret-management UI or an approved KMS-to-GitHub integration; do not provide
the value in chat, commit it, add it to an argv/environment variable, or copy
it to the experimental host.

After both controls exist, dispatch the workflow with a fresh version of the
form `0.0.0-campaign.YYYYMMDD.N`. The next P1-T09 lease can download the
uploaded artifact and use its non-secret `offline-install-facts.json` with the
verified installer. It must then run the existing redacted route runner against
the installed immutable paths.

## Status and blocker

P1-T09 remains `in-progress`. `blocked_paths`: GitHub Environment and its
protected signing-secret configuration, then the experimental host deployment
paths. `blocked_task_ids`: `P1-T09`. `blocked_gate_ids`: `B01`, `GMVP-LINUX`,
and Profile. Owner: repository administrator / authorized campaign-signing
workflow owner. Next action: configure the protected Environment and secret,
dispatch the workflow, and continue with a deployment-only lease.
