import { describe, expect, it } from "vitest";
import {
  acceptanceOfferable,
  lastRingStageId,
  projectOutputDetail,
  projectOutputs,
  projectPublicationPacket,
} from "./outputs";

const ARTIFACT = {
  artifact_id: "artifact-1",
  attempt_id: "dshattempt-1",
  project_id: "proj-1",
  task_ref: "task://personal/p13-t04",
  employee_id: "emp-2",
  cas_ref: `sha256:${"a".repeat(64)}`,
  byte_length: 42,
  format: "text/markdown",
  source: "hosted-dsh-child:candidate:DeliverableDraft",
  source_frame_seq: 2,
  freshness: "current",
  verification_status: "passed",
  latest_evidence_id: "evidence-1",
  stage_id: "s2",
  accepted_at: null,
  produced_at: 60,
};

describe("outputs projections (P13-T04)", () => {
  it("maps daemon artifacts and never invents one without an artifact_id", () => {
    const rows = projectOutputs({ status: "ok", artifacts: [ARTIFACT, { cas_ref: "x" }] });
    expect(rows).toHaveLength(1);
    expect(rows[0]).toMatchObject({
      artifactId: "artifact-1",
      casRef: `sha256:${"a".repeat(64)}`,
      freshness: "current",
      verificationStatus: "passed",
      stageId: "s2",
      acceptedAt: "none",
      byteLength: "42",
    });
    expect(projectOutputs(null)).toEqual([]);
  });

  it("keeps verification not-run when the daemon states nothing", () => {
    const rows = projectOutputs({
      status: "ok",
      artifacts: [{ ...ARTIFACT, verification_status: undefined, latest_evidence_id: null }],
    });
    expect(rows[0].verificationStatus).toBe("not-run");
    expect(rows[0].latestEvidenceId).toBe("none");
  });

  it("maps detail with evidence criteria and the export copy as non-authority", () => {
    const rows = projectOutputDetail({
      status: "ok",
      artifact: ARTIFACT,
      evidence: [
        {
          evidence_id: "evidence-1",
          verifier_ref: "verifier://personal/attempt-artifact",
          principal: "principal://personal/independent-verifier",
          disposition: "passed",
          criteria: [
            { id: "cas-bytes-match-digest", result: "pass" },
            { id: "attempt-response-status", result: "not-used" },
          ],
          report_cas_ref: `sha256:${"b".repeat(64)}`,
          checked_cas_ref: `sha256:${"a".repeat(64)}`,
          verified_at: 90,
        },
      ],
      run_acceptance: null,
      open_route: "/management/project/v1/outputs.open?artifact_id=artifact-1",
      export: { exists: false, path: "/home/x/data/projects/proj-1/outputs/artifact-1.md", is_authority: false },
      files_are_authority: false,
    });
    expect(rows).toHaveLength(1);
    expect(rows[0].evidence[0].criteria).toEqual([
      { id: "cas-bytes-match-digest", result: "pass" },
      { id: "attempt-response-status", result: "not-used" },
    ]);
    expect(rows[0].acceptanceId).toBe("none");
    expect(rows[0].exportExists).toBe("false");
    expect(rows[0].filesAreAuthority).toBe(false);
    expect(projectOutputDetail({ status: "ok" })).toEqual([]);
  });

  it("never reads a publication packet as published or chat-confirmable by default", () => {
    const rows = projectPublicationPacket({
      status: "ok",
      planned: true,
      published: false,
      chat_can_confirm: false,
      connector: "none-qualified",
      artifact: { artifact_id: "artifact-1" },
      autonomy_packet: {
        preview: { what_will_happen: "send", diff: "first" },
        override: { owner_controls: ["confirm", "narrow", "reject"] },
        tiered_authority: {},
        observable: {},
        outcome_verify: { verified: true, accepted: false },
        memory_of_actions: {},
        yield: {},
      },
    });
    expect(rows).toHaveLength(1);
    expect(rows[0].planned).toBe(true);
    expect(rows[0].published).toBe(false);
    expect(rows[0].chatCanConfirm).toBe(false);
    expect(rows[0].sections.map((s) => s.id)).toEqual([
      "preview",
      "override",
      "tiered_authority",
      "observable",
      "outcome_verify",
      "memory_of_actions",
      "yield",
    ]);
    expect(rows[0].sections[1].facts[0]).toEqual({ key: "owner_controls", value: "confirm, narrow, reject" });
    // A packet that omits `published` is not read as safely unpublished.
    const vague = projectPublicationPacket({
      status: "ok",
      artifact: { artifact_id: "artifact-1" },
      autonomy_packet: {},
    });
    expect(vague[0].published).toBe(true);
    expect(vague[0].chatCanConfirm).toBe(true);
    expect(projectPublicationPacket({ status: "ok" })).toEqual([]);
  });

  it("offers close-out only for a verified current last-ring artifact that is not accepted", () => {
    const stages = [
      { stageId: "s1", position: "0" },
      { stageId: "s2", position: "1" },
    ];
    const lastRing = lastRingStageId(stages);
    expect(lastRing).toBe("s2");
    const verified = projectOutputs({ status: "ok", artifacts: [ARTIFACT] })[0];
    expect(acceptanceOfferable(verified, lastRing)).toBe(true);
    expect(acceptanceOfferable({ ...verified, stageId: "s1" }, lastRing)).toBe(false);
    expect(acceptanceOfferable({ ...verified, verificationStatus: "not-run" }, lastRing)).toBe(false);
    expect(acceptanceOfferable({ ...verified, freshness: "superseded" }, lastRing)).toBe(false);
    expect(acceptanceOfferable({ ...verified, acceptedAt: "100" }, lastRing)).toBe(false);
    expect(acceptanceOfferable(verified, undefined)).toBe(false);
    expect(lastRingStageId([])).toBeUndefined();
  });
});
