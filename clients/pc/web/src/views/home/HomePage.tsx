import { Component, useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  composeAttention,
  formatAge,
  mergeCurrentWork,
  projectHomeReadiness,
  projectTaskEffects,
  projectTaskEnvelopes,
  projectTaskEvidence,
  recordSessionMutation,
  shortTaskRef,
  type AttentionItem,
  type EffectEntryView,
  type EvidenceRow,
  type HomeReadinessView,
  type ObservedTask,
  type SessionMutationReceipt,
  type TaskEffectSummary,
  type TaskEnvelopeView,
  type TaskEvidenceView,
  OBSERVED_TASKS_KEY,
  SESSION_RECEIPTS_KEY,
} from "../../data/projections/home";
import {
  projectAuditEvents,
  projectBindings,
  projectProviderAccounts,
  projectProviderAlerts,
  type AuditEventView,
  type BindingView,
  type ProviderAccount,
  type ProviderAlertView,
} from "../../data/projections/providers";
import { appProjections } from "../../data/store";
import { useLastGood, useProjection, useProjections } from "../../data/useProjection";
import { sessionHasChannel } from "../../session";
import { HonestyNote } from "../../state/HonestyNote";
import { AttentionSection, type AttentionSource } from "./AttentionSection";
import { CurrentWorkSection } from "./CurrentWorkSection";
import { ReadinessSection } from "./ReadinessSection";
import { RecentEvidenceSection } from "./RecentEvidenceSection";

export const HOME_READINESS_KEY = "home:readiness";
export const HOME_ACCOUNTS_KEY = "home:accounts";
export const HOME_BINDINGS_KEY = "home:bindings";
export const HOME_ALERTS_KEY = "home:alerts";
export const HOME_AUDIT_KEY = "home:audit";
export const HOME_TASKS_KEY = "home:tasks";

export const homeEffectsKey = (taskRef: string) => `home:effects:${taskRef}`;
export const homeEvidenceKey = (taskRef: string) => `home:evidence:${taskRef}`;

/** Per-task probes are bounded: Home never fans out over an unbounded list. */
export const TASK_PROBE_LIMIT = 6;

const NO_OBSERVED: ObservedTask[] = [];
const NO_RECEIPTS: SessionMutationReceipt[] = [];

/**
 * One region's render failure must not take the surface down with it. Data
 * failures are already handled per projection; this is the last line for an
 * unexpected exception inside a region subtree.
 */
class RegionBoundary extends Component<
  { name: string; children: ReactNode },
  { failed: boolean }
> {
  constructor(props: { name: string; children: ReactNode }) {
    super(props);
    this.state = { failed: false };
  }

  static getDerivedStateFromError() {
    return { failed: true };
  }

  render() {
    if (this.state.failed) {
      return (
        <section className="cp-region" aria-label={`${this.props.name} — unavailable`}>
          <h3 className="cp-section-title">{this.props.name}</h3>
          <p className="cp-reason" role="alert">
            This region failed to render and was isolated. The other regions on this page are
            unaffected; refresh to retry.
          </p>
        </section>
      );
    }
    return this.props.children;
  }
}

/**
 * Home — the attention surface (docs/design/13). It answers three questions
 * in reading order: is the system ready, what needs me, what is in flight.
 * Every element is a navigable authority fact or one action; there are no
 * KPI tiles, charts, trends or scores, and no metric about a metric.
 *
 * Wave 1 behaviour: explicit refresh only (no polling, no watch deepening),
 * one projection per verified route through the shared fetch pipeline, and
 * every region isolated so one failed source cannot blank the page.
 */
export function HomePage() {
  const [nowMs, setNowMs] = useState(() => Date.now());

  const readiness = useProjection<HomeReadinessView>(HOME_READINESS_KEY);
  const accounts = useProjection<ProviderAccount[]>(HOME_ACCOUNTS_KEY);
  const bindings = useProjection<BindingView[]>(HOME_BINDINGS_KEY);
  const alerts = useProjection<ProviderAlertView[]>(HOME_ALERTS_KEY);
  const audit = useProjection<AuditEventView[]>(HOME_AUDIT_KEY);
  const tasks = useProjection<TaskEnvelopeView[]>(HOME_TASKS_KEY);
  const observedProjection = useProjection<ObservedTask[]>(OBSERVED_TASKS_KEY);
  const receiptsProjection = useProjection<SessionMutationReceipt[]>(SESSION_RECEIPTS_KEY);

  const readinessGood = useLastGood(readiness);
  const accountsGood = useLastGood(accounts);
  const bindingsGood = useLastGood(bindings);
  const alertsGood = useLastGood(alerts);
  const auditGood = useLastGood(audit);
  const tasksGood = useLastGood(tasks);

  const observed = observedProjection.data ?? NO_OBSERVED;
  const receipts = receiptsProjection.data ?? NO_RECEIPTS;
  const taskChannel = sessionHasChannel("task");

  const refreshManagement = useCallback(async () => {
    await Promise.all([
      fetchProjection(
        appProjections,
        HOME_READINESS_KEY,
        "/personal/status",
        "management",
        projectHomeReadiness,
      ),
      fetchProjection(
        appProjections,
        HOME_ACCOUNTS_KEY,
        "/management/providers/accounts",
        "management",
        projectProviderAccounts,
      ),
      fetchProjection(
        appProjections,
        HOME_BINDINGS_KEY,
        "/management/agent-bindings",
        "management",
        projectBindings,
      ),
      fetchProjection(
        appProjections,
        HOME_ALERTS_KEY,
        "/management/alerts",
        "management",
        projectProviderAlerts,
      ),
      fetchProjection(
        appProjections,
        HOME_AUDIT_KEY,
        "/management/audit",
        "management",
        projectAuditEvents,
      ),
      fetchProjection(
        appProjections,
        HOME_TASKS_KEY,
        "/management/resource/v1/list?family=task",
        "management",
        projectTaskEnvelopes,
      ),
    ]);
  }, []);

  const refreshTaskProbes = useCallback(async (refs: string[]) => {
    // Effects and evidence exist per known task ref only — there is no task
    // stream — and they live on the Task channel. Without one, nothing is
    // fetched and nothing is inferred.
    if (!sessionHasChannel("task") || refs.length === 0) {
      return;
    }
    for (const taskRef of refs) {
      const encoded = encodeURIComponent(taskRef);
      await fetchProjection(
        appProjections,
        homeEffectsKey(taskRef),
        `/task/effects?task_ref=${encoded}`,
        "task",
        projectTaskEffects,
      );
      await fetchProjection(
        appProjections,
        homeEvidenceKey(taskRef),
        `/task/evidence?task_ref=${encoded}`,
        "task",
        projectTaskEvidence,
      );
    }
  }, []);

  useEffect(() => {
    void refreshManagement();
  }, [refreshManagement]);

  const knownRefs = useMemo(
    () =>
      mergeCurrentWork(tasksGood.data ?? [], observed)
        .map((row) => row.taskRef)
        .slice(0, TASK_PROBE_LIMIT),
    [tasksGood.data, observed],
  );
  const knownRefsKey = knownRefs.join("|");

  useEffect(() => {
    void refreshTaskProbes(knownRefsKey === "" ? [] : knownRefsKey.split("|"));
  }, [knownRefsKey, refreshTaskProbes, taskChannel]);

  const effectProjections = useProjections<EffectEntryView[]>(knownRefs.map(homeEffectsKey));
  const evidenceProjections = useProjections<TaskEvidenceView>(knownRefs.map(homeEvidenceKey));

  const effectSummaries: TaskEffectSummary[] = knownRefs.flatMap((taskRef, index) => {
    const data = effectProjections[index]?.data;
    return data ? [{ taskRef, effects: data }] : [];
  });

  const evidenceEntries: EvidenceRow[] = knownRefs.flatMap((taskRef, index) => {
    const projection = evidenceProjections[index];
    const view = projection?.data;
    return view ? [{ taskRef, shortRef: shortTaskRef(taskRef), view }] : [];
  });

  const evidenceFailures = knownRefs.flatMap((taskRef, index) => {
    const projection = evidenceProjections[index];
    if (!projection || projection.data !== undefined) {
      return [];
    }
    if (projection.status === "loading" || projection.status === "stale") {
      return [];
    }
    return [{ taskRef, code: projection.error?.code ?? projection.status }];
  });

  const mergedWork = mergeCurrentWork(tasksGood.data ?? [], observed);

  const items: AttentionItem[] = composeAttention({
    readiness: readinessGood.data,
    accounts: accountsGood.data ?? [],
    bindings: bindingsGood.data ?? [],
    alerts: alertsGood.data ?? [],
    auditEvents: auditGood.data ?? [],
    receipts,
    effects: effectSummaries,
    // No watch stream is attached on Home in this wave; Current work states
    // the absence rather than inventing a watch-derived row.
    watchState: undefined,
    providersAuthoritativelyEmpty: accounts.status === "empty",
    workAuthoritativelyEmpty: tasksGood.live && mergedWork.length === 0,
  });

  const attentionSources: AttentionSource[] = [
    { what: "provider accounts", projection: accounts, lastGood: accountsGood },
    { what: "agent bindings", projection: bindings, lastGood: bindingsGood },
    { what: "budget alerts", projection: alerts, lastGood: alertsGood },
    { what: "the provider-plane audit", projection: audit, lastGood: auditGood },
  ];

  const refreshAll = useCallback(() => {
    setNowMs(Date.now());
    void (async () => {
      await refreshManagement();
      const refs = mergeCurrentWork(
        appProjections.get<TaskEnvelopeView[]>(HOME_TASKS_KEY)?.data ?? [],
        appProjections.get<ObservedTask[]>(OBSERVED_TASKS_KEY)?.data ?? [],
      )
        .map((row) => row.taskRef)
        .slice(0, TASK_PROBE_LIMIT);
      await refreshTaskProbes(refs);
    })();
  }, [refreshManagement, refreshTaskProbes]);

  const onAcknowledged = useCallback((alertId: string) => {
    recordSessionMutation(appProjections, {
      id: `alert.acknowledge:${alertId}`,
      action: "alert.acknowledge",
      objectRef: alertId,
      atMs: Date.now(),
      detail: "budget alert acknowledged from Home",
    });
    void fetchProjection(
      appProjections,
      HOME_ALERTS_KEY,
      "/management/alerts",
      "management",
      projectProviderAlerts,
    );
  }, []);

  const lastCheckedLabel = formatAge(readinessGood.data?.evaluatedAtMs, nowMs);

  return (
    <>
      <PageHeader
        title="Home"
        lede="Is the system ready, what needs you, and what is in flight. Every row is an authority fact you can open — no scores, no charts, no metrics about metrics."
      />
      <p className="cp-next">
        <button type="button" className="cp-button" onClick={refreshAll}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">
          This wave refreshes only when you ask. Nothing on this page polls the daemon.
        </span>
      </p>
      <RegionBoundary name="Readiness">
        <ReadinessSection projection={readiness} lastGood={readinessGood} nowMs={nowMs} />
      </RegionBoundary>
      <RegionBoundary name="Needs attention">
        <AttentionSection
          items={items}
          sources={attentionSources}
          nowMs={nowMs}
          lastCheckedLabel={lastCheckedLabel}
          onAcknowledged={onAcknowledged}
        />
      </RegionBoundary>
      <RegionBoundary name="Current work">
        <CurrentWorkSection
          projection={tasks}
          lastGood={tasksGood}
          observed={observed}
          nowMs={nowMs}
        />
      </RegionBoundary>
      <RegionBoundary name="Recent evidence">
        <RecentEvidenceSection
          entries={evidenceEntries}
          probedRefs={knownRefs}
          failures={evidenceFailures}
          taskChannelAvailable={taskChannel}
          nowMs={nowMs}
        />
      </RegionBoundary>
      <HonestyNote>
        Home reads six verified routes: <code>/personal/status</code>,{" "}
        <code>/management/providers/accounts</code>, <code>/management/agent-bindings</code>,{" "}
        <code>/management/alerts</code>, <code>/management/audit</code> and{" "}
        <code>/management/resource/v1/list?family=task</code>, plus{" "}
        <code>/task/effects</code> and <code>/task/evidence</code> for task refs it already
        knows (bounded to {TASK_PROBE_LIMIT}). There is no cross-object activity feed, no task
        stream and no unified audit over HTTP, so anything this page does not name here it does
        not know.
      </HonestyNote>
    </>
  );
}
