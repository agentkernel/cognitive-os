/**
 * Fixture gates for the generated shell-family bindings:
 *
 * 1. every sample fixture (typed by the GENERATED interfaces) validates
 *    against the real schema under `specs/schemas/` (ajv draft 2020-12,
 *    full $ref closure) — proves the shared test doubles stay schema-valid
 *    and that schema conditionals still bite through the typed surface;
 * 2. the `isShellStatusView` ingestion guard accepts the valid fixture and
 *    rejects non-view payloads;
 * 3. the runtime `SHELL_STATUS_VALUES` list mirrors the generated union
 *    (the compile-time witness lives in `views.ts`; here we pin the count).
 *
 * The former hand-written interface drift gates (test-time digest
 * re-derivation, YAML re-reads) are gone: schema↔binding parity including
 * `SCHEMA_DIGEST` constants is proven inside contracts-ts (codegen 0.2.0).
 */

import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { Ajv2020 } from "ajv/dist/2020.js";
import addFormatsImport from "ajv-formats";

import {
  sampleControlRequest,
  sampleIntentRecord,
  samplePreview,
  sampleProposal,
  sampleStatusView,
  sampleWatchSubscription,
} from "./fixtures.js";
import { isShellStatusView, SHELL_STATUS_VALUES } from "./views.js";

const addFormats = addFormatsImport as unknown as (ajv: Ajv2020) => Ajv2020;

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const SCHEMA_DIR = path.join(REPO_ROOT, "specs", "schemas");

function buildAjv(): Ajv2020 {
  const ajv = new Ajv2020({ strict: false, allErrors: true, validateFormats: true });
  addFormats(ajv);
  for (const name of readdirSync(SCHEMA_DIR).filter((file) => file.endsWith(".json"))) {
    ajv.addSchema(JSON.parse(readFileSync(path.join(SCHEMA_DIR, name), "utf-8")) as object);
  }
  return ajv;
}

const ajv = buildAjv();

function assertValid(schemaId: string, instance: unknown): void {
  const validate = ajv.getSchema(schemaId);
  assert.ok(validate, `schema ${schemaId} not registered`);
  const valid = validate(instance);
  assert.ok(valid, `${schemaId}: ${JSON.stringify(validate.errors, null, 2)}`);
}

test("sample fixtures validate against the real shell-family schemas", () => {
  assertValid("shell-status-view.schema.json", sampleStatusView());
  assertValid("watch-subscription.schema.json", sampleWatchSubscription());
  assertValid("shell-action-proposal.schema.json", sampleProposal());
  assertValid("shell-command-preview.schema.json", samplePreview());
  assertValid("user-intent-record.schema.json", sampleIntentRecord());
  assertValid("shell-control-request.schema.json", sampleControlRequest());
});

test("schema conditionals still bite through the typed fixtures (negative)", () => {
  // A management-channel proposal without management_session_ref must fail:
  // proves the fixtures exercise the real schema, not a permissive copy.
  const validate = ajv.getSchema("shell-action-proposal.schema.json");
  assert.ok(validate);
  assert.equal(validate(sampleProposal({ channel: "management" })), false);
});

test("SHELL_STATUS_VALUES mirrors the generated status union", () => {
  assert.equal(SHELL_STATUS_VALUES.length, 12);
  assert.equal(new Set(SHELL_STATUS_VALUES).size, 12);
});

test("isShellStatusView accepts the valid view and rejects non-view payloads", () => {
  assert.ok(isShellStatusView(sampleStatusView()));
  assert.ok(!isShellStatusView(null));
  assert.ok(!isShellStatusView({ schema_version: "cognitiveos.event/0.1" }));
  assert.ok(
    !isShellStatusView({
      // Remote-completed evidence report: never a status view.
      protocol: "a2a",
      remote_task_state: "completed",
      artifact_ref: "artifact://remote-org/result-17",
    }),
  );
});
