import { DigestChip } from "../../../components/DigestChip";
import { EmptyState } from "../../../components/states";
import {
  NO_EFFECTS_RECORDED,
  effectNeedsAttention,
  sortEffectsByAttention,
  type EffectHistoryView,
} from "../../../data/projections/workDetail";
import type { Projection } from "../../../data/store";
import { HonestyNote } from "../../../state/HonestyNote";
import { StateChip } from "../../../state/StateChip";
import { readDomainState } from "../../../state/stateMap";

/**
 * Effects — docs/design/15 §3.3. Every external mutation the daemon attempted
 * for this task, worst first. `OUTCOME_UNKNOWN` and `VERIFY_FAILED` come first
 * because they are the two states where the operator, not the daemon, has to
 * decide what the world looks like.
 */
export function EffectsSection({
  view,
  projection,
}: {
  view?: EffectHistoryView;
  projection?: Projection<unknown>;
}) {
  const unreadable = projection != null && projection.data === undefined;
  const entries = view ? sortEffectsByAttention(view.entries) : [];

  return (
    <section className="cp-region" id="section-effects" aria-labelledby="effects-title">
      <h3 className="cp-section-title" id="effects-title">
        Effects
      </h3>

      {unreadable ? (
        <p className="cp-reason" role="status">
          <StateChip reading={readDomainState("load", projection.status)} /> The effect history
          could not be read — <code className="cp-mono">{projection.error?.code ?? projection.status}</code>.
          No mutation is shown, and none is assumed either way.
        </p>
      ) : null}

      {view && entries.length === 0 ? (
        <EmptyState title="No external mutation recorded">{NO_EFFECTS_RECORDED}</EmptyState>
      ) : null}

      {entries.length > 0 ? (
        <table className="cp-table">
          <caption>Recorded effects, needing attention first</caption>
          <thead>
            <tr>
              <th scope="col">Stage</th>
              <th scope="col">Outcome</th>
              <th scope="col">Reconcile</th>
              <th scope="col">Mutations</th>
              <th scope="col">Fixed post-state / report</th>
              <th scope="col">Effect ref</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((entry) => (
              <tr key={entry.effectRef} data-row-key={entry.effectRef}>
                <td>
                  <StateChip reading={readDomainState("effect", entry.stage)} />
                  {effectNeedsAttention(entry) ? (
                    <>
                      {" "}
                      <span className="cp-quiet">needs a decision</span>
                    </>
                  ) : null}
                </td>
                <td className="cp-mono">{entry.outcomeClass}</td>
                <td className="cp-mono">{entry.reconcileClass}</td>
                <td className="cp-mono">
                  {entry.mutationCount == null
                    ? "unknown (the daemon reports no mutation count for this stage)"
                    : String(entry.mutationCount)}
                </td>
                <td>
                  {entry.fixedPostStateRef ? (
                    <DigestChip value={entry.fixedPostStateRef} label="fixed post-state ref" />
                  ) : (
                    <span className="cp-quiet">no fixed post-state recorded</span>
                  )}{" "}
                  {entry.reportRef ? (
                    <DigestChip value={entry.reportRef} label="reconciliation report ref" />
                  ) : (
                    <span className="cp-quiet">no report recorded</span>
                  )}
                </td>
                <td>
                  <DigestChip value={entry.effectRef} label="effect ref" />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}

      {view?.effectsTruncated ? (
        <p className="cp-reason" role="status">
          <strong>Bounded.</strong> The daemon reports this effect history as truncated: more
          effects exist than were returned, so this is not the whole set.
        </p>
      ) : null}

      <HonestyNote>
        <code>OUTCOME_UNKNOWN</code> means the daemon does not know whether the external world
        changed — not that nothing happened. <code>VERIFY_FAILED</code> means a mutation was
        attempted and its verification failed. An empty list is an absence of recorded mutation and
        never a successful outcome.
        {view ? (
          <>
            {" "}
            Reading this history performed no authority action (
            <code className="cp-mono">authority_side_effects={String(view.authoritySideEffects)}</code>
            ).
          </>
        ) : null}
      </HonestyNote>
    </section>
  );
}
