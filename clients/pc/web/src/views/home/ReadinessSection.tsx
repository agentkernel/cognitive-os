import { useState } from "react";
import { Link } from "react-router-dom";
import {
  expandedReadinessComponents,
  formatAge,
  readinessComponentReading,
  worstReadinessComponent,
  type HomeReadinessView,
} from "../../data/projections/home";
import type { Projection } from "../../data/store";
import type { LastGood } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";
import { RegionStatus } from "./RegionStatus";

/**
 * R1 Readiness — docs/design/13 §3. One line by default: overall category,
 * the daemon's own word, the worst component, and how old the check is.
 * Expands to the six components, each a state chip linking to System.
 *
 * Never a gauge, a score, a percentage, or a green banner. A component the
 * daemon did not report renders as unreported (S7), never as ready.
 */
export function ReadinessSection({
  projection,
  lastGood,
  nowMs,
}: {
  projection: Projection<HomeReadinessView>;
  lastGood: LastGood<HomeReadinessView>;
  nowMs: number;
}) {
  const [expanded, setExpanded] = useState(false);
  const view = lastGood.data;
  const overall = view ? readDomainState("readiness", view.overall) : undefined;
  const worst = view ? worstReadinessComponent(view) : undefined;
  const checkedAge = view ? formatAge(view.evaluatedAtMs, nowMs) : undefined;
  // First run (nothing configured) opens the component row by default.
  const firstRun = view != null && view.overall !== "ready" && view.firstConversationReady === false;
  const showComponents = expanded || firstRun;

  return (
    <section className="cp-region" aria-labelledby="home-readiness-title">
      <h3 className="cp-section-title" id="home-readiness-title">
        Readiness
      </h3>
      <RegionStatus projection={projection} lastGood={lastGood} what="readiness" />
      {view && overall ? (
        <>
          <p className="cp-region-line">
            <StateChip reading={overall} />{" "}
            {worst ? (
              <span>
                worst component <code className="cp-mono">{worst.name}</code>{" "}
                {worst.reported ? worst.state : "not reported"}
                {worst.errorClass ? ` (${worst.errorClass})` : ""}
              </span>
            ) : (
              <span>all reported components ready</span>
            )}{" "}
            <span className="cp-quiet">
              last checked {checkedAge ?? "at an unknown time (the daemon reported no evaluation timestamp)"}
            </span>{" "}
            <button
              type="button"
              className="cp-button"
              aria-expanded={showComponents}
              aria-controls="home-readiness-components"
              onClick={() => setExpanded((value) => !value)}
            >
              {showComponents ? "Hide components" : "Show components"}
            </button>
          </p>
          {showComponents ? (
            <div id="home-readiness-components">
              <ul className="cp-region-chips">
                {expandedReadinessComponents(view).map((component) => (
                  <li key={component.name}>
                    <Link to="/system" aria-label={`${component.name} readiness; open System`}>
                      <StateChip reading={readinessComponentReading(component)} />{" "}
                      <span>{component.name}</span>
                    </Link>
                  </li>
                ))}
              </ul>
              <p className="cp-quiet">
                first conversation ready:{" "}
                <code className="cp-mono">
                  {view.firstConversationReady == null
                    ? "unknown"
                    : String(view.firstConversationReady)}
                </code>
              </p>
              <HonestyNote>
                Doctor sub-sections (six-resource, headless-vault, operability probes) are
                placeholder-backed and are not wired over HTTP (BD register). A component the
                daemon did not report is shown as <code>not reported</code> — unknown is never
                rendered as ready.
              </HonestyNote>
            </div>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
