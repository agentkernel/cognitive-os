import { PageHeader } from "../components/PageHeader";
import { SessionForm } from "../shell/SessionGate";

/**
 * Session page — utility chrome, addressable but never a navigation peer
 * (docs/design/10 DD-05).
 */
export function SessionPage() {
  return (
    <>
      <PageHeader
        title="Session"
        lede="Paste this daemon's local-bootstrap.secret once. It is not a Provider LLM API key. It is never written to localStorage, sessionStorage, IndexedDB, the URL, or exported state."
      />
      <SessionForm />
    </>
  );
}
