import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { PageHeader } from "../../../components/PageHeader";
import { fetchProjection } from "../../../data/fetchProjection";
import {
  OBSERVED_TASKS_KEY,
  WORK_CHAIN_KEY,
  projectTaskEnvelopes,
  type ObservedTask,
  type SessionTaskChain,
  type TaskEnvelopeView,
} from "../../../data/projections/work";
import {
  DETAIL_OBSERVATION_FAMILIES,
  DETAIL_SECTIONS,
  completionReading,
  composeAuthorityLane,
  composeObservationLane,
  composeWatchObservation,
  detailConsumptionKey,
  detailEffectsKey,
  detailEvidenceKey,
  detailObservationKey,
  isDetailSection,
  projectConsumption,
  projectEffectHistory,
  projectEvidenceDetail,
  projectObservation,
  type ConsumptionView,
  type DetailSectionId,
  type EffectHistoryView,
  type ObservationView,
  type TaskEvidenceDetail,
} from "../../../data/projections/workDetail";
import { appProjections } from "../../../data/store";
import { useProjection, useProjections } from "../../../data/useProjection";
import { HonestyNote } from "../../../state/HonestyNote";
import { WATCH_POLL_INTERVAL_MS } from "../../../watchStream";
import { ContextSection } from "./ContextSection";
import { EffectsSection } from "./EffectsSection";
import { EvidenceSection } from "./EvidenceSection";
import { FactsInspector } from "./FactsInspector";
import { IntentContractSection } from "./IntentContractSection";
import { OverviewSection } from "./OverviewSection";
import { RunTimeline } from "./RunTimeline";
import { SectionNavigator } from "./SectionNavigator";
import { useTaskWatch } from "./useTaskWatch";
import { WorkHeader } from "./WorkHeader";

const WORK_DETAIL_LIST_KEY = "detail:task-list";
const NO_CHAINS: SessionTaskChain[] = [];
const NO_OBSERVED: ObservedTask[] = [];

/**
 * Work detail (W5) — docs/design/15. One task, six continuous sections, and
 * two clearly separated timeline lanes.
 *
 * The page is deliberately not tabbed: a tab would let an operator believe
 * they had seen a task when they had seen one panel of it. Section links move
 * the viewport; every section is always in the document.
 *
 * A ref this page cannot find in any real source gets a designed object-404
 * rather than an empty shell of headings, because rendering the six sections
 * with "unknown" everywhere would look like a task that exists and has no
 * facts.
 */
export function WorkDetailPage() {
  const params = useParams();
  const taskRef = params.taskRef ?? "";
  const [query, setQuery] = useSearchParams();
  const requestedSection = query.get("section");
  const [active, setActive] = useState<DetailSectionId>(
    isDetailSection(requestedSection) ? requestedSection : "overview",
  );

  const evidenceKey = detailEvidenceKey(taskRef);
  const effectsKey = detailEffectsKey(taskRef);
  const consumptionKey = detailConsumptionKey(taskRef);
  const observationKeys = useMemo(
    () => DETAIL_OBSERVATION_FAMILIES.map((family) => detailObservationKey(taskRef, family)),
    [taskRef],
  );

  const evidence = useProjection<TaskEvidenceDetail>(evidenceKey);
  const effects = useProjection<EffectHistoryView>(effectsKey);
  const consumption = useProjection<ConsumptionView>(consumptionKey);
  const observations = useProjections<ObservationView>(observationKeys);
  const envelopes = useProjection<TaskEnvelopeView[]>(WORK_DETAIL_LIST_KEY);
  const observedProjection = useProjection<ObservedTask[]>(OBSERVED_TASKS_KEY);
  const chainProjection = useProjection<SessionTaskChain[]>(WORK_CHAIN_KEY);
  const watch = useTaskWatch(taskRef);

  const chains = chainProjection.data ?? NO_CHAINS;
  const observed = observedProjection.data ?? NO_OBSERVED;
  const chain = chains.find((entry) => entry.taskRef === taskRef);
  const observedEntry = observed.find((entry) => entry.taskRef === taskRef);

  const load = useCallback(async () => {
    if (taskRef === "") {
      return;
    }
    const encoded = encodeURIComponent(taskRef);
    await fetchProjection(
      appProjections,
      WORK_DETAIL_LIST_KEY,
      "/management/resource/v1/list?family=task",
      "management",
      projectTaskEnvelopes,
    );
    await fetchProjection(
      appProjections,
      detailEvidenceKey(taskRef),
      `/task/evidence?task_ref=${encoded}`,
      "task",
      projectEvidenceDetail,
    );
    await fetchProjection(
      appProjections,
      detailEffectsKey(taskRef),
      `/task/effects?task_ref=${encoded}`,
      "task",
      projectEffectHistory,
    );
    for (const family of DETAIL_OBSERVATION_FAMILIES) {
      await fetchProjection(
        appProjections,
        detailObservationKey(taskRef, family),
        `/task/observation?family=${family}&task_ref=${encoded}`,
        "task",
        projectObservation,
      );
    }
    await fetchProjection(
      appProjections,
      detailConsumptionKey(taskRef),
      `/task/resource/v1/consumption?task_ref=${encoded}`,
      "task",
      projectConsumption,
    );
  }, [taskRef]);

  useEffect(() => {
    void load();
  }, [load]);

  const select = useCallback(
    (section: DetailSectionId) => {
      setActive(section);
      const next = new URLSearchParams(query);
      next.set("section", section);
      setQuery(next, { replace: true });
      document.getElementById(`section-${section}`)?.scrollIntoView?.({ block: "start" });
    },
    [query, setQuery],
  );

  const completion = completionReading(evidence.data, evidence.data !== undefined);
  const authorityLane = evidence.data
    ? composeAuthorityLane(evidence.data)
    : composeAuthorityLane({
        transitions: [],
        transitionsTruncated: false,
        intentRefs: [],
        effectRefs: [],
      });
  const observationLane = [
    ...composeObservationLane(
      observations.map((projection) => projection.data).filter((view): view is ObservationView => view != null),
    ),
    ...composeWatchObservation(watch.snapshot.events, taskRef),
  ];

  /*
   * "Known" means a real source names this ref: the daemon envelope list, this
   * session's observed refs, or a successful evidence read. Anything else is an
   * object this page cannot account for.
   */
  const listAnswered = envelopes.data !== undefined;
  const inEnvelopes = (envelopes.data ?? []).some((row) => row.taskRef === taskRef);
  const evidenceAnswered = evidence.data !== undefined;
  const known = inEnvelopes || evidenceAnswered || observedEntry != null || chain != null;
  const stillReading =
    evidence.status === "loading" || envelopes.status === "loading" || !listAnswered;

  if (taskRef === "") {
    return <ObjectNotFound taskRef="(empty)" reason="No task ref was given in the route." />;
  }

  if (!known && !stillReading) {
    return (
      <ObjectNotFound
        taskRef={taskRef}
        reason="No source this page can read names this task ref: it is not in the daemon envelope list, no terminal evidence exists for it, this session never observed it, and this session did not admit it."
      />
    );
  }

  return (
    <>
      <PageHeader
        title="Task"
        lede="One governed task, from the intent that produced it to the evidence that closes it. Authority facts and observations are kept apart."
      />
      <p className="cp-next">
        <Link to={backToWork(query)}>Back to Work</Link>{" "}
        <button type="button" className="cp-button" onClick={() => void load()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">
          {watch.snapshot.phase === "attached"
            ? `Watch is attached: this page reads GET /task/watch, then a ${WATCH_POLL_INTERVAL_MS / 1000} s bounded poll while the snapshot stream is inert. Nothing else polls.`
            : "Nothing on this page polls the daemon until you attach a watch, and nothing here streams on its own."}
        </span>
      </p>

      <WorkHeader
        taskRef={taskRef}
        evidence={evidence.data}
        completion={completion}
        evidenceReadable={evidenceAnswered}
        watch={watch.snapshot}
        onAttach={() => void watch.attach()}
        onDetach={() => {
          watch.detach();
        }}
        onReconnect={() => void watch.reconnect()}
      />

      <SectionNavigator active={active} onSelect={select} />

      <div className="cp-mi">
        <div className="cp-master">
          <OverviewSection
            evidence={evidence.data}
            completion={completion}
            objective={chain?.preview.objective ?? observedEntry?.objective}
            watchLabel={watch.snapshot.label}
          />
          <RunTimeline
            authority={authorityLane}
            observation={observationLane}
            evidenceProjection={evidence}
            observationProjections={observations}
            watch={watch.snapshot}
            onAttach={() => void watch.attach()}
            onDetach={() => {
              watch.detach();
            }}
            onReconnect={() => void watch.reconnect()}
          />
          <EffectsSection view={effects.data} projection={effects} />
          <EvidenceSection
            evidence={evidence.data}
            completion={completion}
            projection={evidence}
          />
          <IntentContractSection chain={chain} evidence={evidence.data} />
          <ContextSection view={consumption.data} projection={consumption} />
        </div>
        <FactsInspector
          taskRef={taskRef}
          evidence={evidence.data}
          effects={effects.data}
          completion={completion}
          chain={chain}
          watch={watch.snapshot}
          onAttach={() => void watch.attach()}
          onDetach={() => {
            watch.detach();
          }}
          onReconnect={() => void watch.reconnect()}
        />
      </div>

      <HonestyNote>
        This page reads the task envelope list,{" "}
        <code>/task/evidence</code>, <code>/task/effects</code>, bounded{" "}
        <code>/task/observation</code> for {DETAIL_OBSERVATION_FAMILIES.join(" and ")}, and{" "}
        <code>/task/resource/v1/consumption</code>. <code>GET /task/watch</code> is opened only
        after you attach — it is a process-local ring, not a task-exclusive feed. There is no task
        detail route, no run route and no control route on this daemon, so everything above is
        composed from those reads and from this session&apos;s own memory of the chain it ran. All{" "}
        {DETAIL_SECTIONS.length} sections are always rendered; nothing is hidden behind a tab.
      </HonestyNote>
    </>
  );
}

/** Returning to Work keeps the inventory's own scope, filter and selection. */
function backToWork(query: URLSearchParams): string {
  const back = new URLSearchParams();
  for (const key of ["task", "scope", "q"] as const) {
    const value = query.get(key);
    if (value != null && value !== "") {
      back.set(key, value);
    }
  }
  const search = back.toString();
  return search === "" ? "/work" : `/work?${search}`;
}

/**
 * The designed object-404: it names the ref, says which sources were checked,
 * and offers the way back. It deliberately does not render the six sections
 * full of unknowns, which would look like a task that exists with no facts.
 */
function ObjectNotFound({ taskRef, reason }: { taskRef: string; reason: string }) {
  return (
    <>
      <PageHeader title="Task not found" lede="This page cannot account for that task ref." />
      <section className="cp-region" aria-labelledby="notfound-title">
        <h3 className="cp-section-title" id="notfound-title">
          No such task on this daemon
        </h3>
        <p className="cp-reason" role="alert">
          <code className="cp-mono">{taskRef}</code> — {reason}
        </p>
        <p className="cp-quiet">
          That is not a claim that the task never existed: the daemon exposes no task search, so a
          ref this page has never loaded is simply unknown to it.
        </p>
        <p className="cp-next">
          <Link className="cp-button cp-button--primary" to="/work">
            Back to Work
          </Link>
        </p>
      </section>
      <HonestyNote>
        No detail is fabricated for an unknown ref. Rendering empty sections would imply the task
        exists and has no facts, which is a different and unsupported claim.
      </HonestyNote>
    </>
  );
}
