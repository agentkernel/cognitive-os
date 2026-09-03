import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  PATH_EXISTENCE_FILESYSTEM_FALLBACK,
  PATH_EXISTENCE_TRACKED,
  checkAgentRules,
  isLocalOnlyPath,
} from "../src/check-agent-rules.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

function git(root, ...args) {
  return execFileSync("git", ["-C", root, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
}

function write(root, rel, contents) {
  const abs = path.join(root, ...rel.split("/"));
  mkdirSync(path.dirname(abs), { recursive: true });
  writeFileSync(abs, contents, "utf-8");
}

function withFixture(build, run) {
  const root = mkdtempSync(path.join(os.tmpdir(), "agent-rules-"));
  try {
    build(root);
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("agent rules on the current repository tree reference only real paths, skills and commands", () => {
  const { failures, checked } = checkAgentRules(REPO_ROOT);
  assert.deepEqual(failures, []);
  assert.ok(checked.rules >= 4, "expected the repository rule set to be scanned");
  assert.ok(checked.pathReferences > 20, "expected path references to be scanned");
  assert.equal(checked.pathExistence, PATH_EXISTENCE_TRACKED, "the repository run must use git ls-files");
});

test("inside a Git checkout, a rule pointing at an untracked-but-present file fails; the same reference passes only once tracked", () => {
  withFixture(
    (root) => {
      git(root, "init", "-q");
      mkdirSync(path.join(root, "docs"));
      write(root, "docs/tracked.md", "# tracked\n");
      write(root, "docs/untracked.md", "# exists only in this working tree\n");
      write(
        root,
        "AGENTS.md",
        "# Entry\n\nRead `docs/tracked.md` and `docs/untracked.md`; see [draft](docs/untracked.md). Local: `.cursor/skills/`.\n",
      );
      write(root, ".cursor/rules/00-index.mdc", "---\ndescription: index\nalwaysApply: true\n---\n\nnothing\n");
      git(root, "add", "--", "AGENTS.md", "docs/tracked.md", ".cursor/rules/00-index.mdc");
    },
    (root) => {
      const first = checkAgentRules(root);
      assert.equal(first.checked.pathExistence, PATH_EXISTENCE_TRACKED);
      assert.deepEqual(
        first.failures.map((f) => `${f.file}: ${f.message}`).sort(),
        [
          "AGENTS.md: broken relative link: docs/untracked.md (exists locally but is not tracked by Git)",
          "AGENTS.md: referenced path does not exist: docs/untracked.md (exists locally but is not tracked by Git)",
        ],
      );
      // Local-only assets stay a warning class even under tracked-only checking.
      assert.match(
        first.warnings.map((w) => `${w.file}: ${w.message}`).join("\n"),
        /AGENTS\.md: referenced path does not exist: \.cursor\/skills \(local-only asset absent\)/,
      );

      git(root, "add", "--", "docs/untracked.md");
      const second = checkAgentRules(root);
      assert.deepEqual(second.failures, []);
    },
  );
});

test("outside a Git checkout the checker falls back to the filesystem and labels that mode", () => {
  withFixture(
    (root) => {
      mkdirSync(path.join(root, "docs"));
      write(root, "AGENTS.md", "# Entry\n\nRead `docs/plan.md`.\n");
      write(root, "docs/plan.md", "# plan\n");
      write(root, ".cursor/rules/00-index.mdc", "---\ndescription: index\nalwaysApply: true\n---\n\nnothing\n");
    },
    (root) => {
      const { failures, checked } = checkAgentRules(root);
      assert.deepEqual(failures, []);
      assert.equal(checked.pathExistence, PATH_EXISTENCE_FILESYSTEM_FALLBACK);
    },
  );
});

test("local-only prefixes cover exactly the untracked editor assets", () => {
  for (const rel of [
    ".cursor/skills",
    ".cursor/skills/",
    ".cursor/skills/prd-development/SKILL.md",
    ".cursor/commands/pm-write-prd.md",
    ".cursor/rules/30-product-manager-skills.mdc",
    ".cursor/rules/40-personal-20-design-review-skills.mdc",
    ".cursor/mcp.json",
  ]) {
    assert.ok(isLocalOnlyPath(rel), `${rel} should be local-only`);
  }
  for (const rel of [
    ".cursor/rules/00-cognitiveos-personal-project.mdc",
    ".cursor/rules/10-autonomous-personal-development.mdc",
    ".cursor/environment.json",
    "docs/plan/PROGRESS.md",
    "AGENTS.md",
  ]) {
    assert.ok(!isLocalOnlyPath(rel), `${rel} must stay strictly checked`);
  }
});

test("a clean checkout without local-only assets warns but does not fail; real missing paths still fail", () => {
  withFixture(
    (root) => {
      mkdirSync(path.join(root, "docs"));
      write(
        root,
        "AGENTS.md",
        "# Entry\n\nLocal assets: `.cursor/skills/`, `.cursor/commands/`, `.cursor/mcp.json`. Real: `docs/plan.md`, `docs/absent.md`.\n",
      );
      write(root, "docs/plan.md", "# plan\n");
      write(
        root,
        ".cursor/rules/00-index.mdc",
        "---\ndescription: index\nalwaysApply: true\n---\n\n| Rule | Applies |\n|---|---|\n| `10-work.mdc` | always |\n| `30-skills.mdc` local | pm |\n",
      );
      write(
        root,
        ".cursor/rules/10-work.mdc",
        "---\ndescription: work\nalwaysApply: true\n---\n\nSkill routing lives in `30-skills.mdc`; see [rule 30](30-skills.mdc). Use `/pm-write-prd`.\n\n| Job | Skill |\n|---|---|\n| PRD | `prd-development` |\n",
      );
    },
    (root) => {
      const { failures, warnings } = checkAgentRules(root);
      assert.deepEqual(
        failures.map((f) => `${f.file}: ${f.message}`),
        ["AGENTS.md: referenced path does not exist: docs/absent.md"],
      );
      const warningText = warnings.map((w) => `${w.file}: ${w.message}`).join("\n");
      for (const pattern of [
        /AGENTS\.md: referenced path does not exist: \.cursor\/skills \(local-only asset absent\)/,
        /AGENTS\.md: referenced path does not exist: \.cursor\/commands \(local-only asset absent\)/,
        /AGENTS\.md: referenced path does not exist: \.cursor\/mcp\.json \(local-only asset absent\)/,
        /00-index\.mdc: referenced path does not exist: \.cursor\/rules\/30-skills\.mdc \(local-only asset absent\)/,
        /10-work\.mdc: broken relative link: 30-skills\.mdc \(local-only asset absent\)/,
        /10-work\.mdc: command \/pm-write-prd not verified/,
        /10-work\.mdc: line \d+: skill `prd-development` not verified/,
      ]) {
        assert.match(warningText, pattern);
      }
    },
  );
});

test("when local-only assets are present they are checked strictly", () => {
  withFixture(
    (root) => {
      write(root, "AGENTS.md", "# Entry\n\nSee `.cursor/skills/prd-development/SKILL.md` and `.cursor/skills/gone/SKILL.md`.\n");
      write(root, ".cursor/rules/00-index.mdc", "---\ndescription: index\nalwaysApply: true\n---\n\n`30-skills.mdc`\n");
      write(
        root,
        ".cursor/rules/30-skills.mdc",
        "---\ndescription: skills\nalwaysApply: false\n---\n\n| Job | Skill |\n|---|---|\n| ok | `prd-development` |\n| bad | `not-a-skill` |\n\nRun `/pm-missing`.\n",
      );
      write(root, ".cursor/skills/prd-development/SKILL.md", "---\nname: prd-development\n---\n");
      write(root, ".cursor/commands/pm-write-prd.md", "---\nname: pm-write-prd\ndescription: prd\nuses:\n  - prd-development\n  - nope-skill\n---\n\n# /pm-write-prd\n");
    },
    (root) => {
      const { failures, warnings } = checkAgentRules(root);
      assert.deepEqual(warnings, []);
      const messages = failures.map((f) => `${f.file}: ${f.message}`);
      const expect = (pattern) =>
        assert.ok(messages.some((m) => pattern.test(m)), `expected a failure matching ${pattern}\n${messages.join("\n")}`);
      expect(/AGENTS\.md: referenced path does not exist: \.cursor\/skills\/gone\/SKILL\.md$/);
      expect(/30-skills\.mdc: line \d+: skill `not-a-skill` is not installed/);
      expect(/30-skills\.mdc: command \/pm-missing has no/);
      expect(/pm-write-prd\.md: uses: skill `nope-skill` is not installed/);
      assert.equal(failures.length, 4, messages.join("\n"));
    },
  );
});

test("well-formed fixture passes", () => {
  withFixture(
    (root) => {
      mkdirSync(path.join(root, "docs"));
      write(root, "AGENTS.md", "# Entry\n\nRead `docs/plan.md` and [plan](docs/plan.md).\n");
      write(root, "docs/plan.md", "# plan\n");
      write(
        root,
        ".cursor/rules/00-index.mdc",
        "---\ndescription: index\nalwaysApply: true\n---\n\nRules: `10-work.mdc`, `30-skills.mdc`.\n",
      );
      write(
        root,
        ".cursor/rules/10-work.mdc",
        '---\ndescription: work\nglobs:\n  - "docs/**"\nalwaysApply: false\n---\n\nSee [plan](../../docs/plan.md); prose like `build/test/clippy` or `pass/fail` is not a path.\n',
      );
      write(
        root,
        ".cursor/rules/30-skills.mdc",
        "---\ndescription: skills\nalwaysApply: false\n---\n\n| Need | Skill | Command |\n|---|---|---|\n| PRD | `prd-development` | `/pm-write-prd` |\n",
      );
      write(root, ".cursor/skills/prd-development/SKILL.md", "---\nname: prd-development\n---\n");
      write(
        root,
        ".cursor/commands/pm-write-prd.md",
        "---\nname: pm-write-prd\ndescription: prd\nuses:\n  - prd-development\n---\n\n# /pm-write-prd\n",
      );
    },
    (root) => {
      const { failures } = checkAgentRules(root);
      assert.deepEqual(failures, []);
    },
  );
});

test("drifted rules are rejected with one failure per defect", () => {
  withFixture(
    (root) => {
      write(root, "AGENTS.md", "# Entry\n\nRead `docs/missing.md` and [gone](docs/gone.md).\n");
      mkdirSync(path.join(root, "docs"));
      write(root, ".cursor/rules/00-index.mdc", "---\ndescription: index\nalwaysApply: true\n---\n\nRules: `10-work.mdc`.\n");
      write(root, ".cursor/rules/10-work.mdc", "# no frontmatter at all\n");
      write(
        root,
        ".cursor/rules/20-bad-frontmatter.mdc",
        "---\ndescription: ''\nalwaysApply: yes-please\nglobs: []\n---\n\nbody\n",
      );
      write(
        root,
        ".cursor/rules/30-skills.mdc",
        "---\ndescription: skills\nalwaysApply: false\n---\n\n| Job | Skill |\n|---|---|\n| x | `not-a-skill` |\n\nRun `/pm-missing`.\n",
      );
      write(
        root,
        ".cursor/commands/pm-write-prd.md",
        "---\nname: write-prd\ndescription: prd\nuses:\n  - nope-skill\n---\n\n# /pm-write-prd\n\nThen `/pm-plan-roadmap`.\n",
      );
      // The skills pack is present, so missing skills are defects rather than local-only warnings.
      write(root, ".cursor/skills/some-skill/SKILL.md", "---\nname: some-skill\n---\n");
    },
    (root) => {
      const { failures } = checkAgentRules(root);
      const messages = failures.map((f) => `${f.file}: ${f.message}`);
      const expect = (pattern) =>
        assert.ok(messages.some((m) => pattern.test(m)), `expected a failure matching ${pattern}\n${messages.join("\n")}`);

      expect(/^AGENTS\.md: referenced path does not exist: docs\/missing\.md$/);
      expect(/^AGENTS\.md: broken relative link: docs\/gone\.md$/);
      expect(/10-work\.mdc: missing YAML frontmatter block/);
      expect(/20-bad-frontmatter\.mdc: frontmatter\.description must be a non-empty string/);
      expect(/20-bad-frontmatter\.mdc: frontmatter\.alwaysApply must be a boolean/);
      expect(/20-bad-frontmatter\.mdc: frontmatter\.globs must be/);
      expect(/00-index\.mdc: rule index does not list 20-bad-frontmatter\.mdc/);
      expect(/00-index\.mdc: rule index does not list 30-skills\.mdc/);
      expect(/30-skills\.mdc: line \d+: skill `not-a-skill` is not installed/);
      expect(/30-skills\.mdc: command \/pm-missing has no/);
      expect(/pm-write-prd\.md: frontmatter\.name \(write-prd\) must equal the file stem \(pm-write-prd\)/);
      expect(/pm-write-prd\.md: uses: skill `nope-skill` is not installed/);
      expect(/pm-write-prd\.md: command \/pm-plan-roadmap has no/);
    },
  );
});
