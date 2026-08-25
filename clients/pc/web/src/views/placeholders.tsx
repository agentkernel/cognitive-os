/*
 * Honest placeholder spaces (W1). Each says what the space will be, which
 * redesign wave delivers it (docs/design/39), and what is true right now —
 * including links to any legacy capability that remains available.
 * No fake content, no fake progress, no invented capability.
 */

import { PageHeader } from "../components/PageHeader";
import { EmptyState } from "../components/states";

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
