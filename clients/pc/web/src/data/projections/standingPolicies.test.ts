import { describe, expect, it } from "vitest";
import { projectStandingPolicies } from "./standingPolicies";

describe("standing-policies projection (P11-T13)", () => {
  it("maps daemon rows and does not invent active or expiry", () => {
    const rows = projectStandingPolicies({
      status: "ok",
      policies: [
        {
          policy_id: "pol-1",
          subject_class: "grant-expansion",
          subject_ref: "proj-1",
          expires_at: 1_704_067_200_000,
          active: true,
        },
      ],
    });
    expect(rows).toEqual([
      {
        policyId: "pol-1",
        subjectClass: "grant-expansion",
        subjectRef: "proj-1",
        expiresAt: "1704067200000",
        active: "true",
      },
    ]);
  });

  it("does not invent a policy from an empty or malformed body", () => {
    expect(projectStandingPolicies({ status: "ok", policies: [] })).toEqual([]);
    expect(projectStandingPolicies({ status: "ok" })).toEqual([]);
    expect(projectStandingPolicies(null)).toEqual([]);
    expect(projectStandingPolicies({ policies: [{ active: true }] })).toEqual([]);
    expect(projectStandingPolicies({ policies: [{ policy_id: "pol-2" }] })).toEqual([
      {
        policyId: "pol-2",
        subjectClass: "unknown",
        subjectRef: "unknown",
        expiresAt: "unknown",
        active: "unknown",
      },
    ]);
  });
});
