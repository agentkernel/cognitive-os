/**
 * Emit `{fixture id -> digest}` for every positive golden fixture as
 * canonical JSON on stdout. CI runs this and the Rust twin
 * (`cargo run -p cognitive-contracts --example emit_golden`) and asserts the
 * outputs are byte-identical (cross-language digest equality gate).
 */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { canonicalize, digest } from "./canonical.js";

interface PositiveFixture {
  readonly id: string;
  readonly input_json: string;
}

interface FixtureFile {
  readonly digest_domain: string;
  readonly positive: ReadonlyArray<PositiveFixture>;
}

const fixturePath = fileURLToPath(
  new URL("../../../tests/golden/canonical-json-fixtures.json", import.meta.url),
);
const fixtures = JSON.parse(readFileSync(fixturePath, "utf-8")) as FixtureFile;

const entries = fixtures.positive
  .map((fixture) => [fixture.id, digest(canonicalize(fixture.input_json), fixtures.digest_domain)] as const)
  .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));

const doc = JSON.stringify(Object.fromEntries(entries));
// Re-canonicalize so both languages emit byte-identical output.
process.stdout.write(new TextDecoder().decode(canonicalize(doc)));
process.stdout.write("\n");
