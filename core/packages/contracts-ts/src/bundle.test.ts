/**
 * Bundle/set manifest digest tests (canonical-encoding-and-digest.md
 * section 13), twin of the unit tests in
 * `crates/cognitive-contracts/src/bundle.rs`.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  BundleError,
  MEDIA_TYPE_SCHEMA_JSON,
  SCHEMA_BUNDLE_DOMAIN,
  SPEC_SET_DOMAIN,
  SPEC_SUITE_VERSION,
  manifestCanonicalBytes,
  manifestDigest,
  type BundleAsset,
} from "./bundle.js";

const asset = (id: string, contentDigest: string): BundleAsset => ({
  id,
  version: SPEC_SUITE_VERSION,
  media_type: MEDIA_TYPE_SCHEMA_JSON,
  content_digest: contentDigest,
});

const d = (ch: string): string => `sha256:${ch.repeat(64)}`;

test("manifest digest is order-insensitive (deterministic sorted order)", () => {
  const forward = manifestDigest(
    [asset("a.schema.json", d("1")), asset("b.schema.json", d("2"))],
    SCHEMA_BUNDLE_DOMAIN,
  );
  const reversed = manifestDigest(
    [asset("b.schema.json", d("2")), asset("a.schema.json", d("1"))],
    SCHEMA_BUNDLE_DOMAIN,
  );
  assert.equal(forward, reversed);
});

test("duplicate asset ids and empty bundles are rejected", () => {
  assert.throws(
    () => manifestDigest([asset("x.schema.json", d("3")), asset("x.schema.json", d("3"))], SCHEMA_BUNDLE_DOMAIN),
    (err: unknown) => err instanceof BundleError && err.category === "duplicate-asset-id",
  );
  assert.throws(
    () => manifestDigest([], SCHEMA_BUNDLE_DOMAIN),
    (err: unknown) => err instanceof BundleError && err.category === "empty-bundle",
  );
});

test("schema-bundle and spec-set domains separate digests", () => {
  const assets = [asset("a.schema.json", d("4"))];
  assert.notEqual(manifestDigest(assets, SCHEMA_BUNDLE_DOMAIN), manifestDigest(assets, SPEC_SET_DOMAIN));
});

test("manifest canonical bytes match the Rust twin's exact shape", () => {
  const bytes = manifestCanonicalBytes([asset("a.schema.json", d("5"))]);
  const expected =
    `{"assets":[{"content_digest":"${d("5")}","id":"a.schema.json",` +
    `"media_type":"application/schema+json","version":"0.1.0-draft.1"}]}`;
  assert.equal(new TextDecoder().decode(bytes), expected);
});
