import { EmptyState, ErrorState, LoadingState, UnavailableState } from "../../components/states";
import { ReceiptLine } from "../../components/ReceiptLine";
import { HonestyNote } from "../../state/HonestyNote";
import { StateDot } from "../../state/StateDot";
import type { StateCategory } from "../../state/stateMap";

export const STATE_LAB_STATES = [
  "loading",
  "empty",
  "working",
  "error",
  "success",
  "partial",
  "blocked",
  "unknown",
  "offline",
] as const;

export const STATE_LAB_SURFACES = [
  "today",
  "create",
  "projects",
  "members",
  "runs",
  "outputs",
  "hitl",
  "knowledge",
  "settings",
] as const;

export type StateLabState = (typeof STATE_LAB_STATES)[number];
export type StateLabSurface = (typeof STATE_LAB_SURFACES)[number];

const STATE_CATEGORY: Record<StateLabState, StateCategory> = {
  loading: "unknown",
  empty: "unknown",
  working: "active",
  error: "blocked",
  success: "ready",
  partial: "attention",
  blocked: "blocked",
  unknown: "unknown",
  offline: "attention",
};

/**
 * Hidden Settings → Advanced state-lab. Real components, never L1.
 * Nine v9 states × nine product surfaces.
 */
export function StateLabGrid() {
  return (
    <div data-region="opc-state-lab-grid" className="cp-table-wrap">
      <table className="cp-table">
        <caption className="cp-quiet">
          Nine states × nine surfaces on real `/ui/` components. Hidden by
          default. Not a first-level destination. Unknown is never 0.
        </caption>
        <thead>
          <tr>
            <th>Surface</th>
            {STATE_LAB_STATES.map((state) => (
              <th key={state}>{state}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {STATE_LAB_SURFACES.map((surface) => (
            <tr key={surface} data-state-lab-surface={surface}>
              <th>{surface}</th>
              {STATE_LAB_STATES.map((state) => (
                <td
                  key={`${surface}-${state}`}
                  data-state-lab-cell={`${surface}:${state}`}
                >
                  <StateLabCell surface={surface} state={state} />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function StateLabCell({
  surface,
  state,
}: {
  surface: StateLabSurface;
  state: StateLabState;
}) {
  const category = STATE_CATEGORY[state];
  const label = `${surface} ${state}`;
  switch (state) {
    case "loading":
      return <LoadingState label={`Reading ${surface}. Last projection stays.`} />;
    case "empty":
      return (
        <EmptyState title={`${surface}: empty`}>
          No admitted {surface} object. This is not a demo row.
        </EmptyState>
      );
    case "working":
      return (
        <p className="cp-chip">
          <StateDot category={category} /> {label}. Working is not completion.
        </p>
      );
    case "error":
      return (
        <ErrorState
          what={`${surface} read failed`}
          why="Named daemon error. Input is retained."
          retryable={false}
        />
      );
    case "success":
      return (
        <ReceiptLine>
          {surface} receipt. Evidence-linked. Not a toast.
        </ReceiptLine>
      );
    case "partial":
      return (
        <HonestyNote>
          {surface} partial: some facts available, coverage is not ready.
        </HonestyNote>
      );
    case "blocked":
      return (
        <ErrorState
          what={`${surface} blocked`}
          why="Named dependency. Done work is safe."
          next="Go to the owning Project or Settings."
          retryable={false}
        />
      );
    case "unknown":
      return (
        <UnavailableState
          what={`${surface} unknown`}
          dependency="unknown is not 0 and not success"
        />
      );
    case "offline":
      return (
        <HonestyNote>
          {surface} stale / offline. Last-known facts stay. External actions
          cannot run.
        </HonestyNote>
      );
    default:
      return (
        <UnavailableState what={label} dependency="unmapped state-lab cell" />
      );
  }
}
