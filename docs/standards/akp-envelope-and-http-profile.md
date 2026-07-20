# AKP Envelope and HTTP Transport Profile Standard

- Standard ID: `cognitiveos.standard.akp-envelope-http/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: AKP envelope rules in `specs/akp/README.md`;
  `specs/registry/errors.yaml` protocol codes; envelope/stream/control wire
  schemas `specs/schemas/akp-request-envelope.schema.json`,
  `akp-result-envelope.schema.json`, `akp-stream-frame.schema.json`,
  `shell-control-request.schema.json` (D-013/D-014/D-015 closure)
- Normative sources: AKP companion (`specs/akp/README.md`); ADR-0003
  (reference implementation transport decision, not a specification
  requirement)

## 1. Scope and normative language

RFC 2119/8174 language applies. This standard fixes how the reference
implementation carries AKP over HTTP JSON + SSE on a single node. Owning
requirements: [REQ-AKP-ENV-001], [REQ-AKP-ENV-002], [REQ-AKP-VER-001],
[REQ-AKP-CAN-001], [REQ-AKP-IDEM-001], [REQ-AKP-CONT-001],
[REQ-AKP-SEC-001], [REQ-AKP-STR-001], [REQ-AKP-STR-002],
[REQ-AKP-SHELL-001..003], [REQ-AKP-MGMT-001..003], [REQ-AKP-RES-001],
[REQ-AKP-ECO-001..003], [REQ-AKP-INTENT-001], [REQ-AKP-CONF-001]. It
registers no new requirement.

## 2. Envelope invariants

Every AKP message carries the versioned envelope: protocol version, message
type, correlation/causation IDs, actor context reference, payload schema
pin (digest), and idempotency key where the operation is effecting
([REQ-AKP-ENV-001..002], [REQ-AKP-IDEM-001]). The registered wire shapes
are `akp-request-envelope.schema.json` and `akp-result-envelope.schema.json`
(negative vectors `akp-envelope-no-schema-pin-001.json`,
`akp-envelope-ambiguous-payload-002.json`,
`akp-result-error-without-machine-code-003.json`). Payload bytes are
validated against the pinned schema digest before processing
(`PROTOCOL_SCHEMA_DIGEST_MISMATCH` on mismatch); canonicalization follows
`canonical-encoding-and-digest.md` ([REQ-AKP-CAN-001]).

Version negotiation happens before semantic processing; an unsupported major
version fails with `VERSION_UNSUPPORTED` ([REQ-AKP-VER-001]). An unknown
critical extension fails closed with `CRITICAL_EXTENSION_UNKNOWN` before any
payload processing.

## 3. HTTP mapping (reference implementation)

Per ADR-0003: request/response operations map to HTTP POST with JSON
envelope bodies; watch/subscription streams map to Server-Sent Events with
the AKP continuation cursor as the SSE resume position ([REQ-AKP-CONT-001],
`WATCH_CURSOR_STALE` on compaction). HTTP status codes are transport-level
only; the authoritative outcome is the enveloped AKP result and its
registered error code. Implementations MUST NOT infer effect success from a
2xx transport response ([REQ-GW-002] analog at the protocol layer).

## 4. Channel separation

Task-channel and management-channel operations use disjoint endpoint roots,
credentials, and session material ([REQ-AKP-SHELL-001..003],
[REQ-AKP-MGMT-001..003]; vector `shell-channel-isolation-003.json`).
Management operations additionally bind a PrivilegedManagementSession
(`authn-authz-capability.md` section 5). A message authenticated for one
channel presented on the other fails with
`SHELL_CHANNEL_BINDING_MISMATCH`.

## 5. Streaming and continuation

Streams deliver at-least-once with consumer-side dedup by sequence
([REQ-AKP-STR-001..002]). A continuation token encodes stream identity plus
position and is scope-checked on resume; it is not a bearer capability
([REQ-AKP-CONT-001], [REQ-AKP-SEC-001]).

## 6. Compliance checks

Layer 1 vectors (`schema-version-001.json`, `spec-contract-coverage.json`,
`cim-calibration-mismatch.json` for profile-specific negotiation) plus the
Shell/AKP vectors (`shell-watch-resume-006.json`,
`remote-completed-not-acceptance.json`, `shell-cancel-semantics-005.json`)
must execute with evidence at M5. Transport-only tests never substitute for
envelope semantics tests.
