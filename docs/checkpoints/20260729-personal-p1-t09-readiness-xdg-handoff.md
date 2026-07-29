# Personal P1-T09 Readiness and XDG Launch Handoff

**Date:** 2026-07-29
**Branch:** `lane/personal-p1-t08-mvp-single-service`
**Task state:** install-to-first-conversation route `in-progress`; `P1-T09 / B01`
remains `not-started`

**Implementation commit:** `40d779a` (`P1-T09: tighten Personal launch readiness`).
This handoff is a follow-up documentation commit; its push status is recorded
below after the branch push attempt.

## Completed local implementation slice

This slice closes two false-ready/split-layout prerequisites without adding a
public DTO, SSE shape, registry error, schema, transition, or vector.

- The Personal Provider readiness component now requires a valid
  `selected-model.json` whose non-secret snapshot digest matches the digest in
  `provider.json`. Missing selected-model state, an invalid document, or a
  mismatched digest is `Blocked`; each keeps both aggregate and
  `first_conversation_ready` fail-closed. Readiness facts and doctor guidance
  remain non-secret and do not print `SecretRef` or Provider material.
- `cognitive daemon start` now omits `--runtime-root` when the operator did
  not request the explicit hermetic test root. The spawned Personal daemon
  therefore resolves the same user XDG configuration, state, data, cache, and
  runtime layout as `cognitive init` and the Pi extension. An explicit root
  remains forwarded for hermetic tests.
- The CLI default daemon bind is the already canonical loopback service
  address `127.0.0.1:48181`.
- Failure-first tests first demonstrated the stale-selected-model false-ready
  state and the old `7420` default, then verify the fixed fail-closed behavior,
  hermetic-root forwarding, and installed-XDG argument omission.

## Verification executed

All Rust commands below ran in `windows_wsl2_linux_guest` with
`CARGO_TARGET_DIR=/tmp/cognitiveos-p1t09-pi-route`.

```text
cargo test -p kernel-server --bin kernel-server --locked
# 32 passed

cargo test -p kernel-server --test p1_t07_pi_readiness --locked
# 1 passed

cargo test -p kernel-server --test p1_t05_personal_readiness --locked
# 1 passed

cargo test -p kernel-server --test p1_t07_provider_proxy --locked
# 2 passed

cargo test -p admin-cli --lib personal_cli --locked
# 6 passed

cargo test -p admin-cli --test p1_t06_cognitive_cli --locked
# 5 passed

cargo clippy -p kernel-server -p admin-cli -p pi-agent-adapter \
  -p cognitive-provider-transport --all-targets -- -D warnings
# passed

cargo fmt --all -- --check
pnpm run check:consistency
git diff --check
# passed
```

The known local Windows GNU linker exit 121 was not used as functional test
evidence or a functional failure conclusion.

## Claims and contract boundary

This is implementation and local-test evidence only. It is not Linux-native
Secret Service evidence, a Pi launch or real Pi Extension load, a deterministic
binary Provider fixture, a real first conversation, development smoke,
usability learning campaign, clean-VM B01 result, product Gate, release, or
Profile conformance claim.

No normative contract asset changed: registry, schemas, transitions, vectors,
and generated public DTOs/SSE shapes are unchanged. The local readiness
projection behavior was tightened only; if a future Pi provider-session token
needs a public DTO, SSE shape, error code, registry entry, schema, or vector,
stop at that contract boundary and use Lane-CTR.

## Remaining work and next safe entry

1. Confirm the exact, pinned Pi `0.81.1` documented Extension loading syntax
   from the reviewed release artifact before adding a supported Pi
   configuration/launch command. Do not repurpose the candidate-only
   `pi-agent-adapter` direct-secret development exception for this route.
2. Design a daemon-only, narrowly scoped Pi Provider session that never grants
   Pi Provider configuration, `SecretRef`, Provider key bytes, SQLite access,
   authority writes, or Task/Effect/Verification transitions.
3. Add a true composition-root binary-level deterministic Provider fixture
   only after the Pi session boundary is defined. It must not egress to a real
   Provider, use a real key, or leak its synthetic marker through argv, env,
   logs, ordinary configuration, SQLite, test output, or evidence.
4. Keep the existing unrelated uncommitted
   `apps/kernel-server/src/personal/server.rs` modification untouched and out
   of all staging/commits for this route.

**Push status:** pending at creation of this handoff; update from the actual
push result only. Remote visibility is not claimed before a successful push.
