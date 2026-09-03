/**
 * Attempt artifact / verifier / acceptance / publication projections
 * (P13-T04 Dual Track). Sources are the management HTTP routes
 * `outputs`, `outputs.detail`, `outputs.open`, `publication.packet`; writes
 * are `outputs.export`, `attempt.artifact.stage-test`,
 * `run.acceptance.request`, `publication.external-send.request`. This file
 * never invents an artifact, a verification, an acceptance or a publication:
 * every field is a daemon statement or the literal `unknown`. Files are not
 * authority; `published` is never true here.
 */

import { asList, asRecord } from "../projections";

export const OUTPUTS_PATH = "/management/project/v1/outputs";
export const OUTPUT_DETAIL_PATH = "/management/project/v1/outputs.detail";
export const OUTPUT_OPEN_PATH = "/management/project/v1/outputs.open";
export const OUTPUT_EXPORT_PATH = "/management/project/v1/outputs.export";
export const ARTIFACT_STAGE_TEST_PATH = "/management/project/v1/attempt.artifact.stage-test";
export const RUN_ACCEPTANCE_REQUEST_PATH = "/management/project/v1/run.acceptance.request";
export const PUBLICATION_PACKET_PATH = "/management/project/v1/publication.packet";
export const EXTERNAL_SEND_REQUEST_PATH =
  "/management/project/v1/publication.external-send.request";

export function outputsPath(projectId: string): string {
  return `${OUTPUTS_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function outputDetailPath(artifactId: string): string {
  return `${OUTPUT_DETAIL_PATH}?artifact_id=${encodeURIComponent(artifactId)}`;
}

export function outputOpenPath(artifactId: string): string {
  return `${OUTPUT_OPEN_PATH}?artifact_id=${encodeURIComponent(artifactId)}`;
}

export function publicationPacketPath(projectId: string, artifactId: string): string {
  return `${PUBLICATION_PACKET_PATH}?project_id=${encodeURIComponent(projectId)}&artifact_id=${encodeURIComponent(artifactId)}`;
}

export function outputsKey(projectId: string): string {
  return `opc:outputs:${projectId}`;
}

export function outputDetailKey(artifactId: string): string {
  return `opc:output-detail:${artifactId}`;
}

export function publicationPacketKey(projectId: string, artifactId: string): string {
  return `opc:publication-packet:${projectId}:${artifactId}`;
}

export interface OutputArtifactRow {
  artifactId: string;
  attemptId: string;
  employeeId: string;
  taskRef: string;
  casRef: string;
  byteLength: string;
  format: string;
  source: string;
  sourceFrameSeq: string;
  freshness: string;
  verificationStatus: string;
  latestEvidenceId: string;
  stageId: string;
  acceptedAt: string;
  producedAt: string;
}

export interface OutputEvidenceRow {
  evidenceId: string;
  verifierRef: string;
  principal: string;
  disposition: string;
  reportCasRef: string;
  checkedCasRef: string;
  verifiedAt: string;
  criteria: { id: string; result: string }[];
}

export interface OutputDetailRow {
  artifact: OutputArtifactRow;
  evidence: OutputEvidenceRow[];
  acceptanceId: string;
  acceptanceStageId: string;
  acceptanceLastRing: string;
  openRoute: string;
  exportExists: string;
  exportPath: string;
  filesAreAuthority: boolean;
}

export interface PublicationPacketRow {
  planned: boolean;
  published: boolean;
  chatCanConfirm: boolean;
  connector: string;
  artifactId: string;
  verified: string;
  accepted: string;
  sections: { id: string; facts: { key: string; value: string }[] }[];
}

function stated(value: unknown, fallback = "unknown"): string {
  if (typeof value === "string" && value.length > 0) {
    return value;
  }
  if (typeof value === "number") {
    return String(value);
  }
  if (typeof value === "boolean") {
    return String(value);
  }
  return fallback;
}

function statedNullable(value: unknown): string {
  return value === null || value === undefined ? "none" : stated(value);
}

function artifactRow(record: Record<string, unknown>): OutputArtifactRow | undefined {
  if (typeof record.artifact_id !== "string" || record.artifact_id.length === 0) {
    return undefined;
  }
  return {
    artifactId: record.artifact_id,
    attemptId: stated(record.attempt_id),
    employeeId: stated(record.employee_id),
    taskRef: stated(record.task_ref),
    casRef: stated(record.cas_ref),
    byteLength: stated(record.byte_length),
    format: stated(record.format),
    source: stated(record.source),
    sourceFrameSeq: stated(record.source_frame_seq),
    freshness: stated(record.freshness),
    verificationStatus: stated(record.verification_status, "not-run"),
    latestEvidenceId: statedNullable(record.latest_evidence_id),
    stageId: statedNullable(record.stage_id),
    acceptedAt: statedNullable(record.accepted_at),
    producedAt: stated(record.produced_at),
  };
}

export function projectOutputs(body: unknown): OutputArtifactRow[] {
  const rows: OutputArtifactRow[] = [];
  for (const item of asList(body, ["artifacts"])) {
    const row = artifactRow(asRecord(item));
    if (row) {
      rows.push(row);
    }
  }
  return rows;
}

function evidenceRow(record: Record<string, unknown>): OutputEvidenceRow | undefined {
  if (typeof record.evidence_id !== "string" || record.evidence_id.length === 0) {
    return undefined;
  }
  const criteria: { id: string; result: string }[] = [];
  if (Array.isArray(record.criteria)) {
    for (const item of record.criteria) {
      const criterion = asRecord(item);
      if (typeof criterion.id === "string") {
        criteria.push({ id: criterion.id, result: stated(criterion.result) });
      }
    }
  }
  return {
    evidenceId: record.evidence_id,
    verifierRef: stated(record.verifier_ref),
    principal: stated(record.principal),
    disposition: stated(record.disposition),
    reportCasRef: stated(record.report_cas_ref),
    checkedCasRef: stated(record.checked_cas_ref),
    verifiedAt: stated(record.verified_at),
    criteria,
  };
}

export function projectOutputDetail(body: unknown): OutputDetailRow[] {
  const record = asRecord(body);
  const artifact = artifactRow(asRecord(record.artifact));
  if (!artifact) {
    return [];
  }
  const evidence: OutputEvidenceRow[] = [];
  if (Array.isArray(record.evidence)) {
    for (const item of record.evidence) {
      const row = evidenceRow(asRecord(item));
      if (row) {
        evidence.push(row);
      }
    }
  }
  const acceptance = asRecord(record.run_acceptance);
  const exportRecord = asRecord(record.export);
  return [
    {
      artifact,
      evidence,
      acceptanceId: statedNullable(acceptance.acceptance_id),
      acceptanceStageId: statedNullable(acceptance.stage_id),
      acceptanceLastRing: statedNullable(acceptance.last_ring),
      openRoute: stated(record.open_route),
      exportExists: stated(exportRecord.exists),
      exportPath: stated(exportRecord.path, "none"),
      filesAreAuthority: record.files_are_authority === true,
    },
  ];
}

const PACKET_SECTIONS = [
  "preview",
  "override",
  "tiered_authority",
  "observable",
  "outcome_verify",
  "memory_of_actions",
  "yield",
] as const;

export function projectPublicationPacket(body: unknown): PublicationPacketRow[] {
  const record = asRecord(body);
  const artifact = asRecord(record.artifact);
  if (typeof artifact.artifact_id !== "string" || artifact.artifact_id.length === 0) {
    return [];
  }
  const packet = asRecord(record.autonomy_packet);
  const verify = asRecord(packet.outcome_verify);
  const sections = PACKET_SECTIONS.map((id) => {
    const section = asRecord(packet[id]);
    const facts = Object.entries(section).map(([key, value]) => ({
      key,
      value: Array.isArray(value)
        ? value.map((item) => stated(item)).join(", ")
        : statedNullable(value),
    }));
    return { id, facts };
  });
  return [
    {
      // `planned` and `published` are daemon statements; a missing or
      // non-boolean `published` is never read as false-safe success.
      planned: record.planned === true,
      published: record.published !== false,
      chatCanConfirm: record.chat_can_confirm !== false,
      connector: stated(record.connector),
      artifactId: artifact.artifact_id,
      verified: stated(verify.verified),
      accepted: stated(verify.accepted),
      sections,
    },
  ];
}

/** Last ring = the highest position on the current axis. */
export function lastRingStageId(
  stages: { stageId: string; position: string }[],
): string | undefined {
  let best: { stageId: string; position: number } | undefined;
  for (const stage of stages) {
    const position = Number.parseInt(stage.position, 10);
    if (Number.isNaN(position)) {
      continue;
    }
    if (!best || position > best.position) {
      best = { stageId: stage.stageId, position };
    }
  }
  return best?.stageId;
}

/** Close-out is offered only for a verified, current, last-ring artifact that is not yet accepted. */
export function acceptanceOfferable(
  artifact: OutputArtifactRow | undefined,
  lastRing: string | undefined,
): boolean {
  return Boolean(
    artifact &&
      lastRing &&
      artifact.stageId === lastRing &&
      artifact.verificationStatus === "passed" &&
      artifact.freshness === "current" &&
      artifact.acceptedAt === "none",
  );
}
