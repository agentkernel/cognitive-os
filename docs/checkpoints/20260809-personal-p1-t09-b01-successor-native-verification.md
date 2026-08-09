# P1-T09 B01 successor native artifact verification

- Date: 2026-08-09
- Classification: `corrective`
- Task: `P1-T09`
- Gate: `B01`
- Campaign: `B01-clean-linux-first-install-first-conversation-002`
- Branch: `personal/P1-T09-b01-campaign-continuation`
- Lease: `lease/personal/P1-T09/b01-successor-native-verification`

## Independent verification result

The native verifier host restored GitHub connectivity through an isolated,
user-level Mihomo route. The route listens only on `127.0.0.1:17890`, uses the
owner-provided subscription with `url-test` automatic selection, and was used
only as an explicit per-command proxy for source and dependency retrieval. It
does not modify system proxy settings or the existing Mihomo instance.

The verifier host fetched a disposable Git worktree at exact reviewed revision
`4ea42c0c8f856aa22e2a360bd42005c8dbec400f`. A fresh controller download of
GitHub Actions artifact `9029551950` was copied into separate disposable native
staging, where the bundle SHA-256 was recomputed as:

```text
sha256:fb5d006fa25a6545d46b3089e53a8fe08fe9df0d113165c799752154b981abf4
```

That matches the artifact manifest and preregistration. The exact-revision
native command was:

```text
cargo run --locked -p cognitive-runtime --bin linux_bundle_verifier -- \
  --bundle-directory <disposable-native-artifact-directory> \
  --expected-pi-version 0.81.1 \
  --expected-pi-integrity <preregistered-SRI> \
  --keyring-version p1-t09-experimental-20260730 \
  --key-id p1-t09-experimental \
  --public-key-base64url Uui0QQibM4z49Md4N55ANrkpli_12IMpn_W8rmB5vdk
```

It exited `0` and retained only the following non-secret result:

```text
verified-linux-bundle version=0.0.0-campaign.20260809.1
trusted-key-id=p1-t09-experimental
keyring-version=p1-t09-experimental-20260730
```

No bundle was installed, no service was started, and no B01 guest operation or
campaign attempt occurred.

## Remaining start gate

Artifact verification is passing. Before successor Attempt 1, the designated
operator must be available to enter the Provider credential only through the
graphical product hidden-input prompt after the clean-reset checkpoint. The
credential must not be placed in chat, shell input, configuration files, logs,
evidence, or Git. The independent verifier must then review redacted route and
cleanup evidence.
