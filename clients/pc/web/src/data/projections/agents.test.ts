import { describe, expect, it } from "vitest";
import { AGENT_IDENTITY_KEYS, emptyIdentities } from "../../identities";
import {
  agentIsAddressable,
  agentLifecycleReading,
  bindingSummary,
  composeAgentRows,
  extractIdentitiesFromInspect,
  identityCards,
  inspectUnavailableCards,
  isRuntimeInspectUnavailable,
  normalizeAgentId,
  projectRuntimeList,
  projectToolExposure,
} from "./agents";
import { projectDshRuntime, type BindingView, type ProviderAccount } from "./providers";

const PI_BINDING: BindingView = {
  agent: "pi",
  accountId: "acct-1",
  modelId: "deepseek-chat",
  revision: 4,
  status: "active",
};

const DSH_BINDING: BindingView = {
  agent: "agent://personal/dsh",
  accountId: "acct-1",
  modelId: "deepseek-chat",
  revision: 2,
  status: "active",
};

const ACTIVE_ACCOUNT: ProviderAccount = {
  id: "acct-1",
  name: "main",
  kind: "deepseek",
  status: "active",
  secret: "present",
};

describe("agent identity composition", () => {
  it("normalizes binding agent URIs onto pi and dsh", () => {
    expect(normalizeAgentId("agent://personal/dsh")).toBe("dsh");
    expect(normalizeAgentId("pi")).toBe("pi");
  });

  it("always lists pi and dsh even when bindings and runtime are empty", () => {
    const rows = composeAgentRows({ bindings: [] });
    expect(rows.map((row) => row.id)).toEqual(["pi", "dsh"]);
    expect(rows.every((row) => row.dispatch === "unbound")).toBe(true);
    expect(rows[0].currentWorkKind).toBe("unavailable");
    expect(rows[0].currentWorkLabel).toMatch(/BD-2/);
    expect(rows[1].currentWorkKind).toBe("unavailable");
  });

  it("marks dispatch callable only when the account and binding are both usable", () => {
    const rows = composeAgentRows({
      bindings: [PI_BINDING, DSH_BINDING],
      accounts: [ACTIVE_ACCOUNT],
    });
    expect(rows[0].dispatch).toBe("callable");
    expect(rows[1].dispatch).toBe("callable");
    expect(bindingSummary(rows[0])).toContain("acct-1 / deepseek-chat");
  });

  it("blocks dispatch when the bound account is revoked, without inventing a lifecycle", () => {
    const rows = composeAgentRows({
      bindings: [PI_BINDING],
      accounts: [{ ...ACTIVE_ACCOUNT, status: "revoked" }],
    });
    expect(rows[0].dispatch).toBe("blocked");
    expect(rows[0].lifecycleLabel).toMatch(/not exposed over HTTP/);
  });

  it("reads dsh current work from the snapshot task_ref and never from process liveness", () => {
    const runtime = projectDshRuntime({
      state: "ACTIVE",
      process_alive: true,
      process_id: 4812,
      session_count: 1,
      candidate_only: true,
      dsh_response_is_not_task_completion: true,
      sessions: [
        {
          session_id: "sess-1",
          state: "Active",
          fencing_epoch: 3,
          task_ref: "task://personal/a3f9",
        },
      ],
    });
    const rows = composeAgentRows({
      bindings: [DSH_BINDING],
      accounts: [ACTIVE_ACCOUNT],
      runtime,
    });
    const dsh = rows.find((row) => row.id === "dsh");
    expect(dsh?.currentWorkKind).toBe("task");
    expect(dsh?.currentTaskRef).toBe("task://personal/a3f9");
    expect(dsh?.dshState).toBe("ACTIVE");
    expect(agentLifecycleReading(dsh!).label).toBe("ACTIVE");
    expect(dsh?.lifecycleSource).toMatch(/candidate_only/);
  });

  it("says none observed when dsh is alive but has no task_ref", () => {
    const runtime = projectDshRuntime({
      state: "ACTIVE",
      process_alive: true,
      sessions: [{ session_id: "sess-1", state: "Active" }],
    });
    const dsh = composeAgentRows({ runtime }).find((row) => row.id === "dsh");
    expect(dsh?.currentWorkKind).toBe("none");
    expect(dsh?.currentWorkLabel).toBe("none observed");
  });

  it("does not address unknown actors unless a binding named them", () => {
    expect(agentIsAddressable("nope", [])).toBe(false);
    expect(agentIsAddressable("pi", [])).toBe(true);
    expect(agentIsAddressable("custom", ["custom"])).toBe(true);
  });
});

describe("identity cards and inspect honesty", () => {
  it("keeps the nine-key merge and source-labels unknown when inspect is unavailable", () => {
    const cards = inspectUnavailableCards();
    expect(cards.map((card) => card.key)).toEqual([...AGENT_IDENTITY_KEYS]);
    expect(cards.every((card) => card.value === "unknown")).toBe(true);
    expect(cards.find((card) => card.key === "process")?.caption).toMatch(/not task completion/);
    expect(isRuntimeInspectUnavailable("RESOURCE_MANAGER_NOT_FOUND")).toBe(true);
    expect(isRuntimeInspectUnavailable("NOT_FOUND")).toBe(false);
  });

  it("projects inspect envelopes through identities.ts without inventing missing keys", () => {
    const identities = extractIdentitiesFromInspect({
      resource: {
        instance: "pi-01",
        process: "pid-4812",
        identities: { package: "pkg-aaa" },
      },
    });
    expect(identities.instance).toBe("pi-01");
    expect(identities.package).toBe("pkg-aaa");
    expect(identities.registration).toBe("unknown");
    expect(identities.process).not.toBe(identities.task);
    const cards = identityCards(identities, "inspect");
    expect(cards).toHaveLength(9);
    expect(emptyIdentities().task).toBe("unknown");
  });

  it("records an empty projection-only runtime list without fabricating actors", () => {
    const list = projectRuntimeList({
      family: "runtime",
      authority_source: "projection-only",
      resources: [],
    });
    expect(list.resources).toEqual([]);
    expect(list.authoritySource).toBe("projection-only");
  });

  it("projects task-scoped tool exposure without defaulting an empty set to permitted", () => {
    const exposure = projectToolExposure({
      task_ref: "task://personal/a",
      tools: [{ op_id: "workspace.read", lifecycle: "enabled" }],
    });
    expect(exposure.tools[0].id).toBe("workspace.read");
    expect(projectToolExposure({}).tools).toEqual([]);
  });
});
