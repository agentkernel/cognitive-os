import { useContext, useEffect } from "react";
import { Link } from "react-router-dom";
import { fetchProjection } from "../data/fetchProjection";
import { projectAlerts, projectReadiness, type ReadinessView } from "../data/projections";
import { appProjections } from "../data/store";
import { useProjection } from "../data/useProjection";
import { sessionHasChannel, sessionPrincipal } from "../session";
import { StateDot } from "../state/StateDot";
import { readDomainState } from "../state/stateMap";
import { SessionTick } from "./SessionScope";

const READINESS_KEY = "strip:readiness";
const ALERTS_KEY = "strip:alerts";

type DaemonHealth = "unknown" | "ready" | "unreachable";

/**
 * Status strip — the instrument bezel. One line of global truth: daemon
 * reachability, overall readiness, session identity, unacknowledged alerts.
 * Every cell is real data or an honest absence; nothing is decorative.
 */
export function StatusStrip() {
  const { tick } = useContext(SessionTick);
  const health = useProjection<{ status: string }>("strip:health");
  const readiness = useProjection<ReadinessView>(READINESS_KEY);
  const alerts = useProjection<{ alerts: unknown[]; unacknowledged: number }>(ALERTS_KEY);
  const hasManagement = sessionHasChannel("management");

  useEffect(() => {
    void (async () => {
      // Daemon liveness is unauthenticated loopback by design.
      try {
        const response = await fetch("/personal/health", { credentials: "omit" });
        appProjections.set("strip:health", {
          status: "ready",
          data: { status: response.ok ? "ok" : `HTTP ${response.status}` },
          source: "/personal/health",
          updatedAt: Date.now(),
        });
      } catch {
        appProjections.set("strip:health", {
          status: "disconnected",
          data: { status: "unreachable" },
          source: "/personal/health",
          updatedAt: Date.now(),
        });
      }
    })();
  }, [tick]);

  useEffect(() => {
    if (!hasManagement) {
      return;
    }
    void fetchProjection(
      appProjections,
      READINESS_KEY,
      "/personal/status",
      "management",
      projectReadiness,
    );
    void fetchProjection(
      appProjections,
      ALERTS_KEY,
      "/management/alerts",
      "management",
      projectAlerts,
    );
  }, [tick, hasManagement]);

  const daemonHealth: DaemonHealth =
    health.status === "ready" && health.data?.status === "ok"
      ? "ready"
      : health.status === "disconnected"
        ? "unreachable"
        : health.status === "ready"
          ? "ready"
          : "unknown";
  const daemonReading =
    daemonHealth === "ready"
      ? { category: "ready" as const, label: "daemon ready", unmapped: false }
      : daemonHealth === "unreachable"
        ? { category: "blocked" as const, label: "daemon unreachable", unmapped: false }
        : { category: "unknown" as const, label: "daemon …", unmapped: false };

  const overallReading = readiness.data
    ? readDomainState("readiness", readiness.data.overall)
    : undefined;
  const alertCount = alerts.data?.unacknowledged ?? 0;

  return (
    <div className="cp-strip" role="contentinfo" aria-label="System status">
      <span className="cp-strip-cell" aria-disabled="true" title={health.data?.status ?? ""}>
        <StateDot category={daemonReading.category} />
        <span>{daemonReading.label}</span>
      </span>
      <span className="cp-strip-cell" aria-disabled="true">
        {hasManagement ? (
          overallReading ? (
            <>
              <StateDot category={overallReading.category} />
              <span>
                readiness {overallReading.label}
                {overallReading.unmapped ? " (unmapped)" : ""}
              </span>
            </>
          ) : (
            <span>readiness …</span>
          )
        ) : (
          <span className="cp-quiet">readiness: session required</span>
        )}
      </span>
      <span className="cp-strip-spacer" />
      {hasManagement ? (
        <Link
          className="cp-strip-cell"
          to="/activity"
          aria-label={`${alertCount} unacknowledged alerts; open Activity`}
        >
          <StateDot category={alertCount > 0 ? "attention" : "ready"} />
          <span>{alertCount} alerts</span>
        </Link>
      ) : null}
      <Link
        className="cp-strip-cell"
        to="/session"
        aria-label="Session state; open Session"
        title="Sessions are memory-only (BD-7: no daemon introspection route)"
      >
        <StateDot category={hasManagement ? "ready" : "unknown"} />
        <span>
          {sessionPrincipal()} · {hasManagement ? "mgmt" : "no mgmt"}
          {sessionHasChannel("task") ? "+task" : ""}
        </span>
      </Link>
    </div>
  );
}
