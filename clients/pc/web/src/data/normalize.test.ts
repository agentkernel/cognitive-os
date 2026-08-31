import { describe, expect, it } from "vitest";
import { detectStubNote, isKnownRoute, normalizeBody } from "./normalize";

describe("normalizeBody — the three daemon error envelopes", () => {
  it("normalizes the front-door envelope {status,error{code,message,category,retryable}}", () => {
    const result = normalizeBody(403, {
      status: "error",
      error: {
        code: "LOCAL_ORIGIN_HEADER_REJECTED",
        message: "foreign origin",
        category: "protocol",
        retryable: false,
        stage: "personal-front-door",
      },
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.code).toBe("LOCAL_ORIGIN_HEADER_REJECTED");
      expect(result.error.category).toBe("protocol");
      expect(result.error.retryable).toBe(false);
    }
  });

  it("normalizes the flat task/resource envelope {status:error,code,message}", () => {
    const result = normalizeBody(409, {
      status: "error",
      code: "PROVIDER_BINDING_REVISION_STALE",
      message: "expected revision is stale",
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.code).toBe("PROVIDER_BINDING_REVISION_STALE");
    }
  });

  it("normalizes the backup envelope {error:{code,detail}}", () => {
    const result = normalizeBody(409, {
      error: { code: "BACKUP_TAMPERED", detail: "manifest digest mismatch" },
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.code).toBe("BACKUP_TAMPERED");
      expect(result.error.message).toBe("manifest digest mismatch");
    }
  });

  it("passes success bodies through untouched", () => {
    const result = normalizeBody(200, { status: "ok", accounts: [] });
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.data).toEqual({ status: "ok", accounts: [] });
    }
  });

  it("maps bare HTTP failures without an envelope", () => {
    const result = normalizeBody(503, "not available");
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.code).toBe("HTTP_503");
    }
  });
});

describe("200-stub detection (R-1)", () => {
  it("detects the management front-door stub", () => {
    expect(
      detectStubNote({
        status: "ok",
        channel: "management",
        note: "authenticated personal front door; business routes deferred",
      }),
    ).toBe(true);
  });

  it("detects the task-channel stub", () => {
    expect(
      detectStubNote({
        status: "ok",
        channel: "task",
        note: "authenticated task front door; no Task API operation matched",
      }),
    ).toBe(true);
  });

  it("never flags real success bodies", () => {
    expect(detectStubNote({ status: "ok", accounts: [] })).toBe(false);
    expect(detectStubNote({ note: "a real note about work" })).toBe(false);
    expect(detectStubNote([])).toBe(false);
    expect(detectStubNote("text")).toBe(false);
  });

  it("normalizeBody turns a 200 stub into a stub failure, never success", () => {
    const result = normalizeBody(200, {
      status: "ok",
      note: "authenticated task front door; no Task API operation matched",
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.stub).toBe(true);
      expect(result.error.code).toBe("STUB_ROUTE");
    }
  });
});

describe("route whitelist (R-1)", () => {
  it("accepts known routes with query strings", () => {
    expect(isKnownRoute("GET", "/management/providers/accounts")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/list")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/detail?project_id=proj-1")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/axis?project_id=proj-1")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/roster?project_id=proj-1")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/pending-previews?subject_ref=proj-1")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/vault.index?project_id=proj-1")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/standing-policies")).toBe(true);
    expect(isKnownRoute("GET", "/task/evidence?task_ref=x")).toBe(true);
    expect(isKnownRoute("POST", "/management/agent-bindings/remove")).toBe(true);
  });

  it("rejects unknown and forbidden routes", () => {
    expect(isKnownRoute("POST", "/task/cancel")).toBe(false);
    expect(isKnownRoute("POST", "/task/complete")).toBe(false);
    expect(isKnownRoute("POST", "/management/project/v1/confirm")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/preview-detail?preview_id=prev-1")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/preview.reject")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/preview.narrow")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/draft.create")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/preview.request")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/employee.catalog?project_id=proj-1&employee_id=emp-1")).toBe(
      true,
    );
    expect(isKnownRoute("POST", "/management/project/v1/roster.register")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/employee.seat.request")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/employee.seat.confirm")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/vault.apply-authority")).toBe(false);
    expect(isKnownRoute("POST", "/management/agent/transition")).toBe(false);
    expect(isKnownRoute("GET", "/management/providers/accounts/../../etc")).toBe(false);
    expect(isKnownRoute("DELETE", "/management/providers/accounts")).toBe(false);
  });
});
