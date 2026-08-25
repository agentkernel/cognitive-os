import { DigestChip } from "../../../components/DigestChip";
import { FactGrid } from "../../../components/FactGrid";
import type { SessionTaskChain } from "../../../data/projections/work";
import {
  PREVIEW_EPHEMERAL_STATEMENT,
  type TaskEvidenceDetail,
} from "../../../data/projections/workDetail";
import { HonestyNote } from "../../../state/HonestyNote";

/**
 * Intent & Contract — docs/design/15 §3.5. The governed chain that produced
 * this task: intent, interpretation, the preview that was reviewed, and the
 * admission.
 *
 * Only this browser session can supply the first three. The daemon exposes no
 * route that returns an intent record, an interpretation or a preview after
 * admission, so for a ref this session did not admit the chain is stated as
 * absent — never reconstructed from the contract, which would invent the
 * reasoning that produced it.
 */
export function IntentContractSection({
  chain,
  evidence,
}: {
  chain?: SessionTaskChain;
  evidence?: TaskEvidenceDetail;
}) {
  return (
    <section className="cp-region" id="section-intent" aria-labelledby="intent-title">
      <h3 className="cp-section-title" id="intent-title">
        Intent &amp; Contract
      </h3>

      {chain == null ? (
        <>
          <p className="cp-reason" role="status">
            <strong>No chain recorded in this session.</strong> The intent record, the
            interpretation and the preview are only available to the session that ran them: this
            tab did not admit this task, so it holds none of them. {PREVIEW_EPHEMERAL_STATEMENT}
          </p>
          <FactGrid
            facts={[
              {
                label: "Intent record",
                value: "unavailable — no daemon route returns a UserIntentRecord by task ref",
              },
              {
                label: "Interpretation",
                value: "unavailable — no daemon route returns an interpretation by task ref",
              },
              { label: "Preview digest", value: "unavailable — previews are never persisted" },
              {
                label: "Admitted contract",
                value: evidence?.contractEpoch == null ? (
                  "unknown"
                ) : (
                  <span className="cp-mono">epoch {evidence.contractEpoch}</span>
                ),
              },
              {
                label: "Recorded intents",
                value: (
                  <span className="cp-mono">
                    {evidence == null ? "unknown" : String(evidence.intentRefs.length)}
                  </span>
                ),
              },
            ]}
          />
        </>
      ) : (
        <>
          <h4 className="cp-section-title">1 · Intent</h4>
          <FactGrid
            facts={[
              {
                label: "Raw expression",
                value: chain.intent.rawExpression,
              },
              {
                label: "UserIntentRecord",
                value: <span className="cp-mono">{chain.intent.userIntentRecordId}</span>,
              },
              {
                label: "Recorded at",
                value: (
                  <span className="cp-mono">
                    {chain.intent.recordedAt ?? "no timestamp returned by the daemon"}
                  </span>
                ),
              },
            ]}
          />

          <h4 className="cp-section-title">2 · Interpretation</h4>
          <FactGrid
            facts={[
              {
                label: "Status (daemon's own word)",
                value: <span className="cp-mono">{chain.interpretation.status}</span>,
              },
              {
                label: "Interpretation id",
                value: <span className="cp-mono">{chain.interpretation.interpretationId}</span>,
              },
              {
                label: "Interpretation digest",
                value: (
                  <DigestChip
                    value={chain.interpretation.interpretationDigest}
                    label="interpretation digest"
                  />
                ),
              },
              {
                label: "Material ambiguities at admission",
                value: (
                  <span className="cp-mono">{chain.interpretation.materialAmbiguityCount}</span>
                ),
              },
              {
                label: "Superseded interpretations",
                value:
                  chain.interpretation.supersededInterpretationIds.length === 0 ? (
                    "none — the first interpretation was admitted"
                  ) : (
                    <span className="cp-mono">
                      {chain.interpretation.supersededInterpretationIds.join(" · ")}
                    </span>
                  ),
              },
            ]}
          />
          <h5 className="cp-section-title">Ambiguities still open at admission</h5>
          {chain.interpretation.openAmbiguities.length === 0 ? (
            <p className="cp-quiet">
              None stood open. That is a statement about the declared candidate, not a guarantee
              that the objective was unambiguous.
            </p>
          ) : (
            <ul className="cp-plain-list">
              {chain.interpretation.openAmbiguities.map((item) => (
                <li key={item.id}>
                  <code className="cp-mono">{item.id}</code>{" "}
                  {item.material ? <strong>material</strong> : <span className="cp-quiet">non-material</span>} —{" "}
                  {item.question}
                </li>
              ))}
            </ul>
          )}
          <h5 className="cp-section-title">Information gaps declared</h5>
          {chain.interpretation.informationGaps.length === 0 ? (
            <p className="cp-quiet">The candidate declared no information gap.</p>
          ) : (
            <ul className="cp-plain-list">
              {chain.interpretation.informationGaps.map((gap) => (
                <li key={gap}>
                  <code className="cp-mono">{gap}</code>
                </li>
              ))}
            </ul>
          )}
          <h5 className="cp-section-title">Recorded decisions</h5>
          {chain.interpretation.recordedDecisions.length === 0 ? (
            <p className="cp-quiet">No ambiguity was answered before admission.</p>
          ) : (
            <ul className="cp-plain-list">
              {chain.interpretation.recordedDecisions.map((decision) => (
                <li key={decision}>{decision}</li>
              ))}
            </ul>
          )}

          <h4 className="cp-section-title">3 · Preview (ephemeral)</h4>
          <FactGrid
            facts={[
              {
                label: "Preview digest reviewed",
                value: (
                  <DigestChip value={chain.preview.previewDigest} label="preview digest" />
                ),
              },
              { label: "Objective previewed", value: chain.preview.objective },
              {
                label: "Acceptance conditions (daemon count)",
                value: (
                  <span className="cp-mono">
                    {chain.preview.conditionCount == null
                      ? "unknown"
                      : String(chain.preview.conditionCount)}
                  </span>
                ),
              },
              { label: "Persistence", value: PREVIEW_EPHEMERAL_STATEMENT },
            ]}
          />

          <h4 className="cp-section-title">4 · Admission</h4>
          <FactGrid
            facts={[
              {
                label: "Accepted by",
                value: <span className="cp-mono">{chain.admission.acceptedBy}</span>,
              },
              {
                label: "Contract epoch",
                value: (
                  <span className="cp-mono">
                    {chain.admission.contractEpoch == null
                      ? "unknown"
                      : String(chain.admission.contractEpoch)}
                  </span>
                ),
              },
              {
                label: "Contract digest",
                value: chain.admission.contractDigest ? (
                  <DigestChip value={chain.admission.contractDigest} label="contract digest" />
                ) : (
                  "unknown"
                ),
              },
              {
                label: "Task contract ref",
                value: (
                  <span className="cp-mono">
                    {chain.admission.taskContractRef ?? "unknown"}
                  </span>
                ),
              },
            ]}
          />
          <p className="cp-quiet">
            The digest chain is checkable end to end: the interpretation digest above is the one
            admission accepted, and the preview digest is the one the contract was previewed under.
          </p>
        </>
      )}

      <HonestyNote>
        {PREVIEW_EPHEMERAL_STATEMENT} This section reads the daemon only for the admitted contract;
        the intent, interpretation and preview come from this tab&apos;s own memory of the chain it
        ran, and they disappear when the tab closes. Nothing here is reconstructed from the
        contract.
      </HonestyNote>
    </section>
  );
}
