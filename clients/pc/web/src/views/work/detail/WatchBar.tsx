import { StateChip } from "../../../state/StateChip";
import { readDomainState } from "../../../state/stateMap";
import {
  WATCH_DETACH_NOTE,
  WATCH_POLL_INTERVAL_MS,
  WATCH_RING_NOTE,
  WATCH_TRANSPORT_NOTE,
  type WatchSessionSnapshot,
} from "../../../watchStream";

/**
 * Attach/detach/reconnect for GET /task/watch. Class-B observation only:
 * detach is never cancel, and a live chip is never a completion claim.
 */
export function WatchBar({
  snapshot,
  onAttach,
  onDetach,
  onReconnect,
  variant,
}: {
  snapshot: WatchSessionSnapshot;
  onAttach: () => void;
  onDetach: () => void;
  onReconnect: () => void;
  variant: "header" | "run" | "facts";
}) {
  const attached = snapshot.phase === "attached";
  const detached = snapshot.phase === "detached";
  const reading = readDomainState("watch", snapshot.state);

  return (
    <div className="cp-watch" data-variant={variant}>
      <p className="cp-watch-status" role="status">
        <StateChip reading={reading} /> Watch is <strong>{snapshot.label}</strong>
        {snapshot.cursor ? (
          <>
            {" "}
            · cursor <code className="cp-mono">{snapshot.cursor}</code>
          </>
        ) : null}
        {attached && snapshot.delivery === "bounded-poll" ? (
          <>
            {" "}
            · {WATCH_POLL_INTERVAL_MS / 1000} s bounded poll
          </>
        ) : null}
      </p>
      <p className="cp-next">
        {attached ? (
          <button type="button" className="cp-button" onClick={onDetach}>
            Detach watch
          </button>
        ) : (
          <button type="button" className="cp-button" onClick={onAttach}>
            Attach watch
          </button>
        )}
        {attached && snapshot.state === "stale" ? (
          <>
            {" "}
            <button type="button" className="cp-button" onClick={onReconnect}>
              Reconnect snapshot
            </button>
          </>
        ) : null}
        {detached ? (
          <>
            {" "}
            <button type="button" className="cp-button" onClick={onReconnect}>
              Reconnect snapshot
            </button>
          </>
        ) : null}
      </p>
      {snapshot.lastError ? (
        <p className="cp-reason" role="status">
          {snapshot.lastError}
        </p>
      ) : null}
      {variant === "run" ? (
        <>
          <p className="cp-quiet">{WATCH_TRANSPORT_NOTE}</p>
          <p className="cp-quiet">{WATCH_RING_NOTE}</p>
          <p className="cp-quiet">{WATCH_DETACH_NOTE}</p>
        </>
      ) : variant === "header" ? (
        <p className="cp-quiet">{WATCH_DETACH_NOTE}</p>
      ) : (
        <p className="cp-quiet">
          {attached
            ? "Watch controls on this page are observation-only."
            : "Watch is not attached from this inspector."}{" "}
          {WATCH_DETACH_NOTE}
        </p>
      )}
    </div>
  );
}
