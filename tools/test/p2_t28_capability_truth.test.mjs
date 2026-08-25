import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  EXCLUDED_ROW_IDS,
  UJ_MATRIX_SCHEMA,
  validateUjCapabilityTruthMatrix,
} from "../src/p2_t28_uj_matrix.mjs";

const toolsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(toolsDir, "..");
const matrixPath = path.join(toolsDir, "fixtures", "p2_t28_uj_matrix.json");

function loadMatrix() {
  return JSON.parse(readFileSync(matrixPath, "utf8"));
}

test("frozen UJ1..UJ6 matrix names existing callers and oracles", () => {
  const matrix = loadMatrix();
  const result = validateUjCapabilityTruthMatrix(matrix, { repositoryRoot });
  assert.equal(matrix.schema, UJ_MATRIX_SCHEMA);
  assert.equal(matrix.claim_ceiling, "hypothesis");
  assert.equal(result.row_count, 14);
  assert.deepEqual(result.excluded, EXCLUDED_ROW_IDS);
  const required = matrix.rows.filter((row) => row.required);
  assert.equal(required.length, 12);
  assert.ok(required.every((row) => row.family.startsWith("UJ")));
});

test("required rows fail closed when the public caller is missing", () => {
  const matrix = loadMatrix();
  const target = matrix.rows.find((row) => row.id === "UJ6-backup-restore");
  assert.ok(target);
  target.public_caller = "";
  assert.throws(
    () => validateUjCapabilityTruthMatrix(matrix, { repositoryRoot }),
    /UJ6-backup-restore is missing public_caller/,
  );
});

test("required rows fail closed when cleanup is missing", () => {
  const matrix = loadMatrix();
  const target = matrix.rows.find((row) => row.id === "UJ6-pi-lifecycle");
  assert.ok(target);
  target.cleanup = "";
  assert.throws(
    () => validateUjCapabilityTruthMatrix(matrix, { repositoryRoot }),
    /UJ6-pi-lifecycle is missing cleanup/,
  );
});

test("required rows fail closed when the mechanical oracle is missing", () => {
  const matrix = loadMatrix();
  const target = matrix.rows.find((row) => row.id === "UJ1-install-init-first-response");
  assert.ok(target);
  target.mechanical_oracle = "";
  assert.throws(
    () => validateUjCapabilityTruthMatrix(matrix, { repositoryRoot }),
    /UJ1-install-init-first-response is missing mechanical_oracle/,
  );
});

test("required rows fail closed when a named caller path does not exist", () => {
  const matrix = loadMatrix();
  assert.throws(
    () =>
      validateUjCapabilityTruthMatrix(matrix, {
        repositoryRoot,
        fileExists: (relativePath) => relativePath !== "personal/apps/admin-cli/src/personal_cli/init.rs",
      }),
    /UJ1-install-init-first-response public_caller does not exist/,
  );
});

test("Web UI and Multi-Agent stay explicit and must not be required", () => {
  const matrix = loadMatrix();
  const webUi = matrix.rows.find((row) => row.id === "UJ6-web-ui");
  assert.ok(webUi);
  webUi.required = true;
  webUi.scope = "required";
  webUi.public_caller = "personal/apps/admin-cli/src/main.rs";
  webUi.mechanical_oracle = "personal/apps/admin-cli/tests/p2_t27_backup_restore.rs";
  webUi.cleanup = "none";
  webUi.evidence_schema = "forbidden";
  assert.throws(
    () => validateUjCapabilityTruthMatrix(matrix, { repositoryRoot }),
    /UJ6-web-ui is scope-excluded and must not be required/,
  );

  const missingExcluded = loadMatrix();
  missingExcluded.rows = missingExcluded.rows.filter((row) => row.id !== "UJ6-multi-agent");
  assert.throws(
    () => validateUjCapabilityTruthMatrix(missingExcluded, { repositoryRoot }),
    /UJ6-multi-agent/,
  );
});
