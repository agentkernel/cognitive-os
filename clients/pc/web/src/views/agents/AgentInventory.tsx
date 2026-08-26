import { Link } from "react-router-dom";
import { MasterList } from "../../components/MasterList";
import { EmptyState } from "../../components/states";
import {
  agentLifecycleReading,
  bindingReading,
  bindingSummary,
  type AgentInventoryRow,
} from "../../data/projections/agents";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";

export function dossierPath(id: string): string {
  return `/agents/${encodeURIComponent(id)}`;
}

/**
 * Agents inventory — docs/design/16 §2. Rows lead with actor, governability
 * and current work. They are not an installation list: the runtime Resource
 * Manager family is projection-only and empty.
 */
export function AgentInventory({
  rows,
  selectedId,
  onSelect,
  source,
}: {
  rows: AgentInventoryRow[];
  selectedId?: string;
  onSelect: (row: AgentInventoryRow) => void;
  source: "pending" | "failed" | "answered";
}) {
  if (source === "pending") {
    return (
      <EmptyState title="Reading agent facts">
        Bindings and the dsh runtime snapshot have not both answered yet. This is a read in flight,
        not a statement that no agent exists.
      </EmptyState>
    );
  }
  if (source === "failed") {
    return (
      <EmptyState title="Agent facts could not be read">
        The binding list did not answer. pi and dsh are still the actors this HTTP surface can name,
        but dispatchability is unknown until the read succeeds.
      </EmptyState>
    );
  }

  return (
    <>
      <MasterList<AgentInventoryRow>
        caption="HTTP-addressable actors"
        rows={rows}
        rowKey={(row) => row.id}
        selectedKey={selectedId}
        onSelect={onSelect}
        columns={[
          {
            key: "state",
            header: "Lifecycle",
            render: (row) => <StateChip reading={agentLifecycleReading(row)} />,
          },
          {
            key: "actor",
            header: "Agent",
            mono: true,
            render: (row) => row.displayName,
          },
          {
            key: "binding",
            header: "Binding",
            render: (row) => (
              <>
                <StateChip reading={bindingReading(row)} />{" "}
                <span className="cp-quiet">{bindingSummary(row)}</span>
              </>
            ),
          },
          {
            key: "work",
            header: "Current work",
            render: (row) =>
              row.currentWorkKind === "task" && row.currentTaskRef ? (
                <Link to={`/work/${encodeURIComponent(row.currentTaskRef)}`}>{row.currentTaskRef}</Link>
              ) : (
                <span className="cp-quiet">{row.currentWorkLabel}</span>
              ),
          },
          {
            key: "detail",
            header: "Dossier",
            render: (row) => <Link to={dossierPath(row.id)}>Open</Link>,
          },
        ]}
      />
      <HonestyNote>
        This list is composed from <code>/management/agent-bindings</code> and{" "}
        <code>/personal/dsh/runtime</code>. <code>/management/resource/v1/list?family=runtime</code>{" "}
        is projection-only and returns no rows, so it is not an installation inventory. pi and dsh
        are the two agents the binding API can name. Registration, install and lifecycle stay on{" "}
        <code>cognitive</code> (BD-2). Process liveness is never treated as Task completion.
      </HonestyNote>
    </>
  );
}
