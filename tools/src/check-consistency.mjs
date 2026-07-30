/**
 * Static consistency checker (CI gate; docs/standards/docs-sync-contract.md
 * section 5). Checks, in order:
 *
 *  1. every JSON/YAML normative asset parses;
 *  2. every schema compiles under JSON Schema draft 2020-12 with all
 *     relative $refs resolvable;
 *  3. registry <-> schema <-> vector bidirectional orphan freedom;
 *  4. relative markdown links in living docs resolve;
 *  5. traceability matrix and findings ledger are complete and their
 *     referenced paths exist;
 *  6. repository project identity, Personal current snapshot, and active
 *     ownership leases have one consistent source of truth.
 *
 * Exit code 0 = green; 1 = at least one violation, each printed with file
 * and reason. History/ is never scanned (frozen archive).
 */

import { existsSync, statSync } from "node:fs";
import path from "node:path";
import { Ajv2020 } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import {
  REPO_ROOT,
  listMarkdownFiles,
  loadRegistries,
  loadSchemas,
  loadVectors,
  readText,
  readYaml,
  repoPath,
  toRepoRelative,
} from "./lib.mjs";

const failures = [];
const fail = (file, reason) => failures.push({ file, reason });

// ---------- 1 + 2: schemas parse and compile (draft 2020-12, relative $refs)

let schemas = [];
try {
  schemas = loadSchemas();
} catch (err) {
  fail("specs/schemas", `unparseable schema JSON: ${err.message}`);
}

const ajv = new Ajv2020({ strict: false, allErrors: true, validateFormats: true });
addFormats(ajv);
for (const schema of schemas) {
  // $id policy (D-001/D-006 closure): every schema declares a top-level $id
  // exactly equal to its file name, so every relative $ref resolves FROM THE
  // CONTAINING SCHEMA FILE (conformance/README.md "Running") and the $id is
  // the retrieval URI — no stripping compatibility layer.
  if (schema.doc.$id !== schema.name) {
    fail(
      schema.path,
      `schema $id must equal its file name (got ${JSON.stringify(schema.doc.$id)})`,
    );
    continue;
  }
  try {
    ajv.addSchema(schema.doc);
  } catch (err) {
    fail(schema.path, `schema failed to register: ${err.message}`);
  }
}
for (const schema of schemas) {
  try {
    ajv.getSchema(schema.name) ?? ajv.compile(schema.doc);
  } catch (err) {
    fail(schema.path, `schema failed draft 2020-12 compilation/$ref resolution: ${err.message}`);
  }
}

// ---------- registries parse

let registries;
try {
  registries = loadRegistries();
} catch (err) {
  fail("specs/registry", `unparseable registry YAML: ${err.message}`);
}

// ---------- transitions parse and validate against the transition-table schema

const transitionFiles = [
  "agent-execution",
  "effect",
  "loop",
  "task",
  "verification",
].map((d) => repoPath("specs", "transitions", `${d}.transitions.json`));
const transitionValidate = ajv.getSchema("state-transition-table.schema.json");
for (const abs of transitionFiles) {
  const rel = toRepoRelative(abs);
  if (!existsSync(abs)) {
    fail(rel, "registered execution lifecycle domain has no transition table");
    continue;
  }
  try {
    const doc = JSON.parse(readText(abs));
    if (transitionValidate && !transitionValidate(doc)) {
      fail(rel, `transition table does not validate: ${ajv.errorsText(transitionValidate.errors)}`);
    }
  } catch (err) {
    fail(rel, `unparseable transition table: ${err.message}`);
  }
}

// ---------- 3: registry <-> schema <-> vector orphan freedom

let vectors = [];
try {
  vectors = loadVectors();
} catch (err) {
  fail("conformance/vectors", `unparseable vector JSON: ${err.message}`);
}

if (registries) {
  const { requirements, requirementIds, errorCodes } = registries;

  // Registry-side integrity.
  const seenReq = new Set();
  const registeredTestIds = new Set();
  for (const req of requirements.requirements) {
    if (seenReq.has(req.id)) {
      fail("specs/registry/requirements.yaml", `duplicate requirement id ${req.id}`);
    }
    seenReq.add(req.id);
    if (!/^REQ-[A-Z0-9-]+$/.test(req.id)) {
      fail("specs/registry/requirements.yaml", `malformed requirement id ${req.id}`);
    }
    if (!Array.isArray(req.tests) || req.tests.length === 0) {
      fail("specs/registry/requirements.yaml", `${req.id} has no test mapping`);
    }
    for (const testId of req.tests ?? []) {
      registeredTestIds.add(testId);
    }
    if (typeof req.owner_spec === "string") {
      const target = req.owner_spec.split("#")[0];
      if (!existsSync(repoPath(...target.split("/")))) {
        fail("specs/registry/requirements.yaml", `${req.id} owner_spec path missing: ${target}`);
      }
      if (target.startsWith("History/")) {
        fail("specs/registry/requirements.yaml", `${req.id} owner_spec points into frozen History/`);
      }
    } else {
      fail("specs/registry/requirements.yaml", `${req.id} has no owner_spec`);
    }
  }

  // Vector-side integrity.
  const vectorIds = new Set();
  const errorKeyPattern = /code|error/i;
  const errorValuePattern = /^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+$/;
  const collectErrorish = (node, out) => {
    if (Array.isArray(node)) {
      for (const item of node) collectErrorish(item, out);
    } else if (node && typeof node === "object") {
      for (const [key, value] of Object.entries(node)) {
        if (typeof value === "string" && errorKeyPattern.test(key) && errorValuePattern.test(value)) {
          out.push(value);
        }
        collectErrorish(value, out);
      }
    }
  };
  for (const vector of vectors) {
    for (const field of ["id", "layer"]) {
      if (typeof vector.doc[field] !== "string" || vector.doc[field].length === 0) {
        fail(vector.path, `vector missing string field \`${field}\``);
      }
    }
    for (const field of ["profiles", "requirement_ids"]) {
      if (!Array.isArray(vector.doc[field]) || vector.doc[field].length === 0) {
        fail(vector.path, `vector missing non-empty array \`${field}\``);
      }
    }
    if (vector.doc.expected === undefined) {
      fail(vector.path, "vector has no `expected` outcome");
    }
    if (vectorIds.has(vector.doc.id)) {
      fail(vector.path, `duplicate vector id ${vector.doc.id}`);
    }
    vectorIds.add(vector.doc.id);
    for (const reqId of vector.doc.requirement_ids ?? []) {
      if (!requirementIds.has(reqId)) {
        fail(vector.path, `requirement_ids entry not in registry: ${reqId}`);
      }
    }
    const errorish = [];
    collectErrorish(vector.doc, errorish);
    for (const code of errorish) {
      if (!errorCodes.has(code)) {
        fail(vector.path, `error-code-shaped value not in errors registry: ${code}`);
      }
    }
    if (!registeredTestIds.has(vector.doc.id)) {
      fail(vector.path, `vector id ${vector.doc.id} is not referenced by any registry test mapping (orphan vector)`);
    }
  }
  for (const testId of registeredTestIds) {
    if (!vectorIds.has(testId)) {
      fail("specs/registry/requirements.yaml", `test mapping ${testId} has no vector with that id (orphan test id)`);
    }
  }

  // Schema reachability: every schema must be reachable from an owner_spec,
  // a $ref edge, or an explicit mention in a normative doc/vector.
  const schemaNames = new Set(schemas.map((s) => s.name));
  const mentioned = new Set();
  for (const req of requirements.requirements) {
    const target = String(req.owner_spec ?? "").split("#")[0];
    if (target.startsWith("specs/schemas/")) {
      mentioned.add(target.slice("specs/schemas/".length));
    }
  }
  const mentionSources = [
    ...listMarkdownFiles().filter((p) => {
      const rel = toRepoRelative(p);
      return (
        rel.startsWith("specs/") ||
        rel.startsWith("docs/standards/") ||
        rel === "conformance/README.md" ||
        rel.startsWith("docs/adr/")
      );
    }),
  ];
  for (const src of mentionSources) {
    const text = readText(src);
    for (const name of schemaNames) {
      if (text.includes(name)) {
        mentioned.add(name);
      }
    }
  }
  for (const vector of vectors) {
    const text = JSON.stringify(vector.doc);
    for (const name of schemaNames) {
      if (text.includes(name)) {
        mentioned.add(name);
      }
    }
  }
  // Propagate through $ref edges until fixpoint.
  const refEdges = new Map();
  for (const schema of schemas) {
    const refs = [];
    const walk = (node) => {
      if (Array.isArray(node)) {
        node.forEach(walk);
      } else if (node && typeof node === "object") {
        for (const [key, value] of Object.entries(node)) {
          if (key === "$ref" && typeof value === "string") {
            const file = value.split("#")[0];
            if (file.length > 0) {
              refs.push(file);
              if (!schemaNames.has(file)) {
                fail(schema.path, `relative $ref target missing: ${file}`);
              }
            }
          }
          walk(value);
        }
      }
    };
    walk(schema.doc);
    refEdges.set(schema.name, refs);
  }
  let grew = true;
  while (grew) {
    grew = false;
    for (const [name, refs] of refEdges) {
      if (mentioned.has(name)) {
        for (const ref of refs) {
          if (!mentioned.has(ref)) {
            mentioned.add(ref);
            grew = true;
          }
        }
      }
    }
  }
  for (const schema of schemas) {
    if (!mentioned.has(schema.name)) {
      fail(schema.path, "orphan schema: not reachable from any owner_spec, $ref, normative doc, or vector");
    }
  }
}

// ---------- 4: relative markdown links resolve (living docs; frozen root reviews excluded)

const FROZEN_DOCS = new Set([
  "CognitiveOS-Architecture.md",
  "CognitiveOS-Architecture-Independent-Review.md",
  "CognitiveOS-Review-Conclusions.md",
  "RFC-0001-cognitiveos-governance-context-access.md",
]);
const linkPattern = /\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
for (const mdAbs of listMarkdownFiles()) {
  const rel = toRepoRelative(mdAbs);
  if (FROZEN_DOCS.has(rel)) {
    continue;
  }
  const text = readText(mdAbs);
  for (const match of text.matchAll(linkPattern)) {
    const raw = match[1];
    if (/^(https?|mailto|urn):/.test(raw) || raw.startsWith("#")) {
      continue;
    }
    const target = decodeURI(raw.split("#")[0]);
    if (target.length === 0) {
      continue;
    }
    const resolved = path.resolve(path.dirname(mdAbs), target);
    if (!existsSync(resolved)) {
      fail(rel, `broken relative link: ${raw}`);
    } else if (toRepoRelative(resolved).startsWith("History")) {
      fail(rel, `link into frozen History/: ${raw}`);
    }
  }
}

// ---------- 5a: REQ-ID and error-code references in living docs exist

if (registries) {
  const LIVING_SCOPES = [
    "docs/",
    ".cursor/rules/",
    "AGENTS.md",
    "README.md",
    "tools/",
    "crates/",
    "packages/",
  ];
  // Negative lookbehind: a vector id like `CTX-REQ-007` must not have its
  // `REQ-007` tail misread as a requirement reference; a real requirement
  // id is never preceded by another id segment.
  const reqPattern = /(?<![A-Z0-9-])REQ-[A-Z0-9]+(?:-[A-Z0-9]+)*/g;
  for (const mdAbs of listMarkdownFiles()) {
    const rel = toRepoRelative(mdAbs);
    if (!LIVING_SCOPES.some((scope) => rel === scope || rel.startsWith(scope))) {
      continue;
    }
    const text = readText(mdAbs);
    for (const match of text.matchAll(reqPattern)) {
      const id = match[0].replace(/-$/, "");
      // Domain-level references like "REQ-CTX" or wildcard prose "REQ-EFF"
      // are allowed; only full IDs ending in a numeric segment are checked.
      if (!/-\d+$/.test(id)) {
        continue;
      }
      if (!registries.requirementIds.has(id)) {
        fail(rel, `orphan requirement reference: ${id}`);
      }
    }
  }
}

// ---------- 5b: traceability matrix complete and paths exist

const matrixPath = repoPath("docs", "traceability", "matrix.yaml");
if (!existsSync(matrixPath)) {
  fail("docs/traceability/matrix.yaml", "traceability matrix missing");
} else if (registries) {
  try {
    const matrix = readYaml(matrixPath);
    const entries = matrix?.requirements ?? [];
    const matrixIds = new Set(entries.map((e) => e.id));
    for (const req of registries.requirements.requirements) {
      if (!matrixIds.has(req.id)) {
        fail("docs/traceability/matrix.yaml", `registry requirement missing from matrix: ${req.id}`);
      }
    }
    for (const entry of entries) {
      if (!registries.requirementIds.has(entry.id)) {
        fail("docs/traceability/matrix.yaml", `matrix entry not in registry: ${entry.id}`);
      }
      for (const listField of ["vectors", "impl", "impl_tests", "evidence", "docs"]) {
        for (const p of entry[listField] ?? []) {
          const target = String(p).split("#")[0];
          if (!existsSync(repoPath(...target.split("/")))) {
            fail("docs/traceability/matrix.yaml", `${entry.id}.${listField} path missing: ${target}`);
          }
        }
      }
    }
  } catch (err) {
    fail("docs/traceability/matrix.yaml", `unparseable matrix: ${err.message}`);
  }
}

// ---------- 5c: findings ledger covers every F and IMP item

const ledgerPath = repoPath("docs", "traceability", "findings-ledger.md");
if (!existsSync(ledgerPath)) {
  fail("docs/traceability/findings-ledger.md", "findings ledger missing");
} else {
  const ledger = readText(ledgerPath);
  for (let i = 1; i <= 30; i += 1) {
    const id = `F-${String(i).padStart(3, "0")}`;
    if (!ledger.includes(id)) {
      fail("docs/traceability/findings-ledger.md", `missing finding entry ${id}`);
    }
  }
  for (let i = 1; i <= 18; i += 1) {
    const id = `IMP-${String(i).padStart(2, "0")}`;
    if (!ledger.includes(id)) {
      fail("docs/traceability/findings-ledger.md", `missing improvement entry ${id}`);
    }
  }
}

// ---------- 6: project identity, Personal snapshot, and active leases

const projectScopePath = repoPath("docs", "governance", "project-scope.yaml");
if (!existsSync(projectScopePath)) {
  fail("docs/governance/project-scope.yaml", "project identity machine mirror missing");
} else {
  try {
    const projectScope = readYaml(projectScopePath);
    if (projectScope?.repository_role !== "architecture-reference-plus-single-active-project") {
      fail(
        "docs/governance/project-scope.yaml",
        "repository_role must separate the architecture reference from one active project",
      );
    }
    if (projectScope?.active_project?.id !== "cognitiveos-personal") {
      fail(
        "docs/governance/project-scope.yaml",
        "active_project.id must be cognitiveos-personal",
      );
    }
    if (projectScope?.active_project?.status !== "active") {
      fail("docs/governance/project-scope.yaml", "cognitiveos-personal must be active");
    }
    for (const sourceField of ["formal_plan", "current_snapshot", "lease_ledger"]) {
      const sourcePath = projectScope?.active_project?.[sourceField];
      if (typeof sourcePath !== "string" || !existsSync(repoPath(...sourcePath.split("/")))) {
        fail(
          "docs/governance/project-scope.yaml",
          `active_project.${sourceField} must reference an existing repository path`,
        );
      }
    }
  } catch (err) {
    fail("docs/governance/project-scope.yaml", `unparseable project identity: ${err.message}`);
  }
}

const projectIdentityPath = repoPath("docs", "governance", "PROJECT-IDENTITY.md");
if (!existsSync(projectIdentityPath)) {
  fail("docs/governance/PROJECT-IDENTITY.md", "project identity governance document missing");
} else {
  const projectIdentity = readText(projectIdentityPath);
  if (!projectIdentity.includes("`cognitiveos-personal`")) {
    fail(
      "docs/governance/PROJECT-IDENTITY.md",
      "canonical cognitiveos-personal project id is not declared",
    );
  }
  if (!projectIdentity.includes("唯一活动实现项目")) {
    fail(
      "docs/governance/PROJECT-IDENTITY.md",
      "the sole active implementation project boundary is not explicit",
    );
  }
}

const progressPath = repoPath("docs", "plan", "PROGRESS.md");
const lanesPath = repoPath("docs", "plan", "PARALLEL-LANES.md");
if (existsSync(progressPath) && existsSync(lanesPath)) {
  const progressText = readText(progressPath);
  const currentSnapshot = progressText.split(/^## Historical evidence journal/m, 1)[0];
  if (!currentSnapshot.includes("`cognitiveos-personal`")) {
    fail("docs/plan/PROGRESS.md", "Current snapshot does not identify cognitiveos-personal");
  }

  const lanesText = readText(lanesPath);
  const activeLeaseSectionMatch = lanesText.match(
    /^## 3\. 活动 ownership leases[^\n]*\n([\s\S]*?)(?=^### 3\.1 )/m,
  );
  if (!activeLeaseSectionMatch) {
    fail("docs/plan/PARALLEL-LANES.md", "canonical active lease section is missing");
  } else {
    const activeLeaseRows = activeLeaseSectionMatch[1]
      .split(/\r?\n/)
      .filter((line) => line.startsWith("| `lease/"));
    const activeLeases = [];
    const seenLeaseIds = new Set();
    const writableOwners = [];

    for (const row of activeLeaseRows) {
      const columns = row
        .split("|")
        .slice(1, -1)
        .map((column) => column.trim());
      if (columns.length !== 8) {
        fail("docs/plan/PARALLEL-LANES.md", `active lease row must have 8 columns: ${row}`);
        continue;
      }
      const leaseId = columns[0].replaceAll("`", "");
      const writablePaths = [...columns[4].matchAll(/`([^`]+)`/g)].map((match) => match[1]);
      const status = columns[7];
      if (!/^lease\/personal\/[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)+$/.test(leaseId)) {
        fail("docs/plan/PARALLEL-LANES.md", `invalid active lease_id: ${leaseId}`);
      }
      if (seenLeaseIds.has(leaseId)) {
        fail("docs/plan/PARALLEL-LANES.md", `duplicate active lease_id: ${leaseId}`);
      }
      seenLeaseIds.add(leaseId);
      if (status !== "active") {
        fail(
          "docs/plan/PARALLEL-LANES.md",
          `non-active lease ${leaseId} must move out of the active table`,
        );
      }
      if (writablePaths.length === 0) {
        fail("docs/plan/PARALLEL-LANES.md", `active lease ${leaseId} has no writable paths`);
      }
      activeLeases.push(leaseId);
      for (const writablePath of writablePaths) {
        const normalizedPath = writablePath.replace(/\/\*\*$/, "").replaceAll("\\", "/");
        for (const existingOwner of writableOwners) {
          const pathsOverlap =
            normalizedPath === existingOwner.path ||
            normalizedPath.startsWith(`${existingOwner.path}/`) ||
            existingOwner.path.startsWith(`${normalizedPath}/`);
          if (pathsOverlap && existingOwner.leaseId !== leaseId) {
            fail(
              "docs/plan/PARALLEL-LANES.md",
              `overlapping active writable paths: ${existingOwner.leaseId}:${existingOwner.path} and ${leaseId}:${normalizedPath}`,
            );
          }
        }
        writableOwners.push({ leaseId, path: normalizedPath });
      }
    }

    const progressLeaseRow = currentSnapshot
      .split(/\r?\n/)
      .find((line) => line.startsWith("| Active task lease |"));
    if (!progressLeaseRow) {
      fail("docs/plan/PROGRESS.md", "Current snapshot has no Active task lease row");
    } else {
      const referencedLeaseIds = [...progressLeaseRow.matchAll(/`(lease\/personal\/[^`]+)`/g)].map(
        (match) => match[1],
      );
      for (const activeLease of activeLeases) {
        if (!referencedLeaseIds.includes(activeLease)) {
          fail("docs/plan/PROGRESS.md", `active lease is not referenced: ${activeLease}`);
        }
      }
      for (const referencedLeaseId of referencedLeaseIds) {
        if (!activeLeases.includes(referencedLeaseId)) {
          fail("docs/plan/PROGRESS.md", `referenced lease is not active: ${referencedLeaseId}`);
        }
      }
      if (activeLeases.length === 0 && !progressLeaseRow.includes("`none`")) {
        fail("docs/plan/PROGRESS.md", "zero active leases must be represented as `none`");
      }
    }
  }
}

// ---------- report

const schemaCount = schemas.length;
const vectorCount = vectors.length;
const reqCount = registries?.requirements.requirements.length ?? 0;
const errCount = registries ? registries.errorCodes.size : 0;

if (failures.length > 0) {
  console.error(`check-consistency: ${failures.length} violation(s)\n`);
  for (const { file, reason } of failures) {
    console.error(`  ${file}\n    ${reason}`);
  }
  process.exit(1);
}
console.log(
  `check-consistency: OK (${reqCount} requirements, ${errCount} error codes, ` +
    `${schemaCount} schemas, ${vectorCount} vectors, links, traceability, project identity, and leases verified)`,
);
