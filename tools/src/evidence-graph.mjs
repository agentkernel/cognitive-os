import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const DIGEST_PREIMAGE_PREFIX = "CognitiveOS-Digest-V1\n";
const PERFORMANCE_REPORT_DOMAIN = "performance-report/0.1";

function canonicalizeJsonValue(value) {
  if (value === null || typeof value === "boolean" || typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new Error("canonical JSON does not permit non-finite numbers");
    }
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map(canonicalizeJsonValue).join(",")}]`;
  }
  if (typeof value === "object") {
    const members = Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalizeJsonValue(value[key])}`);
    return `{${members.join(",")}}`;
  }
  throw new Error(`unsupported canonical JSON value type: ${typeof value}`);
}

export function fileSha256(filePath) {
  const digest = createHash("sha256").update(readFileSync(filePath)).digest("hex");
  return `sha256:${digest}`;
}

export function domainSeparatedJsonDigest(document, domain = PERFORMANCE_REPORT_DOMAIN) {
  const canonicalDocument = canonicalizeJsonValue(document);
  const digest = createHash("sha256");
  digest.update(DIGEST_PREIMAGE_PREFIX, "utf8");
  digest.update(domain, "utf8");
  digest.update(Buffer.from([0]));
  digest.update(canonicalDocument, "utf8");
  return `sha256:${digest.digest("hex")}`;
}

export function resolveLocalEvidenceReference(manifestPath, reference) {
  if (typeof reference !== "string" || reference.length === 0) {
    throw new Error("evidence reference must be a non-empty string");
  }
  if (/^[A-Za-z][A-Za-z0-9+.-]*:/.test(reference)) {
    throw new Error(`external evidence URI is not locally verifiable: ${reference}`);
  }
  if (reference.includes("?") || reference.includes("#")) {
    throw new Error(`query and fragment evidence references are unsupported: ${reference}`);
  }
  return path.resolve(path.dirname(manifestPath), decodeURIComponent(reference));
}

export function validateLocalEvidenceGraph(manifestPath, manifest) {
  const declaration = manifest.cognitiveos_conformance;
  const errors = [];
  const performanceReportPaths = [];
  let verifiedTestRuns = 0;

  for (const reference of declaration?.evidence_refs ?? []) {
    try {
      const evidencePath = resolveLocalEvidenceReference(manifestPath, reference);
      if (!existsSync(evidencePath)) {
        errors.push(`evidence reference is missing: ${reference}`);
      }
    } catch (error) {
      errors.push(error.message);
    }
  }

  for (const testRun of declaration?.test_runs ?? []) {
    try {
      const resultPath = resolveLocalEvidenceReference(manifestPath, testRun.result_ref);
      if (!existsSync(resultPath)) {
        errors.push(`test run result is missing: ${testRun.result_ref}`);
        continue;
      }
      const actualDigest = fileSha256(resultPath);
      if (actualDigest !== testRun.suite_digest) {
        errors.push(
          `test run digest mismatch for ${testRun.result_ref}: expected ${testRun.suite_digest}, got ${actualDigest}`,
        );
        continue;
      }
      verifiedTestRuns += 1;
    } catch (error) {
      errors.push(error.message);
    }
  }

  for (const performanceReport of declaration?.performance_reports ?? []) {
    try {
      const reportPath = resolveLocalEvidenceReference(
        manifestPath,
        performanceReport.report_ref,
      );
      if (!existsSync(reportPath)) {
        errors.push(`performance report is missing: ${performanceReport.report_ref}`);
        continue;
      }
      const reportDocument = JSON.parse(readFileSync(reportPath, "utf8"));
      if (reportDocument.schema_version !== performanceReport.schema_version) {
        errors.push(
          `performance report schema mismatch for ${performanceReport.report_ref}: expected ${performanceReport.schema_version}, got ${reportDocument.schema_version}`,
        );
        continue;
      }
      const actualDigest = domainSeparatedJsonDigest(reportDocument);
      if (actualDigest !== performanceReport.report_digest) {
        errors.push(
          `performance report digest mismatch for ${performanceReport.report_ref}: expected ${performanceReport.report_digest}, got ${actualDigest}`,
        );
        continue;
      }
      performanceReportPaths.push(reportPath);
    } catch (error) {
      errors.push(error.message);
    }
  }

  return {
    errors,
    performanceReportPaths,
    verifiedTestRuns,
    verifiedPerformanceReports: performanceReportPaths.length,
  };
}
