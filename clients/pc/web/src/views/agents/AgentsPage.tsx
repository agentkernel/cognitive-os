import { useCallback, useEffect, useMemo, useState } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  AGENTS_RUNTIME_LIST_KEY,
  RUNTIME_LIST_PATH,
  composeAgentRows,
  projectRuntimeList,
  type RuntimeListView,
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
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { BINDINGS_KEY } from "../providers/BindingsSection";
import { PROVIDER_ACCOUNTS_KEY } from "../providers/ProvidersPage";
import { ProjectionState } from "../providers/ProjectionState";
import { AgentInspector } from "./AgentInspector";
import { AgentInventory } from "./AgentInventory";

const LIFECYCLE_NOTE =
  "Agent lifecycle verbs (pause/resume/stop/restart/quarantine) are not available over HTTP (BD-2); they run through cognitive admin commands. This page never draws those verbs as buttons.";

/**
 * Agents — the actor inventory (docs/design/16). Read-mostly: bindings, the
 * dsh snapshot, and a named empty runtime list. No lifecycle controls.
 */
export function AgentsPage() {
  const [selectedId, setSelectedId] = useState<string | undefined>();

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
      AGENTS_RUNTIME_LIST_KEY,
      RUNTIME_LIST_PATH,
      "management",
      projectRuntimeList,
    );
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const bindings = useProjection<BindingView[]>(BINDINGS_KEY);
  const accounts = useProjection<ProviderAccount[]>(PROVIDER_ACCOUNTS_KEY);
  const runtime = useProjection<DshRuntimeView>(DSH_RUNTIME_KEY);
  const runtimeList = useProjection<RuntimeListView>(AGENTS_RUNTIME_LIST_KEY);

  const rows = useMemo(
    () =>
      composeAgentRows({
        bindings: bindings.data ?? [],
        accounts: accounts.data,
        runtime: runtime.data,
      }),
    [bindings.data, accounts.data, runtime.data],
  );
  const selected = rows.find((row) => row.id === selectedId);
  const inventorySource =
    bindings.status === "loading" ? "pending" : "answered";

  return (
    <>
      <PageHeader
        title="Agents"
        lede="What this actor is, what it may use, what it is doing, and what it has verifiably done. Lifecycle control is CLI-only."
      />
      <p className="cp-next">
        <button type="button" className="cp-button" onClick={() => void refresh()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">
          This space refreshes only when you ask. Nothing on this page polls the daemon.
        </span>
      </p>

      <ProjectionState projection={bindings} what="Agent bindings" />
      {runtime.status === "denied" ||
      runtime.status === "disconnected" ||
      runtime.status === "not-run" ||
      runtime.status === "unknown" ? (
        <ProjectionState projection={runtime} what="dsh runtime snapshot" />
      ) : null}

      <div className="cp-mi">
        <div className="cp-master">
          <AgentInventory
            rows={rows}
            selectedId={selectedId}
            onSelect={(row) => setSelectedId(row.id)}
            source={inventorySource}
          />
          {runtimeList.status === "ready" || runtimeList.status === "empty" ? (
            <p className="cp-quiet">
              Runtime list <code className="cp-mono">{runtimeList.source}</code>
              {runtimeList.data?.authoritySource
                ? ` · authority_source ${runtimeList.data.authoritySource}`
                : ""}{" "}
              returned {runtimeList.data?.resources.length ?? 0} row
              {(runtimeList.data?.resources.length ?? 0) === 1 ? "" : "s"}.
            </p>
          ) : null}
        </div>
        {selected ? <AgentInspector row={selected} /> : null}
      </div>

      <HonestyNote>{LIFECYCLE_NOTE}</HonestyNote>
    </>
  );
}
