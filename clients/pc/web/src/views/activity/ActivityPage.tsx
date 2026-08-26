import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../../api";
import { FactGrid } from "../../components/FactGrid";
import { Inspector } from "../../components/Inspector";
import { PageHeader } from "../../components/PageHeader";
import { ReceiptLine } from "../../components/ReceiptLine";
import { EmptyState } from "../../components/states";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord, projectAlerts } from "../../data/projections";
import {
  ACTIVITY_ALERTS_KEY,
  ACTIVITY_AUDIT_KEY,
  ACTIVITY_COVERAGE,
  ACTIVITY_KINDS,
  ACTIVITY_KIND_LABEL,
  ACTIVITY_OBJECT_TYPES,
  ACTIVITY_ROW_CAP,
  ACTIVITY_SINCE_NOTE,
  ACTIVITY_TASK_PROBE_LIMIT,
  STRIP_ALERTS_KEY,
  activityEffectsKey,
  activityEvidenceKey,
  activityObjectHref,
  boundActivityRows,
  composeActivity,
  filterActivityRows,
  namedSourceFailure,
  probeObservedRefs,
  type ActivityKindFilter,
  type ActivityObjectFilter,
  type ActivityRow,
  type ActivitySinceFilter,
} from "../../data/projections/activity";
import {
  formatAge,
  OBSERVED_TASKS_KEY,
  SESSION_RECEIPTS_KEY,
  projectTaskEffects,
  projectTaskEvidence,
  recordSessionMutation,
  type EffectEntryView,
  type ObservedTask,
  type SessionMutationReceipt,
  type TaskEffectSummary,
  type TaskEvidenceView,
} from "../../data/projections/home";
import {
  projectAuditEvents,
  projectProviderAlerts,
  type AuditEventView,
  type ProviderAlertView,
} from "../../data/projections/providers";
import { appProjections } from "../../data/store";
import { useLastGood, useProjection, useProjections } from "../../data/useProjection";
import { sessionHasChannel } from "../../session";
import { useInspectorClear } from "../../shell/useInspectorClear";
import { HonestyNote } from "../../state/HonestyNote";
import { StateDot } from "../../state/StateDot";
import { ProjectionState } from "../providers/ProjectionState";

const NO_OBSERVED: ObservedTask[] = [];
const NO_RECEIPTS: SessionMutationReceipt[] = [];

/**
 * Activity — docs/design/19. Time-ordered evidence stream over the sources
 * that actually exist. Home is the attention queue; this page is
 * investigation. Alert acknowledge is class-B and the receipt stays.
 */
export function ActivityPage() {
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [kind, setKind] = useState<ActivityKindFilter>("all");
  const [objectType, setObjectType] = useState<ActivityObjectFilter>("all");
  const [since, setSince] = useState<ActivitySinceFilter>("all");
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();
  const [message, setMessage] = useState<string | undefined>();
  const clearInspector = useCallback(() => setSelectedId(undefined), []);
  useInspectorClear(selectedId, clearInspector);

  const alerts = useProjection<ProviderAlertView[]>(ACTIVITY_ALERTS_KEY);
  const audit = useProjection<AuditEventView[]>(ACTIVITY_AUDIT_KEY);
  const observedProjection = useProjection<ObservedTask[]>(OBSERVED_TASKS_KEY);
  const receiptsProjection = useProjection<SessionMutationReceipt[]>(SESSION_RECEIPTS_KEY);
  const alertsGood = useLastGood(alerts);
  const auditGood = useLastGood(audit);
  const observed = observedProjection.data ?? NO_OBSERVED;
  const receipts = receiptsProjection.data ?? NO_RECEIPTS;
  const taskChannel = sessionHasChannel("task");

  const probed = useMemo(() => probeObservedRefs(observed), [observed]);
  const probedKey = probed.refs.join("|");

  const refreshManagement = useCallback(async () => {
    await Promise.all([
      fetchProjection(
        appProjections,
        ACTIVITY_ALERTS_KEY,
        "/management/alerts",
        "management",
        projectProviderAlerts,
      ),
      fetchProjection(
        appProjections,
        ACTIVITY_AUDIT_KEY,
        "/management/audit",
        "management",
        projectAuditEvents,
      ),
    ]);
  }, []);

  const refreshTaskProbes = useCallback(async (refs: string[]) => {
    if (!sessionHasChannel("task") || refs.length === 0) {
      return;
    }
    for (const taskRef of refs) {
      const encoded = encodeURIComponent(taskRef);
      await fetchProjection(
        appProjections,
        activityEffectsKey(taskRef),
        `/task/effects?task_ref=${encoded}`,
        "task",
        projectTaskEffects,
      );
      await fetchProjection(
        appProjections,
        activityEvidenceKey(taskRef),
        `/task/evidence?task_ref=${encoded}`,
        "task",
        projectTaskEvidence,
      );
    }
  }, []);

  useEffect(() => {
    void refreshManagement();
  }, [refreshManagement]);

  useEffect(() => {
    void refreshTaskProbes(probedKey === "" ? [] : probedKey.split("|"));
  }, [probedKey, refreshTaskProbes, taskChannel]);

  const effectProjections = useProjections<EffectEntryView[]>(
    probed.refs.map(activityEffectsKey),
  );
  const evidenceProjections = useProjections<TaskEvidenceView>(
    probed.refs.map(activityEvidenceKey),
  );

  const effectSummaries: TaskEffectSummary[] = probed.refs.flatMap((taskRef, index) => {
    const data = effectProjections[index]?.data;
    return data ? [{ taskRef, effects: data }] : [];
  });
  const evidenceEntries = probed.refs.flatMap((taskRef, index) => {
    const view = evidenceProjections[index]?.data;
    return view ? [{ taskRef, view }] : [];
  });

  const rows = useMemo(
    () =>
      composeActivity({
        alerts: alertsGood.data ?? [],
        auditEvents: auditGood.data ?? [],
        receipts,
        observed,
        effects: effectSummaries,
        evidence: evidenceEntries,
      }),
    [alertsGood.data, auditGood.data, receipts, observed, effectSummaries, evidenceEntries],
  );

  const filtered = useMemo(
    () => filterActivityRows(rows, { kind, objectType, since, nowMs }),
    [rows, kind, objectType, since, nowMs],
  );
  const bounded = boundActivityRows(filtered);
  const selected = bounded.shown.find((row) => row.id === selectedId);

  useEffect(() => {
    if (selectedId && !bounded.shown.some((row) => row.id === selectedId)) {
      setSelectedId(undefined);
    }
  }, [bounded.shown, selectedId]);

  const sourceFailures = [
    namedSourceFailure(alerts, "budget alerts"),
    namedSourceFailure(audit, "provider-plane audit"),
    ...probed.refs.flatMap((taskRef, index) => {
      if (!taskChannel) {
        return [];
      }
      return [
        namedSourceFailure(effectProjections[index] ?? { status: "loading" }, `task effects ${taskRef}`),
        namedSourceFailure(
          evidenceProjections[index] ?? { status: "loading" },
          `task evidence ${taskRef}`,
        ),
      ];
    }),
  ].filter((line): line is string => line != null);

  const refreshAll = useCallback(() => {
    setNowMs(Date.now());
    void (async () => {
      await refreshManagement();
      await refreshTaskProbes(probeObservedRefs(appProjections.get<ObservedTask[]>(OBSERVED_TASKS_KEY)?.data ?? []).refs);
    })();
  }, [refreshManagement, refreshTaskProbes]);

  async function acknowledge(alertId: string) {
    setReceipt(undefined);
    setMessage(undefined);
    const result = await readJson("/management/alerts/acknowledge", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ alert_id: alertId }),
    });
    if (result.ok) {
      setReceipt(`Alert ${alertId} acknowledged.`);
    } else {
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
    }
    recordSessionMutation(appProjections, {
      id: `alert.acknowledge:${alertId}`,
      action: "alert.acknowledge",
      objectRef: alertId,
      atMs: Date.now(),
      detail: "budget alert acknowledged from Activity",
    });
    await fetchProjection(
      appProjections,
      ACTIVITY_ALERTS_KEY,
      "/management/alerts",
      "management",
      projectProviderAlerts,
    );
    await fetchProjection(
      appProjections,
      STRIP_ALERTS_KEY,
      "/management/alerts",
      "management",
      projectAlerts,
    );
  }

  const alertsPending = alerts.status === "loading" && alerts.data === undefined;
  const auditPending = audit.status === "loading" && audit.data === undefined;
  const sourcesAnswered = !alertsPending && !auditPending;
  const empty =
    sourcesAnswered &&
    bounded.total === 0 &&
    sourceFailures.length === 0 &&
    kind === "all" &&
    objectType === "all" &&
    since === "all";

  return (
    <section>
      <PageHeader
        title="Activity"
        lede="Time-ordered evidence stream. Home is what needs you; this page is what happened in the sources this client can actually read."
      />
      <div data-annotation="activity-coverage">
        <HonestyNote>{ACTIVITY_COVERAGE}</HonestyNote>
        {sourceFailures.map((failure) => (
          <p className="cp-reason" role="status" key={failure}>
            {failure}
          </p>
        ))}
        {!taskChannel ? (
          <p className="cp-reason" role="status">
            Task evidence/effects not-run — this page holds a management session only. Observed refs
            still appear; nothing is inferred for evidence.
          </p>
        ) : null}
        {probed.truncated ? (
          <p className="cp-quiet">
            Task probes bounded to {ACTIVITY_TASK_PROBE_LIMIT} of {probed.total} session-observed
            refs.
          </p>
        ) : null}
      </div>
      <p className="cp-next">
        <button type="button" className="cp-button" onClick={refreshAll}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">This space refreshes only when you ask.</span>
      </p>
      <ProjectionState projection={alerts} what="Budget alerts" />
      <ProjectionState projection={audit} what="Provider-plane audit" />
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
      {message ? (
        <p role="alert" className="cp-reason">
          {message}
        </p>
      ) : null}
      <div className="cp-filters">
        <label className="cp-field">
          <span>Kind</span>
          <select
            value={kind}
            onChange={(event) => setKind(event.target.value as ActivityKindFilter)}
            aria-label="Filter by kind"
          >
            <option value="all">All kinds</option>
            {ACTIVITY_KINDS.map((value) => (
              <option key={value} value={value}>
                {ACTIVITY_KIND_LABEL[value]}
              </option>
            ))}
          </select>
        </label>
        <label className="cp-field">
          <span>Object</span>
          <select
            value={objectType}
            onChange={(event) => setObjectType(event.target.value as ActivityObjectFilter)}
            aria-label="Filter by object"
          >
            <option value="all">All objects</option>
            {ACTIVITY_OBJECT_TYPES.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
        <label className="cp-field">
          <span>Since</span>
          <select
            value={since}
            onChange={(event) => setSince(event.target.value as ActivitySinceFilter)}
            aria-label="Filter by time"
          >
            <option value="all">Any time</option>
            <option value="hour">Last hour</option>
            <option value="day">Last day</option>
          </select>
        </label>
      </div>
      {since !== "all" ? <p className="cp-quiet">{ACTIVITY_SINCE_NOTE}</p> : null}
      {bounded.truncated ? (
        <p className="cp-quiet">
          showing {bounded.shown.length} of {bounded.total} (bounded window of {ACTIVITY_ROW_CAP})
        </p>
      ) : null}
      <div className="cp-mi">
        <div className="cp-master">
          {empty ? (
            <EmptyState title="Nothing recorded in this view yet">{ACTIVITY_COVERAGE}</EmptyState>
          ) : bounded.shown.length > 0 ? (
            <ol className="cp-queue" aria-label="Activity stream">
              {bounded.shown.map((row) => (
                <ActivityRowItem
                  key={row.id}
                  row={row}
                  selected={row.id === selectedId}
                  nowMs={nowMs}
                  onSelect={() => setSelectedId(row.id)}
                  onAcknowledge={(alertId) => void acknowledge(alertId)}
                />
              ))}
            </ol>
          ) : sourcesAnswered && sourceFailures.length === 0 ? (
            <p className="cp-quiet">
              No rows match the current filters from the sources that answered.
            </p>
          ) : null}
        </div>
        <ActivityInspector row={selected} nowMs={nowMs} />
      </div>
    </section>
  );
}

function ActivityRowItem({
  row,
  selected,
  nowMs,
  onSelect,
  onAcknowledge,
}: {
  row: ActivityRow;
  selected: boolean;
  nowMs: number;
  onSelect: () => void;
  onAcknowledge: (alertId: string) => void;
}) {
  const href = activityObjectHref(row);
  const age =
    formatAge(row.atMs, nowMs) ??
    `age unknown${row.ageUnknownReason ? ` (${row.ageUnknownReason})` : ""}`;
  return (
    <li
      className="cp-queue-row"
      data-row-key={row.id}
      data-kind={row.kind}
      data-object={row.objectType}
      aria-selected={selected}
    >
      <span className="cp-activity-kind">
        <StateDot category={row.reading.category} />
        {ACTIVITY_KIND_LABEL[row.kind]}
      </span>
      <span className="cp-queue-object">
        <span className="cp-quiet">{row.objectType}</span>{" "}
        <code className="cp-mono" title={row.objectRef ?? row.objectLabel}>
          {row.objectLabel}
        </code>
      </span>
      <span className="cp-queue-reason">{row.fact}</span>
      <span className="cp-quiet cp-queue-age">{age}</span>
      <span className="cp-queue-action">
        <button type="button" className="cp-button" onClick={onSelect}>
          Inspect
        </button>
        {row.alertId ? (
          <button type="button" className="cp-button" onClick={() => onAcknowledge(row.alertId ?? "")}>
            Acknowledge
          </button>
        ) : null}
        {href ? <Link to={href}>Open</Link> : null}
      </span>
    </li>
  );
}

function ActivityInspector({ row, nowMs }: { row?: ActivityRow; nowMs: number }) {
  if (!row) {
    return (
      <Inspector title="Activity">
        <p className="cp-quiet">Select a row to inspect it. Nothing is inferred.</p>
      </Inspector>
    );
  }
  const href = activityObjectHref(row);
  return (
    <Inspector title={ACTIVITY_KIND_LABEL[row.kind]}>
      <FactGrid
        facts={[
          { label: "kind", value: ACTIVITY_KIND_LABEL[row.kind] },
          { label: "object", value: row.objectType },
          { label: "identity", value: row.objectRef ?? row.objectLabel },
          { label: "source", value: row.source },
          {
            label: "time",
            value:
              formatAge(row.atMs, nowMs) ??
              `unknown${row.ageUnknownReason ? ` (${row.ageUnknownReason})` : ""}`,
          },
          { label: "fact", value: row.fact },
        ]}
      />
      {href ? (
        <p className="cp-next">
          <Link to={href}>Open object</Link>
        </p>
      ) : null}
    </Inspector>
  );
}
