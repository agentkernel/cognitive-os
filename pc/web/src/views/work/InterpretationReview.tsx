import { DigestChip } from "../../components/DigestChip";
import {
  CLARIFICATION_REQUIRED,
  canPreview,
  unresolvedMaterial,
  type AmbiguityDraft,
  type InterpretationView,
} from "../../data/projections/work";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";

/**
 * Step 2 — docs/design/14 §5. The daemon derived a status from the candidate's
 * own material-ambiguity facts; the model never picks it and neither does this
 * page. `clarification_required` is a normal branch of the chain, not an
 * error: it means the intent authority still has to decide something, and the
 * chain deliberately stops here until they do.
 */
export function InterpretationReview({
  interpretation,
  ambiguities,
  informationGaps,
  assumptions,
  onAnswer,
  onReinterpret,
  onPreview,
  busy,
}: {
  interpretation: InterpretationView;
  ambiguities: AmbiguityDraft[];
  informationGaps: string[];
  assumptions: string[];
  onAnswer: (id: string, answer: string) => void;
  onReinterpret: () => void;
  onPreview: () => void;
  busy: boolean;
}) {
  const blocked = interpretation.status === CLARIFICATION_REQUIRED;
  const open = unresolvedMaterial(ambiguities);
  const answered = ambiguities.filter((item) => item.answer.trim() !== "");

  return (
    <section className="cp-region" aria-labelledby="interpretation-title">
      <h3 className="cp-section-title" id="interpretation-title">
        Review the interpretation
      </h3>
      <p className="cp-region-line">
        <StateChip
          reading={{
            category: blocked ? "waiting" : "ready",
            label: interpretation.status,
            unmapped: false,
          }}
        />{" "}
        <span className="cp-quiet">
          {interpretation.materialAmbiguityCount} material ambiguit
          {interpretation.materialAmbiguityCount === 1 ? "y" : "ies"} · interpretation{" "}
        </span>
        <code className="cp-mono">{interpretation.interpretationId}</code>{" "}
        <DigestChip value={interpretation.interpretationDigest} label="interpretation digest" />
      </p>

      {blocked ? (
        <p className="cp-reason" role="status">
          The daemon recorded this candidate as <code className="cp-mono">clarification_required</code>{" "}
          because it declares a material ambiguity. This is the expected path, not a failure:
          admission would be refused with <code className="cp-mono">INTENT_CLARIFICATION_REQUIRED</code>{" "}
          rather than resolved by picking the most likely reading. Answer each material question,
          then re-interpret — the new candidate supersedes this one and nothing is rewritten.
        </p>
      ) : (
        <p className="cp-quiet">
          No material ambiguity stands. The candidate is admissible, and admission will be bound to
          exactly this interpretation digest.
        </p>
      )}

      <h4 className="cp-section-title">Ambiguities</h4>
      {ambiguities.length === 0 ? (
        <p className="cp-quiet">
          The candidate declared none. That is a statement about this candidate, not a guarantee
          that the objective is unambiguous.
        </p>
      ) : (
        <ul className="cp-plain-list">
          {ambiguities.map((item) => (
            <li key={item.id}>
              <label className="cp-field">
                <span>
                  <code className="cp-mono">{item.id}</code>{" "}
                  {item.material ? (
                    <strong>material</strong>
                  ) : (
                    <span className="cp-quiet">non-material</span>
                  )}{" "}
                  — {item.question}
                </span>
                <input
                  name={`answer_${item.id}`}
                  value={item.answer}
                  placeholder="Your decision"
                  onChange={(event) => onAnswer(item.id, event.target.value)}
                />
              </label>
            </li>
          ))}
        </ul>
      )}

      <h4 className="cp-section-title">Information gaps</h4>
      {informationGaps.length === 0 ? (
        <p className="cp-quiet">The candidate declared no information gap.</p>
      ) : (
        <ul className="cp-plain-list">
          {informationGaps.map((gap) => (
            <li key={gap}>
              <code className="cp-mono">{gap}</code>
            </li>
          ))}
        </ul>
      )}
      <p className="cp-quiet">
        An information gap is a reference to something the interpretation needed and did not have.
        The daemon validates each one as a URI and refuses the interpretation otherwise; a gap is
        recorded, never quietly filled in.
      </p>

      {assumptions.length > 0 ? (
        <>
          <h4 className="cp-section-title">Recorded decisions</h4>
          <ul className="cp-plain-list">
            {assumptions.map((assumption) => (
              <li key={assumption}>{assumption}</li>
            ))}
          </ul>
        </>
      ) : null}

      <p className="cp-next">
        <button type="button" className="cp-button" onClick={onReinterpret} disabled={busy}>
          {answered.length > 0 ? "Re-interpret with these decisions" : "Re-interpret"}
        </button>{" "}
        <button
          type="button"
          className="cp-button cp-button--primary"
          onClick={onPreview}
          disabled={busy || !canPreview(interpretation)}
        >
          Preview the contract
        </button>
      </p>
      {blocked ? (
        <HonestyNote>
          Preview is unavailable while {open.length || interpretation.materialAmbiguityCount}{" "}
          material ambiguit{open.length === 1 ? "y" : "ies"} stand
          {open.length === 1 ? "s" : ""} unresolved. Answering a material question moves it into the
          recorded decisions above, where it stays visible on the contract you accept.
        </HonestyNote>
      ) : null}
    </section>
  );
}
