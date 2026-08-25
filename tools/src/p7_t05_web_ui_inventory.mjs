/**
 * P7-T05/D01 Web UI route-inventory validator.
 *
 * The inventory may only name existing daemon method/path literals or an
 * explicit unavailable/not-run disposition. It must not invent a public
 * contract, a generic lifecycle transition, a Task-channel Provider writer,
 * or a browser authority path.
 */

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

export const INVENTORY_SCHEMA = "cognitiveos.personal.web-ui-route-inventory/0.1";
export const FORBIDDEN_INVENTED_ROUTES = Object.freeze([
  "POST /management/agent/transition",
  "POST /management/lifecycle",
  "POST /task/complete",
  "POST /task/cancel",
  "POST /management/resource/v1/complete",
  "POST /management/resource/v1/create",
  "POST /management/resource/v1/install",
  "POST /management/resource/v1/execute",
]);
export const FORBIDDEN_BROWSER_TARGETS = Object.freeze([
  "sqlite",
  "secretstore",
  "filesystem",
  "shell",
  "provider-direct",
]);
export const MEMORY_ONLY_SESSION = "memory";

const PERSONAL_SOURCE_ROOT = ["personal", "apps", "kernel-server", "src", "personal"];
// ADR-0054: the formal SPA path is the imported clients subproject.
const FORMAL_SPA_ROOT = ["clients", "pc", "web"];
const CONSOLE_STUB = ["clients", "legacy", "cognitiveos-console"];
const CONSOLE_IMPLEMENTATION_MARKERS = [
  "package.json",
  "vite.config.ts",
  "vite.config.js",
  "src/main.tsx",
  "src/main.ts",
  "index.html",
];

function walkFiles(root, suffix) {
  const files = [];
  if (!existsSync(root)) {
    return files;
  }
  const stack = [root];
  while (stack.length > 0) {
    const current = stack.pop();
    const entries = readdirSync(current);
    for (const entry of entries) {
      const full = path.join(current, entry);
      const stats = statSync(full);
      if (stats.isDirectory()) {
        stack.push(full);
      } else if (full.endsWith(suffix)) {
        files.push(full);
      }
    }
  }
  return files;
}

function daemonSourceContains(repositoryRoot, methodPath) {
  const root = path.join(repositoryRoot, ...PERSONAL_SOURCE_ROOT);
  const files = walkFiles(root, ".rs");
  const needle = methodPath;
  const needleWithSpace = `${methodPath} `;
  for (const file of files) {
    const text = readFileSync(file, "utf8");
    if (text.includes(`"${needle}"`) || text.includes(`"${needleWithSpace}"`)) {
      return true;
    }
    if (text.includes(needle)) {
      return true;
    }
  }
  return false;
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

export function validateWebUiRouteInventory(inventory, options = {}) {
  const repositoryRoot = options.repositoryRoot;
  const fileExists =
    options.fileExists ??
    ((relativePath) =>
      typeof repositoryRoot === "string" &&
      existsSync(path.join(repositoryRoot, ...relativePath.split("/"))));
  const daemonHasRoute =
    options.daemonHasRoute ??
    ((methodPath) => daemonSourceContains(repositoryRoot, methodPath));

  assert(inventory?.schema === INVENTORY_SCHEMA, `inventory schema must be ${INVENTORY_SCHEMA}`);
  assert(inventory.claim_ceiling === "hypothesis", "inventory claim_ceiling must be hypothesis");
  assert(
    inventory.linux_1_0_blocking === false,
    "P7-T05 must remain non-blocking for Linux 1.0",
  );
  assert(
    inventory.session_storage === MEMORY_ONLY_SESSION,
    "session_storage must be memory-only",
  );
  assert(
    Array.isArray(inventory.browser_forbidden_targets) &&
      FORBIDDEN_BROWSER_TARGETS.every((target) =>
        inventory.browser_forbidden_targets.includes(target),
      ),
    "inventory must forbid sqlite/secretstore/filesystem/shell/provider-direct browser targets",
  );
  assert(Array.isArray(inventory.operations) && inventory.operations.length > 0, "operations required");
  assert(Array.isArray(inventory.forbidden_routes), "forbidden_routes required");

  if (typeof repositoryRoot === "string") {
    // ADR-0054: clients/ is the imported formal client subproject; the only
    // SPA implementation path is clients/pc/web (ADR-0053 location updated).
    assert(
      existsSync(path.join(repositoryRoot, ...FORMAL_SPA_ROOT)),
      "the formal SPA path clients/pc/web must exist (ADR-0054 fold-in)",
    );
    const consoleRoot = path.join(repositoryRoot, ...CONSOLE_STUB);
    for (const marker of CONSOLE_IMPLEMENTATION_MARKERS) {
      assert(
        !existsSync(path.join(consoleRoot, marker)),
        `Web UI must not be implemented in the legacy cognitiveos-console stub (${marker})`,
      );
    }
  }

  const seen = new Set();
  const availableSecretOps = [];
  for (const operation of inventory.operations) {
    assert(typeof operation?.id === "string" && operation.id.length > 0, "operation missing id");
    assert(!seen.has(operation.id), `duplicate operation ${operation.id}`);
    seen.add(operation.id);
    const disposition = operation.disposition;
    assert(
      ["available", "unavailable", "forbidden_cross_channel", "client-only"].includes(disposition),
      `${operation.id} has unknown disposition ${disposition}`,
    );
    if (disposition === "available") {
      assert(typeof operation.method_path === "string", `${operation.id} available op missing method_path`);
      assert(
        !FORBIDDEN_INVENTED_ROUTES.includes(operation.method_path),
        `${operation.id} must not claim invented route ${operation.method_path}`,
      );
      assert(
        daemonHasRoute(operation.method_path),
        `${operation.id} claims ${operation.method_path} which is not present in daemon source`,
      );
      if (operation.secret_bearing === true) {
        availableSecretOps.push(operation);
        assert(
          operation.channel === "management",
          `${operation.id} secret-bearing route must stay on the management channel`,
        );
      }
    }
    if (disposition === "unavailable") {
      assert(operation.ui_render === "not-run" || operation.ui_render === "unavailable", `${operation.id} unavailable op must render not-run/unavailable`);
      assert(typeof operation.missing_dependency === "string" && operation.missing_dependency.length > 0, `${operation.id} unavailable op missing dependency`);
      if (typeof operation.method_path === "string") {
        assert(
          FORBIDDEN_INVENTED_ROUTES.includes(operation.method_path) || !daemonHasRoute(operation.method_path),
          `${operation.id} marked unavailable but ${operation.method_path} exists as a typed daemon route`,
        );
      }
    }
    if (disposition === "forbidden_cross_channel") {
      assert(typeof operation.method_path === "string", `${operation.id} cross-channel op missing method_path`);
      assert(
        daemonHasRoute(operation.method_path),
        `${operation.id} cross-channel negative ${operation.method_path} is not in daemon source`,
      );
    }
    if (disposition === "client-only") {
      assert(Array.isArray(operation.must_not_invoke), `${operation.id} client-only op must list must_not_invoke`);
    }
  }

  const forbiddenDeclared = new Set(inventory.forbidden_routes.map((row) => row.method_path));
  for (const invented of FORBIDDEN_INVENTED_ROUTES) {
    assert(forbiddenDeclared.has(invented), `forbidden_routes must include ${invented}`);
    const claimed = inventory.operations.find(
      (operation) => operation.method_path === invented && operation.disposition === "available",
    );
    assert(!claimed, `invented route ${invented} must not be available`);
  }

  assert(availableSecretOps.length > 0, "inventory must name the management key route as secret-bearing");
  assert(fileExists("docs/adr/0053-personal-web-ui-stack.md"), "ADR-0053 must exist");
  assert(
    inventory.client_repo === "https://github.com/agentkernel/cognitiveos-clients",
    "client_repo must be the approved cognitiveos-clients GitHub repository",
  );
  assert(inventory.client_path === "pc/web/", "client_path must be pc/web/");
  return { operation_count: inventory.operations.length };
}
