import assert from "node:assert/strict";
import { test } from "node:test";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  domainSeparatedJsonDigest,
  validateLocalEvidenceGraph,
} from "../src/evidence-graph.mjs";

const toolsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(toolsDir, "..");
const consistencyCheckerPath = path.join(toolsDir, "src", "check-consistency.mjs");

function runConsistencyFailureInjection(overrides) {
  const overrideDirectory = mkdtempSync(path.join(os.tmpdir(), "cognitiveos-consistency-"));
  try {
    for (const [repositoryRelativePath, transformSource] of Object.entries(overrides)) {
      const sourcePath = path.join(repositoryRoot, ...repositoryRelativePath.split("/"));
      const overridePath = path.join(overrideDirectory, ...repositoryRelativePath.split("/"));
      const originalSource = readFileSync(sourcePath, "utf8");
      const transformedSource = transformSource(originalSource);
      assert.notEqual(
        transformedSource,
        originalSource,
        `failure injection did not change ${repositoryRelativePath}`,
      );
      mkdirSync(path.dirname(overridePath), { recursive: true });
      writeFileSync(overridePath, transformedSource);
    }

    return spawnSync(process.execPath, [consistencyCheckerPath], {
      encoding: "utf8",
      env: {
        ...process.env,
        COGNITIVEOS_CONSISTENCY_OVERRIDE_DIR: overrideDirectory,
      },
    });
  } finally {
    rmSync(overrideDirectory, { recursive: true, force: true });
  }
}

test("check-consistency passes on the current repository tree", () => {
  const out = execFileSync(process.execPath, [consistencyCheckerPath], {
    encoding: "utf-8",
  });
  assert.match(out, /check-consistency: OK/);
});

test("Personal governance drift is rejected by failure injection", () => {
  const result = runConsistencyFailureInjection({
    "AGENTS.md": (source) =>
      source
        .replace("COMMAND-SHELL-PS51", "REMOVED-COMMAND-SHELL-GUARD")
        .replace("CHECKPOINT-DELIVERY-01", "REMOVED-CHECKPOINT-DELIVERY-GUARD")
        .replace("TASK-ATOMIC-DELIVERY-01", "REMOVED-TASK-ATOMIC-DELIVERY-GUARD"),
    "docs/governance/DEVELOPMENT-OPERATING-MODEL.md": (source) =>
      source
        .replace(
          "CHECKPOINT-DELIVERY-01",
          "REMOVED-CHECKPOINT-DELIVERY-GUARD",
        )
        .replace("TASK-ATOMIC-DELIVERY-01", "REMOVED-TASK-ATOMIC-DELIVERY-GUARD"),
    "docs/governance/PROJECT-IDENTITY.md": (source) =>
      source.replace(
        "一个 task branch、一个持续更新的 Draft PR 和一个 task-scoped lease",
        "REMOVED-TASK-ATOMIC-DELIVERY-GUARD",
      ),
    "docs/standards/docs-sync-contract.md": (source) =>
      source
        .replaceAll(
          "CHECKPOINT-DELIVERY-01",
          "REMOVED-CHECKPOINT-DELIVERY-GUARD",
        )
        .replaceAll("TASK-ATOMIC-DELIVERY-01", "REMOVED-TASK-ATOMIC-DELIVERY-GUARD"),
    "docs/plan/PERSONAL-TEST-ENVIRONMENTS.md": (source) =>
      source.replaceAll("RUST-LINK-DEV-WIN-GNU-01", "REMOVED-RUST-LINK-GUARD"),
    "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md": (source) => {
      const taskRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| P7-T08 |") && line.includes("| not-started |"));
      assert.ok(taskRow, "P7-T08 task row must exist for duplicate injection");
      const deliverySliceRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| `P2-T02/D01` |"));
      assert.ok(deliverySliceRow, "P2-T02/D01 row must exist for duplicate injection");
      return source
        .replaceAll("TASK-ATOMIC-DELIVERY-01", "REMOVED-TASK-ATOMIC-DELIVERY-GUARD")
        .replace(taskRow, `${taskRow}\n${taskRow}`)
        .replace(deliverySliceRow, `${deliverySliceRow}\n${deliverySliceRow}`);
    },
    "docs/plan/personal-trace.yaml": (source) =>
      `${source.replace(
        "delivery_slice_status: [ready, in-progress, blocked, done, cancelled]",
        "delivery_slice_status: [ready, in-progress, blocked, done]",
      )}\ncurrent_snapshot:\n  B01: pass\n`,
    "docs/plan/PARALLEL-LANES.md": (source) =>
      source
        .replace(
          "一个 task branch/Draft PR + 一份活动 task lease",
          "REMOVED-TASK-ATOMIC-DELIVERY-GUARD",
        )
        .replace(
          "### 3.1 最近关闭的 leases",
          "| `lease/personal/P0-T01/broad-fixture` | fixture | Lane-DOC | `fixture` | `docs/plan/**` | test fixture | 2026-08-02 / 2026-08-02 | active |\n### 3.1 最近关闭的 leases",
        ),
    "docs/plan/PROGRESS.md": (source) =>
      source
        .replace(/(\| B01 first-install\/first-conversation Gate \| \*\*)(?:running|fail)(\*\* \|)/, "$1pass$2")
        .replace("| `P2-T03/D03` | `done` |", "| `P2-T03/D03` | `in-progress` |")
        .replace("| `P2-T03/D05` | `done` |", "| `P2-T03/D05` | `in-progress` |"),
    "docs/governance/project-scope.yaml": (source) =>
      source.replace(
        "product_design: docs/product/personal/README.md",
        "product_design: docs/product/personal/MISSING.md",
      ),
    "docs/prompts/common-prefix.md": (source) =>
      source.replace("dated non-executable reference", "legacy executable prompt"),
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(
    result.stderr,
    /AGENTS\.md[\s\S]*command\/environment guard is missing required fragment: COMMAND-SHELL-PS51/,
  );
  assert.match(
    result.stderr,
    /PERSONAL-TEST-ENVIRONMENTS\.md[\s\S]*command\/environment guard is missing required fragment: RUST-LINK-DEV-WIN-GNU-01/,
  );
  assert.match(
    result.stderr,
    /AGENTS\.md[\s\S]*checkpoint-delivery guard is missing required fragment: CHECKPOINT-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /DEVELOPMENT-OPERATING-MODEL\.md[\s\S]*checkpoint-delivery guard is missing required fragment: CHECKPOINT-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /docs-sync-contract\.md[\s\S]*checkpoint-delivery guard is missing required fragment: CHECKPOINT-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /AGENTS\.md[\s\S]*task-atomic delivery guard is missing required fragment: TASK-ATOMIC-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /PROJECT-IDENTITY\.md[\s\S]*task-atomic delivery guard is missing required fragment/,
  );
  assert.match(
    result.stderr,
    /DEVELOPMENT-OPERATING-MODEL\.md[\s\S]*task-atomic delivery guard is missing required fragment: TASK-ATOMIC-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /PERSONAL-DEVELOPMENT-PLAN\.md[\s\S]*task-atomic delivery guard is missing required fragment: TASK-ATOMIC-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /PARALLEL-LANES\.md[\s\S]*task-atomic delivery guard is missing required fragment/,
  );
  assert.match(
    result.stderr,
    /docs-sync-contract\.md[\s\S]*task-atomic delivery guard is missing required fragment: TASK-ATOMIC-DELIVERY-01/,
  );
  assert.match(result.stderr, /duplicate formal task definition: P7-T08/);
  assert.match(result.stderr, /duplicate formal delivery slice definition: P2-T02\/D01/);
  assert.match(result.stderr, /summary counts .* do not match task rows/);
  assert.match(result.stderr, /P2-T03 has 2 in-progress delivery slices; maximum is 1/);
  assert.match(result.stderr, /trace must not copy a parallel current_snapshot/);
  assert.match(result.stderr, /delivery_slice_status is missing cancelled/);
  assert.match(result.stderr, /active_project\.product_design must reference an existing/);
  assert.match(result.stderr, /legacy prompts must be explicitly non-executable/);
  assert.match(result.stderr, /B01 cannot pass before the formal attempt denominator is complete/);
  assert.match(result.stderr, /claims forbidden broad protected tree: docs\/plan\/\*\*/);
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
