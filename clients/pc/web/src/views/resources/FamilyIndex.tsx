import { Link } from "react-router-dom";
import type { FamilyIndexRow } from "../../data/projections/resources";

/**
 * Four quiet index rows — docs/design/18 §1. Not a card wall. Context's
 * entry is Work; the other families name that their pages follow this hub.
 */
export function FamilyIndex({ rows }: { rows: FamilyIndexRow[] }) {
  return (
    <table className="cp-table cp-family-index">
      <caption>Resource families</caption>
      <thead>
        <tr>
          <th scope="col">Family</th>
          <th scope="col">Observed from list</th>
          <th scope="col">Entry</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.id} data-family={row.id} data-family-kind={row.kind}>
            <th scope="row">{row.title}</th>
            <td>{row.fact}</td>
            <td>
              {row.action.kind === "work" ? (
                <Link to={row.action.href}>{row.action.label}</Link>
              ) : (
                <span className="cp-quiet">{row.action.label}</span>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
