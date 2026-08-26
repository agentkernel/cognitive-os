/*
 * Agents (W6) view models — docs/design/16, reality map 31, state grammar 22.
 *
 * The Agent surface is a dossier composed from verified HTTP reads, not a
 * control panel and not an installation inventory. Runtime list/inspect are
 * projection-only (empty / RESOURCE_MANAGER_NOT_FOUND). HTTP-visible facts
 * are provider bindings, the dsh runtime snapshot, and (when a current task
 * is observed) task-scoped tool exposure.
 *
 * No fetching happens here. Views drive fetchProjection. Lifecycle verbs
 * have no HTTP route (BD-2) and must never be inferred from process liveness.
 */

import {
  AGENT_IDENTITY_KEYS,
  emptyIdentities,
  mergeIdentities,
  type AgentIdentities,
  type AgentIdentityKey,
} from "../../identities";
import { dispatchAllowed } from "../../policy";
import { readDomainState, type StateReading } from "../../state/stateMap";
import { asList, asRecord } from "../projections";
import type { BindingView, DshRuntimeView, ProviderAccount } from "./providers";

export const HTTP_NAMED_AGENTS = ["pi", "dsh"] as const;

export const AGENTS_RUNTIME_LIST_KEY = "agents:runtime-list";
export const agentInspectKey = (id: string) => `agents:inspect:${id}`;
export const agentExposureKey = (taskRef: string) => `agents:exposure:${taskRef}`;

export const RUNTIME_LIST_PATH = "/management/resource/v1/list?family=runtime";
export const RUNTIME_INSPECT_PATH = "/management/resource/v1/inspect?family=runtime";
export const runtimeInspectPath = (id: string) =>
  `${RUNTIME_INSPECT_PATH}&id=${encodeURIComponent(id)}`;
export const toolExposurePath = (taskRef: string) =>
  `/task/resource/v1/tool/exposure?task_ref=${encodeURIComponent(taskRef)}`;

export const RUNTIME_INSPECT_UNAVAILABLE =
  "context and runtime have no authority-backed Resource Manager rows";

export const CAPABILITY_ANNOTATION =
  "Installed ≠ permitted. Capability = registration + binding + exposure + lifecycle.";

export const LIFECYCLE_HEADER =
  "Lifecycle control runs through `cognitive` CLI (BD-2)";

/** Class-C verbs — text + CLI path, never buttons (DD-08). */
export const AGENT_LIFECYCLE_CLI: readonly {
  verb: string;
  cli: string;
  reason: string;
}[] = [
  {
    verb: "pause",
    cli: "cognitive agent-pause",
    reason: "no agent lifecycle route exists over HTTP (BD-2)",
  },
  {
    verb: "resume",
    cli: "cognitive agent-resume",
    reason: "no agent lifecycle route exists over HTTP (BD-2)",
  },
  {
    verb: "stop",
    cli: "cognitive agent-stop",
    reason: "no agent lifecycle route exists over HTTP (BD-2)",
  },
  {
    verb: "restart",
    cli: "cognitive agent-stop then cognitive activate",
    reason: "restart is a CLI composition; there is no HTTP restart route (BD-2)",
  },
  {
    verb: "recover",
    cli: "cognitive agent-recover",
    reason: "no agent recover route exists over HTTP (BD-2)",
  },
  {
    verb: "quarantine",
    cli: "cognitive (installation-root quarantine)",
    reason: "agent-level quarantine is not exposed over HTTP (BD-2)",
  },
];

/** Captions that keep identity documents from being confused with each other. */
export const IDENTITY_CAPTIONS: Partial<Record<AgentIdentityKey, string>> = {
  package: "a package digest is not a running process",
  installation: "verified private bytes are not registration policy",
  registration: "registration is policy, not a live instance",
  instance: "a supervised instance is not an OS process",
  sidecar: "a sidecar session is not the instance",
  execution: "an execution epoch is not task completion",
  process: "process liveness is not task completion",
  task: "a Task is not an Agent",
  shell_session: "a shell session is not the Agent instance",
};

export interface RuntimeListView {
  family: string;
  authoritySource?: string;
  resources: { id: string }[];
}

export function projectRuntimeList(body: unknown): RuntimeListView {
  const record = asRecord(body);
  return {
    family: String(record.family ?? "runtime"),
    authoritySource:
      record.authority_source == null ? undefined : String(record.authority_source),
    resources: asList(body, ["resources", "items"]).map((row) => {
      const item = asRecord(row);
      return { id: String(item.id ?? "unknown") };
    }),
  };
}

export function extractIdentitiesFromInspect(body: unknown): AgentIdentities {
  const record = asRecord(body);
  const resource = asRecord(record.resource ?? record);
  const nested = asRecord(resource.identities ?? record.identities);
  const partial: AgentIdentities = {};
  for (const key of AGENT_IDENTITY_KEYS) {
    const value = nested[key] ?? resource[key] ?? record[key];
    if (typeof value === "string" && value !== "") {
      partial[key] = value;
    }
  }
  return mergeIdentities(partial);
}

export interface IdentityCardView {
  key: AgentIdentityKey;
  value: string;
  source: string;
  caption?: string;
}

export function identityCards(identities: AgentIdentities, source: string): IdentityCardView[] {
  return AGENT_IDENTITY_KEYS.map((key) => ({
    key,
    value: identities[key] ?? "unknown",
    source,
    caption: IDENTITY_CAPTIONS[key],
  }));
}

export function inspectUnavailableCards(): IdentityCardView[] {
  return identityCards(
    emptyIdentities(),
    `GET ${RUNTIME_INSPECT_PATH} — ${RUNTIME_INSPECT_UNAVAILABLE}`,
  );
}

export function isRuntimeInspectUnavailable(code: string | undefined): boolean {
  return code === "RESOURCE_MANAGER_NOT_FOUND";
}

export function normalizeAgentId(agent: string): string {
  const trimmed = agent.trim();
  if (trimmed === "dsh" || trimmed.endsWith("/dsh")) {
    return "dsh";
  }
  if (trimmed === "pi" || trimmed.endsWith("/pi")) {
    return "pi";
  }
  return trimmed;
}

export function agentIsAddressable(id: string, bindingAgentIds: readonly string[]): boolean {
  const key = normalizeAgentId(id);
  if (key === "pi" || key === "dsh") {
    return true;
  }
  return bindingAgentIds.some((agent) => normalizeAgentId(agent) === key);
}

export type BindingDispatch = "unbound" | "callable" | "blocked";

export type CurrentWorkKind = "task" | "none" | "unavailable";

export interface AgentInventoryRow {
  id: string;
  displayName: string;
  binding?: BindingView;
  dispatch: BindingDispatch;
  /**
   * Best-available lifecycle word. For dsh this is the runtime snapshot
   * state (observation-labeled). For pi there is no HTTP projection.
   */
  lifecycleLabel: string;
  lifecycleSource: string;
  /** dsh snapshot state verbatim, when this row is dsh and a snapshot was read. */
  dshState?: string;
  currentWorkKind: CurrentWorkKind;
  currentWorkLabel: string;
  currentTaskRef?: string;
}

function pickBinding(bindings: BindingView[], id: string): BindingView | undefined {
  const matches = bindings.filter((row) => normalizeAgentId(row.agent) === id);
  return matches.find((row) => row.status === "active") ?? matches[0];
}

function dispatchFor(
  binding: BindingView | undefined,
  accounts: ProviderAccount[] | undefined,
): BindingDispatch {
  if (!binding) {
    return "unbound";
  }
  const account = accounts?.find((row) => row.id === binding.accountId);
  const callable = dispatchAllowed({
    accountStatus: account?.status,
    bindingStatus: binding.status,
  });
  if (callable) {
    return "callable";
  }
  return "blocked";
}

function dshCurrentWork(runtime: DshRuntimeView | undefined): {
  kind: CurrentWorkKind;
  label: string;
  taskRef?: string;
} {
  if (!runtime) {
    return {
      kind: "unavailable",
      label: "dsh runtime snapshot not read yet",
    };
  }
  const withTask = (runtime.sessions ?? []).find(
    (session) => typeof session.taskRef === "string" && session.taskRef !== "",
  );
  if (withTask?.taskRef) {
    return { kind: "task", label: withTask.taskRef, taskRef: withTask.taskRef };
  }
  return { kind: "none", label: "none observed" };
}

function buildRow(
  id: string,
  binding: BindingView | undefined,
  accounts: ProviderAccount[] | undefined,
  runtime: DshRuntimeView | undefined,
): AgentInventoryRow {
  const dispatch = dispatchFor(binding, accounts);
  if (id === "dsh") {
    const work = dshCurrentWork(runtime);
    const state = runtime?.state;
    return {
      id,
      displayName: "dsh",
      binding,
      dispatch,
      lifecycleLabel: state ?? "lifecycle not exposed over HTTP",
      lifecycleSource: state
        ? "GET /personal/dsh/runtime (observation, candidate_only)"
        : "GET /personal/dsh/runtime was not read",
      dshState: state,
      currentWorkKind: work.kind,
      currentWorkLabel: work.label,
      currentTaskRef: work.taskRef,
    };
  }
  if (id === "pi") {
    return {
      id,
      displayName: "pi",
      binding,
      dispatch,
      lifecycleLabel: "lifecycle not exposed over HTTP",
      lifecycleSource: "lifecycle is CLI-observable; HTTP projection is BD-2",
      currentWorkKind: "unavailable",
      currentWorkLabel: "not observable over HTTP (BD-2/BD-3)",
    };
  }
  return {
    id,
    displayName: id,
    binding,
    dispatch,
    lifecycleLabel: "lifecycle not exposed over HTTP",
    lifecycleSource: "no HTTP lifecycle projection for this actor (BD-2)",
    currentWorkKind: "unavailable",
    currentWorkLabel: "not observable over HTTP (BD-2/BD-3)",
  };
}

/**
 * Inventory is composed from bindings + the dsh snapshot. The runtime
 * Resource Manager list is empty (projection-only) and is not a source of
 * actor rows. pi and dsh are the two agents the binding API can name.
 */
export function composeAgentRows(input: {
  bindings?: BindingView[];
  accounts?: ProviderAccount[];
  runtime?: DshRuntimeView;
}): AgentInventoryRow[] {
  const bindings = input.bindings ?? [];
  const extra = [
    ...new Set(
      bindings
        .map((row) => normalizeAgentId(row.agent))
        .filter((id) => id !== "pi" && id !== "dsh" && id !== "unknown"),
    ),
  ].sort();
  const ids = [...HTTP_NAMED_AGENTS, ...extra];
  return ids.map((id) => buildRow(id, pickBinding(bindings, id), input.accounts, input.runtime));
}

export function agentLifecycleReading(row: AgentInventoryRow): StateReading {
  if (row.id === "dsh" && row.dshState) {
    return readDomainState("dsh", row.dshState);
  }
  return {
    category: "unknown",
    label: row.lifecycleLabel,
    unmapped: false,
  };
}

export function bindingReading(row: AgentInventoryRow): StateReading {
  if (row.dispatch === "unbound") {
    return { category: "unknown", label: "unbound", unmapped: false };
  }
  if (row.dispatch === "callable") {
    return { category: "ready", label: "callable", unmapped: false };
  }
  return { category: "blocked", label: "blocked", unmapped: false };
}

export function bindingSummary(row: AgentInventoryRow): string {
  if (!row.binding) {
    return "no binding — this agent cannot call a model";
  }
  const revision =
    row.binding.revision == null ? "revision unknown" : `rev ${row.binding.revision}`;
  return `${row.binding.accountId} / ${row.binding.modelId} · ${revision}`;
}

export interface ToolExposureView {
  taskRef?: string;
  tools: { id: string; lifecycle?: string }[];
}

export function projectToolExposure(body: unknown): ToolExposureView {
  const record = asRecord(body);
  const rows = asList(body, ["tools", "exposure", "items", "operations"]);
  return {
    taskRef: record.task_ref == null ? undefined : String(record.task_ref),
    tools: rows.map((row) => {
      const item = asRecord(row);
      return {
        id: String(item.op_id ?? item.id ?? item.tool_id ?? "unknown"),
        lifecycle:
          item.lifecycle == null && item.status == null
            ? undefined
            : String(item.lifecycle ?? item.status),
      };
    }),
  };
}

export const DOSSIER_SECTIONS = [
  { id: "overview", title: "Overview" },
  { id: "current", title: "Current work" },
  { id: "binding", title: "Binding" },
  { id: "capabilities", title: "Capabilities" },
  { id: "activity", title: "Activity" },
  { id: "evidence", title: "Evidence" },
  { id: "runtime", title: "dsh runtime" },
] as const;

export type DossierSectionId = (typeof DOSSIER_SECTIONS)[number]["id"];

export function isDossierSection(value: string | null): value is DossierSectionId {
  return DOSSIER_SECTIONS.some((section) => section.id === value);
}

export function dossierSectionsFor(agentId: string): typeof DOSSIER_SECTIONS[number][] {
  if (normalizeAgentId(agentId) === "dsh") {
    return [...DOSSIER_SECTIONS];
  }
  return DOSSIER_SECTIONS.filter((section) => section.id !== "runtime");
}
