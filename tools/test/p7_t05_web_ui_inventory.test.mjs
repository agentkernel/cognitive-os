import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  FORBIDDEN_INVENTED_ROUTES,
  INVENTORY_SCHEMA,
  validateWebUiRouteInventory,
} from "../src/p7_t05_web_ui_inventory.mjs";

const toolsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(toolsDir, "..");
const inventoryPath = path.join(
  repositoryRoot,
  "docs",
  "architecture",
  "personal",
  "web-ui-route-inventory.json",
);

function loadInventory() {
  return JSON.parse(readFileSync(inventoryPath, "utf8"));
}

test("P7-T05 inventory refuses invented generic lifecycle and completion routes", () => {
  const inventory = loadInventory();
  const forged = structuredClone(inventory);
  forged.operations.push({
    id: "browser-forged-lifecycle",
    ui_capability: "generic Agent transition",
    method_path: "POST /management/agent/transition",
    channel: "management",
    disposition: "available",
  });
  assert.throws(
    () => validateWebUiRouteInventory(forged, { repositoryRoot }),
    /must not claim invented route/,
  );
});

test("P7-T05 inventory refuses a route that is not in daemon source", () => {
  const inventory = loadInventory();
  const forged = structuredClone(inventory);
  forged.operations.push({
    id: "invented-probe",
    ui_capability: "direct Provider probe",
    method_path: "POST /management/providers/probe-now",
    channel: "management",
    disposition: "available",
  });
  assert.throws(
    () => validateWebUiRouteInventory(forged, { repositoryRoot }),
    /not present in daemon source/,
  );
});

test("P7-T05 inventory refuses Task-channel secret-bearing Provider key writes", () => {
  const inventory = loadInventory();
  const forged = structuredClone(inventory);
  const keyOp = forged.operations.find((row) => row.id === "provider-account-key");
  assert.ok(keyOp);
  keyOp.channel = "task";
  assert.throws(
    () => validateWebUiRouteInventory(forged, { repositoryRoot }),
    /secret-bearing route must stay on the management channel/,
  );
});

test("P7-T05 inventory refuses session persistence in Web storage", () => {
  const inventory = loadInventory();
  const forged = structuredClone(inventory);
  forged.session_storage = "localStorage";
  assert.throws(
    () => validateWebUiRouteInventory(forged, { repositoryRoot }),
    /session_storage must be memory-only/,
  );
});

test("P7-T05 inventory refuses browser SQLite/SecretStore/Provider-direct targets", () => {
  const inventory = loadInventory();
  const forged = structuredClone(inventory);
  forged.browser_forbidden_targets = ["sqlite"];
  assert.throws(
    () => validateWebUiRouteInventory(forged, { repositoryRoot }),
    /must forbid sqlite/,
  );
});

test("P7-T05 inventory marks missing typed lifecycle HTTP as unavailable not-run", () => {
  const inventory = loadInventory();
  const cancel = inventory.operations.find((row) => row.id === "task-cancel");
  const pause = inventory.operations.find((row) => row.id === "agent-pause");
  assert.equal(cancel.disposition, "unavailable");
  assert.equal(cancel.ui_render, "not-run");
  assert.equal(pause.disposition, "unavailable");
  assert.equal(pause.ui_render, "not-run");
});

test("P7-T05 inventory keeps detach observation from cancelling durable work", () => {
  const inventory = loadInventory();
  const detach = inventory.operations.find((row) => row.id === "detach-observation");
  assert.equal(detach.disposition, "client-only");
  assert.ok(detach.must_not_invoke.includes("task-cancel"));
  assert.ok(detach.must_not_invoke.includes("agent-stop"));
});

test("canonical P7-T05 inventory matches daemon source and ADR-0053", () => {
  const inventory = loadInventory();
  assert.equal(inventory.schema, INVENTORY_SCHEMA);
  const result = validateWebUiRouteInventory(inventory, { repositoryRoot });
  assert.ok(result.operation_count >= 20);
  for (const invented of FORBIDDEN_INVENTED_ROUTES) {
    assert.ok(inventory.forbidden_routes.some((row) => row.method_path === invented));
  }
});
