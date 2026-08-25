import type { AuditEventView } from "../../data/projections/providers";
import type { Projection } from "../../data/store";
import { HonestyNote } from "../../state/HonestyNote";
import { ProjectionState } from "./ProjectionState";

/**
 * Audit section — docs/design/17 §2.5. This account's provider-plane audit
 * events, newest first by audit id, bounded. The coverage note is
 * persistent: provider-plane audit is not a complete system event stream
 * (BD-5), and management actions outside the provider plane are not audited
 * over HTTP.
 */
export function AuditSection({
  accountId,
  audit,
}: {
  accountId: string;
  audit: Projection<AuditEventView[]>;
}) {
  const rows = (audit.data ?? [])
    .filter((event) => event.detail?.includes(accountId) || event.action.includes(accountId))
    .sort((a, b) => b.id.localeCompare(a.id))
    .slice(0, 50);

  return (
    <>
      <HonestyNote>
        Provider-plane audit only — not a complete system event stream (BD-5). Filtered to
        events naming this account; the daemon audit has no structured account field.
      </HonestyNote>
      <ProjectionState projection={audit} what="Audit" />
      {audit.status === "ready" || audit.status === "empty" || audit.status === "stale" ? (
        rows.length === 0 ? (
          <p className="cp-quiet">No provider-plane audit events name this account.</p>
        ) : (
          <table className="cp-table">
            <caption>Provider-plane audit events — newest first by audit id, bounded to 50</caption>
            <thead>
              <tr>
                <th scope="col">Audit id</th>
                <th scope="col">Action</th>
                <th scope="col">Outcome</th>
                <th scope="col">Detail</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((event) => (
                <tr key={event.id}>
                  <td className="cp-mono">{event.id}</td>
                  <td className="cp-mono">{event.action}</td>
                  <td className="cp-mono">{event.outcome}</td>
                  <td className="cp-mono">{event.detail ?? "unknown"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )
      ) : null}
    </>
  );
}
