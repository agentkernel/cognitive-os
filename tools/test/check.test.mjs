import assert from "node:assert/strict";
import { test } from "node:test";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

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
