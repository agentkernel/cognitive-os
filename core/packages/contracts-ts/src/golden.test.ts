/**
 * Golden fixture verification (docs/standards/canonical-encoding-and-digest.md
 * section 14). The same fixture file is verified byte-for-byte by the Rust
 * twin in `crates/cognitive-contracts/tests/golden_fixtures.rs`; CI
 * additionally diffs the emitted digest maps of both implementations.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  CanonicalError,
  canonicalize,
  digest,
  parseStrict,
  parseStrictBytes,
  signatureInput,
} from "./canonical.js";

interface PositiveFixture {
  readonly id: string;
  readonly description: string;
  readonly input_json: string;
  readonly expected_canonical_text: string;
  readonly expected_digest: string;
  readonly signature?: {
    readonly domain: string;
    readonly algorithm: string;
    readonly expected_signature_input_hex: string;
  };
}

interface NegativeFixture {
  readonly id: string;
  readonly description: string;
  readonly input_json?: string;
  readonly input_bytes_hex?: string;
  readonly expected_rejection: string;
}

interface FixtureFile {
  readonly fixture_set: string;
  readonly version: string;
  readonly encoding_profile: string;
  readonly digest_domain: string;
  readonly positive: ReadonlyArray<PositiveFixture>;
  readonly negative: ReadonlyArray<NegativeFixture>;
}

export const FIXTURE_PATH = fileURLToPath(
  new URL("../../../tests/golden/canonical-json-fixtures.json", import.meta.url),
);

export function loadFixtures(): FixtureFile {
  return JSON.parse(readFileSync(FIXTURE_PATH, "utf-8")) as FixtureFile;
}

const hex = (bytes: Uint8Array): string => Buffer.from(bytes).toString("hex");

test("golden fixtures: positive cases produce identical canonical bytes and digests", () => {
  const fixtures = loadFixtures();
  assert.equal(fixtures.encoding_profile, "cognitiveos.canonical-json/0.1");
  assert.ok(fixtures.positive.length >= 10, "fixture coverage shrank");
  for (const fixture of fixtures.positive) {
    const canonical = canonicalize(fixture.input_json);
    assert.equal(
      new TextDecoder().decode(canonical),
      fixture.expected_canonical_text,
      `canonical text mismatch for ${fixture.id}`,
    );
    assert.equal(
      digest(canonical, fixtures.digest_domain),
      fixture.expected_digest,
      `digest mismatch for ${fixture.id}`,
    );
    if (fixture.signature) {
      assert.equal(
        hex(signatureInput(canonical, fixture.signature.domain, fixture.signature.algorithm)),
        fixture.signature.expected_signature_input_hex,
        `signature input mismatch for ${fixture.id}`,
      );
    }
  }
});

test("golden fixtures: negative cases are rejected with the expected category", () => {
  const fixtures = loadFixtures();
  assert.ok(fixtures.negative.length >= 6, "fixture coverage shrank");
  for (const fixture of fixtures.negative) {
    let caught: CanonicalError | undefined;
    try {
      if (fixture.input_bytes_hex !== undefined) {
        parseStrictBytes(Uint8Array.from(Buffer.from(fixture.input_bytes_hex, "hex")));
      } else if (fixture.input_json !== undefined) {
        parseStrict(fixture.input_json);
      } else {
        assert.fail(`fixture ${fixture.id} has no input`);
      }
    } catch (err) {
      if (!(err instanceof CanonicalError)) {
        throw err;
      }
      caught = err;
    }
    assert.ok(caught, `fixture ${fixture.id} was not rejected`);
    assert.equal(caught.category, fixture.expected_rejection, `category mismatch for ${fixture.id}`);
  }
});
