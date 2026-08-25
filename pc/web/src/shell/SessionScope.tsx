import { createContext, useCallback, useState, type ReactNode } from "react";

/**
 * Session tick context — bumped whenever session memory changes so gated
 * surfaces re-render. Moved verbatim from the pre-refactor App.tsx.
 */
export const SessionTick = createContext({ tick: 0, bump: () => {} });

export function SessionScope({ children }: { children: ReactNode }) {
  const [tick, setTick] = useState(0);
  const bump = useCallback(() => setTick((value) => value + 1), []);
  return <SessionTick.Provider value={{ tick, bump }}>{children}</SessionTick.Provider>;
}
