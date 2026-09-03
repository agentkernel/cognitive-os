//! Attempt artifacts → daemon CAS → independent verifier evidence → last-ring
//! run acceptance → publication packet / external-send (P13-T04, authority
//! migration v37).
//!
//! A hosted Attempt (P13-T02, v36) ends with a daemon-observed terminal row and
//! an observation ledger. The daemon — never the child — ingests each
//! `DeliverableDraft` candidate frame of a *terminal* Attempt into the single
//! P3-T03 `ArtifactStore` CAS: the canonical candidate payload lands under the
//! digest the broker already recorded on the frame, and the deliverable text
//! lands under its own digest. The artifact row carries digest / format /
//! source frame / freshness; it never carries a filesystem path, and a path is
//! never accepted as an artifact reference (file is not authority).
//!
//! Completion belongs to the independent verifier
//! (`verifier://personal/attempt-artifact`, principal
//! `principal://personal/independent-verifier`): deterministic checks only —
//! CAS re-read digest equality, source-frame binding, terminal Attempt,
//! format parse, non-empty, no secret shape. The child's `response done`,
//! exit code, HTTP receipt, and prose are recorded as `not-used`, never as an
//! input. Evidence rows are append-only and the report bytes live in the same
//! CAS. StageTestPassed (P11-T03) is *derived* from that evidence plus seating
//! and the CAS re-read; there is no caller `passed`. Run acceptance is minted
//! only for the last ring of the current plan revision and only when a current
//! StageTestPassed exists there, through the P11-T09 ApprovalPreview
//! (`run-acceptance`). External send also goes through ApprovalPreview
//! (`external-send`); the confirmed row is a persist-before-dispatch Intent in
//! state `planned` — `published` is unrepresentable in v37 because no qualified
//! connector exists (planned ≠ published).

use crate::artifact_store::{ArtifactStore, ArtifactStoreError};
use crate::employee::EmployeeStore;
use crate::hosted_dsh_attempt::{HostedAttemptFrameRow, HostedAttemptRow, HostedDshAttemptStore};
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{
    ConfirmCaller, ConfirmResult, ProjectAggregateError, ProjectAggregateStore, SeatingFacts,
    StageTestOracle,
};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private artifact / evidence envelope. Hidden capability, not chrome.
pub const ATTEMPT_ARTIFACT_PROJECTION_ID: &str = "cognitiveos.personal.attempt-artifact/0.1";
/// The registered independent verifier identity for business artifacts.
pub const ATTEMPT_ARTIFACT_VERIFIER_REF: &str = "verifier://personal/attempt-artifact";
/// Verifier version pinned into every evidence row.
pub const ATTEMPT_ARTIFACT_VERIFIER_VERSION: &str = "v1";
/// Principal every evidence row is written under (same principal the P2-T13
/// production verifier uses; distinct from the child and from the owner).
pub const ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL: &str = "principal://personal/independent-verifier";
/// Only source an artifact can have in v37.
pub const ATTEMPT_ARTIFACT_SOURCE: &str = "hosted-dsh-child:candidate:DeliverableDraft";
/// Deliverable text format derived for `DeliverableDraft` candidates.
pub const ATTEMPT_ARTIFACT_FORMAT_MARKDOWN: &str = "text/markdown";
/// Canonical candidate payload ceiling (bytes) the daemon will ingest.
pub const ATTEMPT_ARTIFACT_MAX_BYTES: usize = 256 * 1024;
/// ApprovalPreview subject kind for last-ring run acceptance.
pub const RUN_ACCEPTANCE_SUBJECT_KIND: &str = "run-acceptance";
/// ApprovalPreview subject kind for external send of a publication packet.
pub const EXTERNAL_SEND_SUBJECT_KIND: &str = "external-send";
/// The only connector state v37 can record: none is qualified.
pub const EXTERNAL_SEND_CONNECTOR_NONE: &str = "none-qualified";

/// Authority migration v37: artifact / evidence / run acceptance / external
/// send ledgers plus the `run-acceptance` and `external-send` ApprovalPreview
/// subject kinds (table rebuild, v30 precedent).
pub const ATTEMPT_ARTIFACT_SCHEMA_V37: &str = "
CREATE TABLE p13_attempt_artifact (
  artifact_id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL REFERENCES p13_hosted_dsh_attempt(attempt_id),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  task_ref TEXT NOT NULL,
  employee_id TEXT NOT NULL,
  cas_ref TEXT NOT NULL CHECK (cas_ref LIKE 'sha256:%' AND length(cas_ref) = 71),
  byte_length INTEGER NOT NULL CHECK (byte_length > 0),
  format TEXT NOT NULL CHECK (format IN ('text/markdown')),
  source TEXT NOT NULL CHECK (source = 'hosted-dsh-child:candidate:DeliverableDraft'),
  source_frame_seq INTEGER NOT NULL,
  source_payload_digest TEXT NOT NULL CHECK (length(source_payload_digest) = 64),
  context_digest TEXT NOT NULL CHECK (length(context_digest) = 64),
  produced_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(attempt_id, source_frame_seq)
) STRICT;
CREATE INDEX p13_attempt_artifact_project
  ON p13_attempt_artifact(project_id, created_at);
CREATE TRIGGER p13_attempt_artifact_append_only_update
BEFORE UPDATE ON p13_attempt_artifact
BEGIN SELECT RAISE(ABORT, 'append-only: attempt artifacts are immutable'); END;
CREATE TRIGGER p13_attempt_artifact_append_only_delete
BEFORE DELETE ON p13_attempt_artifact
BEGIN SELECT RAISE(ABORT, 'append-only: attempt artifacts are immutable'); END;
CREATE TABLE p13_artifact_evidence (
  evidence_id TEXT PRIMARY KEY,
  artifact_id TEXT NOT NULL REFERENCES p13_attempt_artifact(artifact_id),
  verifier_ref TEXT NOT NULL CHECK (verifier_ref = 'verifier://personal/attempt-artifact'),
  verifier_version TEXT NOT NULL,
  principal TEXT NOT NULL CHECK (principal = 'principal://personal/independent-verifier'),
  disposition TEXT NOT NULL CHECK (disposition IN ('passed','failed','indeterminate')),
  criteria_json TEXT NOT NULL,
  report_cas_ref TEXT NOT NULL CHECK (report_cas_ref LIKE 'sha256:%' AND length(report_cas_ref) = 71),
  checked_cas_ref TEXT NOT NULL,
  verified_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p13_artifact_evidence_artifact
  ON p13_artifact_evidence(artifact_id, verified_at);
CREATE TRIGGER p13_artifact_evidence_append_only_update
BEFORE UPDATE ON p13_artifact_evidence
BEGIN SELECT RAISE(ABORT, 'append-only: artifact evidence is immutable'); END;
CREATE TRIGGER p13_artifact_evidence_append_only_delete
BEFORE DELETE ON p13_artifact_evidence
BEGIN SELECT RAISE(ABORT, 'append-only: artifact evidence is immutable'); END;
CREATE TABLE p13_run_acceptance (
  acceptance_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  plan_revision_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  stage_position INTEGER NOT NULL,
  stage_count INTEGER NOT NULL CHECK (stage_count > 0 AND stage_position = stage_count - 1),
  stage_test_fact_id TEXT NOT NULL UNIQUE,
  artifact_id TEXT NOT NULL REFERENCES p13_attempt_artifact(artifact_id),
  evidence_id TEXT NOT NULL REFERENCES p13_artifact_evidence(evidence_id),
  acceptance_decision_ref TEXT NOT NULL,
  decision_json TEXT NOT NULL,
  accepted_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p13_run_acceptance_project
  ON p13_run_acceptance(project_id, accepted_at);
CREATE TRIGGER p13_run_acceptance_append_only_update
BEFORE UPDATE ON p13_run_acceptance
BEGIN SELECT RAISE(ABORT, 'append-only: run acceptance facts are immutable'); END;
CREATE TRIGGER p13_run_acceptance_append_only_delete
BEFORE DELETE ON p13_run_acceptance
BEGIN SELECT RAISE(ABORT, 'append-only: run acceptance facts are immutable'); END;
CREATE TABLE p13_external_send (
  send_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  artifact_id TEXT NOT NULL REFERENCES p13_attempt_artifact(artifact_id),
  evidence_id TEXT NOT NULL REFERENCES p13_artifact_evidence(evidence_id),
  acceptance_id TEXT,
  preview_id TEXT NOT NULL,
  packet_digest TEXT NOT NULL CHECK (length(packet_digest) = 64),
  recipient_count INTEGER NOT NULL CHECK (recipient_count > 0),
  recipients_digest TEXT NOT NULL CHECK (length(recipients_digest) = 64),
  state TEXT NOT NULL CHECK (state IN ('previewed','planned','superseded')),
  published INTEGER NOT NULL CHECK (published = 0),
  connector TEXT NOT NULL CHECK (connector = 'none-qualified'),
  intent_persisted INTEGER NOT NULL CHECK (intent_persisted = 1),
  receipt_ref TEXT,
  created_at INTEGER NOT NULL,
  planned_at INTEGER
) STRICT;
CREATE INDEX p13_external_send_project
  ON p13_external_send(project_id, created_at);
CREATE TRIGGER p13_external_send_no_delete
BEFORE DELETE ON p13_external_send
BEGIN SELECT RAISE(ABORT, 'append-only: external send intents are never deleted'); END;
CREATE TABLE p11_approval_preview_v37 (
  preview_id TEXT PRIMARY KEY,
  subject_kind TEXT NOT NULL CHECK (subject_kind IN (
    'activation','plan-change','acceptance','grant-expansion','run-acceptance','external-send'
  )),
  subject_ref TEXT NOT NULL,
  base_state_digest TEXT NOT NULL CHECK (length(base_state_digest) = 64),
  preview_bytes_ref TEXT NOT NULL,
  preview_digest TEXT NOT NULL UNIQUE CHECK (length(preview_digest) = 64),
  status TEXT NOT NULL CHECK (status IN (
    'pending','approved','rejected','stale','consumed','superseded'
  )),
  intent_id TEXT,
  receipt_ref TEXT,
  created_at INTEGER NOT NULL,
  decided_at INTEGER,
  superseded_by TEXT
) STRICT;
INSERT INTO p11_approval_preview_v37 (
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
) SELECT
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
FROM p11_approval_preview;
DROP TABLE p11_approval_preview;
ALTER TABLE p11_approval_preview_v37 RENAME TO p11_approval_preview;
";

/// v37 migration entry.
pub fn attempt_artifact_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(37, ATTEMPT_ARTIFACT_SCHEMA_V37)
}

/// Daemon-side ingest input: one candidate frame of one terminal Attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactIngestSpec<'a> {
    pub attempt_id: &'a str,
    pub source_frame_seq: i64,
    /// Canonical JSON of the candidate `payload` (the bytes whose SHA-256 the
    /// broker recorded on the frame).
    pub payload_canonical: &'a str,
    pub now_ms: i64,
}

/// Owner request for an external send of one verified artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalSendSpec<'a> {
    pub project_id: &'a str,
    pub artifact_id: &'a str,
    pub recipients: &'a [String],
    pub now_ms: i64,
}

/// Durable artifact row enriched with derived freshness / verification / acceptance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptArtifactRow {
    pub artifact_id: String,
    pub attempt_id: String,
    pub project_id: String,
    pub task_ref: String,
    pub employee_id: String,
    pub cas_ref: String,
    pub byte_length: i64,
    pub format: String,
    pub source: String,
    pub source_frame_seq: i64,
    pub source_payload_digest: String,
    pub context_digest: String,
    pub produced_at: i64,
    pub created_at: i64,
    /// `current` (newest artifact for this Project + task) or `superseded`.
    pub freshness: String,
    /// `not-run` | latest evidence disposition.
    pub verification_status: String,
    pub latest_evidence_id: Option<String>,
    /// Stage whose *current* StageTestPassed fact points at this artifact's evidence.
    pub stage_id: Option<String>,
    pub accepted_at: Option<i64>,
}

/// Append-only verifier evidence row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactEvidenceRow {
    pub evidence_id: String,
    pub artifact_id: String,
    pub verifier_ref: String,
    pub verifier_version: String,
    pub principal: String,
    pub disposition: String,
    pub criteria_json: String,
    pub report_cas_ref: String,
    pub checked_cas_ref: String,
    pub verified_at: i64,
}

/// Append-only run acceptance fact (last ring only, by CHECK).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunAcceptanceRow {
    pub acceptance_id: String,
    pub project_id: String,
    pub plan_revision_id: String,
    pub stage_id: String,
    pub stage_position: i64,
    pub stage_count: i64,
    pub stage_test_fact_id: String,
    pub artifact_id: String,
    pub evidence_id: String,
    pub acceptance_decision_ref: String,
    pub accepted_at: i64,
}

/// External send Intent row. `published` is always false in v37.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSendRow {
    pub send_id: String,
    pub project_id: String,
    pub artifact_id: String,
    pub evidence_id: String,
    pub acceptance_id: Option<String>,
    pub preview_id: String,
    pub preview_digest: String,
    pub packet_digest: String,
    pub recipient_count: i64,
    pub state: String,
    pub published: bool,
    pub connector: String,
    pub intent_persisted: bool,
    pub receipt_ref: Option<String>,
    pub created_at: i64,
    pub planned_at: Option<i64>,
}

/// Attempt artifact ledger over the daemon-owned writer.
#[derive(Clone)]
pub struct AttemptArtifactStore {
    conn: Arc<Mutex<Connection>>,
    attempts: HostedDshAttemptStore,
}

impl AttemptArtifactStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
            attempts: HostedDshAttemptStore::from_authority_store(store),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let attempts = HostedDshAttemptStore::open_path(path)?;
        Ok(Self {
            conn: attempts.conn_arc(),
            attempts,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// SHA-256 hex of bytes (the same function the broker and CAS use).
    pub fn digest_hex(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    /// The only artifact reference syntax the daemon accepts. A filesystem
    /// path, `file://` URI, or `artifact://` URI is never an openable ref.
    pub fn resolve_openable_ref(&self, reference: &str) -> Result<String, ProjectAggregateError> {
        let Some(digest) = reference.strip_prefix("sha256:") else {
            return Err(ProjectAggregateError::Invalid {
                detail: "artifact reference must be a sha256: CAS digest; files are not authority",
            });
        };
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ProjectAggregateError::Invalid {
                detail: "artifact reference must be a sha256: CAS digest; files are not authority",
            });
        }
        Ok(reference.to_owned())
    }

    // ------------------------------------------------------------------
    // Ingest (daemon-side, after the terminal observation)
    // ------------------------------------------------------------------

    /// Put one `DeliverableDraft` candidate of a terminal Attempt into the CAS
    /// and record the artifact row. The payload must hash to the digest the
    /// broker recorded on the observed frame; the deliverable text is the
    /// `text` field. Verification stays `not-run` until the verifier ran.
    pub fn ingest_candidate(
        &self,
        cas: &ArtifactStore,
        spec: &ArtifactIngestSpec<'_>,
    ) -> Result<AttemptArtifactRow, ProjectAggregateError> {
        let attempt = self.require_attempt(spec.attempt_id)?;
        if attempt.state != "terminal" {
            return Err(ProjectAggregateError::Rejected {
                detail: "artifacts are ingested only from a daemon-observed terminal Attempt",
            });
        }
        if spec.payload_canonical.len() > ATTEMPT_ARTIFACT_MAX_BYTES {
            return Err(ProjectAggregateError::Invalid {
                detail: "candidate payload exceeds the artifact ceiling",
            });
        }
        let frame = self.candidate_frame(spec.attempt_id, spec.source_frame_seq)?;
        let payload_digest = Self::digest_hex(spec.payload_canonical.as_bytes());
        if frame.payload_digest.as_deref() != Some(payload_digest.as_str()) {
            return Err(ProjectAggregateError::Rejected {
                detail: "payload does not match the observed candidate frame digest",
            });
        }
        let text = deliverable_text(spec.payload_canonical)?;
        {
            let conn = self.lock()?;
            let existing: Option<String> = conn
                .query_row(
                    "SELECT artifact_id FROM p13_attempt_artifact
                      WHERE attempt_id = ?1 AND source_frame_seq = ?2",
                    params![spec.attempt_id, spec.source_frame_seq],
                    |row| row.get(0),
                )
                .optional()
                .map_err(unavailable("duplicate artifact lookup"))?;
            if existing.is_some() {
                return Err(ProjectAggregateError::Conflict {
                    detail: "this candidate frame was already ingested",
                });
            }
        }
        let text_digest = Self::digest_hex(text.as_bytes());
        let cas_ref = format!("sha256:{text_digest}");
        cas.put_with_metadata(
            &format!("sha256:{payload_digest}"),
            spec.payload_canonical.as_bytes(),
            "application/vnd.cognitiveos.attempt-candidate+json",
        )
        .map_err(cas_error("put source payload"))?;
        cas.put_with_metadata(&cas_ref, text.as_bytes(), ATTEMPT_ARTIFACT_FORMAT_MARKDOWN)
            .map_err(cas_error("put deliverable"))?;
        let artifact_id = next_id("artifact");
        {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p13_attempt_artifact (
                    artifact_id, attempt_id, project_id, task_ref, employee_id, cas_ref,
                    byte_length, format, source, source_frame_seq, source_payload_digest,
                    context_digest, produced_at, created_at
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    artifact_id,
                    attempt.attempt_id,
                    attempt.project_id,
                    attempt.task_ref,
                    attempt.employee_id,
                    cas_ref,
                    i64::try_from(text.len()).unwrap_or(i64::MAX),
                    ATTEMPT_ARTIFACT_FORMAT_MARKDOWN,
                    ATTEMPT_ARTIFACT_SOURCE,
                    spec.source_frame_seq,
                    payload_digest,
                    attempt.context_digest,
                    attempt.terminal_at.unwrap_or(spec.now_ms),
                    spec.now_ms
                ],
            )
            .map_err(unavailable("insert attempt artifact"))?;
        }
        self.require_artifact(&artifact_id)
    }

    fn candidate_frame(
        &self,
        attempt_id: &str,
        seq: i64,
    ) -> Result<HostedAttemptFrameRow, ProjectAggregateError> {
        let frames = self.attempts.list_frames(attempt_id, 512)?;
        let frame = frames.into_iter().find(|frame| frame.seq == seq).ok_or(
            ProjectAggregateError::NotFound {
                detail: "source frame not found on the Attempt ledger",
            },
        )?;
        if frame.kind != "candidate" || frame.operation.as_deref() != Some("DeliverableDraft") {
            return Err(ProjectAggregateError::Rejected {
                detail: "only a DeliverableDraft candidate frame can become an artifact",
            });
        }
        Ok(frame)
    }

    fn require_attempt(&self, attempt_id: &str) -> Result<HostedAttemptRow, ProjectAggregateError> {
        self.attempts
            .get_attempt(attempt_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "attempt not found",
            })
    }

    // ------------------------------------------------------------------
    // Reads
    // ------------------------------------------------------------------

    /// One artifact by id with derived freshness / verification / acceptance.
    pub fn get_artifact(
        &self,
        artifact_id: &str,
    ) -> Result<Option<AttemptArtifactRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            &format!("{ARTIFACT_SELECT} WHERE a.artifact_id = ?1"),
            [artifact_id],
            map_artifact_row,
        )
        .optional()
        .map_err(unavailable("get artifact"))
    }

    fn require_artifact(
        &self,
        artifact_id: &str,
    ) -> Result<AttemptArtifactRow, ProjectAggregateError> {
        self.get_artifact(artifact_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "artifact not found",
            })
    }

    /// Newest-first artifacts of one Project (the `outputs` read).
    pub fn list_artifacts(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<AttemptArtifactRow>, ProjectAggregateError> {
        let limit = limit.clamp(1, 128);
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "{ARTIFACT_SELECT} WHERE a.project_id = ?1
                 ORDER BY a.created_at DESC, a.rowid DESC LIMIT ?2"
            ))
            .map_err(unavailable("prepare artifacts"))?;
        let rows = statement
            .query_map(params![project_id, limit], map_artifact_row)
            .map_err(unavailable("query artifacts"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect artifacts"))
    }

    /// Newest-first evidence history of one artifact.
    pub fn list_evidence(
        &self,
        artifact_id: &str,
    ) -> Result<Vec<ArtifactEvidenceRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "{EVIDENCE_SELECT} WHERE artifact_id = ?1 ORDER BY verified_at DESC, rowid DESC"
            ))
            .map_err(unavailable("prepare evidence"))?;
        let rows = statement
            .query_map([artifact_id], map_evidence_row)
            .map_err(unavailable("query evidence"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect evidence"))
    }

    /// One evidence row by id.
    pub fn get_evidence(
        &self,
        evidence_id: &str,
    ) -> Result<Option<ArtifactEvidenceRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            &format!("{EVIDENCE_SELECT} WHERE evidence_id = ?1"),
            [evidence_id],
            map_evidence_row,
        )
        .optional()
        .map_err(unavailable("get evidence"))
    }

    /// Run acceptance facts of one Project, newest first.
    pub fn list_run_acceptances(
        &self,
        project_id: &str,
    ) -> Result<Vec<RunAcceptanceRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT acceptance_id, project_id, plan_revision_id, stage_id, stage_position,
                        stage_count, stage_test_fact_id, artifact_id, evidence_id,
                        acceptance_decision_ref, accepted_at
                   FROM p13_run_acceptance WHERE project_id = ?1
                  ORDER BY accepted_at DESC, rowid DESC",
            )
            .map_err(unavailable("prepare acceptances"))?;
        let rows = statement
            .query_map([project_id], |row| {
                Ok(RunAcceptanceRow {
                    acceptance_id: row.get(0)?,
                    project_id: row.get(1)?,
                    plan_revision_id: row.get(2)?,
                    stage_id: row.get(3)?,
                    stage_position: row.get(4)?,
                    stage_count: row.get(5)?,
                    stage_test_fact_id: row.get(6)?,
                    artifact_id: row.get(7)?,
                    evidence_id: row.get(8)?,
                    acceptance_decision_ref: row.get(9)?,
                    accepted_at: row.get(10)?,
                })
            })
            .map_err(unavailable("query acceptances"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect acceptances"))
    }

    /// External send Intents of one Project, newest first.
    pub fn list_external_sends(
        &self,
        project_id: &str,
    ) -> Result<Vec<ExternalSendRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "{SEND_SELECT} WHERE s.project_id = ?1 ORDER BY s.created_at DESC, s.rowid DESC"
            ))
            .map_err(unavailable("prepare sends"))?;
        let rows = statement
            .query_map([project_id], map_send_row)
            .map_err(unavailable("query sends"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect sends"))
    }

    /// Redaction guard for tests and responses: does any durable v37 cell
    /// contain the needle?
    pub fn leak_scan_contains(&self, needle: &str) -> Result<bool, ProjectAggregateError> {
        let conn = self.lock()?;
        for sql in [
            "SELECT COUNT(*) FROM p13_attempt_artifact WHERE task_ref LIKE ?1 OR cas_ref LIKE ?1",
            "SELECT COUNT(*) FROM p13_artifact_evidence WHERE criteria_json LIKE ?1",
            "SELECT COUNT(*) FROM p13_run_acceptance WHERE decision_json LIKE ?1",
        ] {
            let count: i64 = conn
                .query_row(sql, [format!("%{needle}%")], |row| row.get(0))
                .map_err(unavailable("leak scan"))?;
            if count > 0 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ------------------------------------------------------------------
    // Independent verifier
    // ------------------------------------------------------------------

    /// Run the independent verifier over one artifact and append evidence.
    ///
    /// Deterministic checks only. The Attempt's `response_status` / exit code
    /// are recorded as `not-used`. The report bytes are put into the same CAS
    /// and referenced from the evidence row. A CAS read failure is a `failed`
    /// disposition, not an infrastructure error, because the bytes on disk
    /// no longer match what was observed.
    pub fn verify_artifact(
        &self,
        cas: &ArtifactStore,
        artifact_id: &str,
        now_ms: i64,
    ) -> Result<ArtifactEvidenceRow, ProjectAggregateError> {
        let artifact = self.require_artifact(artifact_id)?;
        let attempt = self.require_attempt(&artifact.attempt_id)?;
        let mut criteria: Vec<Value> = Vec::new();
        let mut failed = false;
        let mut indeterminate = false;
        let mut push = |id: &str, result: &str, detail: String| {
            criteria.push(json!({"id": id, "result": result, "detail": detail}));
        };

        if attempt.state == "terminal" {
            push(
                "attempt-terminal-observed",
                "pass",
                format!(
                    "terminal_kind={} exit_code={:?}",
                    attempt.terminal_kind, attempt.exit_code
                ),
            );
        } else {
            indeterminate = true;
            push(
                "attempt-terminal-observed",
                "fail",
                format!("attempt state is {}", attempt.state),
            );
        }
        push(
            "attempt-response-status",
            "not-used",
            format!(
                "child response_status={} completion_claimed={} are observations, never inputs",
                attempt.response_status, attempt.completion_claimed
            ),
        );

        match self.candidate_frame(&artifact.attempt_id, artifact.source_frame_seq) {
            Ok(frame)
                if frame.payload_digest.as_deref()
                    == Some(artifact.source_payload_digest.as_str()) =>
            {
                push(
                    "source-frame-bound",
                    "pass",
                    format!("frame seq {} payload digest matches", frame.seq),
                );
            }
            Ok(_) => {
                failed = true;
                push(
                    "source-frame-bound",
                    "fail",
                    "ledger frame payload digest differs from the artifact".to_owned(),
                );
            }
            Err(error) => {
                failed = true;
                push("source-frame-bound", "fail", error.to_string());
            }
        }

        let source_ref = format!("sha256:{}", artifact.source_payload_digest);
        let mut deliverable_from_source: Option<String> = None;
        match cas.get(&source_ref) {
            Ok(Some(bytes)) => match std::str::from_utf8(&bytes)
                .ok()
                .and_then(|text| deliverable_text(text).ok())
            {
                Some(text) => {
                    push(
                        "source-payload-cas",
                        "pass",
                        "candidate payload re-read from CAS".to_owned(),
                    );
                    deliverable_from_source = Some(text);
                }
                None => {
                    failed = true;
                    push(
                        "source-payload-cas",
                        "fail",
                        "candidate payload in CAS does not parse to a deliverable".to_owned(),
                    );
                }
            },
            Ok(None) => {
                failed = true;
                push(
                    "source-payload-cas",
                    "fail",
                    "candidate payload missing from CAS".to_owned(),
                );
            }
            Err(ArtifactStoreError::DigestMismatch { .. }) => {
                failed = true;
                push(
                    "source-payload-cas",
                    "fail",
                    "candidate payload digest mismatch".to_owned(),
                );
            }
            Err(error) => return Err(cas_error("read source payload")(error)),
        }

        let mut checked_cas_ref = artifact.cas_ref.clone();
        match cas.get(&artifact.cas_ref) {
            Ok(Some(bytes)) => {
                let computed = format!("sha256:{}", Self::digest_hex(&bytes));
                checked_cas_ref = computed.clone();
                let text = String::from_utf8_lossy(&bytes).to_string();
                let matches_source = deliverable_from_source
                    .as_deref()
                    .is_some_and(|source| source == text);
                if computed == artifact.cas_ref && matches_source {
                    push(
                        "cas-bytes-match-digest",
                        "pass",
                        format!("{} bytes re-hash to the artifact digest", bytes.len()),
                    );
                } else {
                    failed = true;
                    push(
                        "cas-bytes-match-digest",
                        "fail",
                        "CAS bytes do not equal the observed deliverable".to_owned(),
                    );
                }
                if text.trim().is_empty() {
                    failed = true;
                    push(
                        "deliverable-non-empty",
                        "fail",
                        "deliverable is empty".to_owned(),
                    );
                } else {
                    push(
                        "deliverable-non-empty",
                        "pass",
                        format!("{} chars", text.chars().count()),
                    );
                }
                if deliverable_has_secret_shape(&text) {
                    failed = true;
                    push(
                        "no-secret-shape",
                        "fail",
                        "secret-shaped token in deliverable".to_owned(),
                    );
                } else {
                    push(
                        "no-secret-shape",
                        "pass",
                        "no secret-shaped token".to_owned(),
                    );
                }
                if std::str::from_utf8(&bytes).is_ok() {
                    push(
                        "format-parses",
                        "pass",
                        format!("{} is valid UTF-8", artifact.format),
                    );
                } else {
                    failed = true;
                    push(
                        "format-parses",
                        "fail",
                        "deliverable is not valid UTF-8".to_owned(),
                    );
                }
            }
            Ok(None) => {
                failed = true;
                push(
                    "cas-bytes-match-digest",
                    "fail",
                    "deliverable bytes missing from CAS".to_owned(),
                );
            }
            Err(ArtifactStoreError::DigestMismatch { computed, .. }) => {
                failed = true;
                checked_cas_ref = computed;
                push(
                    "cas-bytes-match-digest",
                    "fail",
                    "deliverable bytes on disk no longer hash to the artifact digest".to_owned(),
                );
            }
            Err(error) => return Err(cas_error("read deliverable")(error)),
        }

        let disposition = if failed {
            "failed"
        } else if indeterminate {
            "indeterminate"
        } else {
            "passed"
        };
        let evidence_id = next_id("evidence");
        let criteria_json =
            serde_json::to_string(&criteria).map_err(|_| ProjectAggregateError::Unavailable {
                detail: "serialize criteria".to_owned(),
            })?;
        let report = json!({
            "schema": "cognitiveos.personal.attempt-artifact-evidence/0.1",
            "evidence_id": evidence_id,
            "artifact_id": artifact.artifact_id,
            "attempt_id": artifact.attempt_id,
            "project_id": artifact.project_id,
            "task_ref": artifact.task_ref,
            "verifier_ref": ATTEMPT_ARTIFACT_VERIFIER_REF,
            "verifier_version": ATTEMPT_ARTIFACT_VERIFIER_VERSION,
            "principal": ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL,
            "disposition": disposition,
            "criteria": criteria,
            "artifact_cas_ref": artifact.cas_ref,
            "checked_cas_ref": checked_cas_ref,
            "verified_at": now_ms,
        });
        let report_bytes = serde_json_canonicalizer::to_string(&report).map_err(|_| {
            ProjectAggregateError::Unavailable {
                detail: "canonicalize evidence report".to_owned(),
            }
        })?;
        let report_cas_ref = format!("sha256:{}", Self::digest_hex(report_bytes.as_bytes()));
        cas.put_with_metadata(
            &report_cas_ref,
            report_bytes.as_bytes(),
            "application/vnd.cognitiveos.attempt-artifact-evidence+json",
        )
        .map_err(cas_error("put evidence report"))?;
        {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p13_artifact_evidence (
                    evidence_id, artifact_id, verifier_ref, verifier_version, principal,
                    disposition, criteria_json, report_cas_ref, checked_cas_ref, verified_at
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    evidence_id,
                    artifact.artifact_id,
                    ATTEMPT_ARTIFACT_VERIFIER_REF,
                    ATTEMPT_ARTIFACT_VERIFIER_VERSION,
                    ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL,
                    disposition,
                    criteria_json,
                    report_cas_ref,
                    checked_cas_ref,
                    now_ms
                ],
            )
            .map_err(unavailable("insert evidence"))?;
        }
        self.get_evidence(&evidence_id)?
            .ok_or(ProjectAggregateError::Unavailable {
                detail: "evidence row vanished after insert".to_owned(),
            })
    }

    // ------------------------------------------------------------------
    // StageTestPassed derived from evidence
    // ------------------------------------------------------------------

    /// Derive StageTestPassed for `stage_id` from durable facts about this
    /// artifact: real seating, the artifact's freshness, the latest evidence,
    /// a CAS re-read, and the Attempt's terminal state. No caller boolean is
    /// trusted. Owner management only.
    #[allow(clippy::too_many_arguments)]
    pub fn derive_stage_test(
        &self,
        caller: ConfirmCaller,
        projects: &ProjectAggregateStore,
        employees: &EmployeeStore,
        cas: &ArtifactStore,
        artifact_id: &str,
        stage_id: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let artifact = self.require_artifact(artifact_id)?;
        let attempt = self.require_attempt(&artifact.attempt_id)?;
        let project =
            projects
                .get_project(&artifact.project_id)?
                .ok_or(ProjectAggregateError::NotFound {
                    detail: "project not found",
                })?;
        let plan_id = project
            .current_plan_revision_id
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project has no current plan revision",
            })?;
        let stage =
            projects
                .get_stage(&plan_id, stage_id)?
                .ok_or(ProjectAggregateError::NotFound {
                    detail: "stage not on current plan revision",
                })?;
        if !self.employee_holds_slot(
            &artifact.project_id,
            &plan_id,
            &stage.responsible_slot,
            &artifact.employee_id,
        )? {
            return Err(ProjectAggregateError::Rejected {
                detail: "the Attempt's Member is not responsible for this stage",
            });
        }
        if artifact.freshness != "current" {
            return Err(ProjectAggregateError::Rejected {
                detail: "superseded artifact cannot back a stage test",
            });
        }
        let evidence = match artifact.latest_evidence_id.as_deref() {
            Some(evidence_id) => self.get_evidence(evidence_id)?,
            None => None,
        };
        let Some(evidence) = evidence else {
            return Err(ProjectAggregateError::Rejected {
                detail: "completion requires current independent verification evidence",
            });
        };
        let seated = employees.stage_is_seated(&artifact.project_id, &plan_id, stage_id)?;
        let openable = matches!(cas.get(&artifact.cas_ref), Ok(Some(_)));
        let oracle = StageTestOracle {
            project_id: artifact.project_id.clone(),
            plan_revision_id: plan_id,
            stage_id: stage_id.to_owned(),
            task_ref: artifact.task_ref.clone(),
            seating: SeatingFacts { seated },
            verification_current: evidence.checked_cas_ref == artifact.cas_ref,
            verification_report_ref: evidence.evidence_id.clone(),
            openable,
            checks_passed: evidence.disposition == "passed",
            effects_closed: attempt.state == "terminal",
            now_ms,
        };
        projects.derive_stage_test_passed(&oracle)
    }

    fn employee_holds_slot(
        &self,
        project_id: &str,
        plan_revision_id: &str,
        slot: &str,
        employee_id: &str,
    ) -> Result<bool, ProjectAggregateError> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_assignment
                  WHERE project_id = ?1 AND plan_revision_id = ?2 AND slot = ?3 AND employee_id = ?4",
                params![project_id, plan_revision_id, slot, employee_id],
                |row| row.get(0),
            )
            .map_err(unavailable("assignment lookup"))?;
        Ok(count > 0)
    }

    // ------------------------------------------------------------------
    // Run acceptance (last ring only) through ApprovalPreview
    // ------------------------------------------------------------------

    /// Mint a `run-acceptance` ApprovalPreview for the last ring of the
    /// current plan. Refused off the last ring, without a current
    /// StageTestPassed backed by passed evidence, or after that fact was
    /// already accepted. Confirm goes through `ProjectAggregateStore::confirm_preview`.
    pub fn request_run_acceptance(
        &self,
        caller: ConfirmCaller,
        projects: &ProjectAggregateStore,
        project_id: &str,
        stage_id: &str,
        now_ms: i64,
    ) -> Result<(String, String), ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let binding = {
            let conn = self.lock()?;
            let last_ring = last_ring_locked(&conn, project_id)?;
            if last_ring.stage_id != stage_id {
                return Err(ProjectAggregateError::Rejected {
                    detail: "run acceptance is only offered on the last ring",
                });
            }
            last_ring_binding_locked(&conn, project_id)?
        };
        let preview_bytes = json!({
            "subject": RUN_ACCEPTANCE_SUBJECT_KIND,
            "project_id": project_id,
            "plan_revision_id": binding.plan_revision_id,
            "stage_id": binding.stage_id,
            "stage_position": binding.stage_position,
            "stage_count": binding.stage_count,
            "stage_test_fact_id": binding.fact_id,
            "artifact_id": binding.artifact_id,
            "evidence_id": binding.evidence_id,
            "requested_at": now_ms,
        })
        .to_string();
        projects.request_preview(
            RUN_ACCEPTANCE_SUBJECT_KIND,
            project_id,
            preview_bytes.as_bytes(),
            now_ms,
        )
    }

    // ------------------------------------------------------------------
    // Publication packet + external send through ApprovalPreview
    // ------------------------------------------------------------------

    /// The full AUTONOMY packet for one verified artifact: planned, not
    /// published; chat cannot confirm; no connector is qualified. Read-only.
    pub fn publication_packet(
        &self,
        projects: &ProjectAggregateStore,
        project_id: &str,
        artifact_id: &str,
        now_ms: i64,
    ) -> Result<Value, ProjectAggregateError> {
        let artifact = self.require_artifact(artifact_id)?;
        if artifact.project_id != project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project read rejected",
            });
        }
        let evidence = match artifact.latest_evidence_id.as_deref() {
            Some(id) => self.get_evidence(id)?,
            None => None,
        };
        let acceptance = self
            .list_run_acceptances(project_id)?
            .into_iter()
            .find(|row| row.artifact_id == artifact.artifact_id);
        let project = projects.get_project(project_id)?;
        let verified = evidence
            .as_ref()
            .is_some_and(|row| row.disposition == "passed");
        Ok(json!({
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "packet": "publication",
            "planned": true,
            "published": false,
            "chat_can_confirm": false,
            "connector": EXTERNAL_SEND_CONNECTOR_NONE,
            "project_id": project_id,
            "project_state": project.map(|row| row.state),
            "artifact": {
                "artifact_id": artifact.artifact_id,
                "cas_ref": artifact.cas_ref,
                "format": artifact.format,
                "byte_length": artifact.byte_length,
                "source": artifact.source,
                "freshness": artifact.freshness,
                "produced_at": artifact.produced_at,
            },
            "autonomy_packet": {
                "preview": {
                    "what_will_happen": "send the verified deliverable to the selected recipients; nothing is sent by this preview",
                    "diff": "first send of this artifact digest; no earlier send exists",
                    "artifact_cas_ref": artifact.cas_ref,
                },
                "override": {
                    "owner_controls": ["confirm", "narrow", "reject"],
                    "surface": "project canvas ApprovalPreview (P11-T09); chat has no Confirm",
                },
                "tiered_authority": {
                    "external_send_requires": "owner confirm of a digest-bound preview",
                    "assistant_can": "propose only",
                    "member_can": "produce candidates only",
                },
                "observable": {
                    "receipt": "persist-before-dispatch Intent row `p13_external_send`; state planned",
                    "ledger_read": "GET /management/project/v1/publication.sends",
                },
                "outcome_verify": {
                    "verification_status": artifact.verification_status,
                    "evidence_id": evidence.as_ref().map(|row| row.evidence_id.clone()),
                    "verifier_ref": ATTEMPT_ARTIFACT_VERIFIER_REF,
                    "verified": verified,
                    "run_acceptance_id": acceptance.as_ref().map(|row| row.acceptance_id.clone()),
                    "accepted": acceptance.is_some(),
                },
                "memory_of_actions": {
                    "attempt_id": artifact.attempt_id,
                    "source_frame_seq": artifact.source_frame_seq,
                    "evidence_report_cas_ref": evidence.as_ref().map(|row| row.report_cas_ref.clone()),
                },
                "yield": {
                    "stoppable_until": "dispatch (none exists yet)",
                    "unknown_outcome_policy": "no blind retry; unknown is not success",
                },
            },
            "generated_at": now_ms,
        }))
    }

    /// Mint an `external-send` ApprovalPreview for one verified artifact and
    /// record the send Intent as `previewed`. Owner management only; the
    /// artifact must carry passed evidence. Recipients are hashed, not stored.
    pub fn request_external_send(
        &self,
        caller: ConfirmCaller,
        projects: &ProjectAggregateStore,
        spec: &ExternalSendSpec<'_>,
    ) -> Result<ExternalSendRow, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if spec.recipients.is_empty() || spec.recipients.iter().any(|r| r.trim().is_empty()) {
            return Err(ProjectAggregateError::Invalid {
                detail: "external send requires at least one non-empty recipient",
            });
        }
        if spec
            .recipients
            .iter()
            .any(|r| deliverable_has_secret_shape(r))
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret-shaped recipient rejected",
            });
        }
        let artifact = self.require_artifact(spec.artifact_id)?;
        if artifact.project_id != spec.project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        let evidence = match artifact.latest_evidence_id.as_deref() {
            Some(id) => self.get_evidence(id)?,
            None => None,
        };
        let Some(evidence) = evidence.filter(|row| row.disposition == "passed") else {
            return Err(ProjectAggregateError::Rejected {
                detail: "external send requires passed independent verification evidence",
            });
        };
        let acceptance_id = self
            .list_run_acceptances(spec.project_id)?
            .into_iter()
            .find(|row| row.artifact_id == artifact.artifact_id)
            .map(|row| row.acceptance_id);
        let packet =
            self.publication_packet(projects, spec.project_id, spec.artifact_id, spec.now_ms)?;
        let packet_canonical = serde_json_canonicalizer::to_string(&packet).map_err(|_| {
            ProjectAggregateError::Unavailable {
                detail: "canonicalize packet".to_owned(),
            }
        })?;
        let packet_digest = Self::digest_hex(packet_canonical.as_bytes());
        let mut recipients: Vec<&str> = spec.recipients.iter().map(String::as_str).collect();
        recipients.sort_unstable();
        let recipients_digest = Self::digest_hex(recipients.join("\n").as_bytes());
        let send_id = next_id("send");
        {
            let conn = self.lock()?;
            conn.execute(
                "UPDATE p13_external_send SET state = 'superseded'
                  WHERE project_id = ?1 AND state = 'previewed'",
                [spec.project_id],
            )
            .map_err(unavailable("supersede previewed sends"))?;
            conn.execute(
                "INSERT INTO p13_external_send (
                    send_id, project_id, artifact_id, evidence_id, acceptance_id, preview_id,
                    packet_digest, recipient_count, recipients_digest, state, published,
                    connector, intent_persisted, receipt_ref, created_at, planned_at
                 ) VALUES (?1,?2,?3,?4,?5,'',?6,?7,?8,'previewed',0,?9,1,NULL,?10,NULL)",
                params![
                    send_id,
                    spec.project_id,
                    artifact.artifact_id,
                    evidence.evidence_id,
                    acceptance_id,
                    packet_digest,
                    i64::try_from(spec.recipients.len()).unwrap_or(i64::MAX),
                    recipients_digest,
                    EXTERNAL_SEND_CONNECTOR_NONE,
                    spec.now_ms
                ],
            )
            .map_err(unavailable("insert external send"))?;
        }
        let preview_bytes = json!({
            "subject": EXTERNAL_SEND_SUBJECT_KIND,
            "send_id": send_id,
            "project_id": spec.project_id,
            "artifact_id": artifact.artifact_id,
            "evidence_id": evidence.evidence_id,
            "packet_digest": packet_digest,
            "recipient_count": spec.recipients.len(),
            "recipients_digest": recipients_digest,
            "planned": true,
            "published": false,
        })
        .to_string();
        let (preview_id, preview_digest) = match projects.request_preview(
            EXTERNAL_SEND_SUBJECT_KIND,
            spec.project_id,
            preview_bytes.as_bytes(),
            spec.now_ms,
        ) {
            Ok(minted) => minted,
            Err(error) => {
                let conn = self.lock()?;
                let _ = conn.execute(
                    "UPDATE p13_external_send SET state = 'superseded' WHERE send_id = ?1",
                    [&send_id],
                );
                return Err(error);
            }
        };
        {
            let conn = self.lock()?;
            conn.execute(
                "UPDATE p13_external_send SET preview_id = ?1 WHERE send_id = ?2",
                params![preview_id, send_id],
            )
            .map_err(unavailable("bind send preview"))?;
        }
        let mut row = self.require_send(&send_id)?;
        row.preview_digest = preview_digest;
        Ok(row)
    }

    fn require_send(&self, send_id: &str) -> Result<ExternalSendRow, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            &format!("{SEND_SELECT} WHERE s.send_id = ?1"),
            [send_id],
            map_send_row,
        )
        .optional()
        .map_err(unavailable("get send"))?
        .ok_or(ProjectAggregateError::NotFound {
            detail: "external send not found",
        })
    }
}

// ----------------------------------------------------------------------
// Locked helpers used by `ProjectAggregateStore` for the two new subject kinds
// ----------------------------------------------------------------------

struct LastRingBinding {
    plan_revision_id: String,
    stage_id: String,
    stage_position: i64,
    stage_count: i64,
    fact_id: String,
    artifact_id: String,
    evidence_id: String,
}

struct LastRing {
    plan_revision_id: String,
    stage_id: String,
    stage_position: i64,
    stage_count: i64,
}

/// The last ring (highest position) of the current plan revision.
fn last_ring_locked(
    conn: &Connection,
    project_id: &str,
) -> Result<LastRing, ProjectAggregateError> {
    let plan_id: Option<String> = conn
        .query_row(
            "SELECT current_plan_revision_id FROM p11_project WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("project for run acceptance"))?
        .ok_or(ProjectAggregateError::NotFound {
            detail: "project not found",
        })?;
    let Some(plan_id) = plan_id else {
        return Err(ProjectAggregateError::NotFound {
            detail: "project has no current plan revision",
        });
    };
    let (stage_id, stage_position, stage_count): (String, i64, i64) = conn
        .query_row(
            "SELECT stage_id, position,
                    (SELECT COUNT(*) FROM p11_stage c WHERE c.plan_revision_id = s.plan_revision_id)
               FROM p11_stage s WHERE s.plan_revision_id = ?1
              ORDER BY position DESC LIMIT 1",
            [&plan_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(unavailable("last ring"))?
        .ok_or(ProjectAggregateError::NotFound {
            detail: "plan has no stages",
        })?;
    Ok(LastRing {
        plan_revision_id: plan_id,
        stage_id,
        stage_position,
        stage_count,
    })
}

/// The last ring of the current plan and its current StageTestPassed fact,
/// resolved back through the evidence row to the artifact.
fn last_ring_binding_locked(
    conn: &Connection,
    project_id: &str,
) -> Result<LastRingBinding, ProjectAggregateError> {
    let LastRing {
        plan_revision_id: plan_id,
        stage_id,
        stage_position,
        stage_count,
    } = last_ring_locked(conn, project_id)?;
    let fact: Option<(String, String)> = conn
        .query_row(
            "SELECT fact_id, verification_report_ref FROM p11_stage_test_fact
              WHERE plan_revision_id = ?1 AND stage_id = ?2 AND current = 1",
            params![plan_id, stage_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(unavailable("current stage fact"))?;
    let Some((fact_id, evidence_id)) = fact else {
        return Err(ProjectAggregateError::Rejected {
            detail: "last ring has no current StageTestPassed",
        });
    };
    let evidence: Option<(String, String)> = conn
        .query_row(
            "SELECT e.artifact_id, e.disposition FROM p13_artifact_evidence e
              WHERE e.evidence_id = ?1",
            [&evidence_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(unavailable("fact evidence"))?;
    let Some((artifact_id, disposition)) = evidence else {
        return Err(ProjectAggregateError::Rejected {
            detail: "StageTestPassed is not backed by independent verifier evidence",
        });
    };
    if disposition != "passed" {
        return Err(ProjectAggregateError::Rejected {
            detail: "StageTestPassed evidence is not passed",
        });
    }
    let artifact_project: Option<String> = conn
        .query_row(
            "SELECT project_id FROM p13_attempt_artifact WHERE artifact_id = ?1",
            [&artifact_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("fact artifact"))?;
    if artifact_project.as_deref() != Some(project_id) {
        return Err(ProjectAggregateError::Forbidden {
            detail: "cross-project acceptance rejected",
        });
    }
    let already: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM p13_run_acceptance WHERE stage_test_fact_id = ?1",
            [&fact_id],
            |row| row.get(0),
        )
        .map_err(unavailable("existing acceptance"))?;
    if already > 0 {
        return Err(ProjectAggregateError::Conflict {
            detail: "this StageTestPassed fact was already accepted",
        });
    }
    Ok(LastRingBinding {
        plan_revision_id: plan_id,
        stage_id,
        stage_position,
        stage_count,
        fact_id,
        artifact_id,
        evidence_id,
    })
}

/// Base-state digest for a `run-acceptance` preview: moves whenever the
/// current plan, last ring, or its current StageTestPassed fact changes.
pub(crate) fn run_acceptance_base_digest_locked(
    conn: &Connection,
    project_id: &str,
) -> Result<String, ProjectAggregateError> {
    let binding = last_ring_binding_locked(conn, project_id)?;
    Ok(format!(
        "{:x}",
        Sha256::digest(
            format!(
                "run-acceptance\n{project_id}\n{}\n{}\n{}\n{}\n{}",
                binding.plan_revision_id,
                binding.stage_id,
                binding.fact_id,
                binding.artifact_id,
                binding.evidence_id
            )
            .as_bytes()
        )
    ))
}

/// Confirm a `run-acceptance` preview: write the append-only acceptance fact
/// for the last ring. Everything is re-derived from durable state.
pub(crate) fn accept_run_locked(
    conn: &Connection,
    project_id: &str,
    now_ms: i64,
) -> Result<ConfirmResult, ProjectAggregateError> {
    let binding = last_ring_binding_locked(conn, project_id)?;
    let decision = json!({
        "schema_version": 1,
        "decision": "granted",
        "kind": RUN_ACCEPTANCE_SUBJECT_KIND,
        "project_id": project_id,
        "plan_revision_id": binding.plan_revision_id,
        "stage_id": binding.stage_id,
        "stage_position": binding.stage_position,
        "stage_count": binding.stage_count,
        "stage_test_fact_id": binding.fact_id,
        "artifact_id": binding.artifact_id,
        "evidence_id": binding.evidence_id,
        "accepted_at": now_ms,
    });
    let decision_json = serde_json_canonicalizer::to_string(&decision).map_err(|_| {
        ProjectAggregateError::Unavailable {
            detail: "canonicalize acceptance decision".to_owned(),
        }
    })?;
    let acceptance_decision_ref = format!("cas:{:x}", Sha256::digest(decision_json.as_bytes()));
    let acceptance_id = next_id("runaccept");
    conn.execute(
        "INSERT INTO p13_run_acceptance (
            acceptance_id, project_id, plan_revision_id, stage_id, stage_position, stage_count,
            stage_test_fact_id, artifact_id, evidence_id, acceptance_decision_ref, decision_json,
            accepted_at
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            acceptance_id,
            project_id,
            binding.plan_revision_id,
            binding.stage_id,
            binding.stage_position,
            binding.stage_count,
            binding.fact_id,
            binding.artifact_id,
            binding.evidence_id,
            acceptance_decision_ref,
            decision_json,
            now_ms
        ],
    )
    .map_err(unavailable("insert run acceptance"))?;
    Ok(ConfirmResult {
        kind: "run_accepted",
        new_ref: acceptance_id,
        receipt_ref: acceptance_decision_ref,
    })
}

struct PreviewedSend {
    send_id: String,
    artifact_id: String,
    evidence_id: String,
    packet_digest: String,
    recipients_digest: String,
}

fn previewed_send_locked(
    conn: &Connection,
    project_id: &str,
) -> Result<PreviewedSend, ProjectAggregateError> {
    conn.query_row(
        "SELECT send_id, artifact_id, evidence_id, packet_digest, recipients_digest
           FROM p13_external_send
          WHERE project_id = ?1 AND state = 'previewed'
          ORDER BY created_at DESC, rowid DESC LIMIT 1",
        [project_id],
        |row| {
            Ok(PreviewedSend {
                send_id: row.get(0)?,
                artifact_id: row.get(1)?,
                evidence_id: row.get(2)?,
                packet_digest: row.get(3)?,
                recipients_digest: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(unavailable("previewed send"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "no previewed external send for this project",
    })
}

/// Base-state digest for an `external-send` preview: bound to the previewed
/// send Intent, its artifact, evidence, packet and recipients.
pub(crate) fn external_send_base_digest_locked(
    conn: &Connection,
    project_id: &str,
) -> Result<String, ProjectAggregateError> {
    let send = previewed_send_locked(conn, project_id)?;
    Ok(format!(
        "{:x}",
        Sha256::digest(
            format!(
                "external-send\n{project_id}\n{}\n{}\n{}\n{}\n{}",
                send.send_id,
                send.artifact_id,
                send.evidence_id,
                send.packet_digest,
                send.recipients_digest
            )
            .as_bytes()
        )
    ))
}

/// Confirm an `external-send` preview: the Intent becomes `planned`. Nothing
/// is dispatched — no connector is qualified — and `published` stays 0.
pub(crate) fn external_send_locked(
    conn: &Connection,
    project_id: &str,
    now_ms: i64,
) -> Result<ConfirmResult, ProjectAggregateError> {
    let send = previewed_send_locked(conn, project_id)?;
    let receipt_ref = format!("receipt:external-send:{}", send.send_id);
    conn.execute(
        "UPDATE p13_external_send
            SET state = 'planned', planned_at = ?1, receipt_ref = ?2
          WHERE send_id = ?3 AND state = 'previewed'",
        params![now_ms, receipt_ref, send.send_id],
    )
    .map_err(unavailable("plan external send"))?;
    Ok(ConfirmResult {
        kind: "external_send_planned",
        new_ref: send.send_id,
        receipt_ref,
    })
}

// ----------------------------------------------------------------------
// Row mapping
// ----------------------------------------------------------------------

const ARTIFACT_SELECT: &str = "
SELECT a.artifact_id, a.attempt_id, a.project_id, a.task_ref, a.employee_id, a.cas_ref,
       a.byte_length, a.format, a.source, a.source_frame_seq, a.source_payload_digest,
       a.context_digest, a.produced_at, a.created_at,
       (SELECT e.evidence_id FROM p13_artifact_evidence e
         WHERE e.artifact_id = a.artifact_id
         ORDER BY e.verified_at DESC, e.rowid DESC LIMIT 1) AS latest_evidence_id,
       (SELECT e.disposition FROM p13_artifact_evidence e
         WHERE e.artifact_id = a.artifact_id
         ORDER BY e.verified_at DESC, e.rowid DESC LIMIT 1) AS latest_disposition,
       (SELECT COUNT(*) FROM p13_attempt_artifact n
         WHERE n.project_id = a.project_id AND n.task_ref = a.task_ref
           AND (n.created_at > a.created_at
                OR (n.created_at = a.created_at AND n.rowid > a.rowid))) AS newer_count,
       (SELECT f.stage_id FROM p11_stage_test_fact f
          JOIN p13_artifact_evidence e2 ON e2.evidence_id = f.verification_report_ref
         WHERE e2.artifact_id = a.artifact_id AND f.current = 1
         ORDER BY f.passed_at DESC LIMIT 1) AS stage_id,
       (SELECT r.accepted_at FROM p13_run_acceptance r
         WHERE r.artifact_id = a.artifact_id
         ORDER BY r.accepted_at DESC LIMIT 1) AS accepted_at
  FROM p13_attempt_artifact a";

fn map_artifact_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttemptArtifactRow> {
    let latest_disposition: Option<String> = row.get(15)?;
    let newer_count: i64 = row.get(16)?;
    Ok(AttemptArtifactRow {
        artifact_id: row.get(0)?,
        attempt_id: row.get(1)?,
        project_id: row.get(2)?,
        task_ref: row.get(3)?,
        employee_id: row.get(4)?,
        cas_ref: row.get(5)?,
        byte_length: row.get(6)?,
        format: row.get(7)?,
        source: row.get(8)?,
        source_frame_seq: row.get(9)?,
        source_payload_digest: row.get(10)?,
        context_digest: row.get(11)?,
        produced_at: row.get(12)?,
        created_at: row.get(13)?,
        latest_evidence_id: row.get(14)?,
        verification_status: latest_disposition.unwrap_or_else(|| "not-run".to_owned()),
        freshness: if newer_count == 0 {
            "current".to_owned()
        } else {
            "superseded".to_owned()
        },
        stage_id: row.get(17)?,
        accepted_at: row.get(18)?,
    })
}

const EVIDENCE_SELECT: &str =
    "SELECT evidence_id, artifact_id, verifier_ref, verifier_version, principal, disposition,
            criteria_json, report_cas_ref, checked_cas_ref, verified_at
       FROM p13_artifact_evidence";

fn map_evidence_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArtifactEvidenceRow> {
    Ok(ArtifactEvidenceRow {
        evidence_id: row.get(0)?,
        artifact_id: row.get(1)?,
        verifier_ref: row.get(2)?,
        verifier_version: row.get(3)?,
        principal: row.get(4)?,
        disposition: row.get(5)?,
        criteria_json: row.get(6)?,
        report_cas_ref: row.get(7)?,
        checked_cas_ref: row.get(8)?,
        verified_at: row.get(9)?,
    })
}

const SEND_SELECT: &str =
    "SELECT s.send_id, s.project_id, s.artifact_id, s.evidence_id, s.acceptance_id, s.preview_id,
            COALESCE((SELECT p.preview_digest FROM p11_approval_preview p
                       WHERE p.preview_id = s.preview_id), ''),
            s.packet_digest, s.recipient_count, s.state, s.published, s.connector,
            s.intent_persisted, s.receipt_ref, s.created_at, s.planned_at
       FROM p13_external_send s";

fn map_send_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExternalSendRow> {
    let published: i64 = row.get(10)?;
    let intent_persisted: i64 = row.get(12)?;
    Ok(ExternalSendRow {
        send_id: row.get(0)?,
        project_id: row.get(1)?,
        artifact_id: row.get(2)?,
        evidence_id: row.get(3)?,
        acceptance_id: row.get(4)?,
        preview_id: row.get(5)?,
        preview_digest: row.get(6)?,
        packet_digest: row.get(7)?,
        recipient_count: row.get(8)?,
        state: row.get(9)?,
        published: published != 0,
        connector: row.get(11)?,
        intent_persisted: intent_persisted != 0,
        receipt_ref: row.get(13)?,
        created_at: row.get(14)?,
        planned_at: row.get(15)?,
    })
}

// ----------------------------------------------------------------------
// Payload helpers
// ----------------------------------------------------------------------

/// Extract the deliverable text from a canonical `DeliverableDraft` payload.
/// Empty text and secret-shaped text are refused before anything durable.
fn deliverable_text(payload_canonical: &str) -> Result<String, ProjectAggregateError> {
    let value: Value =
        serde_json::from_str(payload_canonical).map_err(|_| ProjectAggregateError::Rejected {
            detail: "candidate payload is not JSON",
        })?;
    let text = value
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(ProjectAggregateError::Rejected {
            detail: "candidate has no deliverable text",
        })?;
    if text.trim().is_empty() {
        return Err(ProjectAggregateError::Rejected {
            detail: "candidate has no deliverable text",
        });
    }
    if deliverable_has_secret_shape(&text) {
        return Err(ProjectAggregateError::Invalid {
            detail: "secret-shaped material must not enter an artifact",
        });
    }
    Ok(text)
}

/// Secret-shaped token detection with a token boundary for `sk-` so ordinary
/// hyphenated prose (`risk-based`, `desk-side`) is not a key.
pub fn deliverable_has_secret_shape(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    if lowered.contains("bearer ")
        || lowered.contains("ssv1:")
        || lowered.contains("secretref:")
        || lowered.contains("x-api-key")
        || lowered.contains("api_key=")
    {
        return true;
    }
    lowered.match_indices("sk-").any(|(index, _)| {
        lowered[..index]
            .chars()
            .next_back()
            .is_none_or(|previous| !previous.is_ascii_alphanumeric())
    })
}

fn next_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::now_v7().as_hyphenated())
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}

fn cas_error(operation: &'static str) -> impl Fn(ArtifactStoreError) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
