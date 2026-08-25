/*
 * Resources hub (W7) view models — docs/design/18, reality map 28/31.
 *
 * The hub is a four-row family index, not a card wall. Memory, skill and tool
 * counts come from GET /management/resource/v1/list?family= (limit 64).
 * Context is per-task: its list is projection-only empty, and the entry
 * action points at Work rather than inventing a standalone browser.
 *
 * Memory list is non-tombstoned admitted rows only — this page never invents
 * a tombstone count. Skill list is bindings, not packages. Mutations
 * (remember/import/enable/forget) are not on the hub.
 *
 * No fetching happens here. Views drive fetchProjection.
 */

import type { Projection } from "../store";
import { asList, asRecord } from "../projections";

export const RESOURCE_LIST_LIMIT = 64;

export const HUB_FAMILIES = [
  { id: "memory", title: "Memory" },
  { id: "skill", title: "Skills" },
  { id: "tool", title: "Tools" },
  { id: "context", title: "Context" },
] as const;

export type HubFamilyId = (typeof HUB_FAMILIES)[number]["id"];

export function resourceListKey(family: HubFamilyId): string {
  return `resources:list:${family}`;
}

export function resourceListPath(family: HubFamilyId): string {
  return `/management/resource/v1/list?family=${family}`;
}

export interface ResourceEnvelope {
  id: string;
  family: string;
  health?: string;
}

export interface ResourceListView {
  family: string;
  authoritySource?: string;
  truncated: boolean;
  resources: ResourceEnvelope[];
}

export function projectResourceList(body: unknown): ResourceListView {
  const record = asRecord(body);
  return {
    family: String(record.family ?? "unknown"),
    authoritySource:
      record.authority_source == null ? undefined : String(record.authority_source),
    truncated: record.truncated === true,
    resources: asList(body, ["resources", "items"]).map((row) => {
      const item = asRecord(row);
      return {
        id: String(item.id ?? "unknown"),
        family: String(item.family ?? record.family ?? "unknown"),
        health: item.health == null ? undefined : String(item.health),
      };
    }),
  };
}

export function isProjectionOnly(view: ResourceListView): boolean {
  return view.authoritySource === "projection-only";
}

export function envelopeAtBound(view: ResourceListView): boolean {
  return view.truncated || view.resources.length >= RESOURCE_LIST_LIMIT;
}

export function envelopeLimitLabel(view: ResourceListView): string {
  return envelopeAtBound(view)
    ? `envelope at bound (limit ${RESOURCE_LIST_LIMIT})`
    : `envelope limit ${RESOURCE_LIST_LIMIT}`;
}

function countHealth(resources: ResourceEnvelope[], health: string): number {
  return resources.filter((row) => row.health === health).length;
}

/** Memory list is non-tombstoned admitted rows. Tombstone counts are not invented. */
export function memoryFact(view: ResourceListView): string {
  return `${view.resources.length} admitted · tombstones are not in this list · ${envelopeLimitLabel(view)}`;
}

/** Skill list is bindings, not packages. */
export function skillFact(view: ResourceListView): string {
  const bound = countHealth(view.resources, "bound");
  const revoked = countHealth(view.resources, "revoked");
  const other = view.resources.length - bound - revoked;
  const parts = [`${bound} bound`, `${revoked} revoked`];
  if (other > 0) {
    parts.push(`${other} unmapped health`);
  }
  parts.push("list is skill bindings, not packages");
  parts.push(envelopeLimitLabel(view));
  return parts.join(" · ");
}

export function toolFact(view: ResourceListView): string {
  const enabled = countHealth(view.resources, "enabled");
  const disabled = countHealth(view.resources, "disabled");
  const quarantined = countHealth(view.resources, "quarantined");
  const revoked = countHealth(view.resources, "revoked");
  const mapped = enabled + disabled + quarantined + revoked;
  const other = view.resources.length - mapped;
  const parts = [
    `${enabled} enabled`,
    `${disabled} disabled`,
    `${quarantined} quarantined`,
    `${revoked} revoked`,
  ];
  if (other > 0) {
    parts.push(`${other} unmapped health`);
  }
  parts.push(envelopeLimitLabel(view));
  return parts.join(" · ");
}

export function contextFact(view: ResourceListView): string {
  if (isProjectionOnly(view)) {
    return "per-task views — open from a task (no standalone HTTP browser)";
  }
  return "Context is per-task; this list is not a Context browser";
}

export type FamilyRowKind =
  | "pending"
  | "denied"
  | "disconnected"
  | "stub"
  | "unknown"
  | "projection-only"
  | "empty"
  | "ready";

export type FamilyRowAction =
  | { kind: "work"; href: "/work"; label: "Work" }
  | { kind: "browse"; href: string; label: "browse" }
  | { kind: "later"; label: "family page follows" };

export interface FamilyIndexRow {
  id: HubFamilyId;
  title: string;
  kind: FamilyRowKind;
  fact: string;
  action: FamilyRowAction;
}

function familyAction(id: HubFamilyId): FamilyRowAction {
  if (id === "context") {
    return { kind: "work", href: "/work", label: "Work" };
  }
  if (id === "memory") {
    return { kind: "browse", href: "/resources/memory", label: "browse" };
  }
  if (id === "skill") {
    return { kind: "browse", href: "/resources/skill", label: "browse" };
  }
  return { kind: "later", label: "family page follows" };
}

function populatedFact(id: HubFamilyId, view: ResourceListView): string {
  if (id === "memory") {
    return memoryFact(view);
  }
  if (id === "skill") {
    return skillFact(view);
  }
  if (id === "tool") {
    return toolFact(view);
  }
  return contextFact(view);
}

function emptyFact(id: HubFamilyId, view: ResourceListView): string {
  if (id === "memory") {
    return `${memoryFact(view)} · remember is on the family page`;
  }
  if (id === "skill") {
    return `${skillFact(view)} · import is on the family page`;
  }
  if (id === "tool") {
    return `${toolFact(view)} · enable is on the family page`;
  }
  return contextFact(view);
}

/**
 * One hub row. Failure of one family never invents a count, and
 * projection-only empty is never an authoritative empty family.
 */
export function readFamilyRow(
  family: (typeof HUB_FAMILIES)[number],
  projection: Projection<ResourceListView>,
): FamilyIndexRow {
  const action = familyAction(family.id);
  const base = { id: family.id, title: family.title, action };

  if (projection.status === "denied") {
    return {
      ...base,
      kind: "denied",
      fact: `session denied · ${projection.error?.code ?? "denied"}`,
    };
  }
  if (projection.status === "disconnected") {
    return { ...base, kind: "disconnected", fact: "daemon unreachable" };
  }
  if (projection.status === "not-run") {
    const code = projection.error?.code ?? "not-run";
    return {
      ...base,
      kind: "stub",
      fact:
        code === "STUB_ROUTE"
          ? "STUB_ROUTE — not an observed zero"
          : `${code} — this family list was not run`,
    };
  }
  if (projection.status === "unknown") {
    return {
      ...base,
      kind: "unknown",
      fact: `unexpected response · ${projection.error?.code ?? "unknown"}`,
    };
  }

  const data = projection.data;
  if (data && isProjectionOnly(data)) {
    return { ...base, kind: "projection-only", fact: contextFact(data) };
  }
  if (data && data.resources.length === 0) {
    return { ...base, kind: "empty", fact: emptyFact(family.id, data) };
  }
  if (data) {
    return { ...base, kind: "ready", fact: populatedFact(family.id, data) };
  }
  return { ...base, kind: "pending", fact: "reading the family list" };
}

export function composeFamilyRows(
  projections: Record<HubFamilyId, Projection<ResourceListView>>,
): FamilyIndexRow[] {
  return HUB_FAMILIES.map((family) => readFamilyRow(family, projections[family.id]));
}
