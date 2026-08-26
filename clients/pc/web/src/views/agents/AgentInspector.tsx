import { Link } from "react-router-dom";
import { FactGrid } from "../../components/FactGrid";
import { Inspector } from "../../components/Inspector";
import {
  bindingSummary,
  type AgentInventoryRow,
} from "../../data/projections/agents";
import { HonestyNote } from "../../state/HonestyNote";
import { dossierPath } from "./AgentInventory";

export function AgentInspector({ row }: { row: AgentInventoryRow }) {
  return (
    <Inspector title={row.displayName} label="Agent inspector">
      <FactGrid
        facts={[
          { label: "actor", value: row.id },
          { label: "lifecycle", value: row.lifecycleLabel },
          { label: "lifecycle source", value: row.lifecycleSource },
          { label: "binding", value: bindingSummary(row) },
          { label: "dispatch", value: row.dispatch },
          { label: "current work", value: row.currentWorkLabel },
        ]}
      />
      <p className="cp-next">
        <Link to={dossierPath(row.id)}>Open dossier</Link>
      </p>
      <HonestyNote>
        Lifecycle verbs are not available over HTTP. This inspector does not pause, resume, stop or
        quarantine the actor.
      </HonestyNote>
    </Inspector>
  );
}
