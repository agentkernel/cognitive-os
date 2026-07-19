/**
 * Contract-layer schema re-verification (F-003 closure evidence, TS side).
 * Twin of `crates/cognitive-contracts/tests/schema_contract.rs`:
 *
 * 1. every schema under `specs/schemas/` compiles under draft 2020-12 with
 *    all relative `$ref`s resolvable;
 * 2. the migrated single-track contracts REJECT the legacy
 *    `common-defs.schema.json#/$defs/{metadata,strongRef}` dual-track shapes
 *    (REQ-GOBJ-HEADER-001, REQ-GOBJ-REF-001, REQ-GOBJ-MIG-001), using the
 *    exact instances pinned by the negative vectors
 *    `conformance/vectors/governed-object-legacy-{metadata,strongref}-001.json`;
 * 3. a migrated positive instance is accepted.
 *
 * This is NOT vector execution (no expected-outcome comparison engine, no
 * result reporting); vector result states remain `not-run` until the
 * Lane-CFR runner executes them (docs/standards/conformance-evidence.md).
 */

import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { Ajv2020 } from "ajv/dist/2020.js";
import addFormatsImport from "ajv-formats";

// ajv-formats ships CJS whose type surface under NodeNext resolves to the
// module namespace; at runtime the callable plugin is the default export.
const addFormats = addFormatsImport as unknown as (ajv: Ajv2020) => Ajv2020;

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const SCHEMA_DIR = path.join(REPO_ROOT, "specs", "schemas");
const VECTOR_DIR = path.join(REPO_ROOT, "conformance", "vectors");

interface SchemaDoc {
  readonly name: string;
  readonly doc: Record<string, unknown>;
}

function loadSchemas(): SchemaDoc[] {
  return readdirSync(SCHEMA_DIR)
    .filter((name) => name.endsWith(".json"))
    .sort()
    .map((name) => ({
      name,
      doc: JSON.parse(readFileSync(path.join(SCHEMA_DIR, name), "utf-8")) as Record<
        string,
        unknown
      >,
    }));
}

/**
 * Register every schema under its file name as retrieval URI so each
 * relative `$ref` resolves from the containing schema file
 * (`conformance/README.md` convention; schema `$id` policy D-001/D-006).
 */
function buildAjv(schemas: SchemaDoc[]): Ajv2020 {
  const ajv = new Ajv2020({ strict: false, allErrors: true, validateFormats: true });
  addFormats(ajv);
  for (const schema of schemas) {
    const { $id: _ignored, ...withoutId } = schema.doc;
    ajv.addSchema(withoutId, schema.name);
  }
  return ajv;
}

function vectorObject(file: string): unknown {
  const vector = JSON.parse(readFileSync(path.join(VECTOR_DIR, file), "utf-8")) as {
    input?: { object?: unknown };
  };
  assert.ok(vector.input?.object, `${file} has no input.object`);
  return vector.input.object;
}

test("every schema compiles under draft 2020-12 with resolvable relative $refs", () => {
  const schemas = loadSchemas();
  assert.ok(schemas.length >= 56, `schema suite shrank: ${schemas.length}`);
  const ajv = buildAjv(schemas);
  for (const schema of schemas) {
    const validate = ajv.getSchema(schema.name);
    assert.ok(validate, `schema ${schema.name} failed to compile`);
  }
});

test("legacy metadata envelope is rejected by the single-track Effect contract", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("effect.schema.json");
  assert.ok(validate);
  const object = vectorObject("governed-object-legacy-metadata-001.json");
  assert.equal(
    validate(object),
    false,
    "legacy common-defs metadata envelope must be rejected (REQ-GOBJ-HEADER-001, REQ-GOBJ-MIG-001)",
  );
});

test("legacy strongRef shape is rejected where a strong ObjectReference is required", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("effect.schema.json");
  assert.ok(validate);
  const object = vectorObject("governed-object-legacy-strongref-001.json");
  assert.equal(
    validate(object),
    false,
    "legacy common-defs strongRef shape must be rejected (REQ-GOBJ-REF-001, REQ-GOBJ-MIG-001)",
  );
});

test("migrated positive Effect instance is accepted", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("effect.schema.json");
  assert.ok(validate);
  const object = vectorObject("governed-object-legacy-strongref-001.json") as Record<
    string,
    unknown
  >;
  object["intent_ref"] = {
    kind: "strong",
    id: "01890a5d-ac96-774b-bcce-b302099a805d",
    object_version: 1,
    content_digest: "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
  };
  assert.equal(
    validate(object),
    true,
    `migrated Effect instance must validate: ${JSON.stringify(validate.errors)}`,
  );
});

test("legacy $defs stay deprecated and unreferenced (F-003 retention decision)", () => {
  const schemas = loadSchemas();
  const common = schemas.find((s) => s.name === "common-defs.schema.json");
  assert.ok(common);
  const defs = common.doc["$defs"] as Record<string, { deprecated?: boolean }>;
  for (const def of ["metadata", "strongRef"]) {
    assert.equal(defs[def]?.deprecated, true, `common-defs $defs/${def} must stay deprecated`);
  }
  for (const schema of schemas) {
    if (schema.name === "common-defs.schema.json") {
      continue;
    }
    const raw = JSON.stringify(schema.doc);
    for (const banned of [
      "common-defs.schema.json#/$defs/metadata",
      "common-defs.schema.json#/$defs/strongRef",
    ]) {
      assert.ok(
        !raw.includes(banned),
        `${schema.name} references legacy shape ${banned} (dual-track ban, F-003)`,
      );
    }
  }
});
