# Error Contract Standard

- Standard ID: `cognitiveos.standard.error-contract/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: `specs/registry/errors.yaml` (55 registered codes, 18 categories)

## 1. Scope and normative language

The key words MUST, MUST NOT, SHOULD, and MAY follow RFC 2119/8174. This
standard binds how registered error codes are produced, propagated, retried,
and audited. It registers no new requirement and no new code: the code set is
frozen with the v0.1 specification surface. Owning requirements:
[REQ-ERR-001] and [REQ-ERR-002] in `specs/registry/requirements.yaml`.

## 2. Registered codes only

A governed failure MUST surface exactly one registered `code` from
`specs/registry/errors.yaml`. Implementations MUST NOT invent codes, reuse a
code for a different meaning, or collapse distinct registered codes into a
generic failure. Free-text detail goes in a `detail` field, never in `code`.

An unregistered code in a machine response is itself a defect; the
consistency check (`pnpm run check:consistency`) fails any vector, matrix or
document referencing a code absent from the registry.

## 3. Retryability is contract, not heuristic

Each registered code carries `retryable`. A client or Loop MUST NOT retry a
`retryable: false` code (for example `EFFECT_IDEMPOTENCY_CONFLICT`,
`CONTEXT_AUTH_DENIED`, `CONTEXT_BUDGET_EXCEEDED`) and MUST NOT treat retry of
a `retryable: true` code (for example `STATE_CONFLICT`,
`AUTH_CAPABILITY_EXPIRED`, `EFFECT_OUTCOME_UNKNOWN`) as implicit success.
Retry of `STATE_CONFLICT` MUST re-read authoritative state and re-evaluate
guards; retry MUST NOT reuse a stale `expected_state_version`.

`EFFECT_OUTCOME_UNKNOWN` is retryable only through reconciliation: the retry
path is reconcile-or-quarantine, never blind re-dispatch
(`docs/standards/intent-effect-idempotency.md`).

## 4. Fail-closed mapping

Failures on the authoritative commit path MUST map to fail-closed codes and
MUST NOT buffer, guess, or degrade silently:

1. Persistence unavailable on a governed write: `STATE_STORE_UNAVAILABLE`;
   the write is rejected, not queued in memory ([REQ-REC-003],
   vector `state-store-degradation.json`).
2. Schema/digest verification failure: `SCHEMA_MISMATCH`,
   `PROTOCOL_SCHEMA_DIGEST_MISMATCH`, or `DIGEST_MISMATCH`; the object is
   rejected before authorization or transition
   (`canonical-encoding-and-digest.md` section 15).
3. Unknown critical extension: `CRITICAL_EXTENSION_UNKNOWN` before any
   payload processing.
4. Version outside the negotiated window: `VERSION_UNSUPPORTED`.

## 5. Denial semantics and information flow

A denial (`CONTEXT_AUTH_DENIED`, `MANAGEMENT_SCOPE_MISMATCH`,
`MANAGEMENT_SELF_AUTHORIZATION_DENIED`, ...) MUST be isomorphic with
not-found where existence itself is protected: same shape, same code class,
no resource metadata leakage (vector `tenant-lateral-read-denial.json`,
[REQ-GOBJ-HEADER-001], [REQ-CTX-002]). A denial MUST be produced before any
side effect: observable
assertions are `dispatches == 0` and `effects_created == false`
(vector `management-gate-denials.json`).

## 6. Audit obligation

Every registered-code failure on an authority path MUST emit an audit-capable
event carrying code, category, stage, and correlation IDs, without secret
content ([REQ-AUDIT-001], [REQ-AUDIT-002]). Error responses and audit records
use the same code value; translation tables are forbidden.

## 7. Compliance checks

An implementation claim against this standard requires: negative tests per
consumed code path (`tests/security/`, `tests/faults/`), the registry
consistency check green, and no occurrence of an unregistered code string in
`crates/**` or `packages/**` machine responses.
