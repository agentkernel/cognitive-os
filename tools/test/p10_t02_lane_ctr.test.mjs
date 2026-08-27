import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const toolsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(toolsDir, "..");

function readRepo(...parts) {
  return readFileSync(path.join(repositoryRoot, ...parts), "utf8");
}

test("P10-T02: Core schemas gain no public MCP family, conversation-projection, or generic Resource contract", () => {
  const schemaDir = path.join(repositoryRoot, "core", "specs", "schemas");
  const names = readdirSync(schemaDir).filter((name) => name.endsWith(".schema.json"));
  const forbidden = names.filter((name) =>
    /mcp-family|conversation-projection|generic-resource|universal-resource/.test(name),
  );
  assert.deepEqual(
    forbidden,
    [],
    `ADR-0058 forbids public Core schemas for MCP family / conversation projection / generic Resource; found ${forbidden.join(", ")}`,
  );
  assert.equal(
    names.includes("conversation-binding.schema.json"),
    true,
    "existing Core ConversationBinding must remain the public conversation identity",
  );
});

test("P10-T02: Core ConversationBinding stays additionalProperties false", () => {
  const schema = JSON.parse(readRepo("core", "specs", "schemas", "conversation-binding.schema.json"));
  assert.equal(
    schema.additionalProperties,
    false,
    "older clients fail closed on unknown ConversationBinding fields",
  );
  assert.equal(schema.title, "ConversationBinding");
  const propertyNames = Object.keys(schema.properties);
  assert.equal(
    propertyNames.some((name) => /transcript|vendor|mcp|history_bytes/.test(name)),
    false,
    "vendor transcript and MCP fields must not land on Core ConversationBinding",
  );
});

test("P10-T02: 1.0 private projection family allowlist stays six-family and rejects mcp", () => {
  const source = readRepo(
    "personal",
    "apps",
    "kernel-server",
    "src",
    "personal",
    "resource_api.rs",
  );
  const allowlist = source.match(
    /const RESOURCE_FAMILIES: \[&str; 6\] = \[([^\]]+)\]/,
  );
  assert.ok(allowlist, "RESOURCE_FAMILIES allowlist of length 6 must exist");
  const families = [...allowlist[1].matchAll(/"([a-z]+)"/g)].map((match) => match[1]);
  assert.deepEqual(families, ["memory", "skill", "tool", "context", "task", "runtime"]);
  assert.equal(families.includes("mcp"), false);
  assert.match(source, /RESOURCE_PROJECTION_FAMILY_INVALID/);
  assert.match(
    source,
    /resource family must be one of memory, skill, tool, context, task, or runtime/,
  );
});

test("P10-T02: ADR-0058 freezes Personal-private envelopes and P5 non-migration", () => {
  const adr = readRepo(
    "docs",
    "adr",
    "0058-personal-2-0-mcp-conversation-private-projection.md",
  );
  assert.match(adr, /cognitiveos\.personal\.mcp-family\/0\.1/);
  assert.match(adr, /cognitiveos\.personal\.conversation-projection\/0\.1/);
  assert.match(adr, /RESOURCE_PROJECTION_FAMILY_INVALID/);
  assert.match(adr, /do not auto-migrate/i);
  assert.match(adr, /No public machine-contract change/i);
  assert.doesNotMatch(
    adr,
    /\$id": "mcp-family/,
    "ADR-0058 must not smuggle a Core schema $id",
  );
});

test("P10-T02: P5 MCP adapter remains transport-only and non-authoritative", () => {
  const adapter = readRepo(
    "personal",
    "crates",
    "cognitive-runtime",
    "src",
    "mcp_tool_adapter.rs",
  );
  assert.match(adapter, /never grant CognitiveOS capability/);
  assert.match(adapter, /Post-1.0 MCP Tool adapter qualification/);
});
