//! Hidden Pi Personal Assistant (P11-T06 candidate path; P13-T03 real inference).
//!
//! Candidate-only engine: exact Pi `0.81.1` / `cognitiveos.private-candidate/1`
//! reused as identity pins, not a second scheduler or Installed Agent.
//! Conversation archive is read-only context. Writes to archive, SecretStore,
//! Memory, and authority confirm/apply are fail-closed.
//!
//! P13-T03: a turn is registered only when the daemon observed a real
//! inference — exact Pi reached the Provider through the daemon-owned private
//! completion proxy at least once and returned a closed candidate object chain
//! whose every field carries typed provenance. Echoing the client payload back
//! as a "candidate" is refused here, not merely discouraged.

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
use crate::vault::CONTEXT_INJECT_ORDER;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// Product pin reused from the existing managed Pi path. Not a Windows OPC claim.
pub const ASSISTANT_PI_PIN: &str = "0.81.1";
/// Existing private-candidate protocol. Not a new carrier.
pub const ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL: &str = "cognitiveos.private-candidate/1";
/// Hidden engine identity. Pi is not an Installed Agent.
pub const ASSISTANT_ENGINE_ID: &str = "cognitiveos.personal.hidden-pi-assistant/0.1";
/// Research fetch reuses the audited read-only family. No parallel HTTP client.
pub const ASSISTANT_RESEARCH_FETCH_FAMILY: &str = "HttpFetchReadOnly";
/// Daemon ⇄ exact-Pi adapter inference frame (P13-T03). Personal-private; not a
/// Core contract and not a second candidate schema: the candidate still lands
/// through v26 `register_candidate`.
pub const ASSISTANT_INFERENCE_PROTOCOL: &str = "cognitiveos.personal.assistant-inference/0.1";
/// Where the create-page chat points when no Provider is bound. The chat does
/// not render an input box, does not bind a model silently, and never asks for
/// a key in chat.
pub const ASSISTANT_SETTINGS_ROUTE: &str = "#/settings";
/// Bounded assistant reply retained in the candidate ops.
pub const ASSISTANT_REPLY_LIMIT: usize = 8 * 1024;
/// Bounded object chain: BusinessBrief → ResearchRun → Charter/Axis/Roster/Recipe.
pub const ASSISTANT_CHAIN_MAX_OBJECTS: usize = 6;
/// Bounded fields per chain object.
pub const ASSISTANT_CHAIN_MAX_FIELDS: usize = 32;

/// Closed candidate object kinds, in chain rank order.
pub const ASSISTANT_OBJECT_KINDS: [&str; 6] = [
    "business-brief",
    "research-run",
    "charter",
    "axis",
    "roster",
    "recipe",
];
/// Closed turn kinds.
pub const ASSISTANT_TURN_KINDS: [&str; 4] = ["explain", "navigate", "research", "propose"];

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
const CHAIN_OBJECT_KEYS: &[&str] = &["object_kind", "fields", "summary"];
const CHAIN_FIELD_KEYS: &[&str] = &["value", "provenance"];

/// Clippy-safe turn input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistantTurnSpec<'a> {
    pub kind: &'a str,
    pub draft_id: &'a str,
    pub object_kind: &'a str,
    /// Owner-authored input for this turn (never treated as the candidate).
    pub payload: &'a Value,
    /// Typed provenance of the owner payload.
    pub provenance_json: &'a str,
    pub project_id: Option<&'a str>,
    pub tools: &'a [&'a str],
    /// Daemon-observed inference for this turn. Required: no echo path exists.
    pub inference: &'a AssistantInferenceRecord<'a>,
    pub now_ms: i64,
}

/// What the daemon observed while exact Pi produced this turn. Pi text alone
/// is never authority; these facts are recorded into the candidate ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistantInferenceRecord<'a> {
    /// Must equal [`ASSISTANT_INFERENCE_PROTOCOL`].
    pub protocol: &'a str,
    /// Bound Provider model the daemon proxied for `agent://personal/pi`.
    pub model_id: &'a str,
    /// Completions the daemon forwarded through its private proxy for this turn.
    /// Zero means Pi never inferred; the turn is refused.
    pub provider_round_trips: u32,
    /// Candidate object chain (JSON array) parsed from Pi's final message.
    pub objects: &'a Value,
    /// Bounded assistant reply for the chat surface.
    pub reply: &'a str,
    /// Daemon-derived citable URIs: research fetches that actually completed
    /// plus owner-supplied sources. Any other `sources[]` uri is fabricated.
    pub allowed_source_uris: &'a [String],
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
    pub inference_protocol: String,
    pub model_id: String,
    pub provider_round_trips: u32,
    pub chain_object_kinds: Vec<String>,
    pub reply: String,
    /// The registered candidate ops (digest = `candidate_digest`). The store
    /// persists the digest only; callers render the chain from this value.
    pub candidate_ops: Value,
}

/// Fixed guidance rendered by the create-page chat when no Provider is bound
/// to `agent://personal/pi`. It is a pointer, not a chat box: no input, no
/// silent bind, no key request, no candidate.
pub fn provider_unbound_guidance() -> Value {
    json!({
        "status": "provider_unbound",
        "engine": ASSISTANT_ENGINE_ID,
        "installed_agent": false,
        "settings_route": ASSISTANT_SETTINGS_ROUTE,
        "guidance": "No model is bound to the assistant yet. Open Settings to connect a Provider and bind the assistant. The chat does not accept keys and does not bind a model silently.",
        "chat_input": false,
        "silent_bind": false,
        "candidate_registered": false,
        "observation_only": true,
    })
}

/// Rank of an object kind inside the closed chain order.
fn chain_rank(kind: &str) -> Option<usize> {
    match kind {
        "business-brief" => Some(0),
        "research-run" => Some(1),
        "charter" | "axis" | "roster" | "recipe" => Some(2),
        _ => None,
    }
}

/// Validate one inferred candidate object chain against the closed schema and
/// the daemon-derived citable sources. This is the single object-chain
/// validator: runtime parsing and HTTP handling call into it; nothing else
/// re-implements the schema.
pub fn validate_inferred_object_chain(
    objects: &Value,
    allowed_source_uris: &[String],
) -> Result<Vec<String>, ProjectAggregateError> {
    let bytes = serde_json::to_vec(objects).map_err(|_| ProjectAggregateError::Invalid {
        detail: "assistant object chain must be JSON",
    })?;
    reject_closed_candidate_schema(&bytes)?;
    let Some(items) = objects.as_array() else {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant object chain must be a JSON array of objects",
        });
    };
    if items.is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant object chain is empty: inference produced no candidate object",
        });
    }
    if items.len() > ASSISTANT_CHAIN_MAX_OBJECTS {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant object chain exceeds the bounded object count",
        });
    }
    let mut kinds = Vec::with_capacity(items.len());
    let mut last_rank = 0usize;
    for item in items {
        let Some(object) = item.as_object() else {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object must be a JSON object",
            });
        };
        if !object
            .keys()
            .all(|key| CHAIN_OBJECT_KEYS.contains(&key.as_str()))
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object has fields outside the closed schema",
            });
        }
        let Some(kind) = object.get("object_kind").and_then(Value::as_str) else {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object requires object_kind",
            });
        };
        let Some(rank) = chain_rank(kind) else {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant object_kind is closed: business-brief/research-run/charter/axis/roster/recipe",
            });
        };
        if kinds.iter().any(|seen: &String| seen == kind) {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant object chain repeats an object kind",
            });
        }
        if rank < last_rank {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant object chain is out of order: BusinessBrief → ResearchRun → Charter/Axis/Roster/Recipe",
            });
        }
        last_rank = rank;
        if let Some(summary) = object.get("summary")
            && !summary.is_string()
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object summary must be a string",
            });
        }
        let Some(fields) = object.get("fields").and_then(Value::as_object) else {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object requires a fields object",
            });
        };
        if fields.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object has no fields",
            });
        }
        if fields.len() > ASSISTANT_CHAIN_MAX_FIELDS {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant chain object exceeds the bounded field count",
            });
        }
        for (name, field) in fields {
            if name.trim().is_empty() {
                return Err(ProjectAggregateError::Invalid {
                    detail: "assistant chain field name is empty",
                });
            }
            let Some(field) = field.as_object() else {
                return Err(ProjectAggregateError::Invalid {
                    detail: "assistant chain field requires typed provenance: {value, provenance}",
                });
            };
            if !field
                .keys()
                .all(|key| CHAIN_FIELD_KEYS.contains(&key.as_str()))
                || !field.contains_key("value")
            {
                return Err(ProjectAggregateError::Invalid {
                    detail: "assistant chain field requires typed provenance: {value, provenance}",
                });
            }
            let Some(provenance) = field.get("provenance") else {
                return Err(ProjectAggregateError::Invalid {
                    detail: "assistant chain field without provenance is refused",
                });
            };
            let provenance_json =
                serde_json::to_string(provenance).map_err(|_| ProjectAggregateError::Invalid {
                    detail: "assistant chain field provenance must be JSON",
                })?;
            validate_assistant_provenance(Some(&provenance_json))?;
            validate_cited_sources(provenance, allowed_source_uris)?;
        }
        kinds.push(kind.to_owned());
    }
    Ok(kinds)
}

/// `sources[]` may cite only URIs the daemon actually fetched or the owner
/// supplied. A model-invented URI is a fabricated source and is refused.
fn validate_cited_sources(
    provenance: &Value,
    allowed_source_uris: &[String],
) -> Result<(), ProjectAggregateError> {
    let cited: Vec<&Value> = match provenance {
        Value::Array(items) => items.iter().collect(),
        Value::Object(object) => match object.get("sources").and_then(Value::as_array) {
            Some(items) => items.iter().collect(),
            None => return Ok(()),
        },
        _ => return Ok(()),
    };
    for source in cited {
        let uri = source
            .get("uri")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if !allowed_source_uris.iter().any(|allowed| allowed == uri) {
            return Err(ProjectAggregateError::Invalid {
                detail: "fabricated sources rejected: uri was neither fetched by HttpFetchReadOnly nor owner-supplied",
            });
        }
    }
    Ok(())
}

/// Validate the daemon-observed inference facts before any registration.
fn validate_inference_record(
    record: &AssistantInferenceRecord<'_>,
) -> Result<(), ProjectAggregateError> {
    if record.protocol != ASSISTANT_INFERENCE_PROTOCOL {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant inference protocol mismatch; echoing a client payload is not inference",
        });
    }
    if record.model_id.trim().is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant inference requires the bound Provider model id",
        });
    }
    if record.provider_round_trips == 0 {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant inference required: exact Pi made no Provider round trip through the daemon proxy",
        });
    }
    if record.reply.trim().is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant inference reply is empty",
        });
    }
    if record.reply.len() > ASSISTANT_REPLY_LIMIT {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant inference reply exceeds the bounded length",
        });
    }
    Ok(())
}

/// Hidden assistant plane over the daemon-owned writer.
#[derive(Clone)]
pub struct AssistantPlane {
    projects: ProjectAggregateStore,
    conversations: ConversationStore,
    /// Read-only handle for candidate accounting; every write goes through
    /// `projects` / `conversations`.
    conn: Arc<Mutex<Connection>>,
}

impl AssistantPlane {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            projects: ProjectAggregateStore::from_authority_store(store),
            conversations: ConversationStore::from_authority_store(store),
            conn: Arc::clone(&store.conn),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let conn = Connection::open(path).map_err(|source| ProjectAggregateError::Unavailable {
            detail: format!("open assistant plane: {source}"),
        })?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .map_err(|source| ProjectAggregateError::Unavailable {
                detail: format!("configure assistant plane: {source}"),
            })?;
        Ok(Self {
            projects: ProjectAggregateStore::open_path(path)?,
            conversations: ConversationStore::open_path(path)?,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Registered candidates for a draft (read-only; tests and the `status`
    /// projection use it to prove refused turns wrote nothing).
    pub fn candidate_count(&self, draft_id: &str) -> Result<i64, ProjectAggregateError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority reader lock poisoned".to_owned(),
            })?;
        conn.query_row(
            "SELECT COUNT(*) FROM p11_candidate WHERE draft_id = ?1",
            [draft_id],
            |row| row.get(0),
        )
        .map_err(|source| ProjectAggregateError::Unavailable {
            detail: format!("count candidates: {source}"),
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

    /// Validate the turn shape before any inference is attempted: closed turn
    /// and object kinds, typed owner provenance, default-deny tools. The HTTP
    /// caller runs this first so an ambient tool is refused before Pi spawns.
    pub fn admit_turn_request(
        kind: &str,
        object_kind: &str,
        provenance_json: &str,
        tools: &[&str],
    ) -> Result<(), ProjectAggregateError> {
        if !ASSISTANT_TURN_KINDS.contains(&kind) {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant turn kind must be explain, navigate, research, or propose",
            });
        }
        if !ASSISTANT_OBJECT_KINDS.contains(&object_kind) {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant object_kind is closed: business-brief/research-run/charter/axis/roster/recipe",
            });
        }
        validate_assistant_provenance(Some(provenance_json))?;
        for tool in tools {
            Self::admit_tool(kind, tool)?;
        }
        Ok(())
    }

    /// Read-only Context refs (conversation archive index) for a Project.
    pub fn context_refs(
        &self,
        project_id: Option<&str>,
    ) -> Result<Vec<String>, ProjectAggregateError> {
        self.read_context_refs(project_id)
    }

    /// Explain / navigate / research / propose → inferred candidate chain +
    /// optional preview. Refuses any turn without a daemon-observed inference.
    pub fn run_turn(
        &self,
        spec: &AssistantTurnSpec<'_>,
    ) -> Result<AssistantTurnOutcome, ProjectAggregateError> {
        Self::admit_turn_request(
            spec.kind,
            spec.object_kind,
            spec.provenance_json,
            spec.tools,
        )?;
        validate_inference_record(spec.inference)?;
        let chain_object_kinds = validate_inferred_object_chain(
            spec.inference.objects,
            spec.inference.allowed_source_uris,
        )?;
        if !chain_object_kinds
            .iter()
            .any(|kind| kind == spec.object_kind)
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "assistant object chain does not contain the requested object_kind",
            });
        }
        let owner_provenance: Value = serde_json::from_str(spec.provenance_json).map_err(|_| {
            ProjectAggregateError::Invalid {
                detail: "assistant provenance must be typed JSON, not an unlabeled blob",
            }
        })?;
        let context_refs = self.read_context_refs(spec.project_id)?;
        // The reply is returned to the chat surface and digest-bound here; the
        // inject order is referenced by name because its first label carries
        // the secret-shape substring the registration guard scans for.
        let ops = json!({
            "engine": ASSISTANT_ENGINE_ID,
            "pi_pin": ASSISTANT_PI_PIN,
            "protocol": ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL,
            "inference_protocol": ASSISTANT_INFERENCE_PROTOCOL,
            "installed_agent": false,
            "turn": spec.kind,
            "object_kind": spec.object_kind,
            "owner_payload": spec.payload,
            "owner_provenance": owner_provenance,
            "model_id": spec.inference.model_id,
            "provider_round_trips": spec.inference.provider_round_trips,
            "chain": spec.inference.objects,
            "reply_digest": ProjectAggregateStore::digest_hex(spec.inference.reply.as_bytes()),
            "allowed_source_uris": spec.inference.allowed_source_uris,
            "context_refs": context_refs,
            "inject_order_ref": "CONTEXT_INJECT_ORDER",
            "inject_order_layers": CONTEXT_INJECT_ORDER.len(),
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
            match self.projects.request_preview(
                "activation",
                spec.draft_id,
                &preview_bytes,
                spec.now_ms,
            ) {
                Ok((preview_id, _)) => Some(preview_id),
                // A second research/propose turn on the same draft re-announces
                // the pending preview instead of failing after the candidate
                // was registered; the announcement is still not an Approve.
                Err(ProjectAggregateError::Conflict { .. }) => self
                    .projects
                    .list_pending_previews(spec.draft_id)?
                    .into_iter()
                    .find(|row| row.subject_kind == "activation")
                    .map(|row| row.preview_id),
                Err(error) => return Err(error),
            }
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
            inference_protocol: ASSISTANT_INFERENCE_PROTOCOL.to_owned(),
            model_id: spec.inference.model_id.to_owned(),
            provider_round_trips: spec.inference.provider_round_trips,
            chain_object_kinds,
            reply: spec.inference.reply.to_owned(),
            candidate_ops: ops,
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
