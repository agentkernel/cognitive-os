import assert from "node:assert/strict";
import { test } from "node:test";
import { decideDocsSync, routeChangedPaths } from "../src/docs-sync-gate.mjs";

const SOURCE_MAP = {
  rules: [
    { id: "secrets-provider", sources: ["crates/cognitive-secret/**"], docs: ["user.provider-and-secrets", "ref.config-files"] },
    { id: "cli-product", sources: ["apps/admin-cli/src/personal_cli/**"], docs: ["ref.cli-cognitive"] },
  ],
};

test("unrelated changes skip the gate without running checks", () => {
  const route = routeChangedPaths(["docs/plan/PROGRESS.md", "README.md"], SOURCE_MAP);
  assert.equal(route.impacted.length, 0);
  assert.equal(route.docsTouched, false);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "skip");
});

test("handbook-only changes run the check set", () => {
  const route = routeChangedPaths(["handbook/en/user/cli-basics.md"], SOURCE_MAP);
  assert.equal(route.docsTouched, true);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "check");
});

test("mapped source change without any handbook update fails closed", () => {
  const route = routeChangedPaths(["crates/cognitive-secret/src/store.rs"], SOURCE_MAP);
  assert.equal(route.impacted.length, 1);
  assert.deepEqual(route.impacted[0].docs, ["user.provider-and-secrets", "ref.config-files"]);
  assert.equal(route.docsTouched, false);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: undefined }).verdict, "fail");
});

test("an explicit DOCS_IMPACT_NONE reason acknowledges a docs-neutral change", () => {
  const route = routeChangedPaths(["crates/cognitive-secret/src/store.rs"], SOURCE_MAP);
  const decision = decideDocsSync({ ...route, allowNoneReason: "comment-only refactor, no behavior change" });
  assert.equal(decision.verdict, "acknowledged");
  assert.match(decision.reason, /comment-only/);
});

test("a blank or trivial reason does not bypass the gate", () => {
  const route = routeChangedPaths(["crates/cognitive-secret/src/store.rs"], SOURCE_MAP);
  assert.equal(decideDocsSync({ ...route, allowNoneReason: "  " }).verdict, "fail");
  assert.equal(decideDocsSync({ ...route, allowNoneReason: "ok" }).verdict, "fail");
});

test("mapped source plus handbook update in the same set runs checks", () => {
  const route = routeChangedPaths(
    ["apps/admin-cli/src/personal_cli/mod.rs", "handbook/en/reference/cli-cognitive.md"],
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
