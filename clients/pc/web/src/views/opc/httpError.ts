export function httpErrorMessage(status: number, body: unknown): string {
  if (body && typeof body === "object") {
    const record = body as Record<string, unknown>;
    const nested = record.error;
    if (nested && typeof nested === "object") {
      const error = nested as Record<string, unknown>;
      const code = typeof error.code === "string" ? error.code : "error";
      const message = typeof error.message === "string" ? error.message : "";
      return `HTTP ${status} · ${code}${message ? ` — ${message}` : ""}`;
    }
    if (typeof record.code === "string") {
      const message = typeof record.message === "string" ? record.message : "";
      return `HTTP ${status} · ${record.code}${message ? ` — ${message}` : ""}`;
    }
  }
  return `HTTP ${status}`;
}

export function jsonStringList(body: unknown, key: string): string[] {
  if (!body || typeof body !== "object") {
    return [];
  }
  const value = (body as Record<string, unknown>)[key];
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string" && item.length > 0);
}
