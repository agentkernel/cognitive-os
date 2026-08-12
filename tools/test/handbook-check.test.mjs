import assert from "node:assert/strict";
import { test } from "node:test";
import { Ajv2020 } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  compileGlob,
  computePageFingerprint,
  runHandbookChecks,
  splitFrontmatter,
} from "../src/handbook-lib.mjs";

const toolsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(toolsDir, "..");

const ajv = new Ajv2020({ strict: false, allErrors: true });
addFormats(ajv);
const frontmatterSchema = JSON.parse(
  readFileSync(path.join(repoRoot, "handbook", "_meta", "handbook-frontmatter.schema.json"), "utf8"),
);
const validateFrontmatter = ajv.compile(frontmatterSchema);

/** Build a fully green in-memory fixture; tests mutate copies of it. */
function greenFixture() {
  const sourceText = 'pub fn admit_pi_launch() {}\n';
  const readSource = (p) => {
    if (p === "crates/demo/src/lib.rs") return sourceText;
    throw new Error(`unreadable ${p}`);
  };
  const fingerprint = computePageFingerprint(["crates/demo/src/lib.rs"], readSource);
  const frontmatter = {
    doc_id: "user.demo",
    kind: "concept",
    audience: ["user"],
    status: "implemented",
    generated: false,
    sources: [{ path: "crates/demo/src/lib.rs", symbols: ["admit_pi_launch"] }],
    contracts: ["specs/schemas/demo.schema.json"],
    tests: ["crates/demo/tests/demo.rs"],
    fingerprint,
    non_claims: ["No Gate, release, or Profile claim."],
  };
  const page = (locale, extraBody = "") => ({
    frontmatter: { ...frontmatter, locale },
    frontmatterError: null,
    body: `Body with a [good link](../../../crates/demo/src/lib.rs).\n${extraBody}`,
    raw: "",
  });
  const pages = new Map([
    ["handbook/en/user/demo.md", page("en")],
    ["handbook/zh-CN/user/demo.md", page("zh-CN")],
    ["handbook/README.md", { frontmatter: null, frontmatterError: null, body: "root nav", raw: "" }],
  ]);
  const trackedPaths = [
    "handbook/README.md",
    "handbook/en/user/demo.md",
    "handbook/zh-CN/user/demo.md",
    "handbook/_meta/manifest.json",
    "crates/demo/src/lib.rs",
    "crates/demo/tests/demo.rs",
    "specs/schemas/demo.schema.json",
    "Cargo.lock",
  ];
  return {
    manifest: {
      root_entry: "handbook/README.md",
      locales: ["en", "zh-CN"],
      locale_roots: { en: "handbook/en", "zh-CN": "handbook/zh-CN" },
      documents: [
        { doc_id: "user.demo", kind: "concept", audience: ["user"], generated: false, rel_path: "user/demo.md" },
      ],
    },
    frontmatterSchemaValidate: validateFrontmatter,
    pages,
    trackedPaths,
    coverage: {
      rules: [
        { glob: "handbook/**", category: "handbook", docs: ["user.demo"] },
        { glob: "crates/**", category: "first-party-source", docs: ["user.demo"] },
        { glob: "specs/**", category: "first-party-source", docs: ["user.demo"] },
        { glob: "Cargo.lock", category: "excluded-lock-payload", reason: "dependency lock" },
      ],
    },
    sourceMap: { rules: [{ id: "demo", sources: ["crates/demo/**"], docs: ["user.demo"] }] },
    readSource,
    generatedOutputs: null,
    handbookFiles: ["handbook/README.md", "handbook/en/user/demo.md", "handbook/zh-CN/user/demo.md"],
  };
}

function rulesOf(diagnostics) {
  return diagnostics.map((d) => d.rule);
}

test("green fixture passes every handbook check", () => {
  const diagnostics = runHandbookChecks(greenFixture());
  assert.deepEqual(diagnostics, []);
});

test("missing source path fails HB006", () => {
  const fixture = greenFixture();
  fixture.pages.get("handbook/en/user/demo.md").frontmatter.sources = [{ path: "crates/demo/src/missing.rs" }];
  assert.ok(rulesOf(runHandbookChecks(fixture)).includes("HB006"));
});

test("stale fingerprint fails HB008 with drift wording", () => {
  const fixture = greenFixture();
  const fm = fixture.pages.get("handbook/en/user/demo.md").frontmatter;
  fm.fingerprint = "sha256:" + "0".repeat(64);
  const diagnostics = runHandbookChecks(fixture);
  const hit = diagnostics.find((d) => d.rule === "HB008");
  assert.ok(hit, "expected HB008");
  assert.match(hit.message, /drift/);
});

test("broken relative link fails HB005", () => {
  const fixture = greenFixture();
  fixture.pages.get("handbook/en/user/demo.md").body = "[dead](../../does/not/exist.md)";
  assert.ok(rulesOf(runHandbookChecks(fixture)).includes("HB005"));
});

test("link into History/ fails HB005", () => {
  const fixture = greenFixture();
  fixture.trackedPaths.push("History/old.md");
  fixture.coverage.rules.push({ glob: "History/**", category: "excluded-frozen-history", reason: "frozen" });
  fixture.pages.get("handbook/en/user/demo.md").body = "[old](../../../History/old.md)";
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(diagnostics.some((d) => d.rule === "HB005" && /History/.test(d.message)));
});

test("new tracked file without a coverage rule fails HB009", () => {
  const fixture = greenFixture();
  fixture.trackedPaths.push("newtree/widget.rs");
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(diagnostics.some((d) => d.rule === "HB009" && d.file === "newtree/widget.rs"));
});

test("illegal capability status fails schema validation (HB002)", () => {
  const fixture = greenFixture();
  fixture.pages.get("handbook/en/user/demo.md").frontmatter.status = "done";
  assert.ok(rulesOf(runHandbookChecks(fixture)).includes("HB002"));
});

test("missing locale twin fails HB003", () => {
  const fixture = greenFixture();
  fixture.pages.delete("handbook/zh-CN/user/demo.md");
  fixture.handbookFiles = fixture.handbookFiles.filter((p) => !p.startsWith("handbook/zh-CN/"));
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(rulesOf(diagnostics).includes("HB003"));
});

test("generated page drift fails HB010", () => {
  const fixture = greenFixture();
  fixture.manifest.documents.push({ doc_id: "ref.demo", kind: "reference", audience: ["user"], generated: true, rel_path: "reference/demo.md" });
  const generatedPage = {
    frontmatter: {
      doc_id: "ref.demo", locale: "en", kind: "reference", audience: ["user"], status: "implemented",
      generated: true, sources: [{ path: "crates/demo/src/lib.rs" }],
      fingerprint: computePageFingerprint(["crates/demo/src/lib.rs"], fixture.readSource),
      non_claims: ["none"],
    },
    frontmatterError: null,
    body: "hand-edited body",
    raw: "hand-edited body",
  };
  fixture.pages.set("handbook/en/reference/demo.md", generatedPage);
  fixture.pages.set("handbook/zh-CN/reference/demo.md", { ...generatedPage, frontmatter: { ...generatedPage.frontmatter, locale: "zh-CN" } });
  fixture.trackedPaths.push("handbook/en/reference/demo.md", "handbook/zh-CN/reference/demo.md");
  fixture.handbookFiles.push("handbook/en/reference/demo.md", "handbook/zh-CN/reference/demo.md");
  fixture.generatedOutputs = new Map([
    ["handbook/en/reference/demo.md", "generator body"],
    ["handbook/zh-CN/reference/demo.md", "generator body"],
  ]);
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(diagnostics.some((d) => d.rule === "HB010" && /generator output/.test(d.message)));
});

test("secret-shaped content fails HB012", () => {
  const fixture = greenFixture();
  fixture.pages.get("handbook/en/user/demo.md").body = "example key sk-abcdefghijklmnopqrstuv";
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(diagnostics.some((d) => d.rule === "HB012" && /secret-shaped/.test(d.message)));
});

test("copied dynamic Gate status fails HB012", () => {
  const fixture = greenFixture();
  fixture.pages.get("handbook/en/user/demo.md").body = "| B08 Memory + Skill Gate | **pass** |";
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(diagnostics.some((d) => d.rule === "HB012" && /dynamic current-status/.test(d.message)));
});

test("unmanifested handbook markdown fails HB004", () => {
  const fixture = greenFixture();
  fixture.handbookFiles.push("handbook/en/user/orphan.md");
  fixture.pages.set("handbook/en/user/orphan.md", { frontmatter: null, frontmatterError: null, body: "", raw: "" });
  fixture.trackedPaths.push("handbook/en/user/orphan.md");
  const diagnostics = runHandbookChecks(fixture);
  assert.ok(diagnostics.some((d) => d.rule === "HB004" && d.file === "handbook/en/user/orphan.md"));
});

test("missing stable symbol fails HB007", () => {
  const fixture = greenFixture();
  const fm = fixture.pages.get("handbook/en/user/demo.md").frontmatter;
  fm.sources = [{ path: "crates/demo/src/lib.rs", symbols: ["renamed_function"] }];
  assert.ok(rulesOf(runHandbookChecks(fixture)).includes("HB007"));
});

test("glob compiler handles ** and * boundaries", () => {
  assert.ok(compileGlob("crates/**").test("crates/a/b/c.rs"));
  assert.ok(!compileGlob("crates/*.rs").test("crates/a/b.rs"));
  assert.ok(compileGlob("apps/kernel-server/src/personal/*doctor*.rs").test("apps/kernel-server/src/personal/six_resource_doctor.rs"));
});

test("frontmatter splitter tolerates missing block", () => {
  assert.equal(splitFrontmatter("no frontmatter").yamlText, null);
  const { yamlText, body } = splitFrontmatter("---\na: 1\n---\nrest");
  assert.match(yamlText, /a: 1/);
  assert.equal(body, "rest");
});
