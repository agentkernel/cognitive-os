import assert from "node:assert/strict";
import { test } from "node:test";

import { CLIENT_CHANNELS, SDK_ENCODING_PROFILE } from "./index.js";

test("sdk exposes the two isolated channels and the shared encoding profile", () => {
  assert.deepEqual([...CLIENT_CHANNELS], ["task", "management"]);
  assert.equal(SDK_ENCODING_PROFILE, "cognitiveos.canonical-json/0.1");
});
