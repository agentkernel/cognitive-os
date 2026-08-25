/**
 * Docs-sync gate: enforce documentation-system synchronization before every
 * commit, push, and merge (docs/standards/docs-sync-contract.md §2/§5;
 * personal/handbook/_meta/sync-policy.md).
 *
 * The gate routes changed paths through personal/handbook/_meta/source-map.json:
 *
 * - no documentation-relevant change        -> fast skip (exit 0, no checks);
 * - personal/handbook/** touched                     -> run the handbook check set;
 * - mapped implementation sources changed
 *   WITH handbook changes in the same set   -> run the handbook check set;
 *   WITHOUT any handbook change             -> fail closed, unless the change
 *     is explicitly acknowledged as documentation-neutral via the environment
 *     variable DOCS_IMPACT_NONE="<concrete reason>" — the reason is echoed and
 *     must also be recorded in the commit/PR description.
 *
 * The handbook check set is `check-handbook.mjs` (manifest, locale pairing,
 * links, sources, fingerprints, coverage, forbidden content) plus
 * `generate-handbook.mjs --check` (generated-page byte equality). Fingerprints
 * make silent drift fail even when this gate is bypassed; CI runs the full set
 * unconditionally as the final backstop.
 *
 * Modes:
 *   --staged        pre-commit: staged paths (index vs HEAD)
 *   --push          pre-push: merge-base(@{upstream} | origin/main)..HEAD
 *   --range [A...B] explicit range; defaults to origin/main...HEAD
 *
 * This tool creates no task, Gate, contract, or release semantics.
 */

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { compileGlob } from "./handbook-lib.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

/** Paths that count as "the documentation system itself was updated". */
const DOCS_SURFACE_PREFIXES = ["personal/handbook/"];
const DOCS_SURFACE_FILES = new Set([
  "llms.txt",
  ".cursor/rules/20-cognitiveos-personal-handbook-sync.mdc",
]);

/**
 * Pure routing: match changed paths against source-map rules.
 * Returns { impacted: [{ id, docs, matchedPaths }], docsTouched }.
 */
export function routeChangedPaths(changedPaths, sourceMap) {
  const impacted = [];
  for (const rule of sourceMap.rules ?? []) {
    const regexes = (rule.sources ?? []).map((glob) => compileGlob(glob));
    const matchedPaths = changedPaths.filter((p) => regexes.some((r) => r.test(p)));
    if (matchedPaths.length > 0) {
      impacted.push({ id: rule.id, docs: rule.docs ?? [], matchedPaths });
    }
  }
  const docsTouched = changedPaths.some(
    (p) => DOCS_SURFACE_PREFIXES.some((prefix) => p.startsWith(prefix)) || DOCS_SURFACE_FILES.has(p),
  );
  return { impacted, docsTouched };
}

/**
 * Pure verdict:
 *  - "skip"         nothing documentation-relevant changed;
 *  - "check"        run the handbook check set;
 *  - "acknowledged" mapped sources changed without docs, but an explicit
 *                   DOCS_IMPACT_NONE reason was provided (checks still run);
 *  - "fail"         mapped sources changed without docs and without a reason.
 */
export function decideDocsSync({ impacted, docsTouched, allowNoneReason }) {
  if (impacted.length === 0 && !docsTouched) {
    return { verdict: "skip" };
  }
  if (impacted.length > 0 && !docsTouched) {
    if (allowNoneReason && allowNoneReason.trim().length > 3) {
      return { verdict: "acknowledged", reason: allowNoneReason.trim() };
    }
    return { verdict: "fail" };
  }
  return { verdict: "check" };
}

function git(...args) {
  return execFileSync("git", ["-C", REPO_ROOT, ...args], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

function changedPathsFor(mode, explicitRange) {
  if (mode === "--staged") {
    return git("diff", "--cached", "--name-only", "-z").split("\0").filter(Boolean);
  }
  let range = explicitRange;
  if (!range) {
    let base = "origin/main";
    if (mode === "--push") {
      try {
        base = git("rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}").trim();
      } catch {
        base = "origin/main";
      }
    }
    let mergeBase;
    try {
      mergeBase = git("merge-base", base, "HEAD").trim();
    } catch {
      // No usable base (fresh clone/detached): check everything relevant.
      return null;
    }
    range = `${mergeBase}..HEAD`;
  }
  return git("diff", "--name-only", "-z", range).split("\0").filter(Boolean);
}

function runHandbookCheckSet() {
  execFileSync(process.execPath, [path.join(REPO_ROOT, "tools", "src", "check-handbook.mjs")], {
    stdio: "inherit",
    cwd: REPO_ROOT,
  });
  execFileSync(
    process.execPath,
    [path.join(REPO_ROOT, "tools", "src", "generate-handbook.mjs"), "--check"],
    { stdio: "inherit", cwd: REPO_ROOT },
  );
}

function main() {
  const argv = process.argv.slice(2);
  const mode = argv.find((a) => a === "--staged" || a === "--push" || a === "--range") ?? "--range";
  const explicitRange = mode === "--range" ? argv[argv.indexOf("--range") + 1] : undefined;

  const sourceMapPath = path.join(REPO_ROOT, "personal", "handbook", "_meta", "source-map.json");
  if (!existsSync(sourceMapPath)) {
    console.log("docs-sync-gate: personal/handbook/_meta/source-map.json not present on this revision; nothing to enforce");
    return;
  }
  const sourceMap = JSON.parse(readFileSync(sourceMapPath, "utf8"));

  const changedPaths = changedPathsFor(mode, explicitRange);
  if (changedPaths === null) {
    console.log("docs-sync-gate: no comparable base revision; running the handbook check set defensively");
    runHandbookCheckSet();
    return;
  }
  if (changedPaths.length === 0) {
    console.log("docs-sync-gate: no changes in scope");
    return;
  }

  const route = routeChangedPaths(changedPaths, sourceMap);
  const decision = decideDocsSync({
    ...route,
    allowNoneReason: process.env.DOCS_IMPACT_NONE,
  });

  if (decision.verdict === "skip") {
    console.log(`docs-sync-gate: no documentation-relevant changes (${changedPaths.length} path(s) checked)`);
    return;
  }

  if (route.impacted.length > 0) {
    console.log("docs-sync-gate: changed paths map to handbook documents:");
    for (const rule of route.impacted) {
      console.log(`  [${rule.id}] ${rule.matchedPaths.length} path(s) -> ${rule.docs.join(", ")}`);
    }
  }

  if (decision.verdict === "fail") {
    console.error(
      [
        "",
        "docs-sync-gate: FAILED — mapped implementation sources changed but no handbook update is part of this change set.",
        "Synchronize the documentation system in the same delivery:",
        "  1. update the mapped pages above in BOTH locales (personal/handbook/en + personal/handbook/zh-CN),",
        "  2. regenerate generated pages:   node tools/src/generate-handbook.mjs",
        "  3. refresh page fingerprints:    node tools/src/fill-handbook-fingerprints.mjs",
        "  4. classify any new files in     personal/handbook/_meta/source-coverage.json,",
        "  5. verify:                       pnpm run check:handbook",
        "If this change genuinely affects no documentation, acknowledge it explicitly:",
        '  DOCS_IMPACT_NONE="<concrete reason>" and record the same reason in the commit/PR description.',
        "Canonical obligations: docs/standards/docs-sync-contract.md §2; personal/handbook/_meta/sync-policy.md.",
      ].join("\n"),
    );
    process.exit(1);
  }

  if (decision.verdict === "acknowledged") {
    console.log(
      `docs-sync-gate: docs-impact: none acknowledged — "${decision.reason}" (record this reason in the commit/PR description)`,
    );
  }

  runHandbookCheckSet();
  console.log("docs-sync-gate: OK");
}

const invokedDirectly = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (invokedDirectly) {
  main();
}
