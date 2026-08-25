import assert from "node:assert/strict";
import { test } from "node:test";

import {
  CanonicalError,
  canonicalize,
  digest,
  digestJson,
  parseStrict,
  parseStrictBytes,
} from "./canonical.js";

const category = (fn: () => unknown): string => {
  try {
    fn();
  } catch (err) {
    if (err instanceof CanonicalError) {
      return err.category;
    }
    throw err;
  }
  throw new Error("expected a CanonicalError");
};

test("rejects duplicate member names", () => {
  assert.equal(category(() => parseStrict('{"a":1,"a":2}')), "duplicate-member-name");
});

test("rejects unsafe integers, keeps safe bounds", () => {
  assert.equal(category(() => parseStrict("9007199254740993")), "unsafe-integer");
  assert.equal(parseStrict("9007199254740991"), 9007199254740991);
  assert.equal(parseStrict("-9007199254740991"), -9007199254740991);
});

test("rejects BOM and invalid UTF-8", () => {
  assert.equal(category(() => parseStrict("\uFEFF{}")), "bom");
  assert.equal(category(() => parseStrictBytes(Uint8Array.of(0xff, 0xfe))), "invalid-utf8");
});

test("rejects unpaired surrogate escapes", () => {
  assert.equal(category(() => parseStrict('"\\ud800"')), "invalid-json");
});

test("canonical key ordering and unicode", () => {
  const bytes = canonicalize('{"b":2,"a":1,"\\u20ac":"euro"}');
  assert.equal(new TextDecoder().decode(bytes), '{"a":1,"b":2,"\u20ac":"euro"}');
});

test("negative zero canonicalizes to 0", () => {
  assert.equal(new TextDecoder().decode(canonicalize("[-0]")), "[0]");
});

test("digest uses domain separation", () => {
  const canonical = canonicalize("{}");
  const d1 = digest(canonical, "conformance-fixture/0.1");
  const d2 = digest(canonical, "schema-bundle/0.1");
  assert.notEqual(d1, d2);
  assert.match(d1, /^sha256:[0-9a-f]{64}$/);
});

test("forbidden domains rejected", () => {
  for (const domain of ["", "generic", "object", "payload", "UPPER", "-lead"]) {
    assert.equal(
      category(() => digestJson("{}", domain)),
      "invalid-domain",
      `domain ${JSON.stringify(domain)} must be rejected`,
    );
  }
});
