import { FactGrid } from "../../../components/FactGrid";
import { factOrUnknown } from "../../../data/projections/work";
import {
  DETAIL_UNAVAILABLE_VIEWS,
  WATCH_NOT_ATTACHED,
  type CompletionReading,
  type TaskEvidenceDetail,
} from "../../../data/projections/workDetail";
import { HonestyNote } from "../../../state/HonestyNote";

/**
 * Overview — docs/design/15 §3.1. What this task is, what the daemon can say
 * about it right now, and an explicit account of what it cannot say.
 */
export function OverviewSection({
  evidence,
  completion,
  objective,
}: {
  evidence?: TaskEvidenceDetail;
  completion: CompletionReading;
  objective?: string;
}) {
  return (
    <section className="cp-region" id="section-overview" aria-labelledby="overview-title">
      <h3 className="cp-section-title" id="overview-title">
        Overview
      </h3>
      <FactGrid
        facts={[
          {
            label: "Objective",
            value:
              objective ??
              "not exposed — the daemon's task listing and evidence carry no objective, and this session did not admit this ref",
          },
          {
            label: "Agent",
            value: "unknown — the daemon exposes no task→agent binding over HTTP",
          },
          {
            label: "Disposition",
            value: completion.label,
          },
          {
            label: "Intents recorded",
            value: <span className="cp-mono">{factOrUnknown(evidence?.intentRefs.length)}</span>,
          },
          {
            label: "Effects recorded",
            value: <span className="cp-mono">{factOrUnknown(evidence?.effectRefs.length)}</span>,
          },
          {
            label: "Durable event cursor",
            value: (
              <span className="cp-mono">
                {factOrUnknown(evidence?.durableCursor?.eventSequence)}
              </span>
            ),
          },
        ]}
      />
      <p className="cp-quiet">{completion.detail}</p>

      <h4 className="cp-section-title">Watch</h4>
      <p className="cp-reason" role="status">
        Watch is <strong>{WATCH_NOT_ATTACHED.state}</strong>. {WATCH_NOT_ATTACHED.detail}
      </p>

      <h4 className="cp-section-title">Not exposed over HTTP</h4>
      <ul className="cp-plain-list">
        {DETAIL_UNAVAILABLE_VIEWS.map((entry) => (
          <li key={entry.subject}>
            <strong>{entry.subject}</strong> — unavailable: {entry.reason}.
          </li>
        ))}
      </ul>
      <HonestyNote>
        Each line above is a named gap in the daemon&apos;s HTTP surface, not a gap in this task.
        This page will not infer a decision trace, an attention set or a context assembly it cannot
        read.
      </HonestyNote>
    </section>
  );
}
