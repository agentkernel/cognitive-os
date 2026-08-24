/*
 * Fetch pipeline: readJson → normalize → stub/whitelist defense → project.
 * Produces the honest Projection status for every failure class. Views never
 * fetch directly and never see raw backend JSON.
 */

import { readJson } from "../api";
import type { ChannelClass } from "../session";
import { isKnownRoute, normalizeBody } from "./normalize";
import type { Projection, ProjectionStore } from "./store";

export async function fetchProjection<T>(
  store: ProjectionStore,
  key: string,
  path: string,
  channel: ChannelClass,
  project: (body: unknown) => T,
): Promise<Projection<T>> {
  const started: Projection<T> = { status: "loading", source: path };
  store.set(key, started);

  // Defense against the daemon's 200-stub fallthrough: this pipeline is for
  // reads (GET) only, and only to whitelisted routes. An unknown route is
  // reported as not-run without issuing a request.
  if (!isKnownRoute("GET", path)) {
    const projection: Projection<T> = {
      status: "not-run",
      source: path,
      updatedAt: Date.now(),
      error: {
        code: "ROUTE_NOT_WHITELISTED",
        message: "This route is not in the client route whitelist; not called.",
        httpStatus: 0,
      },
    };
    store.set(key, projection);
    return projection;
  }

  let result: Awaited<ReturnType<typeof readJson>>;
  try {
    result = await readJson(path, channel);
  } catch (error) {
    const projection: Projection<T> = {
      status: "disconnected",
      source: path,
      updatedAt: Date.now(),
      error: {
        code: "DISCONNECTED",
        message: error instanceof Error ? error.message : "daemon unreachable",
        httpStatus: 0,
      },
    };
    store.set(key, projection);
    return projection;
  }

  const normalized = normalizeBody(result.status, result.body);
  if (!normalized.ok) {
    const status = normalized.stub
      ? "not-run"
      : result.status === 401 || result.status === 403
        ? "denied"
        : "unknown";
    const projection: Projection<T> = {
      status,
      source: path,
      updatedAt: Date.now(),
      error: normalized.error,
    };
    store.set(key, projection);
    return projection;
  }

  const data = project(result.body);
  const projection: Projection<T> = {
    status: Array.isArray(data) && data.length === 0 ? "empty" : "ready",
    data,
    source: path,
    updatedAt: Date.now(),
  };
  store.set(key, projection);
  return projection;
}
