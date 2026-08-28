/**
 * DESIGN PROTOTYPE EVIDENCE BOUNDARY
 * This is a newly created Cursor Canvas interaction prototype. It does not
 * replace any earlier prototype or claim that the Personal 2.0 backend,
 * Windows host, connectors, conversations, receipts, or verification exist.
 * All visible business data is built-in mock data. Local React state only
 * previews navigation, disclosure, drafting, and pin intent; nothing is sent,
 * saved, installed, approved, published, or written to daemon authority.
 *
 * THESIS: outcomes and openable deliverables lead; state follows; runtime
 * configuration stays one disclosure deeper. The visual world is a restrained
 * Windows operating desk: quiet neutral planes, precise rules, compact type,
 * and one blue action accent—never an AI card wall, gradient, or glass effect.
 */

import {
  useHostTheme,
  useState,
  type CSSProperties,
} from "cursor/canvas";

type Scene =
  | "today"
  | "project"
  | "adhoc"
  | "setup"
  | "runtime"
  | "knowledge"
  | "connections"
  | "recovery"
  | "state-lab";

type ProjectSurface = "brief" | "team" | "attention";
type SetupStep =
  | "understand"
  | "research"
  | "contract"
  | "team"
  | "routine"
  | "models"
  | "permissions"
  | "simulate"
  | "preview";
type SetupView = "research-partial" | "launch-preview";
type MemberId = "manager" | "researcher" | "editor";
type KnowledgeTab = "vault" | "memory" | "context";
type MemoryPreview = "inspect" | "correct" | "forget";
type ConnectionMode = "quick" | "custom";
type ProviderId = "anthropic" | "openai" | "google" | "custom";
type RecoveryTab = "approval" | "missed" | "unknown" | "mcp";
type LabState =
  | "empty"
  | "loading"
  | "partial"
  | "stale"
  | "permission"
  | "error"
  | "unknown"
  | "offline"
  | "missed"
  | "running"
  | "success"
  | "archived"
  | "requires-backend";
type Tone = "neutral" | "good" | "warn" | "bad" | "info";
type ChatChannel = "assistant" | "project-group";
type PanelKind = "summary" | "comparison" | "evidence" | "decision";

type Member = {
  id: MemberId;
  name: string;
  role: string;
  roleSource: string;
  responsibility: string;
  currentWork: string;
  nextRun: string;
  acceptedResult: string;
  block: string;
  cost: string;
  ownerNeed: string;
  provider: string;
  model: string;
  workInstruction: string;
  skills: string;
  tools: string;
  mcp: string;
  loop: string;
  memory: string;
  context: string;
  permissions: string;
  harness: string;
};

type Artifact = {
  id: string;
  title: string;
  format: string;
  angle: string;
  openability: string;
  acceptance: string;
  evidence: string;
  metric: string;
  issue: string;
};

type BoardFact = {
  label: string;
  value: string;
  meta: string;
};

type BoardPanel = {
  id: string;
  kind: PanelKind;
  title: string;
  summary?: string;
  artifactIds?: readonly string[];
  facts?: readonly BoardFact[];
};

type PanelProps = {
  panel: BoardPanel;
  selectedArtifact: string;
  setSelectedArtifact: (value: string) => void;
};

const SCENES: ReadonlyArray<{ id: Scene; label: string }> = [
  { id: "today", label: "Today · owner decisions" },
  { id: "project", label: "X project · operating brief" },
  { id: "adhoc", label: "Temporary canvas · compare outcomes" },
  { id: "setup", label: "Research-driven project setup" },
  { id: "runtime", label: "Roles, members, and Runtime" },
  { id: "knowledge", label: "Knowledge, Vault, and Memory" },
  { id: "connections", label: "Settings · Model Connections" },
  { id: "recovery", label: "Approval, missed, unknown, recovery" },
  { id: "state-lab", label: "State Lab" },
];

const SCENE_TITLES: Record<Scene, string> = {
  today: "Today",
  project: "X content operation",
  adhoc: "Compare this week’s three content outcomes",
  setup: "Create a governed Project",
  runtime: "Project team and Runtime",
  knowledge: "Knowledge",
  connections: "Model Connections",
  recovery: "Project attention and recovery",
  "state-lab": "State Lab",
};

const PRIMARY_NAV: ReadonlyArray<{ id: "today" | "project" | "knowledge"; label: string }> = [
  { id: "today", label: "Today" },
  { id: "project", label: "Projects" },
  { id: "knowledge", label: "Knowledge" },
];

const SETUP_STEPS: ReadonlyArray<{
  id: SetupStep;
  label: string;
  description: string;
}> = [
  {
    id: "understand",
    label: "Business",
    description: "Understand the Owner’s business, audience, constraints, and definition of useful work.",
  },
  {
    id: "research",
    label: "Research",
    description: "Run broad, source-backed research; expose coverage, conflicts, rights, and freshness.",
  },
  {
    id: "contract",
    label: "Outcome contract",
    description: "Define the primary goal, openable deliverables, acceptance checks, and evidence basis.",
  },
  {
    id: "team",
    label: "Team",
    description: "Specialize the built-in Project Manager and propose researched Role Runtime Templates.",
  },
  {
    id: "routine",
    label: "Work cycle",
    description: "Define workflow, handoffs, triggers, no-overlap, queue-latest, and missed-run behavior.",
  },
  {
    id: "models",
    label: "Models",
    description: "Choose each Member’s Provider and model explicitly; add reviewed Skills and capabilities.",
  },
  {
    id: "permissions",
    label: "Permissions",
    description: "Set Tool, MCP, network, external-action, and human-review boundaries in plain language.",
  },
  {
    id: "simulate",
    label: "Simulate",
    description: "Walk one complete cycle without external mutation and surface missing inputs or unsafe paths.",
  },
  {
    id: "preview",
    label: "Launch preview",
    description: "Review the structured revision. Only a future daemon-issued exact preview may be confirmed.",
  },
];

const MEMBERS: readonly Member[] = [
  {
    id: "manager",
    name: "Lin",
    role: "Project Manager",
    roleSource: "Built-in base Role · specialized for this Project",
    responsibility: "Deliver three reviewable X content packages each week and keep the cycle governed.",
    currentWork: "Checking the publication package against the output contract.",
    nextRun: "Daily reflection after Owner review.",
    acceptedResult: "Weekly topic brief · prototype evidence sample",
    block: "Qualified X connector is unavailable.",
    cost: "Estimated ¥18.40 · source: mock Provider estimate",
    ownerNeed: "Review the publication package; no real publish action exists.",
    provider: "Anthropic · prototype selection",
    model: "Claude Sonnet · prototype label",
    workInstruction: "Plan, delegate, verify evidence, surface uncertainty, and escalate boundary changes.",
    skills: "Project planning · editorial quality review · reflection",
    tools: "Read-only research · artifact comparison",
    mcp: "None granted",
    loop: "Plan → delegate → verify → brief → reflect",
    memory: "Project-governance decisions only",
    context: "Task contract first; fixed decisions; sourced artifacts; summaries; older narrative",
    permissions: "No external dispatch; no team/model/tool changes without Owner preview",
    harness: "Managed member engine · hidden in daily UI",
  },
  {
    id: "researcher",
    name: "Mei",
    role: "Audience Researcher",
    roleSource: "Assistant-researched Role candidate · source review partial",
    responsibility: "Produce source-backed audience tensions and topic candidates.",
    currentWork: "Reconciling two conflicting claims about developer adoption.",
    nextRun: "Monday 09:00, if the Windows host is online.",
    acceptedResult: "Audience tension memo · 6 sources · mock",
    block: "One source is stale; confidence is not numerically calibrated.",
    cost: "Actual unknown · Provider usage source unavailable",
    ownerNeed: "None now; stale evidence remains visible.",
    provider: "Google · prototype selection",
    model: "Gemini Pro · prototype label",
    workInstruction: "Prefer primary sources, label inference, retain conflicts, and never invent market facts.",
    skills: "Web research · source triage · synthesis",
    tools: "Search candidate · source reader",
    mcp: "Research connector candidate · permission not granted",
    loop: "Question → collect → triangulate → synthesize → handoff",
    memory: "Admitted audience facts with provenance",
    context: "Current research task; fixed audience; source excerpts; sourced summaries",
    permissions: "Read-only network candidate; no executable MCP grant",
    harness: "Managed member engine · hidden in daily UI",
  },
  {
    id: "editor",
    name: "Rui",
    role: "Content Editor",
    roleSource: "Assistant-researched Role candidate · reviewed mock",
    responsibility: "Turn accepted briefs into openable draft, visual brief, and publication package.",
    currentWork: "Revising Outcome B after accessibility copy review.",
    nextRun: "After Mei’s evidence handoff.",
    acceptedResult: "Draft thread + visual brief · openability check passed in mock",
    block: "Asset C is missing source-rights confirmation.",
    cost: "Estimated ¥11.20 · source: mock Provider estimate",
    ownerNeed: "Confirm whether to exclude the unlicensed reference.",
    provider: "OpenAI · prototype selection",
    model: "GPT reasoning model · prototype label",
    workInstruction: "Respect the output contract, preserve citations, and stop before external publication.",
    skills: "Editorial drafting · content adaptation · accessibility copy review",
    tools: "Vault read · artifact writer candidate",
    mcp: "X connector not granted",
    loop: "Brief → draft → self-check → independent check → handoff",
    memory: "Approved voice decisions; no unreviewed summaries",
    context: "Current content contract; fixed brand decisions; accepted research; older examples",
    permissions: "Project Vault candidate write; external publish forbidden",
    harness: "Managed member engine · hidden in daily UI",
  },
];

const ARTIFACTS: readonly Artifact[] = [
  {
    id: "outcome-a",
    title: "A · Local-first control",
    format: "7-post thread + visual brief",
    angle: "Why local authority matters when digital staff act for an OPC.",
    openability: "Mock check: both files open",
    acceptance: "6/6 editorial checks · prototype sample",
    evidence: "Audience memo excerpts 2, 4, and 6",
    metric: "Engagement: unknown · not published",
    issue: "No qualified publish connector.",
  },
  {
    id: "outcome-b",
    title: "B · Digital staff, not Agent plumbing",
    format: "Single post + annotated image brief",
    angle: "Lead with governed outcomes rather than runtime configuration.",
    openability: "Mock check: draft opens; image is placeholder",
    acceptance: "4/6 checks · accessibility copy revision pending",
    evidence: "Product design and Owner-approved positioning",
    metric: "Engagement: unknown · not published",
    issue: "Image alt-text revision is incomplete.",
  },
  {
    id: "outcome-c",
    title: "C · Unknown is not zero",
    format: "5-post educational thread",
    angle: "Show why honest uncertainty is an operating advantage.",
    openability: "Mock check: Markdown opens",
    acceptance: "5/6 checks · source-rights review pending",
    evidence: "Cost and Effect-state product requirements",
    metric: "Engagement: unknown · not published",
    issue: "One reference cannot be copied until rights are confirmed.",
  },
];

const AD_HOC_BOARD: readonly BoardPanel[] = [
  {
    id: "request",
    kind: "summary",
    title: "Owner request",
    summary: "Compare this week’s three content outcomes by promise, openability, acceptance, evidence, and unresolved risk.",
    facts: [
      {
        label: "Composition",
        value: "Typed built-in components",
        meta: "No generated code, eval, or invented live data",
      },
      {
        label: "Lifetime",
        value: "Temporary by default",
        meta: "Pin/save would require future daemon-backed persistence",
      },
    ],
  },
  {
    id: "comparison",
    kind: "comparison",
    title: "Outcome comparison",
    artifactIds: ["outcome-a", "outcome-b", "outcome-c"],
  },
  {
    id: "evidence",
    kind: "evidence",
    title: "Evidence and observability",
    facts: [
      {
        label: "Openability",
        value: "3 drafts inspected in this mock",
        meta: "Prototype data; not product evidence",
      },
      {
        label: "External receipt",
        value: "None",
        meta: "Nothing has been published or dispatched",
      },
      {
        label: "Outcome metric",
        value: "Unknown",
        meta: "Unknown is deliberately not rendered as 0",
      },
    ],
  },
  {
    id: "decision",
    kind: "decision",
    title: "Recommended next decision",
    summary: "Review Outcome A’s package first. Keep B in revision and exclude C’s disputed reference until rights are resolved.",
  },
];

const LAB_STATES: ReadonlyArray<{
  id: LabState;
  label: string;
  mustSay: string;
  example: string;
}> = [
  {
    id: "empty",
    label: "Empty",
    mustSay: "Why nothing exists and the first-value path.",
    example: "No Projects yet. Begin a research-driven draft; nothing is auto-created.",
  },
  {
    id: "loading",
    label: "Loading",
    mustSay: "Exact source/work, retained safe facts, and whether leaving is safe.",
    example: "Research source 4 of 9 is being parsed; the draft remains local.",
  },
  {
    id: "partial",
    label: "Partial",
    mustSay: "What is present, what is missing, and coverage.",
    example: "Two deliverables are openable; the visual asset is still missing.",
  },
  {
    id: "stale",
    label: "Stale",
    mustSay: "Last-known time, affected decisions, and refresh path.",
    example: "Provider catalog was last observed 18 hours ago; model selection is not confirmable.",
  },
  {
    id: "permission",
    label: "Permission",
    mustSay: "Exact scope, benefit, consequence, and deny/narrow route.",
    example: "MCP requests outbound access only to api.x.com; SecretStore access remains separate.",
  },
  {
    id: "error",
    label: "Error",
    mustSay: "Failed object/stage, retained work, retry safety, and next step.",
    example: "Source parsing failed; the original file and destination choice are retained.",
  },
  {
    id: "unknown",
    label: "Unknown",
    mustSay: "Why no conclusion is safe and why retry may be blocked.",
    example: "Dispatch was recorded but no terminal observation exists. Reconcile; do not redispatch.",
  },
  {
    id: "offline",
    label: "Offline",
    mustSay: "Host/dependency state, retained work, and online limitation.",
    example: "The Windows host was off. Scheduled publication did not run.",
  },
  {
    id: "missed",
    label: "Missed",
    mustSay: "Occurrence, reason, denominator, and risk-based catch-up.",
    example: "2 of 3 scheduled occurrences were missed; only the latest research run may queue.",
  },
  {
    id: "running",
    label: "Running",
    mustSay: "Durable step, responsible Member, outputs, and real controls only.",
    example: "Research is collecting primary sources; cancellation support is Requires-backend.",
  },
  {
    id: "success",
    label: "Success",
    mustSay: "Changed object, verified basis, and next action.",
    example: "Prototype example only: an output contract passed mock checks; no daemon receipt exists.",
  },
  {
    id: "archived",
    label: "Archived",
    mustSay: "Stopped triggers plus read, export, restore, and deletion truth.",
    example: "Triggers are stopped. Same-disk restore points are not disaster backup.",
  },
  {
    id: "requires-backend",
    label: "Requires-backend",
    mustSay: "Missing capability and dependency, with no executable false affordance.",
    example: "Project activation needs daemon-owned Project authority and an exact revision preview.",
  },
];

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

function CapabilityGap({
  children,
  environment = false,
}: {
  children: string;
  environment?: boolean;
}) {
  return (
    <aside className="capability-gap" aria-label={environment ? "Requires environment" : "Requires backend"}>
      <strong>{environment ? "Requires-environment" : "Requires-backend"}</strong>
      <span>{children}</span>
    </aside>
  );
}

function StateCallout({
  title,
  children,
  tone = "warn",
}: {
  title: string;
  children: string;
  tone?: Tone;
}) {
  return (
    <aside className="state-callout" data-tone={tone} role={tone === "bad" ? "alert" : "note"}>
      <strong>{title}</strong>
      <span>{children}</span>
    </aside>
  );
}

function SectionHeading({
  title,
  meta,
  action,
}: {
  title: string;
  meta?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
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

function SummaryPanel({ panel }: PanelProps) {
  return (
    <section className="board-panel">
      <SectionHeading title={panel.title} meta={panel.summary} />
      <dl className="fact-list">
        {(panel.facts ?? []).map((fact) => (
          <div key={fact.label}>
            <dt>{fact.label}</dt>
            <dd>
              <strong>{fact.value}</strong>
              <small>{fact.meta}</small>
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function ComparisonPanel({
  panel,
  selectedArtifact,
  setSelectedArtifact,
}: PanelProps) {
  const visibleArtifacts = ARTIFACTS.filter((artifact) =>
    (panel.artifactIds ?? []).includes(artifact.id),
  );
  const selected =
    visibleArtifacts.find((artifact) => artifact.id === selectedArtifact) ??
    visibleArtifacts[0];

  return (
    <section className="board-panel board-span">
      <SectionHeading
        title={panel.title}
        meta="Select a row to inspect the real mock object used by this declarative board."
      />
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th scope="col">Outcome</th>
              <th scope="col">Openability</th>
              <th scope="col">Acceptance</th>
              <th scope="col">External metric</th>
              <th scope="col">Inspect</th>
            </tr>
          </thead>
          <tbody>
            {visibleArtifacts.map((artifact) => (
              <tr key={artifact.id} data-selected={artifact.id === selected?.id}>
                <th scope="row">{artifact.title}</th>
                <td>{artifact.openability}</td>
                <td>{artifact.acceptance}</td>
                <td>{artifact.metric}</td>
                <td>
                  <button
                    className="inline-button"
                    type="button"
                    aria-pressed={artifact.id === selected?.id}
                    onClick={() => setSelectedArtifact(artifact.id)}
                  >
                    Inspect
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {selected ? (
        <div className="artifact-inspector" aria-live="polite">
          <div>
            <span>Promise</span>
            <strong>{selected.angle}</strong>
          </div>
          <div>
            <span>Evidence basis</span>
            <strong>{selected.evidence}</strong>
          </div>
          <div>
            <span>Unresolved</span>
            <strong>{selected.issue}</strong>
          </div>
        </div>
      ) : null}
    </section>
  );
}

function EvidencePanel({ panel }: PanelProps) {
  return (
    <section className="board-panel">
      <SectionHeading title={panel.title} />
      <dl className="fact-list compact">
        {(panel.facts ?? []).map((fact) => (
          <div key={fact.label}>
            <dt>{fact.label}</dt>
            <dd>
              <strong>{fact.value}</strong>
              <small>{fact.meta}</small>
            </dd>
          </div>
        ))}
      </dl>
      <StateCallout title="Non-happy state · incomplete evidence">
        Outcome metrics remain unknown and Outcome C has a rights block. Neither is coerced into success.
      </StateCallout>
    </section>
  );
}

function DecisionPanel({ panel }: PanelProps) {
  return (
    <section className="board-panel">
      <SectionHeading title={panel.title} />
      <p className="decision-copy">{panel.summary}</p>
      <p className="fine-print">
        Assistant recommendation · candidate only. It cannot revise the Project, approve publication, or prove completion.
      </p>
    </section>
  );
}

const PANEL_REGISTRY = {
  summary: SummaryPanel,
  comparison: ComparisonPanel,
  evidence: EvidencePanel,
  decision: DecisionPanel,
};

function TypedBoard({
  selectedArtifact,
  setSelectedArtifact,
}: {
  selectedArtifact: string;
  setSelectedArtifact: (value: string) => void;
}) {
  return (
    <div className="typed-board">
      {AD_HOC_BOARD.map((panel) => {
        const Renderer = PANEL_REGISTRY[panel.kind];
        return (
          <Renderer
            key={panel.id}
            panel={panel}
            selectedArtifact={selectedArtifact}
            setSelectedArtifact={setSelectedArtifact}
          />
        );
      })}
    </div>
  );
}

function TodayScene({
  setScene,
}: {
  setScene: (value: Scene) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="primary-decision">
        <div>
          <Tag tone="warn">Needs Owner · prototype</Tag>
          <h2>Review one publication package; two outcomes can continue without you.</h2>
          <p>
            X content operation is waiting on a structured package review. Nothing is published, and no receipt exists.
          </p>
        </div>
        <button className="primary-button" type="button" onClick={() => setScene("recovery")}>
          Open review context
        </button>
      </section>

      <StateCallout title="Missed while the Windows host was off">
        Two scheduled occurrences did not run. Queue-latest may retain only the newest low-risk research occurrence; publication needs fresh review.
      </StateCallout>

      <div className="two-column">
        <section className="work-surface">
          <SectionHeading
            title="Expected outputs today"
            meta="Deliverable and acceptance first; execution state second."
          />
          <div className="outcome-list">
            <article>
              <div>
                <strong>Publication package A</strong>
                <p>Thread, visual brief, citations, alt text, and preflight summary.</p>
              </div>
              <div className="row-meta">
                <Tag tone="warn">Owner review</Tag>
                <span>Next: inspect package</span>
              </div>
            </article>
            <article>
              <div>
                <strong>Outcome B accessibility revision</strong>
                <p>Replace ambiguous image copy and rerun the mock editorial checklist.</p>
              </div>
              <div className="row-meta">
                <Tag tone="info">In revision</Tag>
                <span>Rui · next after evidence handoff</span>
              </div>
            </article>
            <article>
              <div>
                <strong>Audience source reconciliation</strong>
                <p>Retain the disagreement between two sources instead of averaging it away.</p>
              </div>
              <div className="row-meta">
                <Tag tone="warn">Partial</Tag>
                <span>Mei · one source stale</span>
              </div>
            </article>
          </div>
        </section>

        <section className="work-surface">
          <SectionHeading
            title="Latest accepted outcome"
            meta="A design sample of the evidence treatment—not a current product receipt."
          />
          <div className="accepted-outcome">
            <strong>Weekly topic brief</strong>
            <p>Six source-backed tensions, three selected topics, and a linked handoff contract.</p>
            <dl className="mini-facts">
              <div>
                <dt>Openability</dt>
                <dd>Mock check passed</dd>
              </div>
              <div>
                <dt>Completion basis</dt>
                <dd>Prototype independent-check example</dd>
              </div>
              <div>
                <dt>Cost</dt>
                <dd>Actual unknown · not ¥0</dd>
              </div>
            </dl>
          </div>
          <CapabilityGap>
            Today needs daemon-backed Projects, Routines, conversations, evidence projection, and missed-run facts.
          </CapabilityGap>
        </section>
      </div>

      <section className="work-surface">
        <SectionHeading
          title="Projects"
          meta="Long-lived governed workspaces, ordered by the next consequential decision."
          action={{ label: "Open X project", onClick: () => setScene("project") }}
        />
        <button className="project-row" type="button" onClick={() => setScene("project")}>
          <span>
            <strong>X content operation</strong>
            <small>Goal: three accepted content packages each week</small>
          </span>
          <span>
            <strong>1 decision</strong>
            <small>Connector unavailable · cost partly unknown</small>
          </span>
        </button>
      </section>
    </div>
  );
}

function ProjectTabs({
  surface,
  setSurface,
  setScene,
}: {
  surface: ProjectSurface;
  setSurface: (value: ProjectSurface) => void;
  setScene: (value: Scene) => void;
}) {
  return (
    <nav className="context-tabs" aria-label="X project canvas regions">
      <button
        type="button"
        aria-current={surface === "brief" ? "page" : undefined}
        onClick={() => setSurface("brief")}
      >
        Operating brief
      </button>
      <button
        type="button"
        aria-current={surface === "team" ? "page" : undefined}
        onClick={() => setSurface("team")}
      >
        Team
      </button>
      <button
        type="button"
        aria-current={surface === "attention" ? "page" : undefined}
        onClick={() => setSurface("attention")}
      >
        Needs Owner
      </button>
      <button type="button" onClick={() => setScene("adhoc")}>
        Temporary canvas
      </button>
    </nav>
  );
}

function OperatingBrief({
  setScene,
}: {
  setScene: (value: Scene) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="project-outcome">
        <div>
          <Tag tone="info">Stable operating brief · prototype</Tag>
          <h2>Three accepted X content packages per week, each openable and source-backed.</h2>
          <p>
            Current cycle: one package ready for Owner review, one in revision, one blocked on source rights.
          </p>
        </div>
        <button className="secondary-button" type="button" onClick={() => setScene("adhoc")}>
          Compare outcomes
        </button>
      </section>

      <div className="brief-grid">
        <section className="work-surface brief-main">
          <SectionHeading
            title="Accepted deliverables"
            meta="Stable template section · outcomes remain linked to evidence and unresolved facts."
          />
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th scope="col">Deliverable</th>
                  <th scope="col">Contract</th>
                  <th scope="col">Acceptance</th>
                  <th scope="col">Next</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <th scope="row">Weekly topic brief</th>
                  <td>6 sources · 3 topics · handoff</td>
                  <td><Tag tone="good">Mock accepted</Tag></td>
                  <td>Draft package</td>
                </tr>
                <tr>
                  <th scope="row">Package A</th>
                  <td>Thread · visual · citations · alt text</td>
                  <td><Tag tone="warn">Review needed</Tag></td>
                  <td>Owner review</td>
                </tr>
                <tr>
                  <th scope="row">Package C</th>
                  <td>Thread · sources · rights-safe reuse</td>
                  <td><Tag tone="bad">Blocked</Tag></td>
                  <td>Exclude disputed source</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <section className="work-surface">
          <SectionHeading title="Manager brief" meta="What changed, why, and what remains uncertain." />
          <p className="manager-brief">
            “A is ready for review. B needs one accessibility revision. C should not advance until source rights are resolved.”
          </p>
          <dl className="mini-facts">
            <div>
              <dt>Plan revision</dt>
              <dd>Prototype label only</dd>
            </div>
            <div>
              <dt>External Effect</dt>
              <dd>Unknown sample · excluded from completion</dd>
            </div>
            <div>
              <dt>Cost</dt>
              <dd>¥29.60 estimated + actual unknown</dd>
            </div>
          </dl>
        </section>

        <section className="work-surface">
          <SectionHeading
            title="Team now"
            meta="Responsibility and latest outcome; Runtime stays deeper."
            action={{ label: "Open team canvas", onClick: () => setScene("runtime") }}
          />
          <ul className="compact-list">
            <li><strong>Lin · Manager</strong><span>Package review · connector blocked</span></li>
            <li><strong>Mei · Research</strong><span>Source conflict · actual cost unknown</span></li>
            <li><strong>Rui · Editor</strong><span>Accessibility revision · rights decision needed</span></li>
          </ul>
        </section>

        <section className="work-surface">
          <SectionHeading
            title="Needs Owner"
            meta="Contextual region—not a permanent Inbox destination."
            action={{ label: "Open attention canvas", onClick: () => setScene("recovery") }}
          />
          <ul className="decision-list">
            <li><span>1</span><div><strong>Review package A</strong><small>No dispatch or receipt</small></div></li>
            <li><span>2</span><div><strong>Resolve source rights</strong><small>Exclude or provide licensed source</small></div></li>
            <li><span>3</span><div><strong>Reconcile unknown Effect</strong><small>Retry is unsafe</small></div></li>
          </ul>
        </section>
      </div>

      <section className="work-surface">
        <SectionHeading
          title="X operating loop"
          meta="Every stage retains its own evidence state; a Provider response or connector receipt alone never proves completion."
        />
        <ol className="operating-loop">
          <li data-state="done"><span>1</span><strong>Research</strong><small>Mock accepted brief</small></li>
          <li data-state="done"><span>2</span><strong>Topic</strong><small>3 candidates selected</small></li>
          <li data-state="working"><span>3</span><strong>Draft + media</strong><small>1 revision pending</small></li>
          <li data-state="waiting"><span>4</span><strong>Package review</strong><small>Needs Owner</small></li>
          <li data-state="blocked"><span>5</span><strong>Qualified dispatch</strong><small>Connector unavailable</small></li>
          <li data-state="empty"><span>6</span><strong>Receipt</strong><small>None · not published</small></li>
          <li data-state="empty"><span>7</span><strong>Readback</strong><small>Not run</small></li>
          <li data-state="waiting"><span>8</span><strong>Reflection</strong><small>Waits for evidence</small></li>
          <li data-state="waiting"><span>9</span><strong>Next cycle</strong><small>Not revised yet</small></li>
        </ol>
      </section>

      <StateCallout title="Non-happy state · unknown external outcome" tone="bad">
        A prior dispatch example has no terminal observation. It is not a receipt, not success, and cannot be blindly retried.
      </StateCallout>
      <CapabilityGap>
        The stable report template needs real Project, artifact, evidence, cost, Routine, and manager-reflection projections.
      </CapabilityGap>
    </div>
  );
}

function TeamCanvas({
  setScene,
}: {
  setScene: (value: Scene) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="work-surface">
        <SectionHeading
          title="Project team"
          meta="One current manager; Project-specific Members persist beyond disposable Attempts."
        />
        <div className="member-summary-grid">
          {MEMBERS.map((member) => (
            <article key={member.id}>
              <header>
                <div>
                  <strong>{member.name}</strong>
                  <span>{member.role}</span>
                </div>
                <Tag tone={member.id === "manager" ? "info" : "warn"}>
                  {member.id === "manager" ? "Built-in base" : "Researched"}
                </Tag>
              </header>
              <p>{member.responsibility}</p>
              <dl>
                <div><dt>Current</dt><dd>{member.currentWork}</dd></div>
                <div><dt>Next</dt><dd>{member.nextRun}</dd></div>
                <div><dt>Block</dt><dd>{member.block}</dd></div>
              </dl>
            </article>
          ))}
        </div>
        <div className="action-line">
          <button className="primary-button" type="button" onClick={() => setScene("runtime")}>
            Inspect Member Runtime
          </button>
          <span>Team changes require an Owner-confirmed revision.</span>
        </div>
      </section>
      <StateCallout title="Non-happy state · Role candidate not admitted">
        The Audience Researcher template has partial source review. It cannot silently become a global Role or receive executable capabilities.
      </StateCallout>
      <CapabilityGap>
        Role Runtime Templates, Member Runtime authority, team revisions, and disposable Attempt launch are target behavior.
      </CapabilityGap>
    </div>
  );
}

function AttentionCanvas({
  setScene,
}: {
  setScene: (value: Scene) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="work-surface">
        <SectionHeading
          title="What needs the Owner"
          meta="A Project-scoped canvas region with reasons, consequences, and safe next paths."
        />
        <div className="attention-rows">
          <button type="button" onClick={() => setScene("recovery")}>
            <span><Tag tone="warn">Approval</Tag><strong>Publication package A</strong></span>
            <small>External action · reversible only before dispatch · no connector available</small>
          </button>
          <button type="button" onClick={() => setScene("recovery")}>
            <span><Tag tone="bad">Unknown</Tag><strong>Dispatch observation missing</strong></span>
            <small>Reconcile before retry; outcome is not success</small>
          </button>
          <button type="button" onClick={() => setScene("recovery")}>
            <span><Tag tone="warn">Missed</Tag><strong>Two offline occurrences</strong></span>
            <small>Queue-latest applies only to eligible low-risk work</small>
          </button>
        </div>
      </section>
      <CapabilityGap>
        Serialized approval, Routine ledgers, unknown-Effect reconciliation, and recovery controls are Requires-backend.
      </CapabilityGap>
    </div>
  );
}

function ProjectScene({
  surface,
  setSurface,
  setScene,
}: {
  surface: ProjectSurface;
  setSurface: (value: ProjectSurface) => void;
  setScene: (value: Scene) => void;
}) {
  return (
    <div>
      <ProjectTabs surface={surface} setSurface={setSurface} setScene={setScene} />
      {surface === "brief" ? <OperatingBrief setScene={setScene} /> : null}
      {surface === "team" ? <TeamCanvas setScene={setScene} /> : null}
      {surface === "attention" ? <AttentionCanvas setScene={setScene} /> : null}
    </div>
  );
}

function AdHocScene({
  pinned,
  setPinned,
  selectedArtifact,
  setSelectedArtifact,
}: {
  pinned: boolean;
  setPinned: (value: boolean) => void;
  selectedArtifact: string;
  setSelectedArtifact: (value: string) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="temporary-canvas-head">
        <div>
          <Tag tone="info">Temporary canvas · typed registry</Tag>
          <h2>Compare this week’s three content outcomes</h2>
          <p>
            Assembled from built-in mock Artifact objects using summary, comparison, evidence, and decision components.
          </p>
        </div>
        <button
          className="secondary-button"
          type="button"
          aria-pressed={pinned}
          onClick={() => setPinned(!pinned)}
        >
          {pinned ? "Remove pin preview" : "Preview pin intent"}
        </button>
      </section>

      {pinned ? (
        <StateCallout title="Prototype pin preview · not saved" tone="info">
          This local visual state demonstrates pinning only. Project persistence and template saving are Requires-backend.
        </StateCallout>
      ) : (
        <StateCallout title="Ephemeral by default">
          This canvas disappears when the prototype state resets. No browser storage or filesystem write is used.
        </StateCallout>
      )}

      <TypedBoard
        selectedArtifact={selectedArtifact}
        setSelectedArtifact={setSelectedArtifact}
      />

      <CapabilityGap>
        Reading real Project results, composing a governed board, pinning it to a Project, and saving a template need typed backend projections and persistence.
      </CapabilityGap>
    </div>
  );
}

function SetupScene({
  step,
  setStep,
  view,
  setView,
}: {
  step: SetupStep;
  setStep: (value: SetupStep) => void;
  view: SetupView;
  setView: (value: SetupView) => void;
}) {
  const activeStep =
    SETUP_STEPS.find((candidate) => candidate.id === step) ?? SETUP_STEPS[0];

  return (
    <div className="scene-stack">
      <section className="setup-intro">
        <div>
          <Tag tone="info">Research-driven session · resumable prototype</Tag>
          <h2>Start from the business outcome, then earn the operating design.</h2>
          <p>
            The Assistant researches broadly, converts findings into structured candidates, simulates one cycle, and only then prepares a launch preview.
          </p>
        </div>
        <div className="segmented" aria-label="Setup prototype state">
          <button
            type="button"
            aria-pressed={view === "research-partial"}
            onClick={() => setView("research-partial")}
          >
            Partial research
          </button>
          <button
            type="button"
            aria-pressed={view === "launch-preview"}
            onClick={() => setView("launch-preview")}
          >
            Launch preview
          </button>
        </div>
      </section>

      <nav className="setup-steps" aria-label="Project setup steps">
        {SETUP_STEPS.map((item, index) => (
          <button
            key={item.id}
            type="button"
            aria-current={step === item.id ? "step" : undefined}
            onClick={() => setStep(item.id)}
          >
            <span>{index + 1}</span>
            {item.label}
          </button>
        ))}
      </nav>

      <div className="setup-workbench">
        <section className="work-surface">
          <SectionHeading
            title={activeStep.label}
            meta={activeStep.description}
          />
          {view === "research-partial" ? (
            <div className="research-session">
              <div className="assistant-note">
                <strong>Personal Assistant · candidate</strong>
                <p>
                  “I understand the goal as building a repeatable X content operation that delivers three reviewable packages per week. I found nine sources; two conflict and one has unclear reuse rights.”
                </p>
              </div>
              <dl className="fact-list compact">
                <div>
                  <dt>Business context</dt>
                  <dd><strong>OPC developer product</strong><small>Owner-confirmed mock input</small></dd>
                </div>
                <div>
                  <dt>Source coverage</dt>
                  <dd><strong>6 usable · 2 conflicting · 1 rights unknown</strong><small>Prototype research ledger</small></dd>
                </div>
                <div>
                  <dt>Open question</dt>
                  <dd><strong>Which audience tension should lead cycle one?</strong><small>Owner input can be deferred until preview</small></dd>
                </div>
              </dl>
              <StateCallout title="Non-happy state · research conflict">
                Two sources disagree on adoption. The conflict is retained and excluded from factual Memory until resolved.
              </StateCallout>
            </div>
          ) : (
            <div className="launch-preview">
              <Tag tone="warn">Candidate preview · not confirmable</Tag>
              <dl className="fact-list">
                <div><dt>Primary goal</dt><dd><strong>3 accepted packages / week</strong><small>No outcome guarantee</small></dd></div>
                <div><dt>Outputs</dt><dd><strong>Research brief · drafts · media brief · publication package · reflection</strong><small>Openable plus acceptance contract</small></dd></div>
                <div><dt>Team</dt><dd><strong>Lin manager · Mei researcher · Rui editor</strong><small>Only base Project Manager Role is built in</small></dd></div>
                <div><dt>Triggers</dt><dd><strong>Manual · schedule · accepted artifact · Project state · qualified external event · testable data condition</strong><small>No-overlap + queue-latest; external/data triggers remain capability-gated</small></dd></div>
                <div><dt>External actions</dt><dd><strong>Draft and wait</strong><small>Publication requires exact Owner review</small></dd></div>
                <div><dt>Simulation</dt><dd><strong>1 mock cycle · 2 gaps found</strong><small>Connector and rights review</small></dd></div>
              </dl>
              <StateCallout title="No Project ID or activation receipt">
                This Canvas cannot issue an authority revision, activate a Project, or produce a receipt.
              </StateCallout>
            </div>
          )}
        </section>

        <aside className="setup-outline">
          <h3>Activation contract</h3>
          <ol>
            <li><strong>Outcome</strong><span>Business goal and acceptance measure</span></li>
            <li><strong>Deliverables</strong><span>Openability, evidence, and handoff</span></li>
            <li><strong>People</strong><span>Role Template, Member, Provider/model</span></li>
            <li><strong>Work cycle</strong><span>Triggers, no overlap, queue-latest</span></li>
            <li><strong>Boundaries</strong><span>Permissions, HITL, external rules</span></li>
            <li><strong>Simulation</strong><span>One dry cycle with gaps retained</span></li>
          </ol>
          <CapabilityGap>
            Research orchestration, Project draft custody, simulation, structured preview, and exact activation are not implemented.
          </CapabilityGap>
        </aside>
      </div>
    </div>
  );
}

function RuntimeScene({
  selectedMember,
  setSelectedMember,
  showAdvanced,
  setShowAdvanced,
  showDiagnostics,
  setShowDiagnostics,
}: {
  selectedMember: MemberId;
  setSelectedMember: (value: MemberId) => void;
  showAdvanced: boolean;
  setShowAdvanced: (value: boolean) => void;
  showDiagnostics: boolean;
  setShowDiagnostics: (value: boolean) => void;
}) {
  const member =
    MEMBERS.find((candidate) => candidate.id === selectedMember) ?? MEMBERS[0];

  return (
    <div className="scene-stack">
      <section className="object-chain" aria-label="Role and execution object chain">
        <div><strong>Role Runtime Template</strong><span>Reusable, versioned recipe</span></div>
        <span aria-hidden="true">→</span>
        <div><strong>Member Runtime</strong><span>Long-lived in this Project</span></div>
        <span aria-hidden="true">→</span>
        <div><strong>Task</strong><span>Daemon-owned contract</span></div>
        <span aria-hidden="true">→</span>
        <div><strong>Agent process / Attempt</strong><span>Disposable execution</span></div>
      </section>

      <StateCallout title="Non-happy state · Role generation is partial">
        Only Project Manager is built in. The researcher Role candidate still has incomplete source and capability review.
      </StateCallout>

      <div className="runtime-layout">
        <aside className="member-picker" aria-label="Project members">
          <h3>Members</h3>
          {MEMBERS.map((candidate) => (
            <button
              key={candidate.id}
              type="button"
              aria-current={candidate.id === member.id ? "true" : undefined}
              onClick={() => setSelectedMember(candidate.id)}
            >
              <span><strong>{candidate.name}</strong><small>{candidate.role}</small></span>
              <Tag tone={candidate.block.includes("unavailable") || candidate.block.includes("missing") ? "bad" : "warn"}>
                {candidate.id === "manager" ? "Manager" : "Member"}
              </Tag>
            </button>
          ))}
        </aside>

        <section className="work-surface runtime-detail">
          <SectionHeading
            title={`${member.name} · ${member.role}`}
            meta={member.roleSource}
          />
          <dl className="member-defaults">
            <div><dt>Responsibility</dt><dd>{member.responsibility}</dd></div>
            <div><dt>Current work</dt><dd>{member.currentWork}</dd></div>
            <div><dt>Next execution</dt><dd>{member.nextRun}</dd></div>
            <div><dt>Latest accepted result</dt><dd>{member.acceptedResult}</dd></div>
            <div><dt>Block</dt><dd>{member.block}</dd></div>
            <div><dt>Cost</dt><dd>{member.cost}</dd></div>
            <div><dt>Needs Owner</dt><dd>{member.ownerNeed}</dd></div>
          </dl>

          <div className="runtime-actions">
            <button
              className="secondary-button"
              type="button"
              aria-expanded={showAdvanced}
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              {showAdvanced ? "Hide advanced Runtime" : "Show advanced Runtime"}
            </button>
            <span>Business facts remain visible while advanced mechanics disclose.</span>
          </div>

          {showAdvanced ? (
            <div className="advanced-runtime">
              <h3>Runtime recipe · prototype projection</h3>
              <dl className="runtime-grid">
                <div><dt>Work instruction</dt><dd>{member.workInstruction}</dd></div>
                <div><dt>Skills</dt><dd>{member.skills}</dd></div>
                <div><dt>Tools</dt><dd>{member.tools}</dd></div>
                <div><dt>MCP</dt><dd>{member.mcp}</dd></div>
                <div><dt>Loop</dt><dd>{member.loop}</dd></div>
                <div><dt>Provider / model</dt><dd>{member.provider} / {member.model}</dd></div>
                <div><dt>Memory</dt><dd>{member.memory}</dd></div>
                <div><dt>Context</dt><dd>{member.context}</dd></div>
                <div><dt>Permissions</dt><dd>{member.permissions}</dd></div>
                <div><dt>Harness</dt><dd>{member.harness}</dd></div>
              </dl>
              <StateCallout title="Persistent change requires a revision">
                A manager may tune bounded Tasks, order, frequency, responsibility, and subgoals. Team, Provider/model, tools/MCP, permissions, external rules, and global Role Templates require Owner confirmation.
              </StateCallout>
              <button
                className="text-button"
                type="button"
                aria-expanded={showDiagnostics}
                onClick={() => setShowDiagnostics(!showDiagnostics)}
              >
                {showDiagnostics ? "Hide engine diagnostics" : "Open fault diagnostics"}
              </button>
              {showDiagnostics ? (
                <div className="diagnostics">
                  <strong>Managed engine identities · advanced fault view</strong>
                  <dl className="mini-facts">
                    <div><dt>Member engine</dt><dd>DSH · hidden during ordinary work</dd></div>
                    <div><dt>Assistant engine</dt><dd>Pi · hidden outside advanced Assistant fault diagnosis · candidate-only</dd></div>
                    <div><dt>Version health</dt><dd>Unknown · Requires-backend + environment qualification</dd></div>
                    <div><dt>Update / rollback</dt><dd>No executable control in this prototype</dd></div>
                  </dl>
                </div>
              ) : null}
            </div>
          ) : null}
          <CapabilityGap>
            Role/Member authority, versioned Runtime validation, rollback, Provider binding, and Attempt launch are target-only.
          </CapabilityGap>
        </section>
      </div>
    </div>
  );
}

function KnowledgeScene({
  tab,
  setTab,
  memoryPreview,
  setMemoryPreview,
}: {
  tab: KnowledgeTab;
  setTab: (value: KnowledgeTab) => void;
  memoryPreview: MemoryPreview;
  setMemoryPreview: (value: MemoryPreview) => void;
}) {
  return (
    <div className="scene-stack">
      <div className="section-tabs" aria-label="Knowledge views">
        <button type="button" aria-pressed={tab === "vault"} onClick={() => setTab("vault")}>Project Vault</button>
        <button type="button" aria-pressed={tab === "memory"} onClick={() => setTab("memory")}>Member Memory</button>
        <button type="button" aria-pressed={tab === "context"} onClick={() => setTab("context")}>Context assembly</button>
      </div>

      {tab === "vault" ? (
        <section className="work-surface">
          <SectionHeading
            title="X content operation Vault"
            meta="Ordinary Markdown, stable relative links, and Obsidian-compatible—not an embedded Obsidian product."
          />
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th scope="col">Source</th>
                  <th scope="col">Scope</th>
                  <th scope="col">Rights</th>
                  <th scope="col">Index</th>
                  <th scope="col">Freshness</th>
                </tr>
              </thead>
              <tbody>
                <tr><th scope="row">audience-tensions.md</th><td>Project</td><td>Owner-authored</td><td>Indexed</td><td>18 min</td></tr>
                <tr><th scope="row">source-notes/research-06.md</th><td>Project</td><td>Analyze + cite</td><td>Stale</td><td>18 h</td></tr>
                <tr><th scope="row">reference-image.png</th><td>Project</td><td>Unknown</td><td>Excluded</td><td>Not indexed</td></tr>
              </tbody>
            </table>
          </div>
          <StateCallout title="Non-happy state · secret-like content detected" tone="bad">
            A candidate import resembles credential material. It is excluded from Knowledge and must route to a future approved SecretStore handoff.
          </StateCallout>
          <CapabilityGap>
            Vault custody, import, rights review, parsing, indexing, conflict handling, and SecretStore routing are Requires-backend.
          </CapabilityGap>
        </section>
      ) : null}

      {tab === "memory" ? (
        <section className="work-surface">
          <SectionHeading
            title="Mei · admitted Memory"
            meta="Full source records remain local; summaries do not prove completion or self-admit."
          />
          <div className="memory-card">
            <header>
              <div><strong>Audience prefers concrete operating evidence</strong><span>Project-scoped · candidate conflict attached</span></div>
              <Tag tone="warn">Needs correction review</Tag>
            </header>
            <p>
              A sourced summary currently overstates agreement across six observations. The original excerpts remain inspectable.
            </p>
            <dl className="mini-facts">
              <div><dt>Source</dt><dd>3 admitted excerpts + 1 manager reflection</dd></div>
              <div><dt>Use scope</dt><dd>Topic selection only</dd></div>
              <div><dt>Conflict</dt><dd>One stale source disagrees</dd></div>
            </dl>
            <div className="segmented" aria-label="Memory prototype preview">
              {(["inspect", "correct", "forget"] as MemoryPreview[]).map((value) => (
                <button
                  key={value}
                  type="button"
                  aria-pressed={memoryPreview === value}
                  onClick={() => setMemoryPreview(value)}
                >
                  Preview {value}
                </button>
              ))}
            </div>
            <StateCallout title={`Prototype ${memoryPreview} view · no mutation`} tone="info">
              {memoryPreview === "inspect"
                ? "Shows provenance, exact source excerpts, versions, scope, and conflict without exposing hidden reasoning."
                : memoryPreview === "correct"
                  ? "Would preserve the old version, record the Owner correction, and re-evaluate dependent Context."
                  : "Would preview affected retrieval and create a durable forget/tombstone result; this prototype does neither."}
            </StateCallout>
          </div>
          <CapabilityGap>
            Conversation extraction, semantic admission, correction, conflict propagation, and forget are backend-owned operations.
          </CapabilityGap>
        </section>
      ) : null}

      {tab === "context" ? (
        <section className="work-surface">
          <SectionHeading
            title="Model-window-aware Context package"
            meta="The source archive remains intact; each disposable Attempt receives only an authorized bounded package."
          />
          <ol className="context-ladder">
            <li><span>1</span><div><strong>Current Task contract</strong><small>Never displaced by summary</small></div></li>
            <li><span>2</span><div><strong>Fixed decisions</strong><small>Owner-confirmed boundaries and output contract</small></div></li>
            <li><span>3</span><div><strong>Relevant source and artifact excerpts</strong><small>Scope, provenance, freshness, untrusted labels</small></div></li>
            <li><span>4</span><div><strong>Sourced summaries</strong><small>Loss and conflict explicitly stated</small></div></li>
            <li><span>5</span><div><strong>Older narrative</strong><small>Included only if budget remains</small></div></li>
          </ol>
          <StateCallout title="Non-happy state · stale index">
            One source index is 18 hours old. Its summary is labelled stale and cannot silently outrank the current Task contract.
          </StateCallout>
          <CapabilityGap>
            Scope authorization, redaction, retrieval, ranking, bounded assembly, and source inspection require backend support.
          </CapabilityGap>
        </section>
      ) : null}
    </div>
  );
}

function ConnectionsScene({
  mode,
  setMode,
  provider,
  setProvider,
  modelChoice,
  setModelChoice,
}: {
  mode: ConnectionMode;
  setMode: (value: ConnectionMode) => void;
  provider: ProviderId;
  setProvider: (value: ProviderId) => void;
  modelChoice: string;
  setModelChoice: (value: string) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="settings-head">
        <div>
          <Tag tone="info">Settings · secondary destination</Tag>
          <h2>Connect models without turning the product into account administration.</h2>
          <p>Each Member explicitly selects one admitted Provider and model. Secrets never return to this UI.</p>
        </div>
        <div className="segmented" aria-label="Connection mode">
          <button type="button" aria-pressed={mode === "quick"} onClick={() => setMode("quick")}>Quick</button>
          <button type="button" aria-pressed={mode === "custom"} onClick={() => setMode("custom")}>Custom</button>
        </div>
      </section>

      {mode === "quick" ? (
        <div className="connection-layout">
          <section className="work-surface">
            <SectionHeading
              title="Quick connection"
              meta="Choose a mainstream Provider, then hand a key directly to a future daemon-owned non-logging path."
            />
            <div className="provider-choice" aria-label="Provider prototype selection">
              {([
                ["anthropic", "Anthropic"],
                ["openai", "OpenAI"],
                ["google", "Google"],
              ] as Array<[ProviderId, string]>).map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  aria-pressed={provider === id}
                  onClick={() => setProvider(id)}
                >
                  <strong>{label}</strong>
                  <small>Prototype route · catalog unchecked</small>
                </button>
              ))}
            </div>
            <label className="field">
              <span>Provider API key</span>
              <input
                type="password"
                name="quick-provider-key-placeholder"
                autoComplete="off"
                spellCheck={false}
                value=""
                readOnly
                placeholder="Prototype never accepts or stores a key…"
                aria-describedby="quick-key-help"
              />
              <small id="quick-key-help">Empty, read-only placeholder. Never paste a real secret into this Canvas.</small>
            </label>
            <CapabilityGap>
              SecretStore handoff, endpoint validation, catalog discovery, and connection admission are unavailable; no Connect button is shown.
            </CapabilityGap>
          </section>

          <section className="work-surface">
            <SectionHeading
              title="Explicit Member binding"
              meta="A Role Template never carries credentials or a Provider binding."
            />
            <label className="field">
              <span>Member</span>
              <input name="member-binding" autoComplete="off" type="text" value="Mei · Audience Researcher" readOnly />
            </label>
            <label className="field">
              <span>Provider</span>
              <input name="provider-binding" autoComplete="off" type="text" value={provider === "anthropic" ? "Anthropic" : provider === "openai" ? "OpenAI" : "Google"} readOnly />
            </label>
            <label className="field">
              <span>Model · prototype choice</span>
              <select
                name="model-binding"
                value={modelChoice}
                onChange={(event: { target: { value: string } }) => setModelChoice(event.target.value)}
              >
                <option value="unselected">Choose after a fresh catalog check</option>
                <option value="balanced">Balanced model · mock label</option>
                <option value="research">Research-capable model · mock label</option>
              </select>
            </label>
            <StateCallout title="Non-happy state · quota and price unknown">
              The Provider quota and current pricing version are unavailable. Cost is shown as unknown—not 0—and Personal does not auto-stop on a cost budget.
            </StateCallout>
          </section>
        </div>
      ) : (
        <section className="work-surface custom-connection">
          <SectionHeading
            title="Advanced compatible endpoint"
            meta="URL, compatibility mode, key handoff, and exact model name remain separate fields."
          />
          <div className="form-grid">
            <label className="field">
              <span>Base URL</span>
              <input name="custom-base-url" autoComplete="off" spellCheck={false} type="url" value="https://provider.example/v1" readOnly />
              <small>Example-only address; this Canvas makes no request.</small>
            </label>
            <label className="field">
              <span>Compatibility mode</span>
              <select name="compatibility-mode" defaultValue="openai-compatible">
                <option value="openai-compatible">OpenAI-compatible · prototype</option>
              </select>
            </label>
            <label className="field">
              <span>API key</span>
              <input
                name="custom-provider-key-placeholder"
                autoComplete="off"
                spellCheck={false}
                type="password"
                value=""
                readOnly
                placeholder="Never accepted in prototype…"
              />
              <small>Future one-way daemon handoff only.</small>
            </label>
            <label className="field">
              <span>Exact model name</span>
              <input name="custom-model-name" autoComplete="off" spellCheck={false} type="text" value="provider-model-name" readOnly />
              <small>Must be validated against the selected endpoint.</small>
            </label>
          </div>
          <StateCallout title="Non-happy state · endpoint not checked">
            Compatibility, TLS trust, model existence, quota, and pricing are unknown. No fallback or silent model substitution is implied.
          </StateCallout>
          <CapabilityGap>
            Custom endpoint trust, compatibility probing, SecretStore custody, model validation, and versioned Member rebinding need backend support.
          </CapabilityGap>
        </section>
      )}
    </div>
  );
}

function RecoveryScene({
  tab,
  setTab,
  decision,
  setDecision,
}: {
  tab: RecoveryTab;
  setTab: (value: RecoveryTab) => void;
  decision: string;
  setDecision: (value: string) => void;
}) {
  return (
    <div className="scene-stack">
      <nav className="section-tabs" aria-label="Project attention scenarios">
        <button type="button" aria-current={tab === "approval" ? "page" : undefined} onClick={() => setTab("approval")}>Publication review</button>
        <button type="button" aria-current={tab === "missed" ? "page" : undefined} onClick={() => setTab("missed")}>Missed work</button>
        <button type="button" aria-current={tab === "unknown" ? "page" : undefined} onClick={() => setTab("unknown")}>Unknown Effect</button>
        <button type="button" aria-current={tab === "mcp" ? "page" : undefined} onClick={() => setTab("mcp")}>MCP grant</button>
      </nav>

      {tab === "approval" ? (
        <section className="work-surface">
          <SectionHeading
            title="Publication package A"
            meta="Structured candidate preview · nothing has been dispatched."
          />
          <div className="approval-grid">
            <dl className="fact-list">
              <div><dt>Target</dt><dd><strong>X / Twitter public post</strong><small>Qualified connector unavailable</small></dd></div>
              <div><dt>Package</dt><dd><strong>7-post thread + visual brief + citations + alt text</strong><small>Mock files only</small></dd></div>
              <div><dt>External consequence</dt><dd><strong>Public communication under the Owner’s identity</strong><small>Not reversible after third-party distribution</small></dd></div>
              <div><dt>Dispatch rule</dt><dd><strong>Persist Intent/Effect before connector dispatch</strong><small>Independent verification still required</small></dd></div>
              <div><dt>Cost</dt><dd><strong>Actual unknown</strong><small>Unknown is not 0</small></dd></div>
            </dl>
            <aside className="decision-preview">
              <h3>Preview a decision intent</h3>
              <p>This changes only local prototype state; it does not approve, reject, or revise anything.</p>
              <div className="stacked-actions">
                {["Review only", "Narrow to draft export", "Reject candidate"].map((value) => (
                  <button
                    key={value}
                    type="button"
                    aria-pressed={decision === value}
                    onClick={() => setDecision(value)}
                  >
                    {value}
                  </button>
                ))}
              </div>
              <output aria-live="polite">Prototype intent: {decision}</output>
            </aside>
          </div>
          <StateCallout title="Non-happy state · publishing unavailable" tone="bad">
            There is no qualified connector or daemon-issued preview. Manual publishing is a degraded fallback, not a success receipt.
          </StateCallout>
          <CapabilityGap environment>
            A qualified X connector and Windows acceptance route are absent. The prototype cannot publish or return a receipt.
          </CapabilityGap>
        </section>
      ) : null}

      {tab === "missed" ? (
        <section className="work-surface">
          <SectionHeading
            title="Offline Routine ledger"
            meta="Same Routine never overlaps; at most the latest eligible occurrence is queued."
          />
          <div className="table-wrap">
            <table>
              <thead><tr><th scope="col">Occurrence</th><th scope="col">Observed state</th><th scope="col">Reason</th><th scope="col">Safe next path</th></tr></thead>
              <tbody>
                <tr><th scope="row">Mon 09:00 research</th><td>Missed</td><td>Host offline</td><td>Superseded by latest</td></tr>
                <tr><th scope="row">Tue 09:00 research</th><td>Latest queued candidate</td><td>Host offline</td><td>May resume after policy check</td></tr>
                <tr><th scope="row">Tue 16:00 publish</th><td>Missed</td><td>Host offline</td><td>Fresh Owner review required</td></tr>
              </tbody>
            </table>
          </div>
          <StateCallout title="Non-happy state · 2 of 3 occurrences missed">
            Queue-latest does not silently publish stale content. Consequential work waits for a fresh target and package review.
          </StateCallout>
          <CapabilityGap>
            Routine occurrence ledgers, no-overlap, queue-latest, sleep/offline facts, and risk-based catch-up are Requires-backend.
          </CapabilityGap>
        </section>
      ) : null}

      {tab === "unknown" ? (
        <section className="work-surface">
          <SectionHeading
            title="Unknown external Effect"
            meta="Persisted dispatch fact exists in this mock scenario; no terminal observation exists."
          />
          <div className="reconcile-path">
            <div><span>1</span><strong>Intent recorded</strong><small>Prototype scenario fact</small></div>
            <div><span>2</span><strong>Dispatch observed</strong><small>Destination response missing</small></div>
            <div data-current="true"><span>3</span><strong>Reconcile</strong><small>Read destination before any retry</small></div>
            <div><span>4</span><strong>Independent verification</strong><small>Not run</small></div>
          </div>
          <StateCallout title="Retry blocked" tone="bad">
            Blind redispatch could duplicate a public action. Unknown remains unknown until reconciliation produces a durable terminal fact.
          </StateCallout>
          <CapabilityGap>
            Effect identity, destination reconciliation, fencing, and independent completion verification require daemon support.
          </CapabilityGap>
        </section>
      ) : null}

      {tab === "mcp" ? (
        <section className="work-surface">
          <SectionHeading
            title="MCP capability review"
            meta="Skills may auto-install after source/prompt-injection review; executable MCP grants require exact Owner confirmation."
          />
          <dl className="permission-review">
            <div><dt>Source</dt><dd><strong>Candidate repository · provenance not yet verified</strong><small>Supply-chain review incomplete</small></dd></div>
            <div><dt>Executable scope</dt><dd><strong>One local broker process</strong><small>No arbitrary shell</small></dd></div>
            <div><dt>Network scope</dt><dd><strong>api.x.com only</strong><small>Redirects and additional domains denied</small></dd></div>
            <div><dt>Secret scope</dt><dd><strong>Opaque connector credential reference</strong><small>Raw value never exposed to Member, Canvas, or MCP process</small></dd></div>
            <div><dt>Project scope</dt><dd><strong>X content operation only</strong><small>No global Role Template grant</small></dd></div>
          </dl>
          <StateCallout title="Non-happy state · review incomplete">
            Prompt-injection and executable provenance review are incomplete. Exact Owner authorization is unavailable until a daemon preview exists.
          </StateCallout>
          <CapabilityGap>
            MCP discovery, executable/network/Secret permission review, exact grant preview, and audited admission are Requires-backend.
          </CapabilityGap>
        </section>
      ) : null}
    </div>
  );
}

function StateLabScene({
  state,
  setState,
}: {
  state: LabState;
  setState: (value: LabState) => void;
}) {
  const selected =
    LAB_STATES.find((candidate) => candidate.id === state) ?? LAB_STATES[0];

  return (
    <div className="scene-stack">
      <section className="state-lab-head">
        <div>
          <Tag tone="info">Prototype QA surface</Tag>
          <h2>State language stays stable across Projects, conversations, Vault, models, and recovery.</h2>
        </div>
        <label className="field compact-field">
          <span>Inspect state</span>
          <select
            name="state-lab-state"
            value={state}
            onChange={(event: { target: { value: string } }) => setState(event.target.value as LabState)}
          >
            {LAB_STATES.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>{candidate.label}</option>
            ))}
          </select>
        </label>
      </section>

      <div className="state-lab-grid">
        <section className="work-surface">
          <SectionHeading title={selected.label} meta={selected.mustSay} />
          <StateCallout
            title={`State example · ${selected.label}`}
            tone={selected.id === "success" ? "good" : selected.id === "loading" || selected.id === "running" ? "info" : selected.id === "error" || selected.id === "unknown" ? "bad" : "warn"}
          >
            {selected.example}
          </StateCallout>
          <dl className="mini-facts">
            <div><dt>Authority</dt><dd>{selected.id === "success" ? "Prototype evidence example only" : "No authority transition"}</dd></div>
            <div><dt>Source</dt><dd>Built-in mock object</dd></div>
            <div><dt>Freshness</dt><dd>Static prototype data</dd></div>
            <div><dt>Recovery</dt><dd>Named in plain language; executable only when backend exists</dd></div>
          </dl>
        </section>

        <section className="work-surface">
          <SectionHeading
            title="Coverage matrix"
            meta="Every scene carries at least one visible non-happy state and an honest capability boundary."
          />
          <ul className="coverage-list">
            {LAB_STATES.map((candidate) => (
              <li key={candidate.id} data-selected={candidate.id === state}>
                <button type="button" onClick={() => setState(candidate.id)}>
                  <strong>{candidate.label}</strong>
                  <span>{candidate.mustSay}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      </div>

      <CapabilityGap>
        State Lab is design evidence only. Rendered accessibility, Windows behavior, human comprehension, backend transitions, and qualified acceptance remain not-run.
      </CapabilityGap>
    </div>
  );
}

function MainScene({
  scene,
  setScene,
  projectSurface,
  setProjectSurface,
  pinned,
  setPinned,
  selectedArtifact,
  setSelectedArtifact,
  setupStep,
  setSetupStep,
  setupView,
  setSetupView,
  selectedMember,
  setSelectedMember,
  showAdvanced,
  setShowAdvanced,
  showDiagnostics,
  setShowDiagnostics,
  knowledgeTab,
  setKnowledgeTab,
  memoryPreview,
  setMemoryPreview,
  connectionMode,
  setConnectionMode,
  provider,
  setProvider,
  modelChoice,
  setModelChoice,
  recoveryTab,
  setRecoveryTab,
  decision,
  setDecision,
  labState,
  setLabState,
}: {
  scene: Scene;
  setScene: (value: Scene) => void;
  projectSurface: ProjectSurface;
  setProjectSurface: (value: ProjectSurface) => void;
  pinned: boolean;
  setPinned: (value: boolean) => void;
  selectedArtifact: string;
  setSelectedArtifact: (value: string) => void;
  setupStep: SetupStep;
  setSetupStep: (value: SetupStep) => void;
  setupView: SetupView;
  setSetupView: (value: SetupView) => void;
  selectedMember: MemberId;
  setSelectedMember: (value: MemberId) => void;
  showAdvanced: boolean;
  setShowAdvanced: (value: boolean) => void;
  showDiagnostics: boolean;
  setShowDiagnostics: (value: boolean) => void;
  knowledgeTab: KnowledgeTab;
  setKnowledgeTab: (value: KnowledgeTab) => void;
  memoryPreview: MemoryPreview;
  setMemoryPreview: (value: MemoryPreview) => void;
  connectionMode: ConnectionMode;
  setConnectionMode: (value: ConnectionMode) => void;
  provider: ProviderId;
  setProvider: (value: ProviderId) => void;
  modelChoice: string;
  setModelChoice: (value: string) => void;
  recoveryTab: RecoveryTab;
  setRecoveryTab: (value: RecoveryTab) => void;
  decision: string;
  setDecision: (value: string) => void;
  labState: LabState;
  setLabState: (value: LabState) => void;
}) {
  if (scene === "today") return <TodayScene setScene={setScene} />;
  if (scene === "project") {
    return (
      <ProjectScene
        surface={projectSurface}
        setSurface={setProjectSurface}
        setScene={setScene}
      />
    );
  }
  if (scene === "adhoc") {
    return (
      <AdHocScene
        pinned={pinned}
        setPinned={setPinned}
        selectedArtifact={selectedArtifact}
        setSelectedArtifact={setSelectedArtifact}
      />
    );
  }
  if (scene === "setup") {
    return (
      <SetupScene
        step={setupStep}
        setStep={setSetupStep}
        view={setupView}
        setView={setSetupView}
      />
    );
  }
  if (scene === "runtime") {
    return (
      <RuntimeScene
        selectedMember={selectedMember}
        setSelectedMember={setSelectedMember}
        showAdvanced={showAdvanced}
        setShowAdvanced={setShowAdvanced}
        showDiagnostics={showDiagnostics}
        setShowDiagnostics={setShowDiagnostics}
      />
    );
  }
  if (scene === "knowledge") {
    return (
      <KnowledgeScene
        tab={knowledgeTab}
        setTab={setKnowledgeTab}
        memoryPreview={memoryPreview}
        setMemoryPreview={setMemoryPreview}
      />
    );
  }
  if (scene === "connections") {
    return (
      <ConnectionsScene
        mode={connectionMode}
        setMode={setConnectionMode}
        provider={provider}
        setProvider={setProvider}
        modelChoice={modelChoice}
        setModelChoice={setModelChoice}
      />
    );
  }
  if (scene === "recovery") {
    return (
      <RecoveryScene
        tab={recoveryTab}
        setTab={setRecoveryTab}
        decision={decision}
        setDecision={setDecision}
      />
    );
  }
  return <StateLabScene state={labState} setState={setLabState} />;
}

function ConversationRail({
  channel,
  open,
  drafts,
  setDrafts,
  composerStatus,
  setComposerStatus,
}: {
  channel: ChatChannel;
  open: boolean;
  drafts: Record<ChatChannel, string>;
  setDrafts: (value: Record<ChatChannel, string>) => void;
  composerStatus: string;
  setComposerStatus: (value: string) => void;
}) {
  const isGroup = channel === "project-group";
  const title = isGroup ? "X content operation · Group" : "Personal Assistant";

  const addMention = (mention: string) => {
    const current = drafts[channel];
    const spacer = current.length > 0 && !current.endsWith(" ") ? " " : "";
    setDrafts({ ...drafts, [channel]: `${current}${spacer}${mention} ` });
    setComposerStatus(`${mention} added to the unsent prototype draft.`);
  };

  const previewDraft = () => {
    setComposerStatus(
      drafts[channel].trim().length > 0
        ? "Unsent prototype preview updated. No message, Task, or revision was created."
        : "Draft is empty. Nothing was sent.",
    );
  };

  return (
    <aside className="conversation-rail" data-open={open} aria-label={title}>
      <header>
        <div>
          <span>{isGroup ? "Project conversation" : "Global conversation"}</span>
          <h2>{title}</h2>
        </div>
        <Tag tone="warn">Prototype · no send</Tag>
      </header>

      {isGroup ? (
        <div className="participants" aria-label="Project group participants">
          <span>Owner</span>
          <span>Lin · manager</span>
          <span>Mei · research</span>
          <span>Rui · editor</span>
        </div>
      ) : null}

      <div className="thread" aria-label="Prototype conversation sample">
        {isGroup ? (
          <>
            <article data-author="owner">
              <span>Owner · sample</span>
              <p>@Lin Compare the three outcomes and tell me which one is ready for review.</p>
            </article>
            <article data-author="manager">
              <span>Lin · manager candidate</span>
              <p>
                A is ready for package review. B needs an accessibility revision. C is blocked on source rights. I opened a temporary typed canvas; no Project revision changed.
              </p>
              <small>Basis: mock artifacts A–C · one unknown external Effect excluded</small>
            </article>
            <article data-author="system">
              <span>Authority boundary</span>
              <p>Messages remain candidates until a daemon-owned Task or revision preview exists.</p>
            </article>
          </>
        ) : (
          <>
            <article data-author="assistant">
              <span>Personal Assistant · candidate-only</span>
              <p>
                I can help research a Project, explain today’s decisions, inspect Knowledge, and prepare structured candidates. I cannot create authority or operate a missing connector.
              </p>
            </article>
            <article data-author="system">
              <span>Context</span>
              <p>Global scope · no Project group selected · source data is static prototype content.</p>
            </article>
          </>
        )}
      </div>

      <div className="composer">
        {isGroup ? (
          <div className="mention-row" aria-label="Add a mention to the unsent draft">
            <button type="button" onClick={() => addMention("@manager")}>@manager</button>
            <button type="button" onClick={() => addMention("@member")}>@member</button>
            <button type="button" onClick={() => addMention("@Mei")}>@Mei</button>
            <button type="button" onClick={() => addMention("@Rui")}>@Rui</button>
          </div>
        ) : null}
        <label>
          <span>Message {title}</span>
          <textarea
            name={`draft-${channel}`}
            autoComplete="off"
            value={drafts[channel]}
            onChange={(event: { target: { value: string } }) => {
              setDrafts({ ...drafts, [channel]: event.target.value });
              setComposerStatus("Unsent prototype draft changed.");
            }}
            placeholder={isGroup ? "Ask @manager or redirect a Member inside the current boundary…" : "Describe an outcome or ask what needs you…"}
          />
        </label>
        <div className="composer-footer">
          <button className="secondary-button" type="button" onClick={previewDraft}>
            Preview unsent message
          </button>
          <small aria-live="polite">{composerStatus}</small>
        </div>
        <CapabilityGap>
          Sending, @ routing, conversation archive, and conversion to a Task/revision need daemon-backed capabilities.
        </CapabilityGap>
      </div>
    </aside>
  );
}

export default function Personal2OpcDigitalStaffConsoleV1() {
  const theme = useHostTheme();
  const [scene, setScene] = useState<Scene>("today");
  const [projectSurface, setProjectSurface] = useState<ProjectSurface>("brief");
  const [pinned, setPinned] = useState(false);
  const [selectedArtifact, setSelectedArtifact] = useState("outcome-a");
  const [setupStep, setSetupStep] = useState<SetupStep>("research");
  const [setupView, setSetupView] = useState<SetupView>("research-partial");
  const [selectedMember, setSelectedMember] = useState<MemberId>("manager");
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showDiagnostics, setShowDiagnostics] = useState(false);
  const [knowledgeTab, setKnowledgeTab] = useState<KnowledgeTab>("vault");
  const [memoryPreview, setMemoryPreview] = useState<MemoryPreview>("inspect");
  const [connectionMode, setConnectionMode] = useState<ConnectionMode>("quick");
  const [provider, setProvider] = useState<ProviderId>("anthropic");
  const [modelChoice, setModelChoice] = useState("unselected");
  const [recoveryTab, setRecoveryTab] = useState<RecoveryTab>("approval");
  const [decision, setDecision] = useState("Review only");
  const [labState, setLabState] = useState<LabState>("unknown");
  const [chatOpen, setChatOpen] = useState(true);
  const [drafts, setDrafts] = useState<Record<ChatChannel, string>>({
    assistant: "",
    "project-group": "@manager ",
  });
  const [composerStatus, setComposerStatus] = useState(
    "Drafts are local prototype state only.",
  );

  const projectScenes: readonly Scene[] = [
    "project",
    "adhoc",
    "runtime",
    "recovery",
  ];
  const chatChannel: ChatChannel = projectScenes.includes(scene)
    ? "project-group"
    : "assistant";

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
    "--accent": theme.accent.primary,
    "--good": theme.category.green,
    "--warn": theme.category.yellow,
    "--bad": theme.category.red,
    "--info": theme.category.blue,
  } as CSSProperties;

  return (
    <div className="opc-console" style={variables}>
      <style>{`
        .opc-console {
          min-height: 100vh;
          background: var(--bg);
          color: var(--text);
          font: 13px/1.48 system-ui, "Segoe UI Variable", "Segoe UI", sans-serif;
          color-scheme: light dark;
        }
        .opc-console *,
        .opc-console *::before,
        .opc-console *::after { box-sizing: border-box; }
        .opc-console button,
        .opc-console input,
        .opc-console select,
        .opc-console textarea { color: inherit; font: inherit; }
        .opc-console button,
        .opc-console input,
        .opc-console select,
        .opc-console textarea {
          touch-action: manipulation;
          -webkit-tap-highlight-color: transparent;
        }
        .opc-console button { cursor: pointer; }
        .opc-console button:active { transform: translateY(1px); }
        .opc-console :focus-visible {
          outline: 2px solid var(--focus);
          outline-offset: 2px;
        }
        .opc-console ::selection { background: var(--accent); color: var(--bg); }
        .opc-console h1,
        .opc-console h2,
        .opc-console h3,
        .opc-console p { margin-block-start: 0; }
        .opc-console h1,
        .opc-console h2,
        .opc-console h3 {
          scroll-margin-top: 72px;
          text-wrap: balance;
        }
        .opc-console p { text-wrap: pretty; }
        .opc-console h1 {
          margin-block-end: 2px;
          font-size: 18px;
          line-height: 1.2;
          letter-spacing: -0.018em;
        }
        .opc-console h2 {
          margin-block-end: 6px;
          font-size: clamp(18px, 2vw, 25px);
          line-height: 1.18;
          letter-spacing: -0.025em;
        }
        .opc-console h3 {
          margin-block-end: 4px;
          font-size: 14px;
          line-height: 1.28;
          letter-spacing: -0.008em;
        }
        .opc-console p { margin-block-end: 10px; }
        .opc-console p,
        .opc-console small,
        .opc-console dd { overflow-wrap: anywhere; }
        .skip-link {
          position: fixed;
          z-index: 100;
          inset-block-start: 8px;
          inset-inline-start: 8px;
          transform: translateY(-140%);
          padding: 8px 12px;
          border: 1px solid var(--line-strong);
          background: var(--surface);
          color: var(--text);
        }
        .skip-link:focus { transform: translateY(0); }
        .prototype-banner {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 20px;
          min-height: 64px;
          padding: 10px 16px;
          border-block-end: 1px solid var(--line-strong);
          background: var(--chrome);
        }
        .prototype-banner p {
          margin: 0;
          max-width: 72ch;
          color: var(--muted);
          font-size: 11px;
        }
        .scene-select {
          display: grid;
          gap: 4px;
          min-width: min(320px, 42vw);
        }
        .scene-select span {
          color: var(--muted);
          font-size: 10px;
          font-weight: 650;
        }
        .scene-select select,
        .field input,
        .field select,
        .composer textarea {
          min-height: 40px;
          border: 1px solid var(--line-strong);
          border-radius: 5px;
          background: var(--surface);
          padding: 8px 10px;
        }
        .evidence-strip {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          gap: 8px 16px;
          min-height: 34px;
          padding: 6px 16px;
          border-block-end: 1px solid var(--line);
          background: var(--fill);
          color: var(--muted);
          font-size: 10px;
        }
        .evidence-strip strong { color: var(--text); }
        .product-shell {
          display: grid;
          grid-template-columns: 164px minmax(560px, 1fr) 330px;
          min-height: calc(100vh - 99px);
        }
        .product-nav {
          display: flex;
          flex-direction: column;
          min-width: 0;
          padding: 10px 8px;
          border-inline-end: 1px solid var(--line);
          background: var(--chrome);
        }
        .brand {
          padding: 8px 10px 18px;
          font-size: 15px;
          font-weight: 760;
          letter-spacing: -0.015em;
        }
        .product-nav button {
          display: flex;
          align-items: center;
          justify-content: space-between;
          width: 100%;
          min-height: 42px;
          border: 1px solid transparent;
          border-radius: 5px;
          background: transparent;
          padding: 8px 10px;
          text-align: start;
        }
        .product-nav button:hover { background: var(--fill); }
        .product-nav button[aria-current="page"] {
          border-color: var(--line-strong);
          background: var(--fill-strong);
          font-weight: 700;
        }
        .nav-spacer { flex: 1; min-height: 24px; }
        .settings-button {
          border-block-start-color: var(--line) !important;
          border-radius: 0 !important;
          margin-block-start: 8px;
          padding-block-start: 12px !important;
        }
        .main-column { min-width: 0; }
        .context-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 16px;
          min-height: 64px;
          padding: 10px 16px;
          border-block-end: 1px solid var(--line);
          background: var(--surface);
        }
        .context-header p {
          margin: 0 0 2px;
          color: var(--muted);
          font-size: 10px;
        }
        .context-header h2 {
          margin: 0;
          font-size: 16px;
          letter-spacing: -0.012em;
        }
        .context-actions {
          display: flex;
          align-items: center;
          justify-content: flex-end;
          gap: 7px;
        }
        .main-content {
          min-width: 0;
          padding: 16px;
        }
        .scene-stack {
          display: grid;
          gap: 12px;
        }
        .tag {
          display: inline-flex;
          align-items: center;
          width: max-content;
          min-height: 22px;
          border: 1px solid currentColor;
          border-radius: 999px;
          padding: 2px 7px;
          color: var(--muted);
          font-size: 9px;
          font-weight: 690;
          line-height: 1.2;
          white-space: nowrap;
        }
        .tag[data-tone="good"] { color: var(--good); }
        .tag[data-tone="warn"] { color: var(--warn); }
        .tag[data-tone="bad"] { color: var(--bad); }
        .tag[data-tone="info"] { color: var(--info); }
        .primary-button,
        .secondary-button,
        .text-button,
        .inline-button,
        .segmented button,
        .context-tabs button,
        .section-tabs button,
        .setup-steps button,
        .mention-row button,
        .stacked-actions button {
          min-height: 40px;
          border: 1px solid var(--line-strong);
          border-radius: 5px;
          background: var(--surface);
          padding: 7px 11px;
        }
        .primary-button {
          border-color: var(--accent);
          background: var(--accent);
          color: var(--bg);
          font-weight: 750;
        }
        .secondary-button:hover,
        .text-button:hover,
        .inline-button:hover,
        .segmented button:hover,
        .context-tabs button:hover,
        .section-tabs button:hover,
        .setup-steps button:hover,
        .mention-row button:hover,
        .stacked-actions button:hover { background: var(--fill); }
        .text-button,
        .inline-button {
          min-height: 36px;
          background: transparent;
        }
        .inline-button { padding: 5px 8px; }
        .segmented {
          display: flex;
          flex-wrap: wrap;
          gap: 5px;
        }
        .segmented button[aria-pressed="true"],
        .stacked-actions button[aria-pressed="true"],
        .provider-choice button[aria-pressed="true"] {
          border-color: var(--accent);
          background: var(--fill-strong);
          font-weight: 700;
        }
        .section-heading {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 12px;
          padding-block-end: 9px;
          border-block-end: 1px solid var(--line);
        }
        .section-heading h3 { margin: 0; }
        .section-heading p {
          margin: 3px 0 0;
          max-width: 72ch;
          color: var(--muted);
          font-size: 11px;
        }
        .work-surface,
        .board-panel,
        .setup-outline,
        .member-picker {
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 13px;
        }
        .primary-decision,
        .project-outcome,
        .temporary-canvas-head,
        .setup-intro,
        .settings-head,
        .state-lab-head {
          display: flex;
          align-items: flex-end;
          justify-content: space-between;
          gap: 24px;
          border-block-end: 1px solid var(--line-strong);
          padding: 8px 2px 14px;
        }
        .primary-decision > div,
        .project-outcome > div,
        .temporary-canvas-head > div,
        .setup-intro > div,
        .settings-head > div,
        .state-lab-head > div { max-width: 76ch; }
        .primary-decision h2,
        .project-outcome h2,
        .temporary-canvas-head h2,
        .setup-intro h2,
        .settings-head h2,
        .state-lab-head h2 { margin-block-start: 8px; }
        .primary-decision p,
        .project-outcome p,
        .temporary-canvas-head p,
        .setup-intro p,
        .settings-head p { margin: 0; color: var(--muted); }
        .state-callout,
        .capability-gap {
          display: grid;
          grid-template-columns: minmax(120px, auto) minmax(0, 1fr);
          gap: 10px 14px;
          border: 1px solid var(--line-strong);
          border-radius: 5px;
          background: var(--fill);
          padding: 9px 11px;
        }
        .state-callout strong,
        .capability-gap strong { color: var(--warn); }
        .state-callout[data-tone="bad"] strong { color: var(--bad); }
        .state-callout[data-tone="good"] strong { color: var(--good); }
        .state-callout[data-tone="info"] strong { color: var(--info); }
        .state-callout span,
        .capability-gap span { min-width: 0; color: var(--muted); }
        .two-column,
        .connection-layout,
        .state-lab-grid {
          display: grid;
          grid-template-columns: minmax(0, 1.15fr) minmax(280px, .85fr);
          gap: 12px;
        }
        .brief-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 12px;
        }
        .brief-main { grid-column: 1 / -1; }
        .outcome-list article {
          display: grid;
          grid-template-columns: minmax(0, 1fr) minmax(145px, auto);
          gap: 14px;
          padding: 11px 0;
          border-block-end: 1px solid var(--line);
        }
        .outcome-list article:last-child { border-block-end: 0; }
        .outcome-list p {
          margin: 3px 0 0;
          color: var(--muted);
        }
        .row-meta {
          display: flex;
          flex-direction: column;
          align-items: flex-end;
          gap: 5px;
          text-align: end;
        }
        .row-meta span { color: var(--muted); font-size: 10px; }
        .accepted-outcome { padding-block-start: 12px; }
        .accepted-outcome > strong { font-size: 15px; }
        .accepted-outcome > p { margin: 5px 0 12px; color: var(--muted); }
        .mini-facts,
        .fact-list,
        .member-defaults,
        .runtime-grid,
        .permission-review { margin: 0; }
        .mini-facts > div,
        .fact-list > div,
        .member-defaults > div,
        .runtime-grid > div,
        .permission-review > div {
          display: grid;
          grid-template-columns: minmax(115px, .35fr) minmax(0, 1fr);
          gap: 10px;
          padding: 8px 0;
          border-block-end: 1px solid var(--line);
        }
        .mini-facts > div:last-child,
        .fact-list > div:last-child,
        .member-defaults > div:last-child,
        .runtime-grid > div:last-child,
        .permission-review > div:last-child { border-block-end: 0; }
        dt { color: var(--muted); }
        dd { margin: 0; min-width: 0; overflow-wrap: anywhere; }
        dd strong,
        dd small { display: block; }
        dd small { margin-block-start: 2px; color: var(--muted); font-size: 10px; }
        .fact-list.compact > div { padding: 7px 0; }
        .project-row {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 20px;
          width: 100%;
          margin-block-start: 10px;
          border: 0;
          background: transparent;
          padding: 10px 2px;
          text-align: start;
        }
        .project-row:hover { background: var(--fill); }
        .project-row > span:last-child { text-align: end; }
        .project-row strong,
        .project-row small { display: block; }
        .project-row small { margin-block-start: 3px; color: var(--muted); }
        .context-tabs,
        .section-tabs {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          margin-block-end: 12px;
          border-block-end: 1px solid var(--line);
          padding-block-end: 8px;
        }
        .context-tabs button,
        .section-tabs button {
          min-height: 36px;
          border-color: transparent;
          background: transparent;
        }
        .context-tabs button[aria-current],
        .section-tabs button[aria-current],
        .section-tabs button[aria-pressed="true"] {
          border-color: var(--line-strong);
          background: var(--fill-strong);
          font-weight: 700;
        }
        .table-wrap {
          width: 100%;
          overflow-x: auto;
          margin-block-start: 10px;
        }
        table {
          width: 100%;
          border-collapse: collapse;
          font-variant-numeric: tabular-nums;
        }
        th, td {
          min-width: 110px;
          border-block-end: 1px solid var(--line);
          padding: 9px 8px;
          text-align: start;
          vertical-align: top;
        }
        thead th {
          color: var(--muted);
          font-size: 10px;
          font-weight: 700;
        }
        tbody th { font-weight: 700; }
        tbody tr[data-selected="true"] { background: var(--fill); }
        .manager-brief {
          margin-block-start: 12px;
          color: var(--text);
          font-size: 14px;
          line-height: 1.55;
        }
        .compact-list,
        .decision-list,
        .coverage-list {
          list-style: none;
          margin: 8px 0 0;
          padding: 0;
        }
        .compact-list li {
          display: flex;
          align-items: baseline;
          justify-content: space-between;
          gap: 12px;
          padding: 9px 0;
          border-block-end: 1px solid var(--line);
        }
        .compact-list span { color: var(--muted); text-align: end; }
        .decision-list li {
          display: grid;
          grid-template-columns: 28px minmax(0, 1fr);
          gap: 8px;
          padding: 8px 0;
          border-block-end: 1px solid var(--line);
        }
        .decision-list li > span {
          color: var(--muted);
          font-variant-numeric: tabular-nums;
        }
        .decision-list strong,
        .decision-list small { display: block; }
        .decision-list small { margin-block-start: 2px; color: var(--muted); }
        .operating-loop {
          display: grid;
          grid-template-columns: repeat(9, minmax(128px, 1fr));
          gap: 6px;
          overflow-x: auto;
          list-style: none;
          margin: 10px 0 0;
          padding: 0 0 4px;
          font-variant-numeric: tabular-nums;
        }
        .operating-loop li {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 5px;
          padding: 8px;
        }
        .operating-loop span,
        .operating-loop strong,
        .operating-loop small { display: block; }
        .operating-loop span,
        .operating-loop small { color: var(--muted); }
        .operating-loop strong { margin-block: 3px; }
        .operating-loop li[data-state="done"] { border-color: var(--good); }
        .operating-loop li[data-state="blocked"] { border-color: var(--bad); }
        .operating-loop li[data-state="waiting"],
        .operating-loop li[data-state="working"] { border-color: var(--warn); }
        .member-summary-grid {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 9px;
          margin-block-start: 12px;
        }
        .member-summary-grid article {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 5px;
          padding: 10px;
        }
        .member-summary-grid article header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 8px;
        }
        .member-summary-grid article header strong,
        .member-summary-grid article header span { display: block; }
        .member-summary-grid article header span {
          margin-block-start: 2px;
          color: var(--muted);
          font-size: 10px;
        }
        .member-summary-grid article p { margin: 10px 0; }
        .member-summary-grid article dl { margin: 0; }
        .member-summary-grid article dl div {
          padding: 6px 0;
          border-block-end: 1px solid var(--line);
        }
        .member-summary-grid article dt { font-size: 10px; }
        .action-line,
        .runtime-actions {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-block-start: 12px;
        }
        .action-line span,
        .runtime-actions span { color: var(--muted); }
        .attention-rows {
          display: grid;
          gap: 0;
          margin-block-start: 8px;
        }
        .attention-rows button {
          display: grid;
          gap: 5px;
          width: 100%;
          min-height: 56px;
          border: 0;
          border-block-end: 1px solid var(--line);
          background: transparent;
          padding: 10px 2px;
          text-align: start;
        }
        .attention-rows button:hover { background: var(--fill); }
        .attention-rows button > span {
          display: flex;
          align-items: center;
          gap: 9px;
        }
        .attention-rows small { color: var(--muted); }
        .typed-board {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 12px;
        }
        .board-span { grid-column: 1 / -1; }
        .decision-copy {
          margin: 12px 0 6px;
          font-size: 14px;
          font-weight: 660;
        }
        .fine-print { color: var(--muted); font-size: 10px; }
        .artifact-inspector {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 10px;
          margin-block-start: 10px;
          border-block-start: 1px solid var(--line);
          padding-block-start: 10px;
        }
        .artifact-inspector span,
        .artifact-inspector strong { display: block; }
        .artifact-inspector span { color: var(--muted); font-size: 10px; }
        .artifact-inspector strong { margin-block-start: 3px; }
        .setup-steps {
          display: flex;
          gap: 4px;
          overflow-x: auto;
          padding-block-end: 4px;
        }
        .setup-steps button {
          display: flex;
          align-items: center;
          gap: 6px;
          min-width: max-content;
          background: transparent;
        }
        .setup-steps button span {
          color: var(--muted);
          font-variant-numeric: tabular-nums;
        }
        .setup-steps button[aria-current="step"] {
          border-color: var(--accent);
          background: var(--fill-strong);
          font-weight: 700;
        }
        .setup-workbench {
          display: grid;
          grid-template-columns: minmax(0, 1.35fr) minmax(260px, .65fr);
          gap: 12px;
        }
        .assistant-note {
          margin-block-start: 10px;
          border-block-end: 1px solid var(--line);
          padding: 2px 0 10px;
        }
        .assistant-note p { margin: 5px 0 0; line-height: 1.58; }
        .setup-outline h3 { margin: 0 0 10px; }
        .setup-outline ol {
          display: grid;
          gap: 8px;
          margin: 0 0 12px;
          padding-inline-start: 22px;
        }
        .setup-outline li strong,
        .setup-outline li span { display: block; }
        .setup-outline li span { color: var(--muted); font-size: 10px; }
        .object-chain {
          display: grid;
          grid-template-columns: minmax(140px, 1fr) auto minmax(140px, 1fr) auto minmax(120px, .8fr) auto minmax(160px, 1.2fr);
          align-items: center;
          gap: 8px;
          border-block-end: 1px solid var(--line-strong);
          padding: 8px 2px 14px;
        }
        .object-chain > div {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 5px;
          padding: 9px;
        }
        .object-chain strong,
        .object-chain span { display: block; }
        .object-chain > span { color: var(--muted); text-align: center; }
        .object-chain div span { margin-block-start: 2px; color: var(--muted); font-size: 10px; }
        .runtime-layout {
          display: grid;
          grid-template-columns: 220px minmax(0, 1fr);
          gap: 12px;
        }
        .member-picker h3 { margin: 0 0 8px; }
        .member-picker > button {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 8px;
          width: 100%;
          min-height: 56px;
          border: 1px solid transparent;
          border-block-end-color: var(--line);
          background: transparent;
          padding: 8px;
          text-align: start;
        }
        .member-picker > button:hover { background: var(--fill); }
        .member-picker > button[aria-current="true"] {
          border-color: var(--line-strong);
          background: var(--fill-strong);
        }
        .member-picker button strong,
        .member-picker button small { display: block; }
        .member-picker button small { color: var(--muted); }
        .member-defaults { margin-block-start: 8px; }
        .advanced-runtime {
          margin-block-start: 14px;
          border-block-start: 1px solid var(--line-strong);
          padding-block-start: 13px;
        }
        .runtime-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 0 16px;
        }
        .runtime-grid > div { grid-template-columns: minmax(90px, .3fr) minmax(0, 1fr); }
        .diagnostics {
          margin-block-start: 10px;
          border: 1px solid var(--line);
          border-radius: 5px;
          background: var(--fill);
          padding: 10px;
        }
        .diagnostics .mini-facts { margin-block-start: 6px; }
        .memory-card { margin-block-start: 11px; }
        .memory-card header {
          display: flex;
          justify-content: space-between;
          gap: 12px;
        }
        .memory-card header strong,
        .memory-card header span { display: block; }
        .memory-card header span { color: var(--muted); font-size: 10px; }
        .context-ladder {
          display: grid;
          gap: 0;
          list-style: none;
          margin: 10px 0 0;
          padding: 0;
        }
        .context-ladder li {
          display: grid;
          grid-template-columns: 30px minmax(0, 1fr);
          gap: 10px;
          padding: 9px 0;
          border-block-end: 1px solid var(--line);
        }
        .context-ladder li > span { color: var(--muted); }
        .context-ladder strong,
        .context-ladder small { display: block; }
        .context-ladder small { color: var(--muted); }
        .provider-choice {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 7px;
          margin-block-start: 11px;
        }
        .provider-choice button {
          min-height: 68px;
          border: 1px solid var(--line-strong);
          border-radius: 5px;
          background: transparent;
          padding: 9px;
          text-align: start;
        }
        .provider-choice button strong,
        .provider-choice button small { display: block; }
        .provider-choice button small { margin-block-start: 3px; color: var(--muted); }
        .field {
          display: grid;
          gap: 5px;
          margin-block-start: 12px;
        }
        .field > span { font-weight: 680; }
        .field > small { color: var(--muted); }
        .field input[readonly] { color: var(--muted); }
        .form-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 0 14px;
        }
        .approval-grid {
          display: grid;
          grid-template-columns: minmax(0, 1.25fr) minmax(240px, .75fr);
          gap: 14px;
          margin-block-start: 10px;
        }
        .decision-preview {
          border: 1px solid var(--line);
          border-radius: 5px;
          background: var(--fill);
          padding: 11px;
        }
        .decision-preview p,
        .decision-preview output { color: var(--muted); }
        .stacked-actions {
          display: grid;
          gap: 6px;
          margin-block: 10px;
        }
        .stacked-actions button { text-align: start; }
        .reconcile-path {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 8px;
          margin-block: 11px;
        }
        .reconcile-path > div {
          border: 1px solid var(--line);
          border-radius: 5px;
          padding: 10px;
        }
        .reconcile-path > div[data-current="true"] {
          border-color: var(--warn);
          background: var(--fill);
        }
        .reconcile-path span,
        .reconcile-path strong,
        .reconcile-path small { display: block; }
        .reconcile-path span,
        .reconcile-path small { color: var(--muted); }
        .reconcile-path strong { margin-block: 4px; }
        .compact-field { min-width: min(280px, 42vw); margin: 0; }
        .coverage-list {
          max-height: 430px;
          overflow-y: auto;
        }
        .coverage-list li {
          border-block-end: 1px solid var(--line);
        }
        .coverage-list li[data-selected="true"] { background: var(--fill); }
        .coverage-list button {
          width: 100%;
          min-height: 50px;
          border: 0;
          background: transparent;
          padding: 8px;
          text-align: start;
        }
        .coverage-list strong,
        .coverage-list span { display: block; }
        .coverage-list span { margin-block-start: 2px; color: var(--muted); font-size: 10px; }
        .conversation-rail {
          display: flex;
          flex-direction: column;
          min-width: 0;
          min-height: 0;
          border-inline-start: 1px solid var(--line);
          background: var(--surface);
        }
        .conversation-rail > header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 10px;
          padding: 12px;
          border-block-end: 1px solid var(--line);
        }
        .conversation-rail > header span {
          color: var(--muted);
          font-size: 10px;
        }
        .conversation-rail > header h2 {
          margin: 2px 0 0;
          font-size: 15px;
        }
        .participants {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          padding: 8px 12px;
          border-block-end: 1px solid var(--line);
        }
        .participants span {
          border: 1px solid var(--line);
          border-radius: 999px;
          padding: 3px 7px;
          color: var(--muted);
          font-size: 9px;
        }
        .thread {
          flex: 1;
          min-height: 220px;
          overflow-y: auto;
          padding: 12px;
        }
        .thread article {
          margin-block-end: 13px;
          border-block-end: 1px solid var(--line);
          padding-block-end: 12px;
        }
        .thread article > span {
          color: var(--muted);
          font-size: 10px;
          font-weight: 680;
        }
        .thread article p { margin: 4px 0; }
        .thread article small { color: var(--muted); }
        .thread article[data-author="owner"] p {
          margin-inline-start: 18px;
          font-weight: 620;
        }
        .thread article[data-author="system"] {
          border: 1px solid var(--line);
          border-radius: 5px;
          background: var(--fill);
          padding: 9px;
        }
        .composer {
          display: grid;
          gap: 8px;
          border-block-start: 1px solid var(--line);
          padding: 10px;
        }
        .composer label { display: grid; gap: 5px; }
        .composer label > span { font-weight: 690; }
        .composer textarea {
          width: 100%;
          min-height: 92px;
          resize: vertical;
        }
        .mention-row {
          display: flex;
          flex-wrap: wrap;
          gap: 5px;
        }
        .mention-row button { min-height: 36px; padding: 5px 8px; }
        .composer-footer {
          display: flex;
          align-items: center;
          gap: 9px;
        }
        .composer-footer small {
          color: var(--muted);
          overflow-wrap: anywhere;
        }
        .composer .capability-gap {
          grid-template-columns: 1fr;
          gap: 3px;
          font-size: 10px;
        }
        .chat-toggle { display: none; }
        @media (max-width: 1180px) {
          .product-shell { grid-template-columns: 150px minmax(0, 1fr); }
          .conversation-rail {
            grid-column: 1 / -1;
            min-height: 440px;
            border-inline-start: 0;
            border-block-start: 1px solid var(--line-strong);
          }
          .conversation-rail[data-open="false"] { display: none; }
          .chat-toggle { display: inline-flex; }
        }
        @media (max-width: 840px) {
          .prototype-banner,
          .primary-decision,
          .project-outcome,
          .temporary-canvas-head,
          .setup-intro,
          .settings-head,
          .state-lab-head {
            align-items: stretch;
            flex-direction: column;
          }
          .scene-select,
          .compact-field { min-width: 0; width: 100%; }
          .two-column,
          .brief-grid,
          .connection-layout,
          .state-lab-grid,
          .setup-workbench,
          .runtime-layout,
          .approval-grid,
          .typed-board { grid-template-columns: 1fr; }
          .brief-main,
          .board-span { grid-column: auto; }
          .member-summary-grid,
          .artifact-inspector,
          .provider-choice,
          .runtime-grid,
          .form-grid { grid-template-columns: 1fr; }
          .object-chain {
            grid-template-columns: 1fr;
          }
          .object-chain > span { transform: rotate(90deg); }
          .reconcile-path { grid-template-columns: repeat(2, minmax(0, 1fr)); }
        }
        @media (max-width: 680px) {
          .product-shell { display: block; }
          .product-nav {
            position: sticky;
            z-index: 5;
            inset-block-start: 0;
            flex-direction: row;
            overflow-x: auto;
            border-inline-end: 0;
            border-block-end: 1px solid var(--line);
            padding: 6px;
          }
          .brand,
          .nav-spacer { display: none; }
          .product-nav button {
            min-width: max-content;
            width: auto;
          }
          .settings-button {
            border-block-start-color: transparent !important;
            border-radius: 5px !important;
            margin-block-start: 0;
            padding-block-start: 8px !important;
          }
          .context-header { align-items: flex-start; }
          .context-actions { flex-wrap: wrap; }
          .main-content { padding: 12px; }
          .outcome-list article,
          .state-callout,
          .capability-gap,
          .mini-facts > div,
          .fact-list > div,
          .member-defaults > div,
          .runtime-grid > div,
          .permission-review > div {
            grid-template-columns: 1fr;
          }
          .row-meta { align-items: flex-start; text-align: start; }
          .compact-list li,
          .action-line,
          .runtime-actions,
          .composer-footer {
            align-items: flex-start;
            flex-direction: column;
          }
          .reconcile-path { grid-template-columns: 1fr; }
        }
        @media (prefers-reduced-motion: reduce) {
          .opc-console *,
          .opc-console *::before,
          .opc-console *::after {
            animation-duration: .01ms !important;
            transition-duration: .01ms !important;
            scroll-behavior: auto !important;
          }
          .opc-console button:active { transform: none; }
        }
        @media (prefers-contrast: more) {
          .work-surface,
          .board-panel,
          .setup-outline,
          .member-picker,
          .state-callout,
          .capability-gap { border-color: var(--text); }
        }
      `}</style>

      <a className="skip-link" href="#prototype-main">Skip to main canvas</a>

      <header className="prototype-banner">
        <div>
          <h1>Personal 2.0 · Digital Staff Console V1</h1>
          <p>
            New interaction prototype · built-in mock data · no daemon, network, storage, filesystem, Provider, Agent, approval, or publication effect
          </p>
        </div>
        <label className="scene-select">
          <span>Prototype scene</span>
          <select
            name="prototype-scene"
            value={scene}
            onChange={(event: { target: { value: string } }) => setScene(event.target.value as Scene)}
          >
            {SCENES.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>{candidate.label}</option>
            ))}
          </select>
        </label>
      </header>

      <div className="evidence-strip" role="note">
        <strong>Prototype</strong>
        <span>Requires-backend labels are explanatory, never executable.</span>
        <span>No Project ID, connector receipt, verified publish, real cost, or secret is rendered.</span>
      </div>

      <div className="product-shell">
        <nav className="product-nav" aria-label="Personal primary navigation">
          <div className="brand">Personal</div>
          {PRIMARY_NAV.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-current={
                item.id === "project"
                  ? projectScenes.includes(scene) || scene === "setup"
                    ? "page"
                    : undefined
                  : scene === item.id
                    ? "page"
                    : undefined
              }
              onClick={() => setScene(item.id)}
            >
              {item.label}
              {item.id === "project" ? <span className="tag" data-tone="warn">1</span> : null}
            </button>
          ))}
          <div className="nav-spacer" />
          <button
            className="settings-button"
            type="button"
            aria-current={scene === "connections" ? "page" : undefined}
            onClick={() => setScene("connections")}
          >
            Settings
          </button>
        </nav>

        <main className="main-column" id="prototype-main">
          <header className="context-header">
            <div>
              <p>
                {projectScenes.includes(scene) || scene === "setup"
                  ? "Projects / X content operation"
                  : scene === "connections"
                    ? "Settings"
                    : "Personal"}
              </p>
              <h2 aria-live="polite">{SCENE_TITLES[scene]}</h2>
            </div>
            <div className="context-actions">
              <Tag tone="warn">Prototype</Tag>
              <Tag tone="neutral">Windows-local target</Tag>
              <button
                className="secondary-button chat-toggle"
                type="button"
                aria-expanded={chatOpen}
                onClick={() => setChatOpen(!chatOpen)}
              >
                {chatOpen ? "Hide conversation" : "Open conversation"}
              </button>
            </div>
          </header>

          <div className="main-content">
            <MainScene
              scene={scene}
              setScene={setScene}
              projectSurface={projectSurface}
              setProjectSurface={setProjectSurface}
              pinned={pinned}
              setPinned={setPinned}
              selectedArtifact={selectedArtifact}
              setSelectedArtifact={setSelectedArtifact}
              setupStep={setupStep}
              setSetupStep={setSetupStep}
              setupView={setupView}
              setSetupView={setSetupView}
              selectedMember={selectedMember}
              setSelectedMember={setSelectedMember}
              showAdvanced={showAdvanced}
              setShowAdvanced={setShowAdvanced}
              showDiagnostics={showDiagnostics}
              setShowDiagnostics={setShowDiagnostics}
              knowledgeTab={knowledgeTab}
              setKnowledgeTab={setKnowledgeTab}
              memoryPreview={memoryPreview}
              setMemoryPreview={setMemoryPreview}
              connectionMode={connectionMode}
              setConnectionMode={setConnectionMode}
              provider={provider}
              setProvider={setProvider}
              modelChoice={modelChoice}
              setModelChoice={setModelChoice}
              recoveryTab={recoveryTab}
              setRecoveryTab={setRecoveryTab}
              decision={decision}
              setDecision={setDecision}
              labState={labState}
              setLabState={setLabState}
            />
          </div>
        </main>

        <ConversationRail
          channel={chatChannel}
          open={chatOpen}
          drafts={drafts}
          setDrafts={setDrafts}
          composerStatus={composerStatus}
          setComposerStatus={setComposerStatus}
        />
      </div>
    </div>
  );
}
