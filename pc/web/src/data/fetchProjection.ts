/*
 * Fetch pipeline: readJson → normalize → stub/whitelist defense → project.
 * Produces the honest Projection status for every failure class. Views never
 * fetch directly and never see raw backend JSON.
 *
 * Refetch is stale-while-revalidate (docs/design/22 §state system,
 * docs/design/06): a refresh over an existing projection keeps the last-good
 * data on screen as `stale`, carrying its own updatedAt/source so views label
 * the age instead of blanking. Only a first load (no last-good data) shows
 * `loading`. Failure classes are unchanged and never inherit last-good data —
 * a failed refresh is reported as the failure, not as current content.
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
  const previous = store.get<T>(key);
  const started: Projection<T> =
    previous?.data === undefined
      ? { status: "loading", source: path }
      : {
          status: "stale",
          data: previous.data,
          cursor: previous.cursor,
          source: previous.source ?? path,
          updatedAt: previous.updatedAt,
        };
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
