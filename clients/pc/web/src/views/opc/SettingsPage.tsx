import { useCallback, useEffect } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  projectStandingPolicies,
  STANDING_POLICIES_KEY,
  STANDING_POLICIES_PATH,
  type StandingPolicyRow,
} from "../../data/projections/standingPolicies";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { LINUX_1_0_NAV } from "../../shell/PrimaryNav";
import { DaemonReadPanel } from "./DaemonReadPanel";

const ADVANCED_ROUTES = [
  ...LINUX_1_0_NAV.map(([to, label]) =>
    to === "/home" ? (["/home", "Linux 1.0 Home"] as const) : ([to, label] as const),
  ),
  ["/session", "Session"],
] as const;

/**
 * Settings — bottom chrome, not L1. StandingApprovalPolicy list is
 * announce-only. Linux 1.0 six-family surfaces stay in Advanced, hidden
 * by default (state-lab). Member-level budget is 2.1 / Deferred.
 */
export function SettingsPage() {
  const policies = useProjection<StandingPolicyRow[]>(STANDING_POLICIES_KEY);
  const refresh = useCallback(async () => {
    await fetchProjection(
      appProjections,
      STANDING_POLICIES_KEY,
      STANDING_POLICIES_PATH,
      "management",
      projectStandingPolicies,
    );
  }, []);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <section data-page="opc-settings">
      <PageHeader
        title="Settings"
        lede="Account, Provider, and Linux 1.0 surfaces. Not Team. Not member budget."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Member-level budget hard-stop is 2.1 / Deferred. These links are existing
        daemon-served hash routes, not a new authority plane. StandingApprovalPolicy
        create/revoke stay on management HTTP. Advanced Linux 1.0 surfaces are
        hidden by default.
      </HonestyNote>
      <DaemonReadPanel
        projection={policies}
        surface="Settings StandingApprovalPolicy"
        emptyTitle="Settings: no StandingApprovalPolicy"
        emptyBody="The daemon reports no non-revoked StandingApprovalPolicy. Chat cannot mint a time-box. This is not Inbox L1."
        region="opc-standing-policies"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {STANDING_POLICIES_PATH}</caption>
          <thead>
            <tr>
              <th>Policy</th>
              <th>Class</th>
              <th>Expires</th>
              <th>Active</th>
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
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
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
