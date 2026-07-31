/** Complete pinned-Pi provider whose only model transport is the local daemon. */

import { PersonalDaemonClient } from "./daemon-client.js";
import type {
  AssistantMessageEventStream,
  PiAssistantMessage,
  PiAssistantMessageEvent,
  PiCompletionContext,
  PiModel,
  PiStreamOptions,
  PiTextContent,
  ProviderConfig,
} from "./pi-api.js";

const PROVIDER_ID = "cognitiveos";
const PROVIDER_API = "openai-completions";
const PI_AVAILABILITY_MARKER = "cognitiveos-local-daemon";

/** Load one daemon-selected model and configure Pi's custom stream transport. */
export async function createDaemonProvider(client: PersonalDaemonClient): Promise<ProviderConfig> {
  const projection = await client.fetchSelectedModel();
  const loopbackBaseUrl = `http://${client.readLoopbackEndpoint()}/provider/v1`;
  const model: PiModel = {
    id: projection.selectedModel,
    name: projection.selectedModel,
    provider: PROVIDER_ID,
    api: PROVIDER_API,
    baseUrl: loopbackBaseUrl,
    reasoning: false,
    input: ["text"],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 8_192,
    maxTokens: 1_024,
  };
  return {
    name: "CognitiveOS",
    baseUrl: loopbackBaseUrl,
    apiKey: PI_AVAILABILITY_MARKER,
    api: PROVIDER_API,
    models: [model],
    streamSimple: (requestedModel, context, options) => streamCompletion(client, requestedModel, context, options),
  };
}

function streamCompletion(
  client: PersonalDaemonClient,
  model: PiModel,
  context: PiCompletionContext,
  options?: PiStreamOptions,
): AssistantMessageEventStream {
  const stream = new LocalAssistantMessageEventStream();
  void dispatchCompletion(client, stream, model, context, options?.signal);
  return stream;
}

async function dispatchCompletion(
  client: PersonalDaemonClient,
  stream: LocalAssistantMessageEventStream,
  model: PiModel,
  context: PiCompletionContext,
  signal?: AbortSignal,
): Promise<void> {
  const timestamp = Date.now();
  if (signal?.aborted) return endFailure(stream, model, timestamp, "aborted", "completion cancelled before dispatch");
  const partial = assistantMessage(model, [], "stop", timestamp);
  stream.push({ type: "start", partial });
  try {
    const completion = await client.completeChat(model.id, toDaemonMessages(context), signal);
    if (signal?.aborted) return endFailure(stream, model, timestamp, "aborted", "completion cancelled while waiting");
    const content: PiTextContent = { type: "text", text: completion.content };
    stream.push({ type: "text_start", contentIndex: 0, partial: content });
    stream.push({ type: "text_delta", contentIndex: 0, delta: content.text });
    stream.push({ type: "text_end", contentIndex: 0, content });
    const message = assistantMessage(model, [content], "stop", timestamp);
    stream.push({ type: "done", message });
    stream.end(message);
  } catch (error) {
    endFailure(stream, model, timestamp, signal?.aborted ? "aborted" : "error", safeErrorMessage(error));
  }
}

function toDaemonMessages(context: PiCompletionContext): readonly { role: "system" | "user" | "assistant"; content: string }[] {
  const messages: { role: "system" | "user" | "assistant"; content: string }[] = [];
  if (context.systemPrompt !== undefined && context.systemPrompt.length > 0) messages.push({ role: "system", content: context.systemPrompt });
  for (const rawMessage of context.messages) {
    if (typeof rawMessage !== "object" || rawMessage === null) throw new Error("unsupported Pi message");
    const message = rawMessage as Record<string, unknown>;
    const role = message["role"];
    const content = extractText(message["content"]);
    if ((role !== "user" && role !== "assistant") || content === undefined) throw new Error("unsupported Pi message");
    messages.push({ role, content });
  }
  return messages;
}

function extractText(content: unknown): string | undefined {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return undefined;
  const textParts: string[] = [];
  for (const block of content) {
    if (typeof block !== "object" || block === null) return undefined;
    const record = block as Record<string, unknown>;
    if (record["type"] !== "text" || typeof record["text"] !== "string") return undefined;
    textParts.push(record["text"]);
  }
  return textParts.join("\n");
}

function endFailure(stream: LocalAssistantMessageEventStream, model: PiModel, timestamp: number, stopReason: "error" | "aborted", errorMessage: string): void {
  const message = assistantMessage(model, [], stopReason, timestamp, errorMessage);
  stream.push({ type: "error", error: message });
  stream.end(message);
}

function assistantMessage(model: PiModel, content: readonly PiTextContent[], stopReason: PiAssistantMessage["stopReason"], timestamp: number, errorMessage?: string): PiAssistantMessage {
  return {
    role: "assistant", content, api: model.api, provider: model.provider, model: model.id,
    usage: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0, cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } },
    stopReason, timestamp, ...(errorMessage === undefined ? {} : { errorMessage }),
  };
}

function safeErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "daemon completion failed";
}

class LocalAssistantMessageEventStream implements AssistantMessageEventStream {
  private readonly events: PiAssistantMessageEvent[] = [];
  private completed = false;
  private finalMessage: PiAssistantMessage | undefined;
  private resolveResult: ((message: PiAssistantMessage) => void) | undefined;
  private readonly finalResult = new Promise<PiAssistantMessage>((resolve) => { this.resolveResult = resolve; });

  push(event: PiAssistantMessageEvent): void { if (!this.completed) this.events.push(event); }
  end(result?: PiAssistantMessage): void { if (!this.completed) { this.completed = true; this.finalMessage = result; if (result !== undefined) this.resolveResult?.(result); } }
  result(): Promise<PiAssistantMessage> { return this.finalResult; }
  async *[Symbol.asyncIterator](): AsyncIterator<PiAssistantMessageEvent> {
    let eventIndex = 0;
    while (!this.completed || eventIndex < this.events.length) {
      if (eventIndex < this.events.length) yield this.events[eventIndex++]!;
      else await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }
}
