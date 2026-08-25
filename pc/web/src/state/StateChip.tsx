import { StateDot } from "./StateDot";
import { CATEGORY_META, type StateReading } from "./stateMap";

/**
 * StateChip = category dot + verbatim domain label (mono). The label is the
 * daemon's own state word; the category word is exposed via title for
 * clarification, and unmapped words carry an explicit marker.
 */
export function StateChip({ reading, reason }: { reading: StateReading; reason?: string }) {
  const meta = CATEGORY_META[reading.category];
  return (
    <span className="cp-chip" data-category={reading.category} title={meta.note}>
      <StateDot category={reading.category} />
      <span className="cp-mono">{reading.label}</span>
      {reading.unmapped ? <span className="cp-quiet">(unmapped state)</span> : null}
      {reason ? <span className="cp-reason">{reason}</span> : null}
    </span>
  );
}
