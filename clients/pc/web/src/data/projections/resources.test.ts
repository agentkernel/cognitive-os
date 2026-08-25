import { describe, expect, it } from "vitest";
import type { Projection } from "../store";
import {
  composeFamilyRows,
  envelopeAtBound,
  HUB_FAMILIES,
  isProjectionOnly,
  memoryFact,
  projectResourceList,
  readFamilyRow,
  RESOURCE_LIST_LIMIT,
  skillFact,
  toolFact,
  type ResourceListView,
} from "./resources";

function ready(view: ResourceListView): Projection<ResourceListView> {
  return { status: "ready", data: view, source: "/management/resource/v1/list" };
}

describe("resource list projection", () => {
  it("reads family, authority source, truncated, and envelope health without inventing ids", () => {
    const view = projectResourceList({
      kind: "resource.manager.list",
      family: "memory",
      authority_source: "sqlite-authority-memory-objects",
      truncated: false,
      resources: [{ id: "mem-1", family: "memory", health: "admitted" }],
    });
    expect(view.family).toBe("memory");
    expect(view.authoritySource).toBe("sqlite-authority-memory-objects");
    expect(view.truncated).toBe(false);
    expect(view.resources).toEqual([{ id: "mem-1", family: "memory", health: "admitted" }]);
  });

  it("treats missing resources as empty rather than fabricating a row", () => {
    const view = projectResourceList({ family: "skill", authority_source: "sqlite-authority-skill-bindings" });
    expect(view.resources).toEqual([]);
    expect(view.truncated).toBe(false);
  });
});

describe("hub fact lines", () => {
  it("never invents a tombstone count from the non-tombstoned memory list", () => {
    const view = projectResourceList({
      family: "memory",
      truncated: false,
      resources: [
        { id: "a", health: "admitted" },
        { id: "b", health: "admitted" },
      ],
    });
    expect(memoryFact(view)).toBe("2 admitted · tombstones are not in this list · envelope limit 64");
    expect(memoryFact(view)).not.toMatch(/tombstoned \d/);
  });

  it("labels skill rows as bindings, not packages", () => {
    const view = projectResourceList({
      family: "skill",
      truncated: false,
      resources: [
        { id: "s1", health: "bound" },
        { id: "s2", health: "bound" },
        { id: "s3", health: "revoked" },
      ],
    });
    expect(skillFact(view)).toContain("2 bound");
    expect(skillFact(view)).toContain("1 revoked");
    expect(skillFact(view)).toContain("list is skill bindings, not packages");
  });

  it("counts tool lifecycle health from the envelope and labels the bound", () => {
    const view: ResourceListView = {
      family: "tool",
      truncated: true,
      resources: Array.from({ length: RESOURCE_LIST_LIMIT }, (_, index) => ({
        id: `native.op.${index}`,
        family: "tool",
        health: index === 0 ? "quarantined" : "enabled",
      })),
    };
    expect(envelopeAtBound(view)).toBe(true);
    expect(toolFact(view)).toContain("63 enabled");
    expect(toolFact(view)).toContain("1 quarantined");
    expect(toolFact(view)).toContain("envelope at bound (limit 64)");
  });

  it("does not treat projection-only context as an empty family", () => {
    const view = projectResourceList({
      family: "context",
      authority_source: "projection-only",
      truncated: false,
      resources: [],
    });
    expect(isProjectionOnly(view)).toBe(true);
    const row = readFamilyRow(HUB_FAMILIES[3], ready(view));
    expect(row.kind).toBe("projection-only");
    expect(row.fact).toMatch(/no standalone HTTP browser/);
    expect(row.action).toEqual({ kind: "work", href: "/work", label: "Work" });
  });
});

describe("composeFamilyRows", () => {
  it("keeps Memory, Skills, Tools, Context order and degrades one denied family without inventing counts", () => {
    const rows = composeFamilyRows({
      memory: {
        status: "denied",
        error: { code: "UNAUTHORIZED", message: "no", httpStatus: 401 },
      },
      skill: ready({
        family: "skill",
        truncated: false,
        resources: [],
      }),
      tool: ready({
        family: "tool",
        truncated: false,
        resources: [{ id: "native.workspace.read", family: "tool", health: "enabled" }],
      }),
      context: ready({
        family: "context",
        authoritySource: "projection-only",
        truncated: false,
        resources: [],
      }),
    });
    expect(rows.map((row) => row.id)).toEqual(["memory", "skill", "tool", "context"]);
    expect(rows[0].kind).toBe("denied");
    expect(rows[0].fact).not.toMatch(/admitted/);
    expect(rows[1].kind).toBe("empty");
    expect(rows[1].fact).toMatch(/remember is on the family page|import is on the family page/);
    expect(rows[2].kind).toBe("ready");
    expect(rows[3].kind).toBe("projection-only");
    expect(rows[3].action.kind).toBe("work");
  });

  it("names STUB_ROUTE as not an observed zero", () => {
    const row = readFamilyRow(HUB_FAMILIES[0], {
      status: "not-run",
      error: { code: "STUB_ROUTE", message: "stub", httpStatus: 200 },
    });
    expect(row.kind).toBe("stub");
    expect(row.fact).toContain("STUB_ROUTE");
    expect(row.fact).toContain("not an observed zero");
  });
});
