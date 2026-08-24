import { useState, type ReactNode } from "react";

/**
 * ConfirmSurface — for consequential class-A actions. Exact targets and
 * consequences are named up front; confirmation is an explicit checkbox
 * naming the exact tuple (not a modal chain, not a destructive-default).
 * The action button stays disabled until the exact confirmation is given —
 * this is a confirmation gate, not a capability pretense.
 */
export function ConfirmSurface({
  title,
  consequences,
  targets,
  confirmLabel,
  actionLabel,
  danger,
  onConfirm,
}: {
  title: string;
  /** Plain-language consequences of confirming. */
  consequences: ReactNode;
  /** Exact ids/versions/digests the confirmation binds to. */
  targets: string[];
  /** The checkbox label — names the exact act. */
  confirmLabel: string;
  actionLabel: string;
  danger?: boolean;
  onConfirm: () => void;
}) {
  const [confirmed, setConfirmed] = useState(false);
  return (
    <section className="cp-confirm" aria-label={title}>
      <h3 className="cp-section-title">{title}</h3>
      <div className="cp-quiet">{consequences}</div>
      <ul>
        {targets.map((target) => (
          <li key={target}>{target}</li>
        ))}
      </ul>
      <label className="cp-field">
        <input
          type="checkbox"
          checked={confirmed}
          onChange={(event) => setConfirmed(event.target.checked)}
        />{" "}
        {confirmLabel}
      </label>
      <button
        type="button"
        className={danger ? "cp-button cp-button--danger" : "cp-button cp-button--primary"}
        disabled={!confirmed}
        onClick={() => {
          setConfirmed(false);
          onConfirm();
        }}
      >
        {actionLabel}
      </button>
    </section>
  );
}
