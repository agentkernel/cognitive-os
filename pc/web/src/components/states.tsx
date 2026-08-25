import type { ReactNode } from "react";
import { StateDot } from "../state/StateDot";

/**
 * State views — docs/design/22/23. Loading, Empty, Error, Unavailable are
 * distinct products, not one spinner or one "no data".
 */

export function LoadingState({ label }: { label?: string }) {
  return (
    <section className="cp-stateview" role="status" aria-label={label ?? "loading"}>
      <h3>
        <StateDot category="unknown" /> Loading
      </h3>
      <p>{label ?? "Fetching the latest projection from the daemon."}</p>
    </section>
  );
}

export function EmptyState({
  title,
  children,
  action,
}: {
  title: string;
  children: ReactNode;
  action?: ReactNode;
}) {
  return (
    <section className="cp-stateview" aria-label={title}>
      <h3>
        <StateDot category="unknown" /> {title}
      </h3>
      <p>{children}</p>
      {action ? <p className="cp-next">{action}</p> : null}
    </section>
  );
}

export function ErrorState({
  what,
  why,
  next,
  retryable,
}: {
  /** What happened — plain language, cause-first. */
  what: string;
  /** Why it happened / the stable error class. */
  why?: ReactNode;
  /** What the operator can do next. */
  next?: ReactNode;
  /** Whether retry is safe (shown explicitly). */
  retryable?: boolean;
}) {
  return (
    <section className="cp-stateview" role="alert">
      <h3>
        <StateDot category="blocked" /> {what}
      </h3>
      {why ? <p>{why}</p> : null}
      <p>{retryable ? "Retry is safe." : "Retry may not change the result."}</p>
      {next ? <p className="cp-next">{next}</p> : null}
    </section>
  );
}

/**
 * UnavailableState — the product's signature honest state (S7). Never a
 * disabled button pretending capability: a fact line + the named dependency
 * or CLI path.
 */
export function UnavailableState({
  what,
  dependency,
  cliPath,
}: {
  /** What is unavailable, verbatim. */
  what: string;
  /** Named backend dependency (e.g. "BD-2") or reason. */
  dependency?: string;
  /** The CLI path that does work, when one exists. */
  cliPath?: string;
}) {
  return (
    <section className="cp-stateview" aria-label={`${what} — unavailable`}>
      <h3>
        <StateDot category="unknown" /> {what}
      </h3>
      <p>Not available over HTTP from this daemon.</p>
      {dependency ? <p>Dependency: {dependency}.</p> : null}
      {cliPath ? (
        <p>
          Available through the CLI: <code className="cp-mono">{cliPath}</code>
        </p>
      ) : null}
    </section>
  );
}
