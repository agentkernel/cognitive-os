/*
 * Tools family (W7) view models — docs/design/18 §4.
 *
 * The family table is GET /management/resource/v1/tool (native catalog +
 * overlay), not the Resource Manager list envelope the hub counts. Registered
 * or enabled is not a production call chain.
 */

import { asList, asRecord } from "../projections";

export const TOOL_READINESS_CAVEAT =
  "registered/enabled ≠ production call chain wired";

export const TOOL_QUARANTINE_CONSEQUENCE =
  "Quarantine is one-way except revoke; a quarantined Tool cannot be enabled.";

export const TOOL_REVOKE_CONSEQUENCE =
  "Revoke is terminal; a revoked Tool cannot be re-enabled, disabled, or quarantined.";

export const TOOL_CATALOG_PATH = "/management/resource/v1/tool";
export const TOOL_CATALOG_KEY = "resources:tool:catalog";

export type ToolLifecycle = "enabled" | "disabled" | "quarantined" | "revoked" | "unknown";

export interface ToolCatalogRow {
  operationId: string;
  action?: string;
  family?: string;
  risk?: string;
  lifecycle: ToolLifecycle;
  executionReadiness?: string;
  descriptorDigest?: string;
  agentExposed?: boolean;
}

export interface ToolCatalogView {
  authoritySource?: string;
  resources: ToolCatalogRow[];
}

function asLifecycle(value: unknown): ToolLifecycle {
  const text = String(value ?? "").toLowerCase();
  if (text === "enabled" || text === "disabled" || text === "quarantined" || text === "revoked") {
    return text;
  }
  return "unknown";
}

export function projectToolCatalog(body: unknown): ToolCatalogView {
  const record = asRecord(body);
  return {
    authoritySource:
      record.authority_source == null ? undefined : String(record.authority_source),
    resources: asList(body, ["resources", "items"]).map((row) => {
      const item = asRecord(row);
      return {
        operationId: String(item.operation_id ?? item.id ?? "unknown"),
        action: item.action == null ? undefined : String(item.action),
        family: item.family == null ? undefined : String(item.family),
        risk: item.risk == null ? undefined : String(item.risk),
        lifecycle: asLifecycle(item.lifecycle ?? item.health ?? item.availability),
        executionReadiness:
          item.execution_readiness == null ? undefined : String(item.execution_readiness),
        descriptorDigest:
          item.descriptor_digest == null ? undefined : String(item.descriptor_digest),
        agentExposed: item.agent_exposed === true,
      };
    }),
  };
}

export type ToolMutationKind = "enable" | "disable" | "quarantine" | "revoke";

export function toolMutationPath(kind: ToolMutationKind): string {
  return `/management/resource/v1/tool/${kind}`;
}

export function toolMutationBody(operationId: string): { operation_id: string } {
  return { operation_id: operationId };
}

export function allowedToolMutations(lifecycle: ToolLifecycle): ToolMutationKind[] {
  if (lifecycle === "revoked" || lifecycle === "unknown") {
    return [];
  }
  if (lifecycle === "quarantined") {
    return ["disable", "revoke"];
  }
  if (lifecycle === "enabled") {
    return ["disable", "quarantine", "revoke"];
  }
  return ["enable", "quarantine", "revoke"];
}

export function toolMutationConsequence(kind: ToolMutationKind): string {
  if (kind === "quarantine") {
    return TOOL_QUARANTINE_CONSEQUENCE;
  }
  if (kind === "revoke") {
    return TOOL_REVOKE_CONSEQUENCE;
  }
  if (kind === "enable") {
    return `Enable writes the overlay only. ${TOOL_READINESS_CAVEAT}`;
  }
  return `Disable writes the overlay only. ${TOOL_READINESS_CAVEAT}`;
}

export function readinessLabel(readiness?: string): string {
  if (!readiness) {
    return "unknown";
  }
  if (readiness === "execution_ready") {
    return `execution-ready*`;
  }
  return readiness.split("_").join(" ");
}
