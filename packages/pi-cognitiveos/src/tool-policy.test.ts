/**
 * Tool policy tests (P1-T07 acceptance: direct bash/write/edit disabled).
 *
 * The policy is default-deny, so the interesting assertions are the negatives:
 * nothing executes ungoverned, and a renamed or padded tool name cannot slip
 * past the denylist.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  BLOCKED_MUTATING_TOOLS,
  MUTATING_TOOL_BLOCK_REASON,
  READ_ONLY_TOOL_ALLOWLIST,
  UNGOVERNED_TOOL_BLOCK_REASON,
  decideToolCall,
  isBlockedMutatingTool,
} from "./tool-policy.js";

test("the three Pi mutating built-ins are blocked with a mutating-tool reason", () => {
  for (const toolName of ["bash", "edit", "write"]) {
    const decision = decideToolCall({ toolName });
    assert.ok(decision, `${toolName} must be blocked`);
    assert.equal(decision.block, true);
    assert.equal(decision.reason, MUTATING_TOOL_BLOCK_REASON);
    assert.ok(isBlockedMutatingTool(toolName));
  }
  assert.deepEqual([...BLOCKED_MUTATING_TOOLS].sort(), ["bash", "edit", "write"]);
});

test("case and whitespace variants cannot evade the mutating denylist", () => {
  for (const toolName of ["BASH", " Bash ", "Write", "\tEDIT\n"]) {
    const decision = decideToolCall({ toolName });
    assert.ok(decision);
    assert.equal(decision.reason, MUTATING_TOOL_BLOCK_REASON);
  }
});

test("unknown tools are blocked too: a non-authority client authorizes nothing", () => {
  for (const toolName of ["read", "glob", "grep", "some_future_tool", ""]) {
    const decision = decideToolCall({ toolName });
    assert.ok(decision, `${toolName} must be blocked while the allowlist is empty`);
    assert.equal(decision.block, true);
    assert.equal(decision.reason, UNGOVERNED_TOOL_BLOCK_REASON);
  }
});

test("the read-only allowlist is empty, so no tool call is ever permitted", () => {
  assert.deepEqual([...READ_ONLY_TOOL_ALLOWLIST], []);
  const sampled = ["bash", "edit", "write", "read", "ls", "todo", "mcp__anything"];
  for (const toolName of sampled) {
    assert.notEqual(decideToolCall({ toolName }), undefined);
  }
});

test("block reasons name the governed path instead of just refusing", () => {
  assert.match(MUTATING_TOOL_BLOCK_REASON, /Intent\/Effect/);
  assert.match(UNGOVERNED_TOOL_BLOCK_REASON, /non-authority/);
});
