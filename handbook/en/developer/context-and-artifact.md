---
doc_id: dev.context-artifact
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-kernel/src/context.rs
    symbols: ["resolve", "STAGNATION_BOUND", "effective_control_plane"]
  - path: crates/cognitive-kernel/src/context_cache.rs
    symbols: ["GovernedContextCache", "ContextCacheKey"]
  - path: crates/cognitive-store/src/context_store.rs
  - path: crates/cognitive-store/src/artifact_store.rs
    symbols: ["put_expected", "get_authorized"]
contracts:
  - specs/schemas/context-request.schema.json
  - specs/schemas/context-view.schema.json
tests:
  - crates/cognitive-kernel/tests/context_pipeline.rs
  - crates/cognitive-store/tests/m5_context_store.rs
  - crates/cognitive-store/tests/p3_t03_artifact_store.rs
fingerprint: "sha256:0faeb3aeaee5e36cd150c344ac527eff4fc2fc19520ff85377fffdc420a26ddf"
non_claims:
  - Context correctness evidence is focused-test evidence; benefit/efficiency observations remain non-claims owned elsewhere.
---

# Context and Artifact

## The nine-stage resolution pipeline

`context::resolve` runs: admission → governance pre-filter (tenant/conversation,
**before** any ranking or body read) → retrieval record → per-object
re-authorization via `authorize` (action `read_body`) with content-digest dedup →
ranking (the only probabilistic slot; a proposal may only reorder or shrink the
authorized survivors) → budget fitting (required-first; over-budget fails
`CONTEXT_BUDGET_EXCEEDED` unless `allow_partial` with explicit `missing`) → loss
declaration (silent omission is impossible) → deterministic rendering (partition
order control → authoritative_state → evidence → working → untrusted_input;
prefix-stable, digest domain `cognitiveos.impl.context-render/0.1`) → view
emission with pinned versions and the full `GovernanceBinding`.

Prompt-injection isolation is structural: untrusted content renders as data and
`admit_control_mutation` refuses control-plane changes attributed to it. Bounded
re-resolution stagnation surfaces `CONTEXT_RESOLUTION_STAGNATED`
(`STAGNATION_BOUND` = 2).

## Durable Context rows

`ContextStore` persists append-only ContextRequests and ContextViews with sealed
content digests; a view's strong `request_ref` is checked against the persisted
request digest (not caller input). Workspace sources carry role/trust CHECK
constraints; discovery is metadata-first with separate body loads;
authorization/revocation fact sets reconstruct `AuthzSnapshot`s at the **current**
revocation epoch. On the real scheduler path the daemon reloads
authorization/revocation immediately before every body load and seals the view
before any Pi transport.

## Caches that cannot serve stale authority

Both caches key on the full governance binding (tenant, actor-chain digest,
capability-set version, revocation epoch, purpose, schema digest, encoding
profile, conversation); `GovernedContextCache` additionally binds request/contract
identity+digest, ordered source digests, renderer version, and validated tool
descriptor digest, and stores digest-only prefix/delta metadata. Stale bindings
miss by construction; a declared-stale serve refuses with `CONTEXT_AUTH_DENIED`
and purges all derived cache kinds.

## Artifact CAS

A bounded filesystem CAS: references are strictly `sha256:<64hex>` (never
interpreted as paths), `put_expected` verifies size + digest before staging-file
+ atomic rename publication, `get` re-hashes on read (tamper ⇒ `DigestMismatch`),
`get_authorized(_, false)` fails closed (policy is the caller's), and only
abandoned staging files are ever cleaned. The verifier consumes evidence through
this store, so a report cannot persist unless its evidence bytes exist and hash
correctly. The Personal daemon now opens one process-lifetime instance at
`data_dir()/artifacts` with an 8 MiB per-artifact ceiling; D01 composition alone
does not mean a production verifier has run. A reconciled Effect can now be
pinned with its verification request in the same authority transaction that
publishes Loop `ACT -> VERIFY`. Criteria now derive only from current
TaskContract Acceptance conditions; the registered fixed-Effect verifier writes
its immutable post-state observation into this CAS before a passed report can
enter `VERIFY -> CONTINUE`. P2-T14 re-reads every report Artifact URI before
acceptance. Its daemon-authored completion claim and affirmative
`acceptance_decision` are canonical CAS bytes referenced through the existing
transition `StrongReference`; missing or digest-mismatched bytes fail before the
Task transition.
