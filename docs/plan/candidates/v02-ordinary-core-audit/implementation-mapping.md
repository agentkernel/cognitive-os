# Implementation field mapping

| Candidate field/property | Real implementation evidence | Freeze ruling |
|---|---|---|
| decision `record_kind` | `plane.rs`: literal `privileged_read_decision` | exact |
| `record_id`, `observed_at` | authority UUIDv7 generator and trusted clock | exact |
| `request_digest` | canonical `{domain, object_id}` under `management-privileged-read-request/0.2` | raw selector excluded |
| `outcome`, `safe_reason`, `result_digest` | `inspect_with_audit` branch over result; `ManagementError::registered_parts` supplies a registered code as safe reason | exact terminal shape |
| receipt six fields | `AuditCommitReceipt` and `FileManagementAuditLog` return path | exact |
| positive sequence/epoch | `ResultReleaseGate::validate`; file adapter restart epoch and contiguous global sequence | exact |
| time ordering | `ResultReleaseGate::validate` compares commit with observed time | exact |
| durable-before-result | `append_value` calls `sync_all`; gate validates before returning the inspect result | exact |

The candidate is stricter than Serde's current permissive unknown-field decoding:
`additionalProperties: false` is the intended future public contract boundary.
This is a registration-stage implementation gap, not a reason to change Lane-RUN
in this freeze batch.

## Reachable safe-reason closure

The candidate enum is mechanically compared to every code in
`specs/registry/errors.yaml`; it therefore permits no unregistered value. The
currently reachable `status.inspect` paths evidenced by the implementation are
`CONTEXT_AUTH_DENIED` for protected-read denial/not-found and
`STATE_STORE_UNAVAILABLE` for the in-process durable authority boundary failure
described below. Both are members of the enum. No future AUDIT code is admitted.
If a later Lane-RUN path emits a non-enumerated value, that is a precise
implementation/schema difference for Lane-RUN follow-up, not grounds to weaken
this candidate.

## External error ruling

`STATE_STORE_UNAVAILABLE` is reused only for a transient failure of the same
in-process durable authority boundary: open/lock/readback/write/sync failure or
receipt validation failure that makes the audit commit unavailable. Its existing
meaning already requires a fail-closed authoritative persistence path and is
retryable. The public oracle is **zero inspect success result**. `AuditPortFailure`
remains internal detail. This does not broaden the code to stream integrity,
tamper, retention, export, cryptographic, or High-Assurance audit responsibilities;
those remain deferred. No error registry or generated binding changed.
