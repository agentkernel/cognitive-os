# 20260727 Personal P1-T07 Pi Integration Surface Blocker Handoff

## 1. Decision

P1-T07 remains **in-progress**. The exact pinned Pi `0.81.1` source commit
does expose a supported custom-provider extension surface, but the current
CognitiveOS daemon proxy cannot safely use it yet. No Pi-to-Provider wiring
was added in this batch.

## 2. Pinned upstream evidence

The pinned values are `@earendil-works/pi-coding-agent@0.81.1` and source
commit `20be4b18d4c57487f8993d2762bace129f0cf7c6` (see
`apps/pi-agent-adapter/src/lib.rs`). At that exact source commit:

- `packages/coding-agent/docs/custom-provider.md` documents
  `ExtensionAPI.registerProvider(...)` as the extension-supported route for a
  complete custom Provider and custom streaming transport.
- The custom Provider route is the only safe candidate. Its documented model
  registration and transport callback can route completion traffic to the
  daemon without a persisted Pi Provider file or upstream Provider key.
- `before_provider_request` and `before_provider_headers` are public hooks but
  are not sufficient for the boundary: they do not prove that Pi never resolves
  a legacy provider credential or uses a native Provider endpoint.

The local `packages/pi-cognitiveos/src/pi-api.ts` is intentionally a narrow
structural mirror. It contains only the trust, tool, session-start, and command
surfaces currently used by the extension, so it does not prove or expose
`registerProvider`.

## 3. Precise safe-use blocker

The supported surface cannot be wired through the checked-in implementation
without first closing all of these gaps:

1. Pi's documented complete Provider contract is streaming. The daemon proxy
   at `apps/kernel-server/src/personal/provider_proxy.rs` rejects every
   `{"stream":true}` request before Provider material is resolved, with
   `PERSONAL_PROVIDER_STREAMING_UNSUPPORTED`.
2. The daemon has no management-authenticated, read-only model projection from
   which the extension can register the selected model. The extension must not
   substitute `provider.json`, `SecretRef`, a direct Provider URL, or a
   duplicated Provider model configuration.
3. The local structural mirror has no verified declaration for the Pi
   complete-Provider/event-stream API. Adding a legacy provider config plus a
   header-interception hook would leave credential/config-resolution behavior
   outside the intended daemon-only boundary and is therefore rejected.

Do not work around these blockers with any of the following:

- Pi-side `provider.json`, `auth.json`, upstream Provider key, or `$ENV` key;
- direct Provider HTTP egress or an independently configured Provider;
- SQLite access or any authority write from the extension;
- Pi model-registry/runtime internals, monkey patches, or unpinned APIs.

## 4. Required follow-up to finish P1-T07

The next safe P1-T07 implementation batch must:

1. Add a minimal daemon-owned, management-authenticated, non-secret model
   projection.
2. Define only the exact pinned complete-Provider public types required by the
   extension, with a pinned runtime fixture for registration.
3. Bridge the daemon's bounded one-shot completion response to Pi's documented
   assistant event stream, or add an equally bounded daemon streaming protocol
   with explicit cancellation and size limits.
4. Prove in focused negative tests that the extension uses only daemon routes,
   never observes Provider configuration or secret material, and fails closed
   on session/channel/auth failures.

No implementation, Gate, Profile, containment, or release conclusion follows
from the upstream API investigation or any local/WSL test result.

## 5. P1-T08 pivot boundary

P1-T08 implementation is currently dependency-blocked by P1-T07 in
`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`. Do not create a fake bundle or user
service while the only supported completion path is unclosed. The next P1-T08
safe preparation is limited to reviewing its required verifier, interruption,
and rollback semantics against the existing daemon lifecycle; it must not be
reported as installer implementation or test evidence.

## 6. Verification

The current worktree has unrelated unstaged documentation changes and
`.claude/`; do not stage, revert, or include them in this atomic batch.

Executed from the WSL guest on 2026-07-27; all passed:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo test -p kernel-server --test p1_t07_provider_proxy --locked
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo test -p kernel-server --test p1_t07_pi_readiness --locked
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo test -p kernel-server --locked
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo clippy -p kernel-server --all-targets --locked -- -D warnings
```

Also run `git diff --check`. These are local verification steps only; they do
not replace the supported CI matrix. `git diff --check` passed. `pnpm run
check:consistency` could not start because this worktree lacks the `ajv`
package required by `tools/src/check-consistency.mjs`; this is an environment
dependency failure, not a repository consistency result.
