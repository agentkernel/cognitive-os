import { useCallback, useEffect, useRef, useState } from "react";
import {
  createWatchSession,
  WATCH_POLL_INTERVAL_MS,
  type WatchSessionSnapshot,
} from "../../../watchStream";

const UNATTACHED: WatchSessionSnapshot = {
  phase: "unattached",
  state: "unknown",
  label: "not attached",
  cursor: undefined,
  events: [],
  resumeFrom: undefined,
  delivery: "unattached",
};

/**
 * Per-task-page watch session. Starts unattached (unknown, not live).
 * Unmount and taskRef change detach the client observation only.
 */
export function useTaskWatch(taskRef: string): {
  snapshot: WatchSessionSnapshot;
  attach: () => Promise<void>;
  detach: () => { cancelledTask: false; stoppedAgent: false };
  reconnect: () => Promise<void>;
} {
  const [snapshot, setSnapshot] = useState<WatchSessionSnapshot>(UNATTACHED);
  const sessionRef = useRef<ReturnType<typeof createWatchSession> | null>(null);

  useEffect(() => {
    const session = createWatchSession({
      onChange: (next) => setSnapshot(next),
    });
    sessionRef.current = session;
    setSnapshot(session.snapshot());
    return () => {
      session.detach({ silent: true });
      sessionRef.current = null;
    };
  }, [taskRef]);

  useEffect(() => {
    if (snapshot.phase !== "attached" || snapshot.state === "stale") {
      return;
    }
    const id = window.setInterval(() => {
      void sessionRef.current?.poll();
    }, WATCH_POLL_INTERVAL_MS);
    return () => window.clearInterval(id);
  }, [snapshot.phase, snapshot.state]);

  const attach = useCallback(async () => {
    await sessionRef.current?.attach();
  }, []);
  const detach = useCallback(() => {
    return sessionRef.current?.detach() ?? { cancelledTask: false as const, stoppedAgent: false as const };
  }, []);
  const reconnect = useCallback(async () => {
    await sessionRef.current?.reconnect();
  }, []);

  return { snapshot, attach, detach, reconnect };
}
