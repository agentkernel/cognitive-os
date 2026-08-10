/**
 * Validate a profile manifest instance against
 * `specs/schemas/profile-manifest.schema.json` (draft 2020-12, relative $refs
 * resolved from the containing file per conformance/README.md).
 *
 * Usage: node tools/src/validate-manifest.mjs [path-to-manifest.json]
 * Default path: artifacts/evidence/conformance/sample-profile-manifest.json
 */

import { Ajv2020 } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import { readFileSync } from "node:fs";
import { validateLocalEvidenceGraph } from "./evidence-graph.mjs";
import { loadSchemas, readJson, repoPath } from "./lib.mjs";
import { validatePerformanceReportPolicy } from "./performance-policy.mjs";

const manifestPath =
  process.argv[2] ??
  repoPath("artifacts", "evidence", "conformance", "sample-profile-manifest.json");

const ajv = new Ajv2020({ strict: false, allErrors: true, validateFormats: true });
addFormats(ajv);
// $id policy (D-001/D-006): $id == file name, so schemas register under
// their own $id and relative $refs resolve without any stripping layer.
for (const schema of loadSchemas()) {
  ajv.addSchema(schema.doc);
}

const validate = ajv.getSchema("profile-manifest.schema.json");
if (!validate) {
  console.error("validate-manifest: profile-manifest.schema.json not registered");
  process.exit(1);
}

const manifest = readJson(manifestPath);
if (!validate(manifest)) {
  console.error(`validate-manifest: INVALID ${manifestPath}`);
  console.error(ajv.errorsText(validate.errors, { separator: "\n" }));
  process.exit(1);
}

const evidenceGraph = validateLocalEvidenceGraph(manifestPath, manifest);
const validatePerformanceReport = ajv.getSchema("performance-report.schema.json");
if (!validatePerformanceReport) {
  console.error("validate-manifest: performance-report.schema.json not registered");
  process.exit(1);
}
for (const performanceReportPath of evidenceGraph.performanceReportPaths) {
  const performanceReport = JSON.parse(readFileSync(performanceReportPath, "utf8"));
  if (!validatePerformanceReport(performanceReport)) {
    evidenceGraph.errors.push(
      `invalid performance report ${performanceReportPath}: ${ajv.errorsText(validatePerformanceReport.errors, { separator: "\n" })}`,
    );
  }
  for (const policyError of validatePerformanceReportPolicy(performanceReport)) {
    evidenceGraph.errors.push(
      `invalid performance policy ${performanceReportPath}: ${policyError.code} at ${policyError.path}: ${policyError.message}`,
    );
  }
}
if (evidenceGraph.errors.length > 0) {
  console.error(`validate-manifest: INVALID EVIDENCE GRAPH ${manifestPath}`);
  for (const error of evidenceGraph.errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

const profiles = manifest.cognitiveos_conformance?.profiles ?? {};
const nonPlanned = Object.entries(profiles).filter(([, v]) => v !== "planned");
console.log(
  `validate-manifest: OK ${manifestPath} (${Object.keys(profiles).length} profiles, ` +
    `${nonPlanned.length} non-planned${nonPlanned.length ? `: ${nonPlanned.map(([k, v]) => `${k}=${v}`).join(", ")}` : ""}; ` +
    `${evidenceGraph.verifiedTestRuns} test runs and ${evidenceGraph.verifiedPerformanceReports} performance reports verified)`,
);
