import { bearer, type ChannelClass } from "./session";
import { containsSecretMaterial } from "./policy";

export const MANAGEMENT_PREFIXES = [
  "/management/",
  "/personal/status",
  "/personal/readiness",
  "/personal/doctor",
  "/resource/v1/",
] as const;

export const TASK_PREFIXES = ["/task/"] as const;

export class ChannelError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ChannelError";
  }
}

export function classifyPath(path: string): ChannelClass | "none" {
  if (path.startsWith("/local/session") || path === "/personal/health" || path.startsWith("/ui")) {
    return "none";
  }
  if (TASK_PREFIXES.some((prefix) => path.startsWith(prefix))) {
    return "task";
  }
  if (MANAGEMENT_PREFIXES.some((prefix) => path.startsWith(prefix))) {
    return "management";
  }
  if (path.startsWith("/provider/")) {
    return "management";
  }
  return "management";
}

export function assertChannelBinding(path: string, presented: ChannelClass): void {
  const required = classifyPath(path);
  if (required === "none") {
    return;
  }
  if (required !== presented) {
    throw new ChannelError("SHELL_CHANNEL_BINDING_MISMATCH");
  }
}

export function authorizationHeader(path: string, presented: ChannelClass): string | undefined {
  assertChannelBinding(path, presented);
  if (classifyPath(path) === "none") {
    return undefined;
  }
  const token = bearer(presented);
  if (!token) {
    throw new ChannelError("UNAUTHORIZED");
  }
  return `Bearer ${token}`;
}

export async function daemonFetch(
  path: string,
  presented: ChannelClass,
  init: RequestInit = {},
): Promise<Response> {
  const headers = new Headers(init.headers);
  const authorization = authorizationHeader(path, presented);
  if (authorization) {
    headers.set("Authorization", authorization);
  }
  const url = new URL(path, window.location.origin);
  if (containsSecretMaterial(url.href) || containsSecretMaterial(url.search)) {
    throw new ChannelError("secret material must not enter the URL");
  }
  return fetch(url, { ...init, headers, credentials: "omit" });
}
