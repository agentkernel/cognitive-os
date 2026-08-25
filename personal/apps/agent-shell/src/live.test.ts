import assert from "node:assert/strict";
import { test } from "node:test";

import { createLiveShellSession } from "./live.js";

test("createLiveShellSession binds the task channel only", () => {
  const session = createLiveShellSession({
    baseUrl: "http://127.0.0.1:9",
    bearer: "task-secret",
    context: {
      sender: "principal://tenant-a/user-1",
      audience: "kernel://tenant-a/node-1",
    },
  });
  assert.equal(session.phase, "idle");
});
