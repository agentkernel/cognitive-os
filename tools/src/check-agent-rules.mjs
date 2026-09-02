#!/usr/bin/env node
/**
 * Agent-rule reference check.
 *
 * Verifies that the agent entry documents — `AGENTS.md`, `.cursor/rules/*.mdc`
 * and `.cursor/commands/*.md` — are well-formed and only reference things that
 * exist in the repository:
 *
 *  - every `.mdc` rule has a YAML frontmatter with a non-empty `description`,
 *    a boolean `alwaysApply`, and (when present) string/array `globs`;
 *  - every rule file is listed in the `00-*.mdc` rule index;
 *  - every `.cursor/commands/*.md` has frontmatter whose `name` equals the file
 *    stem, and every skill in its `uses:` list exists under `.cursor/skills/`;
 *  - `/pm-*` command references in commands point at existing command files;
 *  - skill names in markdown tables whose header contains "Skill" (rules 30/40)
 *    exist under `.cursor/skills/`;
 *  - relative markdown links resolve;
 *  - repo-relative paths quoted in backticks exist (globs, placeholders, URLs,
 *    lease ids, ignored roots such as `artifacts/` are skipped).
 *
 * Local-only editor assets (`LOCAL_ONLY_PREFIXES`: imported skills, slash
 * commands, the skill-routing rules 30/40 and `.cursor/mcp.json`) are kept out
 * of Git by owner decision. When such a path is present it is checked as
 * strictly as anything else; when it is absent (a clean CI checkout) the
 * reference is reported as a warning, never as a failure. Every other missing
 * path remains a failure.
 *
 * It creates no task, Gate, contract, or release semantics. It only prevents the
 * rules from drifting away from the tree they describe.
 *
 * Usage: node tools/src/check-agent-rules.mjs [--root <repo-root>]
 */

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const DEFAULT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

const RULES_DIR = ".cursor/rules";
const COMMANDS_DIR = ".cursor/commands";
const SKILLS_DIR = ".cursor/skills";

/**
 * Editor assets that exist only in the owner's local workspace (untracked by
 * decision). Missing → warning; present → strict check. Keep this list in sync
 * with the "local rule, not tracked in Git" rows of the 00-* rule index.
 */
export const LOCAL_ONLY_PREFIXES = [
  `${SKILLS_DIR}/`,
  `${COMMANDS_DIR}/`,
  `${RULES_DIR}/30-`,
  `${RULES_DIR}/40-`,
  ".cursor/mcp.json",
];

export function isLocalOnlyPath(rel) {
  return localOnlyPrefixFor(rel) !== undefined;
}

function localOnlyPrefixFor(rel) {
  const normalized = `${rel.replace(/\\/g, "/").replace(/\/+$/, "")}/`;
  return LOCAL_ONLY_PREFIXES.find((prefix) => normalized.startsWith(prefix));
}

/**
 * The asset root whose absence downgrades a missing reference to a warning:
 * the directory for `…/` prefixes, the referenced file itself for file prefixes.
 * When that root is present, everything beneath it is checked strictly.
 */
function localOnlyRootFor(rel) {
  const prefix = localOnlyPrefixFor(rel);
  if (prefix === undefined) return undefined;
  return prefix.endsWith("/") ? prefix.slice(0, -1) : rel.replace(/\\/g, "/").replace(/\/+$/, "");
}

/** Roots that are intentionally outside Git or outside this checker's scope. */
const SKIP_PREFIXES = [
  "lease/",
  "artifacts/",
  "History/",
  "personal-blog/",
  "target/",
  "node_modules/",
  "dist/",
  "origin/",
  "app/",
  "data/",
];

/** Characters that mark a token as a glob, placeholder, URL, credential or shell fragment. */
const NON_PATH_CHARS = /[*<>{}$()@\\"'=:|,;]/;

function toPosix(p) {
  return p.split(path.sep).join("/");
}

function listFiles(root, relDir, ext) {
  const abs = path.join(root, ...relDir.split("/"));
  if (!statSync(abs, { throwIfNoEntry: false })?.isDirectory()) {
    return [];
  }
  return readdirSync(abs)
    .filter((name) => name.endsWith(ext))
    .sort()
    .map((name) => `${relDir}/${name}`);
}

function splitFrontmatter(text) {
  const normalized = text.replace(/\r\n/g, "\n");
  if (!normalized.startsWith("---\n")) {
    return { frontmatter: null, body: normalized, error: "missing YAML frontmatter block" };
  }
  const end = normalized.indexOf("\n---", 4);
  if (end === -1) {
    return { frontmatter: null, body: normalized, error: "unterminated YAML frontmatter block" };
  }
  const raw = normalized.slice(4, end);
  try {
    const parsed = YAML.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return { frontmatter: null, body: normalized, error: "frontmatter is not a YAML mapping" };
    }
    return { frontmatter: parsed, body: normalized.slice(end + 4), error: null };
  } catch (err) {
    return { frontmatter: null, body: normalized, error: `frontmatter does not parse: ${err.message}` };
  }
}

/** Remove fenced code blocks so shell examples are not mistaken for prose references. */
function stripFences(body) {
  return body.replace(/```[\s\S]*?```/g, "");
}

function markdownLinks(body) {
  const out = [];
  for (const match of body.matchAll(/\[[^\]]*\]\(([^)\s]+)\)/g)) {
    out.push(match[1]);
  }
  return out;
}

/** Inline code spans, excluding spans that are the visible text of a markdown link. */
function inlineCodeSpans(body) {
  const withoutLinkText = body.replace(/\[`[^`]+`\]\([^)]*\)/g, "");
  const out = [];
  for (const match of withoutLinkText.matchAll(/`([^`\n]+)`/g)) {
    out.push(match[1]);
  }
  return out;
}

/**
 * A slash-separated token is treated as a repository path only when its first
 * segment is an existing top-level entry (`docs/…`, `.cursor/…`) or when it
 * carries a file extension (`x/y.md`). Prose such as `build/test/clippy` or
 * `pass/fail/not-run` is therefore ignored.
 */
function looksLikeRepoPath(piece, topLevelEntries) {
  if (piece.length < 2) return false;
  if (NON_PATH_CHARS.test(piece)) return false;
  if (/^[-/#~]/.test(piece)) return false;
  if (/^https?:/i.test(piece) || piece.includes("://")) return false;
  if (SKIP_PREFIXES.some((prefix) => piece.startsWith(prefix))) return false;
  if (piece.includes("/")) {
    const firstSegment = piece.split("/")[0];
    return topLevelEntries.has(firstSegment) || /\.[a-z0-9]{1,5}$/i.test(piece);
  }
  return /^\.(git|cursor|github|vscode)/.test(piece);
}

/** Candidate repo-relative path references inside inline code spans. */
function pathReferences(body, topLevelEntries) {
  const refs = new Set();
  for (const span of inlineCodeSpans(stripFences(body))) {
    for (const rawPiece of span.split(/\s+/)) {
      const piece = rawPiece.replace(/^[`'"(]+|[`'".,;:)]+$/g, "");
      if (looksLikeRepoPath(piece, topLevelEntries)) {
        refs.add(piece.replace(/\/+$/, ""));
      } else if (/^\d{2}-[a-z0-9-]+\.mdc$/.test(piece)) {
        refs.add(`${RULES_DIR}/${piece}`);
      }
    }
  }
  return [...refs];
}

/** Skill names in markdown tables whose header row mentions "Skill". */
function tableSkillReferences(body) {
  const lines = stripFences(body).split("\n");
  const refs = [];
  for (let i = 0; i < lines.length; i += 1) {
    if (!/^\s*\|.*\bSkill\b.*\|\s*$/i.test(lines[i])) continue;
    let j = i + 1;
    if (!/^\s*\|[\s:|-]+\|\s*$/.test(lines[j] ?? "")) continue;
    for (j += 1; j < lines.length && /^\s*\|/.test(lines[j]); j += 1) {
      for (const match of lines[j].matchAll(/`([a-z][a-z0-9]*(?:-[a-z0-9]+)+)`/g)) {
        refs.push({ name: match[1], line: j + 1 });
      }
    }
    i = j;
  }
  return refs;
}

function commandReferences(body) {
  const refs = [];
  for (const span of inlineCodeSpans(body)) {
    const match = /^\/([a-z][a-z0-9-]*)$/.exec(span.trim());
    if (match) refs.push(match[1]);
  }
  return refs;
}

export function checkAgentRules(root = DEFAULT_ROOT) {
  const failures = [];
  const warnings = [];
  const fail = (file, message) => failures.push({ file, message });
  const warn = (file, message) => warnings.push({ file, message });
  const exists = (rel) => existsSync(path.join(root, ...rel.split("/")));
  /** A local-only asset whose root is absent from this checkout (clean CI). */
  const localOnlyAbsent = (rel) => {
    const assetRoot = localOnlyRootFor(rel);
    return assetRoot !== undefined && !exists(assetRoot);
  };
  /** Missing references warn only for absent local-only assets; everything else fails. */
  const missing = (file, rel, message) =>
    localOnlyAbsent(rel)
      ? warn(file, `${message} (local-only asset absent)`)
      : fail(file, message);
  const skillsDirPresent = exists(SKILLS_DIR);
  const commandsDirPresent = exists(COMMANDS_DIR);
  const skillExists = (name) => exists(`${SKILLS_DIR}/${name}/SKILL.md`);
  const commandExists = (name) => exists(`${COMMANDS_DIR}/${name}.md`);
  const read = (rel) => readFileSync(path.join(root, ...rel.split("/")), "utf-8");
  const topLevelEntries = new Set(
    statSync(root, { throwIfNoEntry: false })?.isDirectory() ? readdirSync(root) : [],
  );

  const checkSkillRef = (file, name, describe) => {
    if (skillExists(name)) return;
    if (!skillsDirPresent) {
      warn(file, `${describe}: skill \`${name}\` not verified — ${SKILLS_DIR}/ is not present in this checkout (local-only)`);
    } else {
      fail(file, `${describe}: skill \`${name}\` is not installed under ${SKILLS_DIR}/`);
    }
  };

  const checkCommandRef = (file, name) => {
    if (commandExists(name)) return;
    if (!commandsDirPresent) {
      warn(file, `command /${name} not verified — ${COMMANDS_DIR}/ is not present in this checkout (local-only)`);
    } else {
      fail(file, `command /${name} has no ${COMMANDS_DIR}/${name}.md`);
    }
  };

  const checkLinks = (rel, body) => {
    for (const target of markdownLinks(stripFences(body))) {
      if (/^(https?:|mailto:|#)/i.test(target)) continue;
      const withoutAnchor = target.split("#")[0];
      if (!withoutAnchor) continue;
      const resolved = path.resolve(root, path.dirname(rel), withoutAnchor);
      if (!existsSync(resolved)) {
        const relTarget = toPosix(path.relative(root, resolved));
        missing(rel, relTarget, `broken relative link: ${target}`);
      }
    }
  };

  const checkPathRefs = (rel, body) => {
    let count = 0;
    for (const ref of pathReferences(body, topLevelEntries)) {
      count += 1;
      if (!exists(ref)) {
        missing(rel, ref, `referenced path does not exist: ${ref}`);
      }
    }
    return count;
  };

  let pathRefCount = 0;

  // ---- AGENTS.md --------------------------------------------------------------
  if (!exists("AGENTS.md")) {
    fail("AGENTS.md", "agent entry document is missing");
  } else {
    const text = read("AGENTS.md");
    checkLinks("AGENTS.md", text);
    pathRefCount += checkPathRefs("AGENTS.md", text);
  }

  // ---- .cursor/rules/*.mdc ---------------------------------------------------
  const ruleFiles = listFiles(root, RULES_DIR, ".mdc");
  if (ruleFiles.length === 0) {
    fail(RULES_DIR, "no .mdc rule files found");
  }
  const indexRule = ruleFiles.find((rel) => /\/00-[^/]+\.mdc$/.test(rel));
  const indexText = indexRule ? read(indexRule) : "";

  for (const rel of ruleFiles) {
    const { frontmatter, body, error } = splitFrontmatter(read(rel));
    if (error) {
      fail(rel, error);
    } else {
      if (typeof frontmatter.description !== "string" || frontmatter.description.trim() === "") {
        fail(rel, "frontmatter.description must be a non-empty string");
      }
      if (typeof frontmatter.alwaysApply !== "boolean") {
        fail(rel, "frontmatter.alwaysApply must be a boolean");
      }
      if (frontmatter.globs !== undefined) {
        const globs = Array.isArray(frontmatter.globs) ? frontmatter.globs : [frontmatter.globs];
        if (globs.length === 0 || globs.some((g) => typeof g !== "string" || g.trim() === "")) {
          fail(rel, "frontmatter.globs must be a non-empty string or an array of non-empty strings");
        }
      }
    }
    if (indexRule && rel !== indexRule && !indexText.includes(path.posix.basename(rel))) {
      fail(indexRule, `rule index does not list ${path.posix.basename(rel)}`);
    }
    checkLinks(rel, body);
    pathRefCount += checkPathRefs(rel, body);
    for (const { name, line } of tableSkillReferences(body)) {
      if (!commandExists(name)) {
        checkSkillRef(rel, name, `line ${line}`);
      }
    }
    for (const name of commandReferences(body)) {
      if (name.startsWith("pm-")) {
        checkCommandRef(rel, name);
      }
    }
  }

  // ---- .cursor/commands/*.md -------------------------------------------------
  const commandFiles = listFiles(root, COMMANDS_DIR, ".md");
  for (const rel of commandFiles) {
    const stem = path.posix.basename(rel, ".md");
    const { frontmatter, body, error } = splitFrontmatter(read(rel));
    if (error) {
      fail(rel, error);
    } else {
      if (frontmatter.name !== stem) {
        fail(rel, `frontmatter.name (${frontmatter.name}) must equal the file stem (${stem})`);
      }
      if (typeof frontmatter.description !== "string" || frontmatter.description.trim() === "") {
        fail(rel, "frontmatter.description must be a non-empty string");
      }
      const uses = frontmatter.uses ?? [];
      if (!Array.isArray(uses)) {
        fail(rel, "frontmatter.uses must be a list of skill names");
      } else {
        for (const skill of uses) {
          if (typeof skill !== "string") {
            fail(rel, `uses: entries must be skill names, got ${JSON.stringify(skill)}`);
          } else {
            checkSkillRef(rel, skill, "uses");
          }
        }
      }
    }
    checkLinks(rel, body);
    pathRefCount += checkPathRefs(rel, body);
    for (const name of commandReferences(body)) {
      if (name.startsWith("pm-") || name === stem) {
        checkCommandRef(rel, name);
      }
    }
  }

  return {
    failures,
    warnings,
    checked: {
      rules: ruleFiles.length,
      commands: commandFiles.length,
      pathReferences: pathRefCount,
    },
  };
}

function main() {
  const argv = process.argv.slice(2);
  const rootIndex = argv.indexOf("--root");
  const root = rootIndex !== -1 && argv[rootIndex + 1] ? path.resolve(argv[rootIndex + 1]) : DEFAULT_ROOT;
  const { failures, warnings, checked } = checkAgentRules(root);
  for (const { file, message } of warnings) {
    console.error(`warning: ${toPosix(file)}: ${message}`);
  }
  if (failures.length > 0) {
    for (const { file, message } of failures) {
      console.error(`${toPosix(file)}: ${message}`);
    }
    console.error(`check-agent-rules: FAIL (${failures.length} problem(s), ${warnings.length} warning(s))`);
    process.exit(1);
  }
  console.log(
    `check-agent-rules: OK (${checked.rules} rules, ${checked.commands} commands, ${checked.pathReferences} path references, ${warnings.length} local-only warning(s))`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
