/*
 * API normalization — docs/design/28 (R-1, R-2) and docs/design/03 §2.
 *
 * The daemon has three error envelope shapes and a dangerous 200-stub
 * fallthrough on unmatched /management/* and /task/* routes. This module is
 * the single place that turns raw readJson results into one honest shape.
 * React components never see raw backend error objects.
 */

export interface NormalizedError {
  code: string;
  message: string;
  category?: string;
  retryable?: boolean;
  httpStatus: number;
}

export type Normalized =
  | { ok: true; httpStatus: number; data: unknown; stub: false }
  | { ok: false; httpStatus: number; error: NormalizedError; stub: boolean };

/** The daemon's 200-stub notes (server.rs:1086-1095, task_api.rs:346-356). */
const STUB_NOTE_FRAGMENTS = [
  "business routes deferred",
  "no Task API operation matched",
] as const;

/** Detect the daemon's 200-stub fallthrough so it never renders as success. */
export function detectStubNote(body: unknown): boolean {
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    return false;
  }
  const note = (body as Record<string, unknown>).note;
  if (typeof note !== "string") {
    return false;
  }
  return STUB_NOTE_FRAGMENTS.some((fragment) => note.includes(fragment));
}

function asObj(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

/**
 * Normalize one readJson result. Handles all three daemon error envelopes:
 * front-door {status,error{code,message,category,retryable,stage}},
 * task/resource {status:"error",code,message}, backup {error:{code,detail}}.
 */
export function normalizeBody(httpStatus: number, body: unknown): Normalized {
  const record = asObj(body);

  if (httpStatus === 200 && record && detectStubNote(record)) {
    return {
      ok: false,
      httpStatus,
      stub: true,
      error: {
        code: "STUB_ROUTE",
        message:
          "The daemon returned its authenticated-front-door stub: this operation is not implemented over HTTP.",
        httpStatus,
      },
    };
  }

  const frontDoor = record && asObj(record.error);
  if (record && record.status === "error") {
    if (frontDoor && typeof frontDoor.code === "string") {
      return {
        ok: false,
        httpStatus,
        stub: false,
        error: {
          code: frontDoor.code,
          message: typeof frontDoor.message === "string" ? frontDoor.message : "daemon error",
          category: typeof frontDoor.category === "string" ? frontDoor.category : undefined,
          retryable:
            typeof frontDoor.retryable === "boolean" ? frontDoor.retryable : undefined,
          httpStatus,
        },
      };
    }
    return {
      ok: false,
      httpStatus,
      stub: false,
      error: {
        code: typeof record.code === "string" ? record.code : `HTTP_${httpStatus}`,
        message: typeof record.message === "string" ? record.message : "daemon error",
        httpStatus,
      },
    };
  }
  if (frontDoor && typeof frontDoor.code === "string") {
    return {
      ok: false,
      httpStatus,
      stub: false,
      error: {
        code: frontDoor.code,
        message:
          typeof frontDoor.detail === "string"
            ? frontDoor.detail
            : typeof frontDoor.message === "string"
              ? frontDoor.message
              : "daemon error",
        httpStatus,
      },
    };
  }

  if (httpStatus >= 400) {
    return {
      ok: false,
      httpStatus,
      stub: false,
      error: {
        code: `HTTP_${httpStatus}`,
        message: `HTTP ${httpStatus}`,
        httpStatus,
      },
    };
  }

  return { ok: true, httpStatus, data: body, stub: false };
}

/*
 * Route whitelist (docs/design/28 §10 R-1). The client only ever calls these
 * routes; anything else would hit the 200-stub fallthrough. Paths are exact
 * or prefix-with-boundary (trailing "/" or "?" allowed after the prefix).
 */
export interface KnownRoute {
  method: "GET" | "POST";
  path: string;
  channel: "management" | "task" | "none";
}

export const KNOWN_ROUTES: readonly KnownRoute[] = [
  { method: "POST", path: "/local/session", channel: "none" },
  { method: "GET", path: "/personal/health", channel: "none" },
  { method: "GET", path: "/personal/status", channel: "management" },
  { method: "GET", path: "/personal/readiness", channel: "management" },
  { method: "GET", path: "/personal/doctor", channel: "management" },
  { method: "GET", path: "/personal/dsh/runtime", channel: "management" },
  { method: "POST", path: "/personal/dsh/runtime", channel: "management" },
  { method: "GET", path: "/provider/v1/selected-model", channel: "management" },
  { method: "GET", path: "/provider/v1/dsh/selected-model", channel: "management" },
  { method: "GET", path: "/management/providers/accounts", channel: "management" },
  { method: "POST", path: "/management/providers/accounts", channel: "management" },
  { method: "GET", path: "/management/providers/accounts/inspect", channel: "management" },
  { method: "POST", path: "/management/providers/accounts/update", channel: "management" },
  { method: "POST", path: "/management/providers/accounts/delete", channel: "management" },
  { method: "POST", path: "/management/providers/accounts/key", channel: "management" },
  { method: "GET", path: "/management/providers/models", channel: "management" },
  { method: "POST", path: "/management/providers/models/refresh", channel: "management" },
  { method: "POST", path: "/management/providers/models/add", channel: "management" },
  { method: "POST", path: "/management/providers/models/set-price", channel: "management" },
  { method: "GET", path: "/management/agent-bindings", channel: "management" },
  { method: "POST", path: "/management/agent-bindings", channel: "management" },
  { method: "POST", path: "/management/agent-bindings/remove", channel: "management" },
  { method: "GET", path: "/management/usage", channel: "management" },
  { method: "GET", path: "/management/budgets", channel: "management" },
  { method: "POST", path: "/management/budgets", channel: "management" },
  { method: "POST", path: "/management/budgets/remove", channel: "management" },
  { method: "GET", path: "/management/alerts", channel: "management" },
  { method: "POST", path: "/management/alerts/acknowledge", channel: "management" },
  { method: "GET", path: "/management/audit", channel: "management" },
  { method: "GET", path: "/management/project/v1/list", channel: "management" },
  { method: "GET", path: "/management/project/v1/detail", channel: "management" },
  { method: "GET", path: "/management/project/v1/axis", channel: "management" },
  { method: "GET", path: "/management/project/v1/roster", channel: "management" },
  { method: "POST", path: "/management/project/v1/draft.create", channel: "management" },
  { method: "POST", path: "/management/project/v1/preview.request", channel: "management" },
  { method: "POST", path: "/management/project/v1/confirm", channel: "management" },
  { method: "GET", path: "/management/project/v1/pending-previews", channel: "management" },
  { method: "GET", path: "/management/project/v1/vault.index", channel: "management" },
  { method: "GET", path: "/management/project/v1/standing-policies", channel: "management" },
  { method: "GET", path: "/management/resource/v1/list", channel: "management" },
  { method: "GET", path: "/management/resource/v1/inspect", channel: "management" },
  { method: "POST", path: "/management/resource/v1/bind", channel: "management" },
  { method: "POST", path: "/management/resource/v1/unbind", channel: "management" },
  { method: "POST", path: "/management/resource/v1/enable", channel: "management" },
  { method: "POST", path: "/management/resource/v1/disable", channel: "management" },
  { method: "POST", path: "/management/resource/v1/revoke", channel: "management" },
  { method: "GET", path: "/management/resource/v1/memory/object", channel: "management" },
  { method: "POST", path: "/management/resource/v1/memory/remember", channel: "management" },
  { method: "POST", path: "/management/resource/v1/memory/forget", channel: "management" },
  { method: "GET", path: "/management/resource/v1/skill/binding/explain", channel: "management" },
  { method: "POST", path: "/management/resource/v1/skill/import", channel: "management" },
  { method: "POST", path: "/management/resource/v1/skill/bind", channel: "management" },
  { method: "POST", path: "/management/resource/v1/skill/binding/revoke", channel: "management" },
  { method: "GET", path: "/management/resource/v1/tool", channel: "management" },
  { method: "GET", path: "/management/resource/v1/tool/discover", channel: "management" },
  { method: "GET", path: "/management/resource/v1/tool/exposure", channel: "management" },
  { method: "POST", path: "/management/resource/v1/tool/enable", channel: "management" },
  { method: "POST", path: "/management/resource/v1/tool/disable", channel: "management" },
  { method: "POST", path: "/management/resource/v1/tool/quarantine", channel: "management" },
  { method: "POST", path: "/management/resource/v1/tool/revoke", channel: "management" },
  { method: "POST", path: "/management/resource/v1/backup", channel: "management" },
  { method: "POST", path: "/management/resource/v1/backup/preflight", channel: "management" },
  { method: "POST", path: "/management/resource/v1/restore", channel: "management" },
  { method: "GET", path: "/resource/v1/projection", channel: "management" },
  { method: "GET", path: "/resource/v1/watch", channel: "management" },
  { method: "POST", path: "/task/intent.record", channel: "task" },
  { method: "POST", path: "/task/intent.interpret", channel: "task" },
  { method: "POST", path: "/task/preview", channel: "task" },
  { method: "POST", path: "/task/admit", channel: "task" },
  { method: "POST", path: "/task/candidate", channel: "task" },
  { method: "GET", path: "/task/watch", channel: "task" },
  { method: "GET", path: "/task/evidence", channel: "task" },
  { method: "GET", path: "/task/effects", channel: "task" },
  { method: "GET", path: "/task/observation", channel: "task" },
  { method: "POST", path: "/task/resource/v1/consumption", channel: "task" },
  { method: "GET", path: "/task/resource/v1/consumption", channel: "task" },
  { method: "GET", path: "/task/resource/v1/projection", channel: "task" },
  { method: "GET", path: "/task/resource/v1/watch", channel: "task" },
  { method: "GET", path: "/task/resource/v1/tool", channel: "task" },
  { method: "GET", path: "/task/resource/v1/tool/exposure", channel: "task" },
  { method: "POST", path: "/task/resource/v1/tool/selection", channel: "task" },
] as const;

/** Strip the query string and check the path against the whitelist. */
export function isKnownRoute(method: string, pathWithQuery: string): boolean {
  const path = pathWithQuery.split("?")[0];
  const upper = method.toUpperCase();
  return KNOWN_ROUTES.some((route) => route.method === upper && route.path === path);
}
