//! Hidden Pi Personal Assistant (P11-T06).
//!
//! Candidate-only engine: exact Pi `0.81.1` / `cognitiveos.private-candidate/1`
//! reused as identity pins, not a second scheduler or Installed Agent.
//! Conversation archive is read-only context. Writes to archive, SecretStore,
//! Memory, and authority confirm/apply are fail-closed.

use crate::conversation::{
    ArchiveAppendSpec, ArchiveReadSpec, CONVERSATION_ARCHIVE_PROJECTION_ID,
    CONVERSATION_RESUME_LIMIT, ConversationStore,
};
use crate::employee::EmployeeStore;
use crate::project_aggregate::{
    ConfirmCaller, ProjectAggregateError, ProjectAggregateStore, reject_closed_candidate_schema,
    validate_assistant_provenance,
};
use crate::sqlite::SqliteAuthorityStore;
use serde_json::{Value, json};

/// Product pin reused from the existing managed Pi path. Not a Windows OPC claim.
pub const ASSISTANT_PI_PIN: &str = "0.81.1";
/// Existing private-candidate protocol. Not a new carrier.
pub const ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL: &str = "cognitiveos.private-candidate/1";
/// Hidden engine identity. Pi is not an Installed Agent.
pub const ASSISTANT_ENGINE_ID: &str = "cognitiveos.personal.hidden-pi-assistant/0.1";
/// Research fetch reuses the audited read-only family. No parallel HTTP client.
pub const ASSISTANT_RESEARCH_FETCH_FAMILY: &str = "HttpFetchReadOnly";

const OBJECT_KINDS: &[&str] = &[
    "business-brief",
    "research-run",
    "charter",
    "axis",
    "roster",
    "recipe",
];
const TURN_KINDS: &[&str] = &["explain", "navigate", "research", "propose"];
const AMBIENT_TOOLS: &[&str] = &[
    "bash",
    "sh",
    "shell",
    "cmd",
    "powershell",
    "edit",
    "write",
    "apply_patch",
];

/// Clippy-safe turn input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistantTurnSpec<'a> {
    pub kind: &'a str,
    pub draft_id: &'a str,
    pub object_kind: &'a str,
    pub payload: &'a Value,
    pub provenance_json: &'a str,
    pub project_id: Option<&'a str>,
    pub tools: &'a [&'a str],
    pub now_ms: i64,
}

/// Candidate-only outcome. Preview id is an announcement; chat has no Approve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantTurnOutcome {
    pub candidate_id: String,
    pub candidate_digest: String,
    pub preview_id: Option<String>,
    pub object_kind: String,
    pub context_refs: Vec<String>,
    pub engine_id: String,
    pub pi_pin: String,
    pub protocol: String,
}

/// Hidden assistant plane over the daemon-owned writer.
#[derive(Clone)]
pub struct AssistantPlane {
    projects: ProjectAggregateStore,
    conversations: ConversationStore,
}

impl AssistantPlane {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            projects: ProjectAggregateStore::from_authority_store(store),
            conversations: ConversationStore::from_authority_store(store),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        Ok(Self {
            projects: ProjectAggregateStore::open_path(path)?,
            conversations: ConversationStore::open_path(path)?,
        })
    }

    /// Default-deny tools. Research may name `HttpFetchReadOnly` only.
    pub fn admit_tool(kind: &str, tool: &str) -> Result<(), ProjectAggregateError> {
        if AMBIENT_TOOLS.contains(&tool) {
            return Err(ProjectAggregateError::Forbidden {
                detail: "ambient tool/shell rejected",
            });
        }
        if kind == "research" && tool == ASSISTANT_RESEARCH_FETCH_FAMILY {
            return Ok(());
        }
        Err(ProjectAggregateError::Forbidden {
            detail: "default-deny tools: assistant has no ambient catalog",
        })
    }

    /// Explain / navigate / research / propose → candidate + optional preview.
    pub fn run_turn(
        &self,
        spec: &AssistantTurnSpec<'_>,
    ) -> Result<AssistantTurnOutcome, ProjectAggregateError> {
        if !TURN_KINDS.contains(&spec.kind) {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant turn kind must be explain, navigate, research, or propose",
            });
        }
        if !OBJECT_KINDS.contains(&spec.object_kind) {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant object_kind is closed: business-brief/research-run/charter/axis/roster/recipe",
            });
        }
        validate_assistant_provenance(Some(spec.provenance_json))?;
        for tool in spec.tools {
            Self::admit_tool(spec.kind, tool)?;
        }
        let context_refs = self.read_context_refs(spec.project_id)?;
        let ops = json!({
            "engine": ASSISTANT_ENGINE_ID,
            "pi_pin": ASSISTANT_PI_PIN,
            "protocol": ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL,
            "installed_agent": false,
            "turn": spec.kind,
            "object_kind": spec.object_kind,
            "payload": spec.payload,
            "context_refs": context_refs,
        });
        let ops_bytes = serde_json::to_vec(&ops).map_err(|_| ProjectAggregateError::Invalid {
            detail: "assistant candidate ops must be JSON",
        })?;
        reject_closed_candidate_schema(&ops_bytes)?;
        let base_seq = self.projects.get_draft_seq(spec.draft_id)?;
        let (candidate_id, candidate_digest) = self.projects.register_candidate(
            spec.draft_id,
            base_seq,
            &ops_bytes,
            "assistant",
            Some(spec.provenance_json),
        )?;
        let preview_id = if matches!(spec.kind, "research" | "propose") {
            let preview_bytes = format!("assistant-preview:{candidate_digest}").into_bytes();
            let (preview_id, _) = self.projects.request_preview(
                "activation",
                spec.draft_id,
                &preview_bytes,
                spec.now_ms,
            )?;
            Some(preview_id)
        } else {
            None
        };
        Ok(AssistantTurnOutcome {
            candidate_id,
            candidate_digest,
            preview_id,
            object_kind: spec.object_kind.to_owned(),
            context_refs,
            engine_id: ASSISTANT_ENGINE_ID.to_owned(),
            pi_pin: ASSISTANT_PI_PIN.to_owned(),
            protocol: ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL.to_owned(),
        })
    }

    /// Assistant cannot append the conversation archive.
    pub fn write_archive(
        &self,
        spec: &ArchiveAppendSpec<'_>,
    ) -> Result<String, ProjectAggregateError> {
        self.conversations
            .append(ConfirmCaller::Assistant, spec)
            .map_err(|error| match error {
                ProjectAggregateError::Forbidden { .. } => ProjectAggregateError::Forbidden {
                    detail: "assistant cannot write conversation archive",
                },
                other => other,
            })
    }

    /// Assistant cannot write SecretStore. No secret material is accepted.
    pub fn write_secret(&self, _name: &str, _material: &str) -> Result<(), ProjectAggregateError> {
        let _ = self;
        Err(ProjectAggregateError::Forbidden {
            detail: "assistant cannot write SecretStore",
        })
    }

    /// Assistant cannot confirm authority previews (chat has no Approve).
    pub fn confirm_preview(
        &self,
        preview_id: &str,
        preview_digest: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        match self.projects.confirm_preview(
            ConfirmCaller::Assistant,
            preview_id,
            preview_digest,
            now_ms,
        ) {
            Ok(_) => Err(ProjectAggregateError::Forbidden {
                detail: "assistant cannot mix draft-apply with authority-approve",
            }),
            Err(ProjectAggregateError::Forbidden { .. }) => Err(ProjectAggregateError::Forbidden {
                detail: "assistant cannot confirm authority preview",
            }),
            Err(error) => Err(error),
        }
    }

    /// Assistant cannot apply candidates onto drafts or authority objects.
    pub fn apply_candidate(
        &self,
        draft_id: &str,
        base_seq: i64,
        candidate_digest: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        match self.projects.apply_candidate(
            ConfirmCaller::Assistant,
            draft_id,
            base_seq,
            candidate_digest,
            now_ms,
        ) {
            Ok(_) => Err(ProjectAggregateError::Forbidden {
                detail: "assistant cannot mix draft-apply with authority-approve",
            }),
            Err(ProjectAggregateError::Forbidden { .. }) => Err(ProjectAggregateError::Forbidden {
                detail: "assistant cannot apply candidates onto authority",
            }),
            Err(error) => Err(error),
        }
    }

    /// Assistant cannot grant Employee capabilities.
    pub fn grant_capability(
        &self,
        employees: &EmployeeStore,
        project_id: &str,
        employee_id: &str,
        capability_ref: &str,
        scope: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        match employees.grant_capability(
            ConfirmCaller::Assistant,
            project_id,
            employee_id,
            capability_ref,
            scope,
            now_ms,
        ) {
            Ok(_) => Err(ProjectAggregateError::Forbidden {
                detail: "assistant cannot write authority grants",
            }),
            Err(ProjectAggregateError::Forbidden { .. }) => Err(ProjectAggregateError::Forbidden {
                detail: "assistant cannot write authority grants",
            }),
            Err(error) => Err(error),
        }
    }

    /// Assistant cannot admit Memory. T11 owns admission.
    pub fn write_memory(&self) -> Result<(), ProjectAggregateError> {
        let _ = self;
        Err(ProjectAggregateError::Forbidden {
            detail: "assistant cannot write Memory",
        })
    }

    fn read_context_refs(
        &self,
        project_id: Option<&str>,
    ) -> Result<Vec<String>, ProjectAggregateError> {
        let Some(project_id) = project_id else {
            return Ok(Vec::new());
        };
        let page = self.conversations.read_index(&ArchiveReadSpec {
            projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
            caller_project_id: project_id,
            target_project_id: project_id,
            employee_id: None,
            limit: CONVERSATION_RESUME_LIMIT,
            resume_from: None,
            include_bodies: false,
        })?;
        Ok(page.records.into_iter().map(|row| row.record_id).collect())
    }
}
