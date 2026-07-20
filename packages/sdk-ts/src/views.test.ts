/**
 * Drift gates for the interim hand-modeled shell-family bindings:
 *
 * 1. every sample fixture (typed by `views.ts`) validates against the real
 *    schema under `specs/schemas/` (ajv draft 2020-12, full $ref closure);
 * 2. `SHELL_SCHEMA_DIGESTS` re-derives from the schema files through the
 *    contracts canonical digest (domain `schema-bundle/0.1`, the generated
 *    module header recipe);
 * 3. the `isShellStatusView` ingestion guard accepts the valid fixture and
 *    rejects non-view payloads.
 *
 * A schema change therefore turns this suite red instead of silently
 * desynchronizing the client (interim measure until Lane-CTR codegen covers
 * the shell family; gap registered in the 20260720 lane-tsc handoff).
 */

import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { Ajv2020 } from "ajv/dist/2020.js";
import addFormatsImport from "ajv-formats";

import { digestJson } from "@cognitiveos/contracts-ts";

import {
  sampleIntentRecord,
  samplePreview,
  sampleProposal,
  sampleStatusView,
  sampleWatchSubscription,
} from "./fixtures.js";
import { isShellStatusView, SHELL_SCHEMA_DIGESTS } from "./views.js";

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
});

test("schema conditionals still bite through the typed fixtures (negative)", () => {
  // A management-channel proposal without management_session_ref must fail:
  // proves the fixtures exercise the real schema, not a permissive copy.
  const validate = ajv.getSchema("shell-action-proposal.schema.json");
  assert.ok(validate);
  assert.equal(validate(sampleProposal({ channel: "management" })), false);
});

test("SHELL_SCHEMA_DIGESTS matches the canonical digests of the schema files", () => {
  for (const [file, pinned] of Object.entries(SHELL_SCHEMA_DIGESTS)) {
    const text = readFileSync(path.join(SCHEMA_DIR, file), "utf-8");
    assert.equal(digestJson(text, "schema-bundle/0.1"), pinned, `${file} digest drifted`);
  }
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
