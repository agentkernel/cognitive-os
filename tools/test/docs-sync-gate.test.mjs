import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { decideDocsSync, routeChangedPaths } from "../src/docs-sync-gate.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

test("the live source map routes installer.rs to the Pi package pages and fails closed without a handbook update (P0-T09)", () => {
  const liveSourceMap = JSON.parse(
    readFileSync(path.join(REPO_ROOT, "personal", "handbook", "_meta", "source-map.json"), "utf8"),
  );
  const route = routeChangedPaths(["personal/crates/cognitive-runtime/src/installer.rs"], liveSourceMap);
  const pin = route.impacted.find((rule) => rule.id === "pi-official-package-pin");
  assert.ok(pin, "installer.rs must hit the pi-official-package-pin rule");
  assert.deepEqual(pin.docs, ["ref.compatibility", "dev.agent-pi-lifecycle"]);
  const liveRule = liveSourceMap.rules.find((rule) => rule.id === "pi-official-package-pin");
  assert.deepEqual(liveRule.symbols, ["OFFICIAL_PI_PACKAGE", "OFFICIAL_PI_VERSION"]);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "fail");
});

const SOURCE_MAP = {
  rules: [
    { id: "secrets-provider", sources: ["personal/crates/cognitive-secret/**"], docs: ["user.provider-and-secrets", "ref.config-files"] },
    { id: "cli-product", sources: ["personal/apps/admin-cli/src/personal_cli/**"], docs: ["ref.cli-cognitive"] },
  ],
};

test("unrelated changes skip the gate without running checks", () => {
  const route = routeChangedPaths(["docs/plan/PROGRESS.md", "README.md"], SOURCE_MAP);
  assert.equal(route.impacted.length, 0);
  assert.equal(route.docsTouched, false);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "skip");
});

test("handbook-only changes run the check set", () => {
  const route = routeChangedPaths(["personal/handbook/en/user/cli-basics.md"], SOURCE_MAP);
  assert.equal(route.docsTouched, true);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "check");
});

test("mapped source change without any handbook update fails closed", () => {
  const route = routeChangedPaths(["personal/crates/cognitive-secret/src/store.rs"], SOURCE_MAP);
  assert.equal(route.impacted.length, 1);
  assert.deepEqual(route.impacted[0].docs, ["user.provider-and-secrets", "ref.config-files"]);
  assert.equal(route.docsTouched, false);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "fail");
});

test("an explicit DOCS_IMPACT_NONE reason acknowledges a docs-neutral change", () => {
  const route = routeChangedPaths(["personal/crates/cognitive-secret/src/store.rs"], SOURCE_MAP);
  const decision = decideDocsSync({ ...route, allowNoneReason: "comment-only refactor, no behavior change" });
  assert.equal(decision.verdict, "acknowledged");
  assert.match(decision.reason, /comment-only/);
});

test("a blank or trivial reason does not bypass the gate", () => {
  const route = routeChangedPaths(["personal/crates/cognitive-secret/src/store.rs"], SOURCE_MAP);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: "  " }).verdict, "fail");
  assert.equal(decideDocsSync({ ...route, allowNoneReason: "ok" }).verdict, "fail");
});

test("mapped source plus handbook update in the same set runs checks", () => {
  const route = routeChangedPaths(
    ["personal/apps/admin-cli/src/personal_cli/mod.rs", "personal/handbook/en/reference/cli-cognitive.md"],
    SOURCE_MAP,
  );
  assert.equal(route.impacted.length, 1);
  assert.equal(route.docsTouched, true);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "check");
});

test("rule-20 adapter edits count as documentation-surface changes", () => {
  const route = routeChangedPaths([".cursor/rules/20-cognitiveos-personal-handbook-sync.mdc"], SOURCE_MAP);
  assert.equal(route.docsTouched, true);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "check");
});
