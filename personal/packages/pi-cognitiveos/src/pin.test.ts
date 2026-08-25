/**
 * Pi compatibility pin drift guard.
 *
 * `personal/apps/pi-agent-adapter/src/lib.rs` holds the authoritative
 * `PiCompatibilityPin::expected()`. If this package's mirror drifts from it,
 * the TypeScript surface would claim compatibility with a version the Rust
 * admission path refuses — so the mirror is compared field by field against the
 * Rust source text.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { PI_COMPATIBILITY_PIN } from "./pin.js";

function repositoryRoot(): string {
  let directory = path.dirname(fileURLToPath(import.meta.url));
  for (let depth = 0; depth < 10; depth += 1) {
    try {
      readFileSync(path.join(directory, "pnpm-workspace.yaml"), "utf8");
      return directory;
    } catch {
      const parent = path.dirname(directory);
      if (parent === directory) {
        break;
      }
      directory = parent;
    }
  }
  throw new Error("cannot locate the repository root from the compiled test location");
}

function rustPinField(source: string, fieldName: string): string {
  const match = new RegExp(`${fieldName}:\\s*"([^"]*)"`).exec(source);
  assert.ok(match, `Rust PiCompatibilityPin has no ${fieldName} literal`);
  return match[1] ?? "";
}

test("the TypeScript Pi pin matches the Rust PiCompatibilityPin field by field", () => {
  const rustSource = readFileSync(
    path.join(repositoryRoot(), "personal", "apps", "pi-agent-adapter", "src", "lib.rs"),
    "utf8",
  );

  assert.equal(rustPinField(rustSource, "package_version"), PI_COMPATIBILITY_PIN.packageVersion);
  assert.equal(rustPinField(rustSource, "npm_integrity"), PI_COMPATIBILITY_PIN.npmIntegrity);
  assert.equal(rustPinField(rustSource, "source_commit"), PI_COMPATIBILITY_PIN.sourceCommit);
  assert.equal(rustPinField(rustSource, "repository_url"), PI_COMPATIBILITY_PIN.repositoryUrl);
  assert.equal(
    rustPinField(rustSource, "repository_directory"),
    PI_COMPATIBILITY_PIN.repositoryDirectory,
  );
  assert.equal(rustPinField(rustSource, "node_engine"), PI_COMPATIBILITY_PIN.nodeEngine);
});

test("the pinned package name is the one the integration plan names", () => {
  assert.equal(PI_COMPATIBILITY_PIN.packageName, "@earendil-works/pi-coding-agent");
  const planSource = readFileSync(
    path.join(repositoryRoot(), "docs", "plan", "archive", "PI-AGENT-INTEGRATION-PLAN.md"),
    "utf8",
  );
  assert.ok(planSource.includes(PI_COMPATIBILITY_PIN.packageName));
  assert.ok(planSource.includes(PI_COMPATIBILITY_PIN.packageVersion));
});

test("Pi is not a runtime or lockfile dependency of this package", () => {
  const manifest: unknown = JSON.parse(
    readFileSync(
      path.join(repositoryRoot(), "personal", "packages", "pi-cognitiveos", "package.json"),
      "utf8",
    ),
  );
  assert.ok(typeof manifest === "object" && manifest !== null);
  const record = manifest as Record<string, unknown>;
  const dependencies = JSON.stringify(record["dependencies"] ?? {});
  const devDependencies = JSON.stringify(record["devDependencies"] ?? {});
  // ADR-0025: do not vendor or redistribute Pi. The user installs Pi locally.
  assert.ok(!dependencies.includes("earendil-works"));
  assert.ok(!devDependencies.includes("earendil-works"));

  const lockfile = readFileSync(path.join(repositoryRoot(), "pnpm-lock.yaml"), "utf8");
  assert.ok(
    !lockfile.includes("earendil-works"),
    "adding Pi to the lockfile would pull it into every workspace install and CI job",
  );
});
