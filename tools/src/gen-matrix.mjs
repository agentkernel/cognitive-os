/**
 * Traceability matrix generator: derives the committed baseline of
 * `docs/traceability/matrix.yaml` from `specs/registry/requirements.yaml`
 * plus the vector set, PRESERVING any hand-maintained `impl`, `impl_tests`,
 * `evidence`, `docs` and `notes` fields of an existing matrix.
 *
 * Usage: node tools/src/gen-matrix.mjs [--check]
 *   --check : regenerate in memory and fail (exit 2) if the committed file
 *             differs (drift gate; wired into check-consistency via CI).
 */

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import YAML from "yaml";
import { loadRegistries, loadVectors, repoPath } from "./lib.mjs";

const MATRIX_PATH = repoPath("docs", "traceability", "matrix.yaml");

const { requirements } = loadRegistries();
const vectors = loadVectors();

const vectorsByReq = new Map();
for (const vector of vectors) {
  for (const reqId of vector.doc.requirement_ids ?? []) {
    if (!vectorsByReq.has(reqId)) {
      vectorsByReq.set(reqId, []);
    }
    vectorsByReq.get(reqId).push(vector.path);
  }
}

const existing = new Map();
if (existsSync(MATRIX_PATH)) {
  const current = YAML.parse(readFileSync(MATRIX_PATH, "utf-8"));
  for (const entry of current?.requirements ?? []) {
    existing.set(entry.id, entry);
  }
}

const entries = requirements.requirements.map((req) => {
  const prior = existing.get(req.id) ?? {};
  const entry = {
    id: req.id,
    owner: req.owner,
    owner_spec: req.owner_spec,
    status: req.status,
    registry_tests: req.tests ?? [],
    vectors: (vectorsByReq.get(req.id) ?? []).sort(),
    impl: prior.impl ?? [],
    impl_tests: prior.impl_tests ?? [],
    evidence: prior.evidence ?? [],
    docs: prior.docs ?? [],
  };
  if (prior.notes) {
    entry.notes = prior.notes;
  }
  return entry;
});

const doc = {
  version: "0.1",
  generated_by: "tools/src/gen-matrix.mjs (registry-derived; impl/evidence/docs fields are hand-maintained and preserved)",
  field_semantics: {
    registry_tests: "test IDs mapped in specs/registry/requirements.yaml",
    vectors: "conformance vector files whose requirement_ids include this REQ (derived)",
    impl: "implementation module paths (hand-maintained from M1)",
    impl_tests: "unit/integration/fault test paths (hand-maintained)",
    evidence: "artifacts/evidence digests or checkpoint references (hand-maintained)",
    docs: "standard/ADR/plan sections normatively touching this REQ (hand-maintained)",
  },
  requirements: entries,
};

const rendered = YAML.stringify(doc, { lineWidth: 120 });

if (process.argv.includes("--check")) {
  const committed = existsSync(MATRIX_PATH) ? readFileSync(MATRIX_PATH, "utf-8") : "";
  if (committed !== rendered) {
    console.error(
      "gen-matrix --check: docs/traceability/matrix.yaml is stale. " +
        "Run `pnpm --filter @cognitiveos/repo-tools run gen-matrix` and commit the result.",
    );
    process.exit(2);
  }
  console.log("gen-matrix --check: matrix is up to date");
} else {
  writeFileSync(MATRIX_PATH, rendered, "utf-8");
  console.log(`gen-matrix: wrote ${entries.length} requirement entries to docs/traceability/matrix.yaml`);
}
