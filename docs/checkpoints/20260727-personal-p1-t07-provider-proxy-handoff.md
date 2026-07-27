# 20260727 Personal P1-T07 Provider Proxy Handoff

## 1. Scope and status

- Branch: `lane/personal-p1-t07-provider-proxy`.
- Formal task: **P1-T07 remains `in-progress`**.
- This batch implements the daemon-side Provider proxy boundary and records the
  production `ProviderTransport` decision. It does not claim a complete Pi
  conversation integration.

## 2. Decision recorded

- The production transport is in-process blocking `reqwest` with Rustls in
  `kernel-server`; `cognitive-secret` remains transport-injected and has no
  HTTP/TLS dependency.
- A subprocess backend was rejected: Provider egress is ordinary daemon HTTPS,
  whereas a subprocess would add credential-passing, lifecycle, cancellation
  and output-redaction risk with no authority-boundary benefit.
- The Personal front door currently is one request/one connection and has no
  SSE/backpressure/disconnect-abort protocol. P1 explicitly supports only a
  bounded non-streaming exchange; `stream:true` is rejected.
- ADR: `docs/adr/0027-personal-pi-extension-and-runtime-observation.md`.

## 3. Implemented batch

- `POST /provider/v1/chat/completions` is a daemon route requiring a management
  channel bearer before any Provider configuration or secret lookup.
- `ProviderProxyService` resolves Provider material only through daemon-owned
  `ProviderKeyService` and attaches the Bearer value only to the outbound
  request.
- `RustlsProviderTransport` requires HTTPS, rejects URL user-info and header
  injection, disables redirects, applies a request timeout and caps the
  response body at 1 MiB.
- The proxy is an egress adapter: it creates no Intent/Effect, grants no
  capability, writes no SQLite state and advances no Task or Verification.
- Focused hermetic coverage captures the daemon-to-transport request with a
  synthetic marker and verifies its endpoint, request body, and daemon-built
  authorization header. Existing process coverage verifies unauthenticated and
  unconfigured requests fail closed without echoing the management bearer.

## 4. Verification state

| Command | Result |
|---|---|
| `cargo generate-lockfile` | passed; regenerated `Cargo.lock` with `reqwest`/Rustls dependency resolution after its preexisting worktree state. |
| `cargo fmt --all` | passed. |
| `cargo test -p kernel-server --test p1_t07_provider_proxy --locked` | **not-run to completion**: Windows GNU linker failed with known exit 121 before tests executed; this is a non-supported local baseline. |
| `CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo test -p kernel-server --test p1_t07_provider_proxy --locked` (WSL) | passed after correcting the response-size type mismatch found by the same supported Linux toolchain. |
| `CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo test -p kernel-server --test p1_t07_pi_readiness --locked` (WSL) | passed: real Personal front-door readiness reports absent, configured, and corrupt Pi observations. |
| `CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo test -p kernel-server --locked` (WSL) | passed: complete `kernel-server` package suite, including the P1-T07 focused tests. |
| `CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t07-provider-proxy /root/.cargo/bin/cargo clippy -p kernel-server --all-targets --locked -- -D warnings` (WSL) | passed after resolving the batch's test-only strict-lint findings. |
| `pnpm run check:consistency` | environment-blocked: workspace dependency `ajv` is absent (`ERR_MODULE_NOT_FOUND`); this is neither a consistency pass nor a repository consistency failure. |
| `git diff --check` | passed. |

CI is still required for the complete supported matrix, including Windows/MSVC.
Do not record the Windows GNU linker limitation as a repository test failure.

## 5. Evidence boundaries

- No real Provider credential, token, raw Provider response or raw remote
  transcript was stored.
- Synthetic test material is not a production credential.
- The P0-T06 Linux-native `extension-load` record remains local experimental
  PoC/non-claim evidence; this batch creates no containment, Gate, Profile,
  C0/C1 or release claim.

## 6. Remaining P1-T07 work and next step

- The pinned Pi Extension API mirror has no verified completion-provider
  registration/interception hook. Therefore Pi is not wired to this proxy.
- Do not work around this by letting Pi read `provider.json`, resolve a secret,
  use an API-key environment variable or configure an independent Provider.
- First obtain and validate a supported pinned-Pi completion hook (or document
  a safe alternate official integration point), then add Pi client coverage and
  only then reconsider P1-T07 completion.
- This provider-proxy batch is a clear atomic checkpoint. After CI is green,
  P1-T08 Linux bundle installer/user-service work may start in parallel with
  the remaining P1-T07 compatibility investigation, but P1-T07 itself must
  remain `in-progress`.

## 7. Suggested next-window prompt

"Read `20260727-personal-p1-t07-provider-proxy-handoff.md`, check CI for the
provider-proxy batch, then investigate the pinned Pi 0.81.1 supported
completion-provider integration surface without allowing Pi to access Provider
configuration or secrets. Keep P1-T07 in-progress unless that integration is
implemented and tested; otherwise advance P1-T08 only after the proxy batch is
cleanly merged."
