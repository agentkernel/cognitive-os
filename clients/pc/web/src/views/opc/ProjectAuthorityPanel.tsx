import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { EmptyState, ErrorState, LoadingState, UnavailableState } from "../../components/states";
import type { ProjectListRow } from "../../data/projections/projects";
import type { Projection } from "../../data/store";
import { HonestyNote } from "../../state/HonestyNote";

export const NO_PROJECT_EMPTY =
  "This daemon reports no Project. That is not an empty Task list and not an accepted OPC chrome. No action is available until a Project exists as daemon authority.";

export const TODAY_EMPTY_ONLY_CREATE =
  "This daemon reports no Project. That is not an empty Task list, not a decision packet, and not an accepted OPC chrome. Start create opens the five-step wizard. The wizard does not mint a Project until owner-management preview and write.";

export const TODAY_INCOMPLETE_ONLY_CREATE =
  "Create is not finished. Daily packets wait for activation. Today keeps only continue-create. This is not a decision packet, not a KPI wall, and not T13 empty chrome pretending packets are accepted.";

export const NO_FAKE_CHROME =
  "This slice does not paint Today packets, Team, Inbox, or Requires-backend controls.";

/**
 * Shared honesty for Today / Projects / Knowledge when Project authority is
 * missing. Empty, denied, disconnected, stub, and unexpected are distinct.
 * Today may pass emptyAction (Start create → wizard). Knowledge/Projects leave
 * EmptyState.action unset so they cannot grow a fake Activate control.
 */
export function ProjectAuthorityPanel({
  projection,
  surface,
  emptyBody,
  emptyAction,
  children,
}: {
  projection: Projection<ProjectListRow[]>;
  surface: string;
  emptyBody?: string;
  emptyAction?: ReactNode;
  children?: ReactNode;
}) {
  if (projection.status === "loading") {
    return <LoadingState label={`Fetching ${surface} Project list from the daemon.`} />;
  }
  if (projection.status === "denied") {
    return (
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
    );
  }
  if (projection.status === "disconnected") {
    return (
      <ErrorState
        what={`${surface}: daemon unreachable`}
        why="The daemon did not answer. Nothing is inferred as an empty Project list."
        retryable
      />
    );
  }
  if (projection.status === "not-run") {
    return (
      <UnavailableState
        what={`${surface} Project list`}
        dependency={projection.error?.code ?? "not-run"}
      />
    );
  }
  if (projection.status === "unknown") {
    return (
      <ErrorState
        what={`${surface}: Project list unexpected`}
        why={
          <>
            <code className="cp-mono">{projection.error?.code ?? "unknown"}</code>
            {projection.error?.message ? ` — ${projection.error.message}` : ""}
          </>
        }
        retryable
      />
    );
  }
  if (projection.status === "empty" || (projection.status === "ready" && (projection.data?.length ?? 0) === 0)) {
    return (
      <EmptyState title={`${surface}: no Project`} action={emptyAction}>
        {emptyBody ?? `${NO_PROJECT_EMPTY} ${NO_FAKE_CHROME}`}
      </EmptyState>
    );
  }
  return (
    <>
      <HonestyNote>
        Rows are the daemon list. Cost and title stay as stated. Completing a
        Project is not a model reply.
      </HonestyNote>
      {children}
    </>
  );
}
