import { useSyncExternalStore } from "react";
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
