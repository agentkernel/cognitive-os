/*
 * Typed view models + coercion. Domain model ≠ UI model: backend bodies are
 * validated/normalized/projected here before any view sees them.
 *
 * Secret discipline: `secret_ref` values are never carried into a view model
 * — only presence (present/absent/unknown). redactSecrets in api.ts already
 * strips secret-shaped values at the boundary; this layer enforces the
 * display contract on top.
 */

export function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

export function asList(value: unknown, keys: string[]): unknown[] {
  const record = asRecord(value);
  for (const key of keys) {
    if (Array.isArray(record[key])) {
      return record[key] as unknown[];
    }
  }
  return [];
}

export type SecretPresence = "present" | "absent" | "unknown";

/**
 * Presence-only reading of a secret reference. "unknown" when the field is
 * absent from the payload (the daemon didn't say), "absent" when explicitly
 * empty, "present" otherwise. The value itself is never returned.
 */
export function secretPresenceOf(record: Record<string, unknown>): SecretPresence {
  if (!("secret_ref" in record)) {
    return "unknown";
  }
  const value = record.secret_ref;
  if (value == null || value === "" || value === "absent") {
    return "absent";
  }
  return "present";
}

/* ---------- readiness / status (readiness.rs status projection) ---------- */

export interface ReadinessComponentView {
  name: string;
  state: string;
  detail?: string;
}

export interface ReadinessView {
  overall: string;
  firstConversationReady?: boolean;
  components: ReadinessComponentView[];
}

export function projectReadiness(body: unknown): ReadinessView {
  const record = asRecord(body);
  const componentsRaw = Array.isArray(record.components) ? record.components : [];
  const components: ReadinessComponentView[] = componentsRaw.map((item) => {
    const component = asRecord(item);
    return {
      name: String(component.name ?? component.component ?? "unknown"),
      state: String(component.status ?? component.state ?? "unknown"),
      detail:
        typeof component.message === "string"
          ? component.message
          : typeof component.detail === "string"
            ? component.detail
            : undefined,
    };
  });
  const overall =
    typeof record.overall === "string"
      ? record.overall
      : typeof asRecord(record.readiness).overall === "string"
        ? String(asRecord(record.readiness).overall)
        : "unknown";
  const first =
    typeof record.first_conversation_ready === "boolean"
      ? record.first_conversation_ready
      : undefined;
  return { overall, firstConversationReady: first, components };
}

/* ---------- alerts (provider control plane) ---------- */

export interface AlertView {
  id: string;
  threshold: string;
  acknowledged: boolean;
}

export interface AlertsView {
  alerts: AlertView[];
  unacknowledged: number;
}

export function projectAlerts(body: unknown): AlertsView {
  const rows = asList(body, ["alerts", "events", "items"]).map(asRecord);
  const alerts = rows.map((row) => ({
    id: String(row.alert_id ?? row.id ?? "unknown"),
    threshold: String(row.threshold_kind ?? row.kind ?? "unknown"),
    acknowledged: row.acknowledged_at_ms != null || row.acknowledged === true,
  }));
  return {
    alerts,
    unacknowledged: alerts.filter((alert) => !alert.acknowledged).length,
  };
}

/* ---------- provider accounts (W2 consumes; secret discipline enforced) ---------- */

export interface ProviderAccountView {
  id: string;
  name: string;
  kind: string;
  status: string;
  secret: SecretPresence;
  catalogRevision?: string;
  networkScope?: string;
  lastDiscoveryError?: string;
}

export function projectProviderAccount(row: unknown): ProviderAccountView {
  const record = asRecord(row);
  return {
    id: String(record.id ?? "unknown"),
    name: String(record.display_name ?? record.id ?? "unknown"),
    kind: String(record.provider_kind ?? "unknown"),
    status: String(record.status ?? "unknown"),
    secret: secretPresenceOf(record),
    catalogRevision:
      record.catalog_revision == null ? undefined : String(record.catalog_revision),
    networkScope:
      record.network_scope == null ? undefined : String(record.network_scope),
    lastDiscoveryError:
      typeof record.last_discovery_error === "string" && record.last_discovery_error !== ""
        ? record.last_discovery_error
        : undefined,
  };
}

export function projectProviderAccounts(body: unknown): ProviderAccountView[] {
  return asList(body, ["accounts", "items"]).map(projectProviderAccount);
}
