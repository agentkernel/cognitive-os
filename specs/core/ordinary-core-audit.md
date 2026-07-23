# Ordinary Core AUDIT contract

- Status: normative companion for the registered Ordinary Core `status.inspect`
  AUDIT schemas; it is neither a conformance result nor a Profile claim.
- Scope: only `privileged_read_decision`, `audit_commit_receipt`, and the
  `ManagementAuditPort.commit_privileged_read_decision` responsibility.

## Registered assets and digest rules

- `privileged-read-decision.schema.json` is the complete record admission
  surface. Its digest domain is `management-privileged-read-record/0.2`; its
  projection is all and only admitted fields, with an empty exclusion set.
- `audit-commit-receipt.schema.json` binds an already computed record digest;
  a receipt is not part of the record-digest projection.
- Request digests use `management-privileged-read-request/0.2` over exactly
  `{domain, object_id}`. Result digests use
  `management-privileged-read-result/0.2` over the canonical `InspectReport`
  value.
- All three use `cognitiveos.canonical-json/0.1` and SHA-256.

## Minimal port responsibility

`ManagementAuditPort.commit_privileged_read_decision(record, record_digest) ->
AuditCommitReceipt` MUST re-verify the supplied record digest, durably commit
before returning, and bind the same record ID, record digest, and request
digest in its receipt. `sequence` and `writer_epoch` MUST be positive.
`committed_at` MUST NOT precede `observed_at`. Commit or receipt validation
failure MUST withhold the success result.

Cross-object receipt binding and commit-time ordering are port/runtime behavior
obligations, not claims that JSON Schema alone proves them. `AuditPortFailure`
is internal; the only allowed public reuse is `STATE_STORE_UNAVAILABLE` for an
unavailable same-process durable authority boundary, with zero success result.

This contract does not register query, export, checkpoint, retention, legal
hold, signatures, stream topology, or any High-Assurance AUDIT surface.
