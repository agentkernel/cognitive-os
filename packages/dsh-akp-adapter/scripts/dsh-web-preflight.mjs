/**
 * Fail-closed checks for the native dsh web control panel (P8-T15).
 *
 * The native panel is `dsh --profile web` / `dsh web`, not CognitiveOS Personal
 * `/ui/`. The pinned overlay currently ships `build:lib` only; the web app
 * needs `apps/web/dist`. Missing dist must fail closed rather than serve an
 * empty 404. The webserver has no TLS/auth, so the product bind is loopback
 * only.
 */
import { existsSync } from "node:fs";
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
