/**
 * Fail-closed checks for the native dsh web control panel (P8-T15).
 *
 * The native panel is `dsh --profile web` / `dsh web`, not CognitiveOS Personal
 * `/ui/`. The pinned overlay currently ships `build:lib` only; the web app
 * needs `apps/web/dist`. Missing dist must fail closed rather than serve an
 * empty 404. The webserver has no TLS/auth, so the product bind is loopback
 * only.
 */
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { isIP } from "node:net";
import { join } from "node:path";

export const DEFAULT_WEB_HOST = "127.0.0.1";
export const DEFAULT_WEB_PORT = 3080;

export function frontendDistIndex(dshRoot) {
  return join(dshRoot, "apps/web/dist/index.html");
}

export function assertFrontendDist(dshRoot) {
  const index = frontendDistIndex(dshRoot);
  if (!existsSync(index)) {
    throw new Error(
      `dsh web frontend dist is missing at ${index}; run pnpm run build from the pinned dsh root, then retry. Headless \`cognitive dsh launch --print\` remains available.`,
    );
  }
  return index;
}

export function assertLoopbackHost(host) {
  const value = String(host ?? "").trim();
  if (!value) {
    throw new Error("dsh web --host must be a loopback address");
  }
  if (value === "0.0.0.0" || value === "::" || value === "[::]") {
    throw new Error(
      "dsh web --host 0.0.0.0/:: is refused; native dsh web has no TLS/auth and must bind loopback only",
    );
  }
  if (value.toLowerCase() === "localhost") {
    return DEFAULT_WEB_HOST;
  }
  const unwrapped = value.startsWith("[") && value.endsWith("]") ? value.slice(1, -1) : value;
  const ipVersion = isIP(unwrapped);
  if (ipVersion === 4) {
    const octets = unwrapped.split(".").map(Number);
    if (octets[0] === 127) {
      return unwrapped;
    }
  }
  if (ipVersion === 6) {
    const normalized = unwrapped.toLowerCase();
    if (normalized === "::1" || normalized.endsWith("::1")) {
      return unwrapped;
    }
  }
  throw new Error(
    `dsh web --host must be a loopback address (got ${value}); 0.0.0.0 is refused`,
  );
}

export function assertWebPort(port) {
  const numeric = Number(port);
  if (!Number.isInteger(numeric) || numeric < 1 || numeric > 65535) {
    throw new Error("dsh web --port must be an integer 1..65535");
  }
  return numeric;
}

export function listenUrl(host, port) {
  const bound = assertLoopbackHost(host);
  const boundPort = assertWebPort(port);
  if (bound.includes(":")) {
    return `http://[${bound}]:${boundPort}`;
  }
  return `http://${bound}:${boundPort}`;
}

/** Runtime Path B credential ref used by the `--patch` llm-deepseek overlay. */
export const PATH_B_WEB_DAEMON_KEY_REF = "DAEMON_BEARER";
/** Daemon-written Cos Models overlay; native web reloads when this file changes. */
export const DSH_WEB_OVERLAY_FILE = "control-plane-overlay.json";
export const DSH_WEB_OVERLAY_APPLIED_FILE = "control-plane-overlay.applied.json";
export const DSH_WEB_OVERLAY_SURFACE = "personal-dsh-web-overlay";
export const DSH_WEB_OVERLAY_APPLIED_SURFACE = "personal-dsh-web-overlay-applied";
/**
 * Official DeepSeek catalog ref the Models page still describes by default.
 * Path B aliases this to the daemon management bearer — never a SecretStore key.
 */
export const PATH_B_WEB_OFFICIAL_KEY_REF = "DEEPSEEK_API_KEY";

export function assertPathBProviderBase(providerBase) {
  const base = String(providerBase ?? "").trim().replace(/\/$/, "");
  const allowed = /^https?:\/\/(127\.0\.0\.\d+|\[::1\]|::1):(\d{1,5})\/provider\/v1\/dsh$/i;
  const match = allowed.exec(base);
  if (!match) {
    throw new Error(
      "Path B web settings baseURL must be a loopback daemon /provider/v1/dsh origin",
    );
  }
  const port = Number(match[2]);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error("Path B web settings baseURL port must be 1..65535");
  }
  return base;
}

/**
 * `$DSH_HOME/.credentials.yaml` for native web Path B.
 * Both refs are the daemon management bearer so Models `credentials.describe`
 * reports configured without a second SecretStore copy or dsh `.env` key.
 */
export function pathBWebCredentialsYaml(managementToken) {
  const token = String(managementToken ?? "");
  if (!token.trim()) {
    throw new Error("Path B web credentials require a non-empty daemon management token");
  }
  const quoted = JSON.stringify(token);
  return [
    "version: 1",
    "",
    "refs:",
    `  ${PATH_B_WEB_DAEMON_KEY_REF}: ${quoted}`,
    `  ${PATH_B_WEB_OFFICIAL_KEY_REF}: ${quoted}`,
    "",
  ].join("\n");
}

function safeCatalogId(value) {
  const id = String(value ?? "").trim();
  if (!id || /[\s#:]/.test(id)) {
    return "";
  }
  return id;
}

/**
 * Bound-account catalog only. Do not inject a selected id that is not in the
 * control-plane list (that leftover grok stays on Cos Models after unbind).
 * If discovery returned nothing, keep the bound selected id so chat still has a name.
 * Ids must be YAML-safe; never include secrets.
 */
export function pathBWebCatalogModels(rawModels, selectedModel) {
  const seen = new Set();
  const out = [];
  const push = (id, name) => {
    const modelId = safeCatalogId(id);
    if (!modelId || seen.has(modelId)) {
      return;
    }
    seen.add(modelId);
    const label = safeCatalogId(name) || modelId;
    out.push({ id: modelId, name: label });
  };
  for (const row of Array.isArray(rawModels) ? rawModels : []) {
    if (typeof row === "string") {
      push(row, row);
    } else if (row && typeof row === "object") {
      push(row.id ?? row.model_id, row.name ?? row.id ?? row.model_id);
    }
  }
  if (out.length === 0) {
    push(selectedModel, selectedModel);
  }
  return out;
}

export function readDshWebControlPlaneOverlay(dshHome) {
  const path = join(dshHome, DSH_WEB_OVERLAY_FILE);
  if (!existsSync(path)) {
    return null;
  }
  try {
    const doc = JSON.parse(readFileSync(path, "utf8"));
    if (doc?.schema_version !== 1 || doc?.surface !== DSH_WEB_OVERLAY_SURFACE) {
      return null;
    }
    const bound = doc.bound === true;
    const model = bound ? safeCatalogId(doc.model) : "";
    const catalog = bound ? pathBWebCatalogModels(doc.catalog, model) : [];
    return {
      bound,
      model,
      catalog,
      written_at_ms: Number(doc.written_at_ms) || 0,
    };
  } catch {
    return null;
  }
}

export function overlayStamp(overlay) {
  if (!overlay) {
    return "";
  }
  return `${overlay.written_at_ms}:${overlay.bound ? "1" : "0"}:${overlay.model}:${overlay.catalog.map((item) => item.id).join(",")}`;
}

export function writeDshWebControlPlaneOverlayApplied(dshHome, writtenAtMs, processId) {
  const payload = {
    schema_version: 1,
    surface: DSH_WEB_OVERLAY_APPLIED_SURFACE,
    written_at_ms: Number(writtenAtMs) || 0,
    process_id: Number(processId) || 0,
    applied_at_ms: Date.now(),
  };
  writeFileSync(join(dshHome, DSH_WEB_OVERLAY_APPLIED_FILE), `${JSON.stringify(payload)}\n`, {
    encoding: "utf8",
    mode: 0o600,
  });
}

/**
 * Persist llm-deepseek onto the settings document the Models page joins,
 * so dynamic config cannot fall back to api.deepseek.com + official DeepSeek names.
 */
export function pathBWebSettingsYaml(providerBase, existingYaml, selectedModel, catalogModels) {
  const base = assertPathBProviderBase(providerBase);
  let welcome = "2026-08-13.1";
  const match = String(existingYaml ?? "").match(/welcomeNoticeVersion:\s*(\S+)/);
  if (match) {
    welcome = match[1];
  }
  const model = safeCatalogId(selectedModel);
  const lines = [
    "ui-onboarding:",
    `  welcomeNoticeVersion: ${welcome}`,
    "llm-deepseek:",
    `  baseURL: ${base}`,
    `  apiKeyEnv: ${PATH_B_WEB_DAEMON_KEY_REF}`,
  ];
  if (model) {
    lines.push(`  model: ${model}`);
  }
  const models = pathBWebCatalogModels(catalogModels, model);
  if (!models.length) {
    lines.push("  models: []");
  } else {
    lines.push("  models:");
    for (const item of models) {
      lines.push(`    - id: ${item.id}`);
      lines.push(`      name: ${item.name}`);
    }
  }
  lines.push("");
  return lines.join("\n");
}

/** Non-secret child env: official-catalog fallback URL only. Never an API key. */
export function pathBWebChildExtras(providerBase) {
  return { DEEPSEEK_BASE_URL: assertPathBProviderBase(providerBase) };
}

/**
 * Completion budget for bounded one-shot probes.
 *
 * A reasoning model spends this budget on `reasoning_content` before it emits
 * any `content`, so the budget must leave room for both. The former 256-token
 * pin returned a completed response with an empty `content`, which the harness
 * classifies as `EMPTY_RESPONSE` and retries until the turn fails.
 */
export const PROBE_COMPLETION_BUDGET_TOKENS = 4096;

/**
 * `--patch` overlay lines for the pinned `llm-deepseek` provider.
 *
 * `maxTokens` is omitted unless a caller asks for a bounded probe, so the
 * interactive panel inherits the provider's own default instead of a pin that
 * starves reasoning models.
 */
export function llmDeepseekPatchLines(
  baseURL,
  apiKeyEnv,
  selectedModel,
  catalogModels,
  maxTokens,
) {
  const lines = [
    "- id: llm-deepseek",
    "  config:",
    `    baseURL: ${baseURL}`,
    `    apiKeyEnv: ${apiKeyEnv}`,
    "    thinking: disabled",
    "    reasoningEffort: off",
  ];
  if (maxTokens !== undefined) {
    const budget = Number(maxTokens);
    if (!Number.isSafeInteger(budget) || budget <= 0) {
      throw new Error("llm-deepseek maxTokens must be a positive safe integer when pinned");
    }
    lines.push(`    maxTokens: ${budget}`);
  }
  const model = safeCatalogId(selectedModel);
  if (model) {
    lines.push(`    model: ${model}`);
  }
  const models = pathBWebCatalogModels(catalogModels, model);
  if (!models.length) {
    lines.push("    models: []");
  } else {
    lines.push("    models:");
    for (const item of models) {
      lines.push(`      - id: ${item.id}`);
      lines.push(`        name: ${item.name}`);
    }
  }
  lines.push("");
  return lines.join("\n");
}
