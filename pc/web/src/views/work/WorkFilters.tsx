import type { WorkOriginFilter } from "../../data/projections/work";

/**
 * Inventory scope control — docs/design/14 §3. The default is deliberately
 * the narrow, fully-explainable set: the refs this browser session observed.
 * Widening to the daemon envelope list is an explicit choice, and the copy
 * says what each scope actually contains.
 */
export function WorkFilters({
  origin,
  onOrigin,
  query,
  onQuery,
  sessionCount,
  totalCount,
}: {
  origin: WorkOriginFilter;
  onOrigin: (next: WorkOriginFilter) => void;
  query: string;
  onQuery: (next: string) => void;
  sessionCount: number;
  totalCount: number;
}) {
  return (
    <div className="cp-filters" role="group" aria-label="Inventory scope">
      <fieldset className="cp-fieldset">
        <legend className="cp-quiet">Scope</legend>
        <label className="cp-field">
          <input
            type="radio"
            name="work_origin"
            value="session"
            checked={origin === "session"}
            onChange={() => onOrigin("session")}
          />{" "}
          This session only ({sessionCount})
        </label>
        <label className="cp-field">
          <input
            type="radio"
            name="work_origin"
            value="all"
            checked={origin === "all"}
            onChange={() => onOrigin("all")}
          />{" "}
          Every ref this page has loaded ({totalCount})
        </label>
      </fieldset>
      <label className="cp-field">
        Filter by ref
        <input
          name="work_query"
          value={query}
          onChange={(event) => onQuery(event.target.value)}
          placeholder="task://…"
        />
      </label>
    </div>
  );
}
