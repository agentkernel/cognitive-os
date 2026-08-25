import test from "node:test";
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));

function listen(server) {
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      resolve(`http://127.0.0.1:${address.port}`);
    });
  });
}

function startBridge(upstream) {
  const child = spawn(process.execPath, [join(here, "provider-sse-bridge.mjs"), "--listen", "127.0.0.1:0", "--upstream", upstream], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  return new Promise((resolve, reject) => {
    const onData = (chunk) => {
      const line = chunk.toString("utf8");
      const match = line.match(/listening 127\.0\.0\.1:(\d+)/);
      if (match) {
        child.stdout.off("data", onData);
        resolve({ child, origin: `http://127.0.0.1:${match[1]}` });
      }
    };
    child.stdout.on("data", onData);
    child.on("exit", (code) => reject(new Error(`bridge exited ${code}`)));
    setTimeout(() => reject(new Error("bridge listen timeout")), 5000);
  });
}

test("SSE bridge converts unary provider JSON into a [DONE]-terminated stream", async () => {
  const upstream = createServer((req, res) => {
    const chunks = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", () => {
      const body = JSON.parse(Buffer.concat(chunks).toString("utf8"));
      assert.equal(body.stream, false);
      assert.equal(body.stream_options, undefined);
      assert.equal(req.headers.authorization, "Bearer test-token");
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify({
        choices: [{ message: { role: "assistant", content: "pong" }, finish_reason: "stop" }],
      }));
    });
  });
  const upstreamOrigin = await listen(upstream);
  const bridge = await startBridge(`${upstreamOrigin}/provider/v1`);
  try {
    const response = await fetch(`${bridge.origin}/provider/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json", authorization: "Bearer test-token" },
      body: JSON.stringify({ model: "deepseek-v4-flash", stream: true, stream_options: { include_usage: true }, messages: [] }),
    });
    assert.equal(response.status, 200);
    assert.match(response.headers.get("content-type") ?? "", /text\/event-stream/);
    const text = await response.text();
    assert.match(text, /pong/);
    assert.match(text, /data: \[DONE\]/);
    assert.doesNotMatch(text, /sk-/);
  } finally {
    bridge.child.kill("SIGTERM");
    upstream.close();
  }
});
