import { DigestChip } from "../../../components/DigestChip";
import {
  WATCH_NOT_ATTACHED,
  type AuthorityLaneRow,
  type ObservationLaneRow,
} from "../../../data/projections/workDetail";
import type { Projection } from "../../../data/store";
import { HonestyNote } from "../../../state/HonestyNote";
import { StateChip } from "../../../state/StateChip";
import { readDomainState } from "../../../state/stateMap";
import type { WatchSessionSnapshot } from "../../../watchStream";
import { WatchBar } from "./WatchBar";

/**
 * Run — docs/design/15 §3.2. Two lanes, drawn as two lanes.
 *
 * The authority lane is what the daemon durably recorded as a state change.
 * The observation lane is bounded sampling plus watch deltas. They are never
 * interleaved into one list, because an observation that looked like a
 * transition would be the single most misleading thing this product could
 * render: it would let a sampled counter or a watch frame read as evidence
 * that the task moved.
 */
export function RunTimeline({
  authority,
  observation,
  evidenceProjection,
  observationProjections,
  watch,
  onAttach,
  onDetach,
  onReconnect,
}: {
  authority: AuthorityLaneRow[];
  observation: ObservationLaneRow[];
  evidenceProjection?: Projection<unknown>;
  observationProjections: Projection<unknown>[];
  watch: WatchSessionSnapshot;
  onAttach: () => void;
  onDetach: () => void;
  onReconnect: () => void;
}) {
  const observationFailures = observationProjections.filter(
    (projection) => projection.data === undefined && projection.status !== "loading",
  );

  return (
    <section className="cp-region" id="section-run" aria-labelledby="run-title">
      <h3 className="cp-section-title" id="run-title">
        Run
      </h3>
      <p className="cp-quiet">
        Two independent lanes. The authority lane is the daemon&apos;s durable record of state
        changes; the observation lane is bounded sampling plus watch deltas. Nothing in the
        observation lane is a state transition.
      </p>

      <WatchBar
        snapshot={watch}
        onAttach={onAttach}
        onDetach={onDetach}
        onReconnect={onReconnect}
        variant="run"
      />

      <div className="cp-lanes">
        <div className="cp-lane cp-lane--authority" aria-labelledby="lane-authority">
          <h4 className="cp-lane-title" id="lane-authority">
            Authority lane · <code>/task/evidence</code> lifecycle transitions
          </h4>
          {evidenceProjection && evidenceProjection.data === undefined ? (
            <p className="cp-reason" role="status">
              <StateChip reading={readDomainState("load", evidenceProjection.status)} /> The
              lifecycle record could not be read —{" "}
              <code className="cp-mono">
                {evidenceProjection.error?.code ?? evidenceProjection.status}
              </code>
              . No transition is shown and none is assumed.
            </p>
          ) : (
            <ol className="cp-lane-list">
              {authority.map((row, index) => {
                if (row.kind === "transition") {
                  return (
                    <li key={`t-${row.transition.eventRef}-${index}`} className="cp-lane-row">
                      <span className="cp-lane-marker" aria-hidden="true" />
                      <div>
                        <p className="cp-lane-head">
                          <StateChip
                            reading={
                              row.transition.afterState
                                ? readDomainState("task", row.transition.afterState)
                                : {
                                    category: "unknown",
                                    label: "state not carried by this event",
                                    unmapped: false,
                                  }
                            }
                          />{" "}
                          <code className="cp-mono">{row.transition.eventType}</code>
                        </p>
                        <p className="cp-quiet">
                          version <code className="cp-mono">{row.transition.afterVersion}</code> ·
                          sequence <code className="cp-mono">{row.transition.sequence}</code> ·{" "}
                          {row.transition.eventTime ?? "no event time recorded"}
                          {row.transition.reasonCode ? (
                            <>
                              {" "}
                              · reason <code className="cp-mono">{row.transition.reasonCode}</code>
                            </>
                          ) : null}
                        </p>
                        <p className="cp-quiet">
                          <DigestChip value={row.transition.eventRef} label="event ref" />
                        </p>
                      </div>
                    </li>
                  );
                }
                return (
                  <li key={`n-${row.kind}-${index}`} className="cp-lane-row cp-lane-row--note">
                    <span className="cp-lane-marker cp-lane-marker--gap" aria-hidden="true" />
                    <p className="cp-reason" role="status">
                      {row.kind === "bounded" ? <strong>Bounded. </strong> : null}
                      {row.note}
                    </p>
                  </li>
                );
              })}
            </ol>
          )}
        </div>

        <div className="cp-lane cp-lane--observation" aria-labelledby="lane-observation">
          <h4 className="cp-lane-title" id="lane-observation">
            Observation lane · bounded <code>/task/observation</code> samples and watch deltas
          </h4>
          <p className="cp-quiet">
            Samples, named zeros, and watch frames. An entry here never means the task moved.
          </p>
          {observationFailures.length > 0 ? (
            <p className="cp-reason" role="status">
              <StateChip reading={readDomainState("load", observationFailures[0].status)} />{" "}
              {observationFailures.length} observation famil
              {observationFailures.length === 1 ? "y" : "ies"} could not be read —{" "}
              <code className="cp-mono">
                {observationFailures[0].error?.code ?? observationFailures[0].status}
              </code>
              . That is a missing measurement, not an observed zero.
            </p>
          ) : null}
          <ul className="cp-lane-list">
            {observation.map((row, index) => {
              if (row.kind === "counter") {
                return (
                  <li key={`c-${row.counter.name}-${index}`} className="cp-lane-row">
                    <span className="cp-lane-marker cp-lane-marker--sample" aria-hidden="true" />
                    <div>
                      <p className="cp-lane-head">
                        <code className="cp-mono">{row.counter.name}</code>{" "}
                        <span className="cp-quiet">observation</span>
                      </p>
                      <p className="cp-quiet">
                        count <code className="cp-mono">{row.counter.count}</code> · denominator{" "}
                        <code className="cp-mono">{row.counter.denominator}</code> ·{" "}
                        <code className="cp-mono">{row.counter.negativeControl}</code>
                        {row.counter.observedZero ? " · observed zero, not an inferred zero" : ""}
                      </p>
                    </div>
                  </li>
                );
              }
              if (row.kind === "sample") {
                return (
                  <li key={`s-${row.detail}-${index}`} className="cp-lane-row">
                    <span className="cp-lane-marker cp-lane-marker--sample" aria-hidden="true" />
                    <div>
                      <p className="cp-lane-head">
                        <span className="cp-quiet">{row.label}</span>
                      </p>
                      <p className="cp-quiet cp-mono">{row.detail}</p>
                    </div>
                  </li>
                );
              }
              if (row.kind === "watch") {
                return (
                  <li key={`w-${row.sequence}-${index}`} className="cp-lane-row">
                    <span className="cp-lane-marker cp-lane-marker--sample" aria-hidden="true" />
                    <div>
                      <p className="cp-lane-head">
                        <span className="cp-quiet">obs</span>{" "}
                        <code className="cp-mono">{row.eventKind}</code>
                        {row.scopedToPageTask ? null : (
                          <span className="cp-quiet">
                            {" "}
                            · process-local ring, not this task exclusively
                          </span>
                        )}
                      </p>
                      <p className="cp-quiet">
                        cursor <code className="cp-mono">{row.sequence}</code> · {row.detail}
                      </p>
                    </div>
                  </li>
                );
              }
              return (
                <li key={`o-${row.kind}-${index}`} className="cp-lane-row cp-lane-row--note">
                  <span className="cp-lane-marker cp-lane-marker--gap" aria-hidden="true" />
                  <p className="cp-reason" role="status">
                    {row.kind === "bounded" ? <strong>Bounded. </strong> : null}
                    {row.note}
                  </p>
                </li>
              );
            })}
          </ul>
        </div>
      </div>

      <HonestyNote>
        {watch.phase === "unattached" ? (
          <>
            Watch is <strong>{WATCH_NOT_ATTACHED.state}</strong>. {WATCH_NOT_ATTACHED.detail} Authority
            rows still come only from <code>/task/evidence</code>; a watch delta cannot move a task.
          </>
        ) : (
          <>
            Watch is <strong>{watch.label}</strong>. Deltas append to the observation lane only.
            Stream close is not Task completion, and detaching never cancelled a Task or stopped an
            Agent.
          </>
        )}
      </HonestyNote>
    </section>
  );
}
