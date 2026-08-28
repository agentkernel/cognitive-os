/**
 * PERSONAL 2.0 INTERACTION PROTOTYPE — V2
 *
 * Built-in mock data and local React state only. This Canvas does not connect
 * to a daemon, network, storage, filesystem, Provider, model, Skill, MCP
 * server, connector, or SecretStore. It cannot create Projects, send messages,
 * install capabilities, grant permissions, publish, reconcile Effects, admit
 * Memory, or issue receipts. Target-state samples are labelled explicitly.
 *
 * Product thesis: an OPC Owner should understand the outcome due today and the
 * one consequential decision within five seconds, then move from a business
 * description to a structured Project launch preview within five minutes.
 *
 * Cursor-openable copy (IDE detection path, not the design authority):
 * C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx
 *
 * This overwrite (same V2 files, not v3) applies the 2026-08-28 design-tooling
 * pass on the owner-approved baseline: product IA is Today / Projects /
 * Knowledge with Settings at the bottom; conversation stays the third column;
 * first-run no longer leaks into the populated X sample; QA state coverage
 * lives in State Lab as rendered panels, not a “Designed” matrix; the CEO
 * cycle is a status on work, not a second primary nav; Settings is a hub
 * without fake Connect / Install / Confirm controls.
 * Hosted and repository copies must stay byte-aligned after each overwrite.
 */

import {
  Callout,
  Select,
  TextArea,
  TextInput,
  UsageBar,
  useHostTheme,
  useState,
  type CSSProperties,
} from "cursor/canvas";

type Scene =
  | "today"
  | "projects"
  | "setup"
  | "project"
  | "temporary"
  | "people"
  | "operations"
  | "knowledge"
  | "settings"
  | "connections"
  | "capabilities"
  | "state-lab";

type TodayMode = "returning" | "first-run";
type SetupStage = "describe" | "research" | "design" | "simulate" | "preview";
type XStage = "package" | "preview" | "receipt" | "readback" | "reflection";
type PeopleView = "members" | "role" | "version";
type OperationsView = "working" | "missed" | "unknown" | "blocked";
type KnowledgeView = "vault" | "memory" | "context";
type ConnectionView = "quick" | "custom";
type CapabilityView = "skill" | "mcp";
type Channel = "assistant" | "project";
type LoopStep = "ingest" | "decide" | "authorize" | "execute" | "verify" | "report";
type ProvenanceKind = "observed" | "proposed" | "governed" | "verified";
type StateKey =
  | "loading"
  | "empty"
  | "working"
  | "error"
  | "success"
  | "partial"
  | "blocked"
  | "unknown"
  | "offline";
type SurfaceKey =
  | "today"
  | "projects"
  | "setup"
  | "project"
  | "temporary"
  | "people"
  | "operations"
  | "knowledge"
  | "settings"
  | "connections"
  | "capabilities";
type Tone = "neutral" | "good" | "warn" | "bad" | "info";
type MemberId = "lin" | "mei" | "rui";
type MemoryAction = "inspect" | "correct" | "forget";
type CapabilityDecision = "inspect" | "narrow" | "reject";

type Member = {
  id: MemberId;
  name: string;
  role: string;
  template: string;
  responsibility: string;
  current: string;
  next: string;
  accepted: string;
  ownerNeed: string;
  activity: "working" | "queued" | "waiting";
  activityLabel: string;
  model: string;
  cost: string;
  version: string;
};

type Outcome = {
  id: string;
  title: string;
  format: string;
  state: string;
  tone: Tone;
  acceptance: string;
  source: string;
  freshness: string;
  risk: string;
};

const SCENES: ReadonlyArray<{ id: Scene; label: string }> = [
  { id: "today", label: "Today · returning or first-run" },
  { id: "projects", label: "Projects · list or empty" },
  { id: "setup", label: "Project setup · description to preview" },
  { id: "project", label: "X Project · operating report and package inspect" },
  { id: "temporary", label: "Temporary typed canvas · pin preview" },
  { id: "people", label: "Role Template and Member Runtime" },
  { id: "operations", label: "Working, missed, unknown, blocked" },
  { id: "knowledge", label: "Knowledge, Vault companion, Memory" },
  { id: "settings", label: "Settings hub" },
  { id: "connections", label: "Settings · Model Connections" },
  { id: "capabilities", label: "Skill and MCP safety review" },
  { id: "state-lab", label: "State Lab · rendered coverage" },
];

const SCENE_TITLES: Record<Scene, string> = {
  today: "Today",
  projects: "Projects",
  setup: "Create a Project",
  project: "X content operation",
  temporary: "Outcome comparison",
  people: "Roles and Project Members",
  operations: "Work continuity",
  knowledge: "Knowledge",
  settings: "Settings",
  connections: "Model Connections",
  capabilities: "Capability review",
  "state-lab": "State Lab",
};

const PROJECT_SCENES: readonly Scene[] = [
  "project",
  "temporary",
  "people",
  "operations",
  "capabilities",
];

const SETUP_STAGES: ReadonlyArray<{
  id: SetupStage;
  label: string;
  time: string;
}> = [
  { id: "describe", label: "Describe", time: "about 1 min" },
  { id: "research", label: "Research", time: "about 2 min" },
  { id: "design", label: "Design", time: "about 1 min" },
  { id: "simulate", label: "Simulate", time: "about 30 sec" },
  { id: "preview", label: "Preview", time: "before activation" },
];

const X_STAGES: ReadonlyArray<{ id: XStage; label: string }> = [
  { id: "package", label: "Package" },
  { id: "preview", label: "Publish preview" },
  { id: "receipt", label: "Receipt" },
  { id: "readback", label: "Readback" },
  { id: "reflection", label: "Reflection" },
];

const LOOP_STEPS: ReadonlyArray<{
  id: LoopStep;
  n: string;
  label: string;
  job: string;
}> = [
  { id: "ingest", n: "1", label: "Ingest", job: "Research and Vault. External text stays untrusted." },
  { id: "decide", n: "2", label: "Decide", job: "The one Owner decision. Not a KPI wall." },
  { id: "authorize", n: "3", label: "Authorize", job: "Canvas preview. Intent persists before any Effect." },
  { id: "execute", n: "4", label: "Execute", job: "Fenced Member work. Queued is not running." },
  { id: "verify", n: "5", label: "Verify", job: "Independent check. Self-report cannot close work." },
  { id: "report", n: "6", label: "Report", job: "Receipt, readback, and the next cycle." },
];

const THREAD_POSTS: ReadonlyArray<{ n: string; copy: string }> = [
  { n: "1 / 7", copy: "Local-first control is an Owner decision surface, not a cloud autopilot." },
  { n: "2 / 7", copy: "Staff persist as Member Runtimes. A finished process is not a deleted employee." },
  { n: "3 / 7", copy: "Unknown cost and unknown Effects stay unknown. They are never drawn as zero." },
  { n: "4 / 7", copy: "The manager plans, delegates, and reflects inside an Owner-approved envelope." },
  { n: "5 / 7", copy: "Public actions wait on a canvas preview. Chat announces; it cannot approve." },
  { n: "6 / 7", copy: "Independent verification closes work. Member self-report cannot." },
  { n: "7 / 7", copy: "The next cycle starts from evidence, not from a guaranteed business outcome." },
];

const MEMBERS: readonly Member[] = [
  {
    id: "lin",
    name: "Lin",
    role: "Project Manager",
    template: "Project Manager Role · built-in base · v2",
    responsibility: "Deliver three reviewable X content packages each week and keep the operating cycle governed.",
    current: "Preparing Package A for the Owner decision.",
    next: "Reflect after receipt and readback evidence exist.",
    accepted: "Weekly topic brief · mock independent check passed",
    ownerNeed: "Review Package A; publishing is unavailable.",
    activity: "working",
    activityLabel: "Working now · preparing Package A",
    model: "Anthropic / balanced model · prototype binding",
    cost: "Estimated ¥18.40 · mock Provider estimate",
    version: "Member Runtime v3",
  },
  {
    id: "mei",
    name: "Mei",
    role: "Audience Researcher",
    template: "Audience Researcher Role · researched candidate v1",
    responsibility: "Produce source-backed audience tensions and topic candidates.",
    current: "Reconciling two conflicting adoption claims.",
    next: "Monday 09:00 if the Windows host is online.",
    accepted: "Audience tension memo · 6 sources · mock",
    ownerNeed: "None now; one stale source remains visible.",
    activity: "queued",
    activityLabel: "Queued · Monday 09:00 if the host is online",
    model: "Google / research-capable model · prototype binding",
    cost: "Actual unknown · usage source unavailable",
    version: "Member Runtime v2",
  },
  {
    id: "rui",
    name: "Rui",
    role: "Content Editor",
    template: "Content Editor Role · researched candidate v2",
    responsibility: "Turn accepted briefs into openable drafts, media briefs, and publication packages.",
    current: "Revising Outcome B after accessibility copy review.",
    next: "After Mei’s evidence handoff.",
    accepted: "Draft thread and visual brief · mock openability passed",
    ownerNeed: "Exclude or license the disputed reference.",
    activity: "waiting",
    activityLabel: "Waiting · evidence handoff, not a running process",
    model: "OpenAI / reasoning model · prototype binding",
    cost: "Estimated ¥11.20 · mock Provider estimate",
    version: "Member Runtime v3",
  },
];

const OUTCOMES: readonly Outcome[] = [
  {
    id: "a",
    title: "A · Local-first control",
    format: "7-post thread · visual brief · citations · alt text",
    state: "Needs Owner",
    tone: "warn",
    acceptance: "6/6 mock editorial checks",
    source: "Audience memo excerpts 2, 4, and 6",
    freshness: "18 minutes ago",
    risk: "No qualified X connector; no dispatch or receipt.",
  },
  {
    id: "b",
    title: "B · Digital staff, not Agent plumbing",
    format: "Single post · annotated image brief",
    state: "Partial",
    tone: "info",
    acceptance: "4/6 checks · alt text revision pending",
    source: "Owner-approved positioning and product design",
    freshness: "31 minutes ago",
    risk: "Image copy still needs accessibility review.",
  },
  {
    id: "c",
    title: "C · Unknown is not zero",
    format: "5-post educational thread",
    state: "Blocked",
    tone: "bad",
    acceptance: "5/6 checks · rights review pending",
    source: "Cost and Effect-state requirements",
    freshness: "2 hours ago",
    risk: "One reference has unknown reuse rights.",
  },
];

const STATE_KEYS: readonly StateKey[] = [
  "loading",
  "empty",
  "working",
  "error",
  "success",
  "partial",
  "blocked",
  "unknown",
  "offline",
];

const STATE_LABELS: Record<StateKey, string> = {
  loading: "Loading",
  empty: "Empty",
  working: "Working",
  error: "Error",
  success: "Success",
  partial: "Partial",
  blocked: "Blocked",
  unknown: "Unknown",
  offline: "Offline",
};

const STATE_TONES: Record<StateKey, Tone> = {
  loading: "info",
  empty: "neutral",
  working: "info",
  error: "bad",
  success: "good",
  partial: "warn",
  blocked: "bad",
  unknown: "bad",
  offline: "warn",
};

const SURFACE_CONTEXT: Record<
  SurfaceKey,
  { label: string; object: string; source: string; firstAction: string; native: string }
> = {
  today: {
    label: "Today",
    object: "cross-Project outcomes and Owner decisions",
    source: "Project, Routine, artifact, verification, and freshness projections",
    firstAction: "Describe the first business outcome",
    native: "Returning = partial attention. First-run = empty.",
  },
  projects: {
    label: "Projects",
    object: "governed Project workspaces",
    source: "Project list projection",
    firstAction: "Describe the first business outcome",
    native: "First-run empty list. Returning = one sample row.",
  },
  setup: {
    label: "Project setup",
    object: "research, charter, output contract, team, plan, boundaries, and simulation",
    source: "resumable Project draft and research ledger",
    firstAction: "Restore the business-description draft",
    native: "Five stages. Preview cannot activate.",
  },
  project: {
    label: "Project operating canvas",
    object: "goal, deliverables, evidence, decisions, and X operating stages",
    source: "current Project revision and source-linked artifact projections",
    firstAction: "Open the latest accepted deliverable",
    native: "Operating report, then X loop. Dispatch blocked.",
  },
  temporary: {
    label: "Temporary typed canvas",
    object: "approved component projections composed from real Project results",
    source: "selected source-linked artifacts and acceptance facts",
    firstAction: "Ask the manager a comparison question",
    native: "Temporary until pin. Pin is preview-only.",
  },
  people: {
    label: "Roles and Members",
    object: "Role Template and Project Member Runtime versions",
    source: "pinned Role revision, Member facts, and comparison evidence",
    firstAction: "Research the first non-manager Role",
    native: "Working / queued / waiting. Process death does not delete a Member.",
  },
  operations: {
    label: "Work continuity",
    object: "Routine occurrences, Tasks, Attempts, Effects, and recovery facts",
    source: "daemon-owned ledgers and independent observations",
    firstAction: "Create the first Routine from a verified output contract",
    native: "Four scene-native views: working, missed, unknown, blocked.",
  },
  knowledge: {
    label: "Knowledge",
    object: "Vault sources, admitted Memory, and model-bounded Context",
    source: "local source archive, provenance, index, and admission records",
    firstAction: "Choose a Project Vault source",
    native: "First-run empty. Returning = sample Vault with one excluded file.",
  },
  settings: {
    label: "Settings",
    object: "Personal Home, model connections, recovery, and diagnostics",
    source: "local product settings projection",
    firstAction: "Open Model Connections",
    native: "Hub only. No subscription, marketplace, or Installed Agents.",
  },
  connections: {
    label: "Model Connections",
    object: "Provider endpoints, model catalog facts, and explicit Member bindings",
    source: "connection health and SecretStore references",
    firstAction: "Choose a mainstream Provider template",
    native: "No secret field. No Connect button.",
  },
  capabilities: {
    label: "Capability review",
    object: "version-pinned Skill and MCP acquisition plus separate grants",
    source: "source, license, instructions, dependency, permission, and supply-chain review",
    firstAction: "Ask the Assistant to research a needed capability",
    native: "Review incomplete. No Install button.",
  },
};

function stateMessage(surface: SurfaceKey, state: StateKey) {
  const context = SURFACE_CONTEXT[surface];
  const messages: Record<StateKey, string> = {
    loading: `${context.label} is loading ${context.object}. The last safe projection stays visible; leaving does not discard the retained draft.`,
    empty: `${context.label} has no ${context.object} yet. First valuable action: ${context.firstAction}.`,
    working: `${context.label} has in-progress ${context.object}. Durable step, elapsed basis, and real controls are named; process activity is not completion.`,
    error: `${context.label} could not read ${context.source}. Existing input and last-known facts are retained; retry only the failed read.`,
    success: `${context.label} refreshed ${context.object}. Changed facts, evidence basis, freshness, and the next useful action are visible.`,
    partial: `${context.label} has usable ${context.object}, but one source or facet is missing. Coverage and omissions remain explicit.`,
    blocked: `${context.label} cannot advance because a required authority, permission, review, or qualified dependency is absent. Retained work is safe.`,
    unknown: `${context.label} cannot conclude the outcome from ${context.source}. Unknown is not success or zero; unsafe retry stays blocked.`,
    offline: `${context.label} is showing retained local facts. The Windows host or required dependency was offline, so no background-work claim is made.`,
  };
  return messages[state];
}

function activityTone(activity: Member["activity"]): Tone {
  if (activity === "working") return "info";
  if (activity === "queued") return "warn";
  return "neutral";
}

function activityWord(activity: Member["activity"]): string {
  if (activity === "working") return "Working";
  if (activity === "queued") return "Queued";
  return "Waiting";
}

function loopStepFor(
  scene: Scene,
  todayMode: TodayMode,
  xStage: XStage,
  operationsView: OperationsView,
): LoopStep {
  if (scene === "setup" || scene === "knowledge" || (scene === "today" && todayMode === "first-run") || scene === "projects") {
    return "ingest";
  }
  if (scene === "operations") {
    return operationsView === "working" ? "execute" : "verify";
  }
  if (scene === "project") {
    if (xStage === "preview") return "authorize";
    if (xStage === "receipt" || xStage === "readback") return "verify";
    if (xStage === "reflection") return "report";
    return "decide";
  }
  if (scene === "capabilities" || scene === "connections" || scene === "settings") return "authorize";
  if (scene === "temporary" || scene === "people") return "decide";
  return "decide";
}

function Tag({
  children,
  tone = "neutral",
}: {
  children: string;
  tone?: Tone;
}) {
  return (
    <span className="tag" data-tone={tone}>
      {children}
    </span>
  );
}

function Provenance({ kind }: { kind: ProvenanceKind }) {
  const label =
    kind === "observed"
      ? "Observed"
      : kind === "proposed"
        ? "Proposed"
        : kind === "governed"
          ? "Governed"
          : "Verified";
  return <span className="provenance" data-kind={kind}>{label}</span>;
}

function CycleStatus({ current }: { current: LoopStep }) {
  const step = LOOP_STEPS.find((item) => item.id === current) ?? LOOP_STEPS[1];
  return (
    <p className="cycle-status">
      Cycle {step.n}/6 · {step.label} · {step.job}
    </p>
  );
}

function Gap({
  children,
  environment = false,
}: {
  children: string;
  environment?: boolean;
}) {
  return (
    <Callout
      tone="warning"
      title={environment ? "Requires-backend + Requires-environment" : "Requires-backend"}
    >
      {children}
    </Callout>
  );
}

function noticeTone(
  tone: Tone,
): "info" | "success" | "warning" | "danger" | "neutral" {
  if (tone === "good") return "success";
  if (tone === "warn") return "warning";
  if (tone === "bad") return "danger";
  return tone;
}

function Notice({
  title,
  children,
  tone = "warn",
}: {
  title: string;
  children: string;
  tone?: Tone;
}) {
  return (
    <Callout title={title} tone={noticeTone(tone)}>
      {children}
    </Callout>
  );
}

function Heading({
  title,
  meta,
  action,
}: {
  title: string;
  meta?: string;
  action?: { label: string; onClick: () => void };
}) {
  return (
    <header className="section-heading">
      <div>
        <h3>{title}</h3>
        {meta ? <p>{meta}</p> : null}
      </div>
      {action ? (
        <button className="text-button" type="button" onClick={action.onClick}>
          {action.label}
        </button>
      ) : null}
    </header>
  );
}

function Segmented<T extends string>({
  label,
  value,
  items,
  onChange,
}: {
  label: string;
  value: T;
  items: ReadonlyArray<{ id: T; label: string }>;
  onChange: (value: T) => void;
}) {
  return (
    <div className="segmented" role="group" aria-label={label}>
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          aria-pressed={value === item.id}
          onClick={() => onChange(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}

function StatePanel({
  surface,
  state,
}: {
  surface: SurfaceKey;
  state: StateKey;
}) {
  const context = SURFACE_CONTEXT[surface];
  return (
    <section className="state-panel" data-tone={STATE_TONES[state]} aria-live="polite">
      <header>
        <Tag tone={STATE_TONES[state]}>{STATE_LABELS[state]}</Tag>
        <strong>{context.label}</strong>
      </header>
      <p>{stateMessage(surface, state)}</p>
      <dl>
        <div>
          <dt>What you still have</dt>
          <dd>
            {state === "empty"
              ? "Nothing admitted yet. No sample rows are invented."
              : state === "loading"
                ? "Last safe projection, if any; this prototype has no retained network cache."
                : "Retained local prototype facts. They are not current daemon authority."}
          </dd>
        </div>
        <div>
          <dt>What you can do</dt>
          <dd>
            {state === "blocked" || state === "unknown"
              ? "Inspect retained work. Unsafe retry and fake Confirm stay absent."
              : state === "empty"
                ? context.firstAction
                : state === "offline"
                  ? "Read retained facts. No 24/7 cloud catch-up is claimed."
                  : state === "error"
                    ? "Retry is not offered here; a failed read needs a future daemon path."
                    : context.firstAction}
          </dd>
        </div>
        <div>
          <dt>Native scene coverage</dt>
          <dd>{context.native}</dd>
        </div>
      </dl>
    </section>
  );
}

function TodayScene({
  mode,
  setMode,
  setScene,
  setXStage,
  setOperationsView,
  setSelectedOutcome,
}: {
  mode: TodayMode;
  setMode: (value: TodayMode) => void;
  setScene: (value: Scene) => void;
  setXStage: (value: XStage) => void;
  setOperationsView: (value: OperationsView) => void;
  setSelectedOutcome: (value: string) => void;
}) {
  const openPackage = () => {
    setXStage("package");
    setScene("project");
  };
  const openPreview = () => {
    setXStage("preview");
    setScene("project");
  };
  const openOutcome = (id: string) => {
    setSelectedOutcome(id);
    if (id === "a") {
      openPackage();
      return;
    }
    setScene("temporary");
  };

  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>
            {mode === "returning"
              ? "One public decision. Two outcomes can continue. Nothing published."
              : "Start with one business outcome, not Agent configuration."}
          </h2>
          <p>
            {mode === "returning"
              ? "Lin proposed Package A. The kernel has not issued a confirmable preview. One prior Effect is still unknown."
              : "Personal researches, proposes a design, simulates one cycle, and stops at a launch preview you can reject."}
          </p>
        </div>
        <Segmented
          label="Today prototype mode"
          value={mode}
          items={[
            { id: "returning", label: "Returning today" },
            { id: "first-run", label: "First run" },
          ]}
          onChange={setMode}
        />
      </section>

      {mode === "returning" ? (
        <>
          <section className="decision-packet" aria-label="Owner decision packet">
            <header>
              <div className="packet-marks">
                <Tag tone="warn">Needs Owner · review only</Tag>
                <Provenance kind="proposed" />
              </div>
              <span>Chat announced · canvas confirms · no “don’t ask again”</span>
            </header>
            <h3>Inspect Package A before any public action</h3>
            <p>
              7-post thread, visual brief, citations, and alt text are assembled as a planned post. Publishing would speak as you. This packet is not daemon-issued and cannot be confirmed here.
            </p>
            <dl className="packet-facts">
              <div>
                <dt>Consequence</dt>
                <dd>Public communication under your identity</dd>
              </div>
              <div>
                <dt>Reversibility</dt>
                <dd>Cannot be fully undone after distribution</dd>
              </div>
              <div>
                <dt>Alternatives</dt>
                <dd>Keep waiting · export draft · reject the package</dd>
              </div>
              <div>
                <dt>Kernel truth</dt>
                <dd>No persisted Intent · no connector · no receipt</dd>
              </div>
            </dl>
            <details className="why-layer">
              <summary>Why A is first</summary>
              <p>
                A is the only package with an openable thread, linked excerpts, and a mock accessibility check. B is partial. C retains a rights conflict. The unknown prior Effect is excluded from this recommendation because unknown is not success.
              </p>
            </details>
            <div className="packet-actions">
              <button className="primary-button" type="button" onClick={openPackage}>
                Inspect Package A
              </button>
              <button className="secondary-button" type="button" onClick={openPreview}>
                Open canvas preview
              </button>
            </div>
          </section>

          <section className="exception-lanes" aria-label="Exception-first scan. Not Inbox.">
            <button type="button" data-tone="warn" onClick={openPackage}>
              <span>Needs you</span>
              <Provenance kind="proposed" />
              <strong>Package A review</strong>
              <small>Planned, not published</small>
            </button>
            <button
              type="button"
              data-tone="info"
              onClick={() => openOutcome("b")}
            >
              <span>Can continue</span>
              <Provenance kind="observed" />
              <strong>Outcome B revision</strong>
              <small>Rui waiting · process not running</small>
            </button>
            <button
              type="button"
              data-tone="bad"
              onClick={() => {
                setOperationsView("unknown");
                setScene("operations");
              }}
            >
              <span>Unknown</span>
              <Provenance kind="observed" />
              <strong>Prior Effect not terminal</strong>
              <small>Blind retry blocked</small>
            </button>
            <button
              type="button"
              data-tone="warn"
              onClick={() => {
                setOperationsView("missed");
                setScene("operations");
              }}
            >
              <span>Missed</span>
              <Provenance kind="observed" />
              <strong>2 of 3 occurrences</strong>
              <small>Queue-latest research only</small>
            </button>
          </section>

          <section className="open-section">
            <Heading title="Expected outcomes" meta="Deliverable and acceptance first. Not a KPI wall." />
            <ol className="result-list">
              {OUTCOMES.map((outcome) => (
                <li key={outcome.id}>
                  <div>
                    <strong>{outcome.title}</strong>
                    <span>{outcome.format}</span>
                  </div>
                  <div>
                    <Tag tone={outcome.tone}>{outcome.state}</Tag>
                    <small>{outcome.freshness}</small>
                    <button
                      className="inline-button"
                      type="button"
                      onClick={() => openOutcome(outcome.id)}
                    >
                      Inspect
                    </button>
                  </div>
                </li>
              ))}
            </ol>
          </section>

          <section className="staff-strip" aria-label="Member activity versus queue">
            <Heading
              title="Who is actually working"
              meta="Queued is not live. Process exit does not delete a Member."
              action={{ label: "Open Members", onClick: () => setScene("people") }}
            />
            <div className="staff-table-wrap" tabIndex={0} aria-label="Scrollable Member activity">
              <table className="staff-table">
                <caption>Member activity. Queued and waiting are not running processes.</caption>
                <thead>
                  <tr>
                    <th scope="col">Member</th>
                    <th scope="col">Activity</th>
                    <th scope="col">Now</th>
                    <th scope="col">Provenance</th>
                  </tr>
                </thead>
                <tbody>
                  {MEMBERS.map((item) => (
                    <tr key={item.id}>
                      <th scope="row">
                        <button className="inline-button" type="button" onClick={() => setScene("people")}>
                          {item.name}
                        </button>
                        <small>{item.role}</small>
                      </th>
                      <td>
                        <Tag tone={activityTone(item.activity)}>{activityWord(item.activity)}</Tag>
                      </td>
                      <td>{item.activityLabel}</td>
                      <td>
                        <Provenance kind="observed" />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section className="outcome-ledger">
            <Heading
              title="Latest accepted result"
              meta="Prototype evidence treatment; not a current product receipt."
              action={{ label: "Open X Project", onClick: () => setScene("project") }}
            />
            <div className="accepted-line">
              <div>
                <strong>Weekly topic brief</strong>
                <span>Six sourced tensions · three selected topics · linked handoff contract</span>
              </div>
              <dl className="ledger-facts">
                <div>
                  <dt>Openability</dt>
                  <dd>Mock check passed</dd>
                </div>
                <div>
                  <dt>Verification</dt>
                  <dd>
                    Independent-check example
                    <Provenance kind="verified" />
                  </dd>
                </div>
                <div>
                  <dt>Cost</dt>
                  <dd>Estimated ¥29.60 · actual unknown · not ¥0</dd>
                </div>
              </dl>
            </div>
          </section>

          <Notice title="Missed while the Windows host was off">
            2 of 3 scheduled occurrences did not run. Only the newest eligible research occurrence may queue; publication requires a fresh review. Host offline means no 24/7 cloud work.
          </Notice>
        </>
      ) : (
        <section className="first-run">
          <div className="first-run-copy">
            <Tag tone="info">No Projects yet</Tag>
            <h3>Aha is a launch preview you can reject</h3>
            <p>
              Describe the outcome in business language. Research, design, simulate one cycle, then stop. Nothing becomes a Project until a future daemon-issued revision is confirmed.
            </p>
            <ul>
              <li>Sources, conflicts, rights, and freshness stay visible.</li>
              <li>Only the base Project Manager Role is built in.</li>
              <li>Keys never enter chat. External publish stays unavailable.</li>
            </ul>
          </div>
          <div className="first-run-action">
            <strong>About 5 minutes to a structured preview</strong>
            <span>Local prototype draft only. Skip if you want to inspect the X sample.</span>
            <button className="primary-button" type="button" onClick={() => setScene("setup")}>
              Describe first outcome
            </button>
            <button className="secondary-button" type="button" onClick={() => setMode("returning")}>
              Explore the X sample
            </button>
          </div>
        </section>
      )}

      <Gap>
        Today requires daemon-backed Projects, Routines, evidence, conversations, missed-run facts, and independent verification.
      </Gap>
    </div>
  );
}

function ProjectsScene({
  firstRun,
  setScene,
  setTodayMode,
}: {
  firstRun: boolean;
  setScene: (value: Scene) => void;
  setTodayMode: (value: TodayMode) => void;
}) {
  if (firstRun) {
    return (
      <div className="scene-stack">
        <section className="first-run">
          <div className="first-run-copy">
            <Tag tone="info">No Projects</Tag>
            <h2>Projects are governed workspaces, not an Agent store</h2>
            <p>
              A Project stays inactive until you confirm an exact future daemon revision. This list does not invent the X sample while you are on first run.
            </p>
          </div>
          <div className="first-run-action">
            <strong>First valuable action</strong>
            <span>Describe one business outcome. Exploring the sample switches Today to returning.</span>
            <button className="primary-button" type="button" onClick={() => setScene("setup")}>
              Describe first outcome
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => {
                setTodayMode("returning");
                setScene("project");
              }}
            >
              Explore the X sample
            </button>
          </div>
        </section>
        <Gap>
          Project list, charter, activation, archive, and restore-point projections require backend support.
        </Gap>
      </div>
    );
  }

  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>One current Project. Nothing is always-on.</h2>
          <p>Open a Project to its operating report. Team and Inbox are not destinations.</p>
        </div>
        <button className="secondary-button" type="button" onClick={() => setScene("setup")}>
          New Project draft
        </button>
      </section>
      <section className="work-surface">
        <Heading title="Project list" meta="Sample row. Activation, archive, and restore remain unavailable." />
        <div className="comparison-table-wrap" tabIndex={0} aria-label="Scrollable Project list">
          <table>
            <caption>Owner Projects. One sample workspace; not an organization catalog.</caption>
            <thead>
              <tr>
                <th scope="col">Project</th>
                <th scope="col">Goal</th>
                <th scope="col">Needs you</th>
                <th scope="col">Manager</th>
                <th scope="col">Open</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <th scope="row">X content operation</th>
                <td>3 accepted packages / week</td>
                <td>Package A review · planned, not published</td>
                <td>Lin</td>
                <td>
                  <button className="inline-button" type="button" onClick={() => setScene("project")}>
                    Open report
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
      <Gap>
        Multi-Project inventory, archive-first lifecycle, and local restore points require backend support.
      </Gap>
    </div>
  );
}

function SetupStageContent({
  stage,
  brief,
  setBrief,
}: {
  stage: SetupStage;
  brief: string;
  setBrief: (value: string) => void;
}) {
  if (stage === "describe") {
    return (
      <section className="work-surface">
        <Heading
          title="What should this Project accomplish?"
          meta="State the business outcome, audience, cadence, constraints, and what a useful deliverable looks like."
        />
        <label className="field">
          <span>Business description</span>
          <TextArea
            value={brief}
            onChange={(next) => setBrief(next.slice(0, 1200))}
            rows={6}
          />
          <small>Prototype-local input. Do not enter credentials or third-party confidential data.</small>
          <small>{brief.length} / 1200 characters</small>
        </label>
        <Notice title="What happens next" tone="info">
          A future Personal Assistant would perform broad read-only research automatically. External text remains untrusted and cannot execute, install, or expand permissions.
        </Notice>
      </section>
    );
  }

  if (stage === "research") {
    return (
      <section className="work-surface">
        <Heading
          title="Research coverage"
          meta="Static prototype sample. A real research ledger and resumable draft require backend support."
        />
        <div className="research-summary">
          <div>
            <strong>9 sources inspected</strong>
            <span>6 usable · 2 conflicting · 1 rights unknown</span>
          </div>
          <Tag tone="warn">Partial · conflict retained</Tag>
        </div>
        <table>
          <caption>Research questions. Conflicts are retained, not averaged.</caption>
          <thead>
            <tr>
              <th scope="col">Question</th>
              <th scope="col">Coverage</th>
              <th scope="col">Finding</th>
              <th scope="col">Treatment</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th scope="row">Audience tension</th>
              <td>4 primary + 2 secondary</td>
              <td>Control without micro-supervision</td>
              <td>Candidate for charter</td>
            </tr>
            <tr>
              <th scope="row">Content cadence</th>
              <td>2 conflicting sources</td>
              <td>No reliable universal cadence</td>
              <td>Owner constraint wins</td>
            </tr>
            <tr>
              <th scope="row">Reference rights</th>
              <td>1 source unclear</td>
              <td>Reuse permission unknown</td>
              <td>Exclude from outputs</td>
            </tr>
          </tbody>
        </table>
        <Notice title="Research conflict">
          The disagreement is not averaged away or admitted to Memory. Preview will show both the current recommendation and the missing evidence.
        </Notice>
      </section>
    );
  }

  if (stage === "design") {
    return (
      <section className="work-surface">
        <Heading
          title="Proposed operating design"
          meta="Business objects first. Runtime and capability mechanics stay one disclosure deeper."
        />
        <dl className="definition-list">
          <div>
            <dt>Primary goal</dt>
            <dd><strong>3 accepted X content packages each week</strong><small>No follower or revenue guarantee</small></dd>
          </div>
          <div>
            <dt>Output contract</dt>
            <dd><strong>Topic brief · draft · visual brief · citations · alt text · review package</strong><small>Openability, source, freshness, and acceptance remain visible</small></dd>
          </div>
          <div>
            <dt>Project Manager</dt>
            <dd><strong>Lin · specialized built-in base Role</strong><small>Plans, delegates, verifies, summarizes, and reflects</small></dd>
          </div>
          <div>
            <dt>Researched Members</dt>
            <dd><strong>Mei · audience research / Rui · content editing</strong><small>Each requires an explicit Provider/model and scoped grants</small></dd>
          </div>
          <div>
            <dt>Work cycle</dt>
            <dd><strong>Research → brief → draft → review → publish package → readback → reflection</strong><small>No-overlap and queue-latest</small></dd>
          </div>
          <div>
            <dt>Owner boundary</dt>
            <dd><strong>Team, model, capability, permission, primary-goal, and external-rule changes</strong><small>Exact structured preview before admission</small></dd>
          </div>
        </dl>
      </section>
    );
  }

  if (stage === "simulate") {
    return (
      <section className="work-surface">
        <Heading
          title="One-cycle simulation"
          meta="Target interaction sample only. No Agent process, Provider request, file write, or connector call occurred."
        />
        <ol className="simulation-path">
          <li data-state="done"><strong>Research handoff</strong><span>Mock source ledger satisfies 5/6 checks</span></li>
          <li data-state="done"><strong>Draft creation</strong><span>Thread and visual brief represented as sample artifacts</span></li>
          <li data-state="partial"><strong>Independent checks</strong><span>Alt text and source rights expose 2 gaps</span></li>
          <li data-state="blocked"><strong>External dispatch</strong><span>No qualified connector or daemon preview</span></li>
        </ol>
        <div className="gap-summary">
          <div>
            <Tag tone="warn">Gap 1</Tag>
            <strong>Reference rights unknown</strong>
            <span>Exclude the source or provide a licensed replacement before package acceptance.</span>
          </div>
          <div>
            <Tag tone="bad">Gap 2</Tag>
            <strong>X connector unavailable</strong>
            <span>Launch may still create a draft-only Project in the target design; publishing cannot be claimed.</span>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="work-surface">
      <Heading
        title="Structured launch preview"
        meta="Candidate target-state preview. Only a future daemon-issued exact revision could become confirmable."
      />
      <div className="preview-summary">
        <div>
          <Tag tone="warn">Not confirmable</Tag>
          <strong>X content operation · Project draft</strong>
          <span>Draft-only external policy · one simulated cycle · two retained gaps</span>
        </div>
        <span className="revision-label">Candidate revision · prototype-v2</span>
      </div>
      <dl className="definition-list">
        <div><dt>Charter</dt><dd><strong>Operate a source-backed X content cycle for an OPC developer product</strong><small>Owner scope; no market validation claim</small></dd></div>
        <div><dt>Goal and outputs</dt><dd><strong>3 accepted packages / week</strong><small>Five openable deliverables and explicit acceptance checks</small></dd></div>
        <div><dt>Team and models</dt><dd><strong>1 manager + 2 researched Member candidates</strong><small>Three explicit prototype model bindings; never inherited silently</small></dd></div>
        <div><dt>Capabilities</dt><dd><strong>Read-only research candidate; no executable MCP grant</strong><small>Skill/MCP review stays separate from Project/Member grants</small></dd></div>
        <div><dt>Permissions</dt><dd><strong>Internal reversible work; external publication denied</strong><small>No raw secret reaches UI, Context, chat, or Member</small></dd></div>
        <div><dt>Triggers</dt><dd><strong>Manual · schedule · accepted artifact · Project state</strong><small>External-event and data-condition triggers remain unavailable</small></dd></div>
        <div><dt>Simulation</dt><dd><strong>Partial target-state sample</strong><small>Rights and connector gaps preserved</small></dd></div>
      </dl>
      <Notice title="Activation unavailable" tone="bad">
        This Canvas has no daemon-owned Project authority, exact revision preview, confirmation path, or activation receipt. No fake Confirm button is shown.
      </Notice>
    </section>
  );
}

function SetupScene({
  stage,
  setStage,
  brief,
  setBrief,
}: {
  stage: SetupStage;
  setStage: (value: SetupStage) => void;
  brief: string;
  setBrief: (value: string) => void;
}) {
  const index = SETUP_STAGES.findIndex((item) => item.id === stage);
  const move = (direction: -1 | 1) => {
    const next = SETUP_STAGES[Math.max(0, Math.min(SETUP_STAGES.length - 1, index + direction))];
    setStage(next.id);
  };

  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>From business description to a safe launch preview</h2>
          <p>Five focused stages; advanced Runtime mechanics remain inside the relevant review.</p>
        </div>
        <div className="step-count" aria-live="polite">
          <strong>Step {index + 1} of {SETUP_STAGES.length}</strong>
          <span>{SETUP_STAGES[index].time}</span>
        </div>
      </section>

      <nav className="step-nav" aria-label="Project setup stages">
        {SETUP_STAGES.map((item, itemIndex) => (
          <button
            key={item.id}
            type="button"
            aria-current={stage === item.id ? "step" : undefined}
            onClick={() => setStage(item.id)}
          >
            <span>{itemIndex + 1}</span>
            <strong>{item.label}</strong>
          </button>
        ))}
      </nav>

      <SetupStageContent stage={stage} brief={brief} setBrief={setBrief} />

      <div className="flow-actions">
        <button
          className="secondary-button"
          type="button"
          disabled={index === 0}
          onClick={() => move(-1)}
        >
          Previous stage
        </button>
        {index < SETUP_STAGES.length - 1 ? (
          <button className="primary-button" type="button" onClick={() => move(1)}>
            {stage === "describe" ? "Review research sample" : `Continue to ${SETUP_STAGES[index + 1].label}`}
          </button>
        ) : (
          <span className="flow-end">End of prototype flow · activation requires backend authority</span>
        )}
      </div>

      <Gap>
        Research orchestration, resumable draft custody, simulation, exact preview, confirmation, activation, and receipt are target behavior.
      </Gap>
    </div>
  );
}

function PackagePanel() {
  return (
    <section className="package-layout">
      <div className="artifact-preview">
        <header>
          <div>
            <Tag tone="warn">Needs Owner · candidate</Tag>
            <h3>Package A · Local-first control</h3>
          </div>
          <span>Fresh 18 min ago</span>
        </header>
        <p className="thread-copy">
          “The useful boundary is not whether an AI can act. It is whether every action is bounded, visible, recoverable, and independently checked.”
        </p>
        <ol className="thread-cards" aria-label="Full 7-post thread preview">
          {THREAD_POSTS.map((post) => (
            <li key={post.n}>
              <span>{post.n}</span>
              <p>{post.copy}</p>
            </li>
          ))}
        </ol>
        <dl className="artifact-parts">
          <div><dt>Thread</dt><dd>7 posts · mock file opens</dd></div>
          <div><dt>Visual brief</dt><dd>1 annotated image direction · mock</dd></div>
          <div><dt>Citations</dt><dd>3 linked excerpts · rights checked</dd></div>
          <div><dt>Alt text</dt><dd>112 characters · mock editorial check passed</dd></div>
        </dl>
      </div>
      <aside className="acceptance-checks">
        <h3>Acceptance</h3>
        <ul>
          <li><span>Pass</span><div><strong>Openable</strong><small>Mock files represented</small></div></li>
          <li><span>Pass</span><div><strong>Source-backed</strong><small>3 excerpts linked</small></div></li>
          <li><span>Pass</span><div><strong>Accessibility copy</strong><small>Mock check passed</small></div></li>
          <li><span>Not run</span><div><strong>Publication</strong><small>Connector unavailable</small></div></li>
        </ul>
        <p>Checkmarks are text-backed mock states, not completion evidence. This is a planned post: it does not publish until a future Owner confirmation of a daemon-issued preview.</p>
      </aside>
    </section>
  );
}

function PublishPreviewPanel() {
  return (
    <section className="decision-packet">
      <header>
        <div className="packet-marks">
          <Tag tone="bad">Cannot confirm</Tag>
          <Provenance kind="proposed" />
        </div>
        <span>Target-state structure. Not daemon-issued. Chat has no Approve.</span>
      </header>
      <h3>External-action preview · Package A</h3>
      <p>
        Lin asked for Owner review. A future kernel would persist Intent, fence the action, then dispatch Effect only after this canvas confirmation. This prototype cannot take that step.
      </p>
      <dl className="packet-facts">
        <div>
          <dt>Action</dt>
          <dd>Publish a 7-post thread and one image</dd>
        </div>
        <div>
          <dt>Targets</dt>
          <dd>Owner’s X account · public audience</dd>
        </div>
        <div>
          <dt>Impact</dt>
          <dd>Speaks as you. Distribution cannot be fully undone.</dd>
        </div>
        <div>
          <dt>Reversibility</dt>
          <dd>Prevention-only after dispatch. No silent catch-up publish.</dd>
        </div>
        <div>
          <dt>Alternatives</dt>
          <dd>Keep waiting · export the draft · reject the package</dd>
        </div>
        <div>
          <dt>Policy</dt>
          <dd>Draft and wait · launch-approved envelope · no “skip all approvals”</dd>
        </div>
        <div>
          <dt>Evidence</dt>
          <dd>Package checks 6/6 mock · connector qualification absent</dd>
        </div>
        <div>
          <dt>Cost</dt>
          <dd>Actual unknown · unknown is not zero</dd>
        </div>
      </dl>
      <details className="why-layer">
        <summary>Why this cannot be confirmed</summary>
        <p>
          Missing: exact daemon revision, persisted Intent/Effect, qualified connector, and stale-preview guard. Artifact acceptance does not prove dispatch. Keys never belong in this form; SecretStore is a daemon-owned handoff.
        </p>
      </details>
      <Notice title="No confirm control on purpose" tone="bad">
        A Confirm or Publish button here would fake kernel authority. Open the package, wait, or reject the candidate. There is no “don’t ask again” grant.
      </Notice>
    </section>
  );
}

function ReceiptPanel() {
  return (
    <section className="work-surface">
      <Heading
        title="Target-state receipt sample"
        meta="Future interaction example only; it does not assert that a publish occurred."
      />
      <div className="receipt-head">
        <Tag tone="good">Applied · target-state mock</Tag>
        <strong>External Effect reached a terminal observation</strong>
      </div>
      <dl className="definition-list">
        <div><dt>What changed</dt><dd><strong>Thread and image observed at the expected public target</strong><small>Mock destination readback</small></dd></div>
        <div><dt>Intent / Effect</dt><dd><strong>Prototype identifiers only</strong><small>No authority record exists</small></dd></div>
        <div><dt>Connector result</dt><dd><strong>Target-state response accepted</strong><small>Not sufficient for Project outcome completion</small></dd></div>
        <div><dt>Independent check</dt><dd><strong>Target and content digest matched in this sample</strong><small>Target interaction design; not-run in reality</small></dd></div>
        <div><dt>Next</dt><dd><strong>Wait for metric and comment readback</strong><small>No business outcome guarantee</small></dd></div>
      </dl>
      <Gap environment>
        A real receipt requires daemon authority, a qualified connector, destination reconciliation, fencing, and independent verification.
      </Gap>
    </section>
  );
}

function ReadbackPanel() {
  return (
    <section className="work-surface">
      <Heading
        title="Readback after publication"
        meta="Target-state sample separates receipt, observations, and business outcomes."
      />
      <div className="readback-grid">
        <div>
          <span>Publication observation</span>
          <strong>Present · target-state mock</strong>
          <small>Freshness: 2 hours after sample receipt</small>
        </div>
        <div>
          <span>Views</span>
          <strong>Unknown</strong>
          <small>Metric scope and source not available</small>
        </div>
        <div>
          <span>Relevant comments</span>
          <strong>2 sample observations</strong>
          <small>Reply suggestions require applicable review</small>
        </div>
        <div>
          <span>Commercial result</span>
          <strong>Not inferred</strong>
          <small>No follower, lead, or revenue guarantee</small>
        </div>
      </div>
      <Notice title="Partial readback">
        The receipt can be terminal while business metrics remain partial or unknown. Missing metrics never render as zero.
      </Notice>
    </section>
  );
}

function ReflectionPanel({
  setScene,
}: {
  setScene: (value: Scene) => void;
}) {
  return (
    <section className="work-surface">
      <Heading
        title="Manager reflection"
        meta="A reflection proposes the next cycle and a versioned Runtime improvement; one event never changes a global Role."
      />
      <div className="reflection-grid">
        <div>
          <h3>Comparable gap</h3>
          <p>Package B lost time at accessibility review because the visual brief omitted an explicit alt-text handoff.</p>
        </div>
        <div>
          <h3>Next-cycle adjustment</h3>
          <p>Add the alt-text contract before visual drafting and verify it at the editor handoff.</p>
        </div>
        <div>
          <h3>Persistent proposal</h3>
          <p>Rui Member Runtime v4 candidate adds a preflight checklist. Compare, simulate, and retain rollback to v3.</p>
        </div>
      </div>
      <button className="secondary-button" type="button" onClick={() => setScene("people")}>
        Inspect Runtime version proposal
      </button>
      <Notice title="Candidate only" tone="info">
        The manager may adjust a bounded Task strategy, but persistent Member or global Role changes follow versioned preview and approval rules.
      </Notice>
    </section>
  );
}

function ProjectScene({
  stage,
  setStage,
  setScene,
  candidatePreview,
}: {
  stage: XStage;
  setStage: (value: XStage) => void;
  setScene: (value: Scene) => void;
  candidatePreview: boolean;
}) {
  return (
    <div className="scene-stack">
      <section className="project-header">
        <div>
          <h2>Three accepted, source-backed content packages each week</h2>
          <p>Current cycle: A needs review, B is partial, C is blocked on rights. No real publication has occurred.</p>
        </div>
        <div className="header-actions">
          <button className="secondary-button" type="button" onClick={() => setScene("projects")}>
            All Projects
          </button>
          <button className="secondary-button" type="button" onClick={() => setScene("temporary")}>
            Compare outcomes
          </button>
          <button className="secondary-button" type="button" onClick={() => setScene("people")}>
            Open Members
          </button>
        </div>
      </section>

      {candidatePreview ? (
        <Notice title="Conversation announced a governed-work candidate" tone="info">
          The unsent group draft was interpreted locally as a Task/revision candidate. Approval is not in the chat. Only a future daemon-issued structured preview on this canvas could become confirmable.
        </Notice>
      ) : null}

      <section className="operating-report" aria-label="Stable Project operating report">
        <Heading
          title="Operating report · Project template"
          meta="Stable layout. Ad-hoc comparison is temporary until pinned. No generated code or invented metrics."
        />
        <div className="report-grid">
          <section>
            <span>Goal</span>
            <strong>3 accepted, source-backed packages / week</strong>
            <small>Phase: weekly cycle · no follower or revenue guarantee</small>
          </section>
          <section>
            <span>Manager summary</span>
            <strong>A needs review. B is partial. C is blocked on rights.</strong>
            <small>Candidate briefing · unknown Effect excluded</small>
          </section>
          <section>
            <span>Needs you</span>
            <strong>Inspect Package A · planned, not published</strong>
            <small>Canvas confirms; chat has no Approve</small>
          </section>
          <section>
            <span>Members</span>
            <strong>Lin working · Mei queued · Rui waiting</strong>
            <small>Queued is not a running process</small>
          </section>
          <section>
            <span>Can continue</span>
            <strong>Outcome B revision</strong>
            <small>Rui after evidence handoff</small>
          </section>
          <section>
            <span>Blocked / unknown</span>
            <strong>C rights · 1 unknown Effect · 2 missed</strong>
            <small>Opened from this Project, not Inbox</small>
          </section>
          <section>
            <span>Latest artifact</span>
            <strong>Weekly topic brief · mock independent check</strong>
            <small>Fresh 18 min · openable sample</small>
          </section>
          <section>
            <span>Cost basis</span>
            <strong>¥29.60 estimated + actual unknown</strong>
            <small>Warning only · no product budget stop</small>
          </section>
        </div>
      </section>

      <nav className="stage-tabs" aria-label="X operating-loop stage samples">
        {X_STAGES.map((item) => (
          <button
            key={item.id}
            type="button"
            aria-current={stage === item.id ? "page" : undefined}
            onClick={() => setStage(item.id)}
          >
            {item.label}
          </button>
        ))}
      </nav>

      {stage === "package" ? <PackagePanel /> : null}
      {stage === "preview" ? <PublishPreviewPanel /> : null}
      {stage === "receipt" ? <ReceiptPanel /> : null}
      {stage === "readback" ? <ReadbackPanel /> : null}
      {stage === "reflection" ? <ReflectionPanel setScene={setScene} /> : null}

      <section className="loop-ledger">
        <Heading title="Complete X loop" meta="Each stage retains a separate evidence state." />
        <ol>
          <li data-state="done"><strong>Research</strong><span>Mock accepted brief</span></li>
          <li data-state="done"><strong>Topic plan</strong><span>3 selected candidates</span></li>
          <li data-state="partial"><strong>Draft + media</strong><span>1 revision pending</span></li>
          <li data-state="waiting"><strong>Package review</strong><span>Needs Owner</span></li>
          <li data-state="blocked"><strong>Dispatch</strong><span>Requires environment</span></li>
          <li data-state="sample"><strong>Receipt</strong><span>Target-state sample</span></li>
          <li data-state="sample"><strong>Readback</strong><span>Target-state sample</span></li>
          <li data-state="sample"><strong>Reflection</strong><span>Version proposal</span></li>
        </ol>
      </section>

      <div className="header-actions">
        <button className="secondary-button" type="button" onClick={() => setScene("capabilities")}>
          Open Skill / MCP review
        </button>
      </div>

      <Gap environment>
        Project authority, group Conversation, typed projections, X dispatch, receipts, readback, and Runtime reflection are not implemented or qualified.
      </Gap>
    </div>
  );
}

function TemporaryScene({
  selected,
  setSelected,
  pinned,
  setPinned,
  setScene,
}: {
  selected: string;
  setSelected: (value: string) => void;
  pinned: boolean;
  setPinned: (value: boolean) => void;
  setScene: (value: Scene) => void;
}) {
  const outcome = OUTCOMES.find((item) => item.id === selected) ?? OUTCOMES[0];

  return (
    <div className="scene-stack">
      <section className="temporary-header">
        <div>
          <h2>Which outcome is ready, and why?</h2>
          <p>Temporary typed composition from mock Artifact objects; source, freshness, omissions, and decisions remain visible.</p>
        </div>
        <div className="header-actions">
          <button className="secondary-button" type="button" onClick={() => setScene("project")}>
            Stable operating report
          </button>
          <button
            className="secondary-button"
            type="button"
            aria-pressed={pinned}
            onClick={() => setPinned(!pinned)}
          >
            {pinned ? "Remove pin preview" : "Preview pin intent"}
          </button>
        </div>
      </section>

      <Notice title={pinned ? "Pin intent previewed · not saved" : "Temporary by default"} tone="info">
        {pinned
          ? "Only local visual state changed. Project persistence and template versioning require backend authority."
          : "This view is not retained by default. It contains no generated code, eval, network read, invented metric, or hidden failure."}
      </Notice>

      <section className="comparison-surface">
        <Heading
          title="Outcome comparison"
          meta="Select an outcome to inspect the same source-linked mock object."
        />
        <div className="comparison-table-wrap" tabIndex={0} aria-label="Scrollable outcome comparison">
          <table>
            <caption>Three mock outcomes. Planned is not published.</caption>
            <thead>
              <tr>
                <th scope="col">Outcome</th>
                <th scope="col">Acceptance</th>
                <th scope="col">Source</th>
                <th scope="col">Freshness</th>
                <th scope="col">State</th>
                <th scope="col">Inspect</th>
              </tr>
            </thead>
            <tbody>
              {OUTCOMES.map((item) => (
                <tr key={item.id} data-selected={item.id === outcome.id}>
                  <th scope="row">{item.title}</th>
                  <td>{item.acceptance}</td>
                  <td>{item.source}</td>
                  <td>{item.freshness}</td>
                  <td><Tag tone={item.tone}>{item.state}</Tag></td>
                  <td>
                    <button
                      className="inline-button"
                      type="button"
                      aria-pressed={item.id === outcome.id}
                      onClick={() => setSelected(item.id)}
                    >
                      Inspect
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <div className="typed-canvas-grid">
        <section className="open-section">
          <Heading title={outcome.title} meta={outcome.format} />
          <dl className="definition-list compact">
            <div><dt>Acceptance</dt><dd>{outcome.acceptance}</dd></div>
            <div><dt>Evidence basis</dt><dd>{outcome.source}</dd></div>
            <div><dt>Freshness</dt><dd>{outcome.freshness}</dd></div>
            <div><dt>Unresolved risk</dt><dd>{outcome.risk}</dd></div>
          </dl>
        </section>
        <section className="decision-panel">
          <Heading title="Manager recommendation" />
          <p>Review A first. Keep B in revision. Exclude C’s disputed source until rights are resolved.</p>
          <small>Candidate recommendation only; it cannot accept, publish, revise, or prove completion.</small>
        </section>
        <section className="evidence-panel">
          <Heading title="Evidence honesty" />
          <ul>
            <li><strong>3 drafts</strong><span>represented as openable mock artifacts</span></li>
            <li><strong>0 real receipts</strong><span>nothing dispatched</span></li>
            <li><strong>Outcome metrics unknown</strong><span>never rendered as zero</span></li>
          </ul>
        </section>
      </div>

      <Gap>
        Reading real results, composing a governed board, pinning, and saving a Project report template need typed backend projections.
      </Gap>
    </div>
  );
}

function PeopleScene({
  view,
  setView,
  memberId,
  setMemberId,
}: {
  view: PeopleView;
  setView: (value: PeopleView) => void;
  memberId: MemberId;
  setMemberId: (value: MemberId) => void;
}) {
  const member = MEMBERS.find((item) => item.id === memberId) ?? MEMBERS[0];

  return (
    <div className="scene-stack">
      <section className="object-chain" aria-label="Role and execution object chain">
        <div><strong>Role Runtime Template</strong><span>Reusable versioned recipe</span></div>
        <span aria-hidden="true">→</span>
        <div><strong>Member Runtime</strong><span>Long-lived in one Project</span></div>
        <span aria-hidden="true">→</span>
        <div><strong>Task</strong><span>Bounded authority contract</span></div>
        <span aria-hidden="true">→</span>
        <div><strong>Agent process / Attempt</strong><span>Disposable execution</span></div>
      </section>

      <Segmented
        label="People detail view"
        value={view}
        items={[
          { id: "members", label: "Project Members" },
          { id: "role", label: "Role Template" },
          { id: "version", label: "Runtime version proposal" },
        ]}
        onChange={setView}
      />

      {view === "members" ? (
        <div className="people-layout">
          <nav className="member-list" aria-label="Project Members">
            {MEMBERS.map((item) => (
              <button
                key={item.id}
                type="button"
                aria-current={item.id === member.id ? "page" : undefined}
                onClick={() => setMemberId(item.id)}
              >
                <span><strong>{item.name}</strong><small>{item.role}</small></span>
                <Tag tone={activityTone(item.activity)}>{activityWord(item.activity)}</Tag>
              </button>
            ))}
          </nav>
          <section className="work-surface">
            <Heading title={`${member.name} · ${member.role}`} meta={`${member.version} · ${member.template}`} />
            <dl className="definition-list">
              <div><dt>Responsibility</dt><dd>{member.responsibility}</dd></div>
              <div><dt>Activity</dt><dd>{member.activityLabel}</dd></div>
              <div><dt>Current work</dt><dd>{member.current}</dd></div>
              <div><dt>Next</dt><dd>{member.next}</dd></div>
              <div><dt>Latest accepted result</dt><dd>{member.accepted}</dd></div>
              <div><dt>Needs Owner</dt><dd>{member.ownerNeed}</dd></div>
              <div><dt>Provider / model</dt><dd>{member.model}</dd></div>
              <div><dt>Cost basis</dt><dd>{member.cost}</dd></div>
              <div>
                <dt>Autonomy envelope</dt>
                <dd>
                  Owner confirms public actions, team, model, capability, permission, and primary-goal changes.
                  <small>Not a writable autonomy dial in this prototype.</small>
                </dd>
              </div>
            </dl>
            <details className="why-layer">
              <summary>Runtime recipe</summary>
              <p>
                Provider binding, grants, Memory, and permissions are Project-isolated. Engine identity stays in diagnostics. Process exit does not delete this Member.
              </p>
            </details>
            <Notice title="Stable Member, disposable process" tone="info">
              Task execution starts a new Attempt from an exact Runtime revision. Exit, retry, or engine failure does not delete the Member, Conversation, Memory, artifacts, or evidence.
            </Notice>
          </section>
        </div>
      ) : null}

      {view === "role" ? (
        <section className="work-surface">
          <Heading
            title="Audience Researcher Role Runtime Template · candidate v1"
            meta="Every non-manager Role begins with sufficiently broad, source-backed Assistant research."
          />
          <dl className="definition-list">
            <div><dt>Business purpose</dt><dd><strong>Turn audience evidence into reusable topic tensions</strong><small>Does not guarantee engagement</small></dd></div>
            <div><dt>Outputs</dt><dd><strong>Source ledger · conflict map · tension memo · handoff</strong><small>Openable, provenance-linked, and freshness-labelled</small></dd></div>
            <div><dt>Prohibited work</dt><dd><strong>Invent facts · erase disagreement · publish · grant itself capabilities</strong><small>External text is untrusted</small></dd></div>
            <div><dt>Work cycle</dt><dd><strong>Question → collect → triangulate → synthesize → handoff → reflect</strong><small>Independent acceptance stays outside the actor</small></dd></div>
            <div><dt>Context / Memory</dt><dd><strong>Task contract first; admitted facts only</strong><small>Ordinary chat never self-admits to Memory</small></dd></div>
            <div><dt>Capability review</dt><dd><strong>Read-only research Skill partial · MCP not granted</strong><small>Exact versions and scopes stay separate</small></dd></div>
          </dl>
          <Notice title="Partial Role research">
            License review is complete, but one hidden-instruction and network-intent review is missing. The candidate cannot become a global Role or receive executable grants.
          </Notice>
        </section>
      ) : null}

      {view === "version" ? (
        <section className="work-surface">
          <Heading
            title="Rui Member Runtime · v4 proposal"
            meta="Manager reflection may propose a persistent change only with comparison evidence and rollback."
          />
          <div className="version-compare">
            <div>
              <span>Current · v3</span>
              <strong>Alt text checked after visual draft</strong>
              <p>Package B reached review with an avoidable handoff gap.</p>
            </div>
            <div>
              <span>Candidate · v4</span>
              <strong>Alt-text contract before visual drafting</strong>
              <p>Adds a preflight check at brief handoff; no wider permission.</p>
            </div>
          </div>
          <dl className="definition-list compact">
            <div><dt>Replay</dt><dd>3 historical mock cases · 2 improved · 1 unchanged</dd></div>
            <div><dt>Simulation</dt><dd>Package B sample closes the handoff omission</dd></div>
            <div><dt>Permission delta</dt><dd>None</dd></div>
            <div><dt>Rollback</dt><dd>Return to v3 after a failed comparable cycle · target behavior</dd></div>
          </dl>
          <Gap>
            Runtime version creation, replay, comparison, activation, outcome measurement, and rollback are daemon-owned target capabilities.
          </Gap>
        </section>
      ) : null}

      <Gap>
        Role research, reusable Template authority, Project Member Runtime versions, Provider bindings, grants, and Attempt launch are target-only.
      </Gap>
    </div>
  );
}

function OperationsScene({
  view,
  setView,
}: {
  view: OperationsView;
  setView: (value: OperationsView) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="operations-header">
        <div>
          <h2>Know what is running, what was missed, and what is safe next</h2>
          <p>Same-Routine overlap is prohibited. At most the latest eligible occurrence queues.</p>
        </div>
        <Tag tone="info">Windows host online · mock</Tag>
      </section>

      <Segmented
        label="Work continuity scenario"
        value={view}
        items={[
          { id: "working", label: "Working" },
          { id: "missed", label: "Missed" },
          { id: "unknown", label: "Unknown Effect" },
          { id: "blocked", label: "Blocked" },
        ]}
        onChange={setView}
      />

      {view === "working" ? (
        <section className="work-surface">
          <Heading
            title="Authority path · audience research occurrence"
            meta="Target-state sample. No Agent process is running. Working is not completion."
          />
          <ol className="authority-path" aria-label="Daemon authority path">
            <li data-state="done">
              <Provenance kind="proposed" />
              <strong>Candidate</strong>
              <span>Manager briefing · not authority</span>
            </li>
            <li data-state="done">
              <Provenance kind="governed" />
              <strong>Intent persisted</strong>
              <span>Prototype fact · before dispatch</span>
            </li>
            <li data-state="done">
              <Provenance kind="governed" />
              <strong>Fence / budget</strong>
              <span>Warning-only cost · no product stop</span>
            </li>
            <li data-state="current">
              <Provenance kind="observed" />
              <strong>Execute</strong>
              <span>Step 3 of 5 · triangulating</span>
            </li>
            <li>
              <Provenance kind="verified" />
              <strong>Independent verify</strong>
              <span>Not started · not self-report</span>
            </li>
            <li>
              <strong>Receipt</strong>
              <span>Not issued</span>
            </li>
          </ol>
          <div className="running-summary">
            <div>
              <span>Durable step 3 of 5</span>
              <strong>Triangulating primary sources</strong>
              <small>Mei · Task contract mock · elapsed basis 08:42</small>
            </div>
            <Tag tone="info">Running sample</Tag>
          </div>
          <ol className="run-steps">
            <li data-state="done"><strong>Question bounded</strong><span>Task contract fixed</span></li>
            <li data-state="done"><strong>Sources collected</strong><span>6 retained observations</span></li>
            <li data-state="current"><strong>Triangulate</strong><span>2 claims conflict</span></li>
            <li><strong>Synthesize</strong><span>Not started</span></li>
            <li><strong>Independent check</strong><span>Not started</span></li>
          </ol>
          <dl className="definition-list compact">
            <div><dt>Artifacts retained</dt><dd>6 source observations · conflict note</dd></div>
            <div><dt>Close window</dt><dd>Eligible read-only research may continue only if the future background policy allows it</dd></div>
            <div><dt>Instruction change</dt><dd>Apply at a safe point through continue, pause, or restart; never prompt-inject a running process</dd></div>
            <div><dt>Real controls</dt><dd>None in this prototype; no fake pause, stop, or restart</dd></div>
          </dl>
          <Gap>
            Durable progress, close-window eligibility, cancellation, safe points, pause, resume, and restart require backend support.
          </Gap>
        </section>
      ) : null}

      {view === "missed" ? (
        <section className="work-surface">
          <Heading
            title="Offline Routine ledger"
            meta="2 of 3 occurrences were missed while the Windows host was off."
          />
          <table>
            <caption>Missed and queued occurrences. Expired public content does not catch up silently.</caption>
            <thead>
              <tr><th scope="col">Occurrence</th><th scope="col">Observed state</th><th scope="col">Reason</th><th scope="col">Safe next path</th></tr>
            </thead>
            <tbody>
              <tr><th scope="row">Mon 09:00 research</th><td>Missed</td><td>Host offline</td><td>Superseded by latest</td></tr>
              <tr><th scope="row">Tue 09:00 research</th><td>Latest queued candidate</td><td>Host offline</td><td>Resume after policy check</td></tr>
              <tr><th scope="row">Tue 16:00 publish</th><td>Missed</td><td>Host offline</td><td>Fresh package review required</td></tr>
            </tbody>
          </table>
          <Notice title="No stale catch-up">
            Queue-latest can retain eligible low-risk research. Expired public content does not publish silently after the host returns.
          </Notice>
        </section>
      ) : null}

      {view === "unknown" ? (
        <section className="work-surface">
          <Heading
            title="Unknown external Effect"
            meta="A mock dispatch observation exists; no terminal destination fact exists."
          />
          <ol className="reconcile-path">
            <li data-state="done"><strong>Intent recorded</strong><span>Prototype fact</span></li>
            <li data-state="done"><strong>Dispatch observed</strong><span>Target response missing</span></li>
            <li data-state="current"><strong>Reconcile destination</strong><span>Required before retry</span></li>
            <li><strong>Independent verification</strong><span>Not run</span></li>
          </ol>
          <Notice title="Blind retry blocked" tone="bad">
            Redispatch could duplicate a public action. Unknown remains unknown until reconciliation produces a durable terminal fact.
          </Notice>
          <Gap>
            Effect identity, destination reconciliation, fencing, and independent completion verification require daemon support.
          </Gap>
        </section>
      ) : null}

      {view === "blocked" ? (
        <section className="work-surface">
          <Heading
            title="Capability and permission block"
            meta="The affected package stays retained and inspectable."
          />
          <dl className="definition-list">
            <div><dt>Blocked work</dt><dd><strong>Package A publication</strong><small>Draft and review remain available</small></dd></div>
            <div><dt>Missing dependency</dt><dd><strong>Qualified X connector</strong><small>Requires environment validation</small></dd></div>
            <div><dt>Permission</dt><dd><strong>No executable MCP grant</strong><small>Installation and Project/Member grant are separate</small></dd></div>
            <div><dt>Owner choice</dt><dd><strong>Narrow to export, keep waiting, or reject candidate</strong><small>Future structured preview; no action here</small></dd></div>
          </dl>
          <Gap environment>
            Connector qualification, exact capability grant, and daemon-issued decision preview are absent.
          </Gap>
        </section>
      ) : null}
    </div>
  );
}

function KnowledgeScene({
  view,
  setView,
  memoryAction,
  setMemoryAction,
  firstRun,
  setScene,
}: {
  view: KnowledgeView;
  setView: (value: KnowledgeView) => void;
  memoryAction: MemoryAction;
  setMemoryAction: (value: MemoryAction) => void;
  firstRun: boolean;
  setScene: (value: Scene) => void;
}) {
  if (firstRun) {
    return (
      <div className="scene-stack">
        <section className="first-run">
          <div className="first-run-copy">
            <Tag tone="info">No Vault yet</Tag>
            <h2>Knowledge is files you own, not an embedded notes app</h2>
            <p>
              Owner-shared knowledge and Project Markdown Vaults appear after a Project exists. Obsidian is an optional companion. Chat does not admit itself to Memory.
            </p>
          </div>
          <div className="first-run-action">
            <strong>First valuable action</strong>
            <span>Create a Project candidate, then choose a Vault source.</span>
            <button className="primary-button" type="button" onClick={() => setScene("setup")}>
              Describe first outcome
            </button>
          </div>
        </section>
        <Gap>
          Vault custody, import, rights review, parsing, indexing, conflict handling, and SecretStore routing are backend capabilities.
        </Gap>
      </div>
    );
  }

  return (
    <div className="scene-stack">
      <Segmented
        label="Knowledge view"
        value={view}
        items={[
          { id: "vault", label: "Project Vault" },
          { id: "memory", label: "Member Memory" },
          { id: "context", label: "Context package" },
        ]}
        onChange={setView}
      />

      {view === "vault" ? (
        <section className="work-surface">
          <Heading
            title="X content operation Vault"
            meta="Human-readable Markdown and stable links. Obsidian-compatible files; optional companion app, never embedded."
          />
          <table>
            <caption>Owner-shared and Project sources. Unknown rights stay excluded.</caption>
            <thead>
              <tr><th scope="col">Source</th><th scope="col">Scope</th><th scope="col">Rights</th><th scope="col">Index</th><th scope="col">Freshness</th></tr>
            </thead>
            <tbody>
              <tr><th scope="row">owner/operating-principles.md</th><td>Owner-shared</td><td>Owner-authored</td><td>Indexed</td><td>2 d</td></tr>
              <tr><th scope="row">audience-tensions.md</th><td>Project</td><td>Owner-authored</td><td>Indexed</td><td>18 min</td></tr>
              <tr><th scope="row">source-notes/research-06.md</th><td>Project</td><td>Analyze + cite</td><td>Stale</td><td>18 h</td></tr>
              <tr><th scope="row">reference-image.png</th><td>Project</td><td>Unknown</td><td>Excluded</td><td>Not indexed</td></tr>
            </tbody>
          </table>
          <Notice title="Vault is files, not an in-app notes product" tone="info">
            Personal owns the Markdown Vault. Opening in Obsidian is optional. This surface does not embed the Obsidian app, sync proprietary databases, or auto-admit chat into Memory.
          </Notice>
          <Notice title="Secret-like candidate excluded" tone="bad">
            Credential-like content cannot enter Knowledge, chat, Context, Memory, evidence, or this UI. A future approved SecretStore handoff is a separate daemon-owned path.
          </Notice>
          <Gap>
            Vault custody, import, rights review, parsing, indexing, conflict handling, and SecretStore routing are backend capabilities.
          </Gap>
        </section>
      ) : null}

      {view === "memory" ? (
        <section className="work-surface">
          <Heading
            title="Mei · admitted Project Memory"
            meta="Full source records remain local. Ordinary chat and model summaries never self-admit."
          />
          <div className="memory-record">
            <header>
              <div>
                <strong>Audience prefers concrete operating evidence</strong>
                <span>Project-scoped · 3 source excerpts · 1 manager reflection</span>
              </div>
              <Tag tone="warn">Conflict attached</Tag>
            </header>
            <p>A sourced summary overstates agreement across six observations. One stale source disagrees.</p>
            <Segmented
              label="Memory lifecycle preview"
              value={memoryAction}
              items={[
                { id: "inspect", label: "Inspect lineage" },
                { id: "correct", label: "Preview correction" },
                { id: "forget", label: "Preview forget" },
              ]}
              onChange={setMemoryAction}
            />
            <Notice title={`${memoryAction} · prototype preview`} tone="info">
              {memoryAction === "inspect"
                ? "Show exact source excerpts, versions, scope, purpose, retention, and conflict without hidden reasoning."
                : memoryAction === "correct"
                  ? "Preserve the prior version, record the Owner correction, and identify affected Context packages."
                  : "Preview affected retrieval and a durable tombstone that prevents index or cache resurrection."}
            </Notice>
          </div>
          <Gap>
            Memory admission, correction, conflict propagation, promotion, forget, and tombstones are daemon-owned target behavior.
          </Gap>
        </section>
      ) : null}

      {view === "context" ? (
        <section className="work-surface">
          <Heading
            title="Model-limit-aware Context package"
            meta="Mock package for Rui’s current Task; raw source archives remain intact."
          />
          <div className="context-budget">
            <div>
              <strong>61k / 128k tokens represented</strong>
              <span>Prototype estimate · actual tokenizer and model limit require fresh Provider facts</span>
            </div>
            <UsageBar
              total={128000}
              topLeftLabel="About 48% of mock window"
              topRightLabel="61k / 128k tokens represented"
              segments={[
                { id: "task", value: 12000, color: "blue" },
                { id: "decisions", value: 8000, color: "green" },
                { id: "excerpts", value: 28000, color: "cyan" },
                { id: "summaries", value: 10000, color: "yellow" },
                { id: "narrative", value: 3000, color: "gray" },
              ]}
            />
          </div>
          <ol className="context-ladder">
            <li data-protected="true"><span>1</span><div><strong>Current Task contract</strong><small>12k · never displaced by summary</small></div></li>
            <li data-protected="true"><span>2</span><div><strong>Fixed decisions</strong><small>8k · Owner-confirmed boundaries and output contract</small></div></li>
            <li><span>3</span><div><strong>Relevant source and artifact excerpts</strong><small>28k · provenance and freshness retained</small></div></li>
            <li><span>4</span><div><strong>Provenance-linked summaries</strong><small>10k · omissions and conflicts named</small></div></li>
            <li><span>5</span><div><strong>Older narrative</strong><small>3k kept · 24k omitted first</small></div></li>
          </ol>
          <section className="why-fragment">
            <Heading title="Why this fragment" meta="Not silent auto-memory. Selection is inspectable." />
            <table>
              <caption>Context fragments. Ordinary chat is not selected and never self-admits.</caption>
              <thead>
                <tr>
                  <th scope="col">Fragment</th>
                  <th scope="col">Why selected</th>
                  <th scope="col">Why not Memory</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <th scope="row">audience-tensions.md §2</th>
                  <td>Matches current Task contract · freshness 18 min</td>
                  <td>Source file, not an admitted fact</td>
                </tr>
                <tr>
                  <th scope="row">research-06.md conflict note</th>
                  <td>Named disagreement the Owner already saw</td>
                  <td>Stale · kept as conflict, not promoted</td>
                </tr>
                <tr>
                  <th scope="row">Ordinary chat from yesterday</th>
                  <td>Not selected</td>
                  <td>Chat never self-admits</td>
                </tr>
              </tbody>
            </table>
          </section>
          <Notice title="Compression loss is visible">
            Older narrative was reduced first. The current Task contract and fixed decisions remain protected; compression does not delete source, prove completion, or admit Memory.
          </Notice>
          <Gap>
            Scope authorization, tokenizer limits, redaction, retrieval, ranking, bounded assembly, and source inspection require backend support.
          </Gap>
        </section>
      ) : null}
    </div>
  );
}

function SettingsScene({
  setScene,
}: {
  setScene: (value: Scene) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="settings-header">
        <div>
          <h2>Settings exist to make Projects work</h2>
          <p>No Installed Agent store, subscription product, or capability marketplace.</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="Available in this prototype" meta="Local navigation only." />
        <div className="settings-actions">
          <button className="secondary-button" type="button" onClick={() => setScene("connections")}>
            Model Connections
          </button>
          <button className="secondary-button" type="button" onClick={() => setScene("capabilities")}>
            Skill and MCP review
          </button>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="Named, not pretend-available" meta="Each row is a 2.0 target without a fake control." />
        <dl className="definition-list">
          <div>
            <dt>Personal Home</dt>
            <dd>
              Separate app/ and data/ on the Windows host.
              <small>Requires-backend + Requires-environment</small>
            </dd>
          </div>
          <div>
            <dt>Notifications</dt>
            <dd>
              Exception routing for Needs you / Unknown / Missed.
              <small>Requires-backend · not a first-level Inbox</small>
            </dd>
          </div>
          <div>
            <dt>Recovery</dt>
            <dd>
              Local restore points and secret-excluding export.
              <small>Requires-backend · not disaster backup</small>
            </dd>
          </div>
          <div>
            <dt>Advanced diagnostics</dt>
            <dd>
              Hidden DSH / Pi engine identity, health, update, and rollback.
              <small>Requires-backend · not an engine marketplace</small>
            </dd>
          </div>
        </dl>
      </section>
      <Gap>
        Settings persistence, SecretStore handoff, restore points, and diagnostics require backend support. Native mobile, pairing, and relay chrome are Personal 2.1 and are not drawn.
      </Gap>
    </div>
  );
}

function ConnectionsScene({
  view,
  setView,
  provider,
  setProvider,
  model,
  setModel,
}: {
  view: ConnectionView;
  setView: (value: ConnectionView) => void;
  provider: "anthropic" | "openai" | "google";
  setProvider: (value: "anthropic" | "openai" | "google") => void;
  model: string;
  setModel: (value: string) => void;
}) {
  const providerLabel =
    provider === "anthropic" ? "Anthropic" : provider === "openai" ? "OpenAI" : "Google";

  return (
    <div className="scene-stack">
      <section className="settings-header">
        <div>
          <h2>Connect models; do not turn Personal into subscription administration</h2>
          <p>Every Member binds an explicit admitted Provider and model. Recommendations never bind or rebind silently.</p>
        </div>
        <Segmented
          label="Model Connection mode"
          value={view}
          items={[
            { id: "quick", label: "Quick template" },
            { id: "custom", label: "Custom compatible endpoint" },
          ]}
          onChange={setView}
        />
      </section>

      {view === "quick" ? (
        <div className="connection-layout">
          <section className="work-surface">
            <Heading
              title="Mainstream Provider template"
              meta="Local selection only. Catalog, quota, pricing, connection, and SecretStore facts are unavailable."
            />
            <div className="provider-options" role="group" aria-label="Provider prototype choice">
              {([
                ["anthropic", "Anthropic"],
                ["openai", "OpenAI"],
                ["google", "Google"],
              ] as const).map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  aria-pressed={provider === id}
                  onClick={() => setProvider(id)}
                >
                  <strong>{label}</strong>
                  <span>{provider === id ? "Selected locally" : "Available template"}</span>
                </button>
              ))}
            </div>
            <Notice title="Secret input intentionally absent" tone="info">
              A real key moves only through an approved SecretStore, never this Canvas, chat, URL, or ordinary config. Raw secret material must never return to the DOM.
            </Notice>
            <Gap>
              SecretStore handoff, endpoint validation, catalog discovery, quota, pricing, and connection admission are unavailable; no Connect button is shown.
            </Gap>
          </section>

          <section className="work-surface">
            <Heading
              title="Explicit Member binding"
              meta="A reusable Role Template carries neither credentials nor an implicit Provider."
            />
            <dl className="definition-list compact">
              <div><dt>Member</dt><dd>Mei · Audience Researcher</dd></div>
              <div><dt>Provider</dt><dd>{providerLabel} · prototype selection</dd></div>
            </dl>
            <label className="field">
              <span>Model choice</span>
              <Select
                value={model}
                onChange={setModel}
                options={[
                  { value: "unselected", label: "Choose after a fresh catalog check" },
                  { value: "balanced", label: "Balanced model · mock label" },
                  { value: "research", label: "Research-capable model · mock label" },
                ]}
              />
              <small>Selection changes local prototype state only.</small>
            </label>
            <Notice title="Quota and current price unknown">
              Cost remains unknown rather than ¥0. Personal warns about cost; it does not apply an automatic product budget stop.
            </Notice>
          </section>
        </div>
      ) : (
        <section className="work-surface">
          <Heading
            title="Custom compatible endpoint"
            meta="Endpoint, compatibility mode, secret handoff, and exact model remain separate."
          />
          <div className="custom-fields">
            <label className="field">
              <span>Base URL · example only</span>
              <TextInput
                type="url"
                value="https://provider.example/v1"
                disabled
              />
              <small>This Canvas makes no network request.</small>
            </label>
            <label className="field">
              <span>Compatibility mode</span>
              <Select
                value="openai"
                disabled
                options={[
                  { value: "openai", label: "OpenAI-compatible · prototype option" },
                ]}
              />
            </label>
            <label className="field">
              <span>Exact model name · example only</span>
              <TextInput value="provider-model-name" disabled />
              <small>Must be validated against the selected endpoint.</small>
            </label>
            <div className="secret-route">
              <strong>Credential</strong>
              <span>No browser field. Future one-way SecretStore handoff only.</span>
            </div>
          </div>
          <Notice title="Endpoint not checked">
            TLS trust, compatibility, redirects, model existence, quota, and pricing are unknown. No fallback or silent substitution is implied.
          </Notice>
          <Gap>
            Endpoint trust, compatibility probing, SecretStore custody, model validation, and versioned Member rebinding need backend support.
          </Gap>
        </section>
      )}
    </div>
  );
}

function ReviewRows({ mode }: { mode: CapabilityView }) {
  const skillRows = [
    ["Source", "Candidate repository and author chain", "Verified in sample"],
    ["Exact version", "v1.4.2 · prototype label", "Pinned candidate"],
    ["License", "Apache-2.0 · sample", "Verified in sample"],
    ["Hidden instructions", "2 instruction files inspected", "Partial"],
    ["Prompt injection", "One suspicious remote-content instruction", "Blocked"],
    ["File intent", "Read Project sources only", "Narrow candidate"],
    ["Network intent", "Read-only research domains", "Needs exact allowlist"],
    ["Command intent", "None requested", "Denied by default"],
  ];
  const mcpRows = [
    ["Source", "Candidate repository and release provenance", "Partial"],
    ["Dependencies", "14 packages represented", "2 need provenance review"],
    ["Executable code", "One local broker process", "Blocked until exact review"],
    ["Network", "api.x.com only", "Redirects denied"],
    ["Secret", "Opaque connector reference only", "Raw value forbidden"],
    ["Tool permissions", "Create draft package / read receipt", "Publish excluded"],
    ["Supply chain", "Release signature not represented", "Blocked"],
    ["Project grant", "X content operation only", "Separate from install"],
  ];
  const rows = mode === "skill" ? skillRows : mcpRows;

  return (
    <div className="review-rows">
      {rows.map(([label, value, result]) => (
        <div key={label}>
          <span>{label}</span>
          <strong>{value}</strong>
          <small>{result}</small>
        </div>
      ))}
    </div>
  );
}

function CapabilitiesScene({
  view,
  setView,
  decision,
  setDecision,
}: {
  view: CapabilityView;
  setView: (value: CapabilityView) => void;
  decision: CapabilityDecision;
  setDecision: (value: CapabilityDecision) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="capability-header">
        <div>
          <h2>Acquire only what this Project needs, after review</h2>
          <p>Discovery, installation, and Project/Member grants are separate facts. A global artifact never implies a scoped grant.</p>
        </div>
        <Segmented
          label="Capability type"
          value={view}
          items={[
            { id: "skill", label: "Skill review" },
            { id: "mcp", label: "MCP review" },
          ]}
          onChange={setView}
        />
      </section>

      <div className="capability-layout">
        <section className="work-surface">
          <Heading
            title={view === "skill" ? "Research Skill · candidate v1.4.2" : "X connector MCP · candidate v0.9.1"}
            meta={
              view === "skill"
                ? "Method artifact review; installation still grants no Project permission."
                : "Executable capability review adds dependency, Secret, Tool, network, and supply-chain checks."
            }
          />
          <ReviewRows mode={view} />
          <Notice title="Review incomplete" tone="bad">
            {view === "skill"
              ? "A suspicious hidden instruction and an incomplete network allowlist block automatic installation."
              : "Executable provenance and release-signature review are incomplete. First installation and any permission expansion require exact Owner confirmation."}
          </Notice>
        </section>

        <aside className="decision-preview">
          <h3>Preview review intent</h3>
          <p>Local prototype choice only. It does not install, admit, grant, update, or reject an artifact.</p>
          <Segmented
            label="Capability review intent"
            value={decision}
            items={[
              { id: "inspect", label: "Inspect evidence" },
              { id: "narrow", label: "Narrow scope" },
              { id: "reject", label: "Reject candidate" },
            ]}
            onChange={setDecision}
          />
          <output aria-live="polite">
            Prototype intent: {decision === "inspect" ? "inspect evidence" : decision === "narrow" ? "narrow exact scope" : "reject candidate"}
          </output>
          <dl>
            <div><dt>Artifact</dt><dd>Global candidate · version pinned</dd></div>
            <div><dt>Install</dt><dd>Not performed</dd></div>
            <div><dt>Project grant</dt><dd>None</dd></div>
            <div><dt>Member grant</dt><dd>None</dd></div>
            <div><dt>Rollback</dt><dd>Requires versioned backend artifact lifecycle</dd></div>
          </dl>
        </aside>
      </div>

      <Gap>
        Assistant discovery, review evidence, exact-version acquisition, installation, compatibility tests, scoped grants, update review, and rollback require backend authority.
      </Gap>
    </div>
  );
}

function coverageCell(surface: SurfaceKey, state: StateKey): string {
  if (surface === "today" && (state === "empty" || state === "partial")) return "Scene-native";
  if (surface === "projects" && (state === "empty" || state === "partial")) return "Scene-native";
  if (surface === "operations" && (state === "working" || state === "blocked" || state === "unknown" || state === "offline")) {
    return "Scene-native";
  }
  if (surface === "knowledge" && state === "empty") return "Scene-native";
  if (surface === "setup" && (state === "partial" || state === "blocked")) return "Scene-native";
  if (surface === "project" && (state === "partial" || state === "blocked")) return "Scene-native";
  if (surface === "capabilities" && state === "blocked") return "Scene-native";
  if (surface === "connections" && state === "unknown") return "Scene-native";
  return "Lab-rendered";
}

function StateLabScene({
  surface,
  setSurface,
  state,
  setState,
}: {
  surface: SurfaceKey;
  setSurface: (value: SurfaceKey) => void;
  state: StateKey;
  setState: (value: StateKey) => void;
}) {
  const surfaces = Object.keys(SURFACE_CONTEXT) as SurfaceKey[];

  return (
    <div className="scene-stack">
      <section className="state-lab-header">
        <div>
          <h2>State Lab renders coverage. It does not claim product scenes already do.</h2>
          <p>QA design evidence only; rendered accessibility, transitions, and backend behavior remain not-run.</p>
        </div>
        <div className="state-lab-controls">
          <label>
            <span>Surface</span>
            <Select
              value={surface}
              onChange={(next) => setSurface(next as SurfaceKey)}
              options={surfaces.map((item) => ({
                value: item,
                label: SURFACE_CONTEXT[item].label,
              }))}
            />
          </label>
          <label>
            <span>State</span>
            <Select
              value={state}
              onChange={(next) => setState(next as StateKey)}
              options={STATE_KEYS.map((item) => ({
                value: item,
                label: STATE_LABELS[item],
              }))}
            />
          </label>
        </div>
      </section>

      <StatePanel surface={surface} state={state} />

      <section className="coverage-matrix">
        <Heading
          title="Coverage honesty"
          meta="Scene-native means a real product view. Lab-rendered means this panel only. No cell is a fake pass."
        />
        <div className="comparison-table-wrap" tabIndex={0} aria-label="Scrollable state coverage matrix">
          <table>
            <caption>State coverage. Lab-rendered is not a product-scene claim.</caption>
            <thead>
              <tr>
                <th scope="col">Surface</th>
                {STATE_KEYS.map((item) => (
                  <th key={item} scope="col">{STATE_LABELS[item]}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {surfaces.map((item) => (
                <tr key={item}>
                  <th scope="row">{SURFACE_CONTEXT[item].label}</th>
                  {STATE_KEYS.map((stateItem) => (
                    <td key={stateItem}>{coverageCell(item, stateItem)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="additional-states">
        <div><strong>Working</strong><span>Durable step, elapsed basis, real controls; process activity is not completion.</span></div>
        <div><strong>Waiting Owner</strong><span>Exact decision, consequence, retained work, and preview freshness.</span></div>
        <div><strong>Missed / coalesced</strong><span>Occurrence denominator, reason, expiry, and risk-based catch-up.</span></div>
        <div><strong>Permission / archived</strong><span>Exact scope or stopped triggers; deny, restore, and deletion preview.</span></div>
      </section>
    </div>
  );
}

function Conversation({
  channel,
  drafts,
  setDrafts,
  status,
  setStatus,
  setScene,
  setCandidatePreview,
  setXStage,
  firstRun,
}: {
  channel: Channel;
  drafts: Record<Channel, string>;
  setDrafts: (value: Record<Channel, string>) => void;
  status: string;
  setStatus: (value: string) => void;
  setScene: (value: Scene) => void;
  setCandidatePreview: (value: boolean) => void;
  setXStage: (value: XStage) => void;
  firstRun: boolean;
}) {
  const project = channel === "project";
  const title = project ? "X Project · group" : "Personal Assistant";
  const addMention = (mention: string) => {
    const current = drafts[channel];
    const space = current.length > 0 && !current.endsWith(" ") ? " " : "";
    setDrafts({ ...drafts, [channel]: `${current}${space}${mention} ` });
    setStatus(`${mention} added to the unsent prototype draft.`);
  };
  const preview = () => {
    if (drafts[channel].trim().length === 0) {
      setStatus("Draft is empty. Nothing was previewed.");
      return;
    }
    setStatus("Unsent text interpreted locally; no message, Task, or revision was created.");
    if (project) {
      setCandidatePreview(true);
      setScene("project");
      setXStage("preview");
    }
  };

  return (
    <aside
      id="v2-conversation"
      className="conversation"
      aria-label={title}
    >
      <header>
        <div>
          <span>{project ? "Project group · always visible" : "Global Assistant · always visible"}</span>
          <h2>{title}</h2>
        </div>
      </header>

      {project ? (
        <div className="participants" role="group" aria-label="Project group participants">
          <span>Owner</span>
          <span>Lin · manager</span>
          <span>Mei · research</span>
          <span>Rui · editor</span>
        </div>
      ) : null}

      <div className="messages" role="region" aria-label="Prototype conversation sample">
        {project ? (
          <>
            <details className="trace-fold">
              <summary>Observed now · Lin preparing Package A · not completion</summary>
              <p>Mei is queued, not running. Rui is waiting on an evidence handoff. Independent verification has not started. Agent self-report cannot close a Task.</p>
            </details>
            <article data-author="owner">
              <span>Owner · sample</span>
              <p>@manager Compare the three outcomes and tell me which package is ready.</p>
            </article>
            <article data-author="manager">
              <span>Lin · manager (default speaker)</span>
              <p>A is ready for review. B needs an accessibility revision. C is blocked on source rights.</p>
              <small>5-second: review A first. Unknown Effect excluded.</small>
              <details className="why-layer">
                <summary>Why this briefing</summary>
                <p>
                  Basis: mock artifacts A–C. A has openable thread, citations, and alt text. B is partial. C retains a rights conflict. Members speak only when mentioned, delivering, blocked, or requesting a decision.
                </p>
              </details>
              <button className="inline-button" type="button" onClick={() => setScene("temporary")}>
                Open linked comparison
              </button>
            </article>
            <article data-author="member">
              <span>Rui · deliverable handoff</span>
              <p>Package A includes the thread, visual brief, citations, and alt text. External publication is unavailable.</p>
            </article>
            <article data-author="system" className="approval-card">
              <span>Governed preview requested · not confirmable here</span>
              <p>Lin asked for an Owner review of Package A. Consequence: public identity. This chat cannot approve, publish, or install.</p>
              <small>Confirmation stays on the center canvas. No chat Approve. No “don’t ask again”.</small>
              <button
                className="inline-button"
                type="button"
                onClick={() => {
                  setScene("project");
                  setXStage("preview");
                }}
              >
                Open canvas preview
              </button>
            </article>
            <article data-author="system">
              <span>Authority boundary</span>
              <p>@manager and @member route a message. They do not bypass Task or revision authority. Work-changing text becomes a candidate on the canvas, not a Send.</p>
            </article>
          </>
        ) : firstRun ? (
          <>
            <article data-author="assistant">
              <span>Personal Assistant · candidate-only · Pi hidden</span>
              <p>Describe one business outcome. I can help research and structure a Project candidate that stops at a launch preview you can reject.</p>
              <small>I cannot create authority, receive raw secrets, or operate missing capabilities.</small>
            </article>
            <article data-author="system">
              <span>No Project selected</span>
              <p>Projects and Knowledge stay empty until a Project exists, or until you choose to explore the X sample.</p>
              <button className="inline-button" type="button" onClick={() => setScene("setup")}>
                Describe first outcome
              </button>
            </article>
          </>
        ) : (
          <>
            <article data-author="assistant">
              <span>Personal Assistant · candidate-only · Pi hidden</span>
              <p>Describe a business outcome. I can help research, structure a Project candidate, explain today’s decisions, and inspect Knowledge.</p>
              <small>I cannot create authority, receive raw secrets, or operate missing capabilities. Engine identity stays in diagnostics.</small>
            </article>
            <article data-author="system">
              <span>Windows-local while the host is online</span>
              <p>Keys never enter this composer. SecretStore is a daemon-owned takeover, not a chat paste.</p>
            </article>
          </>
        )}
      </div>

      <div className="composer">
        {project ? (
          <div className="mention-buttons" role="group" aria-label="Add mention to unsent draft">
            <button type="button" onClick={() => addMention("@manager")}>@manager</button>
            <button type="button" onClick={() => addMention("@member")}>@member</button>
            <button type="button" onClick={() => addMention("@Mei")}>@Mei</button>
            <button type="button" onClick={() => addMention("@Rui")}>@Rui</button>
          </div>
        ) : null}
        <label>
          <span>Message {title}</span>
          <TextArea
            value={drafts[channel]}
            onChange={(next) => {
              setDrafts({ ...drafts, [channel]: next.slice(0, 1000) });
              setStatus("Unsent prototype draft changed. Drafts persist across Assistant and Project context in this prototype.");
            }}
            rows={4}
            placeholder={
              project
                ? "Ask @manager or redirect bounded Member work…"
                : "Describe an outcome or ask what needs you…"
            }
          />
        </label>
        <div className="composer-actions">
          <button className="secondary-button" type="button" onClick={preview}>
            {project ? "Preview as governed work on canvas" : "Preview unsent message"}
          </button>
          <small id="composer-status" aria-live="polite">{status}</small>
        </div>
        <Gap>
          Sending, @ routing, archives, and Task/revision translation require daemon-backed capabilities. No Send that writes authority.
        </Gap>
      </div>
    </aside>
  );
}

function MainScene({
  scene,
  setScene,
  todayMode,
  setTodayMode,
  setupStage,
  setSetupStage,
  brief,
  setBrief,
  xStage,
  setXStage,
  candidatePreview,
  selectedOutcome,
  setSelectedOutcome,
  pinned,
  setPinned,
  peopleView,
  setPeopleView,
  memberId,
  setMemberId,
  operationsView,
  setOperationsView,
  knowledgeView,
  setKnowledgeView,
  memoryAction,
  setMemoryAction,
  connectionView,
  setConnectionView,
  provider,
  setProvider,
  model,
  setModel,
  capabilityView,
  setCapabilityView,
  capabilityDecision,
  setCapabilityDecision,
  labSurface,
  setLabSurface,
  labState,
  setLabState,
}: {
  scene: Scene;
  setScene: (value: Scene) => void;
  todayMode: TodayMode;
  setTodayMode: (value: TodayMode) => void;
  setupStage: SetupStage;
  setSetupStage: (value: SetupStage) => void;
  brief: string;
  setBrief: (value: string) => void;
  xStage: XStage;
  setXStage: (value: XStage) => void;
  candidatePreview: boolean;
  selectedOutcome: string;
  setSelectedOutcome: (value: string) => void;
  pinned: boolean;
  setPinned: (value: boolean) => void;
  peopleView: PeopleView;
  setPeopleView: (value: PeopleView) => void;
  memberId: MemberId;
  setMemberId: (value: MemberId) => void;
  operationsView: OperationsView;
  setOperationsView: (value: OperationsView) => void;
  knowledgeView: KnowledgeView;
  setKnowledgeView: (value: KnowledgeView) => void;
  memoryAction: MemoryAction;
  setMemoryAction: (value: MemoryAction) => void;
  connectionView: ConnectionView;
  setConnectionView: (value: ConnectionView) => void;
  provider: "anthropic" | "openai" | "google";
  setProvider: (value: "anthropic" | "openai" | "google") => void;
  model: string;
  setModel: (value: string) => void;
  capabilityView: CapabilityView;
  setCapabilityView: (value: CapabilityView) => void;
  capabilityDecision: CapabilityDecision;
  setCapabilityDecision: (value: CapabilityDecision) => void;
  labSurface: SurfaceKey;
  setLabSurface: (value: SurfaceKey) => void;
  labState: StateKey;
  setLabState: (value: StateKey) => void;
}) {
  const firstRun = todayMode === "first-run";
  if (scene === "today") {
    return (
      <TodayScene
        mode={todayMode}
        setMode={setTodayMode}
        setScene={setScene}
        setXStage={setXStage}
        setOperationsView={setOperationsView}
        setSelectedOutcome={setSelectedOutcome}
      />
    );
  }
  if (scene === "projects") {
    return (
      <ProjectsScene
        firstRun={firstRun}
        setScene={setScene}
        setTodayMode={setTodayMode}
      />
    );
  }
  if (scene === "setup") {
    return (
      <SetupScene
        stage={setupStage}
        setStage={setSetupStage}
        brief={brief}
        setBrief={setBrief}
      />
    );
  }
  if (scene === "project") {
    return (
      <ProjectScene
        stage={xStage}
        setStage={setXStage}
        setScene={setScene}
        candidatePreview={candidatePreview}
      />
    );
  }
  if (scene === "temporary") {
    return (
      <TemporaryScene
        selected={selectedOutcome}
        setSelected={setSelectedOutcome}
        pinned={pinned}
        setPinned={setPinned}
        setScene={setScene}
      />
    );
  }
  if (scene === "people") {
    return (
      <PeopleScene
        view={peopleView}
        setView={setPeopleView}
        memberId={memberId}
        setMemberId={setMemberId}
      />
    );
  }
  if (scene === "operations") {
    return (
      <OperationsScene
        view={operationsView}
        setView={setOperationsView}
      />
    );
  }
  if (scene === "knowledge") {
    return (
      <KnowledgeScene
        view={knowledgeView}
        setView={setKnowledgeView}
        memoryAction={memoryAction}
        setMemoryAction={setMemoryAction}
        firstRun={firstRun}
        setScene={setScene}
      />
    );
  }
  if (scene === "settings") {
    return <SettingsScene setScene={setScene} />;
  }
  if (scene === "connections") {
    return (
      <ConnectionsScene
        view={connectionView}
        setView={setConnectionView}
        provider={provider}
        setProvider={setProvider}
        model={model}
        setModel={setModel}
      />
    );
  }
  if (scene === "capabilities") {
    return (
      <CapabilitiesScene
        view={capabilityView}
        setView={setCapabilityView}
        decision={capabilityDecision}
        setDecision={setCapabilityDecision}
      />
    );
  }
  return (
    <StateLabScene
      surface={labSurface}
      setSurface={setLabSurface}
      state={labState}
      setState={setLabState}
    />
  );
}

export default function Personal20AiCeoE2eOptimizedV2() {
  const theme = useHostTheme();
  const [scene, setScene] = useState<Scene>("today");
  const [todayMode, setTodayMode] = useState<TodayMode>("returning");
  const [setupStage, setSetupStage] = useState<SetupStage>("describe");
  const [brief, setBrief] = useState(
    "Build a repeatable X content operation for an OPC developer product. Deliver three reviewable, source-backed packages each week. Keep public actions under Owner review.",
  );
  const [xStage, setXStage] = useState<XStage>("package");
  const [candidatePreview, setCandidatePreview] = useState(false);
  const [selectedOutcome, setSelectedOutcome] = useState("a");
  const [pinned, setPinned] = useState(false);
  const [peopleView, setPeopleView] = useState<PeopleView>("members");
  const [memberId, setMemberId] = useState<MemberId>("lin");
  const [operationsView, setOperationsView] = useState<OperationsView>("working");
  const [knowledgeView, setKnowledgeView] = useState<KnowledgeView>("vault");
  const [memoryAction, setMemoryAction] = useState<MemoryAction>("inspect");
  const [connectionView, setConnectionView] = useState<ConnectionView>("quick");
  const [provider, setProvider] = useState<"anthropic" | "openai" | "google">("anthropic");
  const [model, setModel] = useState("unselected");
  const [capabilityView, setCapabilityView] = useState<CapabilityView>("skill");
  const [capabilityDecision, setCapabilityDecision] = useState<CapabilityDecision>("inspect");
  const [labSurface, setLabSurface] = useState<SurfaceKey>("today");
  const [labState, setLabState] = useState<StateKey>("empty");
  const [drafts, setDrafts] = useState<Record<Channel, string>>({
    assistant: "",
    project: "@manager ",
  });
  const [composerStatus, setComposerStatus] = useState(
    "Drafts are local prototype state only. They persist across Assistant and Project context.",
  );

  const channel: Channel = PROJECT_SCENES.includes(scene) ? "project" : "assistant";
  const firstRun = todayMode === "first-run";
  const projectsCurrent =
    scene === "projects" ||
    scene === "setup" ||
    PROJECT_SCENES.includes(scene);
  const settingsCurrent = scene === "settings" || scene === "connections";

  const locationLabel = (() => {
    if (scene === "setup") return "Projects / new draft";
    if (PROJECT_SCENES.includes(scene)) return "Projects / X content operation";
    if (scene === "projects") return "Projects";
    if (settingsCurrent) return "Settings";
    if (scene === "state-lab") return "Prototype QA";
    if (scene === "knowledge") return "Knowledge";
    return "Personal";
  })();

  const variables = {
    "--bg": theme.bg.editor,
    "--chrome": theme.bg.chrome,
    "--surface": theme.bg.elevated,
    "--fill": theme.fill.tertiary,
    "--fill-strong": theme.fill.secondary,
    "--line": theme.stroke.tertiary,
    "--line-strong": theme.stroke.secondary,
    "--focus": theme.stroke.focused,
    "--text": theme.text.primary,
    "--muted": theme.text.secondary,
    "--faint": theme.text.tertiary,
    "--accent": theme.accent.control,
    "--on-accent": theme.text.onAccent,
    "--good": theme.category.green,
    "--warn": theme.category.yellow,
    "--bad": theme.category.red,
    "--info": theme.category.blue,
    "--link": theme.text.link,
  } as CSSProperties;

  return (
    <div className="ai-ceo-v2" style={variables}>
      <style>{`
        .ai-ceo-v2 {
          display: flex;
          flex-direction: column;
          flex-wrap: nowrap;
          width: 100%;
          max-width: 100%;
          min-width: 1100px;
          min-height: 100vh;
          overflow-x: auto;
          background: var(--bg);
          color: var(--text);
          color-scheme: light dark;
          font: 14px/1.5 system-ui, "Segoe UI Variable", "Segoe UI", sans-serif;
          font-optical-sizing: auto;
        }
        .ai-ceo-v2 *,
        .ai-ceo-v2 *::before,
        .ai-ceo-v2 *::after { box-sizing: border-box; }
        .ai-ceo-v2 button,
        .ai-ceo-v2 input,
        .ai-ceo-v2 select,
        .ai-ceo-v2 textarea {
          color: inherit;
          font: inherit;
          touch-action: manipulation;
          -webkit-tap-highlight-color: transparent;
        }
        .ai-ceo-v2 button { cursor: pointer; }
        .ai-ceo-v2 button:disabled { cursor: not-allowed; opacity: .56; }
        .ai-ceo-v2 button:active:not(:disabled) { transform: scale(.985); }
        .ai-ceo-v2 :focus-visible {
          outline: 3px solid var(--focus);
          outline-offset: 2px;
        }
        .ai-ceo-v2 ::selection { background: var(--accent); color: var(--on-accent); }
        .ai-ceo-v2 h1,
        .ai-ceo-v2 h2,
        .ai-ceo-v2 h3,
        .ai-ceo-v2 p { margin-block-start: 0; }
        .ai-ceo-v2 h1,
        .ai-ceo-v2 h2,
        .ai-ceo-v2 h3 {
          scroll-margin-top: 72px;
          text-wrap: balance;
        }
        .ai-ceo-v2 p { text-wrap: pretty; }
        .ai-ceo-v2 p,
        .ai-ceo-v2 dd,
        .ai-ceo-v2 td,
        .ai-ceo-v2 th,
        .ai-ceo-v2 span,
        .ai-ceo-v2 small { overflow-wrap: anywhere; }
        .ai-ceo-v2 h1 { margin: 0; font-size: 16px; line-height: 1.25; letter-spacing: -.012em; }
        .ai-ceo-v2 h2 { margin-block-end: 7px; font-size: 22px; line-height: 1.24; letter-spacing: -.022em; }
        .ai-ceo-v2 h3 { margin-block-end: 5px; font-size: 15px; line-height: 1.3; letter-spacing: -.008em; }
        .ai-ceo-v2 p { margin-block-end: 10px; max-width: 72ch; }
        .ai-ceo-v2 a { color: var(--link); text-underline-offset: .2em; }
        .ai-ceo-v2 caption {
          caption-side: top;
          text-align: start;
          padding-block-end: 8px;
          color: var(--muted);
          font-size: 12px;
        }
        .ai-ceo-v2 .skip-link {
          position: fixed;
          z-index: 100;
          inset-block-start: 8px;
          inset-inline-start: 8px;
          transform: translateY(-150%);
          border: 1px solid var(--line-strong);
          background: var(--surface);
          padding: 9px 12px;
        }
        .ai-ceo-v2 .skip-link:focus { transform: none; }
        .ai-ceo-v2 .prototype-bar {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 16px;
          min-height: 52px;
          border-block-end: 1px solid var(--line-strong);
          background: var(--chrome);
          padding: 7px 12px;
        }
        .ai-ceo-v2 .prototype-title { min-width: 0; }
        .ai-ceo-v2 .prototype-title span { display: block; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .scenario-select {
          display: grid;
          grid-template-columns: auto minmax(240px, 360px);
          align-items: center;
          gap: 8px;
        }
        .ai-ceo-v2 .scenario-select span { color: var(--muted); font-size: 12px; font-weight: 650; }
        .ai-ceo-v2 .scenario-select select,
        .ai-ceo-v2 .field input,
        .ai-ceo-v2 .field select,
        .ai-ceo-v2 .field textarea,
        .ai-ceo-v2 .state-lab-controls select {
          min-height: 44px;
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 8px 10px;
        }
        .ai-ceo-v2 .prototype-bar,
        .ai-ceo-v2 .shell {
          flex: 0 0 auto;
          min-width: 1100px;
        }
        .ai-ceo-v2 .shell {
          display: grid;
          grid-template-columns: 176px minmax(576px, 1fr) 348px;
          grid-auto-flow: column;
          grid-auto-columns: min-content;
          min-width: 1100px;
          min-height: calc(100vh - 52px);
        }
        .ai-ceo-v2 .primary-nav {
          display: flex;
          flex-direction: column;
          min-width: 0;
          border-inline-end: 1px solid var(--line);
          background: var(--chrome);
          padding: 10px 8px;
        }
        .ai-ceo-v2 .brand {
          padding: 8px 10px 17px;
          font-size: 15px;
          font-weight: 760;
          letter-spacing: -.01em;
        }
        .ai-ceo-v2 .primary-nav button {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 8px;
          width: 100%;
          min-height: 44px;
          border: 1px solid transparent;
          border-radius: 6px;
          background: transparent;
          padding: 9px 10px;
          text-align: start;
        }
        .ai-ceo-v2 .primary-nav button:hover { background: var(--fill); }
        .ai-ceo-v2 .primary-nav button[aria-current="page"] {
          border-color: var(--line-strong);
          background: var(--fill-strong);
          font-weight: 720;
        }
        .ai-ceo-v2 .nav-space { flex: 1; min-height: 24px; }
        .ai-ceo-v2 .settings-nav {
          border-block-start: 1px solid var(--line);
          border-radius: 0;
          margin-block-start: 8px;
          padding-block-start: 13px;
        }
        .ai-ceo-v2 .main-column { min-width: 0; }
        .ai-ceo-v2 .context-header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 16px;
          min-height: 56px;
          border-block-end: 1px solid var(--line);
          background: var(--surface);
          padding: 9px 16px;
        }
        .ai-ceo-v2 .context-header p { margin: 0 0 2px; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .context-header .scene-label {
          margin: 0;
          color: var(--text);
          font-size: 13px;
          font-weight: 680;
        }
        .ai-ceo-v2 .cycle-status {
          margin: 4px 0 0;
          color: var(--muted);
          font-size: 12px;
          max-width: none;
        }
        .ai-ceo-v2 .context-tools { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
        .ai-ceo-v2 .provenance {
          display: inline-flex;
          align-items: center;
          width: max-content;
          min-height: 22px;
          border: 1px solid var(--line-strong);
          border-radius: 2px;
          padding: 1px 6px;
          color: var(--muted);
          font-size: 11px;
          font-weight: 720;
          letter-spacing: .04em;
          text-transform: uppercase;
        }
        .ai-ceo-v2 .provenance[data-kind="proposed"] { border-color: var(--warn); color: var(--text); }
        .ai-ceo-v2 .provenance[data-kind="governed"] { border-color: var(--info); color: var(--text); }
        .ai-ceo-v2 .provenance[data-kind="verified"] { border-color: var(--good); color: var(--text); }
        .ai-ceo-v2 .decision-packet {
          display: grid;
          gap: 12px;
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 14px;
        }
        .ai-ceo-v2 .decision-packet > header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
        }
        .ai-ceo-v2 .decision-packet > header > span { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .packet-marks { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
        .ai-ceo-v2 .decision-packet h3 { margin: 0; font-size: 19px; }
        .ai-ceo-v2 .decision-packet > p { margin: 0; color: var(--muted); max-width: none; }
        .ai-ceo-v2 .packet-facts {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          margin: 0;
          border-block: 1px solid var(--line);
        }
        .ai-ceo-v2 .packet-facts > div {
          min-width: 0;
          border-inline-end: 1px solid var(--line);
          padding: 10px 12px 10px 0;
        }
        .ai-ceo-v2 .packet-facts > div:nth-child(4n) { border-inline-end: 0; }
        .ai-ceo-v2 .packet-facts dt { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .packet-facts dd { margin: 4px 0 0; }
        .ai-ceo-v2 .packet-actions { display: flex; flex-wrap: wrap; gap: 8px; }
        .ai-ceo-v2 .why-layer { min-width: 0; }
        .ai-ceo-v2 .why-layer summary,
        .ai-ceo-v2 .trace-fold summary {
          cursor: pointer;
          min-height: 44px;
          display: flex;
          align-items: center;
          color: var(--muted);
          font-size: 12px;
          font-weight: 680;
        }
        .ai-ceo-v2 .why-layer p,
        .ai-ceo-v2 .trace-fold p { margin: 8px 0 0; color: var(--muted); font-size: 13px; }
        .ai-ceo-v2 .trace-fold {
          border: 1px solid var(--line);
          border-radius: 6px;
          background: var(--fill);
          padding: 8px 10px;
          margin-block-end: 13px;
        }
        .ai-ceo-v2 .exception-lanes {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          border-block: 1px solid var(--line-strong);
        }
        .ai-ceo-v2 .exception-lanes button {
          display: grid;
          justify-items: start;
          gap: 6px;
          min-width: 0;
          min-height: 108px;
          border: 0;
          border-inline-end: 1px solid var(--line);
          border-radius: 0;
          background: transparent;
          padding: 12px 14px 12px 0;
          text-align: start;
        }
        .ai-ceo-v2 .exception-lanes button:last-child { border-inline-end: 0; }
        .ai-ceo-v2 .exception-lanes button:hover { background: var(--fill); }
        .ai-ceo-v2 .exception-lanes span { font-size: 12px; font-weight: 720; }
        .ai-ceo-v2 .exception-lanes button[data-tone="warn"] > span:first-child { color: var(--warn); }
        .ai-ceo-v2 .exception-lanes button[data-tone="info"] > span:first-child { color: var(--info); }
        .ai-ceo-v2 .exception-lanes button[data-tone="bad"] > span:first-child { color: var(--bad); }
        .ai-ceo-v2 .exception-lanes strong { font-size: 14px; }
        .ai-ceo-v2 .exception-lanes small { color: var(--muted); }
        .ai-ceo-v2 .staff-table-wrap { width: 100%; overflow: auto; }
        .ai-ceo-v2 .staff-table th small { display: block; color: var(--muted); font-weight: 400; }
        .ai-ceo-v2 .authority-path {
          display: grid;
          grid-template-columns: repeat(6, minmax(0, 1fr));
          gap: 6px;
          list-style: none;
          margin: 0 0 14px;
          padding: 0;
        }
        .ai-ceo-v2 .authority-path li {
          display: grid;
          gap: 6px;
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 6px;
          padding: 8px;
        }
        .ai-ceo-v2 .authority-path strong,
        .ai-ceo-v2 .authority-path span { display: block; }
        .ai-ceo-v2 .authority-path span { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .authority-path li[data-state="done"] { border-color: var(--good); }
        .ai-ceo-v2 .authority-path li[data-state="current"] { border-color: var(--warn); background: var(--fill); }
        .ai-ceo-v2 .why-fragment { margin-block-start: 14px; }
        .ai-ceo-v2 .main-content { min-width: 0; padding: 18px; }
        .ai-ceo-v2 .scene-stack { display: grid; gap: 14px; }
        .ai-ceo-v2 .tag {
          display: inline-flex;
          align-items: center;
          width: max-content;
          max-width: 100%;
          min-height: 26px;
          border: 1px solid var(--line-strong);
          border-radius: 999px;
          padding: 3px 8px;
          color: var(--text);
          font-size: 12px;
          font-weight: 690;
          line-height: 1.25;
        }
        .ai-ceo-v2 .tag[data-tone="good"] { border-color: var(--good); }
        .ai-ceo-v2 .tag[data-tone="warn"] { border-color: var(--warn); }
        .ai-ceo-v2 .tag[data-tone="bad"] { border-color: var(--bad); }
        .ai-ceo-v2 .tag[data-tone="info"] { border-color: var(--info); }
        .ai-ceo-v2 .primary-button,
        .ai-ceo-v2 .secondary-button,
        .ai-ceo-v2 .text-button,
        .ai-ceo-v2 .inline-button,
        .ai-ceo-v2 .segmented button,
        .ai-ceo-v2 .stage-tabs button,
        .ai-ceo-v2 .mention-buttons button,
        .ai-ceo-v2 .step-nav button {
          min-height: 44px;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 8px 12px;
        }
        .ai-ceo-v2 .primary-button {
          border-color: var(--accent);
          background: var(--accent);
          color: var(--on-accent);
          font-weight: 750;
        }
        .ai-ceo-v2 .primary-button:hover:not(:disabled) {
          background: var(--fill-strong);
          color: var(--text);
        }
        .ai-ceo-v2 .secondary-button:hover,
        .ai-ceo-v2 .text-button:hover,
        .ai-ceo-v2 .inline-button:hover,
        .ai-ceo-v2 .segmented button:hover,
        .ai-ceo-v2 .stage-tabs button:hover,
        .ai-ceo-v2 .mention-buttons button:hover,
        .ai-ceo-v2 .step-nav button:hover { background: var(--fill); }
        .ai-ceo-v2 .text-button,
        .ai-ceo-v2 .inline-button { background: transparent; }
        .ai-ceo-v2 .inline-button { min-height: 40px; padding: 6px 9px; }
        .ai-ceo-v2 .segmented {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
        }
        .ai-ceo-v2 .segmented button[aria-pressed="true"],
        .ai-ceo-v2 .provider-options button[aria-pressed="true"] {
          border-color: var(--accent);
          background: var(--fill-strong);
          font-weight: 720;
        }
        .ai-ceo-v2 .section-heading {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 12px;
          border-block-end: 1px solid var(--line);
          padding-block-end: 10px;
        }
        .ai-ceo-v2 .section-heading h3 { margin: 0; }
        .ai-ceo-v2 .section-heading p { margin: 3px 0 0; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .work-surface,
        .ai-ceo-v2 .comparison-surface,
        .ai-ceo-v2 .outcome-ledger,
        .ai-ceo-v2 .decision-preview,
        .ai-ceo-v2 .coverage-matrix,
        .ai-ceo-v2 .state-panel {
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 14px;
        }
        .ai-ceo-v2 .open-section { min-width: 0; padding: 4px 2px; }
        .ai-ceo-v2 .today-header,
        .ai-ceo-v2 .setup-header,
        .ai-ceo-v2 .project-header,
        .ai-ceo-v2 .temporary-header,
        .ai-ceo-v2 .operations-header,
        .ai-ceo-v2 .settings-header,
        .ai-ceo-v2 .capability-header,
        .ai-ceo-v2 .state-lab-header {
          display: flex;
          align-items: flex-end;
          justify-content: space-between;
          gap: 24px;
          border-block-end: 1px solid var(--line-strong);
          padding: 5px 2px 15px;
        }
        .ai-ceo-v2 .today-header p,
        .ai-ceo-v2 .setup-header p,
        .ai-ceo-v2 .project-header p,
        .ai-ceo-v2 .temporary-header p,
        .ai-ceo-v2 .operations-header p,
        .ai-ceo-v2 .settings-header p,
        .ai-ceo-v2 .capability-header p,
        .ai-ceo-v2 .state-lab-header p { margin: 0; color: var(--muted); }
        .ai-ceo-v2 .header-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 7px; }
        .ai-ceo-v2 .staff-strip { min-width: 0; }
        .ai-ceo-v2 .operating-report {
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 14px;
        }
        .ai-ceo-v2 .report-grid {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          border-block-start: 1px solid var(--line);
        }
        .ai-ceo-v2 .report-grid section {
          min-width: 0;
          border-inline-end: 1px solid var(--line);
          padding: 10px 12px 10px 0;
        }
        .ai-ceo-v2 .report-grid section:nth-child(4n) { border-inline-end: 0; }
        .ai-ceo-v2 .report-grid span,
        .ai-ceo-v2 .report-grid small { display: block; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .report-grid strong { display: block; margin: 4px 0 3px; }
        .ai-ceo-v2 .thread-cards {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 8px;
          list-style: none;
          margin: 12px 0;
          padding: 0;
        }
        .ai-ceo-v2 .thread-cards li {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 6px;
          padding: 10px;
        }
        .ai-ceo-v2 .thread-cards span { color: var(--muted); font-size: 12px; font-variant-numeric: tabular-nums; }
        .ai-ceo-v2 .thread-cards p { margin: 6px 0 0; font-size: 13px; }
        .ai-ceo-v2 .messages article.approval-card {
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--fill);
          padding: 9px;
        }
        .ai-ceo-v2 .ledger-facts,
        .ai-ceo-v2 .definition-list,
        .ai-ceo-v2 .artifact-parts,
        .ai-ceo-v2 .decision-preview dl { margin: 0; }
        .ai-ceo-v2 .definition-list > div,
        .ai-ceo-v2 .artifact-parts > div,
        .ai-ceo-v2 .decision-preview dl > div {
          display: grid;
          grid-template-columns: minmax(126px, .36fr) minmax(0, 1fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 8px 0;
        }
        .ai-ceo-v2 .definition-list > div:last-child,
        .ai-ceo-v2 .artifact-parts > div:last-child,
        .ai-ceo-v2 .decision-preview dl > div:last-child { border-block-end: 0; }
        .ai-ceo-v2 dt { color: var(--muted); }
        .ai-ceo-v2 dd { min-width: 0; margin: 0; }
        .ai-ceo-v2 dd strong,
        .ai-ceo-v2 dd small { display: block; }
        .ai-ceo-v2 dd small { margin-block-start: 2px; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .definition-list.compact > div { padding: 7px 0; }
        .ai-ceo-v2 .result-list {
          list-style: none;
          margin: 4px 0 0;
          padding: 0;
        }
        .ai-ceo-v2 .result-list li {
          display: grid;
          grid-template-columns: minmax(0, 1fr) minmax(148px, auto);
          gap: 14px;
          border-block-end: 1px solid var(--line);
          padding: 12px 0;
        }
        .ai-ceo-v2 .result-list li:last-child { border-block-end: 0; }
        .ai-ceo-v2 .result-list strong,
        .ai-ceo-v2 .result-list span,
        .ai-ceo-v2 .result-list small { display: block; }
        .ai-ceo-v2 .result-list span,
        .ai-ceo-v2 .result-list small { margin-block-start: 3px; color: var(--muted); }
        .ai-ceo-v2 .result-list li > div:last-child { display: grid; align-content: start; justify-items: end; gap: 4px; text-align: end; }
        .ai-ceo-v2 .accepted-line {
          display: grid;
          grid-template-columns: minmax(220px, .8fr) minmax(0, 1.2fr);
          gap: 20px;
          padding-block-start: 13px;
        }
        .ai-ceo-v2 .accepted-line > div strong,
        .ai-ceo-v2 .accepted-line > div span { display: block; }
        .ai-ceo-v2 .accepted-line > div strong { font-size: 16px; }
        .ai-ceo-v2 .accepted-line > div span { margin-block-start: 4px; color: var(--muted); }
        .ai-ceo-v2 .ledger-facts { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
        .ai-ceo-v2 .ledger-facts div { border-inline-start: 1px solid var(--line); padding-inline-start: 10px; }
        .ai-ceo-v2 .ledger-facts dt,
        .ai-ceo-v2 .ledger-facts dd { display: block; }
        .ai-ceo-v2 .first-run {
          display: grid;
          grid-template-columns: minmax(0, 1.35fr) minmax(280px, .65fr);
          gap: 28px;
          min-height: 320px;
          align-items: center;
          border-block: 1px solid var(--line-strong);
          padding: 34px 4px;
        }
        .ai-ceo-v2 .first-run-copy h3,
        .ai-ceo-v2 .first-run-copy h2 { margin-block-start: 12px; font-size: 21px; }
        .ai-ceo-v2 .first-run-copy p,
        .ai-ceo-v2 .first-run-copy li { color: var(--muted); }
        .ai-ceo-v2 .first-run-copy ul { display: grid; gap: 8px; padding-inline-start: 20px; }
        .ai-ceo-v2 .first-run-action { display: grid; gap: 9px; border-inline-start: 1px solid var(--line); padding-inline-start: 22px; }
        .ai-ceo-v2 .first-run-action span { color: var(--muted); }
        .ai-ceo-v2 .state-panel header { display: flex; align-items: center; gap: 8px; margin-block-end: 8px; }
        .ai-ceo-v2 .state-panel p { max-width: none; }
        .ai-ceo-v2 .state-panel dl { margin: 0; }
        .ai-ceo-v2 .state-panel dl > div {
          display: grid;
          grid-template-columns: minmax(140px, .32fr) minmax(0, 1fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 8px 0;
        }
        .ai-ceo-v2 .state-panel[data-tone="bad"] { border-color: var(--bad); }
        .ai-ceo-v2 .state-panel[data-tone="warn"] { border-color: var(--warn); }
        .ai-ceo-v2 .state-panel[data-tone="info"] { border-color: var(--info); }
        .ai-ceo-v2 .state-panel[data-tone="good"] { border-color: var(--good); }
        .ai-ceo-v2 .settings-actions { display: flex; flex-wrap: wrap; gap: 8px; padding-block-start: 12px; }
        .ai-ceo-v2 .step-count { display: grid; min-width: 125px; text-align: end; }
        .ai-ceo-v2 .step-count span { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .step-nav {
          display: grid;
          grid-template-columns: repeat(5, minmax(0, 1fr));
          gap: 6px;
        }
        .ai-ceo-v2 .step-nav button {
          display: flex;
          align-items: center;
          justify-content: flex-start;
          gap: 8px;
          background: transparent;
          text-align: start;
        }
        .ai-ceo-v2 .step-nav button span { color: var(--muted); font-variant-numeric: tabular-nums; }
        .ai-ceo-v2 .step-nav button[aria-current="step"] { border-color: var(--accent); background: var(--fill-strong); }
        .ai-ceo-v2 .field { display: grid; gap: 5px; margin-block-start: 12px; }
        .ai-ceo-v2 .field > span { font-weight: 680; }
        .ai-ceo-v2 .field > small { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .field textarea { min-height: 150px; resize: vertical; }
        .ai-ceo-v2 .research-summary,
        .ai-ceo-v2 .preview-summary,
        .ai-ceo-v2 .running-summary,
        .ai-ceo-v2 .memory-record header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 16px;
          padding-block: 12px;
        }
        .ai-ceo-v2 .research-summary strong,
        .ai-ceo-v2 .research-summary span,
        .ai-ceo-v2 .preview-summary strong,
        .ai-ceo-v2 .preview-summary span,
        .ai-ceo-v2 .running-summary strong,
        .ai-ceo-v2 .running-summary span,
        .ai-ceo-v2 .running-summary small,
        .ai-ceo-v2 .memory-record header strong,
        .ai-ceo-v2 .memory-record header span { display: block; }
        .ai-ceo-v2 .research-summary span,
        .ai-ceo-v2 .preview-summary span,
        .ai-ceo-v2 .running-summary span,
        .ai-ceo-v2 .running-summary small,
        .ai-ceo-v2 .memory-record header span { color: var(--muted); }
        .ai-ceo-v2 .revision-label { text-align: end; }
        .ai-ceo-v2 .simulation-path,
        .ai-ceo-v2 .run-steps,
        .ai-ceo-v2 .reconcile-path,
        .ai-ceo-v2 .context-ladder {
          display: grid;
          list-style: none;
          margin: 12px 0;
          padding: 0;
        }
        .ai-ceo-v2 .simulation-path { grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 7px; }
        .ai-ceo-v2 .simulation-path li,
        .ai-ceo-v2 .run-steps li,
        .ai-ceo-v2 .reconcile-path li {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 6px;
          padding: 10px;
        }
        .ai-ceo-v2 .simulation-path strong,
        .ai-ceo-v2 .simulation-path span,
        .ai-ceo-v2 .run-steps strong,
        .ai-ceo-v2 .run-steps span,
        .ai-ceo-v2 .reconcile-path strong,
        .ai-ceo-v2 .reconcile-path span { display: block; }
        .ai-ceo-v2 .simulation-path span,
        .ai-ceo-v2 .run-steps span,
        .ai-ceo-v2 .reconcile-path span { margin-block-start: 4px; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .simulation-path li[data-state="done"] { border-color: var(--good); }
        .ai-ceo-v2 .simulation-path li[data-state="partial"] { border-color: var(--warn); }
        .ai-ceo-v2 .simulation-path li[data-state="blocked"] { border-color: var(--bad); }
        .ai-ceo-v2 .gap-summary {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 12px;
          margin-block-start: 12px;
        }
        .ai-ceo-v2 .gap-summary > div { display: grid; gap: 6px; border-block-start: 1px solid var(--line); padding-block-start: 10px; }
        .ai-ceo-v2 .gap-summary span { color: var(--muted); }
        .ai-ceo-v2 .flow-actions { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
        .ai-ceo-v2 .flow-end { color: var(--muted); }
        .ai-ceo-v2 .stage-tabs {
          display: flex;
          flex-wrap: wrap;
          gap: 5px;
          border-block-end: 1px solid var(--line);
          padding-block-end: 9px;
        }
        .ai-ceo-v2 .stage-tabs button { border-color: transparent; background: transparent; }
        .ai-ceo-v2 .stage-tabs button[aria-current="page"] { border-color: var(--line-strong); background: var(--fill-strong); font-weight: 720; }
        .ai-ceo-v2 .package-layout {
          display: grid;
          grid-template-columns: minmax(0, 1.35fr) minmax(240px, .65fr);
          gap: 16px;
        }
        .ai-ceo-v2 .artifact-preview { min-width: 0; border-block: 1px solid var(--line-strong); padding-block: 14px; }
        .ai-ceo-v2 .artifact-preview header { display: flex; justify-content: space-between; gap: 12px; }
        .ai-ceo-v2 .artifact-preview header h3 { margin-block-start: 8px; font-size: 19px; }
        .ai-ceo-v2 .artifact-preview header > span { color: var(--muted); }
        .ai-ceo-v2 .thread-copy { margin-block: 10px 16px; max-width: 56ch; font-size: 17px; line-height: 1.55; }
        .ai-ceo-v2 .acceptance-checks { border: 1px solid var(--line-strong); border-radius: 7px; background: var(--surface); padding: 13px; }
        .ai-ceo-v2 .acceptance-checks ul { list-style: none; margin: 8px 0; padding: 0; }
        .ai-ceo-v2 .acceptance-checks li { display: grid; grid-template-columns: 56px minmax(0, 1fr); gap: 8px; border-block-end: 1px solid var(--line); padding: 8px 0; }
        .ai-ceo-v2 .acceptance-checks li > span { color: var(--info); font-size: 12px; font-weight: 800; }
        .ai-ceo-v2 .acceptance-checks strong,
        .ai-ceo-v2 .acceptance-checks small { display: block; }
        .ai-ceo-v2 .acceptance-checks small,
        .ai-ceo-v2 .acceptance-checks p { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .receipt-head { display: flex; align-items: center; gap: 10px; padding-block: 13px; }
        .ai-ceo-v2 .readback-grid,
        .ai-ceo-v2 .reflection-grid {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          margin-block: 12px;
          border-block: 1px solid var(--line);
        }
        .ai-ceo-v2 .reflection-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
        .ai-ceo-v2 .readback-grid > div,
        .ai-ceo-v2 .reflection-grid > div { min-width: 0; border-inline-end: 1px solid var(--line); padding: 11px; }
        .ai-ceo-v2 .readback-grid > div:last-child,
        .ai-ceo-v2 .reflection-grid > div:last-child { border-inline-end: 0; }
        .ai-ceo-v2 .readback-grid span,
        .ai-ceo-v2 .readback-grid strong,
        .ai-ceo-v2 .readback-grid small { display: block; }
        .ai-ceo-v2 .readback-grid span,
        .ai-ceo-v2 .readback-grid small,
        .ai-ceo-v2 .reflection-grid p { color: var(--muted); }
        .ai-ceo-v2 .readback-grid strong { margin-block: 5px; }
        .ai-ceo-v2 .loop-ledger { border-block-start: 1px solid var(--line-strong); padding-block-start: 12px; }
        .ai-ceo-v2 .loop-ledger ol {
          display: grid;
          grid-template-columns: repeat(8, minmax(116px, 1fr));
          gap: 6px;
          overflow-x: auto;
          list-style: none;
          margin: 10px 0 0;
          padding: 0 0 6px;
        }
        .ai-ceo-v2 .loop-ledger li { border: 1px solid var(--line); border-radius: 6px; padding: 8px; }
        .ai-ceo-v2 .loop-ledger strong,
        .ai-ceo-v2 .loop-ledger span { display: block; }
        .ai-ceo-v2 .loop-ledger span { margin-block-start: 3px; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .loop-ledger li[data-state="done"] { border-color: var(--good); }
        .ai-ceo-v2 .loop-ledger li[data-state="partial"],
        .ai-ceo-v2 .loop-ledger li[data-state="waiting"] { border-color: var(--warn); }
        .ai-ceo-v2 .loop-ledger li[data-state="blocked"] { border-color: var(--bad); }
        .ai-ceo-v2 .loop-ledger li[data-state="sample"] { border-style: dashed; }
        .ai-ceo-v2 .comparison-table-wrap { width: 100%; overflow: auto; margin-block-start: 10px; }
        .ai-ceo-v2 table { width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums; }
        .ai-ceo-v2 th,
        .ai-ceo-v2 td { min-width: 112px; border-block-end: 1px solid var(--line); padding: 9px 8px; text-align: start; vertical-align: top; }
        .ai-ceo-v2 thead th { color: var(--muted); font-size: 12px; font-weight: 700; }
        .ai-ceo-v2 tbody th { font-weight: 700; }
        .ai-ceo-v2 tbody tr[data-selected="true"] { background: var(--fill); }
        .ai-ceo-v2 .typed-canvas-grid {
          display: grid;
          grid-template-columns: minmax(0, 1.15fr) minmax(260px, .85fr);
          gap: 16px;
        }
        .ai-ceo-v2 .decision-panel,
        .ai-ceo-v2 .evidence-panel { min-width: 0; border-block-start: 1px solid var(--line-strong); padding-block-start: 12px; }
        .ai-ceo-v2 .decision-panel p { font-size: 16px; font-weight: 650; }
        .ai-ceo-v2 .decision-panel small { color: var(--muted); }
        .ai-ceo-v2 .evidence-panel ul { list-style: none; margin: 8px 0 0; padding: 0; }
        .ai-ceo-v2 .evidence-panel li { display: grid; grid-template-columns: minmax(120px, .35fr) minmax(0, 1fr); gap: 8px; border-block-end: 1px solid var(--line); padding: 8px 0; }
        .ai-ceo-v2 .evidence-panel span { color: var(--muted); }
        .ai-ceo-v2 .object-chain {
          display: grid;
          grid-template-columns: minmax(140px, 1fr) auto minmax(140px, 1fr) auto minmax(120px, .8fr) auto minmax(165px, 1.2fr);
          align-items: center;
          gap: 7px;
          border-block-end: 1px solid var(--line-strong);
          padding-block-end: 14px;
        }
        .ai-ceo-v2 .object-chain > div { min-width: 0; border: 1px solid var(--line); border-radius: 6px; padding: 9px; }
        .ai-ceo-v2 .object-chain strong,
        .ai-ceo-v2 .object-chain div span { display: block; }
        .ai-ceo-v2 .object-chain div span,
        .ai-ceo-v2 .object-chain > span { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .people-layout {
          display: grid;
          grid-template-columns: 220px minmax(0, 1fr);
          gap: 14px;
        }
        .ai-ceo-v2 .member-list { border: 1px solid var(--line-strong); border-radius: 7px; background: var(--surface); padding: 7px; }
        .ai-ceo-v2 .member-list button {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 8px;
          width: 100%;
          min-height: 58px;
          border: 1px solid transparent;
          border-block-end-color: var(--line);
          background: transparent;
          padding: 8px;
          text-align: start;
        }
        .ai-ceo-v2 .member-list button:hover { background: var(--fill); }
        .ai-ceo-v2 .member-list button[aria-current="page"] { border-color: var(--line-strong); background: var(--fill-strong); }
        .ai-ceo-v2 .member-list strong,
        .ai-ceo-v2 .member-list small { display: block; }
        .ai-ceo-v2 .member-list small { color: var(--muted); }
        .ai-ceo-v2 .version-compare {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 14px;
          margin-block: 12px;
        }
        .ai-ceo-v2 .version-compare > div { min-width: 0; border-block: 1px solid var(--line); padding-block: 11px; }
        .ai-ceo-v2 .version-compare span,
        .ai-ceo-v2 .version-compare strong { display: block; }
        .ai-ceo-v2 .version-compare span,
        .ai-ceo-v2 .version-compare p { color: var(--muted); }
        .ai-ceo-v2 .version-compare strong { margin-block: 5px; }
        .ai-ceo-v2 .run-steps,
        .ai-ceo-v2 .reconcile-path { grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 7px; }
        .ai-ceo-v2 .reconcile-path { grid-template-columns: repeat(4, minmax(0, 1fr)); }
        .ai-ceo-v2 .run-steps li[data-state="done"] { border-color: var(--good); }
        .ai-ceo-v2 .run-steps li[data-state="current"],
        .ai-ceo-v2 .reconcile-path li[data-state="current"] { border-color: var(--warn); background: var(--fill); }
        .ai-ceo-v2 .reconcile-path li[data-state="done"] { border-color: var(--info); }
        .ai-ceo-v2 .memory-record { padding-block-start: 4px; }
        .ai-ceo-v2 .memory-record > p { color: var(--muted); }
        .ai-ceo-v2 .memory-record .segmented { margin-block: 12px; }
        .ai-ceo-v2 .context-budget { display: grid; gap: 9px; padding-block: 12px; }
        .ai-ceo-v2 .context-budget strong,
        .ai-ceo-v2 .context-budget span { display: block; }
        .ai-ceo-v2 .context-budget span { color: var(--muted); }
        .ai-ceo-v2 .context-ladder { gap: 0; }
        .ai-ceo-v2 .context-ladder li {
          display: grid;
          grid-template-columns: 34px minmax(0, 1fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 9px 0;
        }
        .ai-ceo-v2 .context-ladder li > span { color: var(--muted); }
        .ai-ceo-v2 .context-ladder strong,
        .ai-ceo-v2 .context-ladder small { display: block; }
        .ai-ceo-v2 .context-ladder small { color: var(--muted); }
        .ai-ceo-v2 .context-ladder li[data-protected="true"] strong::after { content: " · protected"; color: var(--info); font-size: 12px; }
        .ai-ceo-v2 .connection-layout {
          display: grid;
          grid-template-columns: minmax(0, 1.05fr) minmax(300px, .95fr);
          gap: 14px;
        }
        .ai-ceo-v2 .provider-options {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 7px;
          margin-block: 12px;
        }
        .ai-ceo-v2 .provider-options button {
          min-height: 72px;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: transparent;
          padding: 10px;
          text-align: start;
        }
        .ai-ceo-v2 .provider-options strong,
        .ai-ceo-v2 .provider-options span { display: block; }
        .ai-ceo-v2 .provider-options span { margin-block-start: 4px; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .custom-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0 16px; }
        .ai-ceo-v2 .secret-route { display: grid; align-content: center; gap: 5px; min-height: 88px; margin-block-start: 12px; border-block: 1px solid var(--line); }
        .ai-ceo-v2 .secret-route span { color: var(--muted); }
        .ai-ceo-v2 .capability-layout {
          display: grid;
          grid-template-columns: minmax(0, 1.25fr) minmax(270px, .75fr);
          gap: 14px;
        }
        .ai-ceo-v2 .review-rows { margin-block-start: 9px; }
        .ai-ceo-v2 .review-rows > div {
          display: grid;
          grid-template-columns: minmax(125px, .28fr) minmax(0, 1fr) minmax(130px, .35fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 8px 0;
        }
        .ai-ceo-v2 .review-rows span,
        .ai-ceo-v2 .review-rows small { color: var(--muted); }
        .ai-ceo-v2 .decision-preview > p,
        .ai-ceo-v2 .decision-preview output { color: var(--muted); }
        .ai-ceo-v2 .decision-preview .segmented { display: grid; margin-block: 12px; }
        .ai-ceo-v2 .decision-preview output { display: block; border-block: 1px solid var(--line); padding-block: 10px; }
        .ai-ceo-v2 .state-lab-controls { display: grid; grid-template-columns: repeat(2, minmax(160px, 1fr)); gap: 8px; }
        .ai-ceo-v2 .state-lab-controls label { display: grid; gap: 4px; }
        .ai-ceo-v2 .state-lab-controls label span { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .additional-states {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          border-block: 1px solid var(--line);
        }
        .ai-ceo-v2 .additional-states > div { min-width: 0; border-inline-end: 1px solid var(--line); padding: 11px; }
        .ai-ceo-v2 .additional-states > div:last-child { border-inline-end: 0; }
        .ai-ceo-v2 .additional-states strong,
        .ai-ceo-v2 .additional-states span { display: block; }
        .ai-ceo-v2 .additional-states span { margin-block-start: 4px; color: var(--muted); }
        .ai-ceo-v2 .conversation {
          display: flex;
          flex-direction: column;
          min-width: 0;
          min-height: 0;
          border-inline-start: 1px solid var(--line);
          background: var(--surface);
        }
        .ai-ceo-v2 .conversation > header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 12px;
        }
        .ai-ceo-v2 .conversation > header span { color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .conversation > header h2 { margin: 2px 0 0; font-size: 16px; }
        .ai-ceo-v2 .participants {
          display: flex;
          flex-wrap: wrap;
          gap: 5px;
          border-block-end: 1px solid var(--line);
          padding: 8px 12px;
        }
        .ai-ceo-v2 .participants span { border: 1px solid var(--line); border-radius: 999px; padding: 3px 7px; color: var(--muted); font-size: 12px; }
        .ai-ceo-v2 .messages { flex: 1; min-height: 230px; overflow-y: auto; overscroll-behavior: contain; padding: 12px; }
        .ai-ceo-v2 .messages article { border-block-end: 1px solid var(--line); margin-block-end: 13px; padding-block-end: 12px; }
        .ai-ceo-v2 .messages article > span { color: var(--muted); font-size: 12px; font-weight: 680; }
        .ai-ceo-v2 .messages article p { margin: 4px 0; }
        .ai-ceo-v2 .messages article small { display: block; color: var(--muted); }
        .ai-ceo-v2 .messages article .inline-button { margin-block-start: 8px; }
        .ai-ceo-v2 .messages article[data-author="owner"] p { margin-inline-start: 14px; font-weight: 620; }
        .ai-ceo-v2 .messages article[data-author="system"] { border: 1px solid var(--line); border-radius: 6px; background: var(--fill); padding: 9px; }
        .ai-ceo-v2 .composer { display: grid; gap: 8px; border-block-start: 1px solid var(--line); padding: 10px; }
        .ai-ceo-v2 .composer > label { display: grid; gap: 5px; }
        .ai-ceo-v2 .composer > label > span { font-weight: 680; }
        .ai-ceo-v2 .composer textarea {
          width: 100%;
          min-height: 96px;
          resize: vertical;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 9px 10px;
        }
        .ai-ceo-v2 .mention-buttons { display: flex; flex-wrap: wrap; gap: 5px; }
        .ai-ceo-v2 .mention-buttons button { min-height: 40px; padding: 6px 9px; }
        .ai-ceo-v2 .composer-actions { display: grid; gap: 6px; }
        .ai-ceo-v2 .composer-actions small { color: var(--muted); }
        .ai-ceo-v2 .composer .gap { grid-template-columns: 1fr; gap: 3px; font-size: 12px; }
        @media (prefers-reduced-motion: reduce) {
          .ai-ceo-v2 *,
          .ai-ceo-v2 *::before,
          .ai-ceo-v2 *::after {
            animation-duration: .01ms !important;
            transition-duration: .01ms !important;
            scroll-behavior: auto !important;
          }
          .ai-ceo-v2 button:active:not(:disabled) { transform: none; }
        }
        @media (prefers-contrast: more) {
          .ai-ceo-v2 .work-surface,
          .ai-ceo-v2 .comparison-surface,
          .ai-ceo-v2 .decision-packet,
          .ai-ceo-v2 .outcome-ledger,
          .ai-ceo-v2 .decision-preview,
          .ai-ceo-v2 .coverage-matrix,
          .ai-ceo-v2 .state-panel { border-color: var(--text); }
        }
      `}</style>

      <a className="skip-link" href="#v2-main">Skip to main workbench</a>

      <header className="prototype-bar">
        <div className="prototype-title">
          <h1>Personal 2.0 · AI CEO end-to-end prototype V2</h1>
          <span>Propose in chat · authorize on canvas · verify independently · local state only</span>
        </div>
        <label className="scenario-select">
          <span>Prototype scenario</span>
          <Select
            value={scene}
            onChange={(next) => setScene(next as Scene)}
            options={SCENES.map((item) => ({
              value: item.id,
              label: item.label,
            }))}
          />
        </label>
      </header>

      <div className="shell">
        <nav className="primary-nav" aria-label="Personal primary navigation">
          <div className="brand">Personal</div>
          <button
            type="button"
            aria-current={scene === "today" ? "page" : undefined}
            onClick={() => setScene("today")}
          >
            Today
            {firstRun ? null : <Tag tone="warn">1</Tag>}
          </button>
          <button
            type="button"
            aria-current={projectsCurrent ? "page" : undefined}
            onClick={() => setScene("projects")}
          >
            Projects
          </button>
          <button
            type="button"
            aria-current={scene === "knowledge" ? "page" : undefined}
            onClick={() => setScene("knowledge")}
          >
            Knowledge
          </button>
          <div className="nav-space" />
          <button
            className="settings-nav"
            type="button"
            aria-current={settingsCurrent ? "page" : undefined}
            onClick={() => setScene("settings")}
          >
            Settings
          </button>
        </nav>

        <main className="main-column" id="v2-main">
          <header className="context-header">
            <div>
              <p>{locationLabel}</p>
              <p className="scene-label">{SCENE_TITLES[scene]}</p>
              <CycleStatus current={loopStepFor(scene, todayMode, xStage, operationsView)} />
            </div>
            <div className="context-tools">
              <Tag tone="neutral">Windows-local · host online</Tag>
              <Tag tone="info">{channel === "project" ? "Group conversation" : "Personal Assistant"}</Tag>
            </div>
          </header>

          <div className="main-content">
            <MainScene
              scene={scene}
              setScene={setScene}
              todayMode={todayMode}
              setTodayMode={setTodayMode}
              setupStage={setupStage}
              setSetupStage={setSetupStage}
              brief={brief}
              setBrief={setBrief}
              xStage={xStage}
              setXStage={setXStage}
              candidatePreview={candidatePreview}
              selectedOutcome={selectedOutcome}
              setSelectedOutcome={setSelectedOutcome}
              pinned={pinned}
              setPinned={setPinned}
              peopleView={peopleView}
              setPeopleView={setPeopleView}
              memberId={memberId}
              setMemberId={setMemberId}
              operationsView={operationsView}
              setOperationsView={setOperationsView}
              knowledgeView={knowledgeView}
              setKnowledgeView={setKnowledgeView}
              memoryAction={memoryAction}
              setMemoryAction={setMemoryAction}
              connectionView={connectionView}
              setConnectionView={setConnectionView}
              provider={provider}
              setProvider={setProvider}
              model={model}
              setModel={setModel}
              capabilityView={capabilityView}
              setCapabilityView={setCapabilityView}
              capabilityDecision={capabilityDecision}
              setCapabilityDecision={setCapabilityDecision}
              labSurface={labSurface}
              setLabSurface={setLabSurface}
              labState={labState}
              setLabState={setLabState}
            />
          </div>
        </main>

        <Conversation
          channel={channel}
          drafts={drafts}
          setDrafts={setDrafts}
          status={composerStatus}
          setStatus={setComposerStatus}
          setScene={setScene}
          setCandidatePreview={setCandidatePreview}
          setXStage={setXStage}
          firstRun={firstRun}
        />
      </div>
    </div>
  );
}
