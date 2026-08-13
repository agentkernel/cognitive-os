/** Complete pinned-Pi provider whose only model transport is the local daemon. */

import { PersonalDaemonClient } from "./daemon-client.js";
import type { BoundedCompletion, ProviderUsage } from "./daemon-client.js";
import {
  assemblePiRouteObservation,
  type PiRouteObservationSession,
  type PiRouteStageRecorder,
} from "./pi-route-observation.js";
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

export interface DaemonProviderOptions {
  /**
   * Authorized campaign measurement session. Absent — the default — means the
   * route is not instrumented at all: no correlation id is minted for
   * measurement, no stage is timed and nothing is published.
   */
  readonly session?: PiRouteObservationSession;
}

/** Load one daemon-selected model and configure Pi's custom stream transport. */
export async function createDaemonProvider(
  client: PersonalDaemonClient,
  options: DaemonProviderOptions = {},
): Promise<ProviderConfig> {
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
    streamSimple: (requestedModel, context, streamOptions) =>
      streamCompletion(client, requestedModel, context, streamOptions, options.session),
  };
}

function streamCompletion(
  client: PersonalDaemonClient,
  model: PiModel,
  context: PiCompletionContext,
  options?: PiStreamOptions,
  session?: PiRouteObservationSession,
): AssistantMessageEventStream {
  const stream = new LocalAssistantMessageEventStream();
  void dispatchCompletion(client, stream, model, context, options?.signal, session);
  return stream;
}

async function dispatchCompletion(
  client: PersonalDaemonClient,
  stream: LocalAssistantMessageEventStream,
  model: PiModel,
  context: PiCompletionContext,
  signal?: AbortSignal,
  session?: PiRouteObservationSession,
): Promise<void> {
  const timestamp = Date.now();
  if (signal?.aborted) return endFailure(stream, model, timestamp, "aborted", "completion cancelled before dispatch");
  const recorder = session?.openRequest();
  const partial = assistantMessage(model, [], "stop", timestamp, { availability: "not_available" });
  stream.push({ type: "start", partial });
  try {
    recorder?.begin("pi_request_preparation");
    let daemonMessages: readonly { role: "system" | "user" | "assistant"; content: string }[];
    try {
      daemonMessages = toDaemonMessages(context);
    } finally {
      recorder?.complete("pi_request_preparation");
    }
    const completion = await client.completeChat(model.id, daemonMessages, signal, recorder);
    if (signal?.aborted) return endFailure(stream, model, timestamp, "aborted", "completion cancelled while waiting");
    recorder?.begin("pi_event_delivery");
    const content: PiTextContent = { type: "text", text: completion.content };
    stream.push({ type: "text_start", contentIndex: 0, partial: content });
    stream.push({ type: "text_delta", contentIndex: 0, delta: content.text });
    stream.push({ type: "text_end", contentIndex: 0, content });
    const message = assistantMessage(model, [content], "stop", timestamp, completion.providerUsage);
    stream.push({ type: "done", message });
    stream.end(message);
    recorder?.complete("pi_event_delivery");
    publishObservation(session, recorder, completion);
  } catch (error) {
    endFailure(stream, model, timestamp, signal?.aborted ? "aborted" : "error", safeErrorMessage(error));
  }
}

/**
 * Publish one observation for a completed run.
 *
 * Measurement must never change what Pi returns, so a refusal from the
 * observation surface is contained here: the completion has already been
 * delivered, and a partial or invalid sample is dropped rather than retained.
 */
function publishObservation(
  session: PiRouteObservationSession | undefined,
  recorder: PiRouteStageRecorder | undefined,
  completion: BoundedCompletion,
): void {
  if (session === undefined || recorder === undefined) return;
  try {
    session.publish(
      assemblePiRouteObservation({
        campaignId: session.campaignId,
        correlationId: recorder.readCorrelationId(),
        piStages: recorder.readPiStages(),
        daemonReported: completion.daemonReported,
        providerUsage: completion.providerUsage,
      }),
    );
  } catch {
    // A sample that cannot satisfy the observation rules is not a sample.
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
  const message = assistantMessage(model, [], stopReason, timestamp, { availability: "not_available" }, errorMessage);
  stream.push({ type: "error", error: message });
  stream.end(message);
}

function assistantMessage(
  model: PiModel,
  content: readonly PiTextContent[],
  stopReason: PiAssistantMessage["stopReason"],
  timestamp: number,
  providerUsage: ProviderUsage,
  errorMessage?: string,
): PiAssistantMessage {
  return {
    role: "assistant", content, api: model.api, provider: model.provider, model: model.id,
    usage: toPiUsage(providerUsage),
    stopReason, timestamp, ...(errorMessage === undefined ? {} : { errorMessage }),
  };
}

function toPiUsage(providerUsage: ProviderUsage): PiAssistantMessage["usage"] {
  const unavailableCost = { input: undefined, output: undefined, cacheRead: undefined, cacheWrite: undefined, total: undefined };
  if (providerUsage.availability === "not_available") {
    return { input: undefined, output: undefined, cacheRead: undefined, cacheWrite: undefined, totalTokens: undefined, cost: unavailableCost };
  }
  return {
    input: providerUsage.promptTokens,
    output: providerUsage.completionTokens,
    cacheRead: undefined,
    cacheWrite: undefined,
    totalTokens: providerUsage.totalTokens,
    cost: unavailableCost,
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
