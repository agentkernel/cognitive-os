#![allow(unused, unused_imports)]

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{
    MAXIMUM_WORKSPACE_MUTATION_PAYLOAD_BYTES, NativeOperationFamily, NativeToolDescriptor, ToolRisk,
};
use cognitive_kernel::{
    authz::AuthorizationGrant,
    effects::{EffectError, EffectProtocol, GovernanceCurrency, WriterLease},
    engine::CommittedTransition,
    executor::{
        DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    },
    ports::PortFailure,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedWorkspaceMutationRequest {
    parameters_digest: String,
    action: String,
    target: String,
    family: NativeOperationFamily,
    approved_workspace_root: PathBuf,
    relative_workspace_path: PathBuf,
    expected_preimage: WorkspacePreimage,
    payload: Vec<u8>,
    /// Known before dispatch for a whole-file write; a patch's postimage
    /// depends on the preimage bytes and is resolved during dispatch.
    intended_postimage_digest: Option<String>,
    output_limit_bytes: usize,
}

const MUTATION_STATE_NAMESPACE: &str = "workspace-mutation";
const MUTATION_TARGET_LOCK_NAMESPACE: &str = "workspace-mutation-target";
const MUTATION_STATE_SCHEMA: &str = "native-workspace-mutation-state/0.1";
pub(crate) const MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MutationAttemptStatus {
    Staged,
    Attempted,
    NotExecuted,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MutationStateRecord {
    schema: String,
    idempotency_key: String,
    parameters_digest: String,
    action: String,
    target: String,
    family: String,
    expected_preimage: String,
    payload_digest: String,
    intended_postimage_digest: Option<String>,
    status: MutationAttemptStatus,
    receipt_ref: Option<String>,
    redacted_output: Option<Vec<u8>>,
}

/// Daemon-private workspace mutation sink for `WorkspaceWrite` and
/// `WorkspacePatch`.
///
/// Three properties make this safe to run behind the Effect protocol.
///
/// **Compare-and-swap.** Staging binds the preimage the caller expects to
/// replace. The sink verifies it immediately before building the postimage and
/// again immediately before publishing, so a target that changed underneath the
/// Intent is refused instead of clobbered.
///
/// **Atomic publish.** The postimage is written to a sibling staging file,
/// flushed and synced, and then renamed onto the target. A reader therefore
/// observes either the preimage or the postimage, never a partial file, and a
/// failed publish removes the staging file and re-reads the target to classify
/// what actually happened.
///
/// **A genuinely queryable sink.** Attempt and completion state is persisted
/// outside the workspace under the original idempotency key. `query_outcome`
/// requires that key-bound durable receipt: matching bytes alone are never
/// attributed to this attempt, while a later reversion cannot erase proof that
/// the original key executed.
///
/// Retained output is a bounded receipt line: target, action, byte count and
/// postimage digest. File content is never retained, because a mutation
/// payload is the least appropriate thing to echo into an Effect receipt.
pub(crate) struct NativeWorkspaceMutationExecutor {
    trusted_fencing_epoch: i64,
    staged_requests: Mutex<BTreeMap<String, StagedWorkspaceMutationRequest>>,
    state_store: Arc<DurableExecutorStateStore>,
    #[cfg(test)]
    publish_count: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    after_staging_write_hook: Mutex<Option<AfterStagingWriteHook>>,
    #[cfg(test)]
    after_final_preimage_check_hook: Mutex<Option<AfterFinalPreimageCheckHook>>,
}

/// Test seam invoked with `(target, staging_path)` once the postimage is fully
/// written but before the rename publishes it.
#[cfg(test)]
type AfterStagingWriteHook = Box<dyn Fn(&Path, &Path) + Send>;
#[cfg(test)]
type AfterFinalPreimageCheckHook = Box<dyn Fn() + Send>;

impl NativeWorkspaceMutationExecutor {
    pub(crate) fn new(
        trusted_fencing_epoch: i64,
        state_store: Arc<DurableExecutorStateStore>,
    ) -> Self {
        Self {
            trusted_fencing_epoch,
            staged_requests: Mutex::new(BTreeMap::new()),
            state_store,
            #[cfg(test)]
            publish_count: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            after_staging_write_hook: Mutex::new(None),
            #[cfg(test)]
            after_final_preimage_check_hook: Mutex::new(None),
        }
    }

    /// Bind one validated mutation to its durable Intent identity.
    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        request: &ValidatedNativeToolRequest,
    ) -> Result<(), NativeToolExecutionError> {
        validate_descriptor(&request.descriptor)?;
        if !matches!(
            request.descriptor.family,
            NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch
        ) {
            return Err(NativeToolExecutionError::UnsupportedExecutionFamily);
        }
        let relative_workspace_path = request
            .relative_workspace_path
            .as_ref()
            .ok_or(NativeToolExecutionError::WorkspaceTargetRequired)?;
        let approved_workspace_root = request
            .approved_workspace_root
            .as_ref()
            .ok_or(NativeToolExecutionError::WorkspaceTargetRequired)?;
        let expected_preimage = request
            .expected_preimage
            .as_ref()
            .ok_or(NativeToolExecutionError::MutationPreimageRequired)?;
        if idempotency_key.is_empty() || parameters_digest.is_empty() {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "idempotency key and parameters digest are required".to_owned(),
            ));
        }
        if request.input.is_empty() {
            return Err(NativeToolExecutionError::MutationInputRequired);
        }
        if request.input.len() > MAXIMUM_WORKSPACE_MUTATION_PAYLOAD_BYTES {
            return Err(NativeToolExecutionError::MutationPayloadTooLarge);
        }
        let intended_postimage_digest = match request.descriptor.family {
            NativeOperationFamily::WorkspaceWrite => Some(workspace_image_digest(&request.input)?),
            _ => None,
        };
        let staged_request = StagedWorkspaceMutationRequest {
            parameters_digest,
            action: request.descriptor.action.clone(),
            target: request.target.clone(),
            family: request.descriptor.family,
            approved_workspace_root: approved_workspace_root.clone(),
            relative_workspace_path: relative_workspace_path.clone(),
            expected_preimage: expected_preimage.clone(),
            payload: request.input.clone(),
            intended_postimage_digest,
            output_limit_bytes: request.descriptor.output_limit_bytes,
        };
        let state_guard = self
            .state_store
            .lock_key(MUTATION_STATE_NAMESPACE, &idempotency_key)
            .map_err(|error| {
                NativeToolExecutionError::ExecutorUnavailable(format!(
                    "durable mutation state lock failed: {error}"
                ))
            })?;
        match state_guard.read::<MutationStateRecord>().map_err(|error| {
            NativeToolExecutionError::ExecutorUnavailable(format!(
                "durable mutation state read failed: {error}"
            ))
        })? {
            Some(existing)
                if !mutation_state_matches(&existing, &idempotency_key, &staged_request)? =>
            {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            Some(_) => {}
            None => state_guard
                .write(&MutationStateRecord::staged(
                    &idempotency_key,
                    &staged_request,
                )?)
                .map_err(|error| {
                    NativeToolExecutionError::ExecutorUnavailable(format!(
                        "durable mutation state write failed: {error}"
                    ))
                })?,
        }
        let mut staged_requests = self.staged_requests.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "staged mutation store is poisoned".to_owned(),
            )
        })?;
        if let Some(existing_request) = staged_requests.get(&idempotency_key) {
            if existing_request != &staged_request {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_requests.insert(idempotency_key, staged_request);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn completed_output(&self, idempotency_key: &str) -> Option<Vec<u8>> {
        self.state_store
            .lock_key(MUTATION_STATE_NAMESPACE, idempotency_key)
            .ok()
            .and_then(|guard| guard.read::<MutationStateRecord>().ok().flatten())
            .and_then(|record| record.redacted_output)
    }

    #[cfg(test)]
    pub(crate) fn publish_count(&self) -> usize {
        self.publish_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn install_after_staging_write_hook(
        &self,
        hook: impl Fn(&Path, &Path) + Send + 'static,
    ) {
        let mut after_staging_write_hook = match self.after_staging_write_hook.lock() {
            Ok(after_staging_write_hook) => after_staging_write_hook,
            Err(poisoned) => poisoned.into_inner(),
        };
        *after_staging_write_hook = Some(Box::new(hook));
    }

    #[cfg(test)]
    pub(crate) fn install_after_final_preimage_check_hook(&self, hook: impl Fn() + Send + 'static) {
        let mut stored_hook = match self.after_final_preimage_check_hook.lock() {
            Ok(stored_hook) => stored_hook,
            Err(poisoned) => poisoned.into_inner(),
        };
        *stored_hook = Some(Box::new(hook));
    }

    fn publish_staged_workspace_mutation(
        &self,
        call: &ExecutorCall,
        staged_request: &StagedWorkspaceMutationRequest,
    ) -> Result<DispatchOutcome, PortFailure> {
        if call.action != staged_request.action
            || call.target != staged_request.target
            || call.parameters_digest != staged_request.parameters_digest
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "dispatch does not match the daemon-staged workspace mutation".to_owned(),
            });
        }
        let workspace =
            AnchoredWorkspace::open(&staged_request.approved_workspace_root).map_err(|error| {
                PortFailure {
                    detail: format!("workspace root handle open failed: {error}"),
                }
            })?;
        let (target_parent, target_name) = workspace
            .open_parent(&staged_request.relative_workspace_path)
            .map_err(|error| PortFailure {
                detail: format!("workspace target parent handle open failed: {error}"),
            })?;
        let target_parent_identity =
            directory_identity(&target_parent).map_err(|error| PortFailure {
                detail: format!("workspace target parent identity failed: {error}"),
            })?;
        let target_lock_key = mutation_target_lock_key(target_parent_identity, &target_name);
        let _target_guard = match self
            .state_store
            .try_lock_key(MUTATION_TARGET_LOCK_NAMESPACE, &target_lock_key)
        {
            Ok(guard) => guard,
            Err(StateLockError::WouldBlock) => {
                return Ok(DispatchOutcome::NotExecuted {
                    reason: "workspace mutation target is locked by another compare-and-swap"
                        .to_owned(),
                });
            }
            Err(StateLockError::Io(error)) => {
                return Err(PortFailure {
                    detail: format!("workspace mutation target lock failed: {error}"),
                });
            }
        };
        let state_guard = self
            .state_store
            .lock_key(MUTATION_STATE_NAMESPACE, &call.idempotency_key)
            .map_err(|error| PortFailure {
                detail: format!("durable mutation state lock failed: {error}"),
            })?;
        let Some(mut state) =
            state_guard
                .read::<MutationStateRecord>()
                .map_err(|error| PortFailure {
                    detail: format!("durable mutation state read failed: {error}"),
                })?
        else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no durable daemon-staged mutation for idempotency key".to_owned(),
            });
        };
        if !mutation_state_matches(&state, &call.idempotency_key, staged_request).map_err(
            |error| PortFailure {
                detail: format!("durable mutation binding check failed: {error}"),
            },
        )? {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "durable mutation binding does not match dispatch".to_owned(),
            });
        }
        if state.status == MutationAttemptStatus::Completed {
            let Some(receipt_ref) = state.receipt_ref else {
                return Ok(DispatchOutcome::Unknown {
                    detail: "completed mutation state has no key-bound receipt".to_owned(),
                });
            };
            if receipt_ref != format!("tool-receipt://workspace-mutation/{}", call.idempotency_key)
            {
                return Ok(DispatchOutcome::Unknown {
                    detail: "completed mutation receipt is bound to a different key".to_owned(),
                });
            }
            return Ok(DispatchOutcome::Executed { receipt_ref });
        }
        let staging_name = staging_file_name(&target_name, &call.idempotency_key)?;
        if state.status == MutationAttemptStatus::Attempted {
            if let Err(error) = remove_regular_file(&target_parent, &staging_name) {
                return Ok(DispatchOutcome::Unknown {
                    detail: format!(
                        "prior mutation attempt is unresolved and orphan staging cleanup failed: {error}"
                    ),
                });
            }
            return Ok(DispatchOutcome::Unknown {
                detail: "a prior workspace mutation attempt has no durable terminal receipt"
                    .to_owned(),
            });
        }
        state.status = MutationAttemptStatus::Attempted;
        state.receipt_ref = None;
        state.redacted_output = None;
        state_guard.write(&state).map_err(|error| PortFailure {
            detail: format!("persist workspace mutation attempt before I/O: {error}"),
        })?;

        if let Err(error) = remove_regular_file(&target_parent, &staging_name) {
            return Ok(DispatchOutcome::Unknown {
                detail: format!("orphan staging cleanup failed before publication: {error}"),
            });
        }
        let preimage = match read_target_snapshot(
            &target_parent,
            &target_name,
            staged_request.family == NativeOperationFamily::WorkspacePatch,
        )? {
            TargetSnapshot::NotARegularFile => {
                return finish_not_executed(
                    &state_guard,
                    &mut state,
                    "workspace mutation target is a link, reparse point, or not a regular file",
                );
            }
            TargetSnapshot::PatchPreimageTooLarge => {
                return finish_not_executed(
                    &state_guard,
                    &mut state,
                    &format!(
                        "workspace patch preimage exceeds the explicit {} byte ceiling",
                        MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES
                    ),
                );
            }
            snapshot => snapshot,
        };
        if !preimage_matches_snapshot(&staged_request.expected_preimage, &preimage) {
            return finish_not_executed(
                &state_guard,
                &mut state,
                "workspace mutation preimage does not match the staged expectation",
            );
        }

        let postimage_bytes = match build_postimage(staged_request, &preimage)? {
            Ok(postimage) => postimage,
            Err(reason) => {
                return finish_not_executed(
                    &state_guard,
                    &mut state,
                    &format!("workspace patch does not apply: {reason}"),
                );
            }
        };
        let postimage_digest =
            workspace_image_digest(&postimage_bytes).map_err(|error| PortFailure {
                detail: format!("workspace postimage digest failed: {error}"),
            })?;
        state.intended_postimage_digest = Some(postimage_digest.clone());
        state_guard.write(&state).map_err(|error| PortFailure {
            detail: format!("persist resolved mutation postimage before publication: {error}"),
        })?;
        match self.publish_atomically(
            &workspace,
            &target_parent,
            &target_name,
            target_parent_identity,
            &staging_name,
            &postimage_bytes,
            staged_request,
        )? {
            PublishOutcome::Published => {}
            PublishOutcome::RefusedTargetChanged => {
                return finish_not_executed(
                    &state_guard,
                    &mut state,
                    "workspace mutation target changed before publication",
                );
            }
            PublishOutcome::FailedTargetUnchanged { detail } => {
                return finish_not_executed(
                    &state_guard,
                    &mut state,
                    &format!("workspace mutation publication failed: {detail}"),
                );
            }
            PublishOutcome::FailedTargetUncertain { detail } => {
                return Ok(DispatchOutcome::Unknown { detail });
            }
        }

        let receipt_ref = format!("tool-receipt://workspace-mutation/{}", call.idempotency_key);
        // The receipt records what changed, never the bytes that changed.
        let redacted_output = redact_sensitive_output(&format!(
            "{}:{}:{}:{postimage_digest}\n",
            staged_request.target,
            staged_request.action,
            postimage_bytes.len(),
        ))
        .into_bytes()
        .into_iter()
        .take(staged_request.output_limit_bytes)
        .collect::<Vec<_>>();
        state.status = MutationAttemptStatus::Completed;
        state.receipt_ref = Some(receipt_ref.clone());
        state.redacted_output = Some(redacted_output);
        if let Err(error) = state_guard.write(&state) {
            return Ok(DispatchOutcome::Unknown {
                detail: format!(
                    "workspace mutation published but its key-bound receipt could not be persisted: {error}"
                ),
            });
        }
        Ok(DispatchOutcome::Executed { receipt_ref })
    }

    fn publish_atomically(
        &self,
        workspace: &AnchoredWorkspace,
        target_parent: &cap_std::fs::Dir,
        target_name: &OsStr,
        target_parent_identity: FileIdentity,
        staging_name: &OsStr,
        postimage_bytes: &[u8],
        staged_request: &StagedWorkspaceMutationRequest,
    ) -> Result<PublishOutcome, PortFailure> {
        let mut staging_file = match create_new_regular_file(target_parent, staging_name) {
            Ok(staging_file) => staging_file,
            Err(error) => {
                return Ok(match remove_regular_file(target_parent, staging_name) {
                    Ok(_) => PublishOutcome::FailedTargetUnchanged {
                        detail: error.to_string(),
                    },
                    Err(cleanup_error) => PublishOutcome::FailedTargetUncertain {
                        detail: format!(
                            "staging creation failed ({error}); cleanup also failed ({cleanup_error})"
                        ),
                    },
                });
            }
        };
        if let Err(error) = staging_file
            .write_all(postimage_bytes)
            .and_then(|()| staging_file.flush())
            .and_then(|()| staging_file.sync_all())
        {
            return Ok(match remove_regular_file(target_parent, staging_name) {
                Ok(_) => PublishOutcome::FailedTargetUnchanged {
                    detail: error.to_string(),
                },
                Err(cleanup_error) => PublishOutcome::FailedTargetUncertain {
                    detail: format!(
                        "staging write failed ({error}); cleanup also failed ({cleanup_error})"
                    ),
                },
            });
        }

        #[cfg(test)]
        {
            let target_path = staged_request
                .approved_workspace_root
                .join(&staged_request.relative_workspace_path);
            let staging_path = target_path
                .parent()
                .unwrap_or(&staged_request.approved_workspace_root)
                .join(staging_name);
            let after_staging_write_hook = self
                .after_staging_write_hook
                .lock()
                .map_err(|_| PortFailure {
                    detail: "after-staging-write hook store is poisoned".to_owned(),
                })?
                .take();
            if let Some(after_staging_write_hook) = after_staging_write_hook {
                after_staging_write_hook(&target_path, &staging_path);
            }
        }

        let current_state = read_target_snapshot(
            target_parent,
            target_name,
            staged_request.family == NativeOperationFamily::WorkspacePatch,
        )?;
        if matches!(current_state, TargetSnapshot::NotARegularFile)
            || !preimage_matches_snapshot(&staged_request.expected_preimage, &current_state)
        {
            return cleanup_after_refusal(target_parent, staging_name);
        }

        #[cfg(test)]
        {
            let hook = self
                .after_final_preimage_check_hook
                .lock()
                .map_err(|_| PortFailure {
                    detail: "after-final-preimage-check hook store is poisoned".to_owned(),
                })?
                .take();
            if let Some(hook) = hook {
                hook();
            }
        }

        // The test hook models an uncooperative pathname writer in the exact
        // former check-to-rename interval. Re-read through the held parent
        // handle after that interval; compliant writers are additionally
        // excluded by the target lock held for this whole transaction.
        let final_state = read_target_snapshot(
            target_parent,
            target_name,
            staged_request.family == NativeOperationFamily::WorkspacePatch,
        )?;
        if matches!(
            final_state,
            TargetSnapshot::NotARegularFile | TargetSnapshot::PatchPreimageTooLarge
        ) || !preimage_matches_snapshot(&staged_request.expected_preimage, &final_state)
        {
            return cleanup_after_refusal(target_parent, staging_name);
        }

        // Re-resolve the parent through the held workspace root and compare its
        // handle identity. A pathname swap cannot redirect publication: on
        // Windows the open parent handle also denies delete/rename sharing,
        // while on Unix a replaced path yields a different device/inode and is
        // refused before the handle-relative rename.
        let parent_relative = staged_request
            .relative_workspace_path
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let reopened_parent = match workspace.open_directory(parent_relative) {
            Ok(parent) => parent,
            Err(_) => return cleanup_after_refusal(target_parent, staging_name),
        };
        let reopened_identity =
            directory_identity(&reopened_parent).map_err(|error| PortFailure {
                detail: format!("reopened workspace parent identity failed: {error}"),
            })?;
        if reopened_identity != target_parent_identity {
            return cleanup_after_refusal(target_parent, staging_name);
        }

        #[cfg(test)]
        self.publish_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Err(error) = target_parent.rename(staging_name, target_parent, target_name) {
            let cleanup = remove_regular_file(target_parent, staging_name);
            if let Err(cleanup_error) = cleanup {
                return Ok(PublishOutcome::FailedTargetUncertain {
                    detail: format!(
                        "workspace mutation rename failed ({error}); staging cleanup also failed ({cleanup_error})"
                    ),
                });
            }
            // A failed rename leaves the target untouched on every supported
            // platform, but say so from observation rather than assumption.
            return Ok(
                match read_target_snapshot(
                    target_parent,
                    target_name,
                    staged_request.family == NativeOperationFamily::WorkspacePatch,
                )? {
                    snapshot
                        if preimage_matches_snapshot(
                            &staged_request.expected_preimage,
                            &snapshot,
                        ) =>
                    {
                        PublishOutcome::FailedTargetUnchanged {
                            detail: error.to_string(),
                        }
                    }
                    _ => PublishOutcome::FailedTargetUncertain {
                        detail: format!("workspace mutation publication is uncertain: {error}"),
                    },
                },
            );
        }
        if let Err(error) = sync_directory(target_parent) {
            return Ok(PublishOutcome::FailedTargetUncertain {
                detail: format!(
                    "workspace mutation renamed but parent durability sync failed: {error}"
                ),
            });
        }
        Ok(PublishOutcome::Published)
    }
}

enum PublishOutcome {
    Published,
    RefusedTargetChanged,
    FailedTargetUnchanged { detail: String },
    FailedTargetUncertain { detail: String },
}

enum TargetSnapshot {
    Absent,
    NotARegularFile,
    PatchPreimageTooLarge,
    Present {
        digest: String,
        patch_bytes: Option<Vec<u8>>,
    },
}

impl EffectExecutor for NativeWorkspaceMutationExecutor {
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        }
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        if call.fencing_epoch != self.trusted_fencing_epoch {
            return Ok(DispatchOutcome::FencedStaleEpoch {
                sink_epoch: self.trusted_fencing_epoch,
            });
        }
        let staged_request = self
            .staged_requests
            .lock()
            .map_err(|_| PortFailure {
                detail: "staged mutation store is poisoned".to_owned(),
            })?
            .get(&call.idempotency_key)
            .cloned();
        let Some(staged_request) = staged_request else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged mutation for idempotency key".to_owned(),
            });
        };
        self.publish_staged_workspace_mutation(call, &staged_request)
    }

    /// Reconcile from the durable key-bound attempt/receipt record.
    ///
    /// Matching target bytes are deliberately insufficient: another writer may
    /// have produced the same postimage, and a completed mutation may later
    /// have been reverted. Only a completed receipt proves that this original
    /// idempotency key executed.
    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let state_guard = self
            .state_store
            .lock_key(MUTATION_STATE_NAMESPACE, idempotency_key)
            .map_err(|error| PortFailure {
                detail: format!("durable mutation state lock failed: {error}"),
            })?;
        let state = state_guard
            .read::<MutationStateRecord>()
            .map_err(|error| PortFailure {
                detail: format!("durable mutation state read failed: {error}"),
            })?;
        let Some(state) = state else {
            return Ok(ExecutorQueryResult::Indeterminate);
        };
        if state.schema != MUTATION_STATE_SCHEMA || state.idempotency_key != idempotency_key {
            return Ok(ExecutorQueryResult::Indeterminate);
        }
        let expected_receipt = format!("tool-receipt://workspace-mutation/{idempotency_key}");
        match state.status {
            MutationAttemptStatus::Completed
                if state.receipt_ref.as_deref() == Some(expected_receipt.as_str()) =>
            {
                return Ok(ExecutorQueryResult::ExecutedWithOriginalKey);
            }
            MutationAttemptStatus::Completed | MutationAttemptStatus::Attempted => {
                return Ok(ExecutorQueryResult::Indeterminate);
            }
            MutationAttemptStatus::NotExecuted => {
                return Ok(ExecutorQueryResult::NotExecuted);
            }
            MutationAttemptStatus::Staged => {}
        }
        let staged_request = self
            .staged_requests
            .lock()
            .map_err(|_| PortFailure {
                detail: "staged mutation store is poisoned".to_owned(),
            })?
            .get(idempotency_key)
            .cloned();
        let Some(staged_request) = staged_request else {
            // A durable `staged` record was written before any attempt marker.
            return Ok(ExecutorQueryResult::NotExecuted);
        };
        let workspace =
            AnchoredWorkspace::open(&staged_request.approved_workspace_root).map_err(|error| {
                PortFailure {
                    detail: format!("workspace root handle open failed: {error}"),
                }
            })?;
        let (parent, target_name) = workspace
            .open_parent(&staged_request.relative_workspace_path)
            .map_err(|error| PortFailure {
                detail: format!("workspace target parent handle open failed: {error}"),
            })?;
        let current = read_target_snapshot(&parent, &target_name, false)?;
        Ok(
            if preimage_matches_snapshot(&staged_request.expected_preimage, &current) {
                ExecutorQueryResult::NotExecuted
            } else {
                ExecutorQueryResult::Indeterminate
            },
        )
    }
}

fn read_target_snapshot(
    parent: &cap_std::fs::Dir,
    target_name: &OsStr,
    retain_patch_preimage: bool,
) -> Result<TargetSnapshot, PortFailure> {
    let mut target = match open_entry_at(parent, target_name).map_err(|error| PortFailure {
        detail: format!("workspace target no-follow open failed: {error}"),
    })? {
        SecureEntry::Absent => return Ok(TargetSnapshot::Absent),
        SecureEntry::Rejected | SecureEntry::Directory(_) => {
            return Ok(TargetSnapshot::NotARegularFile);
        }
        SecureEntry::File(file) => file,
    };
    let metadata = target.metadata().map_err(|error| PortFailure {
        detail: format!("workspace target handle metadata failed: {error}"),
    })?;
    if retain_patch_preimage && metadata.len() > MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES {
        return Ok(TargetSnapshot::PatchPreimageTooLarge);
    }
    target
        .seek(SeekFrom::Start(0))
        .map_err(|error| PortFailure {
            detail: format!("workspace target seek failed: {error}"),
        })?;
    let mut hasher = Sha256::new();
    hasher.update(cognitive_contracts::canonical::DIGEST_PREIMAGE_PREFIX);
    hasher.update(WORKSPACE_IMAGE_DIGEST_DOMAIN.as_bytes());
    hasher.update([0]);
    let mut patch_bytes = retain_patch_preimage.then(Vec::new);
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = target.read(&mut buffer).map_err(|error| PortFailure {
            detail: format!("workspace target read failed: {error}"),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        if let Some(bytes) = &mut patch_bytes {
            if u64::try_from(bytes.len().saturating_add(read)).unwrap_or(u64::MAX)
                > MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES
            {
                return Ok(TargetSnapshot::PatchPreimageTooLarge);
            }
            bytes.extend_from_slice(&buffer[..read]);
        }
    }
    Ok(TargetSnapshot::Present {
        digest: format!("sha256:{:x}", hasher.finalize()),
        patch_bytes,
    })
}

fn preimage_matches_snapshot(
    expected_preimage: &WorkspacePreimage,
    current: &TargetSnapshot,
) -> bool {
    match (expected_preimage, current) {
        (WorkspacePreimage::Absent, TargetSnapshot::Absent) => true,
        (
            WorkspacePreimage::Digest(expected_digest),
            TargetSnapshot::Present {
                digest: current_digest,
                ..
            },
        ) => expected_digest == current_digest,
        _ => false,
    }
}

fn build_postimage(
    staged_request: &StagedWorkspaceMutationRequest,
    preimage: &TargetSnapshot,
) -> Result<Result<Vec<u8>, String>, PortFailure> {
    Ok(match staged_request.family {
        NativeOperationFamily::WorkspaceWrite => Ok(staged_request.payload.clone()),
        NativeOperationFamily::WorkspacePatch => {
            let preimage_bytes = match preimage {
                TargetSnapshot::Absent => Vec::new(),
                TargetSnapshot::Present {
                    patch_bytes: Some(bytes),
                    ..
                } => bytes.clone(),
                TargetSnapshot::PatchPreimageTooLarge => {
                    return Ok(Err(format!(
                        "preimage exceeds the explicit {} byte patch ceiling",
                        MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES
                    )));
                }
                TargetSnapshot::NotARegularFile
                | TargetSnapshot::Present {
                    patch_bytes: None, ..
                } => {
                    return Err(PortFailure {
                        detail: "workspace patch preimage was not retained safely".to_owned(),
                    });
                }
            };
            let preimage_text = String::from_utf8(preimage_bytes).map_err(|_| PortFailure {
                detail: "workspace patch preimage is not valid UTF-8".to_owned(),
            })?;
            let patch_text =
                std::str::from_utf8(&staged_request.payload).map_err(|_| PortFailure {
                    detail: "workspace patch payload is not valid UTF-8".to_owned(),
                })?;
            apply_unified_patch(&preimage_text, patch_text).map(String::into_bytes)
        }
        _ => Err("staged family is not a workspace mutation".to_owned()),
    })
}

impl MutationStateRecord {
    fn staged(
        idempotency_key: &str,
        request: &StagedWorkspaceMutationRequest,
    ) -> Result<Self, NativeToolExecutionError> {
        Ok(Self {
            schema: MUTATION_STATE_SCHEMA.to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            parameters_digest: request.parameters_digest.clone(),
            action: request.action.clone(),
            target: request.target.clone(),
            family: mutation_family_token(request.family).to_owned(),
            expected_preimage: preimage_binding_token(&request.expected_preimage),
            payload_digest: workspace_image_digest(&request.payload)?,
            intended_postimage_digest: request.intended_postimage_digest.clone(),
            status: MutationAttemptStatus::Staged,
            receipt_ref: None,
            redacted_output: None,
        })
    }
}

fn mutation_state_matches(
    record: &MutationStateRecord,
    idempotency_key: &str,
    request: &StagedWorkspaceMutationRequest,
) -> Result<bool, NativeToolExecutionError> {
    Ok(record.schema == MUTATION_STATE_SCHEMA
        && record.idempotency_key == idempotency_key
        && record.parameters_digest == request.parameters_digest
        && record.action == request.action
        && record.target == request.target
        && record.family == mutation_family_token(request.family)
        && record.expected_preimage == preimage_binding_token(&request.expected_preimage)
        && record.payload_digest == workspace_image_digest(&request.payload)?)
}

fn mutation_family_token(family: NativeOperationFamily) -> &'static str {
    match family {
        NativeOperationFamily::WorkspaceWrite => "workspace_write",
        NativeOperationFamily::WorkspacePatch => "workspace_patch",
        _ => "unsupported",
    }
}

fn preimage_binding_token(preimage: &WorkspacePreimage) -> String {
    match preimage {
        WorkspacePreimage::Absent => "absent".to_owned(),
        WorkspacePreimage::Digest(digest) => format!("digest:{digest}"),
    }
}

fn mutation_target_lock_key(parent_identity: FileIdentity, target_name: &OsStr) -> String {
    format!("{parent_identity:?}\0{}", target_name.to_string_lossy())
}

fn staging_file_name(target_name: &OsStr, idempotency_key: &str) -> Result<OsString, PortFailure> {
    let digest = cognitive_contracts::canonical::digest(
        idempotency_key.as_bytes(),
        "native-tool-workspace-staging-key/0.1",
    )
    .map(|digest| digest.trim_start_matches("sha256:").to_owned())
    .map_err(|error| PortFailure {
        detail: format!("workspace staging key digest failed: {error}"),
    })?;
    let target_hint = target_name
        .to_string_lossy()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(24)
        .collect::<String>();
    Ok(OsString::from(format!(
        ".cos-staging-{target_hint}-{digest}"
    )))
}

fn finish_not_executed(
    state_guard: &StateGuard<'_>,
    state: &mut MutationStateRecord,
    reason: &str,
) -> Result<DispatchOutcome, PortFailure> {
    state.status = MutationAttemptStatus::NotExecuted;
    state.receipt_ref = None;
    state.redacted_output = None;
    if let Err(error) = state_guard.write(state) {
        return Ok(DispatchOutcome::Unknown {
            detail: format!("mutation was not executed but durable disposition failed: {error}"),
        });
    }
    Ok(DispatchOutcome::NotExecuted {
        reason: reason.to_owned(),
    })
}

fn cleanup_after_refusal(
    target_parent: &cap_std::fs::Dir,
    staging_name: &OsStr,
) -> Result<PublishOutcome, PortFailure> {
    Ok(match remove_regular_file(target_parent, staging_name) {
        Ok(_) => PublishOutcome::RefusedTargetChanged,
        Err(error) => PublishOutcome::FailedTargetUncertain {
            detail: format!("workspace mutation was refused but staging cleanup failed: {error}"),
        },
    })
}

/// Apply a strict single-file unified diff.
///
/// Every context and removed line must match the preimage exactly at the
/// position the hunk header declares. Anything else — an unknown line prefix, a
/// hunk that runs past the end of the file, overlapping or out-of-order hunks,
/// a declared line count that does not match the body — fails closed with a
/// reason instead of applying a partial patch.
pub(crate) fn apply_unified_patch(preimage: &str, patch: &str) -> Result<String, String> {
    let preimage_lines = split_text_lines(preimage);
    let patch_lines = patch.split_terminator('\n').collect::<Vec<_>>();
    let mut postimage_lines: Vec<TextLine> = Vec::new();
    let mut cursor = 0usize;
    let mut applied_hunks = 0usize;
    let mut patch_index = 0usize;

    while let Some(patch_line) = patch_lines.get(patch_index).copied() {
        if patch_line.starts_with("--- ") || patch_line.starts_with("+++ ") {
            patch_index += 1;
            continue;
        }
        if patch_line.is_empty() {
            patch_index += 1;
            continue;
        }
        if !patch_line.starts_with("@@") {
            return Err(format!("unexpected line outside a hunk: {patch_line}"));
        }
        patch_index += 1;
        let (old_start, old_count, new_count) = parse_hunk_header(patch_line)?;
        let hunk_start = old_start.saturating_sub(1);
        if hunk_start < cursor {
            return Err("hunks overlap or are out of order".to_owned());
        }
        if hunk_start > preimage_lines.len() {
            return Err("hunk starts past the end of the file".to_owned());
        }
        for carried_line in preimage_lines
            .get(cursor..hunk_start)
            .ok_or_else(|| "hunk start is outside the file".to_owned())?
        {
            postimage_lines.push(carried_line.clone());
        }
        cursor = hunk_start;

        let mut consumed_old = 0usize;
        let mut produced_new = 0usize;
        let mut last_body: Option<LastPatchBody> = None;
        while let Some(body_line) = patch_lines.get(patch_index).copied() {
            if body_line.starts_with("@@") {
                break;
            }
            patch_index += 1;
            if body_line.starts_with('\\') {
                if body_line != "\\ No newline at end of file" {
                    return Err(format!("unsupported patch marker: {body_line}"));
                }
                let Some(last_body) = &mut last_body else {
                    return Err("no-newline marker does not follow a hunk line".to_owned());
                };
                if last_body.no_newline_marker {
                    return Err("duplicate no-newline marker".to_owned());
                }
                apply_no_newline_marker(last_body, &preimage_lines, &mut postimage_lines)?;
                last_body.no_newline_marker = true;
                continue;
            }
            finalize_patch_body(last_body.take(), &preimage_lines)?;
            if body_line.is_empty() {
                return Err("bare empty patch line has no unified-diff prefix".to_owned());
            }
            let (marker, text) = body_line.split_at(1);
            match marker {
                " " => {
                    let existing_line = preimage_lines
                        .get(cursor)
                        .ok_or_else(|| "context line runs past the end of the file".to_owned())?;
                    if existing_line.text != text {
                        return Err("context line does not match the preimage".to_owned());
                    }
                    let old_index = cursor;
                    postimage_lines.push(existing_line.clone());
                    let output_index = postimage_lines.len() - 1;
                    cursor += 1;
                    consumed_old += 1;
                    produced_new += 1;
                    last_body = Some(LastPatchBody {
                        kind: PatchBodyKind::Context,
                        old_index: Some(old_index),
                        output_index: Some(output_index),
                        no_newline_marker: false,
                    });
                }
                "-" => {
                    let existing_line = preimage_lines
                        .get(cursor)
                        .ok_or_else(|| "removed line runs past the end of the file".to_owned())?;
                    if existing_line.text != text {
                        return Err("removed line does not match the preimage".to_owned());
                    }
                    let old_index = cursor;
                    cursor += 1;
                    consumed_old += 1;
                    last_body = Some(LastPatchBody {
                        kind: PatchBodyKind::Removal,
                        old_index: Some(old_index),
                        output_index: None,
                        no_newline_marker: false,
                    });
                }
                "+" => {
                    postimage_lines.push(TextLine {
                        text: text.to_owned(),
                        terminated: true,
                    });
                    let output_index = postimage_lines.len() - 1;
                    produced_new += 1;
                    last_body = Some(LastPatchBody {
                        kind: PatchBodyKind::Addition,
                        old_index: None,
                        output_index: Some(output_index),
                        no_newline_marker: false,
                    });
                }
                _ => return Err(format!("unsupported patch line prefix: {marker}")),
            }
        }
        finalize_patch_body(last_body.take(), &preimage_lines)?;
        if consumed_old != old_count || produced_new != new_count {
            return Err(format!(
                "hunk body does not match its header: consumed {consumed_old}/{old_count} old and produced {produced_new}/{new_count} new lines"
            ));
        }
        applied_hunks += 1;
    }

    if applied_hunks == 0 {
        return Err("patch contains no hunk".to_owned());
    }
    for trailing_line in preimage_lines
        .get(cursor..)
        .ok_or_else(|| "patch consumed past the end of the file".to_owned())?
    {
        postimage_lines.push(trailing_line.clone());
    }

    if postimage_lines
        .iter()
        .take(postimage_lines.len().saturating_sub(1))
        .any(|line| !line.terminated)
    {
        return Err("no-newline marker appeared before the final output line".to_owned());
    }
    let mut postimage = String::new();
    for line in postimage_lines {
        postimage.push_str(&line.text);
        if line.terminated {
            postimage.push('\n');
        }
    }
    Ok(postimage)
}

#[derive(Debug, Clone)]
struct TextLine {
    text: String,
    terminated: bool,
}

#[derive(Debug, Clone, Copy)]
enum PatchBodyKind {
    Context,
    Removal,
    Addition,
}

struct LastPatchBody {
    kind: PatchBodyKind,
    old_index: Option<usize>,
    output_index: Option<usize>,
    no_newline_marker: bool,
}

fn split_text_lines(text: &str) -> Vec<TextLine> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split_inclusive('\n')
        .map(|line| {
            let terminated = line.ends_with('\n');
            let text = if terminated {
                line.get(..line.len().saturating_sub(1))
                    .unwrap_or_default()
                    .to_owned()
            } else {
                line.to_owned()
            };
            TextLine { text, terminated }
        })
        .collect()
}

fn apply_no_newline_marker(
    body: &LastPatchBody,
    preimage_lines: &[TextLine],
    postimage_lines: &mut [TextLine],
) -> Result<(), String> {
    if let Some(old_index) = body.old_index {
        let old_line = preimage_lines
            .get(old_index)
            .ok_or_else(|| "no-newline marker references a missing old line".to_owned())?;
        if old_line.terminated {
            return Err("old-side no-newline marker contradicts the preimage".to_owned());
        }
    }
    if let Some(output_index) = body.output_index {
        let output_line = postimage_lines
            .get_mut(output_index)
            .ok_or_else(|| "no-newline marker references a missing new line".to_owned())?;
        output_line.terminated = false;
    }
    Ok(())
}

fn finalize_patch_body(
    body: Option<LastPatchBody>,
    preimage_lines: &[TextLine],
) -> Result<(), String> {
    let Some(body) = body else {
        return Ok(());
    };
    if let Some(old_index) = body.old_index {
        let old_line = preimage_lines
            .get(old_index)
            .ok_or_else(|| "hunk references a missing old line".to_owned())?;
        if !old_line.terminated && !body.no_newline_marker {
            return Err("unterminated old line is missing its no-newline marker".to_owned());
        }
    }
    if matches!(body.kind, PatchBodyKind::Addition)
        && body.no_newline_marker
        && body.output_index.is_none()
    {
        return Err("new-side no-newline marker has no output line".to_owned());
    }
    Ok(())
}

fn parse_hunk_header(header: &str) -> Result<(usize, usize, usize), String> {
    let body = header
        .strip_prefix("@@")
        .and_then(|rest| rest.split("@@").next())
        .ok_or_else(|| format!("malformed hunk header: {header}"))?
        .trim();
    let mut ranges = body.split_whitespace();
    let old_range = ranges
        .next()
        .and_then(|range| range.strip_prefix('-'))
        .ok_or_else(|| format!("malformed hunk header: {header}"))?;
    let new_range = ranges
        .next()
        .and_then(|range| range.strip_prefix('+'))
        .ok_or_else(|| format!("malformed hunk header: {header}"))?;
    if ranges.next().is_some() {
        return Err(format!("malformed hunk header: {header}"));
    }
    let (old_start, old_count) = parse_range(old_range)?;
    let (_, new_count) = parse_range(new_range)?;
    Ok((old_start, old_count, new_count))
}

fn parse_range(range: &str) -> Result<(usize, usize), String> {
    let (start, count) = match range.split_once(',') {
        Some((start, count)) => (start, count),
        None => (range, "1"),
    };
    let start = start
        .parse::<usize>()
        .map_err(|_| format!("malformed hunk range: {range}"))?;
    let count = count
        .parse::<usize>()
        .map_err(|_| format!("malformed hunk range: {range}"))?;
    Ok((start, count))
}

/// Drive an already staged workspace mutation through the durable Effect
/// protocol.
pub(crate) fn dispatch_staged_workspace_mutation_effect<S, C, G>(
    effect_protocol: &EffectProtocol<'_, S, C, G>,
    effect_object_id: &ObjectId,
    expected_effect_version: Version,
    grant: &AuthorizationGrant,
    governance_currency: &GovernanceCurrency,
    executor: &NativeWorkspaceMutationExecutor,
    writer_lease: &WriterLease,
) -> Result<CommittedTransition, EffectError>
where
    S: cognitive_kernel::ports::AuthorityStore + cognitive_kernel::ports::ProtocolStore,
    C: cognitive_kernel::ports::Clock,
    G: cognitive_kernel::ports::IdGenerator,
{
    let authorized = effect_protocol.authorize_effect(
        effect_object_id,
        expected_effect_version,
        grant,
        governance_currency,
        writer_lease,
    )?;
    let (dispatched, outcome) = effect_protocol.dispatch_effect(
        effect_object_id,
        authorized.after_version,
        grant,
        governance_currency,
        executor,
        writer_lease,
    )?;
    effect_protocol.record_outcome(
        effect_object_id,
        dispatched.after_version,
        &outcome,
        writer_lease,
    )
}
