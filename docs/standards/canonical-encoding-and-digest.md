# Canonical Encoding and Digest Standard

- Standard ID: `cognitiveos.standard.canonical-encoding-digest/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-19
- Encoding profile: `cognitiveos.canonical-json/0.1`

## 1. Scope and normative language

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are interpreted according to RFC 2119 and RFC 8174 when written in uppercase.

This standard defines interoperable canonical bytes and exact SHA-256 digest and signature inputs. It applies when a digest-pinned contract selects `cognitiveos.canonical-json/0.1`.

## 2. Required base standards

Canonical JSON MUST satisfy:

1. JSON encoded as UTF-8 without BOM.
2. I-JSON, RFC 7493.
3. RFC 8785 JSON Canonicalization Scheme (JCS).
4. The RFC 3339 timestamp restrictions below.

Canonical bytes are the exact RFC 8785 output after schema and profile validation. Pretty printing, whitespace, source key order, transport encoding, archive metadata, and line endings are not canonical input.

## 3. Validation before canonicalization

An implementation MUST parse the complete input, reject duplicate member names, validate against the selected digest-pinned schema, and reject non-I-JSON data before canonicalization.

Canonicalization MUST NOT repair invalid values. It MUST NOT guess a schema version, add a default, drop an unknown critical extension, coerce types, normalize Unicode, truncate numbers, or reinterpret local time.

Unknown non-critical fields are retained or rejected according to the selected schema and behavior contract. If retained, they participate in canonicalization and digest unless an explicit signed projection says otherwise.

## 4. UTF-8 and Unicode

Text MUST be well-formed Unicode encoded as shortest-form UTF-8. BOM, invalid UTF-8, overlong encoding, and unpaired surrogate are rejected.

The canonicalizer MUST NOT apply NFC, NFD, NFKC, or NFKD normalization. Visually equivalent and differently normalized strings remain distinct. A field contract MAY require normalization before object construction.

Member names are ordered exactly as RFC 8785 requires, by lexicographic UTF-16 code units. Escaping follows JCS.

## 5. Numbers

JSON numbers MUST be finite IEEE 754 binary64 values representable under I-JSON and RFC 8785. NaN, infinities, and overflow are rejected. JCS ECMAScript number serialization is authoritative for exponent, decimal point, and zero formatting.

Negative zero serializes as `0`. A domain needing to distinguish it MUST use a schema-defined string or structure.

Exact integers outside `[-9007199254740991, 9007199254740991]` MUST NOT use JSON number. Arbitrary precision integers, scale-sensitive decimals, money, and precision-sensitive measurements MUST use schema-defined decimal strings or structures such as `{value, scale}`.

A parser MUST NOT silently round a lexical number before a security decision or digest.

## 6. RFC 3339 UTC timestamps

A timestamp field MUST be RFC 3339 UTC with uppercase `T` and terminal uppercase `Z`. Numeric offsets, local time, lowercase `t`/`z`, leap second `:60`, and missing timezone are forbidden.

Canonical form is `YYYY-MM-DDTHH:MM:SS[.fraction]Z`. A fraction has 1 through 9 digits, trailing zeros are removed, and a zero fraction is omitted. Examples: `2026-07-19T11:02:03Z` and `2026-07-19T11:02:03.1234Z`.

Timestamps are wall/domain time. Monotonic durations and deadlines follow ADR-0005 and MUST NOT be inferred from wall-clock subtraction where monotonic correctness is required.

## 7. Missing, null, and defaults

Missing and `null` are distinct. Required members MUST be present. Optional members SHOULD be omitted when no value is asserted. `null` is allowed only when the schema includes it and behavior defines it.

Schema defaults are annotations. Canonicalizers and digest/signature verifiers MUST NOT insert defaults. If an application applies a default, it constructs and validates a new explicit object before canonicalization.

Empty string, empty array, empty object, zero, false, null, and missing MUST NOT be interchanged.

## 8. SHA-256 representation

This profile defines SHA-256 only. A formal machine digest MUST be:

`sha256:<64 lowercase hexadecimal digits>`

Uppercase hex, missing leading zeros, base64, shortened forms, or fewer than 64 hex digits are forbidden in machine fields. UI abbreviations MUST be labeled non-authoritative and retain the full digest.

No other algorithm may appear under the `sha256` label. Algorithm agility requires a new profile or an explicitly algorithm-tagged field.

## 9. Domain separation

Every digest and signature MUST use a versioned ASCII domain label registered by its contract. It MUST match `[a-z0-9][a-z0-9._/-]{0,127}`. Empty, `generic`, `object`, and `payload` are forbidden.

For canonical bytes `C` and domain `D`, the exact digest preimage is:

```text
ASCII("CognitiveOS-Digest-V1\n") || ASCII(D) || 0x00 || C
```

The digest is:

```text
"sha256:" || lowercase_hex(SHA-256(preimage))
```

The prefix is case-sensitive. `\n` is byte `0x0a`; `0x00` is one NUL byte. There is no length prefix, trailing newline, BOM, or transport metadata.

Examples: `schema-bundle/0.1`, `spec-set/0.1`, `akp-payload/0.2`, `governed-object-content/0.1`, and `conformance-fixture/0.1`.

## 10. Content digest exact input

Unless a more specific contract defines a projection, `content_digest` covers the complete schema-valid object after removing only JSON Pointer paths explicitly marked `digest_excluded` by that contract. Storage location, transport headers, and cache metadata are excluded only when declared outside the object.

Self-referential `content_digest` and `signature` fields MUST NOT be removed by convention. The contract MUST name excluded paths. If no exclusion exists and self-reference prevents construction, generation MUST fail.

Procedure:

1. Select digest-pinned schema and exact domain.
2. Parse and validate the source object.
3. Remove only declared excluded paths to form the digest projection.
4. Validate a projection schema if defined.
5. Produce RFC 8785 canonical bytes `C`.
6. Apply section 9.

Verification repeats the procedure from the received semantic value. Hashing received non-canonical source bytes is not content-digest verification.

## 11. Payload and artifact digests

An inline JSON `payload_digest` covers the canonical payload value only with the operation's exact payload domain. It does not cover the envelope unless a separate envelope digest is defined.

For `payload_ref`, the reference object and referenced content have separate domains. The envelope MUST identify which digest is present. Fetching requires reauthorization and referenced-content verification.

Binary artifacts are outside JCS. Their contract MUST define media type, exact bytes or normalization, and artifact-specific domain. Raw bytes MUST NOT be silently decoded and re-encoded before hashing.

## 12. Signature exact input

A signature contract MUST identify signature algorithm, key ID, signature domain, signed schema/projection, and excluded paths.

For canonical signed bytes `C`, domain `D`, and case-sensitive ASCII algorithm ID `A`, signature input is:

```text
ASCII("CognitiveOS-Signature-V1\n") || ASCII(D) || 0x00 || ASCII(A) || 0x00 || C
```

The selected algorithm signs these bytes according to its own standard. It MUST NOT sign a display digest, pretty JSON, transport bytes, or an implicitly reparsed object. An API's internal hashing remains part of that algorithm; application SHA-256 MUST NOT be added unless the signature profile requires it.

A detached signature SHOULD include `signature_profile`, `algorithm`, `key_id`, `signed_domain`, `signed_schema_digest`, `signed_content_digest`, and encoded signature bytes. The content digest is a cross-check, not a substitute for signature input.

## 13. Set and bundle digests

A specification set uses domain `spec-set/0.1` and covers its complete canonical logical manifest, excluding only its own digest and signatures when explicitly declared.

A schema bundle uses domain `schema-bundle/0.1` and covers a canonical manifest containing each logical asset ID, SemVer, media type, and full content digest in deterministic sorted order. It MUST NOT hash a variable archive representation.

Suites and fixture collections use the same manifest pattern with distinct versioned domains. Nested digests MUST be verified first.

## 14. Golden fixtures

An implementation MUST run language-neutral golden fixtures. Each fixture MUST provide fixture ID/version, encoding profile, schema digest, domain, lossless semantic input, expected canonical UTF-8 bytes as full hex or exact file, expected full digest, expected signature input where applicable, and expected result/rejection reason.

Positive fixtures MUST cover key ordering, escaping, non-ASCII Unicode, normalization-form distinction, integers and fractions, negative zero, timestamp fractions, missing versus null, nested values, digest projection, set manifests, and signature preimages.

Negative fixtures MUST cover duplicate keys, invalid UTF-8, BOM, unpaired surrogate, NaN/infinity, unsafe integer, offset/local/leap-second timestamps, inserted defaults, unknown critical extension before payload processing, wrong domain, uppercase/truncated digest, altered schema digest, and wrong self-field inclusion/exclusion.

Rust and TypeScript claims MUST produce byte-identical canonical bytes, preimages, full digests, and signature inputs, and identical acceptance outcomes.

## 15. Failure behavior

Canonicalization, digest, or signature failure MUST fail closed before authorization, transition, dispatch, commit, or acceptance based on the object. A mismatch MUST NOT be repaired in place. The implementation SHOULD return `SCHEMA_MISMATCH`, `DIGEST_MISMATCH`, or `CRITICAL_EXTENSION_UNKNOWN` as applicable and audit profile, schema digest, domain, and stage without exposing secret content.
