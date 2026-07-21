//! Task Shell server semantics (M5 RUN batch 2b).
//!
//! Client attach/detach never cancels work. Cancel is a request that closes
//! through Effect (`CANCEL_PENDING` → reconcile; late → `CANCEL_TOO_LATE`).
//! Completion/authority state is never taken from client display.

use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_domain::ObjectId;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Client-local shell phase (non-authority).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellPhase {
    Detached,
    Attached,
    CancelRequested,
}

/// One shell-bound task view (projection only).
#[derive(Debug, Clone)]
pub struct ShellTaskBinding {
    pub task_ref: String,
    pub phase: ShellPhase,
    pub cancel_effect_id: Option<ObjectId>,
    pub authority_cancelled: bool,
    pub effect_terminal: bool,
}

/// Deterministic shell service: proposal/preview/submit bookkeeping plus
/// attach/detach/cancel semantics. Intent admission and Effect dispatch are
/// invoked by callers with kernel ports; this type never writes authority.
#[derive(Debug, Default)]
pub struct ShellService {
    tasks: BTreeMap<String, ShellTaskBinding>,
    proposals: BTreeMap<String, Value>,
    previews: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{detail} ({code})")]
pub struct ShellError {
    pub code: &'static str,
    pub detail: String,
}

impl ShellError {
    fn new(code: RegisteredErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code: code.as_str(),
            detail: detail.into(),
        }
    }
}

impl ShellService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn proposal(&mut self, proposal_id: &str, body: Value) -> Result<Value, ShellError> {
        if self.proposals.contains_key(proposal_id) {
            return Err(ShellError::new(
                RegisteredErrorCode::StateConflict,
                "proposal_id already recorded",
            ));
        }
        self.proposals.insert(proposal_id.to_owned(), body.clone());
        Ok(json!({"proposal_id":proposal_id,"status":"recorded","authority":false}))
    }

    pub fn preview(
        &mut self,
        proposal_id: &str,
        preview_digest: &str,
    ) -> Result<Value, ShellError> {
        if !self.proposals.contains_key(proposal_id) {
            return Err(ShellError::new(
                RegisteredErrorCode::StateConflict,
                "preview requires a recorded proposal",
            ));
        }
        self.previews
            .insert(proposal_id.to_owned(), preview_digest.to_owned());
        Ok(
            json!({"proposal_id":proposal_id,"preview_digest":preview_digest,"status":"previewed","authority":false}),
        )
    }

    /// Submit is receipt-level only until the caller runs intent admission.
    pub fn submit(
        &mut self,
        proposal_id: &str,
        preview_digest: &str,
        task_ref: &str,
    ) -> Result<Value, ShellError> {
        let Some(expected) = self.previews.get(proposal_id) else {
            return Err(ShellError::new(
                RegisteredErrorCode::StateConflict,
                "submit requires a preview",
            ));
        };
        if expected != preview_digest {
            return Err(ShellError::new(
                RegisteredErrorCode::StateConflict,
                "stale preview digest",
            ));
        }
        self.tasks.insert(
            task_ref.to_owned(),
            ShellTaskBinding {
                task_ref: task_ref.to_owned(),
                phase: ShellPhase::Detached,
                cancel_effect_id: None,
                authority_cancelled: false,
                effect_terminal: false,
            },
        );
        Ok(
            json!({"proposal_id":proposal_id,"task_ref":task_ref,"status":"accepted","authority":false,"note":"receipt-level; authority state requires intent/effect path"}),
        )
    }

    pub fn attach(&mut self, task_ref: &str) -> Result<Value, ShellError> {
        let task = self.task_mut(task_ref)?;
        task.phase = ShellPhase::Attached;
        Ok(json!({"task_ref":task_ref,"phase":"attached","cancelled":false}))
    }

    /// Detach never cancels.
    pub fn detach(&mut self, task_ref: &str) -> Result<Value, ShellError> {
        let task = self.task_mut(task_ref)?;
        let was_cancel = matches!(task.phase, ShellPhase::CancelRequested);
        task.phase = ShellPhase::Detached;
        Ok(json!({
            "task_ref":task_ref,
            "phase":"detached",
            "cancelled":false,
            "cancel_still_pending":was_cancel && !task.authority_cancelled,
        }))
    }

    /// Request cancel. If the effect is already terminal → `CANCEL_TOO_LATE`.
    /// Otherwise record `CANCEL_PENDING` (Effect reconcile is caller's job).
    pub fn cancel(
        &mut self,
        task_ref: &str,
        cancel_effect_id: ObjectId,
        effect_already_terminal: bool,
    ) -> Result<Value, ShellError> {
        let task = self.task_mut(task_ref)?;
        if effect_already_terminal || task.effect_terminal {
            return Err(ShellError {
                code: "CANCEL_TOO_LATE",
                detail: "effect already terminal; cancel cannot apply".to_owned(),
            });
        }
        task.phase = ShellPhase::CancelRequested;
        task.cancel_effect_id = Some(cancel_effect_id.clone());
        Ok(json!({
            "task_ref":task_ref,
            "status":"CANCEL_PENDING",
            "cancel_effect_id":cancel_effect_id.as_str(),
            "authority":false,
        }))
    }

    /// Authority projection informs the shell that cancel closed.
    pub fn observe_authority_cancelled(&mut self, task_ref: &str) -> Result<(), ShellError> {
        let task = self.task_mut(task_ref)?;
        task.authority_cancelled = true;
        task.effect_terminal = true;
        Ok(())
    }

    pub fn phase(&self, task_ref: &str) -> Option<ShellPhase> {
        self.tasks.get(task_ref).map(|t| t.phase)
    }

    fn task_mut(&mut self, task_ref: &str) -> Result<&mut ShellTaskBinding, ShellError> {
        self.tasks.get_mut(task_ref).ok_or_else(|| {
            ShellError::new(
                RegisteredErrorCode::StateConflict,
                format!("unknown task_ref {task_ref}"),
            )
        })
    }
}
