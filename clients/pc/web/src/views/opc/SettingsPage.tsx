import { useCallback, useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  projectProviderAccounts,
  projectUsageEvents,
  triageAccounts,
  type ProviderAccount,
  type UsageEventView,
} from "../../data/projections/providers";
import { connectionUsageLabel } from "../../data/projections/connectionUsage";
import {
  hostCanHonorBackground,
  hostStatusPath,
  projectHostStatus,
  HOST_CLOSE_PATH,
  HOST_STATUS_KEY,
  type HostStatusRow,
} from "../../data/projections/host";
import {
  projectStandingPolicies,
  STANDING_POLICIES_KEY,
  STANDING_POLICIES_PATH,
  STANDING_POLICY_REVOKE_PATH,
  type StandingPolicyRow,
} from "../../data/projections/standingPolicies";
import { appProjections } from "../../data/store";
import type { Projection } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { LINUX_1_0_NAV } from "../../shell/PrimaryNav";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { httpErrorMessage } from "./httpError";

const ADVANCED_ROUTES = [
  ...LINUX_1_0_NAV.map(([to, label]) =>
    to === "/home" ? (["/home", "Linux 1.0 Home"] as const) : ([to, label] as const),
  ),
  ["/session", "Session"],
] as const;

export const OPC_CONNECTIONS_KEY = "opc:connections";
export const OPC_USAGE_KEY = "opc:usage";
export const OPC_ACCOUNTS_PATH = "/management/providers/accounts";
export const OPC_USAGE_PATH = "/management/usage";

/**
 * Settings — connections + retractable 「本周不再问」 + CloseBackgroundDialog
 * (P12-T08) on daemon `/ui/`. Chat cannot mint. Unknown usage is never 0.
 * Native close/host E2E is not-run.
 */
export function SettingsPage() {
  const [params] = useSearchParams();
  const homeId = (params.get("home") ?? "").trim();
  const accounts = useProjection<ProviderAccount[]>(OPC_CONNECTIONS_KEY);
  const usage = useProjection<UsageEventView[]>(OPC_USAGE_KEY);
  const policies = useProjection<StandingPolicyRow[]>(STANDING_POLICIES_KEY);
  const host = useProjection<HostStatusRow[]>(HOST_STATUS_KEY);
  const refresh = useCallback(async () => {
    await Promise.all([
      fetchProjection(
        appProjections,
        OPC_CONNECTIONS_KEY,
        OPC_ACCOUNTS_PATH,
        "management",
        projectProviderAccounts,
      ),
      fetchProjection(appProjections, OPC_USAGE_KEY, OPC_USAGE_PATH, "management", projectUsageEvents),
      fetchProjection(
        appProjections,
        STANDING_POLICIES_KEY,
        STANDING_POLICIES_PATH,
        "management",
        projectStandingPolicies,
      ),
    ]);
    if (homeId.length > 0) {
      await fetchProjection(
        appProjections,
        HOST_STATUS_KEY,
        hostStatusPath(homeId),
        "management",
        projectHostStatus,
      );
    }
  }, [homeId]);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <section data-page="opc-settings">
      <PageHeader
        title="Settings"
        lede="Connections, retractable don't-ask-this-week, and close-background. Not Team. Not member budget."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Member-level budget hard-stop is 2.1 / Deferred. Connection usage unknown
        is never 0. StandingApprovalPolicy is a time-box, not a permanent Don't
        ask. Chat cannot mint. CloseBackground uses GET host/v1/status then POST
        close.request. Native close/host E2E is not-run. Advanced Linux 1.0
        surfaces are hidden by default.
      </HonestyNote>
      <ConnectionsTable accounts={accounts} usage={usage} />
      <StandingPolicyTable policies={policies} onRevoked={refresh} />
      <CloseBackgroundDialog homeId={homeId} host={host} onClosed={refresh} />
      <details className="cp-details" data-region="opc-settings-advanced">
        <summary>Advanced — Linux 1.0 surfaces (hidden by default)</summary>
        <p className="cp-quiet">
          These are real daemon-served hash routes from Linux 1.0. They are not
          Personal 2.0 L1. Not Team. Not Inbox.
        </p>
        <ul className="cp-nav">
          {ADVANCED_ROUTES.map(([to, label]) => (
            <li key={to}>
              <Link to={to}>{label}</Link>
            </li>
          ))}
        </ul>
      </details>
    </section>
  );
}

function ConnectionsTable({
  accounts,
  usage,
}: {
  accounts: Projection<ProviderAccount[]>;
  usage: Projection<UsageEventView[]>;
}) {
  const rows = triageAccounts(accounts.data ?? []);
  const events = usage.data ?? [];
  return (
    <DaemonReadPanel
      projection={accounts}
      surface="Settings connections"
      emptyTitle="Settings: no model connection"
      emptyBody="The daemon reports no Provider account. Empty Settings is not connected yet. Open Providers to connect. This table does not invent a connection."
      region="opc-connections"
    >
      <table className="cp-table">
        <caption className="cp-quiet">
          GET {OPC_ACCOUNTS_PATH} + GET {OPC_USAGE_PATH}. Unknown is never 0.
          Secret presence only.{" "}
          <Link to="/providers">Open Providers</Link>
        </caption>
        <thead>
          <tr>
            <th>Account</th>
            <th>Kind</th>
            <th>Status</th>
            <th>Secret</th>
            <th>Usage</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.id} data-row-key={row.id}>
              <td>
                <Link to={`/providers/${encodeURIComponent(row.id)}`}>
                  <code className="cp-mono">{row.id}</code>
                </Link>
              </td>
              <td>{row.kind}</td>
              <td>{row.status}</td>
              <td>{row.secret}</td>
              <td data-usage={row.id}>{connectionUsageLabel(row.id, events)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </DaemonReadPanel>
  );
}

function StandingPolicyTable({
  policies,
  onRevoked,
}: {
  policies: Projection<StandingPolicyRow[]>;
  onRevoked: () => Promise<void>;
}) {
  const [busyId, setBusyId] = useState<string | undefined>();
  const [error, setError] = useState<string | undefined>();

  async function revoke(policyId: string) {
    setBusyId(policyId);
    setError(undefined);
    try {
      const written = await readJson(STANDING_POLICY_REVOKE_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ policy_id: policyId }),
      });
      if (!written.ok) {
        setError(httpErrorMessage(written.status, written.body));
        return;
      }
      await onRevoked();
    } finally {
      setBusyId(undefined);
    }
  }

  return (
    <>
      <DaemonReadPanel
        projection={policies}
        surface="Settings StandingApprovalPolicy"
        emptyTitle="Settings: no StandingApprovalPolicy"
        emptyBody="The daemon reports no non-revoked StandingApprovalPolicy. Chat cannot mint a time-box. Retract is not a permanent Don't ask. This is not Inbox L1."
        region="opc-standing-policies"
      >
        <table className="cp-table">
          <caption className="cp-quiet">
            GET {STANDING_POLICIES_PATH}. Revoke is POST {STANDING_POLICY_REVOKE_PATH}.
            Time-box only; not permanent.
          </caption>
          <thead>
            <tr>
              <th>Policy</th>
              <th>Class</th>
              <th>Expires</th>
              <th>Active</th>
              <th>Retract</th>
            </tr>
          </thead>
          <tbody>
            {(policies.data ?? []).map((row) => (
              <tr key={row.policyId} data-row-key={row.policyId}>
                <td>
                  <code className="cp-mono">{row.policyId}</code>
                </td>
                <td>{row.subjectClass}</td>
                <td>{row.expiresAt}</td>
                <td>{row.active}</td>
                <td>
                  <button
                    type="button"
                    className="cp-button"
                    disabled={busyId === row.policyId}
                    onClick={() => void revoke(row.policyId)}
                  >
                    Retract this week
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
      {error ? (
        <p className="cp-error" data-region="opc-standing-revoke-error">
          {error} The original policy list stays.
        </p>
      ) : null}
    </>
  );
}

function CloseBackgroundDialog({
  homeId,
  host,
  onClosed,
}: {
  homeId: string;
  host: Projection<HostStatusRow[]>;
  onClosed: () => Promise<void>;
}) {
  const [busy, setBusy] = useState<"background" | "pause" | undefined>();
  const [error, setError] = useState<string | undefined>();
  const row = host.data?.[0];
  const honor = hostCanHonorBackground(row);

  async function close(choice: "background" | "pause") {
    if (homeId.length === 0) {
      return;
    }
    if (choice === "background" && !honor) {
      setError(
        "Daemon cannot honor background. Fake background is not posted. Pause remains available when the host is bound.",
      );
      return;
    }
    setBusy(choice);
    setError(undefined);
    try {
      const written = await readJson(HOST_CLOSE_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ home_id: homeId, choice }),
      });
      if (!written.ok) {
        setError(`${httpErrorMessage(written.status, written.body)} The original close status stays.`);
        return;
      }
      await onClosed();
    } finally {
      setBusy(undefined);
    }
  }

  if (homeId.length === 0) {
    return (
      <div data-region="opc-close-background">
        <h2 className="cp-section-title">Close background</h2>
        <p className="cp-quiet">
          No home_id on this Settings session. CloseBackgroundDialog does not
          invent a Personal Home. Native close/host E2E is not-run
          (DEV-WINDOWS-NATIVE-OPC-01). Requires-environment.
        </p>
      </div>
    );
  }

  return (
    <div data-region="opc-close-background">
      <h2 className="cp-section-title">Close background</h2>
      <DaemonReadPanel
        projection={host}
        surface="Windows host close"
        emptyTitle="Settings: host status empty"
        emptyBody="GET host/v1/status did not return a home. CloseBackground does not invent a background."
        region="opc-close-background-status"
      >
        <table className="cp-table">
          <caption className="cp-quiet">
            GET {hostStatusPath(homeId)} then POST {HOST_CLOSE_PATH}. Tray does
            not write authority.
          </caption>
          <thead>
            <tr>
              <th>Home</th>
              <th>Daemon</th>
              <th>State</th>
              <th>Can honor background</th>
              <th>Disposition</th>
              <th>Tray proves work</th>
            </tr>
          </thead>
          <tbody>
            {row ? (
              <tr data-row-key={row.homeId}>
                <td>
                  <code className="cp-mono">{row.homeId}</code>
                </td>
                <td>
                  <code className="cp-mono">{row.daemonId}</code>
                </td>
                <td>{row.daemonState}</td>
                <td>{row.canHonorBackground}</td>
                <td>{row.closeDisposition}</td>
                <td>{row.trayProvesWork}</td>
              </tr>
            ) : null}
          </tbody>
        </table>
        <p className="cp-quiet">
          Continue eligible work in background, or Pause. The choice is explicit.
          Host shutdown never implies 24/7 work.
        </p>
        <div className="cp-actions">
          <button
            type="button"
            className="cp-button"
            disabled={busy !== undefined || !honor}
            onClick={() => void close("background")}
          >
            Continue in background
          </button>
          <button
            type="button"
            className="cp-button"
            disabled={busy !== undefined}
            onClick={() => void close("pause")}
          >
            Pause
          </button>
        </div>
      </DaemonReadPanel>
      {error ? (
        <p className="cp-error" data-region="opc-close-error">
          {error}
        </p>
      ) : null}
    </div>
  );
}
