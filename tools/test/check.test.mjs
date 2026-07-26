import assert from "node:assert/strict";
import { test } from "node:test";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  domainSeparatedJsonDigest,
  validateLocalEvidenceGraph,
} from "../src/evidence-graph.mjs";

const toolsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

test("check-consistency passes on the committed tree", () => {
  const out = execFileSync(process.execPath, [path.join(toolsDir, "src", "check-consistency.mjs")], {
    encoding: "utf-8",
  });
  assert.match(out, /check-consistency: OK/);
});

test("gen-matrix --check confirms the committed matrix is fresh", () => {
  const out = execFileSync(
    process.execPath,
    [path.join(toolsDir, "src", "gen-matrix.mjs"), "--check"],
    { encoding: "utf-8" },
  );
  assert.match(out, /matrix is up to date/);
});

test("evidence graph validates local result and performance references", () => {
  const evidenceDirectory = mkdtempSync(path.join(os.tmpdir(), "cognitiveos-evidence-"));
  const manifestPath = path.join(evidenceDirectory, "manifest.json");
  const resultPath = path.join(evidenceDirectory, "result.json");
  const performancePath = path.join(evidenceDirectory, "performance.json");
  const resultDocument = { status: "pass" };
  const performanceDocument = {
    schema_version: "cognitiveos.performance-report/0.1",
    metrics: [],
  };
  writeFileSync(resultPath, `${JSON.stringify(resultDocument)}\n`);
  writeFileSync(performancePath, `${JSON.stringify(performanceDocument)}\n`);
  const manifest = {
    cognitiveos_conformance: {
      evidence_refs: ["./result.json"],
      test_runs: [
        {
          result_ref: "./result.json",
          suite_digest: `sha256:${createHash("sha256").update(`${JSON.stringify(resultDocument)}\n`).digest("hex")}`,
        },
      ],
      performance_reports: [
        {
          report_ref: "./performance.json",
          schema_version: performanceDocument.schema_version,
          report_digest: domainSeparatedJsonDigest(performanceDocument),
        },
      ],
    },
  };
  writeFileSync(manifestPath, JSON.stringify(manifest));

  const graph = validateLocalEvidenceGraph(manifestPath, manifest);
  assert.deepEqual(graph.errors, []);
  assert.equal(graph.verifiedTestRuns, 1);
  assert.equal(graph.verifiedPerformanceReports, 1);
});

test("POSIX and Windows verify orchestrators share evidence safeguards", () => {
  const scriptsDirectory = path.resolve(toolsDir, "..", "scripts");
  const posixScript = readFileSync(path.join(scriptsDirectory, "v01-auto-run.sh"), "utf8");
  const windowsScript = readFileSync(path.join(scriptsDirectory, "v01-auto-run.ps1"), "utf8");

  for (const script of [posixScript, windowsScript]) {
    assert.match(script, /performance-report-m6-overhead\.json/);
    assert.match(script, /performance-report-v01-sample\.json/);
    assert.match(script, /validate-manifest\.mjs/);
    assert.match(script, /sample_or_builder_only/);
    assert.match(script, /not_executed/);
    assert.match(
      script,
      /perf::tests::overhead_report_requires_ungoverned_baseline_and_forbids_benefit/,
    );
    assert.match(script, /1 passed; 0 failed/);
  }
  assert.match(posixScript, /CARGO_TARGET_DIR/);
  assert.match(posixScript, /echo "\$c"/);
  assert.doesNotMatch(posixScript, /echo "\$REPO_ROOT\/\$c"/);
  assert.match(posixScript, /STOP_REASON=PERF004/);
  assert.match(windowsScript, /CARGO_TARGET_DIR/);
  assert.match(windowsScript, /PERF004 report generation failed/);
});
