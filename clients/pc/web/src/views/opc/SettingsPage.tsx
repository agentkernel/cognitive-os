import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { readJson, rejectCallerHeaderInjection } from "../../api";
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
import { StateLabGrid } from "./StateLabPage";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { httpErrorMessage } from "./httpError";

const ADVANCED_ROUTES = [
  ...LINUX_1_0_NAV.map(([to, label]) => [to, label] as const),
  ["/session", "Session"],
] as const;

export const OPC_CONNECTIONS_KEY = "opc:connections";
export const OPC_USAGE_KEY = "opc:usage";
export const OPC_ACCOUNTS_PATH = "/management/providers/accounts";
export const OPC_USAGE_PATH = "/management/usage";
export const OPC_CONNECT_PATH = "/management/settings/v1/connection.connect";
export const OPC_DIAGNOSTICS_PATH = "/management/settings/v1/diagnostics";
export const OPC_NOTIFICATIONS_PATH = "/management/settings/v1/notifications";
export const OPC_DIAGNOSTICS_KEY = "opc:settings-diagnostics";
export const OPC_NOTIFICATIONS_KEY = "opc:settings-notifications";

const TEMPLATES = [
  { id: "openai", label: "OpenAI" },
  { id: "anthropic", label: "Anthropic" },
  { id: "deepseek", label: "DeepSeek" },
  { id: "custom", label: "自定义" },
] as const;

export type SettingsDiagnostics = {
  dshFacts: string;
  dshHealth: string;
  dshRevision: string;
  dshUpdate: string;
  dshRollback: string;
  piFacts: string;
  piVersion: string;
  piHealth: string;
};

export type SettingsNotificationGroup = {
  missed: string[];
  offline: string[];
  resume: string[];
};

export function projectSettingsDiagnostics(body: unknown): SettingsDiagnostics {
  const record = asObject(body);
  const dsh = asObject(record.dsh);
  const pi = asObject(record.pi);
  return {
    dshFacts: stringOrEmpty(dsh.facts, "empty"),
    dshHealth: stringOrEmpty(dsh.health, "empty"),
    dshRevision: stringOrEmpty(dsh.expected_revision, "empty"),
    dshUpdate: stringOrEmpty(dsh.update, "empty"),
    dshRollback: stringOrEmpty(dsh.rollback, "empty"),
    piFacts: stringOrEmpty(pi.facts, "empty"),
    piVersion: stringOrEmpty(pi.exact_version, "empty"),
    piHealth: stringOrEmpty(pi.health, "empty"),
  };
}

export function projectSettingsNotifications(body: unknown): SettingsNotificationGroup {
  const record = asObject(body);
  return {
    missed: groupDetails(record.missed),
    offline: groupDetails(record.offline),
    resume: groupDetails(record.resume),
  };
}

function asObject(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function stringOrEmpty(value: unknown, fallback: string): string {
  return typeof value === "string" && value.length > 0 ? value : fallback;
}

function groupDetails(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.map((row) => {
    const record = asObject(row);
    return stringOrEmpty(record.detail, stringOrEmpty(record.kind, "unknown"));
  });
}

function notificationsPath(homeId: string): string {
  return homeId.length > 0
    ? `${OPC_NOTIFICATIONS_PATH}?home_id=${encodeURIComponent(homeId)}`
    : OPC_NOTIFICATIONS_PATH;
}

/**
 * Settings — 设置 / 模型连接 through SecretStore, retractable
 * 「本周不再问」, CloseBackgroundDialog, notification/recovery groups,
 * collapsed diagnostics, and hidden state-lab (P13-T08 / P14-T07 / P15-T05)
 * on daemon `/ui/`. Chat cannot mint. Unknown usage is never 0. No `/providers`
 * detour. Linux 1.0 Home/Work/Agents/Providers hashes are retired (No such route).
 */
export function SettingsPage() {
  const [params] = useSearchParams();
  const homeId = (params.get("home") ?? "").trim();
  const [stateLabOpen, setStateLabOpen] = useState(false);
  const accounts = useProjection<ProviderAccount[]>(OPC_CONNECTIONS_KEY);
  const usage = useProjection<UsageEventView[]>(OPC_USAGE_KEY);
  const policies = useProjection<StandingPolicyRow[]>(STANDING_POLICIES_KEY);
  const host = useProjection<HostStatusRow[]>(HOST_STATUS_KEY);
  const diagnostics = useProjection<SettingsDiagnostics>(OPC_DIAGNOSTICS_KEY);
  const notifications = useProjection<SettingsNotificationGroup>(OPC_NOTIFICATIONS_KEY);
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
      fetchProjection(
        appProjections,
        OPC_DIAGNOSTICS_KEY,
        OPC_DIAGNOSTICS_PATH,
        "management",
        projectSettingsDiagnostics,
      ),
      fetchProjection(
        appProjections,
        OPC_NOTIFICATIONS_KEY,
        notificationsPath(homeId),
        "management",
        projectSettingsNotifications,
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
        title="设置"
        lede="连接模型 · 收回本周不再问 · 通知恢复。无账单、无引擎商店、无收件箱。"
      />
      <HonestyNote placement="secondary">
        Product origin is daemon-served hash /ui/. Member-level budget hard-stop
        is 2.1 / Deferred. Connection usage unknown is never 0.
        StandingApprovalPolicy is a time-box, not a permanent Don't ask. Chat
        cannot mint. CloseBackground uses GET host/v1/status then POST
        close.request. Native close/host/SecretStore E2E is not-run. Advanced
        diagnostics and state-lab stay hidden by default. Linux 1.0 Home /
        Work / Agents / Providers hashes are retired; they render No such
        route, same as #/inbox.
      </HonestyNote>
      <ModelConnectionsForm onConnected={refresh} />
      <ConnectionsTable accounts={accounts} usage={usage} />
      <NotificationGroups notifications={notifications} />
      <StandingPolicyTable policies={policies} onRevoked={refresh} />
      <CloseBackgroundDialog homeId={homeId} host={host} onClosed={refresh} />
      <details className="cp-details" data-region="opc-settings-diagnostics">
        <summary>Advanced diagnostics — DSH / Pi (hidden by default)</summary>
        <DiagnosticsPanel diagnostics={diagnostics} />
      </details>
      <details
        className="cp-details"
        data-region="opc-settings-state-lab"
        open={stateLabOpen}
        onToggle={(event) => setStateLabOpen(event.currentTarget.open)}
      >
        <summary>Advanced — state-lab nine × nine (hidden by default)</summary>
        <p className="cp-quiet">
          Real `/ui/` components. Not a first-level destination. Not Installed
          Agents. Unknown is never 0. Closed: the 9×9 grid is unmounted so it
          is not in the accessibility tree.
        </p>
        {stateLabOpen ? <StateLabGrid /> : null}
      </details>
      <details className="cp-details" data-region="opc-settings-advanced">
        <summary>Advanced — remaining daemon surfaces (hidden by default)</summary>
        <p className="cp-quiet">
          Resources / Activity / System remain secondary daemon surfaces. They
          are not Personal 2.0 L1. Linux 1.0 Home / Work / Agents / Providers
          hashes are retired and are not listed. Not Team. Not Inbox.
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

function ModelConnectionsForm({ onConnected }: { onConnected: () => Promise<void> }) {
  const [template, setTemplate] = useState<(typeof TEMPLATES)[number]["id"]>("openai");
  const [hasKey, setHasKey] = useState(false);
  const [hasCustomUrl, setHasCustomUrl] = useState(false);
  const [allowPrivate, setAllowPrivate] = useState(false);
  const [allowInsecure, setAllowInsecure] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();
  const canSubmit = hasKey && (template !== "custom" || hasCustomUrl);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    const apiKey = String(data.get("api_key") ?? "");
    const customUrl = String(data.get("endpoint") ?? "").trim();
    if (apiKey.trim().length === 0 || (template === "custom" && customUrl.length === 0)) {
      setError("Key is required. Custom URL is required in compatible mode. Fake Connect is not posted.");
      return;
    }
    const body = {
      template,
      display_name: String(data.get("display_name") ?? "").trim() || undefined,
      endpoint: template === "custom" ? customUrl : undefined,
      model: String(data.get("model") ?? "").trim() || undefined,
      api_key: apiKey,
      allow_private_network: allowPrivate,
      allow_insecure_http: allowInsecure,
    };
    const keyField = form.querySelector("input[name='api_key']") as HTMLInputElement | null;
    if (keyField) {
      keyField.value = "";
    }
    setHasKey(false);
    rejectCallerHeaderInjection(body);
    setBusy(true);
    setError(undefined);
    setReceipt(undefined);
    try {
      const written = await readJson(OPC_CONNECT_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!written.ok) {
        setError(
          `${httpErrorMessage(written.status, written.body)} Connection failed. The key was cleared and never rendered.`,
        );
        return;
      }
      const record = asObject(asObject(written.body).connection);
      const id = stringOrEmpty(record.id, "unknown");
      const status = stringOrEmpty(record.connection_status, "failed");
      setReceipt(`Account ${id} ${status}. Secret presence only. Windows SecretStore host E2E is not-run.`);
      await onConnected();
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="cp-panel" data-region="opc-model-connections">
      <h2 className="cp-section-title">模型连接</h2>
      <form
        onSubmit={(event) => {
          void submit(event);
        }}
      >
        <p className="cp-quiet">
          主流下拉 + 自定义 URL / 兼容 / 模型。Owner 输入密钥。密钥一次性交给
          daemon SecretStore，这里立刻清空。已交接 / 失败都不回显明文。没有假
          Connect，也不打开 Linux 1.0 Providers 页。
        </p>
        <label className="cp-field">
          <span>供应商模板</span>
          <select
            name="template"
            value={template}
            onChange={(event) => setTemplate(event.target.value as (typeof TEMPLATES)[number]["id"])}
          >
            {TEMPLATES.map((item) => (
              <option key={item.id} value={item.id}>
                {item.label}
              </option>
            ))}
          </select>
        </label>
        <label className="cp-field">
          <span>显示名</span>
          <input name="display_name" />
        </label>
        {template === "custom" ? (
          <label className="cp-field">
            <span>自定义 URL</span>
            <input
              name="endpoint"
              placeholder="https://…"
              onInput={(event) => setHasCustomUrl(event.currentTarget.value.trim().length > 0)}
            />
          </label>
        ) : null}
        <label className="cp-field">
          <span>模型名</span>
          <input name="model" />
        </label>
        <label className="cp-field">
          <span>密钥（一次性交接）</span>
          <input
            name="api_key"
            type="password"
            autoComplete="off"
            placeholder="输入后交接，界面不回显…"
            onInput={(event) => setHasKey(event.currentTarget.value.trim().length > 0)}
          />
        </label>
        {template === "custom" ? (
          <>
            <label className="cp-field">
              <input
                type="checkbox"
                checked={allowPrivate}
                onChange={(event) => setAllowPrivate(event.target.checked)}
              />{" "}
              允许私网
            </label>
            <label className="cp-field">
              <input
                type="checkbox"
                checked={allowInsecure}
                onChange={(event) => setAllowInsecure(event.target.checked)}
              />{" "}
              允许明文 HTTP
            </label>
          </>
        ) : null}
        <button type="submit" className="cp-button cp-button--primary" disabled={busy || !canSubmit}>
          交接密钥到 SecretStore
        </button>
        {!canSubmit ? (
          <p className="cp-quiet">需要密钥。自定义模式还要 URL。没有假 Connect。</p>
        ) : null}
      </form>
      {receipt ? <p className="cp-receipt">{receipt}</p> : null}
      {error ? (
        <p className="cp-error" data-region="opc-connection-error">
          {error}
        </p>
      ) : null}
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
      emptyTitle="设置：还没有模型连接"
      emptyBody="Daemon 报告没有 Provider 账户。空设置不是已连接。用上面的模型连接交接密钥。这张表不发明连接。"
      region="opc-connections"
    >
      <table className="cp-table">
        <caption className="cp-quiet">
          GET accounts + GET usage. Usage is actual / estimated / unknown.
          Unknown is never 0. Secret presence only.
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
                <code className="cp-mono">{row.id}</code>
              </td>
              <td>{row.kind}</td>
              <td data-connection-status={row.id}>
                {row.status === "active" ? "connected" : row.status === "revoked" || row.status === "degraded" ? "failed" : row.status}
              </td>
              <td>{row.secret}</td>
              <td data-usage={row.id}>{connectionUsageLabel(row.id, events)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </DaemonReadPanel>
  );
}

function NotificationGroups({
  notifications,
}: {
  notifications: Projection<SettingsNotificationGroup>;
}) {
  const groups = notifications.data ?? { missed: [], offline: [], resume: [] };
  return (
    <div data-region="opc-settings-notifications">
      <h2 className="cp-section-title">通知与恢复</h2>
      <p className="cp-quiet">
        故障时才需要更深的运行信息。默认不展示底层引擎名。空组不是发明出来的事件。Windows host E2E is not-run.
      </p>
      <section>
        <h3 className="cp-section-title">Missed</h3>
        <FactList items={groups.missed} empty="No missed facts." />
      </section>
      <section>
        <h3 className="cp-section-title">Offline</h3>
        <FactList items={groups.offline} empty="No offline facts." />
      </section>
      <section>
        <h3 className="cp-section-title">Resume</h3>
        <FactList items={groups.resume} empty="No resume facts." />
      </section>
    </div>
  );
}

function FactList({ items, empty }: { items: string[]; empty: string }) {
  if (items.length === 0) {
    return <p className="cp-quiet">{empty}</p>;
  }
  return (
    <ul>
      {items.map((item) => (
        <li key={item}>{item}</li>
      ))}
    </ul>
  );
}

function DiagnosticsPanel({ diagnostics }: { diagnostics: Projection<SettingsDiagnostics> }) {
  const row = diagnostics.data;
  return (
    <div data-region="opc-settings-diagnostics-facts">
      <p className="cp-quiet">
        GET {OPC_DIAGNOSTICS_PATH}. DSH / Pi exact version, health, update, and
        rollback are empty when no engine health fact exists. P13-T02 is not a
        mutex.
      </p>
      <table className="cp-table">
        <caption className="cp-quiet">Honest empty when facts are absent.</caption>
        <thead>
          <tr>
            <th>Engine</th>
            <th>Facts</th>
            <th>Exact version</th>
            <th>Health</th>
            <th>Update</th>
            <th>Rollback</th>
          </tr>
        </thead>
        <tbody>
          <tr data-diagnostics-engine="dsh">
            <td>DSH</td>
            <td>{row?.dshFacts ?? "empty"}</td>
            <td>
              <code className="cp-mono">{row?.dshRevision ?? "empty"}</code>
            </td>
            <td>{row?.dshHealth ?? "empty"}</td>
            <td>{row?.dshUpdate ?? "empty"}</td>
            <td>{row?.dshRollback ?? "empty"}</td>
          </tr>
          <tr data-diagnostics-engine="pi">
            <td>Pi</td>
            <td>{row?.piFacts ?? "empty"}</td>
            <td>
              <code className="cp-mono">{row?.piVersion ?? "empty"}</code>
            </td>
            <td>{row?.piHealth ?? "empty"}</td>
            <td>empty</td>
            <td>empty</td>
          </tr>
        </tbody>
      </table>
    </div>
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
      <h2 className="cp-section-title">本周不再问</h2>
      <DaemonReadPanel
        projection={policies}
        surface="Settings StandingApprovalPolicy"
        emptyTitle="本周不再问：当前没有有效的时间盒跳过。"
        emptyBody="Daemon 报告没有未收回的 StandingApprovalPolicy。聊天不能铸造时间盒。收回不是永久 Don't ask。这不是 Inbox L1。"
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
                    收回跳过
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
