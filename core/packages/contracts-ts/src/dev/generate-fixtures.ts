/**
 * Golden fixture generator. Prints one complete fixture document to stdout;
 * writing to `tests/golden/` is a deliberate, reviewed act:
 *
 *   node packages/contracts-ts/dist/dev/generate-fixtures.js canonical \
 *     > tests/golden/canonical-json-fixtures.json
 *   node packages/contracts-ts/dist/dev/generate-fixtures.js projection \
 *     > tests/golden/digest-and-projection-fixtures.json
 *
 * Fixture INPUTS are defined here; expected values are computed by this
 * implementation and independently re-verified by the Rust twin
 * (`crates/cognitive-contracts/tests/golden_fixtures.rs`,
 * `tests/projection_fixtures.rs`), which is what makes the fixtures a
 * cross-language gate rather than a self-test. Never edit fixture JSON by
 * hand.
 */

import {
  MEDIA_TYPE_SCHEMA_JSON,
  SCHEMA_BUNDLE_DOMAIN,
  SPEC_SET_DOMAIN,
  SPEC_SUITE_VERSION,
  manifestCanonicalBytes,
  manifestDigest,
  type BundleAsset,
} from "../bundle.js";
import { canonicalize, digest, signatureInput } from "../canonical.js";
import { projectionCanonicalBytes, projectionDigest } from "../projection.js";

const DIGEST_DOMAIN = "conformance-fixture/0.1";

interface PositiveSpec {
  id: string;
  description: string;
  input_json: string;
  signature?: { domain: string; algorithm: string };
}

const positiveSpecs: PositiveSpec[] = [
  {
    id: "empty-object",
    description: "Smallest object; digest exercises domain separation prefix",
    input_json: "{}",
  },
  {
    id: "key-ordering-utf16",
    description: "JCS member ordering by UTF-16 code units (RFC 8785 3.2.3 style example)",
    input_json:
      '{"\\u20ac":"Euro Sign","\\r":"Carriage Return","1":"One","\\ud83d\\ude00":"Emoji: Grinning Face","\\u0080":"Control","\\u00f6":"Latin Small Letter O With Diaeresis"}',
  },
  {
    id: "string-escaping",
    description: "JCS string escaping: quotes, backslash, solidus, control characters",
    input_json: '{"a\\"b":"c\\\\d","tab":"\\t","newline":"\\n","control":"\\u0001","slash":"/"}',
  },
  {
    id: "non-ascii-unicode",
    description: "CJK, emoji surrogate pair and combining mark survive untouched",
    input_json: '{"cjk":"\u5bd2\u6c34\u9a8f\u6cb3","emoji":"\ud83d\ude00","combining":"e\\u0301"}',
  },
  {
    id: "normalization-distinction",
    description: "NFC and NFD spellings stay distinct: no Unicode normalization",
    input_json: '{"nfc":"\\u00e9","nfd":"e\\u0301"}',
  },
  {
    id: "numbers-integers-and-fractions",
    description: "I-JSON safe integers, fractions and exponent forms (ECMAScript ToString)",
    input_json:
      "[0,1,-1,100000000,9007199254740991,-9007199254740991,0.1,0.5,1e21,1e-9,3.141592653589793,1e2]",
  },
  {
    id: "negative-zero",
    description: "Negative zero serializes as 0 (standard section 5)",
    input_json: '{"z":-0}',
  },
  {
    id: "timestamp-fraction-strings",
    description: "Canonical RFC 3339 UTC timestamp strings pass through byte-exactly",
    input_json: '{"t1":"2026-07-19T11:02:03Z","t2":"2026-07-19T11:02:03.1234Z"}',
  },
  {
    id: "null-vs-missing",
    description: "Explicit null is preserved; a missing member simply does not appear",
    input_json: '{"present":null}',
  },
  {
    id: "nested-values",
    description: "Nested objects/arrays with mixed types and reordered keys",
    input_json: '{"b":[{"y":2,"x":1},[],[false,null,""]],"a":{"nested":{"deep":true}}}',
  },
  {
    id: "whitespace-insensitivity",
    description: "Source whitespace and key order do not survive canonicalization",
    input_json: '  {\n  "b" :\t2 ,\n  "a" : [ 1 ,\n 2 ] }  ',
  },
  {
    id: "signature-preimage",
    description: "Signature input preimage with domain and algorithm separation",
    input_json: '{"action":"approve","proposal":"mp-1"}',
    signature: { domain: "akp-payload/0.2", algorithm: "ed25519" },
  },
];

interface NegativeSpec {
  id: string;
  description: string;
  input_json?: string;
  input_bytes_hex?: string;
  expected_rejection: string;
}

const negativeSpecs: NegativeSpec[] = [
  {
    id: "duplicate-member-names",
    description: "Same member name twice is rejected, not last-write-wins",
    input_json: '{"a":1,"a":2}',
    expected_rejection: "duplicate-member-name",
  },
  {
    id: "byte-order-mark",
    description: "UTF-8 BOM before the document is rejected",
    input_bytes_hex: "efbbbf7b7d",
    expected_rejection: "bom",
  },
  {
    id: "invalid-utf8-bytes",
    description: "Non-UTF-8 input bytes are rejected before parsing",
    input_bytes_hex: "22ff22",
    expected_rejection: "invalid-utf8",
  },
  {
    id: "unsafe-integer-literal",
    description: "Exact integer 2^53 exceeds the I-JSON safe range",
    input_json: '{"n":9007199254740992}',
    expected_rejection: "unsafe-integer",
  },
  {
    id: "nan-literal",
    description: "NaN is not JSON",
    input_json: '{"n":NaN}',
    expected_rejection: "invalid-json",
  },
  {
    id: "infinity-literal",
    description: "Infinity is not JSON (section 5 non-finite rejection)",
    input_json: '{"n":Infinity}',
    expected_rejection: "invalid-json",
  },
  {
    id: "lone-surrogate-escape",
    description: "Unpaired surrogate escape cannot form well-formed Unicode",
    input_json: '"\\ud800"',
    expected_rejection: "invalid-json",
  },
  {
    id: "unescaped-control-character",
    description: "Raw control character inside a string is invalid JSON",
    input_bytes_hex: "2261016222",
    expected_rejection: "invalid-json",
  },
  {
    id: "trailing-content",
    description: "Content after the top-level value is rejected",
    input_json: "{} x",
    expected_rejection: "invalid-json",
  },
];

const text = new TextDecoder();
const hex = (bytes: Uint8Array): string => Buffer.from(bytes).toString("hex");

const positive = positiveSpecs.map((spec) => {
  const canonical = canonicalize(spec.input_json);
  const entry: Record<string, unknown> = {
    id: spec.id,
    description: spec.description,
    input_json: spec.input_json,
    expected_canonical_text: text.decode(canonical),
    expected_canonical_utf8_hex: hex(canonical),
    expected_digest: digest(canonical, DIGEST_DOMAIN),
  };
  if (spec.signature) {
    entry.signature = {
      domain: spec.signature.domain,
      algorithm: spec.signature.algorithm,
      expected_signature_input_hex: hex(
        signatureInput(canonical, spec.signature.domain, spec.signature.algorithm),
      ),
    };
  }
  return entry;
});

const canonicalDoc = {
  fixture_set: "CognitiveOS canonical JSON golden fixtures",
  version: "0.2.0",
  encoding_profile: "cognitiveos.canonical-json/0.1",
  digest_domain: DIGEST_DOMAIN,
  generator: "packages/contracts-ts/src/dev/generate-fixtures.ts (verify with Rust twin before committing)",
  positive,
  negative: negativeSpecs,
};

// ---------------------------------------------------------------------------
// Digest projection / set manifest / validation fixtures (standard sections
// 6, 8, 10, 13; remaining section-14 coverage items).
// ---------------------------------------------------------------------------

const GOVERNED_DOMAIN = "governed-object-content/0.1";

/** A self-consistent governed object: content digest embedded after computing
 * the projection digest (excluded paths: own digest + signature). */
function selfConsistentObject(): Record<string, unknown> {
  const object: Record<string, unknown> = {
    header: {
      id: "01890a5d-ac96-774b-bcce-b302099a8070",
      type: "MemoryObject",
      object_version: 2,
      content_digest: "sha256:" + "0".repeat(64),
    },
    body: { text: "governed content", tags: ["a", "b"] },
    signature: "detached-signature-placeholder",
  };
  const excluded = ["/header/content_digest", "/signature"];
  const computed = projectionDigest(object, excluded, GOVERNED_DOMAIN);
  (object.header as Record<string, unknown>).content_digest = computed;
  return object;
}

function projectionPositives(): unknown[] {
  const excluded = ["/header/content_digest", "/signature"];
  const object = selfConsistentObject();
  const bytes = projectionCanonicalBytes(object, excluded);
  const entries: unknown[] = [
    {
      id: "projection-self-fields-excluded",
      description:
        "Digest projection removes exactly the declared self digest and signature paths; the embedded content digest verifies against the projection",
      object,
      digest_excluded: excluded,
      domain: GOVERNED_DOMAIN,
      digest_pointer: "/header/content_digest",
      expected_projection_canonical_text: text.decode(bytes),
      expected_projection_canonical_utf8_hex: hex(bytes),
      expected_digest: projectionDigest(object, excluded, GOVERNED_DOMAIN),
      expected_verification: "accept",
    },
  ];
  const escaped = { "a/b": { content_digest: "sha256:" + "1".repeat(64) }, kept: 1 };
  const escapedExcluded = ["/a~1b/content_digest"];
  const escapedBytes = projectionCanonicalBytes(escaped, escapedExcluded);
  entries.push({
    id: "projection-pointer-escaping",
    description: "JSON Pointer ~1 escaping addresses a member name containing a solidus",
    object: escaped,
    digest_excluded: escapedExcluded,
    domain: GOVERNED_DOMAIN,
    expected_projection_canonical_text: text.decode(escapedBytes),
    expected_projection_canonical_utf8_hex: hex(escapedBytes),
    expected_digest: projectionDigest(escaped, escapedExcluded, GOVERNED_DOMAIN),
  });
  const absent = { header: { id: "x" }, body: 1 };
  const absentExcluded = ["/header/content_digest", "/signature"];
  const absentBytes = projectionCanonicalBytes(absent, absentExcluded);
  entries.push({
    id: "projection-absent-excluded-path-noop",
    description:
      "A declared excluded path that is absent removes nothing (signature excluded `if present`)",
    object: absent,
    digest_excluded: absentExcluded,
    domain: GOVERNED_DOMAIN,
    expected_projection_canonical_text: text.decode(absentBytes),
    expected_projection_canonical_utf8_hex: hex(absentBytes),
    expected_digest: projectionDigest(absent, absentExcluded, GOVERNED_DOMAIN),
  });
  return entries;
}

function projectionNegatives(): unknown[] {
  const excluded = ["/header/content_digest", "/signature"];
  const object = selfConsistentObject();

  // Wrong self-field INCLUSION: digest computed without removing the
  // declared self digest path.
  const inclusion = JSON.parse(JSON.stringify(object)) as Record<string, unknown>;
  (inclusion.header as Record<string, unknown>).content_digest = projectionDigest(
    { ...object, header: { ...(object.header as Record<string, unknown>) } },
    ["/signature"],
    GOVERNED_DOMAIN,
  );

  // Wrong self-field EXCLUSION: digest computed with an undeclared path
  // additionally removed.
  const exclusion = JSON.parse(JSON.stringify(object)) as Record<string, unknown>;
  (exclusion.header as Record<string, unknown>).content_digest = projectionDigest(
    object,
    [...excluded, "/body/tags"],
    GOVERNED_DOMAIN,
  );

  // Wrong domain: digest computed under a different registered domain.
  const wrongDomain = JSON.parse(JSON.stringify(object)) as Record<string, unknown>;
  (wrongDomain.header as Record<string, unknown>).content_digest = projectionDigest(
    object,
    excluded,
    "conformance-fixture/0.1",
  );

  // Inserted defaults: digest computed over the object WITH a schema-default
  // member inserted; verification of the actual object must fail.
  const withDefault = JSON.parse(JSON.stringify(object)) as Record<string, unknown>;
  (withDefault.body as Record<string, unknown>).priority = "normal";
  const insertedDefaults = JSON.parse(JSON.stringify(object)) as Record<string, unknown>;
  (insertedDefaults.header as Record<string, unknown>).content_digest = projectionDigest(
    withDefault,
    excluded,
    GOVERNED_DOMAIN,
  );

  const shared = {
    digest_excluded: excluded,
    domain: GOVERNED_DOMAIN,
    digest_pointer: "/header/content_digest",
    expected_rejection: "digest-mismatch",
  };
  return [
    {
      id: "projection-wrong-self-field-inclusion",
      description:
        "Declared digest was computed WITH the self digest field included; verification fails closed",
      object: inclusion,
      ...shared,
    },
    {
      id: "projection-wrong-self-field-exclusion",
      description:
        "Declared digest was computed with an undeclared path additionally excluded; verification fails closed",
      object: exclusion,
      ...shared,
    },
    {
      id: "projection-wrong-domain",
      description: "Declared digest was computed under a different domain; verification fails closed",
      object: wrongDomain,
      ...shared,
    },
    {
      id: "projection-inserted-defaults",
      description:
        "Declared digest was computed after inserting a schema default; verifiers never insert defaults (section 7)",
      object: insertedDefaults,
      ...shared,
    },
    {
      id: "projection-missing-digest",
      description: "No digest value at the declared self-digest pointer",
      object: { header: { id: "x" }, body: 1 },
      digest_excluded: excluded,
      domain: GOVERNED_DOMAIN,
      digest_pointer: "/header/content_digest",
      expected_rejection: "missing-digest",
    },
  ];
}

function setManifestFixtures(): { positive: unknown[]; negative: unknown[] } {
  const assetA: BundleAsset = {
    id: "alpha.schema.json",
    version: SPEC_SUITE_VERSION,
    media_type: MEDIA_TYPE_SCHEMA_JSON,
    content_digest: "sha256:" + "a".repeat(64),
  };
  const assetB: BundleAsset = {
    id: "beta.schema.json",
    version: SPEC_SUITE_VERSION,
    media_type: MEDIA_TYPE_SCHEMA_JSON,
    content_digest: "sha256:" + "b".repeat(64),
  };
  // Given out of order: the manifest must sort deterministically.
  const unordered = [assetB, assetA];
  const bundleBytes = manifestCanonicalBytes(unordered);
  const positive: unknown[] = [
    {
      id: "set-manifest-schema-bundle",
      description:
        "Schema-bundle manifest: entries sorted by asset id, exact canonical bytes, domain schema-bundle/0.1",
      assets: unordered,
      domain: SCHEMA_BUNDLE_DOMAIN,
      expected_manifest_canonical_text: text.decode(bundleBytes),
      expected_manifest_canonical_utf8_hex: hex(bundleBytes),
      expected_digest: manifestDigest(unordered, SCHEMA_BUNDLE_DOMAIN),
    },
    {
      id: "set-manifest-spec-set-domain-separation",
      description: "The same assets under spec-set/0.1 produce a different digest (domain separation)",
      assets: unordered,
      domain: SPEC_SET_DOMAIN,
      expected_manifest_canonical_text: text.decode(bundleBytes),
      expected_manifest_canonical_utf8_hex: hex(bundleBytes),
      expected_digest: manifestDigest(unordered, SPEC_SET_DOMAIN),
    },
  ];
  const negative: unknown[] = [
    {
      id: "set-manifest-duplicate-asset-id",
      description: "Two assets sharing one logical id make the manifest ambiguous",
      assets: [assetA, { ...assetA, content_digest: "sha256:" + "c".repeat(64) }],
      domain: SCHEMA_BUNDLE_DOMAIN,
      expected_rejection: "duplicate-asset-id",
    },
    {
      id: "set-manifest-empty",
      description: "A manifest must cover at least one asset",
      assets: [],
      domain: SCHEMA_BUNDLE_DOMAIN,
      expected_rejection: "empty-bundle",
    },
  ];
  return { positive, negative };
}

function schemaDigestFixtures(): unknown[] {
  const schema = {
    $id: "tiny.schema.json",
    type: "object",
    required: ["id"],
    properties: { id: { type: "string" } },
    additionalProperties: false,
  };
  const good = digest(canonicalize(JSON.stringify(schema)), SCHEMA_BUNDLE_DOMAIN);
  const altered = digest(
    canonicalize(JSON.stringify({ ...schema, additionalProperties: true })),
    SCHEMA_BUNDLE_DOMAIN,
  );
  return [
    {
      id: "schema-digest-pinned",
      description: "Pinned schema digest matches the canonical bytes of the schema in use",
      schema,
      domain: SCHEMA_BUNDLE_DOMAIN,
      pinned_schema_digest: good,
      expected: "accept",
    },
    {
      id: "schema-digest-altered",
      description:
        "Pinned digest was taken from an ALTERED schema (rug pull); verification fails closed before use",
      schema,
      domain: SCHEMA_BUNDLE_DOMAIN,
      pinned_schema_digest: altered,
      expected: "reject",
      expected_rejection: "digest-mismatch",
    },
  ];
}

const projectionDoc = {
  fixture_set: "CognitiveOS digest projection, set manifest and validation golden fixtures",
  version: "0.1.0",
  encoding_profile: "cognitiveos.canonical-json/0.1",
  generator:
    "packages/contracts-ts/src/dev/generate-fixtures.ts projection (verify with Rust twin before committing)",
  projection_positive: projectionPositives(),
  projection_negative: projectionNegatives(),
  set_manifest_positive: setManifestFixtures().positive,
  set_manifest_negative: setManifestFixtures().negative,
  timestamp_positive: [
    "2026-07-19T11:02:03Z",
    "2026-07-19T11:02:03.1234Z",
    "0001-01-01T00:00:00Z",
    "2026-12-31T23:59:59.999999999Z",
  ],
  timestamp_negative: [
    { value: "2026-07-19T11:02:03+02:00", reason: "numeric offset" },
    { value: "2026-07-19T11:02:03-00:00", reason: "negative zero offset" },
    { value: "2026-07-19T11:02:03", reason: "local time without zone" },
    { value: "2026-07-19 11:02:03Z", reason: "space separator" },
    { value: "2026-07-19t11:02:03Z", reason: "lowercase t" },
    { value: "2026-07-19T11:02:03z", reason: "lowercase z" },
    { value: "2026-07-19T23:59:60Z", reason: "leap second" },
    { value: "2026-07-19T11:02:03.1230Z", reason: "trailing zero in fraction" },
    { value: "2026-07-19T11:02:03.000Z", reason: "zero fraction must be omitted" },
    { value: "2026-07-19T11:02:03.Z", reason: "empty fraction" },
    { value: "2026-07-19T11:02:03.1234567890Z", reason: "fraction longer than 9 digits" },
  ],
  digest_string_positive: ["sha256:" + "0123456789abcdef".repeat(4)],
  digest_string_negative: [
    { value: "SHA256:" + "a".repeat(64), reason: "uppercase algorithm label" },
    { value: "sha256:" + "A".repeat(64), reason: "uppercase hex" },
    { value: "sha256:" + "a".repeat(63), reason: "truncated hex" },
    { value: "sha256:" + "a".repeat(65), reason: "overlong hex" },
    { value: "sha512:" + "a".repeat(64), reason: "unregistered algorithm label" },
    { value: "a".repeat(64), reason: "missing algorithm label" },
  ],
  extension_positive: [
    {
      id: "extension-known-critical-and-unknown-noncritical",
      object: {
        extensions: [
          { id: "x-supported", critical: true },
          { id: "x-unknown-noncritical", critical: false },
        ],
        payload: { ok: true },
      },
      supported: ["x-supported"],
    },
  ],
  extension_negative: [
    {
      id: "extension-unknown-critical",
      description: "Unknown critical extension fails closed before payload processing",
      object: { extensions: [{ id: "x-unknown", critical: true }], payload: { ok: true } },
      supported: ["x-supported"],
      expected_rejection: "critical-extension-unknown",
    },
    {
      id: "extension-malformed-entry",
      description: "An unverifiable extension entry is treated as critical (fail closed)",
      object: { extensions: [{ id: "x-supported" }], payload: { ok: true } },
      supported: ["x-supported"],
      expected_rejection: "critical-extension-unknown",
    },
  ],
  schema_digest: schemaDigestFixtures(),
};

const which = process.argv[2] ?? "canonical";
const doc = which === "projection" ? projectionDoc : canonicalDoc;
process.stdout.write(JSON.stringify(doc, null, 2));
process.stdout.write("\n");
