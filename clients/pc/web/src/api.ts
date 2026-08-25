import { daemonFetch } from "./channels";
import { containsSecretMaterial, redactSecrets } from "./policy";
import type { ChannelClass } from "./session";

export type Json = Record<string, unknown>;

export async function readJson(
  path: string,
  channel: ChannelClass,
  init?: RequestInit,
): Promise<{ ok: boolean; status: number; body: unknown; ms: number }> {
  if (containsSecretMaterial(path)) {
    throw new Error("secret material must not enter the URL");
  }
  const started = performance.now();
  const response = await daemonFetch(path, channel, init);
  const text = await response.text();
  let body: unknown = text;
  if (text.length > 0) {
    try {
      body = JSON.parse(text) as unknown;
    } catch {
      body = { raw: text };
    }
  }
  return {
    ok: response.ok,
    status: response.status,
    body: redactSecrets(body),
    ms: Math.round(performance.now() - started),
  };
}

export async function issueChannelSession(
  channel: ChannelClass,
  principalId: string,
  bootstrapSecret: string,
): Promise<{ ok: boolean; status: number; token?: string }> {
  const response = await fetch("/local/session", {
    method: "POST",
    credentials: "omit",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      channel,
      principal_id: principalId,
      bootstrap_secret: bootstrapSecret,
    }),
  });
  if (!response.ok) {
    return { ok: false, status: response.status };
  }
  const body = (await response.json()) as { token?: string };
  return { ok: true, status: response.status, token: body.token };
}

export function rejectCallerHeaderInjection(document: Json): void {
  if ("headers" in document || "authorization" in document) {
    throw new Error("arbitrary header injection is forbidden");
  }
}
