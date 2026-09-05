import { describe, expect, it } from "vitest";
import {
  buildCommandCatalog,
  catalogHasClassC,
  contextActions,
  groupCommands,
  rankCommands,
  rememberCommand,
  resetCommandRecents,
} from "./commands";
import { resourceListKey } from "./projections/resources";
import { WORK_TASKS_KEY } from "./projections/work";
import { createProjectionStore } from "./store";

const TASK_REF = "task://personal/web-ui/abc";

describe("command catalog (W10)", () => {
  it("indexes destinations and class-A landings without class-C verbs", () => {
    const items = buildCommandCatalog(createProjectionStore());
    expect(items.some((item) => item.label === "Work")).toBe(false);
    expect(items.some((item) => item.label === "Today")).toBe(true);
    expect(items.some((item) => item.label === "Settings")).toBe(true);
    expect(items.some((item) => item.href === "/settings")).toBe(true);
    expect(items.some((item) => item.href === "/settings/model-connections")).toBe(true);
    expect(items.some((item) => item.href === "/work/new")).toBe(false);
    expect(items.some((item) => item.href === "/providers")).toBe(false);
    expect(catalogHasClassC(items)).toBe(false);
  });

  it("indexes only loaded objects, not a server search", () => {
    const empty = buildCommandCatalog(createProjectionStore());
    expect(empty.some((item) => item.id.startsWith("object:task:"))).toBe(false);

    const store = createProjectionStore();
    store.set(WORK_TASKS_KEY, {
      status: "ready",
      data: [{ taskRef: TASK_REF }],
    });
    store.set("providers:accounts", {
      status: "ready",
      data: [{ id: "acct-1", name: "deepseek-main", kind: "openai_compat", status: "active", secret: "present" }],
    });
    store.set(resourceListKey("memory"), {
      status: "ready",
      data: { family: "memory", truncated: false, resources: [{ id: "mem-1", family: "memory" }] },
    });
    store.set("home:alerts", {
      status: "ready",
      data: [{ id: "al-live", threshold: "exceeded_80", acknowledged: false }],
    });
    const loaded = buildCommandCatalog(store);
    expect(loaded.some((item) => item.href === "/work/" + encodeURIComponent(TASK_REF))).toBe(false);
    expect(loaded.some((item) => item.href === "/providers/acct-1")).toBe(false);
    expect(loaded.some((item) => item.id === "object:memory:mem-1")).toBe(true);
    expect(loaded.some((item) => item.execution === "acknowledge" && item.alertId === "al-live")).toBe(true);
  });

  it("ranks exact ids ahead of destinations and reports no-results as empty", () => {
    const store = createProjectionStore();
    store.set(resourceListKey("memory"), {
      status: "ready",
      data: { family: "memory", truncated: false, resources: [{ id: "mem-1", family: "memory" }] },
    });
    const items = buildCommandCatalog(store);
    const exact = rankCommands(items, "mem-1");
    expect(exact[0]?.kind).toBe("object");
    expect(rankCommands(items, "zzzz-no-such-object")).toEqual([]);
  });

  it("surfaces recents first when the query is empty", () => {
    resetCommandRecents();
    rememberCommand("dest:/activity");
    const ranked = rankCommands(buildCommandCatalog(createProjectionStore()), "");
    expect(ranked[0]?.id).toBe("dest:/activity");
    resetCommandRecents();
  });

  it("exposes contextual copy/verify/run landings for the current task", () => {
    const actions = contextActions(`/work/${encodeURIComponent(TASK_REF)}`);
    expect(actions.some((item) => item.execution === "copy-ref" && item.copyValue === TASK_REF)).toBe(
      true,
    );
    expect(actions.some((item) => item.href?.includes("section=evidence"))).toBe(true);
    expect(actions.some((item) => item.href?.includes("section=run"))).toBe(true);
    expect(catalogHasClassC(actions)).toBe(false);
  });

  it("groups empty-query results as Actions then Destinations", () => {
    const ranked = rankCommands(buildCommandCatalog(createProjectionStore()), "");
    const groups = groupCommands(ranked, "");
    expect(groups.map((group) => group.kind)).toEqual(["action", "destination", "help"]);
    expect(groups.map((group) => group.label)).toEqual(["ACTIONS", "DESTINATIONS", "HELP"]);
  });
});
