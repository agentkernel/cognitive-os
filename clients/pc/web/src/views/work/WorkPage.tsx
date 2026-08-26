import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  OBSERVED_TASKS_KEY,
  TASK_LIST_LIMIT,
  WORK_PROBE_LIMIT,
  WORK_TASKS_KEY,
  buildWorkRows,
  filterWorkRows,
  preserveSelection,
  projectTaskEffects,
  projectTaskEnvelopes,
  projectTaskEvidence,
  sortWorkRows,
  workEffectsKey,
  workEvidenceKey,
  type EffectEntryView,
  type ObservedTask,
  type TaskEnvelopeView,
  type TaskEvidenceView,
  type WorkOriginFilter,
} from "../../data/projections/work";
import { appProjections } from "../../data/store";
import { useLastGood, useProjection, useProjections } from "../../data/useProjection";
import { sessionHasChannel } from "../../session";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";
import { useInspectorClear } from "../../shell/useInspectorClear";
import { WorkFilters } from "./WorkFilters";
import { WorkInspector } from "./WorkInspector";
import { WorkInventory, type InventorySource } from "./WorkInventory";

const NO_OBSERVED: ObservedTask[] = [];

/**
 * Work — the governed task space (docs/design/14). Two jobs in W4: account for
 * the tasks this page can actually see, and run the daemon's real creation
 * chain end to end. Per-task detail is W5; this space links to no such route
 * until it exists.
 */
export function WorkPage() {
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [params, setParams] = useSearchParams();
  /*
   * Scope, filter and selection live in the URL so a round trip through the
   * detail view returns to the same list the operator left, rather than
   * resetting to the default scope with nothing selected.
   */
  const [origin, setOrigin] = useState<WorkOriginFilter>(() =>
    params.get("scope") === "all" ? "all" : "session",
  );
  const [query, setQuery] = useState(() => params.get("q") ?? "");
  const [selectedRef, setSelectedRef] = useState<string | undefined>(
    () => params.get("task") ?? undefined,
  );

  const tasks = useProjection<TaskEnvelopeView[]>(WORK_TASKS_KEY);
  const observedProjection = useProjection<ObservedTask[]>(OBSERVED_TASKS_KEY);
  const tasksGood = useLastGood(tasks);
  const observed = observedProjection.data ?? NO_OBSERVED;
  const taskChannel = sessionHasChannel("task");

  const refreshList = useCallback(async () => {
    await fetchProjection(
      appProjections,
      WORK_TASKS_KEY,
      "/management/resource/v1/list?family=task",
      "management",
      projectTaskEnvelopes,
    );
  }, []);

  useEffect(() => {
    void refreshList();
  }, [refreshList]);

  const envelopes = useMemo(() => tasksGood.data ?? [], [tasksGood.data]);

  /*
   * Probe order is deterministic and bounded: the selected ref first so the
   * inspector is never waiting behind unrelated reads, then a prefix of the
   * merged set. There is no bulk evidence route, so this is a fan-out and it
   * stays small on purpose.
   */
  const probeRefs = useMemo(() => {
    const merged = sortWorkRows(
      buildWorkRows({ envelopes, observed, evidence: new Map(), probed: new Set() }),
    ).map((row) => row.taskRef);
    const ordered = selectedRef
      ? [selectedRef, ...merged.filter((ref) => ref !== selectedRef)]
      : merged;
    return ordered.slice(0, WORK_PROBE_LIMIT);
  }, [envelopes, observed, selectedRef]);
  const probeKey = probeRefs.join("|");

  const probe = useCallback(async (refs: string[]) => {
    if (!sessionHasChannel("task") || refs.length === 0) {
      return;
    }
    for (const taskRef of refs) {
      const encoded = encodeURIComponent(taskRef);
      await fetchProjection(
        appProjections,
        workEvidenceKey(taskRef),
        `/task/evidence?task_ref=${encoded}`,
        "task",
        projectTaskEvidence,
      );
      await fetchProjection(
        appProjections,
        workEffectsKey(taskRef),
        `/task/effects?task_ref=${encoded}`,
        "task",
        projectTaskEffects,
      );
    }
  }, []);

  useEffect(() => {
    void probe(probeKey === "" ? [] : probeKey.split("|"));
  }, [probeKey, probe, taskChannel]);

  const evidenceProjections = useProjections<TaskEvidenceView>(probeRefs.map(workEvidenceKey));
  const effectProjections = useProjections<EffectEntryView[]>(probeRefs.map(workEffectsKey));

  const evidenceByRef = useMemo(() => {
    const map = new Map<string, TaskEvidenceView | undefined>();
    probeRefs.forEach((taskRef, index) => map.set(taskRef, evidenceProjections[index]?.data));
    return map;
  }, [probeKey, evidenceProjections]);

  const probedRefs = useMemo(() => {
    const set = new Set<string>();
    probeRefs.forEach((taskRef, index) => {
      const projection = evidenceProjections[index];
      if (projection && projection.status !== "loading") {
        set.add(taskRef);
      }
    });
    return set;
  }, [probeKey, evidenceProjections]);

  const allRows = sortWorkRows(
    buildWorkRows({ envelopes, observed, evidence: evidenceByRef, probed: probedRefs }),
  );
  const sessionRows = filterWorkRows(allRows, "session");
  const scoped = filterWorkRows(allRows, origin);
  const visible =
    query.trim() === ""
      ? scoped
      : scoped.filter((row) => row.taskRef.toLowerCase().includes(query.trim().toLowerCase()));

  // Selection survives a refresh only while its ref is still in the list.
  const activeRef = preserveSelection(visible, selectedRef);
  const selectedRow = visible.find((row) => row.taskRef === activeRef);
  const selectedIndex = activeRef ? probeRefs.indexOf(activeRef) : -1;

  const writeListState = useCallback(
    (patch: { task?: string; scope?: WorkOriginFilter; q?: string }) => {
      const next = new URLSearchParams(params);
      for (const [key, value] of Object.entries(patch)) {
        if (value == null || value === "" || (key === "scope" && value === "session")) {
          next.delete(key);
        } else {
          next.set(key, value);
        }
      }
      setParams(next, { replace: true });
    },
    [params, setParams],
  );

  const select = useCallback(
    (taskRef: string | undefined) => {
      setSelectedRef(taskRef);
      writeListState({ task: taskRef });
    },
    [writeListState],
  );
  const clearInspector = useCallback(() => select(undefined), [select]);
  useInspectorClear(activeRef, clearInspector);

  const changeOrigin = useCallback(
    (next: WorkOriginFilter) => {
      setOrigin(next);
      writeListState({ scope: next });
    },
    [writeListState],
  );

  const changeQuery = useCallback(
    (next: string) => {
      setQuery(next);
      writeListState({ q: next });
    },
    [writeListState],
  );

  /** Carries the list state forward so the detail view can hand it back. */
  const listStateSearch = useMemo(() => {
    const carried = new URLSearchParams();
    if (origin === "all") {
      carried.set("scope", "all");
    }
    if (query.trim() !== "") {
      carried.set("q", query);
    }
    return carried.toString();
  }, [origin, query]);

  const refreshAll = useCallback(() => {
    setNowMs(Date.now());
    void (async () => {
      await refreshList();
      await probe(probeKey === "" ? [] : probeKey.split("|"));
    })();
  }, [refreshList, probe, probeKey]);

  const failed =
    tasks.status === "denied" ||
    tasks.status === "disconnected" ||
    tasks.status === "unknown" ||
    tasks.status === "not-run";

  /*
   * A zero-row inventory is only an authoritative empty when the envelope list
   * actually answered. Reporting a pending or failed read as "no task in this
   * scope" would be the page asserting knowledge it does not have.
   */
  const inventorySource: InventorySource = failed
    ? "failed"
    : tasks.status === "loading"
      ? "pending"
      : "answered";

  return (
    <>
      <PageHeader
        title="Work"
        lede="Governed tasks this page can account for, and the daemon's real creation chain. Nothing here infers that a task is running."
      />
      <p className="cp-next">
        <Link className="cp-button cp-button--primary" to="/work/new">
          New task
        </Link>{" "}
        <button type="button" className="cp-button" onClick={refreshAll}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">
          This space refreshes only when you ask. Nothing on this page polls the daemon.
        </span>
      </p>

      {failed ? (
        <p className="cp-reason" role="alert">
          <StateChip reading={readDomainState("load", tasks.status)} /> The daemon task list could
          not be read from <code className="cp-mono">/management/resource/v1/list?family=task</code>{" "}
          — <code className="cp-mono">{tasks.error?.code ?? tasks.status}</code>. Refs observed in
          this session are still listed below; no task is assumed to exist or not exist.
          {tasks.status === "denied" ? (
            <>
              {" "}
              <Link to="/session">Open Session</Link>
            </>
          ) : null}
        </p>
      ) : null}
      {!tasksGood.live && tasksGood.data !== undefined && !failed ? (
        <p className="cp-quiet">
          Showing the last known task list. It is not claimed as current.
        </p>
      ) : null}
      {!taskChannel ? (
        <p className="cp-reason" role="status">
          Not run: lifecycle evidence and effects live on the Task channel and this page holds no
          Task session. Rows below show <code className="cp-mono">state not exposed</code> — that
          is a missing read, not a claim about the task.
        </p>
      ) : null}

      <WorkFilters
        origin={origin}
        onOrigin={changeOrigin}
        query={query}
        onQuery={changeQuery}
        sessionCount={sessionRows.length}
        totalCount={allRows.length}
      />

      <div className="cp-mi">
        <div className="cp-master">
          <WorkInventory
            rows={visible}
            selectedRef={activeRef}
            onSelect={(row) => select(row.taskRef)}
            nowMs={nowMs}
            atBound={envelopes.length >= TASK_LIST_LIMIT}
            source={inventorySource}
            listStateSearch={listStateSearch}
          />
        </div>
        {selectedRow ? (
          <WorkInspector
            row={selectedRow}
            evidence={selectedIndex >= 0 ? evidenceProjections[selectedIndex] : undefined}
            effects={selectedIndex >= 0 ? effectProjections[selectedIndex] : undefined}
            nowMs={nowMs}
            listStateSearch={listStateSearch}
          />
        ) : null}
      </div>

      <HonestyNote>
        Work reads three things and nothing else: the daemon envelope list at{" "}
        <code>/management/resource/v1/list?family=task</code>, the task refs this browser session
        observed, and — for at most {WORK_PROBE_LIMIT} of those refs —{" "}
        <code>/task/evidence</code> and <code>/task/effects</code>. There is no task search, no
        task-list-with-state route and no cross-task stream, so this page can only ever account for
        refs it already knows. Opening a row composes per-task detail from the same reads plus
        bounded observation and consumption — there is no task detail route on this daemon. Watch
        attach lives on the Work detail Run section.
      </HonestyNote>
    </>
  );
}
