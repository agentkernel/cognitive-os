/**
 * Shared helpers for the repo consistency tools.
 * History/ is a frozen archive: it is never scanned, loaded, or referenced.
 */

import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

export const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

/**
 * Git-tracked path index (P0-T09): existence checks use `git ls-files`, not
 * the filesystem, so a committed document that links an untracked local file
 * fails on the author's machine exactly as it fails in CI.
 *
 * Returns `{ files: Set<string>, directories: Set<string> }` of repo-relative
 * POSIX paths, or `null` when `root` is not a Git checkout (callers must fail
 * closed or label the fallback; they must not silently use the filesystem).
 */
export function loadTrackedPaths(root = REPO_ROOT) {
  let output;
  try {
    output = execFileSync("git", ["-C", root, "ls-files", "-z"], {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return null;
  }
  const files = new Set();
  const directories = new Set();
  for (const entry of output.split("\0")) {
    if (!entry) continue;
    files.add(entry);
    const segments = entry.split("/");
    for (let depth = 1; depth < segments.length; depth += 1) {
      directories.add(segments.slice(0, depth).join("/"));
    }
  }
  return { files, directories };
}

/** True when `rel` (repo-relative POSIX path) is a tracked file or a directory containing tracked files. */
export function isTrackedPath(tracked, rel) {
  const normalized = rel.replaceAll("\\", "/").replace(/\/+$/, "");
  return tracked.files.has(normalized) || tracked.directories.has(normalized);
}

/**
 * Directories the scanners may enter (ADR-0054 subproject roots). History/,
 * build outputs, the imported clients/ project (own governance), and the
 * handbook (own checker) are excluded by design.
 */
const SCAN_ROOTS = ["core", "personal", "enterprise", "docs", "tools"];
const EXCLUDED_DIR_NAMES = new Set([
  "History",
  "node_modules",
  "target",
  "dist",
  ".git",
  "artifacts",
  "handbook",
]);

export function repoPath(...segments) {
  return path.join(REPO_ROOT, ...segments);
}

export function toRepoRelative(absPath) {
  return path.relative(REPO_ROOT, absPath).split(path.sep).join("/");
}

export function readText(absPath) {
  const overrideDirectory = process.env.COGNITIVEOS_CONSISTENCY_OVERRIDE_DIR;
  const repositoryRelativePath = toRepoRelative(absPath);
  const pathIsInsideRepository =
    repositoryRelativePath !== ".." && !repositoryRelativePath.startsWith("../");
  if (overrideDirectory && pathIsInsideRepository) {
    const overridePath = path.join(overrideDirectory, ...repositoryRelativePath.split("/"));
    if (statSync(overridePath, { throwIfNoEntry: false })?.isFile()) {
      return readFileSync(overridePath, "utf-8");
    }
  }
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

/**
 * All scannable markdown files (repo docs + root-level docs). When a tracked
 * index is supplied, untracked local markdown is skipped so the scan set is
 * identical on the author's machine and on a clean CI checkout.
 */
export function listMarkdownFiles(tracked = undefined) {
  const rootMd = readdirSync(REPO_ROOT, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith(".md"))
    .map((e) => path.join(REPO_ROOT, e.name));
  const nested = SCAN_ROOTS.flatMap((root) => listFiles(root, (name) => name.endsWith(".md")));
  const all = [...rootMd, ...nested].sort();
  if (!tracked) {
    return all;
  }
  return all.filter((abs) => tracked.files.has(toRepoRelative(abs)));
}

/** Load the registries once. */
export function loadRegistries() {
  const requirements = readYaml(repoPath("core", "specs", "registry", "requirements.yaml"));
  const errors = readYaml(repoPath("core", "specs", "registry", "errors.yaml"));
  const stateDomains = readYaml(repoPath("core", "specs", "registry", "state-domains.yaml"));
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
  return listFiles("core/conformance/vectors", (name) => name.endsWith(".json")).map((abs) => ({
    path: toRepoRelative(abs),
    abs,
    doc: readJson(abs),
  }));
}

/** Load every schema with its repo-relative path. */
export function loadSchemas() {
  return listFiles("core/specs/schemas", (name) => name.endsWith(".json")).map((abs) => ({
    path: toRepoRelative(abs),
    abs,
    name: path.basename(abs),
    doc: readJson(abs),
  }));
}
