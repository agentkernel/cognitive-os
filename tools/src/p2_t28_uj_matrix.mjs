/**
 * P2-T28/D01 frozen UJ1..UJ6 capability-truth matrix validator.
 * Required rows must name an existing public caller and mechanical oracle.
 * Web UI and Multi-Agent stay scope-excluded and must not be required.
 */

import { existsSync } from "node:fs";
import path from "node:path";

export const UJ_MATRIX_SCHEMA = "cognitiveos.personal.uj-capability-truth/0.1";
export const EXCLUDED_ROW_IDS = Object.freeze(["UJ6-web-ui", "UJ6-multi-agent"]);

export function validateUjCapabilityTruthMatrix(matrix, options = {}) {
  const repositoryRoot = options.repositoryRoot;
  const fileExists =
    options.fileExists ??
    ((relativePath) =>
      typeof repositoryRoot === "string" &&
      existsSync(path.join(repositoryRoot, ...relativePath.split("/"))));

  if (matrix?.schema !== UJ_MATRIX_SCHEMA) {
    throw new Error(`uj matrix schema must be ${UJ_MATRIX_SCHEMA}`);
  }
  if (matrix.claim_ceiling !== "hypothesis") {
    throw new Error("uj matrix claim_ceiling must be hypothesis");
  }
  if (!Array.isArray(matrix.rows) || matrix.rows.length === 0) {
    throw new Error("uj matrix rows must be a non-empty array");
  }

  const seen = new Set();
  const excluded = [];
  for (const row of matrix.rows) {
    if (typeof row?.id !== "string" || row.id.length === 0) {
      throw new Error("uj matrix row is missing id");
    }
    if (seen.has(row.id)) {
      throw new Error(`uj matrix duplicate row ${row.id}`);
    }
    seen.add(row.id);

    const required = row.required === true;
    const scope = row.scope;
    if (required && scope !== "required") {
      throw new Error(`${row.id} is required but scope is ${scope}`);
    }
    if (!required && scope !== "excluded") {
      throw new Error(`${row.id} is not required but scope is ${scope}`);
    }

    if (required) {
      if (EXCLUDED_ROW_IDS.includes(row.id)) {
        throw new Error(`${row.id} is scope-excluded and must not be required`);
      }
      if (!nonEmpty(row.public_caller)) {
        throw new Error(`${row.id} is missing public_caller`);
      }
      if (!nonEmpty(row.mechanical_oracle)) {
        throw new Error(`${row.id} is missing mechanical_oracle`);
      }
      if (!nonEmpty(row.cleanup)) {
        throw new Error(`${row.id} is missing cleanup`);
      }
      if (!nonEmpty(row.evidence_schema)) {
        throw new Error(`${row.id} is missing evidence_schema`);
      }
      if (!fileExists(row.public_caller)) {
        throw new Error(`${row.id} public_caller does not exist: ${row.public_caller}`);
      }
      if (!fileExists(row.mechanical_oracle)) {
        throw new Error(
          `${row.id} mechanical_oracle does not exist: ${row.mechanical_oracle}`,
        );
      }
    } else {
      excluded.push(row.id);
      if (nonEmpty(row.public_caller) || nonEmpty(row.mechanical_oracle)) {
        throw new Error(`${row.id} is excluded and must not claim a public caller/oracle`);
      }
    }
  }

  for (const excludedId of EXCLUDED_ROW_IDS) {
    if (!excluded.includes(excludedId)) {
      throw new Error(`uj matrix must keep ${excludedId} explicit and excluded`);
    }
  }
  return { row_count: matrix.rows.length, excluded };
}

function nonEmpty(value) {
  return typeof value === "string" && value.trim().length > 0;
}
