import { describe, expect, it } from "vitest";
import {
  ACTIVITY_COVERAGE,
  ACTIVITY_KIND_LABEL,
  ACTIVITY_ROW_CAP,
  boundActivityRows,
  composeActivity,
  filterActivityRows,
  namedSourceFailure,
  probeObservedRefs,
  sortActivity,
} from "./activity";
import type { ObservedTask, SessionMutationReceipt, TaskEvidenceView } from "./home";

const TASK = "task://personal/web-ui/0193a3f9-1111-7000-8000-0000000000a1";
const NOW = 1_700_000_000_000;

const passedEvidence: TaskEvidenceView = {
  taskRef: TASK,
  lifecycleState: "COMPLETED",
  hasVerification: true,
  verificationStatus: "passed",
  verificationCurrent: true,
  reportRef: "report://personal/r-881",
  completedAt: new Date(NOW - 60_000).toISOString(),
  acceptancePresent: true,
  acceptanceCurrent: true,
};

describe("Activity composition (W8)", () => {
  it("maps the seven kinds onto real sources and never invents a unified feed", () => {
    const rows = composeActivity({
      alerts: [
        {
          id: "al-1",
          budgetId: "bud-1",
          threshold: "exceeded_80",
          issuedAtMs: NOW - 10_000,
          acknowledged: false,
        },
      ],
      auditEvents: [
        { id: "aud-2", action: "key.rotate", outcome: "ok", detail: "acct-1" },
        { id: "aud-1", action: "account.create", outcome: "ok", detail: "acct-1" },
        { id: "aud-3", action: "binding.set", outcome: "error", detail: "acct-1" },
      ],
      receipts: [
        {
          id: "ack-1",
          action: "alert.acknowledge",
          objectRef: "al-0",
          atMs: NOW - 5_000,
          detail: "from Activity",
        },
      ],
      observed: [
        {
          taskRef: TASK,
          objective: "search the workspace",
          observedAtMs: NOW - 20_000,
          origin: "task/admit",
        },
      ],
      effects: [
        {
          taskRef: TASK,
          effects: [
            {
              effectRef: "e-1",
              stage: "EXECUTED",
              outcomeClass: "ok",
              reconcileClass: "closed",
            },
            {
              effectRef: "e-2",
              stage: "VERIFY_FAILED",
              outcomeClass: "failed",
              reconcileClass: "must_reconcile",
            },
          ],
        },
      ],
      evidence: [{ taskRef: TASK, view: passedEvidence }],
    });
    const kinds = new Set(rows.map((row) => row.kind));
    expect([...kinds].sort()).toEqual([
      "acceptance",
      "change",
      "effect",
      "error",
      "event",
      "intervention",
      "verification",
    ]);
    expect(ACTIVITY_COVERAGE).toContain("Not a complete authority event log (BD-5)");
    expect(rows.find((row) => row.kind === "change")?.source).toBe("GET /management/audit");
    expect(rows.find((row) => row.id === "audit:aud-1")?.kind).toBe("event");
    expect(rows.find((row) => row.id === "audit:aud-3")?.kind).toBe("error");
    expect(rows.find((row) => row.id === `effect:${TASK}:e-2`)?.kind).toBe("error");
  });

  it("orders known timestamps newest-first and keeps untimestamped audit after them", () => {
    const rows = sortActivity(
      composeActivity({
        alerts: [
          {
            id: "al-new",
            threshold: "exceeded_80",
            issuedAtMs: NOW,
            acknowledged: false,
          },
        ],
        auditEvents: [{ id: "aud-z", action: "account.create", outcome: "ok" }],
        receipts: [],
        observed: [],
        effects: [],
        evidence: [],
      }),
    );
    expect(rows[0].id).toBe("alert:al-new");
    expect(rows[1].id).toBe("audit:aud-z");
    expect(rows[1].ageUnknownReason).toContain("no timestamp");
  });

  it("does not render acknowledged alerts or treat a missing verification as acceptance", () => {
    const rows = composeActivity({
      alerts: [
        { id: "al-old", threshold: "exceeded_80", acknowledged: true },
        { id: "al-live", threshold: "exceeded_100", acknowledged: false },
      ],
      auditEvents: [],
      receipts: [],
      observed: [],
      effects: [],
      evidence: [
        {
          taskRef: TASK,
          view: {
            hasVerification: false,
            acceptancePresent: false,
          },
        },
      ],
    });
    expect(rows.map((row) => row.id)).toEqual(["alert:al-live"]);
    expect(rows[0].kind).toBe("error");
    expect(rows[0].alertId).toBe("al-live");
  });

  it("labels a non-current verification as Verification, never as Acceptance success", () => {
    const rows = composeActivity({
      alerts: [],
      auditEvents: [],
      receipts: [],
      observed: [],
      effects: [],
      evidence: [
        {
          taskRef: TASK,
          view: {
            ...passedEvidence,
            verificationCurrent: false,
            acceptancePresent: true,
            acceptanceCurrent: true,
          },
        },
      ],
    });
    expect(rows.find((row) => row.kind === "verification")?.fact).toMatch(/not current/);
    expect(rows.find((row) => row.kind === "acceptance")).toBeUndefined();
  });

  it("keeps timestamp-less rows when filtering by since", () => {
    const rows = composeActivity({
      alerts: [
        {
          id: "al-old",
          threshold: "exceeded_80",
          issuedAtMs: NOW - 3 * 60 * 60 * 1000,
          acknowledged: false,
        },
      ],
      auditEvents: [{ id: "aud-1", action: "account.create", outcome: "ok", detail: "acct-1" }],
      receipts: [],
      observed: [],
      effects: [],
      evidence: [],
    });
    const filtered = filterActivityRows(rows, {
      kind: "all",
      objectType: "all",
      since: "hour",
      nowMs: NOW,
    });
    expect(filtered.some((row) => row.id === "al-old")).toBe(false);
    expect(filtered.some((row) => row.id === "audit:aud-1")).toBe(true);
  });

  it("names the bounded window instead of scrolling forever", () => {
    const many = composeActivity({
      alerts: [],
      auditEvents: Array.from({ length: ACTIVITY_ROW_CAP + 3 }, (_, index) => ({
        id: `aud-${String(index).padStart(3, "0")}`,
        action: "account.create",
        outcome: "ok",
      })),
      receipts: [],
      observed: [],
      effects: [],
      evidence: [],
    });
    const bounded = boundActivityRows(many);
    expect(bounded.shown).toHaveLength(ACTIVITY_ROW_CAP);
    expect(bounded.total).toBe(ACTIVITY_ROW_CAP + 3);
    expect(bounded.truncated).toBe(true);
  });

  it("probes only a bounded set of session-observed refs", () => {
    const observed: ObservedTask[] = Array.from({ length: 8 }, (_, index) => ({
      taskRef: `${TASK}-${index}`,
      observedAtMs: NOW - index,
      origin: "task/admit",
    }));
    const probed = probeObservedRefs(observed);
    expect(probed.refs).toHaveLength(6);
    expect(probed.truncated).toBe(true);
    expect(probed.total).toBe(8);
  });

  it("names a failed source instead of coercing it to empty", () => {
    expect(
      namedSourceFailure(
        { status: "unknown", error: { code: "AUDIT_UNAVAILABLE", message: "down", httpStatus: 503 } },
        "provider-plane audit",
      ),
    ).toBe("provider-plane audit unavailable — AUDIT_UNAVAILABLE");
    expect(namedSourceFailure({ status: "empty" }, "provider-plane audit")).toBeUndefined();
  });

  it("labels every kind in text, never as a color-only token", () => {
    expect(ACTIVITY_KIND_LABEL.intervention).toBe("Intervention");
    expect(ACTIVITY_KIND_LABEL.acceptance).toBe("Acceptance");
  });
});

describe("Activity receipts", () => {
  it("records an acknowledge as Intervention, not as a toast-shaped absence", () => {
    const receipts: SessionMutationReceipt[] = [
      {
        id: "alert.acknowledge:al-1",
        action: "alert.acknowledge",
        objectRef: "al-1",
        atMs: NOW,
        detail: "budget alert acknowledged from Activity",
      },
    ];
    const rows = composeActivity({
      alerts: [],
      auditEvents: [],
      receipts,
      observed: [],
      effects: [],
      evidence: [],
    });
    expect(rows).toHaveLength(1);
    expect(rows[0].kind).toBe("intervention");
    expect(rows[0].source).toContain("session");
  });
});
