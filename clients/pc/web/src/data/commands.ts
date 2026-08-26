/*
 * Command palette index — docs/design/21, wave 10 in 39.
 *
 * The index is the loaded projection set plus the frozen destinations.
 * There is no server search (BD-6). Class-A items only land on governed
 * routes. Class-B may run inline. Class-C/D verbs are absent, not disabled.
 */

import { HTTP_NAMED_AGENTS } from "./projections/agents";
import { type TaskEnvelopeView } from "./projections/home";
import { type BindingView, type ProviderAccount, type ProviderAlertView } from "./projections/providers";
import { resourceListKey, type ResourceListView } from "./projections/resources";
import { TOOL_CATALOG_KEY, type ToolCatalogView } from "./projections/tools";
import { WORK_TASKS_KEY } from "./projections/work";
import { PRIMARY_NAV } from "../shell/PrimaryNav";
import type { ProjectionStore } from "./store";

const HOME_ALERTS_KEY = "home:alerts";
const HOME_TASKS_KEY = "home:tasks";

export const COMMAND_INDEX_HONESTY =
  "Palette searches known objects in this tab (BD-6). There is no server search. The inventory is partial (BD-3).";

export const COMMAND_NO_CLASS_C =
  "Class-C/D verbs are absent here. Cancel, pause, retry, upgrade and uninstall are not palette entries.";

export const COMMAND_NO_RESULTS =
  "no known object matches — inventory is partial (BD-3)";

export const COMMAND_GROUP_LABEL: Record<CommandKind, string> = {
  action: "ACTIONS",
  object: "OBJECTS",
  destination: "DESTINATIONS",
  help: "HELP",
};

export type CommandKind = "action" | "object" | "destination" | "help";
export type CommandExecution = "navigate" | "copy-location" | "copy-ref" | "acknowledge";

export interface CommandItem {
  id: string;
  kind: CommandKind;
  execution: CommandExecution;
  label: string;
  detail?: string;
  keywords: string[];
  href?: string;
  alertId?: string;
  copyValue?: string;
  contextual?: boolean;
}

export interface CommandGroup {
  kind: CommandKind;
  label: string;
  items: CommandItem[];
}

const recents: string[] = [];

export function rememberCommand(id: string): void {
  const next = [id, ...recents.filter((item) => item !== id)].slice(0, 6);
  recents.length = 0;
  recents.push(...next);
}

export function recentCommandIds(): string[] {
  return [...recents];
}

export function resetCommandRecents(): void {
  recents.length = 0;
}

const CREATE_ACTIONS: CommandItem[] = [
  {
    id: "action:new-task",
    kind: "action",
    execution: "navigate",
    label: "New task",
    detail: "Opens the governed creation chain",
    keywords: ["create", "admit", "work"],
    href: "/work/new",
  },
  {
    id: "action:add-provider",
    kind: "action",
    execution: "navigate",
    label: "Add provider account",
    detail: "Lands on Providers; key entry stays in that form",
    keywords: ["create", "account", "provider", "configure"],
    href: "/providers",
  },
  {
    id: "action:remember",
    kind: "action",
    execution: "navigate",
    label: "Remember",
    detail: "Lands on Memory class-A",
    keywords: ["create", "memory"],
    href: "/resources/memory",
  },
  {
    id: "action:import-skill",
    kind: "action",
    execution: "navigate",
    label: "Import skill",
    detail: "Lands on Skills class-A",
    keywords: ["create", "skill", "import"],
    href: "/resources/skill",
  },
  {
    id: "action:configure-provider",
    kind: "action",
    execution: "navigate",
    label: "Configure provider",
    detail: "Binding, budget and price stay on the account page",
    keywords: ["configure", "binding", "budget", "price"],
    href: "/providers",
  },
  {
    id: "action:copy-location",
    kind: "action",
    execution: "copy-location",
    label: "Copy current location",
    detail: "Class-B; copies this tab's hash",
    keywords: ["copy", "url", "hash"],
  },
];

const HELP: CommandItem = {
  id: "help:index",
  kind: "help",
  execution: "navigate",
  label: "Search honesty",
  detail: COMMAND_INDEX_HONESTY,
  keywords: ["bd-6", "bd-3", "search", "partial", "help"],
};

function destinations(): CommandItem[] {
  const spaces = PRIMARY_NAV.map(([to, label]) => ({
    id: `dest:${to}`,
    kind: "destination" as const,
    execution: "navigate" as const,
    label,
    detail: "Navigate",
    keywords: [label.toLowerCase(), "go", "space"],
    href: to,
  }));
  return [
    ...spaces,
    {
      id: "dest:system-doctor",
      kind: "destination",
      execution: "navigate",
      label: "System · Doctor",
      keywords: ["doctor", "system"],
      href: "/system?section=doctor",
    },
    {
      id: "dest:system-stewardship",
      kind: "destination",
      execution: "navigate",
      label: "System · Stewardship",
      keywords: ["backup", "restore", "system"],
      href: "/system?section=stewardship",
    },
  ];
}

function namedAgents(): CommandItem[] {
  return HTTP_NAMED_AGENTS.map((id) => ({
    id: `object:agent:${id}`,
    kind: "object" as const,
    execution: "navigate" as const,
    label: `agent ${id}`,
    detail: "Named agent dossier",
    keywords: [id, "agent", "inspect"],
    href: `/agents/${encodeURIComponent(id)}`,
  }));
}

function taskItems(rows: TaskEnvelopeView[] | undefined): CommandItem[] {
  return (rows ?? []).map((row) => ({
    id: `object:task:${row.taskRef}`,
    kind: "object" as const,
    execution: "navigate" as const,
    label: `task ${row.taskRef}`,
    detail: "Open Work detail (evidence is on that page)",
    keywords: [row.taskRef, "task", "inspect", "verify"],
    href: `/work/${encodeURIComponent(row.taskRef)}`,
  }));
}

function accountItems(rows: ProviderAccount[] | undefined): CommandItem[] {
  return (rows ?? []).map((row) => ({
    id: `object:account:${row.id}`,
    kind: "object" as const,
    execution: "navigate" as const,
    label: row.name,
    detail: `provider ${row.id}`,
    keywords: [row.id, row.name, row.kind, "provider", "account", "inspect"],
    href: `/providers/${encodeURIComponent(row.id)}`,
  }));
}

function envelopeItems(family: "memory" | "skill", view: ResourceListView | undefined): CommandItem[] {
  const href = family === "memory" ? "/resources/memory" : "/resources/skill";
  return (view?.resources ?? []).map((row) => ({
    id: `object:${family}:${row.id}`,
    kind: "object" as const,
    execution: "navigate" as const,
    label: `${family} ${row.id}`,
    keywords: [row.id, family, "inspect"],
    href,
  }));
}

function toolItems(view: ToolCatalogView | undefined): CommandItem[] {
  return (view?.resources ?? []).map((row) => ({
    id: `object:tool:${row.operationId}`,
    kind: "object" as const,
    execution: "navigate" as const,
    label: row.operationId,
    detail: row.lifecycle,
    keywords: [row.operationId, "tool", row.family ?? "", "inspect"].filter(Boolean),
    href: "/resources/tool",
  }));
}

function bindingItems(rows: BindingView[] | undefined): CommandItem[] {
  return (rows ?? []).map((row) => ({
    id: `object:binding:${row.agent}:${row.accountId}`,
    kind: "object" as const,
    execution: "navigate" as const,
    label: `binding ${row.agent} → ${row.accountId}`,
    keywords: [row.agent, row.accountId, row.modelId, "binding", "inspect"],
    href: `/agents/${encodeURIComponent(row.agent)}`,
  }));
}

function alertActions(rows: ProviderAlertView[] | undefined): CommandItem[] {
  return (rows ?? [])
    .filter((row) => !row.acknowledged)
    .map((row) => ({
      id: `action:ack:${row.id}`,
      kind: "action" as const,
      execution: "acknowledge" as const,
      label: `Acknowledge ${row.id}`,
      detail: "Class-B; receipt stays in this tab",
      keywords: [row.id, "acknowledge", "alert", "repair"],
      alertId: row.id,
    }));
}

function decodeRest(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

/**
 * Route-derived class-A landings and class-B copies. Watch attach/detach is
 * not executed here: W11 owns the stream, and the palette only lands on Run.
 */
export function contextActions(pathname: string): CommandItem[] {
  const path = pathname.split("?")[0] ?? pathname;
  if (path.startsWith("/work/") && path !== "/work/new") {
    const ref = decodeRest(path.slice("/work/".length));
    if (ref === "" || ref === "new") {
      return [];
    }
    const encoded = encodeURIComponent(ref);
    return [
      {
        id: `action:copy-ref:${ref}`,
        kind: "action",
        execution: "copy-ref",
        label: "Copy task ref",
        detail: "Class-B; copies the current task_ref",
        keywords: [ref, "copy", "ref"],
        copyValue: ref,
        contextual: true,
      },
      {
        id: `action:evidence:${ref}`,
        kind: "action",
        execution: "navigate",
        label: "Open evidence",
        detail: "Verify lands on Work detail Evidence",
        keywords: [ref, "verify", "evidence", "inspect"],
        href: `/work/${encoded}?section=evidence`,
        contextual: true,
      },
      {
        id: `action:run:${ref}`,
        kind: "action",
        execution: "navigate",
        label: "Open Run",
        detail: "Watch attach/detach lives on that page; not a palette mutation",
        keywords: [ref, "watch", "attach", "run"],
        href: `/work/${encoded}?section=run`,
        contextual: true,
      },
    ];
  }
  if (path.startsWith("/providers/") && path !== "/providers/") {
    const id = decodeRest(path.slice("/providers/".length));
    if (id === "") {
      return [];
    }
    return [
      {
        id: `action:probe:${id}`,
        kind: "action",
        execution: "navigate",
        label: "Bounded probe",
        detail: "Lands on the account Models section",
        keywords: [id, "probe", "catalog", "run"],
        href: `/providers/${encodeURIComponent(id)}`,
        contextual: true,
      },
      {
        id: `action:bind:${id}`,
        kind: "action",
        execution: "navigate",
        label: "Change binding",
        detail: "Lands on the account Bindings section",
        keywords: [id, "binding", "configure"],
        href: `/providers/${encodeURIComponent(id)}`,
        contextual: true,
      },
    ];
  }
  if (path.startsWith("/agents/") && path !== "/agents/") {
    const id = decodeRest(path.slice("/agents/".length));
    if (id === "") {
      return [];
    }
    return [
      {
        id: `action:agent-bind:${id}`,
        kind: "action",
        execution: "navigate",
        label: "Change binding",
        detail: "Lands on the agent dossier",
        keywords: [id, "binding", "configure"],
        href: `/agents/${encodeURIComponent(id)}`,
        contextual: true,
      },
    ];
  }
  return [];
}

/**
 * Build the palette catalog from destinations plus whatever this tab has
 * already loaded. Unloaded lists do not appear — that is the BD-6 boundary.
 */
export function buildCommandCatalog(store: ProjectionStore, pathname = ""): CommandItem[] {
  const tasks =
    store.get<TaskEnvelopeView[]>(WORK_TASKS_KEY)?.data ??
    store.get<TaskEnvelopeView[]>(HOME_TASKS_KEY)?.data;
  const accounts = store.get<ProviderAccount[]>("providers:accounts")?.data;
  const bindings = store.get<BindingView[]>("bindings:all")?.data;
  const alerts = store.get<ProviderAlertView[]>(HOME_ALERTS_KEY)?.data;
  const memory = store.get<ResourceListView>(resourceListKey("memory"))?.data;
  const skill = store.get<ResourceListView>(resourceListKey("skill"))?.data;
  const tools = store.get<ToolCatalogView>(TOOL_CATALOG_KEY)?.data;

  return [
    ...contextActions(pathname),
    ...CREATE_ACTIONS,
    ...alertActions(alerts),
    ...taskItems(tasks),
    ...accountItems(accounts),
    ...namedAgents(),
    ...bindingItems(bindings),
    ...envelopeItems("memory", memory),
    ...envelopeItems("skill", skill),
    ...toolItems(tools),
    ...destinations(),
    HELP,
  ];
}

const KIND_ORDER: Record<CommandKind, number> = {
  action: 0,
  object: 1,
  destination: 2,
  help: 3,
};

function scoreItem(item: CommandItem, query: string): number {
  const id = String(item.id ?? "").toLowerCase();
  const label = String(item.label ?? "").toLowerCase();
  const keywords = item.keywords.map((word) => String(word ?? "").toLowerCase());
  if (id === query || label === query || item.copyValue?.toLowerCase() === query) return 0;
  const hay = [id, label, item.detail ?? "", item.copyValue ?? "", ...keywords].join("\n");
  if (!hay.includes(query) && !label.startsWith(query) && !id.endsWith(query)) return 99;
  if (item.contextual) return 1;
  if (keywords.includes(query)) return 2;
  if (label.startsWith(query) || id.endsWith(query)) return 3;
  return 4;
}

export function rankCommands(
  items: CommandItem[],
  query: string,
  recentIds: string[] = recentCommandIds(),
): CommandItem[] {
  const q = query.trim().toLowerCase();
  if (q === "") {
    const byId = new Map(items.map((item) => [item.id, item]));
    const recent = recentIds
      .map((id) => byId.get(id))
      .filter((item): item is CommandItem => item != null);
    const rest = items.filter(
      (item) =>
        !recentIds.includes(item.id) &&
        (item.kind === "action" ||
          item.kind === "destination" ||
          item.kind === "help" ||
          item.contextual === true),
    );
    return [...recent, ...rest];
  }
  const scored = items
    .map((item) => ({ item, score: scoreItem(item, q) }))
    .filter((entry) => entry.score < 99);
  scored.sort(
    (a, b) =>
      a.score - b.score ||
      KIND_ORDER[a.item.kind] - KIND_ORDER[b.item.kind] ||
      a.item.label.localeCompare(b.item.label),
  );
  return scored.map((entry) => entry.item);
}

export function groupCommands(items: CommandItem[], query: string): CommandGroup[] {
  const q = query.trim();
  const order: CommandKind[] =
    q === ""
      ? ["action", "object", "destination", "help"]
      : [...new Set(items.map((item) => item.kind))];
  return order
    .map((kind) => ({
      kind,
      label: COMMAND_GROUP_LABEL[kind],
      items: items.filter((item) => item.kind === kind),
    }))
    .filter((group) => group.items.length > 0);
}

export function catalogHasClassC(items: CommandItem[]): boolean {
  const banned = /\b(cancel|pause|retry|uninstall|upgrade)\b/i;
  return items.some((item) => banned.test(`${item.label} ${item.detail ?? ""} ${item.keywords.join(" ")}`));
}
