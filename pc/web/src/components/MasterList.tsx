import type { ReactNode } from "react";

export interface MasterColumn<T> {
  key: string;
  header: string;
  render: (row: T) => ReactNode;
  /** Mono font for the column (digests/ids/counts). */
  mono?: boolean;
}

/**
 * MasterList — stable, dense, keyboard-navigable table. Selection by object
 * id (never index); rows are real buttons/links so focus and keyboard work.
 * The list never re-sorts under the pointer; callers control row order.
 */
export function MasterList<T>({
  caption,
  columns,
  rows,
  rowKey,
  selectedKey,
  onSelect,
  detailHref,
}: {
  caption: string;
  columns: MasterColumn<T>[];
  rows: T[];
  rowKey: (row: T) => string;
  selectedKey?: string;
  onSelect?: (row: T) => void;
  detailHref?: (row: T) => string;
}) {
  return (
    <table className="cp-table">
      <caption>{caption}</caption>
      <thead>
        <tr>
          {columns.map((column) => (
            <th key={column.key} scope="col">
              {column.header}
            </th>
          ))}
          {onSelect || detailHref ? <th scope="col" aria-label="actions" /> : null}
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => {
          const key = rowKey(row);
          const selected = selectedKey === key;
          return (
            <tr key={key} aria-selected={selected} data-row-key={key}>
              {columns.map((column) => (
                <td key={column.key} className={column.mono ? "cp-mono" : undefined}>
                  {column.render(row)}
                </td>
              ))}
              {onSelect || detailHref ? (
                <td>
                  {onSelect ? (
                    <button type="button" className="cp-button" onClick={() => onSelect(row)}>
                      Inspect
                    </button>
                  ) : null}
                  {detailHref ? (
                    <a className="cp-button" href={detailHref(row)}>
                      Open
                    </a>
                  ) : null}
                </td>
              ) : null}
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
