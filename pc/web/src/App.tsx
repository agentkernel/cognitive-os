import { useEffect, useMemo, useState } from "react";
import { HashRouter, NavLink, Navigate, Route, Routes, useParams } from "react-router-dom";
import { issueChannelSession, readJson, rejectCallerHeaderInjection } from "./api";
import { AGENT_IDENTITY_KEYS, mergeIdentities, type AgentIdentities } from "./identities";
import {
  acceptBindingMutation,
  displayCost,
  escapeUntrustedText,
  inferCompletionFromObservation,
  unavailableLabel,
} from "./policy";
import { clearSession, rememberBearer, sessionHasChannel } from "./session";
import { createWatchController } from "./watch";

type LoadState = {
  status: "loading" | "ready" | "empty" | "denied" | "disconnected" | "unknown" | "not-run";
  ms?: number;
  body?: unknown;
  message?: string;
};

const NAV = [
  ["/", "Home"],
  ["/agents", "Agents"],
  ["/providers", "Providers"],
  ["/bindings", "Bindings"],
  ["/tasks", "Tasks"],
  ["/activity", "Activity"],
  ["/resources", "Resources"],
] as const;

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function asList(value: unknown, keys: string[]): unknown[] {
  const record = asRecord(value);
  for (const key of keys) {
    if (Array.isArray(record[key])) {
      return record[key] as unknown[];
    }
  }
  return [];
}

async function load(path: string, channel: "management" | "task"): Promise<LoadState> {
  try {
    const result = await readJson(path, channel);
    if (result.status === 401 || result.status === 403) {
      return { status: "denied", ms: result.ms, body: result.body, message: `HTTP ${result.status}` };
    }
    if (!result.ok) {
      return { status: "unknown", ms: result.ms, body: result.body, message: `HTTP ${result.status}` };
    }
    const list = asList(result.body, ["items", "accounts", "bindings", "events", "alerts", "models"]);
    if (list.length === 0 && JSON.stringify(result.body).includes("[]")) {
      return { status: "empty", ms: result.ms, body: result.body };
    }
    return { status: "ready", ms: result.ms, body: result.body };
  } catch (error) {
    return {
      status: "disconnected",
      message: error instanceof Error ? error.message : "disconnected",
    };
  }
}

function StateNote({ state }: { state: LoadState }) {
  return (
    <p className="muted" role="status">
      {state.status}
      {state.ms != null ? ` · ${state.ms} ms` : ""}
      {state.message ? ` · ${state.message}` : ""}
    </p>
  );
}

function JsonPanel({ title, value }: { title: string; value: unknown }) {
  return (
    <section className="panel">
      <h3>{title}</h3>
      <pre>{JSON.stringify(value ?? {}, null, 2)}</pre>
    </section>
  );
}

function Shell({ children }: { children: React.ReactNode }) {
  return (
    <div className="shell">
      <a className="skip" href="#main">
        Skip to content
      </a>
      <nav className="side" aria-label="Primary">
        <h1>CognitiveOS Personal</h1>
        <p className="muted">Daemon client only. Not an authority writer.</p>
        <ul>
          {NAV.map(([to, label]) => (
            <li key={to}>
              <NavLink to={to} end={to === "/"}>
                {label}
              </NavLink>
            </li>
          ))}
          <li>
            <NavLink to="/session">Session</NavLink>
          </li>
        </ul>
      </nav>
      <main id="main">{children}</main>
    </div>
  );
}

function RequireSession({
  channel,
  children,
}: {
  channel: "management" | "task";
  children: React.ReactNode;
}) {
  if (!sessionHasChannel(channel)) {
    return <Navigate to="/session" replace />;
  }
  return <>{children}</>;
}

function SessionPage() {
  const [secret, setSecret] = useState("");
  const [principal, setPrincipal] = useState("owner-local");
  const [message, setMessage] = useState<string>("Session tokens stay in memory only.");

  async function issue(event: React.FormEvent) {
    event.preventDefault();
    const bootstrap = secret;
    setSecret("");
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
  }

  return (
    <>
      <h2>Session bootstrap</h2>
      <p className="muted">
        Paste the daemon bootstrap secret once. It is never written to localStorage,
        sessionStorage, IndexedDB, the URL, or exported state.
      </p>
      <form onSubmit={issue}>
        <label>
          Principal
          <input value={principal} onChange={(event) => setPrincipal(event.target.value)} />
        </label>
        <label>
          Bootstrap secret
          <input
            type="password"
            autoComplete="off"
            value={secret}
            onChange={(event) => setSecret(event.target.value)}
          />
        </label>
        <button type="submit">Issue management and Task sessions</button>
        <button type="button" onClick={() => { clearSession(); setMessage("Session cleared."); }}>
          Clear memory session
        </button>
      </form>
      <p role="status">{message}</p>
    </>
  );
}

function HomePage() {
  const [health, setHealth] = useState<LoadState>({ status: "loading" });
  const [status, setStatus] = useState<LoadState>({ status: "loading" });
  const [readiness, setReadiness] = useState<LoadState>({ status: "loading" });
  const [doctor, setDoctor] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    void (async () => {
      try {
        const started = performance.now();
        const response = await fetch("/personal/health", { credentials: "omit" });
        setHealth({
          status: response.ok ? "ready" : "unknown",
          ms: Math.round(performance.now() - started),
          body: await response.json().catch(() => ({})),
        });
      } catch {
        setHealth({ status: "disconnected", message: "daemon unreachable" });
      }
      setStatus(await load("/personal/status", "management"));
      setReadiness(await load("/personal/readiness", "management"));
      setDoctor(await load("/personal/doctor", "management"));
    })();
  }, []);

  return (
    <>
      <h2>Home</h2>
      <div className="status-grid">
        <section className="panel">
          <h3>Health</h3>
          <StateNote state={health} />
        </section>
        <section className="panel">
          <h3>Status</h3>
          <StateNote state={status} />
        </section>
        <section className="panel">
          <h3>Readiness</h3>
          <StateNote state={readiness} />
        </section>
        <section className="panel">
          <h3>Doctor</h3>
          <StateNote state={doctor} />
        </section>
      </div>
      <JsonPanel title="Readiness projection" value={readiness.body} />
      <JsonPanel title="Doctor projection" value={doctor.body} />
    </>
  );
}

function identitiesFromResource(item: Record<string, unknown>): AgentIdentities {
  return mergeIdentities({
    package: String(item.package_id ?? item.package ?? "unknown"),
    installation: String(item.installation_id ?? item.installation ?? "unknown"),
    registration: String(item.registration_id ?? item.registration ?? "unknown"),
    instance: String(item.id ?? item.instance_id ?? item.instance ?? "unknown"),
    sidecar: String(item.sidecar_id ?? item.sidecar ?? "unknown"),
    execution: String(item.execution_id ?? item.execution ?? "unknown"),
    process: String(item.process_id ?? item.process ?? "unknown"),
    task: String(item.task_id ?? item.current_task ?? "unknown"),
    shell_session: String(item.shell_session_id ?? item.shell_session ?? "unknown"),
  });
}

function AgentsPage() {
  const [runtime, setRuntime] = useState<LoadState>({ status: "loading" });
  const [bindings, setBindings] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    void (async () => {
      setRuntime(await load("/management/resource/v1/list?family=runtime", "management"));
      setBindings(await load("/management/agent-bindings", "management"));
    })();
  }, []);

  const items = asList(runtime.body, ["items", "resources"]).map(asRecord);

  return (
    <>
      <h2>Agents</h2>
      <StateNote state={runtime} />
      <p className="muted">
        Pause, resume, stop, restart, and quarantine are {unavailableLabel("agent-pause")},{" "}
        {unavailableLabel("agent-resume")}, {unavailableLabel("agent-stop")},{" "}
        {unavailableLabel("agent-restart")}, {unavailableLabel("agent-quarantine")}. No generic
        lifecycle route is offered.
      </p>
      {items.length === 0 ? (
        <p className="warn">No runtime family items. Binding identities still stay distinct.</p>
      ) : (
        <table>
          <caption>Runtime family inventory</caption>
          <thead>
            <tr>
              <th>Instance</th>
              <th>Package</th>
              <th>Status</th>
              <th>Detail</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item, index) => {
              const identities = identitiesFromResource(item);
              return (
                <tr key={String(item.id ?? index)}>
                  <td>{identities.instance}</td>
                  <td>{identities.package}</td>
                  <td>{String(item.status ?? item.lifecycle ?? "unknown")}</td>
                  <td>
                    <NavLink to={`/agents/${encodeURIComponent(String(item.id ?? index))}`}>
                      Inspect
                    </NavLink>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
      <JsonPanel title="Bindings projection" value={bindings.body} />
    </>
  );
}

function AgentDetailPage() {
  const { id } = useParams();
  const [inspect, setInspect] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    if (!id) {
      return;
    }
    void (async () => {
      setInspect(
        await load(
          `/management/resource/v1/inspect?family=runtime&id=${encodeURIComponent(id)}`,
          "management",
        ),
      );
    })();
  }, [id]);

  const item = asRecord(asRecord(inspect.body).item ?? inspect.body);
  const identities = identitiesFromResource(item);

  return (
    <>
      <h2>Agent detail</h2>
      <StateNote state={inspect} />
      <div className="identity-grid">
        {AGENT_IDENTITY_KEYS.map((key) => (
          <article className="identity-card" key={key}>
            <h3>{key}</h3>
            <p>{identities[key]}</p>
          </article>
        ))}
      </div>
      <section className="panel">
        <h3>Typed lifecycle</h3>
        <p>{unavailableLabel("agent-pause")}</p>
        <p>{unavailableLabel("agent-resume")}</p>
        <p>{unavailableLabel("agent-stop")}</p>
        <p>{unavailableLabel("agent-restart")}</p>
        <p>{unavailableLabel("agent-quarantine")}</p>
      </section>
      <JsonPanel title="Inspect projection" value={inspect.body} />
    </>
  );
}

function ProvidersPage() {
  const [accounts, setAccounts] = useState<LoadState>({ status: "loading" });
  const [message, setMessage] = useState("Keys travel only in the key POST body, then SecretStore.");

  async function refresh() {
    setAccounts(await load("/management/providers/accounts", "management"));
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function create(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const body = {
      display_name: String(form.get("display_name") ?? ""),
      provider_kind: String(form.get("provider_kind") ?? ""),
      endpoint: String(form.get("endpoint") ?? "") || undefined,
      allow_private_network: form.get("allow_private_network") === "on",
      allow_insecure_http: form.get("allow_insecure_http") === "on",
    };
    rejectCallerHeaderInjection(body);
    const result = await readJson("/management/providers/accounts", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    setMessage(result.ok ? "Account created without embedding a key in the create form." : `HTTP ${result.status}`);
    event.currentTarget.reset();
    await refresh();
  }

  const rows = asList(accounts.body, ["accounts", "items"]).map(asRecord);

  return (
    <>
      <h2>Providers</h2>
      <StateNote state={accounts} />
      <form onSubmit={create}>
        <h3>Create named account</h3>
        <p className="muted">
          Trust confirmation happens on the daemon. The browser does not write SecretStore.
          Create first; rotate the key on the account page.
        </p>
        <label>
          Display name
          <input name="display_name" required />
        </label>
        <label>
          Kind
          <select name="provider_kind" required defaultValue="openai">
            <option value="openai">openai</option>
            <option value="anthropic">anthropic</option>
            <option value="openai_compatible">openai_compatible</option>
          </select>
        </label>
        <label>
          Endpoint
          <input name="endpoint" placeholder="https://api.openai.com/v1" />
        </label>
        <label>
          <input type="checkbox" name="allow_private_network" /> Allow private network
        </label>
        <label>
          <input type="checkbox" name="allow_insecure_http" /> Allow insecure HTTP
        </label>
        <button type="submit">Create account</button>
      </form>
      <p role="status">{message}</p>
      <table>
        <caption>Provider accounts (SecretRef shown only as present/absent)</caption>
        <thead>
          <tr>
            <th>Id</th>
            <th>Name</th>
            <th>Kind</th>
            <th>Status</th>
            <th>Secret</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={String(row.id)}>
              <td>{String(row.id)}</td>
              <td>{String(row.display_name ?? "")}</td>
              <td>{String(row.provider_kind ?? "")}</td>
              <td>{String(row.status ?? "unknown")}</td>
              <td>{String(row.secret_ref ?? "absent")}</td>
              <td>
                <NavLink to={`/providers/${encodeURIComponent(String(row.id))}`}>Open</NavLink>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

function ProviderDetailPage() {
  const { id } = useParams();
  const [account, setAccount] = useState<LoadState>({ status: "loading" });
  const [models, setModels] = useState<LoadState>({ status: "loading" });
  const [key, setKey] = useState("");
  const [message, setMessage] = useState("Key field is memory-only and cleared after submit.");

  async function refresh() {
    if (!id) {
      return;
    }
    setAccount(
      await load(
        `/management/providers/accounts/inspect?id=${encodeURIComponent(id)}`,
        "management",
      ),
    );
    setModels(
      await load(`/management/providers/models?account_id=${encodeURIComponent(id)}`, "management"),
    );
  }

  useEffect(() => {
    void refresh();
  }, [id]);

  async function rotate(event: React.FormEvent) {
    event.preventDefault();
    const apiKey = key;
    setKey("");
    const body = { id, op: "rotate", api_key: apiKey };
    rejectCallerHeaderInjection(body);
    const result = await readJson("/management/providers/accounts/key", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    setMessage(
      result.ok
        ? "Key handed to daemon SecretStore path. Response redacted."
        : `HTTP ${result.status}`,
    );
    await refresh();
  }

  async function probe() {
    const result = await readJson("/management/providers/models/refresh", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ id }),
    });
    setMessage(result.ok ? `Probe completed in ${result.ms} ms.` : `Probe HTTP ${result.status}`);
    await refresh();
  }

  const record = asRecord(asRecord(account.body).account ?? account.body);
  const modelRows = asList(models.body, ["models", "items"]).map(asRecord);

  return (
    <>
      <h2>Provider account</h2>
      <StateNote state={account} />
      <section className="panel">
        <p>Status: {String(record.status ?? "unknown")}</p>
        <p>Catalog revision: {String(record.catalog_revision ?? "unknown")}</p>
        <p>Secret: {String(record.secret_ref ?? "absent")}</p>
        <p>Last discovery error: {String(record.last_discovery_error ?? "none")}</p>
      </section>
      <form onSubmit={rotate}>
        <h3>SecretStore handoff</h3>
        <label>
          API key
          <input
            type="password"
            autoComplete="off"
            value={key}
            onChange={(event) => setKey(event.target.value)}
          />
        </label>
        <button type="submit">Rotate key via daemon</button>
      </form>
      <button type="button" onClick={() => void probe()}>
        Bounded model/capability probe
      </button>
      <p role="status">{message}</p>
      <table>
        <caption>Catalog (failed refresh must keep the last catalog)</caption>
        <thead>
          <tr>
            <th>Model</th>
            <th>Source</th>
            <th>Input cost</th>
            <th>Output cost</th>
          </tr>
        </thead>
        <tbody>
          {modelRows.map((model) => (
            <tr key={String(model.model_id)}>
              <td>{String(model.model_id)}</td>
              <td>{String(model.source ?? "unknown")}</td>
              <td>{displayCost(model.price_input_per_million)}</td>
              <td>{displayCost(model.price_output_per_million)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

function BindingsPage() {
  const [bindings, setBindings] = useState<LoadState>({ status: "loading" });
  const [accounts, setAccounts] = useState<LoadState>({ status: "loading" });
  const [message, setMessage] = useState("At most one active fixed account+model per Agent.");

  async function refresh() {
    setBindings(await load("/management/agent-bindings", "management"));
    setAccounts(await load("/management/providers/accounts", "management"));
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const agent = String(form.get("agent") ?? "");
    const accountId = String(form.get("account_id") ?? "");
    const modelId = String(form.get("model_id") ?? "");
    const expectedRevision = Number(form.get("expected_revision"));
    const current = asList(bindings.body, ["bindings", "items"])
      .map(asRecord)
      .find((row) => row.agent === agent);
    const account = asList(accounts.body, ["accounts", "items"])
      .map(asRecord)
      .find((row) => row.id === accountId);
    const gate = acceptBindingMutation({
      expectedRevision: Number.isFinite(expectedRevision) ? expectedRevision : undefined,
      currentRevision: current ? Number(current.revision ?? 0) : expectedRevision,
      accountStatus: account ? String(account.status) : undefined,
      fallback: form.get("fallback") === "on",
      perRequestOverride: form.get("per_request") === "on",
    });
    if (!gate.ok) {
      setMessage(gate.reason);
      return;
    }
    const result = await readJson("/management/agent-bindings", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ agent, account_id: accountId, model_id: modelId }),
    });
    setMessage(result.ok ? "Binding stored." : `HTTP ${result.status}`);
    await refresh();
  }

  const rows = asList(bindings.body, ["bindings", "items"]).map(asRecord);

  return (
    <>
      <h2>Agent Provider bindings</h2>
      <StateNote state={bindings} />
      <form onSubmit={submit}>
        <label>
          Agent
          <select name="agent" required defaultValue="pi">
            <option value="pi">pi</option>
            <option value="dsh">dsh</option>
          </select>
        </label>
        <label>
          Account id
          <input name="account_id" required />
        </label>
        <label>
          Model id
          <input name="model_id" required />
        </label>
        <label>
          Expected revision
          <input name="expected_revision" type="number" required defaultValue={0} />
        </label>
        <label>
          <input type="checkbox" name="fallback" /> Request fallback (must be rejected)
        </label>
        <label>
          <input type="checkbox" name="per_request" /> Per-request override (must be rejected)
        </label>
        <button type="submit">Confirm fixed binding</button>
      </form>
      <p role="status">{message}</p>
      <table>
        <caption>Active fixed bindings</caption>
        <thead>
          <tr>
            <th>Agent</th>
            <th>Account</th>
            <th>Model</th>
            <th>Revision</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={String(row.agent)}>
              <td>{String(row.agent)}</td>
              <td>{String(row.account_id)}</td>
              <td>{String(row.model_id)}</td>
              <td>{String(row.revision)}</td>
              <td>{String(row.status)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

function TasksPage() {
  const [effects, setEffects] = useState<LoadState>({ status: "loading" });
  const [observation, setObservation] = useState<LoadState>({ status: "loading" });
  const watch = useMemo(() => createWatchController(), []);
  const [watchState, setWatchState] = useState(watch.state);
  const [taskRef, setTaskRef] = useState("task://personal/example");

  async function refresh(ref: string) {
    const encoded = encodeURIComponent(ref);
    setEffects(await load(`/task/effects?task_ref=${encoded}`, "task"));
    setObservation(await load(`/task/observation?task_ref=${encoded}`, "task"));
    const evidence = await load(`/task/evidence?task_ref=${encoded}`, "task");
    const inferred = inferCompletionFromObservation({
      processExit: 0,
      providerResponse: observation.body,
      httpReceipt: evidence.body,
      streamClosed: true,
    });
    if (inferred !== "unknown") {
      watch.noteGap();
    }
    setWatchState(watch.state);
  }

  useEffect(() => {
    void refresh(taskRef);
  }, [taskRef]);

  return (
    <>
      <h2>Tasks, Effects, Evidence</h2>
      <p className="muted">
        Cancel is {unavailableLabel("task-cancel")}. Detach does not cancel a Task or stop an Agent.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const next = String(new FormData(event.currentTarget).get("task_ref") ?? "");
          setTaskRef(next);
        }}
      >
        <label>
          Task ref
          <input name="task_ref" defaultValue={taskRef} />
        </label>
        <button type="submit">Load projections</button>
        <button
          type="button"
          onClick={() => {
            watch.noteGap();
            setWatchState(watch.state);
          }}
        >
          Simulate cursor gap
        </button>
        <button
          type="button"
          onClick={() => {
            watch.detach();
            setWatchState(watch.state);
          }}
        >
          Detach observation
        </button>
      </form>
      <p className="live" role="status" aria-live="polite">
        Watch {watchState}. Completion from observation remains unknown.
      </p>
      <StateNote state={effects} />
      <JsonPanel title="Effects" value={effects.body} />
      <JsonPanel
        title="Observation (escaped)"
        value={escapeUntrustedText(JSON.stringify(observation.body ?? {}, null, 2))}
      />
    </>
  );
}

function ActivityPage() {
  const [usage, setUsage] = useState<LoadState>({ status: "loading" });
  const [budgets, setBudgets] = useState<LoadState>({ status: "loading" });
  const [alerts, setAlerts] = useState<LoadState>({ status: "loading" });
  const [audit, setAudit] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    void (async () => {
      setUsage(await load("/management/usage", "management"));
      setBudgets(await load("/management/budgets", "management"));
      setAlerts(await load("/management/alerts", "management"));
      setAudit(await load("/management/audit", "management"));
    })();
  }, []);

  return (
    <>
      <h2>Activity</h2>
      <StateNote state={usage} />
      <JsonPanel title="Usage" value={usage.body} />
      <JsonPanel title="Budgets" value={budgets.body} />
      <JsonPanel title="Alerts" value={alerts.body} />
      <JsonPanel title="Audit" value={audit.body} />
    </>
  );
}

function ResourcesPage() {
  const families = ["tool", "memory", "skill", "task", "context", "runtime"] as const;
  const [family, setFamily] = useState<(typeof families)[number]>("tool");
  const [list, setList] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    void (async () => {
      setList(await load(`/management/resource/v1/list?family=${family}`, "management"));
    })();
  }, [family]);

  return (
    <>
      <h2>Six-family resources</h2>
      <label>
        Family
        <select value={family} onChange={(event) => setFamily(event.target.value as typeof family)}>
          {families.map((item) => (
            <option key={item} value={item}>
              {item}
            </option>
          ))}
        </select>
      </label>
      <StateNote state={list} />
      <JsonPanel title={`${family} list`} value={list.body} />
    </>
  );
}

export function App() {
  return (
    <HashRouter>
      <Shell>
        <Routes>
          <Route path="/session" element={<SessionPage />} />
          <Route
            path="/"
            element={
              <RequireSession channel="management">
                <HomePage />
              </RequireSession>
            }
          />
          <Route
            path="/agents"
            element={
              <RequireSession channel="management">
                <AgentsPage />
              </RequireSession>
            }
          />
          <Route
            path="/agents/:id"
            element={
              <RequireSession channel="management">
                <AgentDetailPage />
              </RequireSession>
            }
          />
          <Route
            path="/providers"
            element={
              <RequireSession channel="management">
                <ProvidersPage />
              </RequireSession>
            }
          />
          <Route
            path="/providers/:id"
            element={
              <RequireSession channel="management">
                <ProviderDetailPage />
              </RequireSession>
            }
          />
          <Route
            path="/bindings"
            element={
              <RequireSession channel="management">
                <BindingsPage />
              </RequireSession>
            }
          />
          <Route
            path="/tasks"
            element={
              <RequireSession channel="task">
                <TasksPage />
              </RequireSession>
            }
          />
          <Route
            path="/activity"
            element={
              <RequireSession channel="management">
                <ActivityPage />
              </RequireSession>
            }
          />
          <Route
            path="/resources"
            element={
              <RequireSession channel="management">
                <ResourcesPage />
              </RequireSession>
            }
          />
        </Routes>
      </Shell>
    </HashRouter>
  );
}
