import { Link } from "react-router-dom";
import { ErrorState, LoadingState, UnavailableState } from "../../components/states";
import type { Projection } from "../../data/store";

/**
 * ProjectionState — the honest non-content states shared by every provider
 * surface. ready/empty/stale render content (null here); every failure class
 * gets its own designed state with the normalized error code. The daemon's
 * 200-stub (R-1) renders as unavailable, never as success.
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
