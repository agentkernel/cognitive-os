import { Link } from "react-router-dom";
import { DigestChip } from "../../components/DigestChip";
import { FactGrid } from "../../components/FactGrid";
import { ReceiptLine } from "../../components/ReceiptLine";
import { factOrUnknown, type AdmissionView } from "../../data/projections/work";
import { HonestyNote } from "../../state/HonestyNote";

/**
 * Step 4 — docs/design/14 §5. The receipt of an authority act: what was
 * admitted, under which epoch, bound to which digest. It is deliberately not a
 * success banner, because admission is not execution.
 */
export function AdmissionReceipt({ admission }: { admission: AdmissionView }) {
  return (
    <section className="cp-region" aria-labelledby="admitted-title">
      <h3 className="cp-section-title" id="admitted-title">
        Admitted
      </h3>
      <ReceiptLine>
        The daemon admitted <code className="cp-mono">{admission.taskRef}</code> at contract epoch{" "}
        {factOrUnknown(admission.contractEpoch)}.
      </ReceiptLine>
      <FactGrid
        facts={[
          { label: "Task ref", value: <span className="cp-mono">{admission.taskRef}</span> },
          {
            label: "Contract epoch",
            value: <span className="cp-mono">{factOrUnknown(admission.contractEpoch)}</span>,
          },
          {
            label: "Contract digest",
            value: admission.contractDigest ? (
              <DigestChip value={admission.contractDigest} label="contract digest" />
            ) : (
              "unknown"
            ),
          },
          {
            label: "Task contract ref",
            value: (
              <span className="cp-mono">{factOrUnknown(admission.taskContractRef)}</span>
            ),
          },
        ]}
      />
      <p className="cp-next">
        <Link
          className="cp-button cp-button--primary"
          to={`/work?task=${encodeURIComponent(admission.taskRef)}`}
        >
          Open in Work
        </Link>
      </p>
      <HonestyNote>
        Admitted means the contract is durable and the task is schedulable. It is not a claim that
        the task has started, progressed or completed, and this page will not say otherwise: in
        Work the new ref reads <code>state not exposed</code> until a real{" "}
        <code>/task/evidence</code> read returns a lifecycle state. This ref is now remembered for
        this browser session only.
      </HonestyNote>
    </section>
  );
}
