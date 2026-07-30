# P1-T09 coherent bundle delivery handoff

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Closed lease: `lease/personal/P1-T09/coherent-bundle-delivery`
- Branch: `lane/personal-p1-t09-coherent-bundle-delivery`
- Development track: `experimental-local-only`

## Delivered implementation

The immutable Linux bundle payload now contains one coherent product layout:

```text
bin/kernel-server
bin/cognitive
extensions/pi-cognitiveos/dist/**
```

The campaign builder requires both product executables and a complete Extension
distribution. It rejects missing `dist/index.js`, unsafe paths, links, unsafe
file kinds, oversized payloads, and executable Extension files. The installer
extractor rejects incomplete or unsafe archives before service mutation, and
version comparison covers all regular product-payload files rather than only
the daemon executable.

## Executed evidence

- Supported CI run `30537927521` passed its Ubuntu and Windows matrices,
  including TypeScript/Rust builds and tests, Clippy, formatting, consistency,
  conformance, and code-generation-drift checks.
- On `personal-linux-native-01`, a separate non-secret source tree built:
  `kernel-server`, `cognitive`, `linux_bundle_installer`,
  `linux_bundle_campaign_builder`, and
  `packages/pi-cognitiveos/dist/index.js`.
- The host's direct GitHub clone timed out under its low-throughput policy.
  The isolated source tree was instead transferred from the verified local
  branch with `git archive`; it contains no repository metadata and no
  Provider/user secret material.

## Bounded blocker and non-claims

No authorized protected campaign-signing-material workflow was available to
this slice. The signing seed was not requested, inspected, copied, generated,
exported, or passed through an environment variable or command line.
Consequently no campaign archive was signed, no installer bootstrap was
rendered or executed, and no deployment changed the active service.

The existing host remains without a deployed product CLI and product Extension
entry. Therefore `cognitive pi configure`, `cognitive doctor`,
`cognitive pi launch`, direct Pi first-response testing, and a post-deployment
native Secret Service route smoke remain `not-run` for this route.

P1-T09 remains `in-progress`. B01, GMVP-LINUX, release, containment, and
Profile conformance remain `not-run` or non-claim. No Provider configuration,
SecretRef, bootstrap secret, SQLite path, selected-model digest, token, key,
or model response is recorded here.

## Next owner action

Claim a separate deployment-only P1-T09 lease after an authorized protected
campaign-signing-material workflow is available. Build and offline-verify a
coherent experimental artifact through that workflow, deploy only through the
verified installer, then claim a separate product-route-validation lease for
the redacted non-secret Pi configuration and bounded first-response diagnostics.
