# Personal P1-T09 deterministic Provider fixture handoff

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Classification: implementation-only; normative surface unchanged
- Implementation commits: `96844b8`, CI repair `56d0f7f`
- Branch: `lane/personal-p1-t09-provider-fixture`
- Remote visibility: PR [#117](https://github.com/agentkernel/cognitive-os/pull/117);
  latest pushed commit `56d0f7f`; required Ubuntu/Windows CI is green

## Completed

1. Added `p1-t09-provider-fixture`, a deterministic loopback-only HTTPS binary
   that publishes an ephemeral `localhost` endpoint and a test-only DER trust
   root through explicit files. It supports ready, malformed catalog,
   unauthorized, non-chat-capable, timeout, oversized-response, and redirect
   scenarios. It records only method/path and authorization presence, never
   request headers, bodies, or secret-like values.
2. Added an implementation-local additional-root constructor to
   `RustlsProviderTransport`. Production defaults remain unchanged: HTTPS-only,
   credential-free URLs, no redirects, header-injection rejection, bounded
   timeout/cancellation, Rustls, and the 1 MiB response limit remain enforced.
3. Added failure-first process-level tests that run the real Rustls transport
   and `ProviderDiscoveryService`, assert exact model selection and capability
   outcomes, verify deterministic request counts, exercise malformed/status/
   timeout/size/redirect negatives, and check that Provider state, diagnostics,
   and fixture observations do not contain the synthetic secret marker.
4. Reconciled the formal P1-T09 row, current progress snapshot, and active
   Lane-RUN lease. No registry, schema, public DTO, registered error,
   transition, generated contract, conformance vector, Pi argv/environment,
   Task, Effect, Verification, capability, or authority writer was changed.
5. Repaired the two real CI failures: the non-chat scenario now returns no
   `choices` marker, and the test child preserves only OS execution variables
   required by the Windows loader after clearing its inherited environment.

## CI follow-up

The first CI run found two compile integration issues and both were corrected
without changing the fixture contract: existing composition roots now use the
configured transport's `Default` value (`006ccff`), and the process test now
derives the fixture beside the Cargo test executable rather than relying on an
unset `CARGO_BIN_EXE_*` environment variable (`c424093`, `b4beb50`,
`62c6782`). The replacement checks passed in PR #117: the exact fixture
integration suite ran **3/3** in both Ubuntu and Windows workspace test jobs
of CI run 30513254161. All required Ubuntu/Windows checks are green.

## Failure-first evidence

The new focused test was written before the fixture and transport seam. The
first attempted command was:

```text
cargo test -p cognitive-provider-transport --test p1_t09_deterministic_provider_fixture --locked
```

It exited 101 before repository tests started because the local unsupported
Windows GNU linker exited 121 while building dependencies. After the
implementation, the same local environment also blocked `cargo check` before
type checking for the same linker reason. WSL was inspected for a supported
toolchain but has no `cargo` available. Therefore the intended focused suite
is explicitly **not-run**, not pass evidence.

## Verification

- `cargo fmt --all -- --check` — passed.
- `git diff --check` — passed.
- IDE lint diagnostics for the three changed Rust source/test paths — none.
- Local `cargo test ...p1_t09_deterministic_provider_fixture...` — not-run to
  completion; unsupported Windows GNU linker exit 121 before tests.
- Local strict Clippy — not-run for the same unsupported local toolchain
  limitation.
- Supported CI `cargo test --workspace --locked` — passed; the exact fixture
  suite was **3/3** in both Ubuntu and Windows jobs of run 30513254161.
- Supported CI strict Clippy and formatter — passed in both required jobs.
- `pnpm run check:consistency` — passed: 273 requirements, 55 error codes, 63
  schemas, 85 vectors, links, and traceability verified.
- Secret scan, full workspace regression, and real Pi load — not-run.

## Status and explicit non-claims

- `P1-T09`: `in-progress`; `experimental-local-only`;
  `implementation_evidence: tested-supported-ci` for the deterministic
  fixture, with prior route evidence retained at `tested-local`.
- `B01`: `not-run`.
- `GMVP-LINUX`: `not-run`.
- Profile `implemented`: `0`.
- This batch does not claim a real Provider conversation, real pinned Pi
  Extension load, native Secret Service campaign, clean Linux VM campaign,
  B01, release, GMVP-LINUX, or Profile conformance.
- The fixture cannot create Task, Effect, Verification, capability, or
  authority side effects; its process boundary is only a Provider test seam.

## Lease and next entry

- The P1-T09 Lane-RUN CI-repair lease is closed after the green required CI
  and closure docs. Its paths included the Provider transport crate and the
  listed plan/progress/handoff documents.
- Next entry: claim a separate non-overlapping Lane-RUN lease for the smallest
  real pinned Pi Extension-load slice. Do not run B01 until all route
  acceptance prerequisites and campaign pre-registration are complete.

## Linux-native environment qualification follow-up

On 2026-07-30, non-interactive no-secret SSH to
`personal-linux-native-01` (`wuz@192.168.1.2`) confirmed an available native
Linux x86_64 environment: a present user runtime directory, running
user-systemd, active user D-Bus, Rust `1.97.1`, and Node `22.19.0`. This
qualifies the host only for disposable `experimental-local-only` /
`tested-local` debugging; it is not B01, release, containment, Profile, or
product-Gate evidence.

`pi` was absent from PATH. An uncredentialed exact-package probe,
`npm exec --yes --package=@earendil-works/pi-coding-agent@0.81.1 -- pi --version`,
did not emit a version after two minutes and was stopped. Exact Pi availability
and a real P1-T09 Extension load therefore remain `not-run`. The next slice
must first resolve the version-verified package/binary in a disposable remote
directory, then run the smallest `--extension <absolute-path>` load with only
redacted session-local observations and no Provider/secret/SQLite/authority
material in the child environment.
