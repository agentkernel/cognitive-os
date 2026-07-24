# Lane-RUN Custom User-Provided installation handoff

- Date: 2026-07-24
- Base: `origin/main@7d23e62` (PR #79 durable installation authority)
- Branch: `codex/custom-agent-install`
- Scope: runtime trust-policy consumption for REQ-AGENT-INSTALL-001/002; no
  KRN store, schema, registry, vector, Profile, or error-code changes.

## 1. Completed

- Added `InstallationTrustMode` and an explicit
  `CustomUserProvidedProjectVerifier` implementation of
  `SignatureProvenancePort`.
- Custom mode requires the caller to display a fixed risk notice and receive an
  affirmative acknowledgement for an exact artifact digest, `principal://`
  operator ref, and `file://` immutable project-bundle ref. The standard
  verifier does not accept these custom references unless this policy is
  deliberately selected.
- After acknowledgement, a custom project follows the same normal authorization,
  execution, and lifecycle path as a normal installation. As with normal
  installation, commit itself grants no capability and creates no Task completion
  or Effect. It is not publisher provenance, C0/C1, Profile, or sandbox evidence.

## 2. Verification

- Focused red tests first failed because the verifier was absent; the three
  focused tests then passed after implementation.
- `cargo test -p cognitive-runtime -j 1`: pass (41 unit + 2 integration).
- `cargo clippy -p cognitive-runtime --all-targets -- -D warnings`: pass.
- `cargo fmt --check`, `git diff --check`, and `pnpm run check:consistency`:
  pass (273 requirements, 55 error codes, 63 schemas, 85 vectors).

## 3. Remaining boundaries

- P2 official-publisher verification is still blocked: npm SRI alone does not
  establish trusted signature/provenance.
- Custom mode does not replace Linux-native OS sandbox evidence, Pi lifecycle/
  I/O mediation, management authorization, cross-process lifecycle leasing,
  performance evidence, or Profile conformance.
- Custom mode accepts a bundle identity, not a mutable project directory; a
  caller must package and digest the project before installation.

## 4. Next entry

- Suggested prompt: `docs/prompts/lane-run.md`.
- First action: push the verified branch, open a PR, and wait for CI. Continue
  P2 official verification separately; do not collapse Custom User-Provided
  provenance into an upstream publisher-signature claim.
