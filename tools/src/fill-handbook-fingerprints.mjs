/**
 * Recompute and rewrite the `fingerprint:` line of hand-authored handbook
 * pages from their declared `sources[]`. Run after editing a page or its
 * mapped sources; generated pages are owned by generate-handbook.mjs instead.
 *
 * Usage: node tools/src/fill-handbook-fingerprints.mjs [--check]
 */

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";
import { computePageFingerprint, normalizeEol, splitFrontmatter } from "./handbook-lib.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const repoPath = (...segments) => path.join(REPO_ROOT, ...segments);
const check = process.argv.includes("--check");

const readSource = (rel) => readFileSync(repoPath(...rel.split("/")), "utf8");
const trackedHandbookPages = execFileSync("git", ["-C", REPO_ROOT, "ls-files", "personal/handbook"], {
  encoding: "utf8",
})
  .split("\n")
  .filter((p) => p.endsWith(".md"));

let updated = 0;
let drifted = 0;
for (const pagePath of trackedHandbookPages) {
  const raw = readSource(pagePath);
  const { yamlText } = splitFrontmatter(raw);
  if (!yamlText) continue;
  let frontmatter;
  try {
    frontmatter = YAML.parse(yamlText);
  } catch {
    continue; // the checker reports parse failures
  }
  if (!frontmatter?.sources?.length || frontmatter.generated === true) continue;
  const computed = computePageFingerprint(
    frontmatter.sources.map((s) => s.path),
    readSource,
  );
  if (frontmatter.fingerprint === computed) continue;
  if (check) {
    console.error(`fingerprint drift: ${pagePath}`);
    drifted += 1;
    continue;
  }
  const normalized = normalizeEol(raw);
  const replaced = normalized.replace(/^fingerprint: "sha256:[0-9a-fA-F]{64}"$|^fingerprint: "sha256:PENDING"$/m, `fingerprint: "${computed}"`);
  if (replaced === normalized) {
    console.error(`could not locate fingerprint line in ${pagePath}`);
    drifted += 1;
    continue;
  }
  writeFileSync(repoPath(...pagePath.split("/")), replaced, "utf8");
  console.log(`updated ${pagePath}`);
  updated += 1;
}

if (check && drifted > 0) process.exit(1);
console.log(check ? "fill-handbook-fingerprints --check: OK" : `fill-handbook-fingerprints: ${updated} page(s) updated`);
