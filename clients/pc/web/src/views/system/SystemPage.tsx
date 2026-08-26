import { useCallback, useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { readJson } from "../../api";
import { ConfirmSurface } from "../../components/ConfirmSurface";
import { FactGrid } from "../../components/FactGrid";
import { PageHeader } from "../../components/PageHeader";
import { ReceiptLine } from "../../components/ReceiptLine";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord } from "../../data/projections";
import {
  expandedReadinessComponents,
  formatAge,
  projectHomeReadiness,
  readinessComponentReading,
  worstReadinessComponent,
  type HomeReadinessView,
} from "../../data/projections/home";
import {
  SYSTEM_BACKUP_NEVER,
  SYSTEM_CLAIM_CEILING,
  SYSTEM_DOCTOR_REDACTION,
  SYSTEM_RESTORE_409,
  SYSTEM_SECTIONS,
  isSystemSection,
  projectBackupReceipt,
  projectDoctor,
  projectRestoreReceipt,
  type BackupReceiptView,
  type DoctorView,
  type SystemSectionId,
} from "../../data/projections/system";
import { appProjections, type Projection } from "../../data/store";
import { useLastGood, useProjection } from "../../data/useProjection";
import {
  clearSession,
  sessionHasChannel,
  sessionPrincipal,
} from "../../session";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";
import { ProjectionState } from "../providers/ProjectionState";

export const SYSTEM_STATUS_KEY = "system:status";
export const SYSTEM_DOCTOR_KEY = "system:doctor";

/**
 * System — docs/design/20. The surface an operator reads when something is
 * wrong. Vocabulary is the daemon's; healthy rows stay one line; recovery is
 * a link. Upgrade/uninstall stay class-C CLI.
 */
export function SystemPage() {
  const [query, setQuery] = useSearchParams();
  const requested = query.get("section");
  const [section, setSection] = useState<SystemSectionId>(
    isSystemSection(requested) ? requested : "readiness",
  );
  const [nowMs, setNowMs] = useState(() => Date.now());

  const status = useProjection<HomeReadinessView>(SYSTEM_STATUS_KEY);
  const doctor = useProjection<DoctorView>(SYSTEM_DOCTOR_KEY);
  const statusGood = useLastGood(status);
  const doctorGood = useLastGood(doctor);

  const refresh = useCallback(async () => {
    setNowMs(Date.now());
    await Promise.all([
      fetchProjection(
        appProjections,
        SYSTEM_STATUS_KEY,
        "/personal/status",
        "management",
        projectHomeReadiness,
      ),
      fetchProjection(
        appProjections,
        SYSTEM_DOCTOR_KEY,
        "/personal/doctor",
        "management",
        projectDoctor,
      ),
    ]);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  function go(next: SystemSectionId) {
    setSection(next);
    const params = new URLSearchParams(query);
    params.set("section", next);
    setQuery(params, { replace: true });
  }

  return (
    <section>
      <PageHeader
        title="System"
        lede="Readiness, doctor, stewardship, session, and diagnostics. This is the surface you read when something is wrong — not a dashboard."
      />
      <p className="cp-quiet" data-annotation="system-claim">
        {SYSTEM_CLAIM_CEILING}
      </p>
      <p className="cp-next">
        <button type="button" className="cp-button" onClick={() => void refresh()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">This space refreshes only when you ask.</span>
      </p>
      <nav className="cp-subnav" aria-label="System sections">
        {SYSTEM_SECTIONS.map((item) => (
          <button
            key={item.id}
            type="button"
            className="cp-button"
            aria-current={section === item.id ? "page" : undefined}
            onClick={() => go(item.id)}
          >
            {item.title}
          </button>
        ))}
      </nav>
      {section === "readiness" ? (
        <ReadinessDetail projection={status} view={statusGood.data} nowMs={nowMs} />
      ) : null}
      {section === "doctor" ? (
        <DoctorDetail projection={doctor} view={doctorGood.data} nowMs={nowMs} />
      ) : null}
      {section === "stewardship" ? <StewardshipDetail /> : null}
      {section === "session" ? <SessionDetail /> : null}
      {section === "about" ? <AboutDetail view={doctorGood.data} /> : null}
    </section>
  );
}

function ReadinessDetail({
  projection,
  view,
  nowMs,
}: {
  projection: Projection<HomeReadinessView>;
  view?: HomeReadinessView;
  nowMs: number;
}) {
  const overall = view ? readDomainState("readiness", view.overall) : undefined;
  const worst = view ? worstReadinessComponent(view) : undefined;
  const checked = view ? formatAge(view.evaluatedAtMs, nowMs) : undefined;
  return (
    <section className="cp-section" aria-labelledby="system-readiness">
      <h3 className="cp-section-title" id="system-readiness">
        Readiness
      </h3>
      <HonestyNote>
        Static checks are not runtime readiness. Integrity is not-claimed. First conversation ready
        means the six components the daemon reports can support a first governed conversation — not
        that an Agent is running.
      </HonestyNote>
      <ProjectionState projection={projection} what="Readiness" />
      {view ? (
        <>
          <p>
            overall: {overall ? <StateChip reading={overall} /> : "unknown"}
            {worst ? ` — ${worst.name} ${worst.state}` : ""}
          </p>
          <p className="cp-quiet">
            first conversation:{" "}
            {view.firstConversationReady == null
              ? "unknown"
              : view.firstConversationReady
                ? "ready"
                : "not ready"}
            {checked ? ` · last check ${checked}` : " · last-checked time unknown"}
          </p>
          <ul className="cp-queue" aria-label="Readiness components">
            {expandedReadinessComponents(view).map((component) => (
              <li className="cp-queue-row" key={component.name}>
                <span className="cp-queue-object">
                  <code className="cp-mono">{component.name}</code>
                </span>
                <span>
                  <StateChip reading={readinessComponentReading(component)} />
                </span>
                <span className="cp-queue-reason">
                  {component.detail ?? (component.reported ? component.state : "not reported")}
                </span>
              </li>
            ))}
          </ul>
        </>
      ) : null}
    </section>
  );
}

function DoctorDetail({
  projection,
  view,
  nowMs,
}: {
  projection: Projection<DoctorView>;
  view?: DoctorView;
  nowMs: number;
}) {
  return (
    <section className="cp-section" aria-labelledby="system-doctor">
      <h3 className="cp-section-title" id="system-doctor">
        Doctor
      </h3>
      <HonestyNote>{SYSTEM_DOCTOR_REDACTION}</HonestyNote>
      <ProjectionState projection={projection} what="Doctor" />
      {view ? (
        <>
          <p>
            overall: <StateChip reading={readDomainState("readiness", view.overall)} />
            {view.evaluatedAtMs
              ? ` · as of ${formatAge(view.evaluatedAtMs, nowMs)}`
              : " · evaluation time unknown"}
          </p>
          {view.components.map((component) => (
            <div className="cp-subblock" key={component.name}>
              <h4 className="cp-section-title">{component.name}</h4>
              <StateChip reading={readDomainState("readiness", component.state)} />
              <FactGrid
                facts={[
                  { label: "source", value: component.source ?? "unknown" },
                  {
                    label: "observed",
                    value: formatAge(component.observedAtMs, nowMs) ?? "unknown",
                  },
                  { label: "error class", value: component.errorClass ?? "none" },
                  ...component.facts.map((fact) => ({ label: fact.key, value: fact.value })),
                ]}
              />
            </div>
          ))}
          {view.guidance.length > 0 ? (
            <ul>
              {view.guidance.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          ) : (
            <p className="cp-quiet">No doctor guidance in this projection.</p>
          )}
          <h4 className="cp-section-title">Sub-sections</h4>
          <p className="cp-quiet">
            Six-resource, headless-vault, and operability render in their true probe state. A
            NOT_PROBED error is named unavailable, not omitted to keep the page green.
          </p>
          {[
            { label: "six-resource", section: view.sixResource },
            { label: "headless-vault", section: view.headlessVault },
            { label: "operability", section: view.operability },
          ].map((entry) => (
            <p key={entry.label} data-subsection={entry.label}>
              <strong>{entry.label}</strong>: {entry.section.overall ?? "unknown"}
              {entry.section.errorCode ? ` · ${entry.section.errorCode}` : ""}
              {entry.section.probed ? "" : " — not probed over HTTP"}
            </p>
          ))}
        </>
      ) : null}
    </section>
  );
}

function StewardshipDetail() {
  const [backupReceipt, setBackupReceipt] = useState<BackupReceiptView | undefined>();
  const [restoreReceipt, setRestoreReceipt] = useState<string | undefined>();
  const [preflight, setPreflight] = useState<string | undefined>();
  const [message, setMessage] = useState<string | undefined>();
  const [archiveId, setArchiveId] = useState("");

  async function runBackup() {
    setMessage(undefined);
    const result = await readJson("/management/resource/v1/backup", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "{}",
    });
    if (!result.ok) {
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
      return;
    }
    setBackupReceipt(projectBackupReceipt(result.body));
  }

  async function runPreflight() {
    setMessage(undefined);
    setPreflight(undefined);
    const result = await readJson("/management/resource/v1/backup/preflight", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ archive_id: archiveId }),
    });
    if (!result.ok) {
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
      return;
    }
    const record = asRecord(result.body);
    setPreflight(
      record.preflight_only === true
        ? `Preflight only for ${archiveId}. Digest/compatibility checked; nothing was applied.`
        : `Preflight response for ${archiveId} did not set preflight_only.`,
    );
  }

  async function runRestore() {
    setMessage(undefined);
    const result = await readJson("/management/resource/v1/restore", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ archive_id: archiveId }),
    });
    if (!result.ok) {
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
      return;
    }
    const receipt = projectRestoreReceipt(result.body);
    setRestoreReceipt(
      receipt.liveApplied
        ? `Restore of ${receipt.archiveId ?? archiveId} live-applied. Secrets were not restored.`
        : `Restore of ${receipt.archiveId ?? archiveId} returned without live_applied=true.`,
    );
  }

  return (
    <section className="cp-section" aria-labelledby="system-stewardship">
      <h3 className="cp-section-title" id="system-stewardship">
        Stewardship
      </h3>
      <HonestyNote>
        {SYSTEM_BACKUP_NEVER} Upgrade and uninstall are class-C: use the CLI/systemd verbs, not this
        page.
      </HonestyNote>
      <ConfirmSurface
        title="Backup"
        consequences={
          <>
            Writes a secret-excluding archive of authority data, memory, skill registry, task/context
            metadata, runtime registrations and evidence. {SYSTEM_BACKUP_NEVER}
          </>
        }
        targets={["POST /management/resource/v1/backup", "body {}"]}
        confirmLabel="I confirm a secret-excluding backup"
        actionLabel="Create backup"
        onConfirm={() => void runBackup()}
      />
      {backupReceipt ? (
        <ReceiptLine>
          archive {backupReceipt.archiveId ?? "unknown"} · excluded secrets{" "}
          {backupReceipt.excludedSecretCount ?? "unknown"} · sqlite copied{" "}
          {backupReceipt.sqliteCopied == null ? "unknown" : String(backupReceipt.sqliteCopied)}
          {backupReceipt.manifestDigest ? ` · ${backupReceipt.manifestDigest}` : ""}
        </ReceiptLine>
      ) : null}
      <label className="cp-field">
        <span>Archive id</span>
        <input
          value={archiveId}
          onChange={(event) => setArchiveId(event.target.value)}
          autoComplete="off"
        />
      </label>
      <p className="cp-next">
        <button type="button" className="cp-button" onClick={() => void runPreflight()}>
          Preflight restore
        </button>
      </p>
      {preflight ? <ReceiptLine>{preflight}</ReceiptLine> : null}
      <ConfirmSurface
        title="Restore"
        consequences={<>{SYSTEM_RESTORE_409}</>}
        targets={[`archive_id ${archiveId || "(none)"}`, "POST /management/resource/v1/restore"]}
        confirmLabel="I confirm a live-apply restore of this archive id"
        actionLabel="Restore now"
        danger
        onConfirm={() => void runRestore()}
      />
      {restoreReceipt ? <ReceiptLine>{restoreReceipt}</ReceiptLine> : null}
      {message ? (
        <p role="alert" className="cp-reason">
          {message}
        </p>
      ) : null}
    </section>
  );
}

function SessionDetail() {
  const management = sessionHasChannel("management");
  const task = sessionHasChannel("task");
  return (
    <section className="cp-section" aria-labelledby="system-session">
      <h3 className="cp-section-title" id="system-session">
        Session
      </h3>
      <HonestyNote>
        Sessions are memory-only (BD-7: no daemon introspection route). Idle and absolute expiry are
        not exposed over HTTP; this page shows only what this tab holds.
      </HonestyNote>
      <FactGrid
        facts={[
          { label: "principal", value: sessionPrincipal() },
          { label: "management channel", value: management ? "held" : "absent" },
          { label: "task channel", value: task ? "held" : "absent" },
          { label: "expiry", value: "unknown (BD-7)" },
        ]}
      />
      <p className="cp-next">
        <Link to="/session">Re-authenticate</Link>
        {" · "}
        <button
          type="button"
          className="cp-button"
          onClick={() => {
            clearSession();
            window.location.hash = "#/session";
          }}
        >
          Clear this tab&apos;s session
        </button>
      </p>
    </section>
  );
}

function AboutDetail({ view }: { view?: DoctorView }) {
  return (
    <section className="cp-section" aria-labelledby="system-about">
      <h3 className="cp-section-title" id="system-about">
        About
      </h3>
      <p data-annotation="system-claim">{SYSTEM_CLAIM_CEILING}</p>
      <FactGrid
        facts={[
          { label: "gate claim", value: view?.gateClaim ?? "not-claimed" },
          { label: "profile claim", value: view?.profileClaim ?? "not-claimed" },
          {
            label: "static check is not runtime ready",
            value: view?.staticCheckIsNotRuntimeReady === false ? "false" : "true",
          },
          { label: "product version", value: "unknown (not in this projection)" },
        ]}
      />
      <p className="cp-quiet">
        Diagnostics bundles are a CLI path: <code className="cp-mono">cognitive doctor --bundle</code>.
      </p>
    </section>
  );
}
