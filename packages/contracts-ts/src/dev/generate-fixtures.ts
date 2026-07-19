/**
 * Golden fixture generator. Prints the complete fixture document to stdout;
 * writing to `tests/golden/canonical-json-fixtures.json` is a deliberate,
 * reviewed act:
 *
 *   node packages/contracts-ts/dist/dev/generate-fixtures.js > tests/golden/canonical-json-fixtures.json
 *
 * Fixture INPUTS are defined here; expected values are computed by this
 * implementation and independently re-verified by the Rust twin
 * (`crates/cognitive-contracts/tests/golden_fixtures.rs`), which is what
 * makes the fixtures a cross-language gate rather than a self-test.
 * Never edit the fixture JSON by hand.
 */

import { canonicalize, digest, signatureInput } from "../canonical.js";

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

const doc = {
  fixture_set: "CognitiveOS canonical JSON golden fixtures",
  version: "0.1.0",
  encoding_profile: "cognitiveos.canonical-json/0.1",
  digest_domain: DIGEST_DOMAIN,
  generator: "packages/contracts-ts/src/dev/generate-fixtures.ts (verify with Rust twin before committing)",
  positive,
  negative: negativeSpecs,
};

process.stdout.write(JSON.stringify(doc, null, 2));
process.stdout.write("\n");
