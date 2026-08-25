#!/usr/bin/env node
/**
 * Measurement-only raw HTTPS chat/completions probe (P8-T11).
 *
 * Hits the Provider directly with no dsh, AKP, or daemon proxy. The API key is
 * read from --api-key-file (0600 or "-") and is never logged. Body bytes are
 * counted, not printed. Claim ceiling: hypothesis. No Gate/release/Profile.
 */
import { readFileSync } from "node:fs";
import https from "node:https";
import { stdin as stdinStream } from "node:process";
import { URL } from "node:url";

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index >= 0 && process.argv[index + 1]) return process.argv[index + 1];
  return fallback;
}

const apiKeyFile = arg("--api-key-file");
const model = arg("--model", "deepseek-v4-flash");
const baseUrl = arg("--base-url", "https://api.deepseek.com");
const prompt = arg(
  "--prompt",
  "Reply with one sentence that summarizes this text and nothing else: CognitiveOS Personal is a local-first OS for governed agent work.",
);
const stream = arg("--stream", "true") !== "false";
if (!apiKeyFile) {
  throw new Error("--api-key-file <0600-path|-> is required");
}

async function readKey() {
  if (apiKeyFile === "-") {
    const chunks = [];
    for await (const chunk of stdinStream) chunks.push(chunk);
    return Buffer.concat(chunks).toString("utf8").trim();
  }
  return readFileSync(apiKeyFile, "utf8").trim();
}

const apiKey = await readKey();
if (!apiKey) {
  throw new Error("provider key was empty");
}

const url = new URL("/chat/completions", baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`);
const body = JSON.stringify({
  model,
  stream,
  messages: [{ role: "user", content: prompt }],
  max_tokens: 64,
  temperature: 0,
});

const outcome = await new Promise((resolve) => {
  const started = Date.now();
  let ttfbMs = null;
  let status = 0;
  let bytes = 0;
  const request = https.request(
    {
      hostname: url.hostname,
      port: url.port || 443,
      path: `${url.pathname}${url.search}`,
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
        Accept: stream ? "text/event-stream" : "application/json",
        "Content-Length": Buffer.byteLength(body),
      },
    },
    (response) => {
      status = response.statusCode ?? 0;
      response.on("data", (chunk) => {
        if (ttfbMs === null) ttfbMs = Date.now() - started;
        bytes += chunk.length;
      });
      response.on("end", () => {
        resolve({
          status,
          elapsed_ms: Date.now() - started,
          ttfb_ms: ttfbMs,
          body_bytes: bytes,
        });
      });
    },
  );
  request.on("error", (error) => {
    resolve({
      status: 0,
      elapsed_ms: Date.now() - started,
      ttfb_ms: ttfbMs,
      body_bytes: bytes,
      error_class: error.code || "network",
    });
  });
  request.write(body);
  request.end();
});

const summary = {
  kind: "p8-t11-raw-provider-probe",
  model,
  stream,
  host: url.hostname,
  path: url.pathname,
  status: outcome.status,
  elapsed_ms: outcome.elapsed_ms,
  ttfb_ms: outcome.ttfb_ms,
  body_bytes: outcome.body_bytes,
  error_class: outcome.error_class ?? null,
  key_present: true,
  key_length: apiKey.length,
  non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
};
process.stdout.write(`${JSON.stringify(summary)}\n`);
process.exit(outcome.status === 200 ? 0 : 1);
