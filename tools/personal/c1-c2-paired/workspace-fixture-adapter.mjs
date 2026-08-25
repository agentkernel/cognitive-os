/**
 * P-arm equivalent Workspace* fixture adapter (C1/C2 addendum §1).
 *
 * Advertises the same tool names and parameter keys as the O-arm Extension.
 * Executes only inside a fixture root. Never talks to the CognitiveOS daemon.
 *
 * WorkspacePatch `input_b64` is a UTF-8 unified diff, matching
 * `apply_unified_patch` in `personal/apps/kernel-server/src/personal/tool_executor/mutate.rs`.
 * Replacement bytes are refused. WorkspaceWrite remains replacement bytes.
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

export const WORKSPACE_PATCH_PAYLOAD = "unified-diff";

const SHA256_PREIMAGE = /^digest:sha256:[a-f0-9]{64}$/;
const UTF8 = new TextDecoder("utf-8", { fatal: true });

function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function preimageOf(bytes) {
  return `digest:sha256:${sha256Hex(bytes)}`;
}

function utf8Fatal(bytes, label) {
  try {
    return UTF8.decode(bytes);
  } catch {
    throw new Error(label);
  }
}

function splitTerminator(text) {
  if (text === "") {
    return [];
  }
  const parts = text.split("\n");
  if (text.endsWith("\n")) {
    parts.pop();
  }
  return parts;
}

function splitTextLines(text) {
  if (text === "") {
    return [];
  }
  const lines = [];
  let start = 0;
  for (let index = 0; index < text.length; index += 1) {
    if (text[index] === "\n") {
      lines.push({ text: text.slice(start, index), terminated: true });
      start = index + 1;
    }
  }
  if (start < text.length) {
    lines.push({ text: text.slice(start), terminated: false });
  }
  return lines;
}

function parseRange(range) {
  const [startText, countText] = range.includes(",") ? range.split(",", 2) : [range, "1"];
  const start = Number.parseInt(startText, 10);
  const count = Number.parseInt(countText, 10);
  if (!Number.isInteger(start) || !Number.isInteger(count) || start < 0 || count < 0) {
    throw new Error(`malformed hunk range: ${range}`);
  }
  return [start, count];
}

function parseHunkHeader(header) {
  if (!header.startsWith("@@")) {
    throw new Error(`malformed hunk header: ${header}`);
  }
  const after = header.slice(2);
  const close = after.indexOf("@@");
  if (close < 0) {
    throw new Error(`malformed hunk header: ${header}`);
  }
  const body = after.slice(0, close).trim();
  const ranges = body.split(/\s+/);
  if (ranges.length !== 2) {
    throw new Error(`malformed hunk header: ${header}`);
  }
  const oldRange = ranges[0].startsWith("-") ? ranges[0].slice(1) : null;
  const newRange = ranges[1].startsWith("+") ? ranges[1].slice(1) : null;
  if (oldRange === null || newRange === null) {
    throw new Error(`malformed hunk header: ${header}`);
  }
  const [oldStart, oldCount] = parseRange(oldRange);
  const [, newCount] = parseRange(newRange);
  return [oldStart, oldCount, newCount];
}

function applyNoNewlineMarker(body, preimageLines, postimageLines) {
  if (body.oldIndex !== undefined) {
    const oldLine = preimageLines[body.oldIndex];
    if (oldLine === undefined) {
      throw new Error("no-newline marker references a missing old line");
    }
    if (oldLine.terminated) {
      throw new Error("old-side no-newline marker contradicts the preimage");
    }
  }
  if (body.outputIndex !== undefined) {
    const outputLine = postimageLines[body.outputIndex];
    if (outputLine === undefined) {
      throw new Error("no-newline marker references a missing new line");
    }
    outputLine.terminated = false;
  }
}

function finalizePatchBody(body, preimageLines) {
  if (body === undefined) {
    return;
  }
  if (body.oldIndex !== undefined) {
    const oldLine = preimageLines[body.oldIndex];
    if (oldLine === undefined) {
      throw new Error("hunk references a missing old line");
    }
    if (!oldLine.terminated && !body.noNewlineMarker) {
      throw new Error("unterminated old line is missing its no-newline marker");
    }
  }
  if (body.kind === "addition" && body.noNewlineMarker && body.outputIndex === undefined) {
    throw new Error("new-side no-newline marker has no output line");
  }
}

/**
 * Apply a strict single-file unified diff.
 *
 * Semantics match `apply_unified_patch`: unknown prefixes, overlapping or
 * out-of-order hunks, drifted context, and header/body count mismatches fail
 * closed. Replacement bytes are not a fallback.
 */
export function applyUnifiedPatch(preimage, patch) {
  const preimageLines = splitTextLines(preimage);
  const patchLines = splitTerminator(patch);
  const postimageLines = [];
  let cursor = 0;
  let appliedHunks = 0;
  let patchIndex = 0;

  while (patchIndex < patchLines.length) {
    const patchLine = patchLines[patchIndex];
    if (patchLine.startsWith("--- ") || patchLine.startsWith("+++ ")) {
      patchIndex += 1;
      continue;
    }
    if (patchLine === "") {
      patchIndex += 1;
      continue;
    }
    if (!patchLine.startsWith("@@")) {
      throw new Error(`unexpected line outside a hunk: ${patchLine}`);
    }
    patchIndex += 1;
    const [oldStart, oldCount, newCount] = parseHunkHeader(patchLine);
    const hunkStart = Math.max(oldStart - 1, 0);
    if (hunkStart < cursor) {
      throw new Error("hunks overlap or are out of order");
    }
    if (hunkStart > preimageLines.length) {
      throw new Error("hunk starts past the end of the file");
    }
    const carried = preimageLines.slice(cursor, hunkStart);
    if (carried.length !== hunkStart - cursor) {
      throw new Error("hunk start is outside the file");
    }
    for (const carriedLine of carried) {
      postimageLines.push({ ...carriedLine });
    }
    cursor = hunkStart;

    let consumedOld = 0;
    let producedNew = 0;
    let lastBody;
    while (patchIndex < patchLines.length) {
      const bodyLine = patchLines[patchIndex];
      if (bodyLine.startsWith("@@")) {
        break;
      }
      patchIndex += 1;
      if (bodyLine.startsWith("\\")) {
        if (bodyLine !== "\\ No newline at end of file") {
          throw new Error(`unsupported patch marker: ${bodyLine}`);
        }
        if (lastBody === undefined) {
          throw new Error("no-newline marker does not follow a hunk line");
        }
        if (lastBody.noNewlineMarker) {
          throw new Error("duplicate no-newline marker");
        }
        applyNoNewlineMarker(lastBody, preimageLines, postimageLines);
        lastBody.noNewlineMarker = true;
        continue;
      }
      finalizePatchBody(lastBody, preimageLines);
      lastBody = undefined;
      if (bodyLine === "") {
        throw new Error("bare empty patch line has no unified-diff prefix");
      }
      const marker = bodyLine.slice(0, 1);
      const text = bodyLine.slice(1);
      if (marker === " ") {
        const existingLine = preimageLines[cursor];
        if (existingLine === undefined) {
          throw new Error("context line runs past the end of the file");
        }
        if (existingLine.text !== text) {
          throw new Error("context line does not match the preimage");
        }
        const oldIndex = cursor;
        postimageLines.push({ ...existingLine });
        const outputIndex = postimageLines.length - 1;
        cursor += 1;
        consumedOld += 1;
        producedNew += 1;
        lastBody = { kind: "context", oldIndex, outputIndex, noNewlineMarker: false };
      } else if (marker === "-") {
        const existingLine = preimageLines[cursor];
        if (existingLine === undefined) {
          throw new Error("removed line runs past the end of the file");
        }
        if (existingLine.text !== text) {
          throw new Error("removed line does not match the preimage");
        }
        const oldIndex = cursor;
        cursor += 1;
        consumedOld += 1;
        lastBody = { kind: "removal", oldIndex, outputIndex: undefined, noNewlineMarker: false };
      } else if (marker === "+") {
        postimageLines.push({ text, terminated: true });
        const outputIndex = postimageLines.length - 1;
        producedNew += 1;
        lastBody = { kind: "addition", oldIndex: undefined, outputIndex, noNewlineMarker: false };
      } else {
        throw new Error(`unsupported patch line prefix: ${marker}`);
      }
    }
    finalizePatchBody(lastBody, preimageLines);
    if (consumedOld !== oldCount || producedNew !== newCount) {
      throw new Error(
        `hunk body does not match its header: consumed ${consumedOld}/${oldCount} old and produced ${producedNew}/${newCount} new lines`,
      );
    }
    appliedHunks += 1;
  }

  if (appliedHunks === 0) {
    throw new Error("patch contains no hunk");
  }
  for (const trailingLine of preimageLines.slice(cursor)) {
    postimageLines.push({ ...trailingLine });
  }
  if (postimageLines.slice(0, -1).some((line) => !line.terminated)) {
    throw new Error("no-newline marker appeared before the final output line");
  }
  let postimage = "";
  for (const line of postimageLines) {
    postimage += line.text;
    if (line.terminated) {
      postimage += "\n";
    }
  }
  return postimage;
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
    patch_payload: WORKSPACE_PATCH_PAYLOAD,
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
        const payload = Buffer.from(parameters.input_b64, "base64");
        let next;
        if (toolName === "WorkspacePatch") {
          const currentText = utf8Fatal(
            existed ? current : Buffer.alloc(0),
            "workspace patch preimage is not valid UTF-8",
          );
          const patchText = utf8Fatal(payload, "workspace patch payload is not valid UTF-8");
          next = Buffer.from(applyUnifiedPatch(currentText, patchText), "utf8");
        } else {
          next = payload;
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
