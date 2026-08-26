import { NavLink } from "react-router-dom";

/**
 * Primary navigation — the seven frozen spaces (owner-frozen labels,
 * docs/design/06 DD-02/DD-06). Session is chrome (status strip), never a
 * peer. Counts are shown only when they change behavior and are backed by
 * real data (none in W1 — alerts live in the strip).
 */
export const PRIMARY_NAV = [
  ["/", "Home"],
  ["/work", "Work"],
  ["/agents", "Agents"],
  ["/providers", "Providers"],
  ["/resources", "Resources"],
  ["/activity", "Activity"],
  ["/system", "System"],
] as const;

export function PrimaryNav({
  onOpenPalette,
  paletteOpen = false,
}: {
  onOpenPalette?: () => void;
  paletteOpen?: boolean;
}) {
  return (
    <div className="cp-side">
      <div className="cp-brand">
        <span className="cp-brand-mark" aria-hidden="true" />
        <div>
          <h1>CognitiveOS Personal</h1>
          <p>Daemon client · not an authority writer</p>
        </div>
      </div>
      <nav aria-label="Primary">
        <ul className="cp-nav">
          {PRIMARY_NAV.map(([to, label]) => (
            <li key={to}>
              <NavLink to={to} end={to === "/"}>
                {label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>
      <div className="cp-side-foot">
        <p>
          <button
            type="button"
            className="cp-button"
            aria-label="Open command palette"
            aria-haspopup="dialog"
            aria-expanded={paletteOpen}
            aria-keyshortcuts="Control+K Meta+K Slash"
            onClick={() => onOpenPalette?.()}
          >
            ⌘K
          </button>
        </p>
        <p>Unknown is a value. Nothing here is inferred.</p>
      </div>
    </div>
  );
}
