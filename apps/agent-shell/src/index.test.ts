import assert from "node:assert/strict";
import { test } from "node:test";

import { SHELL_CHANNEL, SHELL_VERBS } from "./index.js";

test("shell binds the task channel and exposes non-authority verbs", () => {
  assert.equal(SHELL_CHANNEL, "task");
  assert.ok(SHELL_VERBS.includes("cancel"));
  assert.ok(!("acceptTask" in SHELL_VERBS), "the Shell must never accept/complete a Task itself");
});
