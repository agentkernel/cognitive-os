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
 * $id policy (D-001/D-006 closure): every schema declares `$id` equal to its
 * file name, so each relative `$ref` resolves from the containing schema
 * file (`conformance/README.md` convention) and `$id` is the retrieval URI.
 */
function buildAjv(schemas: SchemaDoc[]): Ajv2020 {
  const ajv = new Ajv2020({ strict: false, allErrors: true, validateFormats: true });
  addFormats(ajv);
  for (const schema of schemas) {
    assert.equal(schema.doc["$id"], schema.name, `${schema.name}: $id must equal file name`);
    ajv.addSchema(schema.doc);
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

/**
 * Positive AKP request envelope (D-013 wire schema): the members the
 * companion describes (specs/akp/README.md section 3) must be accepted, so
 * the negative vectors are not passing vacuously.
 */
function positiveRequestEnvelope(): Record<string, unknown> {
  return {
    message_id: "01890a5d-ac96-774b-bcce-b302099a8070",
    operation: "shell.submit",
    protocol_version: "cognitiveos.akp/0.2",
    schema_digest: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    sender: "principal://tenant-a/user-alice",
    audience: "kernel://task-gateway",
    correlation_id: "conv://tenant-a/session-1/turn-9",
    causation_id: "01890a5d-ac96-774b-bcce-b302099a806f",
    deadline: "2026-07-20T00:05:00Z",
    idempotency_key: "idem-shell-submit-0001",
    authorization_ref: "cap://tenant-a/lease-77",
    budget: { wall_time_ms: 60000 },
    payload: { proposal_ref: "proposal://tenant-a/sap-0001" },
    payload_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    extensions: [{ id: "x-trace", critical: false }],
  };
}

test("AKP request envelope accepts described members and rejects vector negatives", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("akp-request-envelope.schema.json");
  assert.ok(validate);
  assert.equal(
    validate(positiveRequestEnvelope()),
    true,
    `described request envelope must validate: ${JSON.stringify(validate.errors)}`,
  );
  // Management members ride the same envelope (AKP section 10.1) but the
  // session ref never travels alone.
  const management = positiveRequestEnvelope();
  management["management_session_ref"] = "session://tenant-a/pms-1";
  assert.equal(validate(management), false, "lone management_session_ref must be rejected");
  management["actor_chain_digest"] = `sha256:${"d".repeat(64)}`;
  management["activity_context_ref"] = "activity://tenant-a/act-1";
  assert.equal(
    validate(management),
    true,
    `management-bound envelope must validate: ${JSON.stringify(validate.errors)}`,
  );
  for (const vector of [
    "akp-envelope-no-schema-pin-001.json",
    "akp-envelope-ambiguous-payload-002.json",
  ]) {
    assert.equal(
      validate(vectorObject(vector)),
      false,
      `${vector} object must be rejected (REQ-AKP-ENV-001/002)`,
    );
  }
});

test("AKP result envelope requires the machine error and partial continuation", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("akp-result-envelope.schema.json");
  assert.ok(validate);
  const ok: Record<string, unknown> = {
    in_reply_to: "01890a5d-ac96-774b-bcce-b302099a8070",
    correlation_id: "conv://tenant-a/session-1/turn-9",
    protocol_version: "cognitiveos.akp/0.2",
    status: "ok",
    result: { accepted_ref: "task://tenant-a/tsk-0007" },
    observed_versions: { task: 4 },
    cost: { wall_time_ms: 12 },
    audit_ref: "audit://tenant-a/rec-1",
  };
  assert.equal(validate(ok), true, `ok result must validate: ${JSON.stringify(validate.errors)}`);
  const errorResult = {
    in_reply_to: "01890a5d-ac96-774b-bcce-b302099a8070",
    correlation_id: "conv://tenant-a/session-1/turn-9",
    protocol_version: "cognitiveos.akp/0.2",
    status: "error",
    error: { code: "STATE_CONFLICT", category: "state", stage: "authorization", retryable: true },
  };
  assert.equal(
    validate(errorResult),
    true,
    `error result with machine error must validate: ${JSON.stringify(validate.errors)}`,
  );
  assert.equal(
    validate(vectorObject("akp-result-error-without-machine-code-003.json")),
    false,
    "error status without the machine error envelope must be rejected (REQ-ERR-001)",
  );
  const partial = { ...ok, status: "partial" };
  assert.equal(validate(partial), false, "partial without continuation must be rejected");
  assert.equal(
    validate({ ...partial, continuation: { high_watermark: 7 } }),
    true,
    `partial with continuation must validate: ${JSON.stringify(validate.errors)}`,
  );
});

test("AKP stream frame kinds carry their required members", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("akp-stream-frame.schema.json");
  assert.ok(validate);
  const snapshot: Record<string, unknown> = {
    stream_id: "watch://tenant-a/wsub-1",
    sequence: 0,
    kind: "snapshot",
    snapshot_version: 4,
    payload: { view: "initial" },
    final: false,
    cost: { context_bytes: 2048 },
  };
  assert.equal(
    validate(snapshot),
    true,
    `snapshot frame must validate: ${JSON.stringify(validate.errors)}`,
  );
  const errorFrame = {
    stream_id: "watch://tenant-a/wsub-1",
    sequence: 9,
    kind: "error",
    error: { code: "WATCH_CURSOR_STALE", category: "watch", stage: "resume", retryable: true },
    final: true,
  };
  assert.equal(
    validate(errorFrame),
    true,
    `machine-coded error frame must validate: ${JSON.stringify(validate.errors)}`,
  );
  const unversioned = { ...snapshot };
  delete unversioned["snapshot_version"];
  assert.equal(validate(unversioned), false, "snapshot without snapshot_version must be rejected");
  assert.equal(
    validate(vectorObject("akp-stream-frame-unsequenced-004.json")),
    false,
    "frame without stream identity/sequence must be rejected (REQ-AKP-STR-001)",
  );
});

test("shell control request is a cancel with target and reason", () => {
  const ajv = buildAjv(loadSchemas());
  const validate = ajv.getSchema("shell-control-request.schema.json");
  assert.ok(validate);
  const cancel = {
    schema_version: "cognitiveos.shell-control-request/0.1",
    control: "cancel",
    target_ref: "task://tenant-a/tsk-0007",
    reason: "user requested stop from the shell",
  };
  assert.equal(
    validate(cancel),
    true,
    `cancel control request must validate: ${JSON.stringify(validate.errors)}`,
  );
  assert.equal(
    validate(vectorObject("shell-control-unreasoned-cancel-001.json")),
    false,
    "cancel without reason must be rejected (REQ-AKP-CAN-001)",
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
