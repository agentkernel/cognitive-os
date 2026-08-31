/**
 * Vault index projection (P11-T13 Dual Track).
 * Source is GET /management/project/v1/vault.index for one Project.
 * Files are not Project authority (P11-T10). caller_project_id equals
 * project_id so this client never performs a cross-project read.
 */

import { asList, asRecord } from "../projections";

export const VAULT_INDEX_KEY = "opc:vault-index";
export const VAULT_INDEX_PATH = "/management/project/v1/vault.index";

export function vaultIndexPath(projectId: string): string {
  const id = encodeURIComponent(projectId);
  return `${VAULT_INDEX_PATH}?project_id=${id}&caller_project_id=${id}`;
}

export interface VaultIndexEntry {
  entryId: string;
  documentId: string;
  layer: string;
  excerpt: string;
}

export function projectVaultIndex(body: unknown): VaultIndexEntry[] {
  const rows: VaultIndexEntry[] = [];
  for (const item of asList(body, ["entries"])) {
    const record = asRecord(item);
    if (typeof record.entry_id !== "string" || record.entry_id.length === 0) {
      continue;
    }
    rows.push({
      entryId: record.entry_id,
      documentId: typeof record.document_id === "string" ? record.document_id : "unknown",
      layer: typeof record.layer === "string" ? record.layer : "unknown",
      excerpt: typeof record.excerpt === "string" ? record.excerpt : "unknown",
    });
  }
  return rows;
}
