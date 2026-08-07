// Daemon-private Pi provider for one candidate completion. This extension owns
// no credential: it forwards one non-streaming request over a daemon-created
// Unix-domain socket and receives only the Provider response body.

import net from "node:net";

const providerId = "cognitiveos-private-candidate";
const maximumRequestBytes = 256 * 1024;
const maximumResponseBytes = 256 * 1024;
const socketPath = process.env.COGNITIVEOS_PRIVATE_COMPLETION_SOCKET;
const selectedModel = process.env.COGNITIVEOS_PRIVATE_COMPLETION_MODEL;

if (!socketPath || !selectedModel) {
  throw new Error("daemon private completion socket is not configured");
}

export default function registerPrivateCandidateProvider(pi) {
  const model = {
    id: selectedModel,
    name: selectedModel,
    provider: providerId,
    api: "openai-completions",
    baseUrl: "http://cognitiveos-private-completion.invalid/v1",
    reasoning: false,
    input: ["text"],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 8192,
    maxTokens: 1024,
  };
  pi.registerProvider(providerId, {
    name: "CognitiveOS private candidate",
    baseUrl: model.baseUrl,
    apiKey: "daemon-private-socket",
    api: "openai-completions",
    models: [model],
    streamSimple: (requestedModel, context) => streamCandidateCompletion(requestedModel, context),
  });
}

function streamCandidateCompletion(model, context) {
  const stream = new CandidateMessageStream();
  void completeCandidate(model, context, stream);
  return stream;
}

async function completeCandidate(model, context, stream) {
  const timestamp = Date.now();
  const partial = assistantMessage(model, [], "stop", timestamp);
  stream.push({ type: "start", partial });
  try {
    if (!model || model.id !== selectedModel) {
      throw new Error("daemon private completion model was not selected");
    }
    const response = await requestDaemonCompletion({
      model: model.id,
      stream: false,
      messages: toDaemonMessages(context),
    });
    const choice = response?.choices?.[0];
    const content = choice?.message?.content;
    if (
      !Array.isArray(response?.choices)
      || response.choices.length !== 1
      || typeof content !== "string"
      || choice?.message?.tool_calls !== undefined
    ) throw new Error("daemon completion response is malformed");
    const text = { type: "text", text: content };
    stream.push({ type: "text_start", contentIndex: 0, partial: text });
    stream.push({ type: "text_delta", contentIndex: 0, delta: text.text });
    stream.push({ type: "text_end", contentIndex: 0, content: text });
    const message = assistantMessage(model, [text], "stop", timestamp);
    stream.push({ type: "done", message });
    stream.end(message);
  } catch (_error) {
    const message = assistantMessage(model, [], "error", timestamp, "daemon private completion failed");
    stream.push({ type: "error", error: message });
    stream.end(message);
  }
}

function toDaemonMessages(context) {
  const messages = [];
  if (typeof context.systemPrompt === "string" && context.systemPrompt.length > 0) {
    messages.push({ role: "system", content: context.systemPrompt });
  }
  for (const message of context.messages) {
    if (!message || (message.role !== "user" && message.role !== "assistant")) {
      throw new Error("Pi candidate context contains an unsupported message");
    }
    const content = textContent(message.content);
    if (content === undefined) throw new Error("Pi candidate context contains non-text content");
    messages.push({ role: message.role, content });
  }
  return messages;
}

function textContent(content) {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return undefined;
  const parts = [];
  for (const block of content) {
    if (!block || block.type !== "text" || typeof block.text !== "string") return undefined;
    parts.push(block.text);
  }
  return parts.join("\n");
}

function requestDaemonCompletion(request) {
  const body = Buffer.from(JSON.stringify(request));
  if (body.length === 0 || body.length > maximumRequestBytes) {
    return Promise.reject(new Error("daemon private completion request is out of bounds"));
  }
  return new Promise((resolve, reject) => {
    const socket = net.createConnection(socketPath);
    const chunks = [];
    let responseLength = 0;
    let settled = false;
    const fail = () => {
      if (!settled) {
        settled = true;
        reject(new Error("daemon private completion was refused"));
      }
    };
    socket.setTimeout(60000);
    socket.once("connect", () => {
      socket.write(
        `POST /chat/completions HTTP/1.1\r\nHost: private\r\nContent-Type: application/json\r\nContent-Length: ${body.length}\r\nConnection: close\r\n\r\n`,
      );
      socket.write(body);
    });
    socket.on("data", (chunk) => {
      responseLength += chunk.length;
      if (responseLength > maximumResponseBytes) {
        socket.destroy(new Error("daemon private completion response is out of bounds"));
        return;
      }
      chunks.push(chunk);
    });
    socket.once("timeout", () => socket.destroy(new Error("daemon private completion timed out")));
    socket.once("error", fail);
    socket.once("close", () => {
      if (settled) return;
      const response = Buffer.concat(chunks);
      const separator = response.indexOf("\r\n\r\n");
      if (separator < 0 || !response.subarray(0, separator).toString("ascii").startsWith("HTTP/1.1 200")) {
        fail();
        return;
      }
      try {
        settled = true;
        resolve(JSON.parse(response.subarray(separator + 4).toString("utf8")));
      } catch {
        fail();
      }
    });
  });
}

function assistantMessage(model, content, stopReason, timestamp, errorMessage) {
  return {
    role: "assistant", content, api: model.api, provider: model.provider, model: model.id,
    usage: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0, cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } },
    stopReason, timestamp, ...(errorMessage ? { errorMessage } : {}),
  };
}

class CandidateMessageStream {
  #events = [];
  #complete = false;
  #resolve;
  #result = new Promise((resolve) => { this.#resolve = resolve; });
  push(event) { if (!this.#complete) this.#events.push(event); }
  end(message) { if (!this.#complete) { this.#complete = true; this.#resolve(message); } }
  result() { return this.#result; }
  async *[Symbol.asyncIterator]() {
    let index = 0;
    while (!this.#complete || index < this.#events.length) {
      if (index < this.#events.length) yield this.#events[index++];
      else await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }
}
