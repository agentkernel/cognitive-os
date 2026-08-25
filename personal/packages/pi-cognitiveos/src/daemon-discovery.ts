/**
 * Locate the Personal daemon the same way `cognitive` does (ADR-0022 §2,
 * `personal/crates/cognitive-store/src/layout.rs`, `personal/apps/admin-cli/src/personal_cli`).
 *
 * The Extension reads exactly two local files:
 *   - `$XDG_STATE_HOME/cognitiveos/daemon-endpoint.json` — the loopback address
 *     published by `cognitive daemon start`;
 *   - `$XDG_RUNTIME_DIR/cognitiveos/local-bootstrap.secret` — the 0600 local
 *     auth bootstrap used to mint a channel-scoped bearer.
 *
 * It never reads `provider.json`, never resolves a `SecretRef`, never touches a
 * SQLite database, and never reads a Provider API key from the environment or
 * anywhere else. The bootstrap secret is a *local session* bootstrap, not a
 * Provider credential; it is held in memory for the length of one session
 * request and is never logged, returned or written.
 *
 * `XDG_RUNTIME_DIR` is required and fails closed exactly as the Rust layout
 * does — the Extension must not invent a fallback location for a 0600 secret.
 */

import { readFileSync } from "node:fs";
import path from "node:path";

import { DaemonClientError } from "./errors.js";

/** Product subdirectory appended to every XDG root (`PERSONAL_PRODUCT_DIR_NAME`). */
export const PERSONAL_PRODUCT_DIR_NAME = "cognitiveos";

export const ENDPOINT_FILE_NAME = "daemon-endpoint.json";
export const BOOTSTRAP_SECRET_FILE_NAME = "local-bootstrap.secret";
export const ENDPOINT_SCHEMA_VERSION = 1;
export const ENDPOINT_SURFACE = "personal-daemon-endpoint";

/** Environment slice this module reads. Injected so tests never touch the host. */
export type EnvironmentSlice = Readonly<Record<string, string | undefined>>;

/** Minimal filesystem port, injected so tests never touch the host. */
export interface FileReader {
  readTextFile(filePath: string): string;
}

export const nodeFileReader: FileReader = {
  readTextFile(filePath: string): string {
    return readFileSync(filePath, "utf8");
  },
};

export interface PersonalDaemonPaths {
  readonly stateDir: string;
  readonly runtimeDir: string;
  readonly endpointFile: string;
  readonly bootstrapSecretFile: string;
}

/**
 * Resolve the Personal XDG paths this Extension needs.
 *
 * Mirrors `PersonalDataLayout::resolve_from_env`: `HOME` (or `USERPROFILE`) is
 * required, `XDG_STATE_HOME` defaults to `$HOME/.local/state`, and
 * `XDG_RUNTIME_DIR` has no default.
 */
export function resolvePersonalDaemonPaths(environment: EnvironmentSlice): PersonalDaemonPaths {
  const home = firstNonEmpty(environment["HOME"], environment["USERPROFILE"]);
  if (home === undefined) {
    throw new DaemonClientError(
      "PI_EXTENSION_HOME_MISSING",
      "cannot resolve the CognitiveOS data layout: neither HOME nor USERPROFILE is set",
    );
  }

  const runtimeRoot = firstNonEmpty(environment["XDG_RUNTIME_DIR"]);
  if (runtimeRoot === undefined) {
    throw new DaemonClientError(
      "PI_EXTENSION_RUNTIME_DIR_MISSING",
      "cannot locate the CognitiveOS daemon: XDG_RUNTIME_DIR is not set, and a 0600 bootstrap secret has no fallback location",
    );
  }

  const stateRoot =
    firstNonEmpty(environment["XDG_STATE_HOME"]) ?? path.join(home, ".local", "state");
  const stateDir = path.join(stateRoot, PERSONAL_PRODUCT_DIR_NAME);
  const runtimeDir = path.join(runtimeRoot, PERSONAL_PRODUCT_DIR_NAME);

  return {
    stateDir,
    runtimeDir,
    endpointFile: path.join(stateDir, ENDPOINT_FILE_NAME),
    bootstrapSecretFile: path.join(runtimeDir, BOOTSTRAP_SECRET_FILE_NAME),
  };
}

/**
 * Read the published loopback endpoint. Refuses any address that is not
 * loopback: the daemon binds loopback only, so a non-loopback endpoint file
 * means the file is wrong, and following it would send a bearer somewhere else.
 */
export function readDaemonEndpoint(paths: PersonalDaemonPaths, files: FileReader): string {
  let document: string;
  try {
    document = files.readTextFile(paths.endpointFile);
  } catch {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_MISSING",
      `daemon endpoint file missing at ${paths.endpointFile}; start the daemon with \`cognitive daemon start\``,
    );
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(document);
  } catch {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_CORRUPT",
      `daemon endpoint file at ${paths.endpointFile} is not valid JSON`,
    );
  }

  if (typeof parsed !== "object" || parsed === null) {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_CORRUPT",
      `daemon endpoint file at ${paths.endpointFile} is not a JSON object`,
    );
  }

  const record = parsed as Record<string, unknown>;
  if (record["schema_version"] !== ENDPOINT_SCHEMA_VERSION) {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_CORRUPT",
      `daemon endpoint file at ${paths.endpointFile} declares an unsupported schema_version; this Extension speaks version ${ENDPOINT_SCHEMA_VERSION}`,
    );
  }
  if (record["surface"] !== ENDPOINT_SURFACE) {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_CORRUPT",
      `daemon endpoint file at ${paths.endpointFile} is not a ${ENDPOINT_SURFACE} document`,
    );
  }

  const endpoint = record["endpoint"];
  if (typeof endpoint !== "string" || endpoint.trim().length === 0) {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_CORRUPT",
      `daemon endpoint file at ${paths.endpointFile} has no endpoint field`,
    );
  }

  const trimmed = endpoint.trim();
  if (!isLoopbackEndpoint(trimmed)) {
    throw new DaemonClientError(
      "PI_EXTENSION_ENDPOINT_FILE_CORRUPT",
      `daemon endpoint file at ${paths.endpointFile} does not point at loopback; the Personal daemon binds loopback only`,
    );
  }
  return trimmed;
}

/**
 * Read the local bootstrap secret. The value is returned to the caller for a
 * single session exchange and is never stored, logged or embedded in an error.
 */
export function readBootstrapSecret(paths: PersonalDaemonPaths, files: FileReader): string {
  let contents: string;
  try {
    contents = files.readTextFile(paths.bootstrapSecretFile);
  } catch {
    throw new DaemonClientError(
      "PI_EXTENSION_BOOTSTRAP_SECRET_MISSING",
      `local bootstrap secret missing at ${paths.bootstrapSecretFile}; the Personal daemon is not running`,
    );
  }

  const secret = contents.trim();
  if (secret.length === 0) {
    throw new DaemonClientError(
      "PI_EXTENSION_BOOTSTRAP_SECRET_MISSING",
      `local bootstrap secret at ${paths.bootstrapSecretFile} is empty`,
    );
  }
  return secret;
}

/** Loopback host check matching the daemon's own `Host` header rule. */
export function isLoopbackEndpoint(endpoint: string): boolean {
  const host = hostOf(endpoint);
  return host === "127.0.0.1" || host === "localhost" || host === "localhost." || host === "::1";
}

function hostOf(endpoint: string): string {
  if (endpoint.startsWith("[")) {
    const closing = endpoint.indexOf("]");
    return closing === -1 ? "" : endpoint.slice(1, closing);
  }
  const lastColon = endpoint.lastIndexOf(":");
  return lastColon === -1 ? endpoint : endpoint.slice(0, lastColon);
}

function firstNonEmpty(...values: readonly (string | undefined)[]): string | undefined {
  for (const value of values) {
    if (value !== undefined && value.trim().length > 0) {
      return value;
    }
  }
  return undefined;
}
