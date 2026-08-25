/** Parse snapshot-first Task watch frames. Completion is never inferred. */

export type WatchFrame = {
  event: string;
  id?: string;
  data: unknown;
};

export function parseSse(text: string): WatchFrame[] {
  const frames: WatchFrame[] = [];
  for (const block of text.split("\n\n")) {
    if (!block.trim()) {
      continue;
    }
    let event = "message";
    let id: string | undefined;
    const dataLines: string[] = [];
    for (const line of block.split("\n")) {
      if (line.startsWith("event:")) {
        event = line.slice(6).trim();
      } else if (line.startsWith("id:")) {
        id = line.slice(3).trim();
      } else if (line.startsWith("data:")) {
        dataLines.push(line.slice(5).trimStart());
      }
    }
    let data: unknown = dataLines.join("\n");
    try {
      data = JSON.parse(String(data));
    } catch {
      /* keep raw text */
    }
    frames.push({ event, id, data });
  }
  return frames;
}

export function latestSequence(frames: WatchFrame[]): number | undefined {
  let latest: number | undefined;
  for (const frame of frames) {
    if (frame.id) {
      const parsed = Number(frame.id);
      if (Number.isFinite(parsed)) {
        latest = parsed;
      }
    }
    const data = frame.data;
    if (data && typeof data === "object" && "sequence" in data) {
      const sequence = Number((data as { sequence?: unknown }).sequence);
      if (Number.isFinite(sequence)) {
        latest = sequence;
      }
    }
  }
  return latest;
}

export function isWatchResumeStale(status: number, body: unknown): boolean {
  if (status !== 409) {
    return false;
  }
  return JSON.stringify(body ?? {}).includes("TASK_WATCH_RESUME_STALE");
}
