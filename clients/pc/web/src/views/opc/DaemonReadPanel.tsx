import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { EmptyState, ErrorState, LoadingState, UnavailableState } from "../../components/states";
import type { Projection } from "../../data/store";

/**
 * Fail-closed honesty for one daemon GET. Empty, denied, disconnected, stub,
 * and unexpected stay distinct. EmptyState.action is never set.
 */
export function DaemonReadPanel<T>({
  projection,
  surface,
  emptyTitle,
  emptyBody,
  region,
  children,
}: {
  projection: Projection<T[]>;
  surface: string;
  emptyTitle: string;
  emptyBody: string;
  region: string;
  children?: ReactNode;
}) {
  if (projection.status === "loading") {
    return (
      <div data-region={region}>
        <LoadingState label={`Fetching ${surface} from the daemon.`} />
      </div>
    );
  }
  if (projection.status === "denied") {
    return (
      <div data-region={region}>
        <ErrorState
          what={`${surface}: session denied`}
          why={
            <>
              HTTP {projection.error?.httpStatus} ·{" "}
              <code className="cp-mono">{projection.error?.code ?? "denied"}</code> on{" "}
              <code className="cp-mono">{projection.source ?? "unknown"}</code>
            </>
          }
          next={<Link to="/session">Open Session</Link>}
          retryable={false}
        />
      </div>
    );
  }
  if (projection.status === "disconnected") {
    return (
      <div data-region={region}>
        <ErrorState
          what={`${surface}: daemon unreachable`}
          why="The daemon did not answer. Nothing is inferred as an empty list."
          retryable
        />
      </div>
    );
  }
  if (projection.status === "not-run") {
    return (
      <div data-region={region}>
        <UnavailableState
          what={surface}
          dependency={projection.error?.code ?? "not-run"}
        />
      </div>
    );
  }
  if (projection.status === "unknown") {
    return (
      <div data-region={region}>
        <ErrorState
          what={`${surface}: unexpected`}
          why={
            <>
              <code className="cp-mono">{projection.error?.code ?? "unknown"}</code>
              {projection.error?.message ? ` — ${projection.error.message}` : ""}
            </>
          }
          retryable
        />
      </div>
    );
  }
  if (
    projection.status === "empty" ||
    (projection.status === "ready" && (projection.data?.length ?? 0) === 0)
  ) {
    return (
      <div data-region={region}>
        <EmptyState title={emptyTitle}>{emptyBody}</EmptyState>
      </div>
    );
  }
  return <div data-region={region}>{children}</div>;
}
