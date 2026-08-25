import { DigestChip } from "../../../components/DigestChip";
import { FactGrid } from "../../../components/FactGrid";
import { EmptyState } from "../../../components/states";
import {
  consumptionRefusal,
  type ConsumptionView,
} from "../../../data/projections/workDetail";
import type { Projection } from "../../../data/store";
import { HonestyNote } from "../../../state/HonestyNote";
import { StateChip } from "../../../state/StateChip";
import { readDomainState } from "../../../state/stateMap";

/**
 * Context — docs/design/15 §3.6. The durable Memory/Skill pins the daemon
 * actually consumed for this task, read back from
 * `GET /task/resource/v1/consumption`.
 *
 * The route has several distinct refusals and they mean different things — a
 * missing record, a missing ContextRequest, a mismatched ContextRequest and a
 * stale request digest are not the same fact. Each is named rather than
 * collapsed into "unavailable", because a mismatch is a real conflict.
 */
export function ContextSection({
  view,
  projection,
}: {
  view?: ConsumptionView;
  projection?: Projection<unknown>;
}) {
  const unreadable = projection != null && projection.data === undefined;

  return (
    <section className="cp-region" id="section-context" aria-labelledby="context-title">
      <h3 className="cp-section-title" id="context-title">
        Context
      </h3>

      {unreadable ? (
        <p className="cp-reason" role="status">
          <StateChip reading={readDomainState("load", projection.status)} />{" "}
          {consumptionRefusal(projection.error?.code)}
        </p>
      ) : null}

      {view ? (
        <>
          <FactGrid
            facts={[
              {
                label: "Decision class",
                value: <span className="cp-mono">{view.decisionClass ?? "unknown"}</span>,
              },
              {
                label: "Context request",
                value: (
                  <span className="cp-mono">{view.contextRequestId ?? "unknown"}</span>
                ),
              },
              {
                label: "Context request digest",
                value: view.contextRequestDigest ? (
                  <DigestChip
                    value={view.contextRequestDigest}
                    label="context request digest"
                  />
                ) : (
                  "unknown"
                ),
              },
              {
                label: "Session",
                value: <span className="cp-mono">{view.sessionRef ?? "unknown"}</span>,
              },
              {
                label: "Reuse of",
                value: (
                  <span className="cp-mono">
                    {view.reuseOf ?? "none — this is a first resolution, not a session resume"}
                  </span>
                ),
              },
            ]}
          />

          <h4 className="cp-section-title">Memory pins</h4>
          {view.memoryPins.length === 0 ? (
            <p className="cp-quiet">
              No Memory pin was consumed. The record exists and pins nothing — that is an
              authoritative empty, not a missing read.
            </p>
          ) : (
            <ul className="cp-plain-list">
              {view.memoryPins.map((pin) => (
                <li key={pin.memoryId}>
                  <code className="cp-mono">{pin.memoryId}</code>{" "}
                  {pin.sourceDigest ? (
                    <DigestChip value={pin.sourceDigest} label="source digest" />
                  ) : (
                    <span className="cp-quiet">no source digest recorded</span>
                  )}
                </li>
              ))}
            </ul>
          )}

          <h4 className="cp-section-title">Skill pins</h4>
          {view.skillPins.length === 0 ? (
            <p className="cp-quiet">No Skill binding was consumed for this task.</p>
          ) : (
            <ul className="cp-plain-list">
              {view.skillPins.map((pin) => (
                <li key={pin.bindingId}>
                  <code className="cp-mono">{pin.bindingId}</code> · revision{" "}
                  <code className="cp-mono">{pin.revisionId ?? "unknown"}</code>{" "}
                  {pin.contentDigest ? (
                    <DigestChip value={pin.contentDigest} label="content digest" />
                  ) : (
                    <span className="cp-quiet">no content digest recorded</span>
                  )}
                </li>
              ))}
            </ul>
          )}
        </>
      ) : null}

      {!view && !unreadable ? (
        <EmptyState title="Context has not been read">
          The consumption record for this task has not been read yet. This is a pending read, not a
          statement that the task consumed nothing.
        </EmptyState>
      ) : null}

      <HonestyNote>
        These are the daemon&apos;s durable, redacted consumption pins — identifiers and digests
        only, never memory content or skill source. How the context was assembled, and what the
        agent was attending to, are not exposed over HTTP and are named as unavailable in Overview
        rather than guessed at here.
        {view ? (
          <>
            {" "}
            Reading this record performed no authority action (
            <code className="cp-mono">authority_side_effects={String(view.authoritySideEffects)}</code>
            ).
          </>
        ) : null}
      </HonestyNote>
    </section>
  );
}
