/**
 * P14-T06/D01 Today live surface. Packets are fetched per live Project
 * (creating drafts are not subjects). The run overview is one daemon GET
 * for the period. This module never writes authority and never treats
 * empty chrome as packet acceptance.
 */

import { fetchProjection } from "../../data/fetchProjection";
import {
  HITL_KEY,
  pendingPreviewsPath,
  projectPendingPreviews,
  type PendingPreviewRow,
} from "../../data/projections/hitl";
import { liveProjectRows, type ProjectListRow } from "../../data/projections/projects";
import type { TodayPeriod } from "../../data/projections/todayOverview";
import { appProjections, type Projection } from "../../data/store";
import { loadTodayOverview } from "./loadOpcReads";

export function livePacketKey(projectId: string): string {
  return `${HITL_KEY}:${projectId}`;
}

export function mergeLivePacketProjection(
  liveIds: string[],
  projections: Projection<PendingPreviewRow[]>[],
): Projection<PendingPreviewRow[]> {
  if (liveIds.length === 0) {
    return { status: "empty", data: [] };
  }
  const live = new Set(liveIds);
  for (const projection of projections) {
    if (
      projection.status === "denied" ||
      projection.status === "disconnected" ||
      projection.status === "not-run" ||
      projection.status === "unknown"
    ) {
      return projection;
    }
  }
  if (projections.length === 0 || projections.every((projection) => projection.status === "loading")) {
    return { status: "loading" };
  }
  const packets: PendingPreviewRow[] = [];
  const seen = new Set<string>();
  for (const projection of projections) {
    if (
      projection.status !== "ready" &&
      projection.status !== "empty" &&
      projection.status !== "stale"
    ) {
      continue;
    }
    for (const row of projection.data ?? []) {
      if (!live.has(row.subjectRef) || seen.has(row.previewId)) {
        continue;
      }
      seen.add(row.previewId);
      packets.push(row);
    }
  }
  return {
    status: packets.length === 0 ? "empty" : "ready",
    data: packets,
  };
}

export async function loadTodayLiveSurface(
  list: Projection<ProjectListRow[]>,
  period: TodayPeriod,
): Promise<void> {
  const live = liveProjectRows(list.status === "ready" ? list.data : undefined);
  if (live.length === 0) {
    return;
  }
  const liveIds = live.map((row) => row.projectId);
  await Promise.all([
    ...live.map((row) =>
      fetchProjection(
        appProjections,
        livePacketKey(row.projectId),
        pendingPreviewsPath(row.projectId),
        "management",
        projectPendingPreviews,
      ),
    ),
    loadTodayOverview(period),
  ]);
  const packetProjections: Projection<PendingPreviewRow[]>[] = liveIds.map(
    (id) => appProjections.get<PendingPreviewRow[]>(livePacketKey(id)) ?? { status: "loading" },
  );
  appProjections.set(HITL_KEY, mergeLivePacketProjection(liveIds, packetProjections));
}
