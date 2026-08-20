/**
 * Mechanical redactor for C1/C2 paired evidence (measurement-only).
 * Rejects unredacted secret-shaped bytes. Does not write authority state.
 */

import { isSecretShaped, PI_PLACEHOLDER_TOKEN } from "./pure-pi-broker.mjs";

const FORBIDDEN =
  /Authorization:\s*Bearer\s+(?!campaign-broker-nonsecret-token)[A-Za-z0-9._-]{8,}/i;

export function redactPairedEvidence(value) {
  const text = typeof value === "string" ? value : JSON.stringify(value);
  if (isSecretShaped(text) || FORBIDDEN.test(text)) {
    throw new Error("unredacted secret-shaped evidence refused");
  }
  return {
    redacted: true,
    placeholder_token_allowed: text.includes(PI_PLACEHOLDER_TOKEN),
    retry: 0,
  };
}
