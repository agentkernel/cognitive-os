import { DigestChip } from "../../components/DigestChip";
import { MasterList } from "../../components/MasterList";
import { EmptyState } from "../../components/states";
import {
  inventoryFooter,
  workRowAge,
  workRowObjective,
  workRowReading,
  type WorkRow,
} from "../../data/projections/work";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";

const ORIGIN_LABEL: Record<string, string> = {
  envelope: "daemon envelope list",
  session: "observed this session",
  "envelope+session": "envelope list · observed this session",
};

/**
 * R1 inventory — docs/design/14 §3. A Tier-1 list of the tasks this page can
 * actually account for. It is not a task browser and does not pretend to be
 * complete: the daemon exposes no task-list-with-state route, so every row's
 * state comes from a real per-ref evidence read or reads `state not exposed`.
 *
 * Selecting a row opens the inspector. Nothing here edits anything.
 */
export function WorkInventory({
  rows,
  selectedRef,
  onSelect,
  nowMs,
  atBound,
}: {
  rows: WorkRow[];
  selectedRef?: string;
  onSelect: (row: WorkRow) => void;
  nowMs: number;
  atBound: boolean;
}) {
  if (rows.length === 0) {
    return (
      <>
        <EmptyState title="No task refs in this scope">
          This page knows of no task in the selected scope. Widening the scope shows every ref
          this page has loaded; it does not fetch more, because the daemon exposes no task
          search.
        </EmptyState>
        <HonestyNote>{inventoryFooter(0)}</HonestyNote>
      </>
    );
  }

  return (
    <>
      <MasterList<WorkRow>
        caption="Known tasks"
        rows={rows}
        rowKey={(row) => row.taskRef}
        selectedKey={selectedRef}
        onSelect={onSelect}
        columns={[
          {
            key: "state",
            header: "State",
            render: (row) => <StateChip reading={workRowReading(row)} />,
          },
          {
            key: "ref",
            header: "Task ref",
            mono: true,
            render: (row) => (
              <span title={row.taskRef}>
                {row.shortRef} <DigestChip value={row.taskRef} label="task ref" />
              </span>
            ),
          },
          {
            key: "objective",
            header: "Objective",
            render: (row) => workRowObjective(row),
          },
          {
            key: "epoch",
            header: "Contract epoch",
            mono: true,
            render: (row) => (row.contractEpoch == null ? "unknown" : String(row.contractEpoch)),
          },
          {
            key: "origin",
            header: "Source",
            render: (row) => ORIGIN_LABEL[row.origin] ?? row.origin,
          },
          {
            key: "age",
            header: "Observed",
            render: (row) => workRowAge(row, nowMs),
          },
        ]}
      />
      <HonestyNote>
        {inventoryFooter(rows.length)}. The daemon's task listing carries a task ref, a contract
        epoch and a digest — no lifecycle state and no objective — so a row shows a state only
        where a real <code>/task/evidence</code> read returned one. Refs observed this session are
        this tab&apos;s own memory, not an inventory, and they disappear when the tab closes.
        {atBound
          ? " The daemon list came back at its row bound, so further contracts may exist without appearing here."
          : ""}{" "}
        The task watch snapshot is not used here: its snapshot list is always empty and its event
        ring is process-local, so it does not represent task inventory.
      </HonestyNote>
    </>
  );
}
