/**
 * Standalone handbook consistency checker (CI gate; see handbook/_meta/).
 *
 * Verifies that the informative handbook layer stays machine-consistent with
 * the tracked source tree: manifest/schema validity, doc-id uniqueness and
 * locale pairing, link and source-path existence, stable-symbol presence,
 * per-page source fingerprints, total coverage classification of every
 * tracked file, generated-page byte equality, capability-status legality,
 * and the absence of secret-shaped or copied dynamic-status content.
 *
 * Exit code 0 = green; 1 = at least one violation, each printed with rule id,
 * file, and reason. History/ is never scanned. The checker itself creates no
 * task, Gate, contract, or release facts.
 *
 * Usage:
 *   node tools/src/check-handbook.mjs [--diff-base <rev>]
 *
 * `--diff-base` additionally verifies (for the handbook task's own PR) that
 * legacy documentation changed only on the allowlisted governance paths.
 */

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Ajv2020 } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import YAML from "yaml";
import { runHandbookChecks, splitFrontmatter, computeSourceSetDigest, normalizeEol } from "./handbook-lib.mjs";
import { buildGeneratedPages } from "./generate-handbook.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const repoPath = (...segments) => path.join(REPO_ROOT, ...segments);

function git(...args) {
  return execFileSync("git", ["-C", REPO_ROOT, ...args], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

function readRepoText(rel) {
  return readFileSync(repoPath(...rel.split("/")), "utf8");
}

function revisionAvailable(revision) {
  try {
    execFileSync("git", ["-C", REPO_ROOT, "cat-file", "-e", `${revision}^{tree}`], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function main() {
  const argv = process.argv.slice(2);
  const diffBaseIndex = argv.indexOf("--diff-base");
  const diffBase = diffBaseIndex >= 0 ? argv[diffBaseIndex + 1] : null;

  if (!existsSync(repoPath("handbook", "_meta", "manifest.json"))) {
    console.error("check-handbook: handbook/_meta/manifest.json not found");
    process.exit(1);
  }

  const manifest = JSON.parse(readRepoText("handbook/_meta/manifest.json"));
  const coverage = JSON.parse(readRepoText("handbook/_meta/source-coverage.json"));
  const sourceMap = JSON.parse(readRepoText("handbook/_meta/source-map.json"));
  const frontmatterSchema = JSON.parse(readRepoText("handbook/_meta/handbook-frontmatter.schema.json"));

  const ajv = new Ajv2020({ strict: false, allErrors: true });
  addFormats(ajv);
  const frontmatterSchemaValidate = ajv.compile(frontmatterSchema);

  // -z avoids C-style quoting of non-ASCII paths (History/… archive names).
  const trackedPaths = git("ls-files", "-z").split("\0").filter(Boolean);
  const handbookFiles = trackedPaths.filter((p) => p.startsWith("handbook/"));

  const pages = new Map();
  for (const filePath of handbookFiles) {
    if (!filePath.endsWith(".md")) continue;
    const raw = readRepoText(filePath);
    const { yamlText, body } = splitFrontmatter(raw);
    let frontmatter = null;
    let frontmatterError = null;
    if (yamlText !== null) {
      try {
        frontmatter = YAML.parse(yamlText);
      } catch (err) {
        frontmatterError = err.message;
      }
    }
    pages.set(filePath, { frontmatter, frontmatterError, body, raw });
  }

  let generatedOutputs = null;
  let generatorFailure = null;
  try {
    generatedOutputs = buildGeneratedPages({ readSource: readRepoText, manifest, trackedPaths });
  } catch (err) {
    generatorFailure = `generator failed: ${err.message}`;
  }

  const diagnostics = runHandbookChecks({
    manifest,
    frontmatterSchemaValidate,
    pages,
    trackedPaths,
    coverage,
    sourceMap,
    readSource: readRepoText,
    generatedOutputs,
    handbookFiles,
  });
  if (generatorFailure) {
    diagnostics.push({ rule: "HB010", file: "tools/src/generate-handbook.mjs", message: generatorFailure });
  }

  // ---- HB013: source-set record is reproducible from its recorded revision ------
  // Shallow CI checkouts usually lack the baseline commit object; reproducibility
  // is then skipped with a notice and enforced on full clones (authoring/closure).
  try {
    const sourceSet = JSON.parse(readRepoText("handbook/_meta/source-set.json"));
    const revision = sourceSet.implementation_baseline_revision;
    if (!/^[0-9a-f]{40}$/.test(revision ?? "")) {
      diagnostics.push({ rule: "HB013", file: "handbook/_meta/source-set.json", message: "implementation_baseline_revision must be a full 40-hex commit" });
    } else if (!revisionAvailable(revision)) {
      console.log(`check-handbook: note — source-set baseline ${revision.slice(0, 12)} is not in this (shallow) clone; digest reproducibility skipped here and verified on full clones`);
    } else {
      const lsTree = git("ls-tree", "-r", revision);
      const entries = [];
      for (const line of lsTree.split("\n")) {
        if (!line) continue;
        const [meta, filePath] = line.split("\t");
        const blob = meta.split(" ")[2];
        if (
          filePath.startsWith("handbook/") ||
          filePath === "llms.txt" ||
          filePath === "docs/plan/PROGRESS.md" ||
          filePath === "docs/plan/PARALLEL-LANES.md" ||
          filePath === "Cargo.lock" ||
          filePath === "pnpm-lock.yaml" ||
          filePath.startsWith("History/") ||
          filePath.startsWith("\"History")
        ) {
          continue;
        }
        entries.push({ path: filePath, blob });
      }
      const digest = computeSourceSetDigest(entries);
      if (digest !== sourceSet.digest) {
        diagnostics.push({ rule: "HB013", file: "handbook/_meta/source-set.json", message: `source-set digest is not reproducible from ${revision.slice(0, 12)} (recorded ${sourceSet.digest?.slice(0, 18)}…, computed ${digest.slice(0, 18)}…)` });
      }
    }
  } catch (err) {
    diagnostics.push({ rule: "HB013", file: "handbook/_meta/source-set.json", message: `unreadable or unverifiable: ${err.message}` });
  }

  // ---- HB014 (optional): legacy docs unchanged except the allowlist --------------
  if (diffBase) {
    try {
      const allowlist = JSON.parse(readRepoText("handbook/_meta/legacy-change-allowlist.json"));
      const allowed = new Set(allowlist.allowed_paths ?? []);
      const changed = git("diff", "--name-only", `${diffBase}...HEAD`).split("\n").filter(Boolean);
      for (const changedPath of changed) {
        const isNewSurface =
          changedPath.startsWith("handbook/") ||
          changedPath === "llms.txt" ||
          changedPath.startsWith("tools/") ||
          changedPath === ".cursor/rules/20-cognitiveos-personal-handbook-sync.mdc" ||
          changedPath === ".github/workflows/ci.yml" ||
          changedPath === "package.json";
        if (!isNewSurface && !allowed.has(changedPath)) {
          diagnostics.push({ rule: "HB014", file: changedPath, message: "legacy path changed outside the recorded governance allowlist for the handbook task" });
        }
      }
    } catch (err) {
      diagnostics.push({ rule: "HB014", file: "handbook/_meta/legacy-change-allowlist.json", message: `diff-base verification failed: ${err.message}` });
    }
  }

  if (diagnostics.length > 0) {
    console.error(`check-handbook: ${diagnostics.length} violation(s)\n`);
    for (const { rule, file, message } of diagnostics) {
      console.error(`  [${rule}] ${file}\n    ${message}`);
    }
    process.exit(1);
  }
  const generatedCount = (manifest.documents ?? []).filter((d) => d.generated).length;
  console.log(
    `check-handbook: OK (${(manifest.documents ?? []).length} documents x ${(manifest.locales ?? []).length} locales, ` +
      `${generatedCount} generated, coverage/link/fingerprint/status/secret checks verified)`,
  );
}

main();
