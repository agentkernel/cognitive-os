import { useCallback, useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { readJson } from "../../api";
import { ConfirmSurface } from "../../components/ConfirmSurface";
import { FactGrid } from "../../components/FactGrid";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState, ErrorState, LoadingState, UnavailableState } from "../../components/states";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord } from "../../data/projections";
import {
  projectAuditEvents,
  projectBindings,
  projectBudgets,
  projectProviderAccountDetail,
  projectProviderAlerts,
  projectProviderModels,
  projectUsageEvents,
  type AuditEventView,
  type BindingView,
  type BudgetView,
  type ProviderAccountDetail,
  type ProviderAlertView,
  type ProviderModel,
  type UsageEventView,
} from "../../data/projections/providers";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { redactSecrets } from "../../policy";
import { capabilityDisposition } from "../../probe";
import { StateChip } from "../../state/StateChip";
import { StateDot } from "../../state/StateDot";
import { readDomainState } from "../../state/stateMap";
import { AlertsBlock } from "./AlertsBlock";
import { AuditSection } from "./AuditSection";
import { BINDINGS_KEY, BindingsSection } from "./BindingsSection";
import { KeyHandoffForm } from "./KeyHandoffForm";
import { ModelsSection } from "./ModelsSection";
import { asOfLabel } from "./ProjectionState";
import { UsageSection } from "./UsageSection";

const SECTIONS = [
  ["provider-overview", "Overview"],
  ["provider-models", "Models"],
  ["provider-bindings", "Bindings"],
  ["provider-usage", "Usage"],
  ["provider-audit", "Audit"],
] as const;

/**
 * Provider account detail — docs/design/17 §2. Five sections (Overview,
 * Models, Bindings, Usage, Audit) behind a secondary nav. Detail-not-found
 * (404 PROVIDER_ACCOUNT_NOT_FOUND), stub responses, and error envelopes
 * each get their own honest state. The raw projection is available only as
 * a collapsed secondary details element, redacted via policy.redactSecrets.
 */
export function ProviderDetailPage() {
  const { id = "" } = useParams();
  const navigate = useNavigate();
  const [message, setMessage] = useState("");

  const account = useProjection<ProviderAccountDetail>(`provider:${id}:account`);
  const models = useProjection<ProviderModel[]>(`provider:${id}:models`);
  const bindings = useProjection<BindingView[]>(BINDINGS_KEY);
  const usage = useProjection<UsageEventView[]>("usage:all");
  const budgets = useProjection<BudgetView[]>("budgets:all");
  const alerts = useProjection<ProviderAlertView[]>("alerts:all");
  const audit = useProjection<AuditEventView[]>("audit:all");

  const refreshAll = useCallback(async () => {
    if (!id) {
      return;
    }
    const enc = encodeURIComponent(id);
    await Promise.all([
      fetchProjection(
        appProjections,
        `provider:${id}:account`,
        `/management/providers/accounts/inspect?id=${enc}`,
        "management",
        projectProviderAccountDetail,
      ),
      fetchProjection(
        appProjections,
        `provider:${id}:models`,
        `/management/providers/models?account_id=${enc}`,
        "management",
        projectProviderModels,
      ),
      fetchProjection(
        appProjections,
        BINDINGS_KEY,
        "/management/agent-bindings",
        "management",
        projectBindings,
      ),
      fetchProjection(
        appProjections,
        "usage:all",
        "/management/usage",
        "management",
        projectUsageEvents,
      ),
      fetchProjection(
        appProjections,
        "budgets:all",
        "/management/budgets",
        "management",
        projectBudgets,
      ),
      fetchProjection(
        appProjections,
        "alerts:all",
        "/management/alerts",
        "management",
        projectProviderAlerts,
      ),
      fetchProjection(
        appProjections,
        "audit:all",
        "/management/audit",
        "management",
        projectAuditEvents,
      ),
    ]);
  }, [id]);

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  async function deleteAccount() {
    const result = await readJson("/management/providers/accounts/delete", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ id }),
    });
    if (result.ok) {
      navigate("/providers");
      return;
    }
    setMessage(
      `HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}. Active Agent bindings block delete.`,
    );
  }

  if (!id) {
    return (
      <EmptyState title="No account selected" action={<Link to="/providers">Back to Providers</Link>}>
        The route carries no account id.
      </EmptyState>
    );
  }

  const notFound =
    account.status === "unknown" && account.error?.code === "PROVIDER_ACCOUNT_NOT_FOUND";

  const detail = account.data;
  const activeBindingsHere = (bindings.data ?? []).filter(
    (row) => row.accountId === id && row.status === "active",
  );

  return (
    <>
      <PageHeader
        title={detail ? detail.name : "Provider account"}
        lede={
          detail
            ? undefined
            : "Account facts load from the daemon; nothing is inferred."
        }
      />
      {account.status === "loading" ? (
        <LoadingState label="Fetching the account projection from the daemon." />
      ) : null}
      {notFound ? (
        <section className="cp-stateview" aria-label="Provider account not found">
          <h3>
            <StateDot category="unknown" /> No such provider account
          </h3>
          <p>
            The daemon reports <code className="cp-mono">PROVIDER_ACCOUNT_NOT_FOUND</code> for
            this id. Nothing was changed.
          </p>
          <p className="cp-next">
            <Link to="/providers">Back to Providers</Link>
          </p>
        </section>
      ) : null}
      {account.status === "not-run" ? (
        <UnavailableState
          what="Provider account inspect"
          dependency="daemon front-door stub (R-1)"
        />
      ) : null}
      {account.status === "denied" ? (
        <ErrorState
          what="Provider account: session denied"
          why={
            <>
              HTTP {account.error?.httpStatus} ·{" "}
              <code className="cp-mono">{account.error?.code ?? "denied"}</code> on the
              management channel
            </>
          }
          next={<Link to="/session">Open Session</Link>}
          retryable={false}
        />
      ) : null}
      {account.status === "disconnected" ? (
        <ErrorState
          what="Provider account: daemon unreachable"
          why="The daemon did not answer; the last known state is not shown as current."
          retryable
        />
      ) : null}
      {account.status === "unknown" && !notFound ? (
        <ErrorState
          what="Provider account: unexpected response"
          why={
            <code className="cp-mono">{account.error?.code ?? "unknown"}</code>
          }
          retryable
        />
      ) : null}
      {detail && (account.status === "ready" || account.status === "stale") ? (
        <>
          <p className="cp-quiet">
            <StateChip reading={readDomainState("provider", detail.status)} /> ·{" "}
            <code className="cp-mono">{detail.kind}</code> · secret {detail.secret} · catalog
            rev {detail.catalogRevision ?? "unknown"} · Source:{" "}
            <code className="cp-mono">{account.source}</code>
            {account.updatedAt
              ? ` · updated ${new Date(account.updatedAt).toLocaleTimeString()}`
              : ""}
            {account.status === "stale" ? (
              <>
                {" · "}
                <StateChip reading={readDomainState("load", "stale")} /> last good as of{" "}
                {asOfLabel(account.updatedAt)}; a refresh is in flight
              </>
            ) : null}
          </p>
          <nav aria-label="Account sections" className="cp-subnav">
            {SECTIONS.map(([sectionId, label]) => (
              <button
                key={sectionId}
                type="button"
                className="cp-button"
                onClick={() => document.getElementById(sectionId)?.scrollIntoView?.()}
              >
                {label}
              </button>
            ))}
          </nav>

          <section id="provider-overview" aria-labelledby="provider-overview-title" className="cp-section">
            <h3 id="provider-overview-title" className="cp-section-title">
              Overview
            </h3>
            <FactGrid
              facts={[
                { label: "id", value: detail.id },
                { label: "kind", value: detail.kind },
                { label: "endpoint", value: detail.endpoint ?? "unknown" },
                { label: "network scope", value: detail.networkScope ?? "unknown" },
                {
                  label: "trust grants",
                  value: `allow_private_network: ${String(detail.allowPrivateNetwork ?? "unknown")} · allow_insecure_http: ${String(detail.allowInsecureHttp ?? "unknown")}`,
                },
                {
                  label: "catalog revision",
                  value: detail.catalogRevision ?? "unknown",
                },
                { label: "secret", value: `secret ${detail.secret}` },
                {
                  label: "last discovery error",
                  value: detail.lastDiscoveryError ?? "none",
                },
                {
                  label: "capability probe",
                  value: `${capabilityDisposition(undefined)} (bounded capability checks are not exposed)`,
                },
              ]}
            />
            <KeyHandoffForm
              accountId={id}
              presence={detail.secret}
              onDone={() => void refreshAll()}
            />
            <ConfirmSurface
              title="Delete account"
              consequences={
                activeBindingsHere.length > 0
                  ? `Active bindings block delete: ${activeBindingsHere
                      .map((row) => `${row.agent} → ${row.modelId}`)
                      .join(", ")}. Deleting is separate from repairing and is never the suggested recovery.`
                  : "No active bindings reference this account. Deleting is separate from repairing and is never the suggested recovery."
              }
              targets={[`account: ${id}`]}
              confirmLabel="Confirm deleting this account"
              actionLabel="Delete account"
              danger
              onConfirm={() => void deleteAccount()}
            />
            {message ? <p role="status">{message}</p> : null}
            <details className="cp-details">
              <summary>Raw projection</summary>
              <pre className="cp-mono">{JSON.stringify(redactSecrets(detail), null, 2)}</pre>
            </details>
          </section>

          <section id="provider-models" aria-labelledby="provider-models-title" className="cp-section">
            <h3 id="provider-models-title" className="cp-section-title">
              Models
            </h3>
            <ModelsSection
              accountId={id}
              models={models}
              onChanged={() => void refreshAll()}
            />
          </section>

          <section id="provider-bindings" aria-labelledby="provider-bindings-title" className="cp-section">
            <h3 id="provider-bindings-title" className="cp-section-title">
              Bindings
            </h3>
            <BindingsSection
              accountId={id}
              accountStatus={detail.status}
              bindings={bindings}
              models={models.data ?? []}
              onChanged={() => void refreshAll()}
            />
          </section>

          <section id="provider-usage" aria-labelledby="provider-usage-title" className="cp-section">
            <h3 id="provider-usage-title" className="cp-section-title">
              Usage
            </h3>
            <UsageSection accountId={id} usage={usage} budgets={budgets} />
            <AlertsBlock
              accountId={id}
              alerts={alerts}
              budgets={budgets}
              onChanged={() => void refreshAll()}
            />
          </section>

          <section id="provider-audit" aria-labelledby="provider-audit-title" className="cp-section">
            <h3 id="provider-audit-title" className="cp-section-title">
              Audit
            </h3>
            <AuditSection accountId={id} audit={audit} />
          </section>
        </>
      ) : null}
    </>
  );
}
