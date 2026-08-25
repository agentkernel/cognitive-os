/*
 * System space view models — docs/design/20, wave 9 in 39.
 *
 * Doctor sub-sections that the daemon marks not-probed stay named
 * unavailable. Backup/restore receipts never carry secrets. No fetching here.
 */

import { asList, asRecord } from "../projections";

export const SYSTEM_SECTIONS = [
  { id: "readiness", title: "Readiness" },
  { id: "doctor", title: "Doctor" },
  { id: "stewardship", title: "Stewardship" },
  { id: "session", title: "Session" },
  { id: "about", title: "About" },
] as const;

export type SystemSectionId = (typeof SYSTEM_SECTIONS)[number]["id"];

export function isSystemSection(value: string | null | undefined): value is SystemSectionId {
  return SYSTEM_SECTIONS.some((section) => section.id === value);
}

export const SYSTEM_CLAIM_CEILING =
  "Local facts; no Gate, release, or Profile claim. Static checks are not runtime readiness.";

export const SYSTEM_BACKUP_NEVER =
  "Secrets and raw SQLite are never included. Re-binding secrets after a machine move is a follow-up step, not part of restore.";

export const SYSTEM_RESTORE_409 =
  "Restore is live-apply. 409 classes the daemon names: tampered, schema-incompatible, incomplete, partial-refused, daemon-lock (RESOURCE_BACKUP_TAMPERED and siblings). Restore never auto-retries.";

export const SYSTEM_DOCTOR_REDACTION =
  "Doctor output is redacted facts and digests; it never contains secrets, raw prompts, or provider traffic. Browser-side bundle download is not offered — use `cognitive doctor --bundle`.";

export interface DoctorFact {
  key: string;
  value: string;
}

export interface DoctorComponentView {
  name: string;
  state: string;
  source?: string;
  observedAtMs?: number;
  durationMs?: number;
  errorClass?: string;
  facts: DoctorFact[];
}

export interface DoctorSubsectionView {
  schema?: string;
  overall?: string;
  errorCode?: string;
  probed: boolean;
}

export interface DoctorView {
  overall: string;
  firstConversationReady?: boolean;
  evaluatedAtMs?: number;
  components: DoctorComponentView[];
  guidance: string[];
  sixResource: DoctorSubsectionView;
  headlessVault: DoctorSubsectionView;
  operability: DoctorSubsectionView;
  staticCheckIsNotRuntimeReady: boolean;
  profileClaim: string;
  gateClaim: string;
}

function subsection(raw: unknown): DoctorSubsectionView {
  const record = asRecord(raw);
  const errorCode = record.error_code == null ? undefined : String(record.error_code);
  const probed = !(errorCode ?? "").includes("NOT_PROBED");
  return {
    schema: record.schema == null && record.surface == null ? undefined : String(record.schema ?? record.surface),
    overall: record.overall == null ? undefined : String(record.overall),
    errorCode,
    probed,
  };
}

export function projectDoctor(body: unknown): DoctorView {
  const record = asRecord(body);
  const componentsRaw = Array.isArray(record.components) ? record.components : [];
  const guidanceRaw = Array.isArray(record.guidance) ? record.guidance : [];
  const evaluated = Number(record.evaluated_at_unix_ms);
  return {
    overall: String(record.overall ?? "unknown"),
    firstConversationReady:
      typeof record.first_conversation_ready === "boolean"
        ? record.first_conversation_ready
        : undefined,
    evaluatedAtMs:
      record.evaluated_at_unix_ms != null && Number.isFinite(evaluated) ? evaluated : undefined,
    components: componentsRaw.map((item) => {
      const component = asRecord(item);
      const observed = Number(component.observed_at_unix_ms);
      const duration = Number(component.duration_ms);
      return {
        name: String(component.component ?? component.name ?? "unknown"),
        state: String(component.status ?? component.state ?? "unknown"),
        source: component.source == null ? undefined : String(component.source),
        observedAtMs:
          component.observed_at_unix_ms != null && Number.isFinite(observed) ? observed : undefined,
        durationMs:
          component.duration_ms != null && Number.isFinite(duration) ? duration : undefined,
        errorClass: component.error_class == null ? undefined : String(component.error_class),
        facts: asList(component, ["facts"]).map((fact) => {
          const row = asRecord(fact);
          return {
            key: String(row.key ?? "unknown"),
            value: String(row.value ?? "unknown"),
          };
        }),
      };
    }),
    guidance: guidanceRaw.map((item) => String(item)),
    sixResource: subsection(record.six_resource),
    headlessVault: subsection(record.headless_vault),
    operability: subsection(record.operability),
    staticCheckIsNotRuntimeReady: record.static_check_is_not_runtime_ready !== false,
    profileClaim: String(record.profile_claim ?? "not-claimed"),
    gateClaim: String(record.gate_claim ?? "not-claimed"),
  };
}

export interface BackupReceiptView {
  archiveId?: string;
  archivePath?: string;
  manifestDigest?: string;
  excludedSecretCount?: number;
  sqliteCopied?: boolean;
}

export function projectBackupReceipt(body: unknown): BackupReceiptView {
  const record = asRecord(body);
  const excluded = Number(record.excluded_secret_count);
  return {
    archiveId: record.archive_id == null ? undefined : String(record.archive_id),
    archivePath: record.archive_path == null ? undefined : String(record.archive_path),
    manifestDigest: record.manifest_digest == null ? undefined : String(record.manifest_digest),
    excludedSecretCount:
      record.excluded_secret_count != null && Number.isFinite(excluded) ? excluded : undefined,
    sqliteCopied: typeof record.sqlite_copied === "boolean" ? record.sqlite_copied : undefined,
  };
}

export interface RestoreReceiptView {
  liveApplied?: boolean;
  archiveId?: string;
}

export function projectRestoreReceipt(body: unknown): RestoreReceiptView {
  const record = asRecord(body);
  return {
    liveApplied: typeof record.live_applied === "boolean" ? record.live_applied : undefined,
    archiveId: record.archive_id == null ? undefined : String(record.archive_id),
  };
}
