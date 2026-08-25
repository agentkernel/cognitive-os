#!/usr/bin/env node
/**
 * Loopback SSE adapter in front of the Personal unary Provider proxy (P8-T09).
 *
 * dsh-llm-deepseek always sends `stream: true`. The daemon proxy rejects
 * streaming before SecretStore resolution. This process accepts the SSE
 * request on loopback, forwards a non-streaming body to
 * POST /provider/v1/chat/completions, and wraps the unary JSON as one SSE
 * content delta plus [DONE]. It never logs Authorization, bodies, or keys.
 *
 * Argv: --listen 127.0.0.1:PORT --upstream http://127.0.0.1:DAEMON/provider/v1
 */
import { createServer } from "node:http";

function arg(name) {
  const index = process.argv.indexOf(name);
  if (index >= 0 && process.argv[index + 1]) return process.argv[index + 1];
  return undefined;
}

const listen = arg("--listen") ?? "127.0.0.1:0";
const upstream = arg("--upstream");
if (!upstream) {
  throw new Error("--upstream is required");
}

const [listenHost, listenPortText] = listen.split(":");
const listenPort = Number(listenPortText);

function sseWrap(unary) {
  const message = unary?.choices?.[0]?.message ?? {};
  const content = typeof message.content === "string" ? message.content : "";
  const finish = unary?.choices?.[0]?.finish_reason ?? "stop";
  const usage = unary?.usage ?? null;
  const contentEvent = JSON.stringify({
    choices: [{ index: 0, delta: { role: "assistant", content } }],
  });
  const finishEvent = JSON.stringify({
    choices: [{ index: 0, delta: {}, finish_reason: finish }],
    ...(usage ? { usage } : {}),
  });
  return `data: ${contentEvent}\n\ndata: ${finishEvent}\n\ndata: [DONE]\n\n`;
}

const server = createServer(async (req, res) => {
  try {
    if (req.method !== "POST" || !req.url?.endsWith("/chat/completions")) {
      res.writeHead(404, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: { message: "not found", code: "NOT_FOUND" } }));
      return;
    }
    const chunks = [];
    for await (const chunk of req) chunks.push(chunk);
    const raw = Buffer.concat(chunks);
    let parsed;
    try {
      parsed = JSON.parse(raw.toString("utf8"));
    } catch {
      res.writeHead(400, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: { message: "invalid json", code: "INVALID_REQUEST" } }));
      return;
    }
    const wantedStream = parsed.stream === true;
    const forward = { ...parsed, stream: false };
    delete forward.stream_options;
    const authorization = req.headers.authorization ?? "";
    const upstreamResponse = await fetch(`${upstream.replace(/\/$/, "")}/chat/completions`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(authorization ? { authorization } : {}),
      },
      body: JSON.stringify(forward),
    });
    const upstreamText = await upstreamResponse.text();
    if (!upstreamResponse.ok) {
      res.writeHead(upstreamResponse.status, { "content-type": "application/json" });
      res.end(upstreamText);
      return;
    }
    if (!wantedStream) {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(upstreamText);
      return;
    }
    let unary;
    try {
      unary = JSON.parse(upstreamText);
    } catch {
      res.writeHead(502, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: { message: "upstream was not JSON", code: "UPSTREAM" } }));
      return;
    }
    res.writeHead(200, {
      "content-type": "text/event-stream",
      "cache-control": "no-cache",
      connection: "close",
    });
    res.end(sseWrap(unary));
  } catch {
    if (!res.headersSent) {
      res.writeHead(502, { "content-type": "application/json" });
    }
    res.end(JSON.stringify({ error: { message: "bridge failed", code: "BRIDGE" } }));
  }
});

server.listen(listenPort, listenHost, () => {
  const address = server.address();
  if (address && typeof address === "object") {
    process.stdout.write(`listening ${address.address}:${address.port}\n`);
  }
});
