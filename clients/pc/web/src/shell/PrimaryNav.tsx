import { NavLink } from "react-router-dom";

/**
 * Personal 2.0 L1: Today / Projects / Knowledge / Settings. Linux 1.0
 * Home / Work / Agents / Providers hashes are retired from Owner chrome
 * (P14-T07); leftover tests keep them via LinuxLegacyApp.
 */
export const PRIMARY_NAV = [
  ["/", "Today"],
  ["/projects", "Projects"],
  ["/knowledge", "Knowledge"],
  ["/settings", "Settings"],
] as const;

/** Retired Dual Track hashes — product App 404s these; leftover suites keep them. */
export const RETIRED_LINUX_1_0_NAV = [
  ["/home", "Home"],
  ["/work", "Work"],
  ["/agents", "Agents"],
  ["/providers", "Providers"],
] as const;

/** Remaining daemon surfaces that are not Owner L1 and not the retired hashes. */
export const LINUX_1_0_NAV = [
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
