import { readFileSync } from "node:fs";
import {
  ADAPTER_ID,
  DshAdapterError,
  HttpAkpTransport,
  PINNED_AKP_SCHEMA_DIGEST,
  PINNED_DSH_REVISION,
  attachDshCordisPlugin,
  type AkpTransport,
  type DshAkpAdapter,
  type DshAkpResult,
  type DshCordisEventSource,
  type DshPluginEvent,
} from "./index.js";

export const name = "cognitiveos-akp";
export const inject: readonly string[] = [];

export interface DshAkpCordisPluginConfig {
  readonly endpoint: string;
  readonly bearerFile: string;
  readonly sessionId?: string;
  readonly pluginId?: string;
  readonly taskRef?: string;
  readonly fencingEpoch?: number;
  readonly timeoutMs?: number;
  readonly startupEvents?: readonly DshPluginEvent[];
}

export interface DshAkpCordisPluginDeps {
  readonly transport?: AkpTransport;
  readonly readBearer?: (path: string) => string;
  readonly onResult?: (result: DshAkpResult) => void;
  readonly onError?: (error: unknown) => void;
}

/**
 * Cordis `apply` entry for `dsh --patch`. The management bearer is read from a
 * 0600 file path in plugin config. It is never taken from a plugin event, argv
 * secret, or ordinary log line.
 */
export function apply(
  ctx: DshCordisEventSource,
  config: DshAkpCordisPluginConfig,
): DshAkpAdapter {
  return applyDshAkpCordisPlugin(ctx, config);
}

export function applyDshAkpCordisPlugin(
  ctx: DshCordisEventSource,
  config: DshAkpCordisPluginConfig,
  deps: DshAkpCordisPluginDeps = {},
): DshAkpAdapter {
  if (!config.endpoint.trim() || !config.bearerFile.trim()) {
    throw new DshAdapterError(
      "INVALID_EVENT",
      "dsh AKP plugin requires an HTTP endpoint and a bearer file path",
    );
  }
  const transport =
    deps.transport ??
    new HttpAkpTransport({
      endpoint: config.endpoint,
      bearer: readBearerMaterial(config.bearerFile, deps.readBearer),
    });
  const adapter = attachDshCordisPlugin(ctx, {
    dshVersion: PINNED_DSH_REVISION,
    schemaDigest: PINNED_AKP_SCHEMA_DIGEST,
    sessionId: config.sessionId ?? "dsh-cordis",
    pluginId: config.pluginId ?? ADAPTER_ID,
    transport,
    ...(config.fencingEpoch !== undefined ? { fencingEpoch: config.fencingEpoch } : {}),
    ...(config.taskRef !== undefined ? { taskRef: config.taskRef } : {}),
    ...(config.timeoutMs !== undefined ? { timeoutMs: config.timeoutMs } : {}),
    ...(deps.onResult ? { onResult: deps.onResult } : {}),
    ...(deps.onError ? { onError: deps.onError } : {}),
  });
  for (const event of config.startupEvents ?? []) {
    void adapter.submit(event).then(deps.onResult).catch(deps.onError);
  }
  return adapter;
}

function readBearerMaterial(path: string, readBearer?: (path: string) => string): string {
  const bearer = (readBearer ?? defaultReadBearer)(path);
  if (!bearer) {
    throw new DshAdapterError("INVALID_EVENT", "dsh AKP bearer file is empty");
  }
  return bearer;
}

function defaultReadBearer(path: string): string {
  return readFileSync(path, "utf8").trim();
}
