/**
 * Emit `{fixture id -> digest}` for every positive golden fixture, plus the
 * live schema-bundle manifest digest of `specs/schemas/`, as canonical JSON
 * on stdout. CI runs this and the Rust twin
 * (`cargo run -p cognitive-contracts --example emit_golden`) and asserts the
 * outputs are byte-identical (cross-language digest equality gate).
 */

import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  MEDIA_TYPE_SCHEMA_JSON,
  SCHEMA_BUNDLE_DOMAIN,
  SPEC_SUITE_VERSION,
  assetContentDigest,
  manifestDigest,
  type BundleAsset,
} from "./bundle.js";
import { canonicalize, digest } from "./canonical.js";

interface PositiveFixture {
  readonly id: string;
  readonly input_json: string;
}

interface FixtureFile {
  readonly digest_domain: string;
  readonly positive: ReadonlyArray<PositiveFixture>;
}

const REPO_ROOT = fileURLToPath(new URL("../../..", import.meta.url));

/**
 * Live schema-bundle manifest digest over the current specs/schemas suite
 * (registered section-13 procedure; twin logic in emit_golden.rs).
 */
function liveSchemaBundleDigest(): string {
  const dir = path.join(REPO_ROOT, "specs", "schemas");
  const assets: BundleAsset[] = readdirSync(dir)
    .filter((name) => name.endsWith(".json"))
    .sort()
    .map((name) => ({
      id: name,
      version: SPEC_SUITE_VERSION,
      media_type: MEDIA_TYPE_SCHEMA_JSON,
      content_digest: assetContentDigest(
        JSON.parse(readFileSync(path.join(dir, name), "utf-8")),
        SCHEMA_BUNDLE_DOMAIN,
      ),
    }));
  return manifestDigest(assets, SCHEMA_BUNDLE_DOMAIN);
}

const fixturePath = path.join(REPO_ROOT, "tests", "golden", "canonical-json-fixtures.json");
const fixtures = JSON.parse(readFileSync(fixturePath, "utf-8")) as FixtureFile;

const entries = fixtures.positive
  .map((fixture) => [fixture.id, digest(canonicalize(fixture.input_json), fixtures.digest_domain)] as const)
  .concat([["live:schema-bundle-manifest", liveSchemaBundleDigest()] as const])
  .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));

const doc = JSON.stringify(Object.fromEntries(entries));
// Re-canonicalize so both languages emit byte-identical output.
process.stdout.write(new TextDecoder().decode(canonicalize(doc)));
process.stdout.write("\n");
