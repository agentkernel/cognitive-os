import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body?: unknown; raw?: string; contentType?: string };
type FetchCall = { method: string; path: string; pathname: string; body?: string };

function installFetch(routes: Record<string, RouteResponse>): FetchCall[] {
  const calls: FetchCall[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
      calls.push({
        method,
        path: `${url.pathname}${url.search}`,
        pathname: url.pathname,
        body: typeof init?.body === "string" ? init.body : undefined,
      });
      const handler = routes[`${method} ${url.pathname}`];
      const resolved =
        handler ??
        (url.pathname === "/personal/health" || url.pathname === "/personal/status"
          ? { status: 200, body: { status: "ok", overall: "ready", alerts: [] } }
          : { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } });
      if (typeof resolved.raw === "string") {
        return new Response(resolved.raw, {
          status: resolved.status,
          headers: { "content-type": resolved.contentType ?? "text/markdown; charset=utf-8" },
        });
      }
      return new Response(JSON.stringify(resolved.body), {
        status: resolved.status,
        headers: { "content-type": "application/json" },
      });
    }),
  );
  return calls;
}

function renderAppAt(hash: string) {
  window.location.hash = hash;
  const host = document.createElement("div");
  document.body.appendChild(host);
  const root = createRoot(host);
  act(() => {
    root.render(<App />);
  });
  return { host, root };
}

async function flush(ticks = 20) {
  for (let i = 0; i < ticks; i += 1) {
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
    });
  }
}

function unmount(host: HTMLDivElement, root: ReturnType<typeof createRoot>) {
  act(() => {
    root.unmount();
  });
  host.remove();
}

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority|install|publish/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const labels: string[] = [];
  for (const node of host.querySelectorAll("button, a.cp-button")) {
    if (node.closest("[data-region='opc-rail-write']")) {
      continue;
    }
    const label = (node.textContent ?? "").trim();
    if (FAKE_ACTION.test(label)) {
      labels.push(label);
    }
  }
  return labels;
}

const DIGEST_A = `sha256:${"a".repeat(64)}`;
const DIGEST_B = `sha256:${"b".repeat(64)}`;

function artifact(overrides: Record<string, unknown>) {
  return {
    artifact_id: "artifact-1",
    attempt_id: "dshattempt-1",
    project_id: "proj-1",
    task_ref: "task://personal/p13-t04",
    employee_id: "emp-2",
    cas_ref: DIGEST_A,
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
    ...overrides,
  };
}

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const READY_AXIS: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    plan_revision_id: "plan-1",
    stages: [
      { stage_id: "s1", position: 0, title: "Plan", confirm_status: "confirmed", ready: true, seated: true, output_contract: {}, gaps: [], responsible_slot: "manager" },
      { stage_id: "s2", position: 1, title: "Report", confirm_status: "confirmed", ready: true, seated: true, output_contract: {}, gaps: [], responsible_slot: "researcher" },
    ],
  },
};

const READY_ROSTER: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    authority_note: "employee",
    roster: [
      { employee_id: "emp-1", state: "seated", model_bound: true, is_current_manager: true, runtime_binding_ref: "r1", responsible_stage_ids: ["s1"] },
      { employee_id: "emp-2", state: "seated", model_bound: true, is_current_manager: false, runtime_binding_ref: "r2", responsible_stage_ids: ["s2"] },
    ],
  },
};

function outputsRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/axis": READY_AXIS,
    "GET /management/project/v1/roster": READY_ROSTER,
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    "GET /management/project/v1/outputs": {
      status: 200,
      body: {
        status: "ok",
        artifacts: [
          artifact({}),
          artifact({ artifact_id: "artifact-0", cas_ref: DIGEST_B, freshness: "superseded", stage_id: null, latest_evidence_id: null, verification_status: "not-run" }),
        ],
        run_acceptances: [],
        files_are_authority: false,
        chat_can_confirm: false,
        host_file_open_e2e: "not-run",
      },
    },
    "GET /management/project/v1/outputs.detail": {
      status: 200,
      body: {
        status: "ok",
        artifact: artifact({}),
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
            report_cas_ref: DIGEST_B,
            checked_cas_ref: DIGEST_A,
            verified_at: 90,
          },
        ],
        run_acceptance: null,
        open_route: "/management/project/v1/outputs.open?artifact_id=artifact-1",
        export: { exists: false, path: "/home/o/.local/share/cognitiveos/projects/proj-1/outputs/artifact-1.md", is_authority: false },
        files_are_authority: false,
        chat_can_confirm: false,
      },
    },
    "GET /management/project/v1/outputs.open": { status: 200, raw: "# Weekly report\n\nTASK COMPLETE: four follow-ups." },
    "GET /management/project/v1/publication.packet": {
      status: 200,
      body: {
        status: "ok",
        planned: true,
        published: false,
        chat_can_confirm: false,
        connector: "none-qualified",
        artifact: { artifact_id: "artifact-1", cas_ref: DIGEST_A },
        autonomy_packet: {
          preview: { what_will_happen: "send the verified deliverable", diff: "first send" },
          override: { owner_controls: ["confirm", "narrow", "reject"] },
          tiered_authority: { external_send_requires: "owner confirm of a digest-bound preview" },
          observable: { receipt: "p13_external_send planned" },
          outcome_verify: { verified: true, accepted: false, evidence_id: "evidence-1" },
          memory_of_actions: { attempt_id: "dshattempt-1" },
          yield: { stoppable_until: "dispatch (none exists yet)" },
        },
      },
    },
    "POST /management/project/v1/outputs.export": {
      status: 200,
      body: { status: "ok", artifact_id: "artifact-1", export: { path: "/home/o/.local/share/cognitiveos/projects/proj-1/outputs/artifact-1.md", is_authority: false } },
    },
    "POST /management/project/v1/run.acceptance.request": {
      status: 200,
      body: { status: "ok", subject_kind: "run-acceptance", preview_id: "preview-accept-1", preview_digest: "d".repeat(64), chat_can_confirm: false },
    },
    "POST /management/project/v1/publication.external-send.request": {
      status: 200,
      body: { status: "ok", subject_kind: "external-send", preview_id: "preview-send-1", preview_digest: "e".repeat(64), planned: true, published: false },
    },
    ...extras,
  };
}

async function renderOutputs(extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(outputsRoutes(extras));
  const view = renderAppAt("#/projects/proj-1/outputs");
  await flush();
  return { ...view, calls };
}

function buttonLabelled(host: HTMLElement, label: string | RegExp): HTMLButtonElement | undefined {
  return [...host.querySelectorAll("button")].find((candidate) => {
    const text = (candidate.textContent ?? "").trim();
    return typeof label === "string" ? text === label : label.test(text);
  });
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P13-T04 outputs select-then-view", () => {
  it("whitelists the artifact routes and calls only GET outputs on load", async () => {
    for (const route of [
      "/management/project/v1/outputs",
      "/management/project/v1/outputs.detail",
      "/management/project/v1/outputs.open",
      "/management/project/v1/publication.packet",
    ]) {
      expect(isKnownRoute("GET", route)).toBe(true);
    }
    for (const route of [
      "/management/project/v1/outputs.export",
      "/management/project/v1/attempt.artifact.stage-test",
      "/management/project/v1/run.acceptance.request",
      "/management/project/v1/publication.external-send.request",
    ]) {
      expect(isKnownRoute("POST", route)).toBe(true);
    }
    const { host, root, calls } = await renderOutputs();
    expect(host.querySelector("[data-page='opc-project-outputs']")).not.toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/outputs")).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    // Select-then-view: nothing is opened until a row is chosen.
    expect(host.textContent).toMatch(/no output selected/i);
    expect(host.querySelector("[data-region='opc-output-selected']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/outputs.detail")).toBe(false);
    expect(host.querySelector("[data-row-key='artifact-1']")?.textContent).toContain("passed");
    expect(host.querySelector("[data-row-key='artifact-0']")?.textContent).toContain("superseded");
    expect(host.querySelector("[data-row-key='artifact-0']")?.textContent).toContain("not-run");
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("opens real CAS bytes, exports a non-authority copy, and never shows Publish", async () => {
    const { host, root, calls } = await renderOutputs();
    await act(async () => {
      buttonLabelled(host, "artifact-1")?.click();
    });
    await flush();
    const selected = host.querySelector("[data-region='opc-output-selected']");
    expect(selected).not.toBeNull();
    expect(selected?.textContent).toContain(DIGEST_A);
    expect(selected?.textContent).toContain("verifier://personal/attempt-artifact");
    expect(selected?.textContent).toContain("attempt-response-status=not-used");
    expect(calls.some((call) => call.pathname === "/management/project/v1/outputs.detail")).toBe(true);
    expect(host.textContent).toMatch(/Files are not Project authority/i);
    // Open reads the bytes from the daemon CAS route, not from a file path.
    await act(async () => {
      buttonLabelled(host, "Open from CAS")?.click();
    });
    await flush();
    expect(host.querySelector("[data-region='opc-output-bytes']")?.textContent).toContain("TASK COMPLETE: four follow-ups.");
    expect(calls.some((call) => call.pathname === "/management/project/v1/outputs.open")).toBe(true);
    const download = host.querySelector("[data-region='opc-output-download']");
    expect(download?.getAttribute("download")).toBe("artifact-1.md");
    expect(download?.getAttribute("href")?.startsWith("data:text/markdown")).toBe(true);
    // Export copy goes to Personal Home data/ and is labelled non-authority.
    await act(async () => {
      buttonLabelled(host, "Export copy to data/")?.click();
    });
    await flush();
    expect(host.querySelector("[data-region='opc-output-exported']")?.textContent).toContain("projects/proj-1/outputs/artifact-1.md");
    expect(host.querySelector("[data-region='opc-output-exported']")?.textContent).toMatch(/Not authority/);
    const exportCall = calls.find((call) => call.method === "POST" && call.pathname === "/management/project/v1/outputs.export");
    expect(exportCall?.body).toContain("artifact-1");
    // Publication package: planned, not published; no Publish control anywhere.
    const publication = host.querySelector("[data-region='opc-output-publication']");
    expect(publication?.textContent).toMatch(/Planned/);
    expect(publication?.textContent).toMatch(/not published/);
    expect(publication?.textContent).toContain("none-qualified");
    expect(publication?.textContent).toMatch(/chat has no Confirm/);
    expect(publication?.querySelector("[data-output-packet-published='false']")).not.toBeNull();
    for (const section of ["preview", "override", "tiered_authority", "observable", "outcome_verify", "memory_of_actions", "yield"]) {
      expect(publication?.querySelector(`[data-packet-section='${section}']`)).not.toBeNull();
    }
    expect(fakeActionLabels(host)).toEqual([]);
    expect([...host.querySelectorAll("button")].some((b) => /publish/i.test(b.textContent ?? ""))).toBe(false);
    unmount(host, root);
  });

  it("offers 验收 only on the last ring and routes it through the canvas preview", async () => {
    const { host, root, calls } = await renderOutputs();
    // Superseded, unverified artifact: no stage test, no acceptance offered.
    await act(async () => {
      buttonLabelled(host, "artifact-0")?.click();
    });
    await flush();
    expect(host.querySelector("[data-output-closeout='not-verified']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-output-accept']")).toBeNull();
    // Verified last-ring artifact: request the acceptance preview.
    await act(async () => {
      buttonLabelled(host, "artifact-1")?.click();
    });
    await flush();
    const accept = host.querySelector("[data-region='opc-output-accept']");
    expect(accept).not.toBeNull();
    expect(accept?.textContent).toContain("验收，回 Today");
    await act(async () => {
      (accept as HTMLButtonElement).click();
    });
    await flush();
    const request = calls.find((call) => call.method === "POST" && call.pathname === "/management/project/v1/run.acceptance.request");
    expect(request?.body).toContain("\"stage_id\":\"s2\"");
    const link = host.querySelector("[data-region='opc-output-accept-preview'] a");
    expect(link?.getAttribute("href")).toBe("#/projects/proj-1?preview=preview-accept-1");
    expect(host.textContent).toMatch(/Chat cannot accept/);
    // No Confirm is offered on this page: confirm lives on the HITL canvas.
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not offer acceptance for an intermediate ring or a not-yet-tested artifact", async () => {
    const { host, root } = await renderOutputs({
      "GET /management/project/v1/outputs": {
        status: 200,
        body: {
          status: "ok",
          artifacts: [
            artifact({ artifact_id: "artifact-mid", employee_id: "emp-1", stage_id: "s1" }),
            artifact({ artifact_id: "artifact-untested", stage_id: null }),
          ],
        },
      },
    });
    await act(async () => {
      buttonLabelled(host, "artifact-mid")?.click();
    });
    await flush();
    expect(host.querySelector("[data-output-closeout='intermediate-ring']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-output-accept']")).toBeNull();
    await act(async () => {
      buttonLabelled(host, "artifact-untested")?.click();
    });
    await flush();
    expect(host.querySelector("[data-region='opc-output-accept']")).toBeNull();
    const select = host.querySelector("select[name='stage-id']") as HTMLSelectElement | null;
    expect(select).not.toBeNull();
    // The stage list comes from the Member's responsible stages; nothing is assumed.
    expect([...(select?.options ?? [])].map((o) => o.value)).toEqual(["", "s2"]);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("requests an external-send preview without sending or confirming", async () => {
    const { host, root, calls } = await renderOutputs();
    await act(async () => {
      buttonLabelled(host, "artifact-1")?.click();
    });
    await flush();
    const input = host.querySelector("input[name='recipients']") as HTMLInputElement | null;
    expect(input).not.toBeNull();
    await act(async () => {
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      setter?.call(input, "customer-a, customer-b");
      input?.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      buttonLabelled(host, "Request external-send preview")?.click();
    });
    await flush();
    const send = calls.find((call) => call.method === "POST" && call.pathname === "/management/project/v1/publication.external-send.request");
    expect(send?.body).toContain("customer-a");
    const link = host.querySelector("[data-region='opc-output-send-preview'] a");
    expect(link?.getAttribute("href")).toBe("#/projects/proj-1?preview=preview-send-1");
    expect(host.querySelector("[data-region='opc-output-send-preview']")?.textContent).toMatch(/Planned is not published/);
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("keeps a daemon 403 on outputs as denied, not an empty gallery", async () => {
    const { host, root } = await renderOutputs({
      "GET /management/project/v1/outputs": {
        status: 403,
        body: { status: "error", error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" } },
      },
    });
    expect(host.textContent).toMatch(/session denied/i);
    expect(host.textContent).not.toMatch(/no openable artifact yet/i);
    unmount(host, root);
  });
});
