/**
 * Vault index / ingest projection (P11-T13 Dual Track + P12-T07 ingest).
 * Source is GET /management/project/v1/vault.index for one Project.
 * Import is POST /management/project/v1/vault.import (owner-paste).
 * Files are not Project authority (P11-T10). caller_project_id equals
 * project_id so this client never performs a cross-project read.
 */

import { asList, asRecord } from "../projections";

export const VAULT_INDEX_KEY = "opc:vault-index";
export const VAULT_INDEX_PATH = "/management/project/v1/vault.index";
export const VAULT_INJECT_ORDER_KEY = "opc:vault-inject-order";
export const VAULT_CONFLICTS_KEY = "opc:vault-conflicts";
export const VAULT_CONFLICTS_PATH = "/management/project/v1/vault.conflicts";
export const VAULT_IMPORT_PATH = "/management/project/v1/vault.import";
export const VAULT_REBUILD_PATH = "/management/project/v1/vault.index.rebuild";
export const VAULT_LABELED_PATH = "/management/resource/v1/vault.labeled";
export const VAULT_DOCUMENTS_PATH = "/management/resource/v1/vault.documents";
export const VAULT_RIGHTS_CLASSES = [
  "owner-owned",
  "licensed",
  "open-license",
  "public-domain",
  "citation-only",
] as const;

export function vaultIndexPath(projectId: string): string {
  const id = encodeURIComponent(projectId);
  return `${VAULT_INDEX_PATH}?project_id=${id}&caller_project_id=${id}`;
}

export function vaultConflictsPath(projectId: string): string {
  const id = encodeURIComponent(projectId);
  return `${VAULT_CONFLICTS_PATH}?project_id=${id}&caller_project_id=${id}`;
}

export function vaultLabeledPath(projectId: string): string {
  const id = encodeURIComponent(projectId);
  return `${VAULT_LABELED_PATH}?project_id=${id}&caller_project_id=${id}`;
}

export function vaultDocumentsPath(projectId: string): string {
  const id = encodeURIComponent(projectId);
  return `${VAULT_DOCUMENTS_PATH}?project_id=${id}&caller_project_id=${id}`;
}

export interface VaultIndexEntry {
  entryId: string;
  documentId: string;
  layer: string;
  excerpt: string;
}

export interface VaultLabeledEntry {
  entryId: string;
  documentId: string;
  relativePath: string;
  excerpt: string;
  layer: string;
  provenanceSourceUri: string;
  rightsClass: string;
  freshness: string;
  exclusion: string;
  exclusionReason: string;
  untrustedObservation: boolean;
  isAuthority: boolean;
}

export interface VaultDocumentStatus {
  documentId: string;
  relativePath: string;
  provenanceSourceUri: string;
  indexStatus: string;
}

export interface VaultConflictRow {
  conflictId: string;
  relativePath: string;
  incumbentDocumentId: string;
  incomingDocumentId: string;
  resolution: string;
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

/** Daemon inject_order from vault.index. Never invents Task-contract layers. */
export function projectVaultInjectOrder(body: unknown): string[] {
  const record = asRecord(body);
  if (!Array.isArray(record.inject_order)) {
    return [];
  }
  return record.inject_order.filter(
    (item): item is string => typeof item === "string" && item.length > 0,
  );
}

export function projectVaultConflicts(body: unknown): VaultConflictRow[] {
  const rows: VaultConflictRow[] = [];
  for (const item of asList(body, ["conflicts"])) {
    const record = asRecord(item);
    if (typeof record.conflict_id !== "string" || record.conflict_id.length === 0) {
      continue;
    }
    rows.push({
      conflictId: record.conflict_id,
      relativePath: typeof record.relative_path === "string" ? record.relative_path : "unknown",
      incumbentDocumentId:
        typeof record.incumbent_document_id === "string" ? record.incumbent_document_id : "unknown",
      incomingDocumentId:
        typeof record.incoming_document_id === "string" ? record.incoming_document_id : "unknown",
      resolution: typeof record.resolution === "string" ? record.resolution : "unknown",
    });
  }
  return rows;
}

export function projectVaultLabeled(body: unknown): VaultLabeledEntry[] {
  const rows: VaultLabeledEntry[] = [];
  for (const item of asList(body, ["entries"])) {
    const record = asRecord(item);
    if (typeof record.entry_id !== "string" || record.entry_id.length === 0) {
      continue;
    }
    rows.push({
      entryId: record.entry_id,
      documentId: typeof record.document_id === "string" ? record.document_id : "unknown",
      relativePath: typeof record.relative_path === "string" ? record.relative_path : "unknown",
      excerpt: typeof record.excerpt === "string" ? record.excerpt : "unknown",
      layer: typeof record.layer === "string" ? record.layer : "unknown",
      provenanceSourceUri:
        typeof record.provenance_source_uri === "string" ? record.provenance_source_uri : "unknown",
      rightsClass: typeof record.rights_class === "string" ? record.rights_class : "unknown",
      freshness: typeof record.freshness === "string" ? record.freshness : "unknown",
      exclusion: typeof record.exclusion === "string" ? record.exclusion : "unknown",
      exclusionReason: typeof record.exclusion_reason === "string" ? record.exclusion_reason : "",
      untrustedObservation: record.untrusted_observation === true,
      isAuthority: record.is_authority === true,
    });
  }
  return rows;
}

export function projectVaultDocuments(body: unknown): VaultDocumentStatus[] {
  const rows: VaultDocumentStatus[] = [];
  for (const item of asList(body, ["documents"])) {
    const record = asRecord(item);
    if (typeof record.document_id !== "string" || record.document_id.length === 0) {
      continue;
    }
    rows.push({
      documentId: record.document_id,
      relativePath: typeof record.relative_path === "string" ? record.relative_path : "unknown",
      provenanceSourceUri:
        typeof record.provenance_source_uri === "string" ? record.provenance_source_uri : "unknown",
      indexStatus: typeof record.index_status === "string" ? record.index_status : "unknown",
    });
  }
  return rows;
}

export function vaultImportIsAuthority(body: unknown): boolean {
  const record = asRecord(body);
  return record.is_authority === true;
}
