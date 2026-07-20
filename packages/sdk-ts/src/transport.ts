/**
 * Injectable AKP transport boundary (ADR-0003 HTTP JSON + SSE mapping;
 * docs/standards/akp-envelope-and-http-profile.md §3/§4).
 *
 * The transport moves envelope text; it never interprets outcomes. HTTP
 * status codes are transport-level only — the authoritative outcome is the
 * enveloped AKP result (implementations MUST NOT infer effect success from
 * a 2xx transport response). Channels bind to disjoint endpoint roots and
 * disjoint session material.
 *
 * `InMemoryTransport` is the scriptable fake used by unit tests and by the
 * pre-M5 Shell; `HttpSseTransport` is a thin default HTTP binding whose
 * exact routes are provisional until kernel-server lands at M5.
 */

import type { ClientChannel } from "./channel.js";
import { parseRequestEnvelope, serializeEnvelope, type RequestEnvelope, type ResultEnvelope } from "./envelope.js";

/** Reply from one request/response exchange. */
export interface TransportReply {
  /** Transport-level status; NEVER an outcome (REQ-GW-002 analog). */
  readonly transportStatus: number;
  /** Result envelope JSON text, parsed by the envelope layer. */
  readonly body: string;
}

/** Transport boundary: one instance is bound to exactly one channel. */
export interface AkpTransport {
  readonly channel: ClientChannel;
  request(envelopeText: string): Promise<TransportReply>;
  /** Open a watch/subscription stream; yields raw frame JSON texts. */
  openStream(envelopeText: string): AsyncIterable<string>;
}

type RequestScript = (envelope: RequestEnvelope) => ResultEnvelope;
type StreamScript = (envelope: RequestEnvelope) => Iterable<string> | AsyncIterable<string>;

/**
 * Scriptable in-memory transport. Records every parsed request envelope so
 * tests can assert what was — and was not — sent (e.g. detach sends no
 * cancel). Handlers are plain functions; sequencing state lives in test
 * closures.
 */
export class InMemoryTransport implements AkpTransport {
  readonly channel: ClientChannel;
  /** Every request envelope this transport ever saw, in order. */
  readonly requests: RequestEnvelope[] = [];
  /** Every stream-open envelope this transport ever saw, in order. */
  readonly streamOpens: RequestEnvelope[] = [];

  private requestScript: RequestScript;
  private streamScript: StreamScript | undefined;

  constructor(channel: ClientChannel, requestScript: RequestScript, streamScript?: StreamScript) {
    this.channel = channel;
    this.requestScript = requestScript;
    this.streamScript = streamScript;
  }

  /** Replace the request script mid-test. */
  scriptRequest(script: RequestScript): void {
    this.requestScript = script;
  }

  /** Replace the stream script mid-test. */
  scriptStream(script: StreamScript): void {
    this.streamScript = script;
  }

  request(envelopeText: string): Promise<TransportReply> {
    const envelope = parseRequestEnvelope(envelopeText);
    this.requests.push(envelope);
    const result = this.requestScript(envelope);
    return Promise.resolve({ transportStatus: 200, body: serializeEnvelope(result) });
  }

  async *openStream(envelopeText: string): AsyncIterable<string> {
    const envelope = parseRequestEnvelope(envelopeText);
    this.streamOpens.push(envelope);
    if (!this.streamScript) {
      throw new Error("InMemoryTransport: no stream script installed");
    }
    yield* this.streamScript(envelope);
  }
}

export interface HttpSseTransportInit {
  readonly baseUrl: string;
  readonly channel: ClientChannel;
  /** Channel session material; never shared with the other channel. */
  readonly bearer: string;
  readonly fetchImpl?: typeof fetch;
}

/**
 * Default HTTP JSON + SSE binding (ADR-0003). Request/response operations
 * POST to `<base>/akp/<channel>/request`; streams POST to
 * `<base>/akp/<channel>/stream` and consume `data:` SSE lines. Routes are
 * provisional until M5 pins the kernel-server surface; integration against
 * a real server is executed at M5, not here.
 */
export class HttpSseTransport implements AkpTransport {
  readonly channel: ClientChannel;
  private readonly baseUrl: string;
  private readonly bearer: string;
  private readonly fetchImpl: typeof fetch;

  constructor(init: HttpSseTransportInit) {
    this.channel = init.channel;
    this.baseUrl = init.baseUrl.replace(/\/+$/, "");
    this.bearer = init.bearer;
    this.fetchImpl = init.fetchImpl ?? fetch;
  }

  private headers(): Record<string, string> {
    return {
      "content-type": "application/json",
      authorization: `Bearer ${this.bearer}`,
    };
  }

  async request(envelopeText: string): Promise<TransportReply> {
    const response = await this.fetchImpl(`${this.baseUrl}/akp/${this.channel}/request`, {
      method: "POST",
      headers: this.headers(),
      body: envelopeText,
    });
    return { transportStatus: response.status, body: await response.text() };
  }

  async *openStream(envelopeText: string): AsyncIterable<string> {
    const response = await this.fetchImpl(`${this.baseUrl}/akp/${this.channel}/stream`, {
      method: "POST",
      headers: { ...this.headers(), accept: "text/event-stream" },
      body: envelopeText,
    });
    if (response.body === null) {
      return;
    }
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      for (;;) {
        const { done, value } = await reader.read();
        if (done) {
          break;
        }
        buffer += decoder.decode(value, { stream: true });
        let boundary = buffer.indexOf("\n\n");
        while (boundary >= 0) {
          const eventBlock = buffer.slice(0, boundary);
          buffer = buffer.slice(boundary + 2);
          const data = eventBlock
            .split("\n")
            .filter((line) => line.startsWith("data:"))
            .map((line) => line.slice(5).trimStart())
            .join("\n");
          if (data.length > 0) {
            yield data;
          }
          boundary = buffer.indexOf("\n\n");
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
}
