import { describe, expect, it } from "vitest";
import {
  SKILL_PERMISSION_ANNOTATION,
  projectSkillExplain,
  skillBindBody,
  skillImportBody,
  skillMasterFooter,
} from "./skills";

describe("skill explain projection", () => {
  it("reads binding explain fields and does not invent missing digests", () => {
    const view = projectSkillExplain({
      kind: "skill.binding.explain",
      binding: {
        binding_id: "bind-1",
        revision_id: "rev-1",
        workspace_scope: "workspace://personal",
        target_kind: "workspace",
        target_ref: "workspace://personal",
        status: "active",
        package_id: "pkg-1",
        manifest_digest: "sha256:manifest",
        content_digest: "sha256:content",
      },
    });
    expect(view.bindingId).toBe("bind-1");
    expect(view.revisionId).toBe("rev-1");
    expect(view.packageId).toBe("pkg-1");
    expect(view.manifestDigest).toBe("sha256:manifest");
    expect(view.compatibility).toBeUndefined();
    expect(view.revocationReason).toBeUndefined();
  });

  it("reads compatibility from canonical_json when the envelope omits the field", () => {
    const view = projectSkillExplain({
      binding: {
        binding_id: "bind-2",
        canonical_json: JSON.stringify({ compatibility: "compatible" }),
      },
    });
    expect(view.compatibility).toBe("compatible");
  });
});

describe("skill import/bind bodies and footer", () => {
  it("posts operator-supplied digests and minted ids without a previous_revision_id", () => {
    const body = skillImportBody({
      packageId: "pkg",
      revisionId: "rev",
      workspaceScope: "workspace://personal",
      localSourcePath: "skills/example",
      provenanceRef: "file://skills/example",
      manifestDigest: "sha256:m",
      contentDigest: "sha256:c",
      compatibility: "compatible",
    });
    expect(body.previous_revision_id).toBeUndefined();
    expect(body.local_source_path).toBe("skills/example");
    expect(skillBindBody({
      bindingId: "bind",
      revisionId: "rev",
      workspaceScope: "workspace://personal",
      targetKind: "workspace",
      targetRef: "workspace://personal",
    }).binding_id).toBe("bind");
  });

  it("labels the list as bindings, not packages", () => {
    expect(skillMasterFooter(2, false)).toBe(
      "Showing 2 skill bindings · list is bindings, not packages · envelope limit 64",
    );
    expect(skillMasterFooter(1, true)).toContain("envelope at bound");
    expect(SKILL_PERMISSION_ANNOTATION).toMatch(/grants no tool/);
  });
});
