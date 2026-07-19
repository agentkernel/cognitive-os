/**
 * Shared helpers for the repo consistency tools.
 * History/ is a frozen archive: it is never scanned, loaded, or referenced.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

export const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

/** Directories the scanners may enter. History/ and build outputs are excluded by design. */
const SCAN_ROOTS = ["specs", "conformance", "docs", "crates", "packages", "apps", "tests", "tools"];
const EXCLUDED_DIR_NAMES = new Set([
  "History",
  "node_modules",
  "target",
  "dist",
  ".git",
  "artifacts",
]);

export function repoPath(...segments) {
  return path.join(REPO_ROOT, ...segments);
}

export function toRepoRelative(absPath) {
  return path.relative(REPO_ROOT, absPath).split(path.sep).join("/");
}

export function readText(absPath) {
  return readFileSync(absPath, "utf-8");
}

export function readJson(absPath) {
  return JSON.parse(readText(absPath));
}

export function readYaml(absPath) {
  return YAML.parse(readText(absPath));
}

/** Recursively list files under a repo-relative root, honoring exclusions. */
export function listFiles(rootRel, predicate = () => true) {
  const out = [];
  const walk = (abs) => {
    for (const entry of readdirSync(abs, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        if (!EXCLUDED_DIR_NAMES.has(entry.name)) {
          walk(path.join(abs, entry.name));
        }
      } else if (predicate(entry.name)) {
        out.push(path.join(abs, entry.name));
      }
    }
  };
  const absRoot = repoPath(rootRel);
  if (statSync(absRoot, { throwIfNoEntry: false })?.isDirectory()) {
    walk(absRoot);
  }
  return out.sort();
}

/** All scannable markdown files (repo docs + root-level docs). */
export function listMarkdownFiles() {
  const rootMd = readdirSync(REPO_ROOT, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith(".md"))
    .map((e) => path.join(REPO_ROOT, e.name));
  const nested = SCAN_ROOTS.flatMap((root) => listFiles(root, (name) => name.endsWith(".md")));
  return [...rootMd, ...nested].sort();
}

/** Load the registries once. */
export function loadRegistries() {
  const requirements = readYaml(repoPath("specs", "registry", "requirements.yaml"));
  const errors = readYaml(repoPath("specs", "registry", "errors.yaml"));
  const stateDomains = readYaml(repoPath("specs", "registry", "state-domains.yaml"));
  return {
    requirements,
    errors,
    stateDomains,
    requirementIds: new Set(requirements.requirements.map((r) => r.id)),
    errorCodes: new Set(errors.errors.map((e) => e.code)),
  };
}

/** Load every conformance vector with its repo-relative path. */
export function loadVectors() {
  return listFiles("conformance/vectors", (name) => name.endsWith(".json")).map((abs) => ({
    path: toRepoRelative(abs),
    abs,
    doc: readJson(abs),
  }));
}

/** Load every schema with its repo-relative path. */
export function loadSchemas() {
  return listFiles("specs/schemas", (name) => name.endsWith(".json")).map((abs) => ({
    path: toRepoRelative(abs),
    abs,
    name: path.basename(abs),
    doc: readJson(abs),
  }));
}
