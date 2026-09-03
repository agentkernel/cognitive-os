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
        .replace("COMMAND-SHELL-PS51", "REMOVED-COMMAND-SHELL-GUARD"),
    "docs/governance/DEVELOPMENT-OPERATING-MODEL.md": (source) =>
      source
        .replace(
          "CHECKPOINT-DELIVERY-01",
          "REMOVED-CHECKPOINT-DELIVERY-GUARD",
        )
        .replace("TASK-ATOMIC-DELIVERY-01", "REMOVED-TASK-ATOMIC-DELIVERY-GUARD"),
    "docs/plan/PERSONAL-TEST-ENVIRONMENTS.md": (source) =>
      source.replaceAll("RUST-LINK-DEV-WIN-GNU-01", "REMOVED-RUST-LINK-GUARD"),
    "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md": (source) => {
      const taskRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| P9-T01 |"));
      assert.ok(taskRow, "P9-T01 task row must exist for duplicate injection");
      const deliverySliceRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| `P2-T02/D01` |"));
      assert.ok(deliverySliceRow, "P2-T02/D01 row must exist for duplicate injection");
      return source
        .replace(taskRow, `${taskRow}\n${taskRow}`)
        .replace(deliverySliceRow, `${deliverySliceRow}\n${deliverySliceRow}`);
    },
    "docs/plan/personal-trace.yaml": (source) =>
      `${source.replace(
        "delivery_slice_status: [ready, in-progress, blocked, done, cancelled]",
        "delivery_slice_status: [ready, in-progress, blocked, done]",
      )}\ncurrent_snapshot:\n  B01: pass\n`,
    "docs/plan/PARALLEL-LANES.md": (source) =>
      source.replace(
          "### 3.1 最近关闭的 leases",
          "| `lease/personal/P0-T01/broad-fixture` | fixture | Lane-DOC | `fixture` | `docs/plan/**` | test fixture | 2026-08-02 / 2026-08-02 | active |\n### 3.1 最近关闭的 leases",
        ),
    "docs/plan/PROGRESS.md": (source) =>
      source
        .replace(/(\| B01 first-install\/first-conversation Gate \| \*\*)(?:running|fail|blocked)(\*\* \|)/, "$1pass$2")
        .replace("Attempt 6 of formal minimum 6", "Attempt 5 of formal minimum 6")
        .replace("| `P2-T03/D03` | `done` |", "| `P2-T03/D03` | `in-progress` |")
        .replace("| `P2-T03/D05` | `done` |", "| `P2-T03/D05` | `in-progress` |"),
    "docs/governance/project-scope.yaml": (source) =>
      source.replace(
        "product_design: personal/docs/product/README.md",
        "product_design: personal/docs/product/MISSING.md",
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
    /DEVELOPMENT-OPERATING-MODEL\.md[\s\S]*checkpoint-delivery guard is missing required fragment: CHECKPOINT-DELIVERY-01/,
  );
  assert.match(
    result.stderr,
    /DEVELOPMENT-OPERATING-MODEL\.md[\s\S]*task-atomic delivery guard is missing required fragment: TASK-ATOMIC-DELIVERY-01/,
  );
  assert.match(result.stderr, /duplicate formal task definition: P9-T01/);
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

test("duplicate Current snapshot lease rows are rejected", () => {
  const result = runConsistencyFailureInjection({
    "docs/plan/PROGRESS.md": (source) => {
      const activeLeaseRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| Active task lease |"));
      assert.ok(activeLeaseRow, "canonical Active task lease row must exist");
      return source.replace(activeLeaseRow, `${activeLeaseRow}\n${activeLeaseRow}`);
    },
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /CURRENT_SNAPSHOT_DUPLICATE_CANONICAL_ROW/);
});

test("active lease must match the unique in-progress Slice", () => {
  const result = runConsistencyFailureInjection({
    "docs/plan/PARALLEL-LANES.md": (source) => {
      const normalized = source.replace(/\r\n/g, "\n");
      const header =
        "| Lease ID | Task / slice | Primary lane | Branch | Writable paths | Owner/session | Claimed / heartbeat | Status |\n|---|---|---|---|---|---|---|---|";
      assert.ok(normalized.includes(header), "canonical active lease table header must exist");
      const fakeRow =
        "| `lease/personal/P7-T04/performance-governance` | P7-T04/D99 mismatch fixture | Lane-CFR | `personal/P7-T04-performance-governance` | `docs/plan/PROGRESS.md` | Cursor continuous-development session | 2026-08-10 / 2026-08-10 | active |";
      const injected = normalized.replace(header, `${header}\n${fakeRow}`);
      // Preserve original newline style so the override mirrors the on-disk file.
      return source.includes("\r\n") ? injected.replace(/\n/g, "\r\n") : injected;
    },
    "docs/plan/PROGRESS.md": (source) => {
      const activeLeaseRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| Active task lease |"));
      assert.ok(activeLeaseRow, "canonical Active task lease row must exist");
      // Force Current-snapshot lease ids that cannot match the unique in-progress
      // Slice ownership, whether the live snapshot currently names one or more
      // active leases or intentionally records `none`.
      const mismatchedRow = /`none`/.test(activeLeaseRow)
        ? activeLeaseRow.replace(
            /`none`/,
            "`lease/personal/P7-T04/performance-governance`",
          )
        : activeLeaseRow.replace(
            /`(lease\/personal\/[^`]+)`/g,
            "`lease/personal/P7-T04/performance-governance`",
          );
      assert.notEqual(
        mismatchedRow,
        activeLeaseRow,
        "Active task lease injection must rewrite the lease id",
      );
      return source.replace(activeLeaseRow, mismatchedRow);
    },
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /CURRENT_SNAPSHOT_LEASE_MISMATCH/);
});

const evaluationLeaseTableHeader =
  "| Lease ID | Task / slice | Primary lane | Branch | Writable paths | Owner/session | Claimed / heartbeat | Status |\n|---|---|---|---|---|---|---|---|";

function injectEvaluationLease(leaseRow) {
  return {
    "docs/plan/PARALLEL-LANES.md": (source) => {
      const normalized = source.replace(/\r\n/g, "\n");
      assert.ok(
        normalized.includes(evaluationLeaseTableHeader),
        "canonical active lease table header must exist",
      );
      // Replace the active-lease table body rather than appending to it, so the
      // fixture exercises the injected lease in isolation. Appending would make
      // the outcome depend on whichever real task lease happens to be active,
      // and any real lease that also owns `docs/plan/PROGRESS.md` would trip the
      // overlapping-writable-paths rule instead of the behaviour under test.
      const headerIndex = normalized.indexOf(evaluationLeaseTableHeader);
      const bodyStart = headerIndex + evaluationLeaseTableHeader.length;
      const remainder = normalized.slice(bodyStart);
      const existingRows = remainder.match(/^(?:\n\|[^\n]*)*/)?.[0] ?? "";
      const injected =
        normalized.slice(0, bodyStart) +
        `\n${leaseRow}` +
        remainder.slice(existingRows.length);
      return source.includes("\r\n") ? injected.replace(/\n/g, "\r\n") : injected;
    },
    "docs/plan/PROGRESS.md": (source) => {
      const activeLeaseRow = source
        .split(/\r?\n/)
        .find((line) => line.startsWith("| Active task lease |"));
      assert.ok(activeLeaseRow, "canonical Active task lease row must exist");
      const leaseId = leaseRow.match(/`(lease\/personal\/[^`]+)`/)?.[1];
      assert.ok(leaseId, "evaluation lease row must carry a lease id");
      // The override tree replaced the whole active-lease table with the
      // injected lease, so the temporary Current snapshot must stop naming any
      // real lease. Rewrite every reference in the row — the canonical id and
      // any narrative mention in the later columns — so the fixture depends
      // only on the injected lease whatever the live table currently holds.
      const sanitizedRow = activeLeaseRow.replaceAll(
        /`lease\/personal\/[^`]+`/g,
        `\`${leaseId}\``,
      );
      const referencedRow = sanitizedRow.includes(`\`${leaseId}\``)
        ? sanitizedRow
        : sanitizedRow.replace(/`none`/, `\`${leaseId}\``);
      assert.notEqual(referencedRow, activeLeaseRow);
      return source.replace(activeLeaseRow, referencedRow);
    },
  };
}

test("owner-directed evaluation lease is accepted without a formal task slice", () => {
  const result = runConsistencyFailureInjection(
    injectEvaluationLease(
      "| `lease/personal/EVAL-20260812/performance-evaluation-002` | `PERSONAL-PERF-EVAL-002` owner-directed evaluation campaign; no formal task/slice | Lane-CFR + Lane-DOC | `evaluation/personal-performance-002` | `docs/evaluation/personal-performance-benchmark-execution-plan.md`; `docs/checkpoints/`; `docs/plan/PROGRESS.md` | evaluation session | 2026-08-12 / 2026-08-12 | active |",
    ),
  );

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /check-consistency: OK/);
});

test("evaluation lease rejects product paths and unregistered campaigns", () => {
  const result = runConsistencyFailureInjection(
    injectEvaluationLease(
      "| `lease/personal/EVAL-20260812/rogue` | `PERSONAL-PERF-EVAL-999` unregistered evaluation fixture | Lane-CFR | `evaluation/fixture` | `personal/crates/cognitive-runtime/src/lib.rs`; `docs/plan/PROGRESS.md` | evaluation session | 2026-08-12 / 2026-08-12 | active |",
    ),
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /EVAL_LEASE_UNREGISTERED: evaluation campaign PERSONAL-PERF-EVAL-999/);
  assert.match(
    result.stderr,
    /EVAL_LEASE_PATH_FORBIDDEN: evaluation lease lease\/personal\/EVAL-20260812\/rogue .* not personal\/crates\/cognitive-runtime\/src\/lib\.rs/,
  );
});

function injectGovernanceLease(leaseRow, { registerDelivery = true } = {}) {
  const base = injectEvaluationLease(leaseRow);
  if (!registerDelivery) {
    return base;
  }
  const leaseId = leaseRow.match(/`(lease\/personal\/[^`]+)`/)?.[1];
  const deliveryId = leaseId?.match(/^lease\/personal\/(GOV-[^/]+)\//)?.[1];
  assert.ok(deliveryId, "governance lease row must carry a lease/personal/GOV-<id>/... id");
  return {
    ...base,
    "docs/plan/PROGRESS.md": (source) => {
      const withLeaseReference = base["docs/plan/PROGRESS.md"](source);
      const activeLeaseRow = withLeaseReference
        .split(/\r?\n/)
        .find((line) => line.startsWith("| Active task lease |"));
      assert.ok(activeLeaseRow, "canonical Active task lease row must exist");
      // The checker strips the lease-reference row and the snapshot heading
      // before looking for the delivery registration, so the fixture must
      // register the delivery in an ordinary snapshot table row.
      const registrationRow = `| ${deliveryId} governance delivery fixture | fixture registration for the injected governance lease | governance/documentation only | fixture |`;
      return withLeaseReference.replace(activeLeaseRow, `${activeLeaseRow}\n${registrationRow}`);
    },
  };
}

test("owner-directed governance lease is accepted without a formal task slice", () => {
  const result = runConsistencyFailureInjection(
    injectGovernanceLease(
      // The fixture delivery id must never collide with a real live lease:
      // injectEvaluationLease asserts its lease-reference rewrite changed the
      // live Active task lease row, which a real-id collision would violate.
      "| `lease/personal/GOV-FIXTURE01/credential-import-boundary` | `GOV-FIXTURE01` owner-directed governance delivery fixture; no formal task/slice | Lane-DOC | `personal/gov-fixture01-fixture` | `docs/governance/AXIOMS.md`; `docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md`; `docs/plan/PROGRESS.md`; `tools/src/check-consistency.mjs`; `tools/test/check.test.mjs`; `personal/handbook/en/developer/conformance-and-testing.md` | governance session fixture | 2026-08-26 / 2026-08-26 | active |",
    ),
  );

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /check-consistency: OK/);
});

test("governance lease rejects unregistered deliveries and product paths", () => {
  const result = runConsistencyFailureInjection(
    injectGovernanceLease(
      "| `lease/personal/GOV-ZZZ9/rogue` | `GOV-ZZZ9` unregistered governance fixture | Lane-DOC | `personal/gov-zzz9-fixture` | `personal/crates/cognitive-runtime/src/lib.rs`; `docs/plan/PROGRESS.md` | governance session fixture | 2026-08-26 / 2026-08-26 | active |",
      { registerDelivery: false },
    ),
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /GOV_LEASE_UNREGISTERED: governance delivery GOV-ZZZ9/);
  assert.match(
    result.stderr,
    /GOV_LEASE_PATH_FORBIDDEN: governance lease lease\/personal\/GOV-ZZZ9\/rogue .* not personal\/crates\/cognitive-runtime\/src\/lib\.rs/,
  );
});

test("governance lease rejects id/description delivery mismatch", () => {
  const result = runConsistencyFailureInjection(
    injectGovernanceLease(
      "| `lease/personal/GOV-A5/mismatched` | owner-directed governance fixture without a delivery token | Lane-DOC | `personal/gov-a5-fixture` | `docs/plan/PROGRESS.md` | governance session fixture | 2026-08-26 / 2026-08-26 | active |",
    ),
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(
    result.stderr,
    /GOV_LEASE_MALFORMED: governance lease lease\/personal\/GOV-A5\/mismatched/,
  );
});

function injectDocumentationLease(leaseRow, { registerDelivery = true } = {}) {
  const base = injectEvaluationLease(leaseRow);
  if (!registerDelivery) {
    return base;
  }
  const leaseId = leaseRow.match(/`(lease\/personal\/[^`]+)`/)?.[1];
  const deliveryId = leaseId?.match(/^lease\/personal\/(DOC-[^/]+)\//)?.[1];
  assert.ok(deliveryId, "documentation lease row must carry a lease/personal/DOC-<id>/... id");
  return {
    ...base,
    "docs/plan/PROGRESS.md": (source) => {
      const withLeaseReference = base["docs/plan/PROGRESS.md"](source);
      const activeLeaseRow = withLeaseReference
        .split(/\r?\n/)
        .find((line) => line.startsWith("| Active task lease |"));
      assert.ok(activeLeaseRow, "canonical Active task lease row must exist");
      const registrationRow = `| ${deliveryId} documentation delivery fixture | fixture registration for the injected documentation lease | documentation only | fixture |`;
      return withLeaseReference.replace(activeLeaseRow, `${activeLeaseRow}\n${registrationRow}`);
    },
  };
}

test("owner-directed documentation lease is accepted without a formal task slice", () => {
  const result = runConsistencyFailureInjection(
    injectDocumentationLease(
      "| `lease/personal/DOC-FIXTURE01/dev-prep` | `DOC-FIXTURE01` owner-directed documentation delivery fixture; no formal task/slice | Lane-DOC | `main` | `docs/plan/PROGRESS.md`; `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`; `tools/src/check-consistency.mjs`; `tools/test/check.test.mjs`; `personal/handbook/en/ai/docs-impact.md`; `docs/checkpoints/2026-08-30-personal-doc-fixture01-report.md` | documentation session fixture | 2026-08-30 / 2026-08-30 | active |",
    ),
  );

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /check-consistency: OK/);
});

test("documentation lease rejects unregistered deliveries and product paths", () => {
  const result = runConsistencyFailureInjection(
    injectDocumentationLease(
      "| `lease/personal/DOC-ZZZ9/rogue` | `DOC-ZZZ9` unregistered documentation fixture | Lane-DOC | `main` | `personal/crates/cognitive-runtime/src/lib.rs`; `docs/checkpoints/`; `docs/plan/PROGRESS.md` | documentation session fixture | 2026-08-30 / 2026-08-30 | active |",
      { registerDelivery: false },
    ),
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /DOC_LEASE_UNREGISTERED: documentation delivery DOC-ZZZ9/);
  assert.match(
    result.stderr,
    /DOC_LEASE_PATH_FORBIDDEN: documentation lease lease\/personal\/DOC-ZZZ9\/rogue .* not personal\/crates\/cognitive-runtime\/src\/lib\.rs/,
  );
  // Only exact report/closure files under docs/checkpoints/ are allowed, never the directory.
  assert.match(
    result.stderr,
    /DOC_LEASE_PATH_FORBIDDEN: documentation lease lease\/personal\/DOC-ZZZ9\/rogue .* not docs\/checkpoints$/m,
  );
});

test("documentation lease rejects id/description delivery mismatch", () => {
  const result = runConsistencyFailureInjection(
    injectDocumentationLease(
      "| `lease/personal/DOC-FIXTURE01/mismatched` | owner-directed documentation fixture without a delivery token | Lane-DOC | `main` | `docs/plan/PROGRESS.md` | documentation session fixture | 2026-08-30 / 2026-08-30 | active |",
    ),
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(
    result.stderr,
    /DOC_LEASE_MALFORMED: documentation lease lease\/personal\/DOC-FIXTURE01\/mismatched/,
  );
});

test("a committed document linking an untracked local file fails the tracked-only link check", () => {
  // The target exists on disk (like the owner's untracked design drafts) but is
  // not in `git ls-files`; the filesystem alone would have accepted the link.
  const untrackedRel = `docs/checkpoints/.p0-t09-untracked-fixture-${process.pid}.md`;
  const untrackedAbs = path.join(repositoryRoot, ...untrackedRel.split("/"));
  writeFileSync(untrackedAbs, "# untracked fixture\n");
  try {
    const trackedList = execFileSync("git", ["-C", repositoryRoot, "ls-files", "--", untrackedRel], {
      encoding: "utf8",
    });
    assert.equal(trackedList.trim(), "", "fixture file must stay untracked");
    const result = runConsistencyFailureInjection({
      "docs/plan/plan.md": (source) =>
        `${source}\n\n[untracked fixture link](../checkpoints/${path.posix.basename(untrackedRel)})\n`,
    });
    assert.equal(result.status, 1, result.stdout);
    assert.match(
      result.stderr,
      /docs\/plan\/plan\.md[\s\S]*broken relative link: \.\.\/checkpoints\/\.p0-t09-untracked-fixture-\d+\.md \(exists locally but is not tracked by Git\)/,
    );
  } finally {
    rmSync(untrackedAbs, { force: true });
  }
});

test("Phase 13 build-order edge sets must match between the formal plan and the dev-prep index", () => {
  const result = runConsistencyFailureInjection({
    "personal/docs/architecture/personal-2.0.0-dev-prep-index.md": (source) => {
      assert.ok(source.includes("  P13T05 --> P13T13\n"), "index graph must contain P13T05 --> P13T13");
      // Drop one formal edge and add one edge the formal plan does not have.
      return source
        .replace("  P13T05 --> P13T13\n", "")
        .replace("  P13T13 --> P11T15\n", "  P13T13 --> P11T15\n  P13T09 --> P13T13\n");
    },
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /BUILD_ORDER_EDGE_MISSING: formal plan edge absent from the dev-prep index graph: T05 --> T13/);
  assert.match(result.stderr, /BUILD_ORDER_EDGE_EXTRA: dev-prep index edge absent from the formal plan graph .*: T09 --> T13/);
  assert.doesNotMatch(result.stderr, /BUILD_ORDER_EDGE_(MISSING|EXTRA)[^\n]*T12b/);
});

test("Phase 13 build-order check distinguishes dashed from solid edges", () => {
  const result = runConsistencyFailureInjection({
    "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md": (source) => {
      assert.ok(source.includes("  T06 -.-> T07\n"), "formal graph must contain the dashed T06 -.-> T07 edge");
      return source.replace("  T06 -.-> T07\n", "  T06 --> T07\n");
    },
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /BUILD_ORDER_EDGE_MISSING: [^\n]*: T06 --> T07/);
  assert.match(result.stderr, /BUILD_ORDER_EDGE_EXTRA: [^\n]*: T06 -\.-> T07/);
});

test("Phase 13 build-order check fails closed when the dev-prep index graph disappears", () => {
  const result = runConsistencyFailureInjection({
    "personal/docs/architecture/personal-2.0.0-dev-prep-index.md": (source) =>
      source.replace("### Phase 13 build order", "### Phase 13 construction sequence"),
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /BUILD_ORDER_GRAPH_MISSING: no「Phase 13 build order」mermaid graph/);
});

test("B01 pass rejects incomplete arithmetic and threshold evidence", () => {
  const result = runConsistencyFailureInjection({
    "docs/plan/PROGRESS.md": (source) =>
      source
        .replace(/(\| B01 first-install\/first-conversation Gate \| \*\*)running(\*\* \|)/, "$1pass$2")
        .replace("5 successes, 1 failure", "4 successes, 1 failure")
        .replace("zero critical safety failures", "one recorded critical safety failure")
        .replace("and an independently verified artifact/signature", "with aggregate statistics and affirmative independent verifier closure"),
  });

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /B01 pass must record success and failure counts that equal the formal denominator/);
  assert.match(result.stderr, /B01 cannot pass below the formal success-count threshold/);
  assert.match(result.stderr, /B01 pass must record success rate, zero critical failures, and aggregate statistics/);
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

test("verify:local orchestrators pin the same conformance counts as ci.yml", () => {
  const repoRoot = path.resolve(toolsDir, "..");
  const ci = readFileSync(path.join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const posixScript = readFileSync(path.join(repoRoot, "scripts", "v01-auto-run.sh"), "utf8");
  const windowsScript = readFileSync(path.join(repoRoot, "scripts", "v01-auto-run.ps1"), "utf8");

  const ciPins = ci.match(
    /const pinned = \{ total_vectors: (\d+), pass: (\d+), fail: (\d+), 'not-applicable': (\d+), 'documented-degradation': (\d+), 'not-run': (\d+) \}/,
  );
  assert.ok(ciPins, "ci.yml must pin the five-state conformance counts");
  const ciSelfCheckMin = ci.match(/r\.must_flip\.length < (\d+)/);
  assert.ok(ciSelfCheckMin, "ci.yml must pin the self-check corpus floor");
  const expected = {
    total: Number(ciPins[1]),
    pass: Number(ciPins[2]),
    fail: Number(ciPins[3]),
    notApplicable: Number(ciPins[4]),
    documentedDegradation: Number(ciPins[5]),
    notRun: Number(ciPins[6]),
  };

  const posixPins = posixScript.match(
    /PIN_TOTAL=(\d+) PIN_PASS=(\d+) PIN_FAIL=(\d+) PIN_NA=(\d+) PIN_DD=(\d+) PIN_NR=(\d+) PIN_SC=(\d+)/,
  );
  assert.ok(posixPins, "v01-auto-run.sh must carry the PIN_* line");
  assert.deepEqual(
    posixPins.slice(1, 7).map(Number),
    Object.values(expected),
    "v01-auto-run.sh pins drifted from ci.yml",
  );
  assert.ok(Number(posixPins[7]) >= Number(ciSelfCheckMin[1]), "v01-auto-run.sh self-check floor below ci.yml");

  const readWindowsPin = (key) => {
    const match = windowsScript.match(new RegExp(`^\\s*"?${key}"?\\s*=\\s*(\\d+)\\s*$`, "m"));
    assert.ok(match, `v01-auto-run.ps1 must pin ${key}`);
    return Number(match[1]);
  };
  assert.deepEqual(
    {
      total: readWindowsPin("total_vectors"),
      pass: readWindowsPin("pass"),
      fail: readWindowsPin("fail"),
      notApplicable: readWindowsPin("not-applicable"),
      documentedDegradation: readWindowsPin("documented-degradation"),
      notRun: readWindowsPin("not-run"),
    },
    expected,
    "v01-auto-run.ps1 pins drifted from ci.yml",
  );
  assert.ok(readWindowsPin("self_check_min") >= Number(ciSelfCheckMin[1]), "v01-auto-run.ps1 self-check floor below ci.yml");
});
