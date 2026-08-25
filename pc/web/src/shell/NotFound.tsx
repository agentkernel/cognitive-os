import { Link } from "react-router-dom";

/** Designed 404 — the previous app rendered an empty main area here. */
export function NotFound() {
  return (
    <section className="cp-stateview" aria-labelledby="cp-404">
      <h3 id="cp-404">
        <span className="cp-dot cp-dot--unknown" aria-hidden="true" /> No such route
      </h3>
      <p>This address does not exist in the Control Plane. Nothing was changed.</p>
      <p className="cp-next">
        <Link to="/">Back to Home</Link>
      </p>
    </section>
  );
}
