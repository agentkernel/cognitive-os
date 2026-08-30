import { Link } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { HonestyNote } from "../../state/HonestyNote";

const LINUX_1_0 = [
  ["/", "Today"],
  ["/home", "Linux 1.0 Home"],
  ["/work", "Work"],
  ["/agents", "Agents"],
  ["/providers", "Providers"],
  ["/resources", "Resources"],
  ["/activity", "Activity"],
  ["/system", "System"],
  ["/session", "Session"],
] as const;

/**
 * Settings — bottom chrome, not L1. First slice is a hub of real existing
 * routes. Member-level budget is 2.1 / Deferred.
 */
export function SettingsPage() {
  return (
    <section data-page="opc-settings">
      <PageHeader
        title="Settings"
        lede="Account, Provider, and Linux 1.0 surfaces. Not Team. Not member budget."
      />
      <HonestyNote>
        Member-level budget hard-stop is 2.1 / Deferred. These links are existing
        daemon-served hash routes, not a new authority plane.
      </HonestyNote>
      <ul className="cp-nav">
        {LINUX_1_0.map(([to, label]) => (
          <li key={to}>
            <Link to={to}>{label}</Link>
          </li>
        ))}
      </ul>
    </section>
  );
}
