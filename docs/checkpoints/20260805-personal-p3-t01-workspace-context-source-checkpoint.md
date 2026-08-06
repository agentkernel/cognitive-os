# P3-T01/D01 Workspace Context source checkpoint

- Classification: `implementation-only`
- Branch: `lane/ctr-p3-t01-context-request-binding`
- Slice status: `in-progress`; validation and authorization composition remain open.

## Implemented

This checkpoint adds the first daemon-owned durable source for later Context
resolution without admitting arbitrary files, client payloads, or Pi output as
Context authority:

1. migration v13 creates append-only `workspace_context_sources` records;
2. each source is sealed and validated against its durable identity, digest,
   governance metadata, provenance, role, trust, representation, and cost;
3. Context discovery is tenant/scope/conversation metadata-only and therefore
   cannot expose a source body to ranking; and
4. body loading is a distinct port operation for a later caller that has first
   reconstructed authorization and revocation facts.

## Non-claims and remaining work

This is not a Context resolver integration or an authorization grant. It does
not reconstruct actor-chain, membership, capability, explicit-deny, or current
revocation facts, and it does not modify the P2-T04-owned resolver, scheduler,
or Pi paths. A source body must not become a `CandidateObject` until a durable
authorization source permits its `read_body` access.

The next required increment is a durable authorization snapshot source and
focused revocation regressions proving a revoked source cannot reach body load
or ranking. Exact native Linux validation and required CI for this checkpoint
are `not-run` pending the pushed immutable revision.

## Local checks

- `cargo fmt --all`: passed on the Windows static-only host.
- `git diff --check`: passed.
- Rust compile/test/Clippy: `not-run` locally; the registered Windows GNU host
  cannot perform Rust linking. Route exact committed revision to native Linux
  before any completion claim.
