import { Link } from "react-router-dom";
import { LoadingState } from "../../components/states";
import type { LastGood } from "../../data/useProjection";
import type { Projection } from "../../data/store";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";
import { asOfLabel } from "../providers/ProjectionState";

/**
 * Region-level load state for Home. Unlike the provider surfaces, a Home
 * region keeps its last good content on screen through a failed refresh
 * (docs/design/13 §5: disconnected/partial keep last-known, labelled), so
 * this renders the failure *and* the age/source of what is still shown —
 * never a blank region, never a silent claim of freshness.
 *
 * One region's failure is stated here and nowhere else: the other regions
 * read their own projections and are unaffected.
 */
export function RegionStatus({
  projection,
  lastGood,
  what,
}: {
  projection: Projection<unknown>;
  lastGood: LastGood<unknown>;
  what: string;
}) {
  const failed =
    projection.status === "denied" ||
    projection.status === "disconnected" ||
    projection.status === "unknown" ||
    projection.status === "not-run";

  if (projection.status === "loading" && lastGood.data === undefined) {
    return <LoadingState label={`Fetching ${what} from the daemon.`} />;
  }

  const showLastGood = !lastGood.live && lastGood.data !== undefined;

  return (
    <>
      {failed ? (
        <p className="cp-reason" role="alert">
          <StateChip reading={readDomainState("load", projection.status)} /> {what} could not be
          read from <code className="cp-mono">{projection.source ?? "unknown"}</code> —{" "}
          <code className="cp-mono">{projection.error?.code ?? projection.status}</code>
          {projection.error?.message ? ` — ${projection.error.message}` : ""}. This region is
          reduced; the other regions are unaffected.
          {projection.status === "denied" ? (
            <>
              {" "}
              <Link to="/session">Open Session</Link>
            </>
          ) : null}
        </p>
      ) : null}
      {projection.status === "stale" ? (
        <p className="cp-quiet">
          <StateChip reading={readDomainState("load", "stale")} /> A refresh of {what} is in
          flight. The rows below are the last good read, as of{" "}
          {asOfLabel(lastGood.updatedAt)}.
        </p>
      ) : null}
      {showLastGood && projection.status !== "stale" ? (
        <p className="cp-quiet">
          Showing the last known {what}, as of {asOfLabel(lastGood.updatedAt)} · Source:{" "}
          <code className="cp-mono">{lastGood.source ?? "unknown"}</code>. It is not claimed as
          current.
        </p>
      ) : null}
    </>
  );
}
