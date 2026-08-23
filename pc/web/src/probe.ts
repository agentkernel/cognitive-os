/** Map daemon Provider probe/error facts into UI classes. Not an authority. */

export type ProbeClass =
  | "reachability"
  | "authentication"
  | "model_discovery"
  | "capability"
  | "ok"
  | "unknown";

export type ProbeView = {
  class: ProbeClass;
  label: string;
  nextAction: string;
};

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

export function envelopeCode(body: unknown): string {
  const record = asRecord(body);
  return String(record.code ?? "");
}

export function envelopeMessage(body: unknown): string {
  const record = asRecord(body);
  return String(record.message ?? record.detail ?? "");
}

export function classifyProbe(input: {
  ok: boolean;
  httpStatus: number;
  body: unknown;
  accountStatus?: string;
}): ProbeView {
  const code = envelopeCode(input.body).toUpperCase();
  const message = envelopeMessage(input.body).toLowerCase();
  const combined = `${code} ${message}`.toLowerCase();

  if (
    combined.includes("key_missing") ||
    combined.includes("secret") ||
    combined.includes("401") ||
    combined.includes("403")
  ) {
    return {
      class: "authentication",
      label: "authentication",
      nextAction: "Rotate the API key through the daemon SecretStore path.",
    };
  }
  if (
    combined.includes("endpoint") ||
    combined.includes("transport") ||
    combined.includes("dns") ||
    combined.includes("5xx") ||
    combined.includes("discovery_failed")
  ) {
    return {
      class: "reachability",
      label: "reachability",
      nextAction: "Check endpoint trust, network grants, and that the daemon can reach the host.",
    };
  }
  if (combined.includes("discovery_malformed") || combined.includes("404")) {
    return {
      class: "model_discovery",
      label: "model_discovery",
      nextAction: "Keep the last catalog. Add a model manually or retry refresh.",
    };
  }
  if (input.ok && input.httpStatus >= 200 && input.httpStatus < 300) {
    return {
      class: "ok",
      label: "model_discovery",
      nextAction: "Catalog refreshed. Capability remains a separate daemon fact, not implied by TCP.",
    };
  }
  if (input.accountStatus === "degraded") {
    return {
      class: "model_discovery",
      label: "model_discovery",
      nextAction: "Last catalog and binding are preserved. Repair discovery, then re-probe.",
    };
  }
  return {
    class: "unknown",
    label: "unknown",
    nextAction: "Treat the account as not ready. Do not display unknown as zero or ready.",
  };
}

export function capabilityDisposition(flags: unknown): string {
  if (flags == null) {
    return "not-run";
  }
  return "unknown";
}

export const PROVIDER_KINDS = [
  "openai_official",
  "anthropic_official",
  "openai_compatible",
] as const;

export type ProviderKind = (typeof PROVIDER_KINDS)[number];

export function requiresTrustConfirmation(input: {
  kind: string;
  allowPrivateNetwork: boolean;
  allowInsecureHttp: boolean;
}): boolean {
  if (input.kind !== "openai_compatible") {
    return false;
  }
  return input.allowPrivateNetwork || input.allowInsecureHttp;
}
