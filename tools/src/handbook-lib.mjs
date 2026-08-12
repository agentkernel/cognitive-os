/**
 * Handbook checker library (pure logic; no process.exit, no direct git calls).
 *
 * The handbook (`handbook/`) is an informative derived documentation layer.
 * These checks keep it machine-consistent with the tracked source tree without
 * creating any task, Gate, contract, or release semantics. Every rule returns
 * structured diagnostics: { rule, file, message }.
 *
 * Inputs are injected (file map, tracked path list, generator output) so the
 * focused negative fixtures in tools/test can exercise every failure mode
 * without a real Git repository.
 */

import { createHash } from "node:crypto";
import path from "node:path";

export const HANDBOOK_SOURCE_FINGERPRINT_DOMAIN = "cognitiveos-handbook-source/0.1\n";
export const HANDBOOK_SOURCE_SET_DOMAIN = "cognitiveos-handbook-source-set/0.1\n";

/** Normalize CRLF to LF so fingerprints are checkout-independent. */
export function normalizeEol(text) {
  return text.replaceAll("\r\n", "\n");
}

/** Compute the frontmatter fingerprint over sorted source paths + contents. */
export function computePageFingerprint(sourcePaths, readFile) {
  const hash = createHash("sha256");
  hash.update(HANDBOOK_SOURCE_FINGERPRINT_DOMAIN);
  for (const sourcePath of [...sourcePaths].sort()) {
    hash.update(sourcePath);
    hash.update("\0");
    hash.update(normalizeEol(readFile(sourcePath)));
    hash.update("\n");
  }
  return `sha256:${hash.digest("hex")}`;
}

/** Compute the source-set digest over (path, blobSha) pairs. */
export function computeSourceSetDigest(entries) {
  const hash = createHash("sha256");
  hash.update(HANDBOOK_SOURCE_SET_DOMAIN);
  for (const { path: p, blob } of [...entries].sort((a, b) => (a.path < b.path ? -1 : 1))) {
    hash.update(`${p}\0${blob}\n`);
  }
  return `sha256:${hash.digest("hex")}`;
}

/** Split a Markdown file into { frontmatter (parsed by caller), body, raw }. */
export function splitFrontmatter(raw) {
  // Tolerate a UTF-8 BOM (easily introduced by Windows editors/shells).
  const normalized = normalizeEol(raw.replace(/^\uFEFF/, ""));
  if (!normalized.startsWith("---\n")) {
    return { yamlText: null, body: normalized };
  }
  const end = normalized.indexOf("\n---\n", 4);
  if (end === -1) {
    return { yamlText: null, body: normalized };
  }
  return { yamlText: normalized.slice(4, end + 1), body: normalized.slice(end + 5) };
}

/** Compile a coverage/source-map glob into a regex. Supports **, * and literal text. */
export function compileGlob(glob) {
  let pattern = "";
  for (let i = 0; i < glob.length; i += 1) {
    const ch = glob[i];
    if (ch === "*") {
      if (glob[i + 1] === "*") {
        // `**` crosses path separators; a following `/` is folded in so that
        // `a/**` also matches `a` children at any depth.
        pattern += "(?:.*)";
        i += 1;
        if (glob[i + 1] === "/") i += 1;
      } else {
        pattern += "[^/]*";
      }
    } else if ("\\^$.|?+()[]{}".includes(ch)) {
      pattern += `\\${ch}`;
    } else {
      pattern += ch;
    }
  }
  return new RegExp(`^${pattern}$`);
}

const SECRET_SHAPED_PATTERNS = [
  { id: "provider-key", regex: /sk-[A-Za-z0-9]{16,}/ },
  { id: "aws-key", regex: /AKIA[0-9A-Z]{16}/ },
  { id: "private-key-block", regex: /-----BEGIN [A-Z ]*PRIVATE KEY-----/ },
  { id: "vault-material", regex: /ssv1:[A-Za-z0-9+/=]{16,}/ },
  { id: "bearer-literal", regex: /Authorization:\s*Bearer\s+[A-Za-z0-9._-]{20,}/i },
];

// Dynamic current-status shapes owned by docs/plan/PROGRESS.md. The handbook
// links to the snapshot; it never copies these rows or asserts live Gate state.
const DYNAMIC_STATUS_PATTERNS = [
  { id: "lease-row", regex: /\|\s*Active task lease\s*\|/ },
  { id: "layer-table", regex: /Layer\s*[123]\s*—/ },
  { id: "remaining-counter", regex: /Remaining\s*(=|:)\s*\d+/ },
  { id: "gate-disposition", regex: /\b(B0[1-9]|B1[0-2]|GMVP-LINUX)\b[^\n]{0,40}\*\*(pass|fail|running|not-run)\*\*/i },
  { id: "slice-status-row", regex: /\|\s*`P\d+-T\d+\/D\d+`\s*\|\s*`?(done|in-progress|blocked|ready|cancelled)`?\s*\|/ },
];

/**
 * Run every handbook check. `inputs`:
 * - manifest: parsed manifest.json
 * - frontmatterSchemaValidate: compiled ajv validator or null
 * - pages: Map<repoPath, { frontmatter: object|null, frontmatterError: string|null, body: string }>
 * - trackedPaths: string[] (git ls-files of the whole repo)
 * - coverage: parsed source-coverage.json
 * - sourceMap: parsed source-map.json
 * - readSource: (repoPath) => string  (throws when unreadable)
 * - generatedOutputs: Map<repoPath, string> | null (expected bytes for generated pages)
 * - handbookFiles: string[] (all tracked files under handbook/)
 */
export function runHandbookChecks(inputs) {
  const diagnostics = [];
  const fail = (rule, file, message) => diagnostics.push({ rule, file, message });
  const {
    manifest,
    frontmatterSchemaValidate,
    pages,
    trackedPaths,
    coverage,
    sourceMap,
    readSource,
    generatedOutputs,
    handbookFiles,
  } = inputs;

  const tracked = new Set(trackedPaths);
  const localeRoots = manifest.locale_roots ?? {};
  const locales = manifest.locales ?? [];

  // ---- HB001: manifest integrity -------------------------------------------------
  const docsById = new Map();
  for (const doc of manifest.documents ?? []) {
    if (docsById.has(doc.doc_id)) {
      fail("HB001", "handbook/_meta/manifest.json", `duplicate doc_id: ${doc.doc_id}`);
    }
    docsById.set(doc.doc_id, doc);
  }
  if (!manifest.root_entry || !tracked.has(manifest.root_entry)) {
    fail("HB001", "handbook/_meta/manifest.json", `root_entry missing or untracked: ${manifest.root_entry}`);
  }

  /** Resolve the expected on-disk paths of a manifest document. */
  const docPaths = (doc) =>
    doc.locale_neutral
      ? [{ locale: null, p: `handbook/${doc.rel_path}` }]
      : locales.map((locale) => ({ locale, p: `${localeRoots[locale]}/${doc.rel_path}` }));

  // ---- HB004: manifest <-> filesystem --------------------------------------------
  const manifestedPaths = new Set([manifest.root_entry]);
  for (const doc of docsById.values()) {
    for (const { p } of docPaths(doc)) {
      manifestedPaths.add(p);
      if (!pages.has(p)) {
        fail("HB004", p, `manifest document ${doc.doc_id} has no file at this path`);
      }
    }
  }
  for (const filePath of handbookFiles) {
    if (!filePath.endsWith(".md")) continue;
    if (!manifestedPaths.has(filePath)) {
      fail("HB004", filePath, "markdown page is not registered in handbook/_meta/manifest.json");
    }
  }

  // ---- HB002/HB003/HB011: frontmatter, pairing, status ---------------------------
  const byDocLocale = new Map();
  for (const [filePath, page] of pages) {
    if (filePath === manifest.root_entry) continue;
    if (page.frontmatterError) {
      fail("HB002", filePath, `frontmatter does not parse: ${page.frontmatterError}`);
      continue;
    }
    if (!page.frontmatter) {
      fail("HB002", filePath, "missing YAML frontmatter block");
      continue;
    }
    if (frontmatterSchemaValidate && !frontmatterSchemaValidate(page.frontmatter)) {
      for (const err of frontmatterSchemaValidate.errors ?? []) {
        fail("HB002", filePath, `frontmatter schema violation: ${err.instancePath || "/"} ${err.message}`);
      }
    }
    const fm = page.frontmatter;
    byDocLocale.set(`${fm.doc_id}\0${fm.locale ?? "neutral"}`, { filePath, fm });
    const doc = docsById.get(fm.doc_id);
    if (!doc) {
      fail("HB003", filePath, `doc_id ${fm.doc_id} is not in the manifest`);
      continue;
    }
    if (doc.kind !== fm.kind || doc.generated !== fm.generated) {
      fail("HB011", filePath, `kind/generated disagree with manifest for ${fm.doc_id}`);
    }
  }
  for (const doc of docsById.values()) {
    if (doc.locale_neutral) continue;
    const seen = locales
      .map((locale) => byDocLocale.get(`${doc.doc_id}\0${locale}`))
      .filter(Boolean);
    if (seen.length !== locales.length) {
      fail("HB003", `handbook (${doc.doc_id})`, `document must exist in every locale (${locales.join(", ")}); found ${seen.length}`);
      continue;
    }
    const [a, b] = seen;
    if (b && (a.fm.status !== b.fm.status || JSON.stringify(a.fm.sources ?? []) !== JSON.stringify(b.fm.sources ?? []))) {
      fail("HB003", b.filePath, `locale pair for ${doc.doc_id} disagrees on status or sources`);
    }
  }

  // ---- HB005/HB012: body scans ----------------------------------------------------
  const linkPattern = /\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
  for (const [filePath, page] of pages) {
    const body = page.body ?? "";
    for (const match of body.matchAll(linkPattern)) {
      const raw = match[1];
      if (/^(https?|mailto|urn):/.test(raw) || raw.startsWith("#")) continue;
      if (/^[A-Za-z]:[\\/]/.test(raw) || raw.startsWith("file:")) {
        fail("HB005", filePath, `absolute or file: link is not portable: ${raw}`);
        continue;
      }
      const target = decodeURI(raw.split("#")[0]);
      if (target.length === 0) continue;
      const resolved = path.posix
        .normalize(path.posix.join(path.posix.dirname(filePath), target))
        .replaceAll("\\", "/");
      if (resolved.startsWith("History/") || resolved.includes("/History/")) {
        fail("HB005", filePath, `link into frozen History/: ${raw}`);
      } else if (!tracked.has(resolved)) {
        fail("HB005", filePath, `broken relative link: ${raw} -> ${resolved}`);
      }
    }
    for (const { id, regex } of SECRET_SHAPED_PATTERNS) {
      if (regex.test(body)) {
        fail("HB012", filePath, `secret-shaped content (${id}) must never enter the handbook`);
      }
    }
    for (const { id, regex } of DYNAMIC_STATUS_PATTERNS) {
      if (regex.test(body)) {
        fail("HB012", filePath, `dynamic current-status content (${id}) is owned by docs/plan/PROGRESS.md and must be linked, not copied`);
      }
    }
  }

  // ---- HB006/HB007/HB008: sources, symbols, fingerprints --------------------------
  for (const [filePath, page] of pages) {
    const fm = page.frontmatter;
    if (!fm || page.frontmatterError) continue;
    const sourceEntries = fm.sources ?? [];
    for (const listName of ["contracts", "tests"]) {
      for (const ref of fm[listName] ?? []) {
        if (!tracked.has(ref)) {
          fail("HB006", filePath, `${listName} path does not exist or is untracked: ${ref}`);
        }
      }
    }
    let sourcesOk = true;
    for (const source of sourceEntries) {
      if (!tracked.has(source.path)) {
        fail("HB006", filePath, `source path does not exist or is untracked: ${source.path}`);
        sourcesOk = false;
        continue;
      }
      if (source.symbols?.length) {
        let text;
        try {
          text = readSource(source.path);
        } catch (err) {
          fail("HB006", filePath, `source unreadable: ${source.path} (${err.message})`);
          sourcesOk = false;
          continue;
        }
        for (const symbol of source.symbols) {
          if (!text.includes(symbol)) {
            fail("HB007", filePath, `stable symbol not found in ${source.path}: ${symbol}`);
          }
        }
      }
    }
    if (fm.fingerprint && sourcesOk && sourceEntries.length > 0) {
      let computed;
      try {
        computed = computePageFingerprint(sourceEntries.map((s) => s.path), readSource);
      } catch (err) {
        fail("HB008", filePath, `fingerprint recomputation failed: ${err.message}`);
        continue;
      }
      if (computed !== fm.fingerprint) {
        fail(
          "HB008",
          filePath,
          `source fingerprint drift: mapped sources changed since this page was reviewed (recorded ${fm.fingerprint.slice(0, 18)}…, current ${computed.slice(0, 18)}…). Update the page and its fingerprint in the same PR.`,
        );
      }
    }
  }

  // ---- HB009: total coverage of the tracked tree ----------------------------------
  const compiledRules = (coverage.rules ?? []).map((rule) => ({ rule, regex: compileGlob(rule.glob) }));
  for (const trackedPath of trackedPaths) {
    const matched = compiledRules.find(({ regex }) => regex.test(trackedPath));
    if (!matched) {
      fail("HB009", trackedPath, "tracked file matches no rule in handbook/_meta/source-coverage.json; classify it (first-party-source with owning docs, or an excluded category with a reason)");
      continue;
    }
    const { rule } = matched;
    if (rule.category === "first-party-source") {
      for (const docId of rule.docs ?? []) {
        if (!docsById.has(docId)) {
          fail("HB009", "handbook/_meta/source-coverage.json", `rule ${rule.glob} names unknown doc ${docId}`);
        }
      }
      if (!rule.docs?.length) {
        fail("HB009", "handbook/_meta/source-coverage.json", `first-party rule ${rule.glob} must own at least one doc`);
      }
    } else if (rule.category?.startsWith("excluded-") && !rule.reason) {
      fail("HB009", "handbook/_meta/source-coverage.json", `excluded rule ${rule.glob} must record a reason`);
    }
  }

  // ---- HB015: source-map rules stay live ------------------------------------------
  for (const rule of sourceMap.rules ?? []) {
    for (const glob of rule.sources ?? []) {
      const regex = compileGlob(glob);
      if (!trackedPaths.some((p) => regex.test(p))) {
        fail("HB015", "handbook/_meta/source-map.json", `rule ${rule.id}: glob matches no tracked file: ${glob}`);
      }
    }
    for (const docId of rule.docs ?? []) {
      if (!docsById.has(docId)) {
        fail("HB015", "handbook/_meta/source-map.json", `rule ${rule.id}: unknown doc ${docId}`);
      }
    }
  }

  // ---- HB010: generated pages match generator output ------------------------------
  if (generatedOutputs) {
    for (const doc of docsById.values()) {
      if (!doc.generated) continue;
      for (const { p } of docPaths(doc)) {
        const expected = generatedOutputs.get(p);
        const actual = pages.get(p);
        if (expected === undefined) {
          fail("HB010", p, `generator produced no output for generated page ${doc.doc_id}`);
          continue;
        }
        const actualRaw = actual ? rebuildRaw(actual) : null;
        if (actualRaw === null) continue; // HB004 already failed
        if (normalizeEol(expected) !== actualRaw) {
          fail("HB010", p, "generated page differs from generator output; run `node tools/src/generate-handbook.mjs` instead of hand-editing");
        }
      }
    }
  }

  return diagnostics;
}

/** Reassemble the normalized raw text of a parsed page for byte comparison. */
function rebuildRaw(page) {
  if (page.raw !== undefined) return normalizeEol(page.raw);
  return null;
}
