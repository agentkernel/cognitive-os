import { useRef, useSyncExternalStore } from "react";
import { appProjections, type Projection, type ProjectionStore } from "./store";

const ABSENT: Projection<never> = { status: "loading" };

/**
 * React bridge for the projection store. Snapshot is the stored object
 * reference (stable between writes) or a constant when absent.
 */
export function useProjection<T>(
  key: string,
  store: ProjectionStore = appProjections,
): Projection<T> {
  return useSyncExternalStore(
    (listener) => store.subscribe(listener),
    () => store.get<T>(key) ?? (ABSENT as Projection<T>),
  ) as Projection<T>;
}

/**
 * Same bridge for a set of keys whose size varies between renders (per-object
 * projections such as one effect history per known task ref). The snapshot
 * array is cached by element identity so useSyncExternalStore sees a stable
 * reference until one of the underlying projections is actually replaced.
 */
export function useProjections<T>(
  keys: readonly string[],
  store: ProjectionStore = appProjections,
): Projection<T>[] {
  const cache = useRef<{ keys: readonly string[]; value: Projection<T>[] }>({
    keys: [],
    value: [],
  });
  return useSyncExternalStore(
    (listener) => store.subscribe(listener),
    () => {
      const next = keys.map((key) => store.get<T>(key) ?? (ABSENT as Projection<T>));
      const previous = cache.current;
      const unchanged =
        previous.keys.length === keys.length &&
        previous.keys.every((key, index) => key === keys[index]) &&
        previous.value.every((projection, index) => projection === next[index]);
      if (unchanged) {
        return previous.value;
      }
      cache.current = { keys: [...keys], value: next };
      return next;
    },
  );
}

export interface LastGood<T> {
  /** The most recent data this component ever saw for this projection. */
  data?: T;
  updatedAt?: number;
  source?: string;
  /** True when the current projection itself is the live, non-degraded read. */
  live: boolean;
}

/**
 * Keep the last good read for a region. fetchProjection deliberately reports
 * a failed refresh as the failure rather than as current content (no data on
 * denied/disconnected/not-run), so the surface that still wants to show
 * last-known content must remember it — and label its age and source.
 */
export function useLastGood<T>(projection: Projection<T>): LastGood<T> {
  const kept = useRef<{ data?: T; updatedAt?: number; source?: string }>({});
  if (projection.data !== undefined) {
    kept.current = {
      data: projection.data,
      updatedAt: projection.updatedAt ?? kept.current.updatedAt,
      source: projection.source ?? kept.current.source,
    };
  }
  return {
    ...kept.current,
    live: projection.status === "ready" || projection.status === "empty",
  };
}
