export type WatchState = "live" | "stale" | "disconnected" | "reconciling" | "unknown";

export type WatchEvent = {
  id: string;
  cursor: string;
  kind?: string;
  state?: string;
};

export type WatchController = {
  cursor: string | undefined;
  state: WatchState;
  events: WatchEvent[];
  accept(event: WatchEvent): void;
  noteGap(): void;
  reconnect(): void;
  detach(): { cancelledTask: false; stoppedAgent: false };
  inferCompletion(from: "process-exit" | "provider" | "pi" | "http"): "unknown";
};

export function createWatchController(initialCursor?: string): WatchController {
  const seen = new Set<string>();
  const events: WatchEvent[] = [];
  let cursor = initialCursor;
  let state: WatchState = initialCursor ? "live" : "unknown";

  return {
    get cursor() {
      return cursor;
    },
    get state() {
      return state;
    },
    get events() {
      return events;
    },
    accept(event: WatchEvent) {
      if (seen.has(event.id)) {
        return;
      }
      seen.add(event.id);
      events.push(event);
      cursor = event.cursor;
      state = "live";
    },
    noteGap() {
      state = "stale";
    },
    reconnect() {
      state = state === "stale" ? "reconciling" : "live";
    },
    detach() {
      state = "disconnected";
      return { cancelledTask: false, stoppedAgent: false };
    },
    inferCompletion() {
      return "unknown";
    },
  };
}
