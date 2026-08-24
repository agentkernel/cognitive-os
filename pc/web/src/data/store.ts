/*
 * Minimal zero-dependency projection store (owner decision OQ-6).
 * A projection is what a view reads: {status, data, error, cursor, source,
 * updatedAt}. Stores are immutable-replace (set swaps the object), so
 * useSyncExternalStore snapshots stay referentially stable between writes.
 */

import type { NormalizedError } from "./normalize";

export type ProjectionStatus =
  | "loading"
  | "ready"
  | "empty"
  | "denied"
  | "disconnected"
  | "unknown"
  | "not-run"
  | "stale";

export interface Projection<T> {
  status: ProjectionStatus;
  data?: T;
  error?: NormalizedError;
  cursor?: number;
  /** Which daemon route produced this projection. */
  source?: string;
  updatedAt?: number;
}

export interface ProjectionStore {
  get<T>(key: string): Projection<T> | undefined;
  set<T>(key: string, value: Projection<T>): void;
  subscribe(listener: () => void): () => void;
}

export function createProjectionStore(): ProjectionStore {
  const entries = new Map<string, Projection<unknown>>();
  const listeners = new Set<() => void>();
  return {
    get<T>(key: string): Projection<T> | undefined {
      return entries.get(key) as Projection<T> | undefined;
    },
    set<T>(key: string, value: Projection<T>): void {
      entries.set(key, value as Projection<unknown>);
      for (const listener of listeners) {
        listener();
      }
    },
    subscribe(listener: () => void): () => void {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

/** App-wide projection store (one per SPA instance; tests create their own). */
export const appProjections: ProjectionStore = createProjectionStore();
