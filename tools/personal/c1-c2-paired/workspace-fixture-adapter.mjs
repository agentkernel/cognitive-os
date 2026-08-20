/**
 * P-arm equivalent Workspace* fixture adapter (C1/C2 addendum §1).
 *
 * Advertises the same tool names and parameter keys as the O-arm Extension.
 * Executes only inside a fixture root. Never talks to the CognitiveOS daemon.
 */

import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

export const WORKSPACE_TOOL_SCHEMAS = Object.freeze([
  { name: "WorkspaceRead", parameters: Object.freeze(["target"]) },
  { name: "WorkspaceSearch", parameters: Object.freeze(["query", "target"]) },
  { name: "WorkspaceWrite", parameters: Object.freeze(["input_b64", "preimage", "target"]) },
  { name: "WorkspacePatch", parameters: Object.freeze(["input_b64", "preimage", "target"]) },
]);

const SHA256_PREIMAGE = /^digest:sha256:[a-f0-9]{64}$/;

function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function preimageOf(bytes) {
  return `digest:sha256:${sha256Hex(bytes)}`;
}

export function createWorkspaceFixtureAdapter({ root }) {
  if (typeof root !== "string" || root.trim() === "") {
    throw new Error("fixture root is required");
  }
  const fixtureRoot = path.resolve(root);

  const resolveContained = (target) => {
    if (typeof target !== "string" || target.trim() === "") {
      throw new Error("workspace target is required");
    }
    if (path.isAbsolute(target) || target.includes("\0")) {
      throw new Error("workspace target refused");
    }
    const resolved = path.resolve(fixtureRoot, target);
    const relative = path.relative(fixtureRoot, resolved);
    if (relative.startsWith("..") || path.isAbsolute(relative)) {
      throw new Error("workspace target escaped fixture root");
    }
    return resolved;
  };

  return Object.freeze({
    kind: "pure-pi-workspace-fixture",
    schemas: WORKSPACE_TOOL_SCHEMAS,
    daemon: false,
    execute(toolName, parameters = {}) {
      const schema = WORKSPACE_TOOL_SCHEMAS.find((entry) => entry.name === toolName);
      if (schema === undefined) {
        throw new Error(`unknown workspace tool: ${toolName}`);
      }
      for (const key of Object.keys(parameters)) {
        if (!schema.parameters.includes(key)) {
          throw new Error(`unknown field ${key} for ${toolName}`);
        }
      }
      const target = resolveContained(parameters.target);

      if (toolName === "WorkspaceRead") {
        const bytes = readFileSync(target);
        return {
          family: "WorkspaceRead",
          target: parameters.target,
          bytes: Buffer.from(bytes),
          preimage: preimageOf(bytes),
        };
      }

      if (toolName === "WorkspaceSearch") {
        if (typeof parameters.query !== "string") {
          throw new Error("WorkspaceSearch query is required");
        }
        const text = readFileSync(target, "utf8");
        const hits = [];
        for (const [index, line] of text.split(/\r?\n/).entries()) {
          if (line.includes(parameters.query)) {
            hits.push({ line: index + 1, text: line });
          }
        }
        return { family: "WorkspaceSearch", target: parameters.target, hits };
      }

      if (toolName === "WorkspaceWrite" || toolName === "WorkspacePatch") {
        if (typeof parameters.input_b64 !== "string" || typeof parameters.preimage !== "string") {
          throw new Error(`${toolName} requires input_b64 and preimage`);
        }
        let existed = false;
        let current;
        try {
          current = readFileSync(target);
          existed = true;
        } catch (error) {
          if (error?.code !== "ENOENT") {
            throw error;
          }
        }
        const expected =
          parameters.preimage === "absent"
            ? "absent"
            : SHA256_PREIMAGE.test(parameters.preimage)
              ? parameters.preimage
              : null;
        if (expected === null) {
          throw new Error("invalid preimage");
        }
        const actual = existed ? preimageOf(current) : "absent";
        if (actual !== expected) {
          throw new Error("preimage mismatch");
        }
        const next = Buffer.from(parameters.input_b64, "base64");
        if (toolName === "WorkspacePatch") {
          // Fixture patch payload is the replacement bytes, not a daemon unified diff.
          // B0 freeze will pin the C2a corpus format separately.
        }
        mkdirSync(path.dirname(target), { recursive: true });
        writeFileSync(target, next);
        return {
          family: toolName,
          target: parameters.target,
          preimage: preimageOf(next),
          size: next.length,
        };
      }

      throw new Error(`unhandled tool ${toolName}`);
    },
    stat(target) {
      return statSync(resolveContained(target));
    },
  });
}
