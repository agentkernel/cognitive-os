/**
 * StandingApprovalPolicy list (P11-T13) plus Settings revoke (P12-T08).
 * Source is GET /management/project/v1/standing-policies. Revoke is POST
 * standing-policy.revoke. Chat cannot mint. Time-box is not permanent.
 * Not Inbox L1.
 */

import { asList, asRecord } from "../projections";

export const STANDING_POLICIES_KEY = "opc:standing-policies";
export const STANDING_POLICIES_PATH = "/management/project/v1/standing-policies";
export const STANDING_POLICY_REVOKE_PATH = "/management/project/v1/standing-policy.revoke";

export interface StandingPolicyRow {
  policyId: string;
  subjectClass: string;
  subjectRef: string;
  expiresAt: string;
  active: string;
}

export function projectStandingPolicies(body: unknown): StandingPolicyRow[] {
  const rows: StandingPolicyRow[] = [];
  for (const item of asList(body, ["policies"])) {
    const record = asRecord(item);
    if (typeof record.policy_id !== "string" || record.policy_id.length === 0) {
      continue;
    }
    rows.push({
      policyId: record.policy_id,
      subjectClass: typeof record.subject_class === "string" ? record.subject_class : "unknown",
      subjectRef: typeof record.subject_ref === "string" ? record.subject_ref : "unknown",
      expiresAt:
        typeof record.expires_at === "number"
          ? String(record.expires_at)
          : typeof record.expires_at === "string"
            ? record.expires_at
            : "unknown",
      active:
        record.active === true ? "true" : record.active === false ? "false" : "unknown",
    });
  }
  return rows;
}
