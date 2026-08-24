import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { FactGrid } from "../../components/FactGrid";
import { Inspector } from "../../components/Inspector";
import { MasterList, type MasterColumn } from "../../components/MasterList";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState } from "../../components/states";
import { fetchProjection } from "../../data/fetchProjection";
import {
  projectProviderAccounts,
  triageAccounts,
  type ProviderAccount,
} from "../../data/projections/providers";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";
import { AccountCreateFlow } from "./AccountCreateFlow";
import { ProjectionState } from "./ProjectionState";

export const PROVIDER_ACCOUNTS_KEY = "providers:accounts";

/**
 * Providers master — docs/design/17 §1. The list itself is the triage:
 * revoked/degraded accounts float above active ones. State chips carry
 * verbatim text labels (never color-only); secret_ref renders as presence
 * only. Not a raw JSON dump.
 */
export function ProvidersPage() {
  const projection = useProjection<ProviderAccount[]>(PROVIDER_ACCOUNTS_KEY);
  const [selectedId, setSelectedId] = useState<string | undefined>();

  const refresh = useCallback(
    () =>
      fetchProjection(
        appProjections,
        PROVIDER_ACCOUNTS_KEY,
        "/management/providers/accounts",
        "management",
        projectProviderAccounts,
      ),
    [],
  );

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const rows = triageAccounts(projection.data ?? []);
  const selected = rows.find((row) => row.id === selectedId);

  const columns: MasterColumn<ProviderAccount>[] = [
    {
      key: "state",
      header: "State",
      render: (row) => <StateChip reading={readDomainState("provider", row.status)} />,
    },
    { key: "name", header: "Name", render: (row) => row.name },
    { key: "kind", header: "Kind", mono: true, render: (row) => row.kind },
    {
      key: "catalog",
      header: "Catalog rev",
      mono: true,
      render: (row) => row.catalogRevision ?? "unknown",
    },
    {
      key: "secret",
      header: "Secret",
      render: (row) => {
        const reading = readDomainState("secret", row.secret);
        return <StateChip reading={{ ...reading, label: `secret ${row.secret}` }} />;
      },
    },
    {
      key: "error",
      header: "Last discovery error",
      render: (row) =>
        row.lastDiscoveryError ? (
          <span className="cp-reason">{row.lastDiscoveryError}</span>
        ) : (
          <span className="cp-quiet">none</span>
        ),
    },
  ];

  return (
    <>
      <PageHeader
        title="Providers"
        lede="Egress governance: named Provider accounts, their catalogs, bindings, usage, and audit. Keys never render back to this browser."
      />
      <AccountCreateFlow onCreated={() => void refresh()} />
      <ProjectionState projection={projection} what="Provider accounts" />
      {projection.status === "empty" ||
      (projection.status === "stale" && rows.length === 0) ? (
        <EmptyState title="No provider accounts yet">
          Create one above — this is not a loading placeholder. Accounts appear here once the
          daemon persists them.
        </EmptyState>
      ) : null}
      {(projection.status === "ready" || projection.status === "stale") && rows.length > 0 ? (
        <div className="cp-mi">
          <div className="cp-master">
            <MasterList
              caption="Provider accounts — triage order: revoked and degraded first; SecretRef shown only as presence"
              columns={columns}
              rows={rows}
              rowKey={(row) => row.id}
              selectedKey={selectedId}
              onSelect={(row) => setSelectedId(row.id)}
              detailHref={(row) => `#/providers/${encodeURIComponent(row.id)}`}
            />
            <p className="cp-quiet">
              Source: <code className="cp-mono">{projection.source}</code>
              {projection.updatedAt
                ? ` · updated ${new Date(projection.updatedAt).toLocaleTimeString()}`
                : ""}
            </p>
          </div>
          {selected ? (
            <Inspector title={selected.name} label="Provider account inspector">
              <FactGrid
                facts={[
                  { label: "id", value: selected.id },
                  { label: "kind", value: selected.kind },
                  { label: "status", value: selected.status },
                  { label: "network scope", value: selected.networkScope ?? "unknown" },
                  { label: "catalog revision", value: selected.catalogRevision ?? "unknown" },
                  { label: "secret", value: `secret ${selected.secret}` },
                  {
                    label: "last discovery error",
                    value: selected.lastDiscoveryError ?? "none",
                  },
                ]}
              />
              <p className="cp-next">
                <Link to={`/providers/${encodeURIComponent(selected.id)}`}>
                  Open account detail
                </Link>
              </p>
            </Inspector>
          ) : null}
        </div>
      ) : null}
    </>
  );
}
