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
 *  6. Personal plan/trace/Gate facts, project identity, prompt status, and
 *     active ownership leases have one consistent source of truth.
 *
 * Exit code 0 = green; 1 = at least one violation, each printed with file
 * and reason. History/ is never scanned (frozen archive).
 */

import { existsSync } from "node:fs";
import path from "node:path";
import { Ajv2020 } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import {
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

function parseMarkdownTableRow(line) {
  if (!line.startsWith("|") || !line.endsWith("|")) {
    return [];
  }
  return line
    .split("|")
    .slice(1, -1)
    .map((column) => column.trim());
}

function normalizeMarkdownCell(cell) {
  return cell.replaceAll("**", "").replaceAll("`", "").trim();
}

function containsObjectKey(value, searchedKey) {
  if (Array.isArray(value)) {
    return value.some((item) => containsObjectKey(item, searchedKey));
  }
  if (value && typeof value === "object") {
    return Object.entries(value).some(
      ([key, childValue]) =>
        key === searchedKey || containsObjectKey(childValue, searchedKey),
    );
  }
  return false;
}

function parseIsoCalendarDate(value) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return undefined;
  }
  const timestamp = Date.parse(`${value}T00:00:00Z`);
  if (Number.isNaN(timestamp)) {
    return undefined;
  }
  return new Date(timestamp).toISOString().slice(0, 10) === value ? timestamp : undefined;
}

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

// ---------- 6a: Personal plan task definitions, summary counts, and trace

const personalPlanPath = repoPath("docs", "plan", "PERSONAL-DEVELOPMENT-PLAN.md");
const personalTracePath = repoPath("docs", "plan", "personal-trace.yaml");
const knownTaskStatuses = new Set([
  "not-started",
  "in-progress",
  "blocked",
  "done",
  "cancelled",
]);
const knownDeliverySliceStatuses = new Set([
  "ready",
  "in-progress",
  "blocked",
  "done",
  "cancelled",
]);
const personalTaskIds = new Set();
const personalDeliverySliceIds = new Set();
let personalPlanText = "";
let computedPersonalTaskTotals;

if (!existsSync(personalPlanPath)) {
  fail("docs/plan/PERSONAL-DEVELOPMENT-PLAN.md", "formal Personal plan missing");
} else {
  personalPlanText = readText(personalPlanPath);
  const taskDefinitions = [];
  for (const line of personalPlanText.split(/\r?\n/)) {
    const columns = parseMarkdownTableRow(line);
    if (columns.length < 6) {
      continue;
    }
    const taskId = normalizeMarkdownCell(columns[0]);
    const taskStatus = normalizeMarkdownCell(columns[4]);
    const taskIdMatch = taskId.match(/^P(\d+)-T\d+$/);
    if (!taskIdMatch || !knownTaskStatuses.has(taskStatus)) {
      continue;
    }
    if (personalTaskIds.has(taskId)) {
      fail(
        "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
        `duplicate formal task definition: ${taskId}`,
      );
    }
    personalTaskIds.add(taskId);
    taskDefinitions.push({
      id: taskId,
      phase: Number(taskIdMatch[1]),
      status: taskStatus,
    });
  }

  for (const line of personalPlanText.split(/\r?\n/)) {
    const columns = parseMarkdownTableRow(line);
    if (columns.length !== 5) {
      continue;
    }
    const deliverySliceId = normalizeMarkdownCell(columns[0]);
    const deliverySliceIdMatch = deliverySliceId.match(/^(P\d+-T\d+)\/D\d{2}$/);
    if (!deliverySliceIdMatch) {
      continue;
    }
    if (personalDeliverySliceIds.has(deliverySliceId)) {
      fail(
        "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
        `duplicate formal delivery slice definition: ${deliverySliceId}`,
      );
    }
    personalDeliverySliceIds.add(deliverySliceId);

    const parentTaskId = deliverySliceIdMatch[1];
    const declaredTaskId = normalizeMarkdownCell(columns[1]);
    if (!personalTaskIds.has(parentTaskId) || declaredTaskId !== parentTaskId) {
      fail(
        "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
        `${deliverySliceId} must reference its existing parent task ${parentTaskId}`,
      );
    }
    const requiredDefinitionCells = columns.slice(2).map(normalizeMarkdownCell);
    if (requiredDefinitionCells.some((cell) => cell.length === 0 || cell === "—")) {
      fail(
        "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
        `${deliverySliceId} must declare outcome, implementation dependency, and required validation`,
      );
    }
  }
  if (personalDeliverySliceIds.size === 0) {
    fail(
      "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
      "formal plan has no registered delivery slices",
    );
  }

  const summaryRows = new Map();
  let declaredTotals;
  for (const line of personalPlanText.split(/\r?\n/)) {
    const columns = parseMarkdownTableRow(line);
    if (columns.length !== 7) {
      continue;
    }
    const summaryLabel = normalizeMarkdownCell(columns[0]);
    const phaseMatch = summaryLabel.match(/^Phase (\d+)\b/);
    const parsedCounts = columns.slice(1, 6).map((column) => Number(normalizeMarkdownCell(column)));
    if (parsedCounts.some((count) => !Number.isInteger(count))) {
      continue;
    }
    if (phaseMatch) {
      summaryRows.set(Number(phaseMatch[1]), parsedCounts);
    } else if (summaryLabel === "合计") {
      declaredTotals = parsedCounts;
    }
  }

  const computedTotals = [0, 0, 0, 0, 0];
  for (const [phase, declaredCounts] of summaryRows) {
    const phaseTasks = taskDefinitions.filter((task) => task.phase === phase);
    const computedCounts = [
      phaseTasks.length,
      phaseTasks.filter((task) => task.status === "done").length,
      phaseTasks.filter((task) => task.status === "in-progress").length,
      phaseTasks.filter((task) => task.status === "blocked").length,
      phaseTasks.filter((task) => task.status === "not-started").length,
    ];
    for (let countIndex = 0; countIndex < computedCounts.length; countIndex += 1) {
      computedTotals[countIndex] += computedCounts[countIndex];
    }
    if (declaredCounts.some((count, countIndex) => count !== computedCounts[countIndex])) {
      fail(
        "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
        `Phase ${phase} summary counts ${declaredCounts.join("/")} do not match task rows ${computedCounts.join("/")}`,
      );
    }
  }
  if (summaryRows.size === 0) {
    fail("docs/plan/PERSONAL-DEVELOPMENT-PLAN.md", "progress summary has no Phase rows");
  }
  for (const taskDefinition of taskDefinitions) {
    if (!summaryRows.has(taskDefinition.phase)) {
      fail(
        "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
        `${taskDefinition.id} belongs to Phase ${taskDefinition.phase}, which has no summary row`,
      );
    }
  }
  if (!declaredTotals) {
    fail("docs/plan/PERSONAL-DEVELOPMENT-PLAN.md", "progress summary has no total row");
  } else if (
    declaredTotals.some((declaredCount, countIndex) => declaredCount !== computedTotals[countIndex])
  ) {
    fail(
      "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
      `summary totals ${declaredTotals.join("/")} do not match task rows ${computedTotals.join("/")}`,
    );
  }
  computedPersonalTaskTotals = computedTotals;
}

if (!existsSync(personalTracePath)) {
  fail("docs/plan/personal-trace.yaml", "Personal plan trace missing");
} else {
  try {
    const personalTrace = readYaml(personalTracePath);
    if (containsObjectKey(personalTrace, "current_snapshot")) {
      fail(
        "docs/plan/personal-trace.yaml",
        "trace must not copy a parallel current_snapshot; PROGRESS.md owns current facts",
      );
    }

    for (const [sourceName, sourcePath] of Object.entries(personalTrace?.sources ?? {})) {
      if (typeof sourcePath !== "string" || !existsSync(repoPath(...sourcePath.split("/")))) {
        fail(
          "docs/plan/personal-trace.yaml",
          `sources.${sourceName} must reference an existing repository path`,
        );
      }
    }

    const tracedDeliverySliceStatuses = new Set(
      personalTrace?.status_dimensions?.delivery_slice_status ?? [],
    );
    for (const knownDeliverySliceStatus of knownDeliverySliceStatuses) {
      if (!tracedDeliverySliceStatuses.has(knownDeliverySliceStatus)) {
        fail(
          "docs/plan/personal-trace.yaml",
          `status_dimensions.delivery_slice_status is missing ${knownDeliverySliceStatus}`,
        );
      }
    }
    for (const tracedDeliverySliceStatus of tracedDeliverySliceStatuses) {
      if (!knownDeliverySliceStatuses.has(tracedDeliverySliceStatus)) {
        fail(
          "docs/plan/personal-trace.yaml",
          `status_dimensions.delivery_slice_status contains unknown value ${tracedDeliverySliceStatus}`,
        );
      }
    }

    const referencedTasks = new Set(personalTrace?.enabling_tasks ?? []);
    const referencedGates = new Set();
    for (const requirement of personalTrace?.personal_requirements ?? []) {
      for (const taskId of requirement.tasks ?? []) {
        referencedTasks.add(taskId);
      }
      for (const gateId of requirement.gates ?? []) {
        referencedGates.add(gateId);
      }
    }
    for (const gateDefinition of Object.values(personalTrace?.gate_catalog ?? {})) {
      for (const taskId of gateDefinition?.tasks ?? []) {
        referencedTasks.add(taskId);
      }
      for (const benchmarkId of gateDefinition?.benchmarks ?? []) {
        referencedGates.add(benchmarkId);
      }
    }
    for (const taskId of referencedTasks) {
      if (!personalTaskIds.has(taskId)) {
        fail("docs/plan/personal-trace.yaml", `trace references unknown formal task: ${taskId}`);
      }
    }

    const formalGateIds = new Set(
      personalPlanText.match(/\b(?:G\d+|B\d{2}(?:-W)?|GMVP-LINUX|RC)\b/g) ?? [],
    );
    for (const gateId of [
      ...Object.keys(personalTrace?.gate_catalog ?? {}),
      ...referencedGates,
    ]) {
      if (!formalGateIds.has(gateId)) {
        fail("docs/plan/personal-trace.yaml", `trace references unknown formal Gate: ${gateId}`);
      }
    }
  } catch (err) {
    fail("docs/plan/personal-trace.yaml", `unparseable Personal plan trace: ${err.message}`);
  }
}

// ---------- 6b: project identity and canonical design sources

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
    for (const sourceField of [
      "formal_plan",
      "current_snapshot",
      "lease_ledger",
      "product_design",
      "architecture_design",
    ]) {
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

// ---------- 6c: fail-fast local shell and validation-environment guards

const commandEnvironmentGuardDocuments = [
  {
    path: "AGENTS.md",
    requiredFragments: [
      "COMMAND-SHELL-PS51",
      "Windows PowerShell 5.1",
      "RUST-LINK-DEV-WIN-GNU-01",
      "linker exit 121",
      "CI-WINDOWS-MSVC-01",
    ],
  },
  {
    path: "docs/governance/DEVELOPMENT-OPERATING-MODEL.md",
    requiredFragments: [
      "COMMAND-SHELL-PS51",
      "Do not use `&&` or `||`",
      "RUST-LINK-DEV-WIN-GNU-01",
      "Do not repeat them",
      "DEV-LINUX-NATIVE-01",
    ],
  },
  {
    path: "docs/plan/PERSONAL-TEST-ENVIRONMENTS.md",
    requiredFragments: [
      "COMMAND-SHELL-PS51",
      "RUST-LINK-DEV-WIN-GNU-01",
      "No-repeat rule",
      "CI-UBUNTU-01",
      "CI-WINDOWS-MSVC-01",
    ],
  },
  {
    path: "tests/baseline/README.md",
    requiredFragments: [
      "COMMAND-SHELL-PS51",
      "RUST-LINK-DEV-WIN-GNU-01",
      "must not rerun",
      "linker reported exit code `121`",
    ],
  },
];

for (const commandEnvironmentGuardDocument of commandEnvironmentGuardDocuments) {
  const commandEnvironmentGuardPath = repoPath(
    ...commandEnvironmentGuardDocument.path.split("/"),
  );
  if (!existsSync(commandEnvironmentGuardPath)) {
    fail(
      commandEnvironmentGuardDocument.path,
      "command/environment guard document is missing",
    );
    continue;
  }
  const commandEnvironmentGuardText = readText(commandEnvironmentGuardPath);
  for (const requiredFragment of commandEnvironmentGuardDocument.requiredFragments) {
    if (!commandEnvironmentGuardText.includes(requiredFragment)) {
      fail(
        commandEnvironmentGuardDocument.path,
        `command/environment guard is missing required fragment: ${requiredFragment}`,
      );
    }
  }
}

// ---------- 6c.1: recoverable checkpoint and merge-boundary guards

const checkpointDeliveryGuardDocuments = [
  {
    path: "AGENTS.md",
    requiredFragments: [
      "CHECKPOINT-DELIVERY-01",
      "后台持久化事件",
      "Draft PR",
      "完整 task acceptance",
      "转为 ready 并合并",
      "禁止 merge",
      "dirty handoff",
    ],
  },
  {
    path: "docs/governance/DEVELOPMENT-OPERATING-MODEL.md",
    requiredFragments: [
      "CHECKPOINT-DELIVERY-01",
      "Checkpoint persistence is not task closure",
      "standing delivery authorization",
      "mark the PR ready and merge it",
      "Draft PR",
      "ready PR",
      "Fast resume protocol",
      "dirty handoff",
    ],
  },
  {
    path: "docs/standards/docs-sync-contract.md",
    requiredFragments: [
      "CHECKPOINT-DELIVERY-01",
      "checkpoint-delivery guard removal",
      "后台持久化事件",
      "同一个 task Draft PR",
      "完整 task acceptance 未满足前",
      "dirty handoff",
    ],
  },
];

for (const checkpointDeliveryGuardDocument of checkpointDeliveryGuardDocuments) {
  const checkpointDeliveryGuardPath = repoPath(
    ...checkpointDeliveryGuardDocument.path.split("/"),
  );
  if (!existsSync(checkpointDeliveryGuardPath)) {
    fail(
      checkpointDeliveryGuardDocument.path,
      "checkpoint-delivery guard document is missing",
    );
    continue;
  }
  const checkpointDeliveryGuardText = readText(checkpointDeliveryGuardPath);
  for (const requiredFragment of checkpointDeliveryGuardDocument.requiredFragments) {
    if (!checkpointDeliveryGuardText.includes(requiredFragment)) {
      fail(
        checkpointDeliveryGuardDocument.path,
        `checkpoint-delivery guard is missing required fragment: ${requiredFragment}`,
      );
    }
  }
}

// ---------- 6c.2: whole-task atomic delivery and deterministic closure guards

const taskAtomicDeliveryGuardDocuments = [
  {
    path: "AGENTS.md",
    requiredFragments: [
      "TASK-ATOMIC-DELIVERY-01",
      "task branch、一个 Draft PR 和一个 task lease",
      "acceptance-assessment 分支",
      "MVP-first 授权与实现深度",
      "完整任务收口协议",
      "本地安全切回",
    ],
  },
  {
    path: "docs/governance/PROJECT-IDENTITY.md",
    requiredFragments: [
      "一个 task branch、一个持续更新的 Draft PR 和一个 task-scoped lease",
      "阶段总结和可恢复故障都不是",
      "首个 MVP 优先使用",
      "fast-forward `main`",
    ],
  },
  {
    path: "docs/governance/DEVELOPMENT-OPERATING-MODEL.md",
    requiredFragments: [
      "TASK-ATOMIC-DELIVERY-01",
      "one task branch, one Draft PR and one task-scoped lease",
      "MVP-first implementation and authorization",
      "Deterministic task closure",
      "Multiple formal tasks must not share one",
      "branch or PR",
    ],
  },
  {
    path: "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
    requiredFragments: [
      "TASK-ATOMIC-DELIVERY-01",
      "Slice 是内部检查点",
      "MVP-first",
      "不得留下“代码完成但验收、分支或状态待收口”",
    ],
  },
  {
    path: "docs/plan/PARALLEL-LANES.md",
    requiredFragments: [
      "一个 task branch/Draft PR + 一份活动 task lease",
      "不得因 Slice、checkpoint、push、CI 轮次",
      "一个 branch/PR 不得承载多个正式任务",
      "fast-forward `main`",
    ],
  },
  {
    path: "docs/standards/docs-sync-contract.md",
    requiredFragments: [
      "TASK-ATOMIC-DELIVERY-01",
      "一个正式任务使用一个 task branch、一个 Draft PR 和一个",
      "不得遗留独立 `acceptance-assessment` 分支",
      "deterministic task closure",
    ],
  },
];

for (const taskAtomicDeliveryGuardDocument of taskAtomicDeliveryGuardDocuments) {
  const taskAtomicDeliveryGuardPath = repoPath(
    ...taskAtomicDeliveryGuardDocument.path.split("/"),
  );
  if (!existsSync(taskAtomicDeliveryGuardPath)) {
    fail(
      taskAtomicDeliveryGuardDocument.path,
      "task-atomic delivery guard document is missing",
    );
    continue;
  }
  const taskAtomicDeliveryGuardText = readText(taskAtomicDeliveryGuardPath);
  for (const requiredFragment of taskAtomicDeliveryGuardDocument.requiredFragments) {
    if (!taskAtomicDeliveryGuardText.includes(requiredFragment)) {
      fail(
        taskAtomicDeliveryGuardDocument.path,
        `task-atomic delivery guard is missing required fragment: ${requiredFragment}`,
      );
    }
  }
}

// ---------- 6d: dated prompt boundary and B01 Gate honesty

const legacyPromptPrefixPath = repoPath("docs", "prompts", "common-prefix.md");
if (!existsSync(legacyPromptPrefixPath)) {
  fail("docs/prompts/common-prefix.md", "legacy prompt boundary document missing");
} else {
  const legacyPromptPrefix = readText(legacyPromptPrefixPath);
  if (
    !legacyPromptPrefix.includes("dated non-executable reference") ||
    !legacyPromptPrefix.includes("不是 CognitiveOS Personal 的可执行任务入口")
  ) {
    fail(
      "docs/prompts/common-prefix.md",
      "legacy prompts must be explicitly non-executable for current Personal work",
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

  const formalTaskProgressSectionMatch = currentSnapshot.match(
    /^### Layer 1 .*Formal task progress\s*\n([\s\S]*?)(?=^### Layer 2 )/m,
  );
  if (!formalTaskProgressSectionMatch) {
    fail("docs/plan/PROGRESS.md", "Current snapshot has no formal task progress layer");
  } else {
    const formalTaskProgressCounts = formalTaskProgressSectionMatch[1]
      .split(/\r?\n/)
      .map(parseMarkdownTableRow)
      .map((columns) => columns.map((column) => Number(normalizeMarkdownCell(column))))
      .find(
        (counts) =>
          counts.length === 6 && counts.every((count) => Number.isInteger(count)),
      );
    if (!formalTaskProgressCounts) {
      fail("docs/plan/PROGRESS.md", "formal task progress layer has no numeric summary row");
    } else if (computedPersonalTaskTotals) {
      const expectedRemainingCount =
        computedPersonalTaskTotals[0] - computedPersonalTaskTotals[1];
      const expectedProgressCounts = [
        ...computedPersonalTaskTotals,
        expectedRemainingCount,
      ];
      if (
        formalTaskProgressCounts.some(
          (count, countIndex) => count !== expectedProgressCounts[countIndex],
        )
      ) {
        fail(
          "docs/plan/PROGRESS.md",
          `formal task progress ${formalTaskProgressCounts.join("/")} does not match plan ${expectedProgressCounts.join("/")}`,
        );
      }
    }
  }

  const deliveryQueueSectionMatch = currentSnapshot.match(
    /^### Layer 2 .*Delivery Slice queue\s*\n([\s\S]*?)(?=^### Layer 3 )/m,
  );
  if (!deliveryQueueSectionMatch) {
    fail("docs/plan/PROGRESS.md", "Current snapshot has no Delivery Slice queue");
  } else {
    const currentDeliverySliceIds = new Set();
    const inProgressCountsByTask = new Map();
    const deliveryQueueRows = deliveryQueueSectionMatch[1]
      .split(/\r?\n/)
      .map(parseMarkdownTableRow)
      .filter((columns) => /^(P\d+-T\d+)\/D\d{2}$/.test(normalizeMarkdownCell(columns[0] ?? "")));

    for (const columns of deliveryQueueRows) {
      if (columns.length !== 4) {
        fail(
          "docs/plan/PROGRESS.md",
          `Delivery Slice queue row must have 4 columns: ${columns.join(" | ")}`,
        );
        continue;
      }
      const deliverySliceId = normalizeMarkdownCell(columns[0]);
      const deliverySliceStatus = normalizeMarkdownCell(columns[1]);
      if (currentDeliverySliceIds.has(deliverySliceId)) {
        fail("docs/plan/PROGRESS.md", `duplicate current delivery slice: ${deliverySliceId}`);
      }
      currentDeliverySliceIds.add(deliverySliceId);
      if (!personalDeliverySliceIds.has(deliverySliceId)) {
        fail("docs/plan/PROGRESS.md", `current queue references undefined slice: ${deliverySliceId}`);
      }
      if (!knownDeliverySliceStatuses.has(deliverySliceStatus)) {
        fail(
          "docs/plan/PROGRESS.md",
          `${deliverySliceId} has unknown delivery status: ${deliverySliceStatus}`,
        );
      }
      if (deliverySliceStatus === "in-progress") {
        const parentTaskId = deliverySliceId.split("/")[0];
        inProgressCountsByTask.set(
          parentTaskId,
          (inProgressCountsByTask.get(parentTaskId) ?? 0) + 1,
        );
      }
    }

    for (const deliverySliceId of personalDeliverySliceIds) {
      if (!currentDeliverySliceIds.has(deliverySliceId)) {
        fail("docs/plan/PROGRESS.md", `formal delivery slice is missing current status: ${deliverySliceId}`);
      }
    }
    for (const [parentTaskId, inProgressCount] of inProgressCountsByTask) {
      if (inProgressCount > 1) {
        fail(
          "docs/plan/PROGRESS.md",
          `${parentTaskId} has ${inProgressCount} in-progress delivery slices; maximum is 1`,
        );
      }
    }
  }

  const formalB01Row = personalPlanText
    .split(/\r?\n/)
    .map(parseMarkdownTableRow)
    .find((columns) => normalizeMarkdownCell(columns[0] ?? "") === "B01");
  const formalB01MinimumMatch = formalB01Row
    ?.join(" ")
    .match(/(?:固定\s*N?\s*=\s*|fixed\s+N\s*=\s*|至少\s*)(\d+)/i);
  const formalB01Minimum = formalB01MinimumMatch
    ? Number(formalB01MinimumMatch[1])
    : undefined;
  const formalB01SuccessThresholdMatch = formalB01Row
    ?.join(" ")
    .match(/至少\s*(\d+)\s*次成功/);
  const formalB01SuccessThreshold = formalB01SuccessThresholdMatch
    ? Number(formalB01SuccessThresholdMatch[1])
    : undefined;
  if (!formalB01Minimum || formalB01Minimum < 2) {
    fail(
      "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
      "B01 formal Gate must declare a multi-attempt minimum denominator",
    );
  }
  if (
    !formalB01SuccessThreshold ||
    !formalB01Minimum ||
    formalB01SuccessThreshold > formalB01Minimum
  ) {
    fail(
      "docs/plan/PERSONAL-DEVELOPMENT-PLAN.md",
      "B01 formal Gate must declare a valid success-count threshold",
    );
  }

  const currentB01Line = currentSnapshot
    .split(/\r?\n/)
    .find((line) => line.startsWith("| B01 first-install/first-conversation Gate |"));
  if (!currentB01Line) {
    fail("docs/plan/PROGRESS.md", "Current snapshot has no canonical B01 Gate row");
  } else {
    const currentB01Columns = parseMarkdownTableRow(currentB01Line);
    const currentB01Status = normalizeMarkdownCell(currentB01Columns[1] ?? "");
    const currentB01Evidence = normalizeMarkdownCell(currentB01Columns[2] ?? "");
    const currentAttemptMatch = currentB01Evidence.match(/Attempt\s+(\d+)/i);
    const currentMinimumMatch = currentB01Evidence.match(/minimum\s+(\d+)/i);
    const currentAttemptCount = currentAttemptMatch ? Number(currentAttemptMatch[1]) : undefined;
    const currentMinimum = currentMinimumMatch ? Number(currentMinimumMatch[1]) : undefined;
    const currentSuccessMatch = currentB01Evidence.match(/(\d+)\s+successes/i);
    const currentFailureMatch = currentB01Evidence.match(/(\d+)\s+failures/i);
    const currentSuccessCount = currentSuccessMatch ? Number(currentSuccessMatch[1]) : undefined;
    const currentFailureCount = currentFailureMatch ? Number(currentFailureMatch[1]) : undefined;

    if (formalB01Minimum && currentMinimum !== formalB01Minimum) {
      fail(
        "docs/plan/PROGRESS.md",
        `B01 Current snapshot denominator ${String(currentMinimum)} does not match formal minimum ${formalB01Minimum}`,
      );
    }
    if (currentB01Status === "pass") {
      if (!currentAttemptCount || !formalB01Minimum || currentAttemptCount < formalB01Minimum) {
        fail(
          "docs/plan/PROGRESS.md",
          "B01 cannot pass before the formal attempt denominator is complete",
        );
      }
      if (
        !currentSuccessCount ||
        !currentFailureCount ||
        currentSuccessCount + currentFailureCount !== formalB01Minimum
      ) {
        fail(
          "docs/plan/PROGRESS.md",
          "B01 pass must record success and failure counts that equal the formal denominator",
        );
      }
      if (
        formalB01SuccessThreshold &&
        (!currentSuccessCount || currentSuccessCount < formalB01SuccessThreshold)
      ) {
        fail(
          "docs/plan/PROGRESS.md",
          "B01 cannot pass below the formal success-count threshold",
        );
      }
      const verifierClosureIsMissing =
        !/independent verifier/i.test(currentB01Evidence) ||
        /not yet|missing|尚未|未完成|不完整/i.test(currentB01Evidence);
      if (verifierClosureIsMissing) {
        fail(
          "docs/plan/PROGRESS.md",
          "B01 cannot pass without affirmative independent verifier closure",
        );
      }
      if (
        !/(?:success rate|成功率)/i.test(currentB01Evidence) ||
        !/(?:zero critical|关键安全失败为?\s*0)/i.test(currentB01Evidence) ||
        !/(?:aggregate|汇总|统计)/i.test(currentB01Evidence)
      ) {
        fail(
          "docs/plan/PROGRESS.md",
          "B01 pass must record success rate, zero critical failures, and aggregate statistics",
        );
      }
    }
  }

  // ---------- 6e: active ownership leases

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
    const forbiddenBroadProtectedTrees = new Set([
      "docs/plan/**",
      "docs/standards/**",
      "docs/adr/**",
      "specs/**",
    ]);

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
      const taskDescription = normalizeMarkdownCell(columns[1]);
      const primaryLane = normalizeMarkdownCell(columns[2]);
      const branch = normalizeMarkdownCell(columns[3]);
      const ownerSession = normalizeMarkdownCell(columns[5]);
      const claimedHeartbeatMatch = normalizeMarkdownCell(columns[6]).match(
        /^(\d{4}-\d{2}-\d{2})\s*\/\s*(\d{4}-\d{2}-\d{2})$/,
      );
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
      if (!taskDescription || !primaryLane || !branch || !ownerSession) {
        fail(
          "docs/plan/PARALLEL-LANES.md",
          `active lease ${leaseId} must declare task, lane, branch, and owner/session`,
        );
      }
      if (!claimedHeartbeatMatch) {
        fail(
          "docs/plan/PARALLEL-LANES.md",
          `active lease ${leaseId} must declare claimed/heartbeat as YYYY-MM-DD / YYYY-MM-DD`,
        );
      } else {
        const claimedTimestamp = parseIsoCalendarDate(claimedHeartbeatMatch[1]);
        const heartbeatTimestamp = parseIsoCalendarDate(claimedHeartbeatMatch[2]);
        const tomorrowTimestamp = Date.now() + 24 * 60 * 60 * 1000;
        if (
          claimedTimestamp === undefined ||
          heartbeatTimestamp === undefined ||
          heartbeatTimestamp < claimedTimestamp ||
          heartbeatTimestamp > tomorrowTimestamp
        ) {
          fail(
            "docs/plan/PARALLEL-LANES.md",
            `active lease ${leaseId} has invalid or non-monotonic claimed/heartbeat dates`,
          );
        }
      }
      activeLeases.push(leaseId);
      for (const writablePath of writablePaths) {
        const normalizedDeclaredPath = writablePath.replaceAll("\\", "/");
        if (forbiddenBroadProtectedTrees.has(normalizedDeclaredPath)) {
          fail(
            "docs/plan/PARALLEL-LANES.md",
            `active lease ${leaseId} claims forbidden broad protected tree: ${normalizedDeclaredPath}`,
          );
        }
        if (normalizedDeclaredPath === "docs/plan/PARALLEL-LANES.md") {
          fail(
            "docs/plan/PARALLEL-LANES.md",
            `active lease ${leaseId} must not own the lease ledger itself`,
          );
        }
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
    `${schemaCount} schemas, ${vectorCount} vectors, links, traceability, Personal plan/Gates, ` +
    `design sources, command/environment routing, checkpoint delivery, task-atomic delivery, ` +
    `prompt boundary, and leases verified)`,
);
