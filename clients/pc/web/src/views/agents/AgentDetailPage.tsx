import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { FactGrid } from "../../components/FactGrid";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState, LoadingState, UnavailableState } from "../../components/states";
import { fetchProjection } from "../../data/fetchProjection";
import {
  AGENT_LIFECYCLE_CLI,
  CAPABILITY_ANNOTATION,
  LIFECYCLE_HEADER,
  agentExposureKey,
  agentInspectKey,
  agentIsAddressable,
  agentLifecycleReading,
  bindingReading,
  bindingSummary,
  composeAgentRows,
  dossierSectionsFor,
  extractIdentitiesFromInspect,
  identityCards,
  inspectUnavailableCards,
  isDossierSection,
  isRuntimeInspectUnavailable,
  normalizeAgentId,
  projectToolExposure,
  runtimeInspectPath,
  toolExposurePath,
  type DossierSectionId,
  type ToolExposureView,
} from "../../data/projections/agents";
import {
  DSH_RUNTIME_KEY,
  projectBindings,
  projectDshRuntime,
  projectProviderAccounts,
  type BindingView,
  type DshRuntimeView,
  type ProviderAccount,
} from "../../data/projections/providers";
import type { AgentIdentities } from "../../identities";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { sessionHasChannel } from "../../session";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { BINDINGS_KEY } from "../providers/BindingsSection";
import { PROVIDER_ACCOUNTS_KEY } from "../providers/ProvidersPage";
import { ProjectionState } from "../providers/ProjectionState";
import { IdentityCards } from "./IdentityCards";

/**
 * Agent dossier — docs/design/16 §3. Continuous sections, no tabs. Identity
 * cards reuse identities.ts. No fake lifecycle HTTP.
 */
export function AgentDetailPage() {
  const params = useParams();
  const rawId = params.id ?? "";
  const agentId = normalizeAgentId(rawId);
  const [query, setQuery] = useSearchParams();
  const requested = query.get("section");
  const sections = dossierSectionsFor(agentId);
  const [active, setActive] = useState<DossierSectionId>(
    isDossierSection(requested) ? requested : "overview",
  );

  const inspectKey = agentInspectKey(agentId);

  const refresh = useCallback(async () => {
    await fetchProjection(
      appProjections,
      BINDINGS_KEY,
      "/management/agent-bindings",
      "management",
      projectBindings,
    );
    await fetchProjection(
      appProjections,
      PROVIDER_ACCOUNTS_KEY,
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    await fetchProjection(
      appProjections,
      DSH_RUNTIME_KEY,
      "/personal/dsh/runtime",
      "management",
      projectDshRuntime,
    );
    await fetchProjection(
      appProjections,
      inspectKey,
      runtimeInspectPath(agentId),
      "management",
      extractIdentitiesFromInspect,
    );
  }, [agentId, inspectKey]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const bindings = useProjection<BindingView[]>(BINDINGS_KEY);
  const accounts = useProjection<ProviderAccount[]>(PROVIDER_ACCOUNTS_KEY);
  const runtime = useProjection<DshRuntimeView>(DSH_RUNTIME_KEY);
  const inspect = useProjection<AgentIdentities>(inspectKey);

  const rows = useMemo(
    () =>
      composeAgentRows({
        bindings: bindings.data ?? [],
        accounts: accounts.data,
        runtime: runtime.data,
      }),
    [bindings.data, accounts.data, runtime.data],
  );
  const row = rows.find((entry) => entry.id === agentId);
  const addressable = agentIsAddressable(
    agentId,
    (bindings.data ?? []).map((binding) => binding.agent),
  );
  const bindingsSettled = bindings.status !== "loading";

  const exposureKey = row?.currentTaskRef ? agentExposureKey(row.currentTaskRef) : "";
  const taskChannel = sessionHasChannel("task");

  useEffect(() => {
    if (!row?.currentTaskRef || !taskChannel) {
      return;
    }
    void fetchProjection(
      appProjections,
      agentExposureKey(row.currentTaskRef),
      toolExposurePath(row.currentTaskRef),
      "task",
      projectToolExposure,
    );
  }, [row?.currentTaskRef, taskChannel]);

  const exposure = useProjection<ToolExposureView>(exposureKey || "agents:exposure:none");

  const selectSection = useCallback(
    (section: DossierSectionId) => {
      setActive(section);
      const next = new URLSearchParams(query);
      next.set("section", section);
      setQuery(next, { replace: true });
      document.getElementById(`section-${section}`)?.scrollIntoView({ block: "start" });
    },
    [query, setQuery],
  );

  if (bindingsSettled && !addressable) {
    return (
      <>
        <PageHeader title="Agent not found" lede="This page cannot account for that actor." />
        <section className="cp-region" aria-labelledby="notfound-title">
          <h3 className="cp-section-title" id="notfound-title">
            No such agent on this HTTP surface
          </h3>
          <p className="cp-reason" role="alert">
            <code className="cp-mono">{rawId}</code> is not pi, not dsh, and does not appear in{" "}
            <code className="cp-mono">/management/agent-bindings</code>.
          </p>
          <p className="cp-next">
            <Link className="cp-button cp-button--primary" to="/agents">
              Back to Agents
            </Link>
          </p>
        </section>
        <HonestyNote>
          No dossier is fabricated for an unknown actor. The runtime Resource Manager has no
          authority-backed agent rows.
        </HonestyNote>
      </>
    );
  }

  if (!row) {
    return (
      <>
        <PageHeader title="Agent" />
        <LoadingState label="Reading agent bindings and runtime facts." />
      </>
    );
  }

  const inspectCards =
    inspect.status === "ready" && inspect.data
      ? identityCards(inspect.data, inspect.source ?? runtimeInspectPath(agentId))
      : inspectUnavailableCards();
  const identityReady =
    inspect.status === "ready" || isRuntimeInspectUnavailable(inspect.error?.code);
  const bindingHref = row.binding
    ? `/providers/${encodeURIComponent(row.binding.accountId)}`
    : "/providers";

  return (
    <>
      <p className="cp-next">
        <Link to="/agents">Back to Agents</Link>
        {" · "}
        <button type="button" className="cp-button" onClick={() => void refresh()}>
          Refresh
        </button>
      </p>
      <header className="cp-detail-head">
        <PageHeader
          title={row.displayName}
          lede={
            <>
              <StateChip reading={agentLifecycleReading(row)} />{" "}
              <StateChip reading={bindingReading(row)} /> {LIFECYCLE_HEADER}
            </>
          }
        />
        <p className="cp-quiet">{row.lifecycleSource}</p>
      </header>

      <nav className="cp-sectionnav" aria-label="Agent dossier sections">
        <ul className="cp-sectionnav-list">
          {sections.map((section) => (
            <li key={section.id}>
              <button
                type="button"
                className="cp-sectionnav-link"
                aria-current={active === section.id ? "true" : undefined}
                onClick={() => selectSection(section.id)}
              >
                {section.title}
              </button>
            </li>
          ))}
        </ul>
        <p className="cp-quiet">Every section below is always rendered. This only moves you to one of them.</p>
      </nav>

      <section className="cp-region" id="section-overview" aria-labelledby="overview-title">
        <h3 className="cp-section-title" id="overview-title">
          Overview
        </h3>
        {inspect.status === "loading" ? (
          <LoadingState label="Reading runtime inspect." />
        ) : identityReady ? (
          <IdentityCards cards={inspectCards} />
        ) : (
          <ProjectionState projection={inspect} what="Runtime inspect" />
        )}
        {isRuntimeInspectUnavailable(inspect.error?.code) ? (
          <HonestyNote>
            Identity cards stay at unknown because{" "}
            <code>GET /management/resource/v1/inspect?family=runtime</code> has no
            authority-backed rows (<code>RESOURCE_MANAGER_NOT_FOUND</code>). That is a named gap,
            not a zero inventory of identities.
          </HonestyNote>
        ) : null}
      </section>

      <section className="cp-region" id="section-current" aria-labelledby="current-title">
        <h3 className="cp-section-title" id="current-title">
          Current work
        </h3>
        {row.currentWorkKind === "task" && row.currentTaskRef ? (
          <p className="cp-region-line">
            Observed on the dsh snapshot:{" "}
            <Link to={`/work/${encodeURIComponent(row.currentTaskRef)}`}>{row.currentTaskRef}</Link>
          </p>
        ) : row.currentWorkKind === "none" ? (
          <EmptyState title="none observed">
            The dsh snapshot recorded no bound task_ref. That is not idle, and it is not inferred
            from process liveness.
          </EmptyState>
        ) : (
          <UnavailableState
            what="Current work for this actor"
            dependency="BD-2/BD-3"
            cliPath="cognitive agent-health"
          />
        )}
        <p className="cp-quiet">{row.currentWorkLabel}</p>
      </section>

      <section className="cp-region" id="section-binding" aria-labelledby="binding-title">
        <h3 className="cp-section-title" id="binding-title">
          Binding
        </h3>
        <FactGrid
          facts={[
            { label: "dispatch", value: row.dispatch },
            { label: "summary", value: bindingSummary(row) },
            {
              label: "account",
              value: row.binding?.accountId ?? "no binding — this agent cannot call a model",
            },
            { label: "model", value: row.binding?.modelId ?? "unbound" },
            {
              label: "revision",
              value: row.binding?.revision == null ? "unknown" : String(row.binding.revision),
            },
          ]}
        />
        <p className="cp-next">
          <Link to={bindingHref}>Change binding on Providers</Link>
        </p>
      </section>

      <section className="cp-region" id="section-capabilities" aria-labelledby="capabilities-title">
        <h3 className="cp-section-title" id="capabilities-title">
          Capabilities
        </h3>
        <p className="cp-reason" role="note">
          {CAPABILITY_ANNOTATION}
        </p>
        <FactGrid
          facts={[
            {
              label: "model route",
              value: row.binding
                ? `${row.binding.accountId} / ${row.binding.modelId}`
                : "no binding — this agent cannot call a model",
            },
            {
              label: "workspace scope",
              value: "not exposed over HTTP",
            },
          ]}
        />
        {row.currentTaskRef && taskChannel ? (
          exposure.status === "ready" ? (
            <ul className="cp-plain-list">
              {(exposure.data?.tools.length ?? 0) === 0 ? (
                <li>No tool exposure recorded for this task.</li>
              ) : (
                exposure.data?.tools.map((tool) => (
                  <li key={tool.id}>
                    <code className="cp-mono">{tool.id}</code>
                    {tool.lifecycle ? ` · ${tool.lifecycle}` : ""}
                  </li>
                ))
              )}
            </ul>
          ) : (
            <ProjectionState projection={exposure} what="Task-scoped tool exposure" />
          )
        ) : (
          <UnavailableState
            what="Tool exposure"
            dependency="tool/exposure is task-scoped; no current task_ref is observed"
          />
        )}
      </section>

      <section className="cp-region" id="section-activity" aria-labelledby="activity-title">
        <h3 className="cp-section-title" id="activity-title">
          Activity
        </h3>
        <p>
          There is no per-agent activity feed over HTTP (BD-5). The Activity space is a later wave.
        </p>
        <p className="cp-next">
          <Link to="/activity">Open Activity</Link>
          {row.currentTaskRef ? (
            <>
              {" · "}
              <Link to={`/work/${encodeURIComponent(row.currentTaskRef)}`}>Open current work</Link>
            </>
          ) : null}
        </p>
      </section>

      <section className="cp-region" id="section-evidence" aria-labelledby="evidence-title">
        <h3 className="cp-section-title" id="evidence-title">
          Evidence
        </h3>
        <p>
          Evidence is per-task. This dossier does not invent an actor-scoped verification stream.
        </p>
        {row.currentTaskRef ? (
          <p className="cp-next">
            <Link to={`/work/${encodeURIComponent(row.currentTaskRef)}?section=evidence`}>
              Open evidence on current work
            </Link>
          </p>
        ) : (
          <UnavailableState
            what="Actor-scoped evidence"
            dependency="BD-2/BD-3 — no per-agent evidence route"
          />
        )}
      </section>

      {agentId === "dsh" ? (
        <section className="cp-region" id="section-runtime" aria-labelledby="runtime-title">
          <h3 className="cp-section-title" id="runtime-title">
            dsh runtime
          </h3>
          {runtime.status === "ready" || runtime.status === "empty" || runtime.status === "stale" ? (
            <>
              <FactGrid
                facts={[
                  { label: "state", value: runtime.data?.state ?? "unknown" },
                  {
                    label: "process alive",
                    value:
                      runtime.data?.processAlive == null
                        ? "unknown"
                        : runtime.data.processAlive
                          ? "true"
                          : "false",
                  },
                  {
                    label: "process id",
                    value:
                      runtime.data?.processId == null ? "unknown" : String(runtime.data.processId),
                  },
                  {
                    label: "sessions",
                    value:
                      runtime.data?.sessionCount == null
                        ? "unknown"
                        : String(runtime.data.sessionCount),
                  },
                  {
                    label: "candidate_only",
                    value: runtime.data?.candidateOnly === true ? "true" : "unknown",
                  },
                  {
                    label: "dsh response is not task completion",
                    value:
                      runtime.data?.dshResponseIsNotTaskCompletion === true ? "true" : "unknown",
                  },
                ]}
              />
              {runtime.data?.state === "CRASHED" ? (
                <p className="cp-reason" role="alert">
                  Runtime is CRASHED. Restart via <code className="cp-mono">cognitive dsh …</code>{" "}
                  (class-C). Process liveness is an observation, not Task completion.
                </p>
              ) : null}
              {(runtime.data?.sessions ?? []).length > 0 ? (
                <ul className="cp-plain-list">
                  {runtime.data?.sessions.map((session) => (
                    <li key={session.sessionId}>
                      <code className="cp-mono">{session.sessionId}</code>
                      {session.state ? ` · ${session.state}` : ""}
                      {session.fencingEpoch != null ? ` · fencing ${session.fencingEpoch}` : ""}
                      {session.taskRef ? (
                        <>
                          {" · "}
                          <Link to={`/work/${encodeURIComponent(session.taskRef)}`}>
                            {session.taskRef}
                          </Link>
                        </>
                      ) : (
                        " · no task_ref"
                      )}
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="cp-quiet">No sessions in the snapshot.</p>
              )}
              {row.binding ? (
                <p className="cp-next">
                  <Link to={bindingHref}>Apply Cos model from the Providers binding flow</Link>
                </p>
              ) : null}
            </>
          ) : (
            <ProjectionState projection={runtime} what="dsh runtime snapshot" />
          )}
          <HonestyNote>
            The dsh snapshot is observation-only (<code>candidate_only</code>). A dsh response is
            never Task completion.
          </HonestyNote>
        </section>
      ) : null}

      <section className="cp-region" id="section-lifecycle" aria-labelledby="lifecycle-title">
        <h3 className="cp-section-title" id="lifecycle-title">
          Lifecycle (class C)
        </h3>
        <ul className="cp-plain-list">
          {AGENT_LIFECYCLE_CLI.map((entry) => (
            <li key={entry.verb}>
              <strong>{entry.verb}</strong> is not available over HTTP — {entry.reason}. CLI:{" "}
              <code className="cp-mono">{entry.cli}</code>
            </li>
          ))}
        </ul>
        <HonestyNote>
          These lines occupy the control slot so a disabled button is never drawn. When BD-2 lands,
          the same slots can upgrade without a redesign.
        </HonestyNote>
      </section>
    </>
  );
}
