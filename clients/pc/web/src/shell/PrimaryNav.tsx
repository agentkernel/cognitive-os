import { NavLink } from "react-router-dom";

/**
 * Personal 2.0 L1: Today / Projects / Knowledge. Settings lives in the
 * side-foot, not L1. Linux 1.0 six-family routes remain reachable as
 * secondary destinations (Settings hub + palette), not as Team/Inbox.
 */
export const PRIMARY_NAV = [
  ["/", "Today"],
  ["/projects", "Projects"],
  ["/knowledge", "Knowledge"],
] as const;

/** Linux 1.0 spaces kept as real routes, not L1 chrome. */
export const LINUX_1_0_NAV = [
  ["/home", "Home"],
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
          <NavLink to="/settings">Settings</NavLink>
        </p>
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
