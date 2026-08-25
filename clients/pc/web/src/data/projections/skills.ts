/*
 * Skills family (W7) view models — docs/design/18 §3.
 *
 * The Resource Manager skill list is bindings, not packages. Explain is
 * GET /management/resource/v1/skill/binding/explain?id=. An enabled skill
 * grants no tool/filesystem/network/model capability.
 */

import { asRecord } from "../projections";

export const SKILL_PERMISSION_ANNOTATION =
  "An enabled skill grants no tool, filesystem, network, or model capability. Scripts execute only through registered tools.";

export const SKILL_IMPORT_HONESTY =
  "The browser does not read the local path. You supply the path and the digests the daemon will persist. Importing a package does not create a binding.";

export const SKILL_REVOKE_CONSEQUENCE =
  "Revoke writes a durable revocation; the binding cannot be rebound without a new bind. This is not a package delete.";

export function skillExplainPath(id: string): string {
  return `/management/resource/v1/skill/binding/explain?id=${encodeURIComponent(id)}`;
}

export function skillExplainKey(id: string): string {
  return `resources:skill:explain:${id}`;
}

export interface SkillExplainView {
  bindingId: string;
  revisionId?: string;
  workspaceScope?: string;
  targetKind?: string;
  targetRef?: string;
  status?: string;
  packageId?: string;
  manifestDigest?: string;
  contentDigest?: string;
  compatibility?: string;
  revocationReason?: string;
}

function optionalString(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return value == null || value === "" ? undefined : String(value);
}

function compatibilityFromBinding(binding: Record<string, unknown>): string | undefined {
  const direct = optionalString(binding, "compatibility");
  if (direct) {
    return direct;
  }
  const canonical = binding.canonical_json;
  if (typeof canonical !== "string") {
    return undefined;
  }
  try {
    return optionalString(asRecord(JSON.parse(canonical)), "compatibility");
  } catch {
    return undefined;
  }
}

export function projectSkillExplain(body: unknown): SkillExplainView {
  const record = asRecord(body);
  const binding = asRecord(record.binding ?? record);
  return {
    bindingId: String(binding.binding_id ?? binding.id ?? "unknown"),
    revisionId: optionalString(binding, "revision_id"),
    workspaceScope: optionalString(binding, "workspace_scope"),
    targetKind: optionalString(binding, "target_kind"),
    targetRef: optionalString(binding, "target_ref"),
    status: optionalString(binding, "status"),
    packageId: optionalString(binding, "package_id"),
    manifestDigest: optionalString(binding, "manifest_digest"),
    contentDigest: optionalString(binding, "content_digest"),
    compatibility: compatibilityFromBinding(binding),
    revocationReason: optionalString(binding, "revocation_reason"),
  };
}

export function skillMasterFooter(count: number, atBound: boolean): string {
  const bound = atBound ? "envelope at bound (limit 64)" : "envelope limit 64";
  return `Showing ${count} skill binding${count === 1 ? "" : "s"} · list is bindings, not packages · ${bound}`;
}

export interface SkillImportPreview {
  packageId: string;
  revisionId: string;
  workspaceScope: string;
  localSourcePath: string;
  provenanceRef: string;
  manifestDigest: string;
  contentDigest: string;
  compatibility: string;
}

export function skillImportBody(preview: SkillImportPreview): Record<string, string> {
  return {
    package_id: preview.packageId,
    revision_id: preview.revisionId,
    workspace_scope: preview.workspaceScope,
    local_source_path: preview.localSourcePath,
    provenance_ref: preview.provenanceRef,
    manifest_digest: preview.manifestDigest,
    content_digest: preview.contentDigest,
    compatibility: preview.compatibility,
  };
}

export interface SkillBindPreview {
  bindingId: string;
  revisionId: string;
  workspaceScope: string;
  targetKind: string;
  targetRef: string;
}

export function skillBindBody(preview: SkillBindPreview): Record<string, string> {
  return {
    binding_id: preview.bindingId,
    revision_id: preview.revisionId,
    workspace_scope: preview.workspaceScope,
    target_kind: preview.targetKind,
    target_ref: preview.targetRef,
  };
}
