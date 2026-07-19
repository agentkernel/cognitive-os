/**
 * Golden fixture verification for digest projections, set manifests,
 * canonical timestamp / digest-string forms, and the critical-extension
 * gate — TypeScript side (the Rust twin re-verifies the same file byte for
 * byte: `crates/cognitive-contracts/tests/projection_fixtures.rs`).
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { BundleError, manifestCanonicalBytes, manifestDigest, type BundleAsset } from "./bundle.js";
import { canonicalize, digest } from "./canonical.js";
import {
  ProjectionError,
  assertNoUnknownCriticalExtensions,
  projectionCanonicalBytes,
  projectionDigest,
  validateCanonicalTimestamp,
  validateDigestString,
  verifyContentDigest,
} from "./projection.js";

const FIXTURE_PATH = fileURLToPath(
  new URL("../../../tests/golden/digest-and-projection-fixtures.json", import.meta.url),
);

interface ProjectionFixture {
  readonly id: string;
  readonly object: unknown;
  readonly digest_excluded: ReadonlyArray<string>;
  readonly domain: string;
  readonly digest_pointer?: string;
  readonly expected_projection_canonical_text?: string;
  readonly expected_projection_canonical_utf8_hex?: string;
  readonly expected_digest?: string;
  readonly expected_verification?: string;
  readonly expected_rejection?: string;
}

interface ManifestFixture {
  readonly id: string;
  readonly assets: ReadonlyArray<BundleAsset>;
  readonly domain: string;
  readonly expected_manifest_canonical_text?: string;
  readonly expected_manifest_canonical_utf8_hex?: string;
  readonly expected_digest?: string;
  readonly expected_rejection?: string;
}

interface ExtensionFixture {
  readonly id: string;
  readonly object: unknown;
  readonly supported: ReadonlyArray<string>;
  readonly expected_rejection?: string;
}

interface FixtureFile {
  readonly projection_positive: ReadonlyArray<ProjectionFixture>;
  readonly projection_negative: ReadonlyArray<ProjectionFixture>;
  readonly set_manifest_positive: ReadonlyArray<ManifestFixture>;
  readonly set_manifest_negative: ReadonlyArray<ManifestFixture>;
  readonly timestamp_positive: ReadonlyArray<string>;
  readonly timestamp_negative: ReadonlyArray<{ value: string; reason: string }>;
  readonly digest_string_positive: ReadonlyArray<string>;
  readonly digest_string_negative: ReadonlyArray<{ value: string; reason: string }>;
  readonly extension_positive: ReadonlyArray<ExtensionFixture>;
  readonly extension_negative: ReadonlyArray<ExtensionFixture>;
  readonly schema_digest: ReadonlyArray<{
    readonly id: string;
    readonly schema: unknown;
    readonly domain: string;
    readonly pinned_schema_digest: string;
    readonly expected: string;
  }>;
}

const fixtures = JSON.parse(readFileSync(FIXTURE_PATH, "utf-8")) as FixtureFile;
const hex = (bytes: Uint8Array): string => Buffer.from(bytes).toString("hex");

test("projection positives: canonical bytes, digests, verification", () => {
  assert.ok(fixtures.projection_positive.length >= 3, "projection coverage shrank");
  for (const fixture of fixtures.projection_positive) {
    const bytes = projectionCanonicalBytes(fixture.object, fixture.digest_excluded);
    assert.equal(
      new TextDecoder().decode(bytes),
      fixture.expected_projection_canonical_text,
      `projection text mismatch for ${fixture.id}`,
    );
    assert.equal(
      hex(bytes),
      fixture.expected_projection_canonical_utf8_hex,
      `projection hex mismatch for ${fixture.id}`,
    );
    assert.equal(
      projectionDigest(fixture.object, fixture.digest_excluded, fixture.domain),
      fixture.expected_digest,
      `projection digest mismatch for ${fixture.id}`,
    );
    if (fixture.expected_verification === "accept" && fixture.digest_pointer) {
      verifyContentDigest(fixture.object, fixture.digest_excluded, fixture.domain, fixture.digest_pointer);
    }
  }
});

test("projection negatives fail closed with the expected category", () => {
  assert.ok(fixtures.projection_negative.length >= 5, "projection negative coverage shrank");
  for (const fixture of fixtures.projection_negative) {
    assert.throws(
      () =>
        verifyContentDigest(
          fixture.object,
          fixture.digest_excluded,
          fixture.domain,
          fixture.digest_pointer ?? "/",
        ),
      (err: unknown) =>
        err instanceof ProjectionError && err.category === fixture.expected_rejection,
      `category mismatch for ${fixture.id}`,
    );
  }
});

test("set manifest fixtures: exact bytes, digests, rejections", () => {
  for (const fixture of fixtures.set_manifest_positive) {
    const bytes = manifestCanonicalBytes(fixture.assets);
    assert.equal(
      new TextDecoder().decode(bytes),
      fixture.expected_manifest_canonical_text,
      `manifest text mismatch for ${fixture.id}`,
    );
    assert.equal(
      hex(bytes),
      fixture.expected_manifest_canonical_utf8_hex,
      `manifest hex mismatch for ${fixture.id}`,
    );
    assert.equal(
      manifestDigest(fixture.assets, fixture.domain),
      fixture.expected_digest,
      `manifest digest mismatch for ${fixture.id}`,
    );
  }
  for (const fixture of fixtures.set_manifest_negative) {
    assert.throws(
      () => manifestDigest(fixture.assets, fixture.domain),
      (err: unknown) => err instanceof BundleError && err.category === fixture.expected_rejection,
      `category mismatch for ${fixture.id}`,
    );
  }
});

test("canonical timestamp and digest-string forms", () => {
  for (const value of fixtures.timestamp_positive) {
    validateCanonicalTimestamp(value);
  }
  for (const fixture of fixtures.timestamp_negative) {
    assert.throws(
      () => validateCanonicalTimestamp(fixture.value),
      (err: unknown) => err instanceof ProjectionError && err.category === "invalid-timestamp",
      `${fixture.value} must be rejected (${fixture.reason})`,
    );
  }
  for (const value of fixtures.digest_string_positive) {
    validateDigestString(value);
  }
  for (const fixture of fixtures.digest_string_negative) {
    assert.throws(
      () => validateDigestString(fixture.value),
      (err: unknown) => err instanceof ProjectionError && err.category === "invalid-digest",
      `${fixture.value} must be rejected (${fixture.reason})`,
    );
  }
});

test("unknown critical extensions fail closed before payload processing", () => {
  for (const fixture of fixtures.extension_positive) {
    assertNoUnknownCriticalExtensions(fixture.object, fixture.supported);
  }
  for (const fixture of fixtures.extension_negative) {
    assert.throws(
      () => assertNoUnknownCriticalExtensions(fixture.object, fixture.supported),
      (err: unknown) =>
        err instanceof ProjectionError && err.category === fixture.expected_rejection,
      `category mismatch for ${fixture.id}`,
    );
  }
});

test("pinned schema digests verify; altered pins fail closed", () => {
  for (const fixture of fixtures.schema_digest) {
    const computed = digest(canonicalize(JSON.stringify(fixture.schema)), fixture.domain);
    const accepted = computed === fixture.pinned_schema_digest;
    if (fixture.expected === "accept") {
      assert.ok(accepted, `${fixture.id}: pinned schema digest must verify`);
    } else {
      assert.ok(!accepted, `${fixture.id}: altered pinned schema digest must fail closed`);
    }
  }
});
