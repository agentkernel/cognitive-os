/*
 * Honest placeholder spaces (W1). Each says what the space will be, which
 * redesign wave delivers it (docs/design/39), and what is true right now —
 * including links to any legacy capability that remains available.
 * No fake content, no fake progress, no invented capability.
 */

import { Link } from "react-router-dom";
import { PageHeader } from "../components/PageHeader";
import { EmptyState } from "../components/states";
import { HonestyNote } from "../state/HonestyNote";

function Placeholder({
  title,
  wave,
  willBe,
  now,
}: {
  title: string;
  wave: string;
  willBe: string;
  now?: React.ReactNode;
}) {
  return (
    <section className="cp-placeholder">
      <PageHeader title={title} />
      <EmptyState title="Under reconstruction" action={now}>
        {willBe} <span className="cp-wave">Lands in {wave} of the Control Plane redesign.</span>
      </EmptyState>
    </section>
  );
}

export function AgentsPage() {
  return (
    <>
      <Placeholder
        title="Agents"
        wave="Wave 6"
        willBe="The actor dossier: the seven runtime identities, binding, capabilities, current work, activity — read-honest, with lifecycle control marked CLI-only until BD-2."
        now={<Link to="/providers">Govern agent provider bindings</Link>}
      />
      <HonestyNote>
        Agent lifecycle verbs (pause/resume/stop/restart/quarantine) are not available over HTTP
        (BD-2); they run through <code>cognitive</code> admin commands.
      </HonestyNote>
    </>
  );
}

export function ResourcesPage() {
  return (
    <Placeholder
      title="Resources"
      wave="Wave 7"
      willBe="The four cognitive-resource families with family-native depth: Memory, Skills, Tools, Context."
      now={null}
    />
  );
}

export function ActivityPage() {
  return (
    <Placeholder
      title="Activity"
      wave="Wave 8"
      willBe="The evidence stream — events, changes, effects, errors, interventions, verifications, acceptances — with its coverage honestly stated (no unified feed exists over HTTP yet, BD-5)."
      now={null}
    />
  );
}

export function SystemPage() {
  return (
    <Placeholder
      title="System"
      wave="Wave 9"
      willBe="Readiness detail, doctor, stewardship (backup/restore), session, and diagnostics."
      now={null}
    />
  );
}
