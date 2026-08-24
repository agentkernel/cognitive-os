import { Link } from "react-router-dom";
import { ErrorState, LoadingState, UnavailableState } from "../../components/states";
import type { Projection } from "../../data/store";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";

/** "as of <time>" for last-good content; never a guessed freshness. */
export function asOfLabel(updatedAt: number | undefined): string {
  return updatedAt ? new Date(updatedAt).toLocaleTimeString() : "unknown";
}

/**
 * ProjectionState — the honest non-content states shared by every provider
 * surface. ready/empty render content (null here); stale renders content too,
 * with a last-good marker naming its age and source (docs/design/22) so a
 * refresh never blanks the surface and never claims to be current. Every
 * failure class gets its own designed state with the normalized error code.
 * The daemon's 200-stub (R-1) renders as unavailable, never as success.
 */
export function ProjectionState({
  projection,
  what,
}: {
  projection: Projection<unknown>;
  what: string;
}) {
  switch (projection.status) {
    case "loading":
      return <LoadingState label={`Fetching ${what} from the daemon.`} />;
    case "stale":
      return (
        <p className="cp-quiet">
          <StateChip reading={readDomainState("load", "stale")} /> {what} below is the last
          good read, as of {asOfLabel(projection.updatedAt)} · Source:{" "}
          <code className="cp-mono">{projection.source ?? "unknown"}</code>. A refresh is in
          flight.
        </p>
      );
    case "denied":
      return (
        <ErrorState
          what={`${what}: session denied`}
          why={
            <>
              HTTP {projection.error?.httpStatus} ·{" "}
              <code className="cp-mono">{projection.error?.code ?? "denied"}</code> on the
              management channel
            </>
          }
          next={<Link to="/session">Open Session</Link>}
          retryable={false}
        />
      );
    case "disconnected":
      return (
        <ErrorState
          what={`${what}: daemon unreachable`}
          why="The daemon did not answer; the last known state is not shown as current."
          retryable
        />
      );
    case "not-run":
      return (
        <UnavailableState
          what={what}
          dependency={
            projection.error?.code === "STUB_ROUTE"
              ? "daemon front-door stub (R-1)"
              : (projection.error?.code ?? "not-run")
          }
        />
      );
    case "unknown":
      return (
        <ErrorState
          what={`${what}: unexpected response`}
          why={
            <>
              <code className="cp-mono">{projection.error?.code ?? "unknown"}</code>
              {projection.error?.message ? ` — ${projection.error.message}` : ""}
            </>
          }
          retryable
        />
      );
    default:
      return null;
  }
}
