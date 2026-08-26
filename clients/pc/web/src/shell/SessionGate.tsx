import { useContext, useState, type FormEvent, type ReactNode } from "react";
import { issueChannelSession } from "../api";
import { clearSession, rememberBearer, rememberPrincipal, sessionHasChannel } from "../session";
import { SessionTick } from "./SessionScope";

/**
 * SessionForm — bootstrap-secret → channel-scoped bearers. Memory-only,
 * non-echoing, cleared on submit. Moved verbatim from the pre-refactor
 * App.tsx (behavior and copy are load-bearing for security tests).
 */
export function SessionForm() {
  const { bump } = useContext(SessionTick);
  const [secret, setSecret] = useState("");
  const [principal, setPrincipal] = useState("principal://local/owner");
  const [message, setMessage] = useState("Session tokens stay in memory only.");

  async function issue(event: FormEvent) {
    event.preventDefault();
    const bootstrap = secret;
    setSecret("");
    rememberPrincipal(principal);
    const management = await issueChannelSession("management", principal, bootstrap);
    const task = await issueChannelSession("task", principal, bootstrap);
    if (management.ok && management.token) {
      rememberBearer("management", management.token);
    }
    if (task.ok && task.token) {
      rememberBearer("task", task.token);
    }
    setMessage(
      `management ${management.ok ? "ready" : `HTTP ${management.status}`}; task ${
        task.ok ? "ready" : `HTTP ${task.status}`
      }. Bootstrap discarded.`,
    );
    bump();
  }

  return (
    <form className="cp-session-form" onSubmit={(event) => void issue(event)}>
      <label className="cp-field">
        Principal
        <input value={principal} onChange={(event) => setPrincipal(event.target.value)} />
      </label>
      <label className="cp-field">
        Daemon bootstrap secret
        <input
          type="password"
          autoComplete="off"
          value={secret}
          onChange={(event) => setSecret(event.target.value)}
        />
      </label>
      <p className="cp-quiet">
        File <code>local-bootstrap.secret</code> on this daemon. Not a Provider LLM API key and
        not a SecretRef. The browser cannot read the file. Sessions stay in memory only.
      </p>
      <button type="submit" className="cp-button cp-button--primary">
        Issue management and Task sessions
      </button>
      <button
        type="button"
        className="cp-button"
        onClick={() => {
          clearSession();
          setMessage("Session cleared.");
          bump();
        }}
      >
        Clear memory session
      </button>
      <p role="status">{message}</p>
    </form>
  );
}

/**
 * SessionGate — inline gate over the intended destination (never a redirect).
 * The destination title stays visible behind the gate, preserving orientation.
 * Moved from RequireSession; behavior identical.
 */
export function SessionGate({
  channel,
  title,
  children,
}: {
  channel: "management" | "task";
  title: string;
  children: ReactNode;
}) {
  const { tick } = useContext(SessionTick);
  void tick;
  if (!sessionHasChannel(channel)) {
    return (
      <section data-page="session-gate" className="cp-gate">
        <header className="cp-page-head">
          <h2>{title}</h2>
          <p className="cp-lede">
            This page needs a {channel} session. Sidebar navigation still changes the view.
          </p>
        </header>
        <p className="cp-warn" role="status">
          Paste this daemon&apos;s bootstrap secret — not a Provider LLM API key.
        </p>
        <SessionForm />
      </section>
    );
  }
  return <>{children}</>;
}
