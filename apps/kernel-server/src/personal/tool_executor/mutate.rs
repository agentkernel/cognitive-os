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
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedWorkspaceMutationRequest {
    parameters_digest: String,
    action: String,
    target: String,
    family: NativeOperationFamily,
    approved_workspace_root: PathBuf,
    resolved_workspace_path: PathBuf,
    expected_preimage: WorkspacePreimage,
    payload: Vec<u8>,
    /// Known before dispatch for a whole-file write; a patch's postimage
    /// depends on the preimage bytes and is resolved during dispatch.
    intended_postimage_digest: Option<String>,
    output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedWorkspaceMutation {
    receipt_ref: String,
    redacted_output: Vec<u8>,
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
/// **A genuinely queryable sink.** `query_outcome` does not rely on process
/// memory. It re-reads the target and compares it against the digests bound at
/// staging, so an OUTCOME_UNKNOWN Effect can be reconciled from durable
/// filesystem state — including by a fresh executor after a restart.
///
/// Retained output is a bounded receipt line: target, action, byte count and
/// postimage digest. File content is never retained, because a mutation
/// payload is the least appropriate thing to echo into an Effect receipt.
pub(crate) struct NativeWorkspaceMutationExecutor {
    trusted_fencing_epoch: i64,
    staged_requests: Mutex<BTreeMap<String, StagedWorkspaceMutationRequest>>,
    resolved_postimage_digests: Mutex<BTreeMap<String, String>>,
    completed_mutations: Mutex<BTreeMap<String, CompletedWorkspaceMutation>>,
    #[cfg(test)]
    publish_count: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    after_staging_write_hook: Mutex<Option<AfterStagingWriteHook>>,
}

/// Test seam invoked with `(target, staging_path)` once the postimage is fully
/// written but before the rename publishes it.
#[cfg(test)]
type AfterStagingWriteHook = Box<dyn Fn(&Path, &Path) + Send>;

impl NativeWorkspaceMutationExecutor {
    pub(crate) fn new(trusted_fencing_epoch: i64) -> Self {
        Self {
            trusted_fencing_epoch,
            staged_requests: Mutex::new(BTreeMap::new()),
            resolved_postimage_digests: Mutex::new(BTreeMap::new()),
            completed_mutations: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            publish_count: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            after_staging_write_hook: Mutex::new(None),
        }
    }

    /// Bind one validated mutation to its durable Intent identity.
    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        request: &ValidatedNativeToolRequest,
    ) -> Result<(), NativeToolExecutionError> {
        if !matches!(
            request.descriptor.family,
            NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch
        ) {
            return Err(NativeToolExecutionError::UnsupportedExecutionFamily);
        }
        let resolved_workspace_path = request
            .resolved_workspace_path
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
            resolved_workspace_path: resolved_workspace_path.clone(),
            expected_preimage: expected_preimage.clone(),
            payload: request.input.clone(),
            intended_postimage_digest,
            output_limit_bytes: request.descriptor.output_limit_bytes,
        };
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
        self.completed_mutations
            .lock()
            .ok()
            .and_then(|completed| completed.get(idempotency_key).cloned())
            .map(|completed| completed.redacted_output)
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
        let Some(contained_target) = self.resolve_contained_target(staged_request)? else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "workspace mutation target is not contained in the approved root"
                    .to_owned(),
            });
        };

        // Hold the completed-result ledger across the whole publish so two
        // concurrent calls for one idempotency key cannot both write.
        let mut completed_mutations = self.completed_mutations.lock().map_err(|_| PortFailure {
            detail: "completed mutation store is poisoned".to_owned(),
        })?;
        if let Some(existing) = completed_mutations.get(&call.idempotency_key) {
            return Ok(DispatchOutcome::Executed {
                receipt_ref: existing.receipt_ref.clone(),
            });
        }

        let preimage_bytes = match read_target_bytes(&contained_target)? {
            TargetState::Absent => None,
            TargetState::NotARegularFile => {
                return Ok(DispatchOutcome::NotExecuted {
                    reason: "workspace mutation target is a link or not a regular file".to_owned(),
                });
            }
            TargetState::Present(bytes) => Some(bytes),
        };
        if !preimage_matches(&staged_request.expected_preimage, preimage_bytes.as_deref())? {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "workspace mutation preimage does not match the staged expectation"
                    .to_owned(),
            });
        }

        let postimage_bytes = match staged_request.family {
            NativeOperationFamily::WorkspaceWrite => staged_request.payload.clone(),
            NativeOperationFamily::WorkspacePatch => {
                let preimage_text = String::from_utf8(preimage_bytes.clone().unwrap_or_default())
                    .map_err(|_| PortFailure {
                    detail: "workspace patch preimage is not valid UTF-8".to_owned(),
                })?;
                let patch_text =
                    std::str::from_utf8(&staged_request.payload).map_err(|_| PortFailure {
                        detail: "workspace patch payload is not valid UTF-8".to_owned(),
                    })?;
                match apply_unified_patch(&preimage_text, patch_text) {
                    Ok(postimage_text) => postimage_text.into_bytes(),
                    Err(reason) => {
                        return Ok(DispatchOutcome::NotExecuted {
                            reason: format!("workspace patch does not apply: {reason}"),
                        });
                    }
                }
            }
            _ => {
                return Ok(DispatchOutcome::NotExecuted {
                    reason: "staged family is not a workspace mutation".to_owned(),
                });
            }
        };
        let postimage_digest =
            workspace_image_digest(&postimage_bytes).map_err(|error| PortFailure {
                detail: format!("workspace postimage digest failed: {error}"),
            })?;
        self.resolved_postimage_digests
            .lock()
            .map_err(|_| PortFailure {
                detail: "resolved postimage store is poisoned".to_owned(),
            })?
            .insert(call.idempotency_key.clone(), postimage_digest.clone());

        match self.publish_atomically(
            &contained_target,
            &postimage_bytes,
            staged_request,
            &call.idempotency_key,
        )? {
            PublishOutcome::Published => {}
            PublishOutcome::RefusedTargetChanged => {
                return Ok(DispatchOutcome::NotExecuted {
                    reason: "workspace mutation target changed before publication".to_owned(),
                });
            }
            PublishOutcome::FailedTargetUnchanged { detail } => {
                return Ok(DispatchOutcome::NotExecuted {
                    reason: format!("workspace mutation publication failed: {detail}"),
                });
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
        completed_mutations.insert(
            call.idempotency_key.clone(),
            CompletedWorkspaceMutation {
                receipt_ref: receipt_ref.clone(),
                redacted_output,
            },
        );
        Ok(DispatchOutcome::Executed { receipt_ref })
    }

    /// Resolve the target's parent inside the canonicalized approved root.
    ///
    /// The target itself may legitimately not exist yet, so containment is
    /// proven on the parent directory, which must exist and canonicalize under
    /// the approved root.
    fn resolve_contained_target(
        &self,
        staged_request: &StagedWorkspaceMutationRequest,
    ) -> Result<Option<PathBuf>, PortFailure> {
        let canonical_workspace_root =
            std::fs::canonicalize(&staged_request.approved_workspace_root).map_err(|error| {
                PortFailure {
                    detail: format!("workspace root resolution failed: {error}"),
                }
            })?;
        let Some(target_parent) = staged_request.resolved_workspace_path.parent() else {
            return Ok(None);
        };
        let Some(target_name) = staged_request.resolved_workspace_path.file_name() else {
            return Ok(None);
        };
        let Ok(canonical_parent) = std::fs::canonicalize(target_parent) else {
            return Ok(None);
        };
        if !canonical_parent.starts_with(&canonical_workspace_root) {
            return Ok(None);
        }
        Ok(Some(canonical_parent.join(target_name)))
    }

    fn publish_atomically(
        &self,
        contained_target: &Path,
        postimage_bytes: &[u8],
        staged_request: &StagedWorkspaceMutationRequest,
        idempotency_key: &str,
    ) -> Result<PublishOutcome, PortFailure> {
        let Some(target_parent) = contained_target.parent() else {
            return Ok(PublishOutcome::FailedTargetUnchanged {
                detail: "target has no parent directory".to_owned(),
            });
        };
        let staging_path = target_parent.join(format!(
            ".{}.{}.cos-staging",
            contained_target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            sanitize_key_for_file_name(idempotency_key)
        ));
        if let Err(error) = write_and_sync(&staging_path, postimage_bytes) {
            let _ = std::fs::remove_file(&staging_path);
            return Ok(PublishOutcome::FailedTargetUnchanged {
                detail: error.to_string(),
            });
        }

        #[cfg(test)]
        {
            let after_staging_write_hook = self
                .after_staging_write_hook
                .lock()
                .map_err(|_| PortFailure {
                    detail: "after-staging-write hook store is poisoned".to_owned(),
                })?
                .take();
            if let Some(after_staging_write_hook) = after_staging_write_hook {
                after_staging_write_hook(contained_target, &staging_path);
            }
        }

        // Re-verify the preimage immediately before the rename. Between the
        // first check and here the sink built a postimage, so a concurrent
        // writer must not be silently overwritten.
        let current_state = read_target_bytes(contained_target)?;
        let current_bytes = match current_state {
            TargetState::Absent => None,
            TargetState::NotARegularFile => {
                let _ = std::fs::remove_file(&staging_path);
                return Ok(PublishOutcome::RefusedTargetChanged);
            }
            TargetState::Present(bytes) => Some(bytes),
        };
        if !preimage_matches(&staged_request.expected_preimage, current_bytes.as_deref())? {
            let _ = std::fs::remove_file(&staging_path);
            return Ok(PublishOutcome::RefusedTargetChanged);
        }

        #[cfg(test)]
        self.publish_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Err(error) = std::fs::rename(&staging_path, contained_target) {
            let _ = std::fs::remove_file(&staging_path);
            // A failed rename leaves the target untouched on every supported
            // platform, but say so from observation rather than assumption.
            return Ok(match read_target_bytes(contained_target)? {
                TargetState::Present(bytes)
                    if preimage_matches(&staged_request.expected_preimage, Some(&bytes))
                        .unwrap_or(false) =>
                {
                    PublishOutcome::FailedTargetUnchanged {
                        detail: error.to_string(),
                    }
                }
                TargetState::Absent
                    if matches!(staged_request.expected_preimage, WorkspacePreimage::Absent) =>
                {
                    PublishOutcome::FailedTargetUnchanged {
                        detail: error.to_string(),
                    }
                }
                _ => PublishOutcome::FailedTargetUncertain {
                    detail: format!("workspace mutation publication is uncertain: {error}"),
                },
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

enum TargetState {
    Absent,
    NotARegularFile,
    Present(Vec<u8>),
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

    /// Reconcile from durable filesystem state, not from process memory.
    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        if self
            .completed_mutations
            .lock()
            .map_err(|_| PortFailure {
                detail: "completed mutation store is poisoned".to_owned(),
            })?
            .contains_key(idempotency_key)
        {
            return Ok(ExecutorQueryResult::ExecutedWithOriginalKey);
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
            return Ok(ExecutorQueryResult::Indeterminate);
        };
        let Some(contained_target) = self.resolve_contained_target(&staged_request)? else {
            return Ok(ExecutorQueryResult::Indeterminate);
        };
        let expected_postimage_digest =
            staged_request
                .intended_postimage_digest
                .clone()
                .or_else(|| {
                    self.resolved_postimage_digests
                        .lock()
                        .ok()
                        .and_then(|resolved| resolved.get(idempotency_key).cloned())
                });
        let current_bytes = match read_target_bytes(&contained_target)? {
            TargetState::Present(bytes) => Some(bytes),
            TargetState::Absent => None,
            TargetState::NotARegularFile => return Ok(ExecutorQueryResult::Indeterminate),
        };
        if let (Some(expected_postimage_digest), Some(current_bytes)) =
            (expected_postimage_digest.as_ref(), current_bytes.as_deref())
        {
            let current_digest =
                workspace_image_digest(current_bytes).map_err(|error| PortFailure {
                    detail: format!("workspace image digest failed: {error}"),
                })?;
            if &current_digest == expected_postimage_digest {
                return Ok(ExecutorQueryResult::ExecutedWithOriginalKey);
            }
        }
        if preimage_matches(&staged_request.expected_preimage, current_bytes.as_deref())? {
            return Ok(ExecutorQueryResult::NotExecuted);
        }
        Ok(ExecutorQueryResult::Indeterminate)
    }
}

fn read_target_bytes(target: &Path) -> Result<TargetState, PortFailure> {
    match std::fs::symlink_metadata(target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(TargetState::Absent),
        Err(error) => Err(PortFailure {
            detail: format!("workspace target inspection failed: {error}"),
        }),
        Ok(metadata) => {
            if !metadata.is_file() {
                return Ok(TargetState::NotARegularFile);
            }
            std::fs::read(target)
                .map(TargetState::Present)
                .map_err(|error| PortFailure {
                    detail: format!("workspace target read failed: {error}"),
                })
        }
    }
}

fn preimage_matches(
    expected_preimage: &WorkspacePreimage,
    current_bytes: Option<&[u8]>,
) -> Result<bool, PortFailure> {
    Ok(match (expected_preimage, current_bytes) {
        (WorkspacePreimage::Absent, None) => true,
        (WorkspacePreimage::Absent, Some(_)) | (WorkspacePreimage::Digest(_), None) => false,
        (WorkspacePreimage::Digest(expected_digest), Some(current_bytes)) => {
            let current_digest =
                workspace_image_digest(current_bytes).map_err(|error| PortFailure {
                    detail: format!("workspace image digest failed: {error}"),
                })?;
            &current_digest == expected_digest
        }
    })
}

fn write_and_sync(staging_path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut staging_file = std::fs::File::create(staging_path)?;
    staging_file.write_all(bytes)?;
    staging_file.flush()?;
    staging_file.sync_all()
}

/// Keep the staging file name inside the same directory and free of separators.
fn sanitize_key_for_file_name(idempotency_key: &str) -> String {
    idempotency_key
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// Apply a strict single-file unified diff.
///
/// Every context and removed line must match the preimage exactly at the
/// position the hunk header declares. Anything else — an unknown line prefix, a
/// hunk that runs past the end of the file, overlapping or out-of-order hunks,
/// a declared line count that does not match the body — fails closed with a
/// reason instead of applying a partial patch.
fn apply_unified_patch(preimage: &str, patch: &str) -> Result<String, String> {
    let ends_with_newline = preimage.ends_with('\n');
    let preimage_lines: Vec<&str> = if preimage.is_empty() {
        Vec::new()
    } else {
        let body = if ends_with_newline {
            preimage.get(..preimage.len() - 1).unwrap_or_default()
        } else {
            preimage
        };
        body.split('\n').collect()
    };

    let mut postimage_lines: Vec<String> = Vec::new();
    let mut cursor = 0usize;
    let mut applied_hunks = 0usize;
    let mut patch_lines = patch.split('\n').peekable();

    while let Some(patch_line) = patch_lines.next() {
        if patch_line.starts_with("--- ") || patch_line.starts_with("+++ ") {
            continue;
        }
        if patch_line.is_empty() {
            continue;
        }
        if !patch_line.starts_with("@@") {
            return Err(format!("unexpected line outside a hunk: {patch_line}"));
        }
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
            postimage_lines.push((*carried_line).to_owned());
        }
        cursor = hunk_start;

        let mut consumed_old = 0usize;
        let mut produced_new = 0usize;
        while let Some(body_line) = patch_lines.peek() {
            if body_line.starts_with("@@") {
                break;
            }
            let body_line = patch_lines.next().unwrap_or_default();
            if body_line.starts_with('\\') {
                continue;
            }
            if body_line.is_empty() {
                // A bare empty line in a unified diff is an empty context line
                // only while the hunk still owes old lines; otherwise it is
                // trailing whitespace after the hunk body.
                if consumed_old >= old_count {
                    continue;
                }
                let existing_line = preimage_lines
                    .get(cursor)
                    .ok_or_else(|| "context line runs past the end of the file".to_owned())?;
                if !existing_line.is_empty() {
                    return Err("context line does not match the preimage".to_owned());
                }
                postimage_lines.push(String::new());
                cursor += 1;
                consumed_old += 1;
                produced_new += 1;
                continue;
            }
            let (marker, text) = body_line.split_at(1);
            match marker {
                " " => {
                    let existing_line = preimage_lines
                        .get(cursor)
                        .ok_or_else(|| "context line runs past the end of the file".to_owned())?;
                    if *existing_line != text {
                        return Err("context line does not match the preimage".to_owned());
                    }
                    postimage_lines.push(text.to_owned());
                    cursor += 1;
                    consumed_old += 1;
                    produced_new += 1;
                }
                "-" => {
                    let existing_line = preimage_lines
                        .get(cursor)
                        .ok_or_else(|| "removed line runs past the end of the file".to_owned())?;
                    if *existing_line != text {
                        return Err("removed line does not match the preimage".to_owned());
                    }
                    cursor += 1;
                    consumed_old += 1;
                }
                "+" => {
                    postimage_lines.push(text.to_owned());
                    produced_new += 1;
                }
                _ => return Err(format!("unsupported patch line prefix: {marker}")),
            }
        }
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
        postimage_lines.push((*trailing_line).to_owned());
    }

    let mut postimage = postimage_lines.join("\n");
    if (ends_with_newline || preimage.is_empty()) && !postimage.is_empty() {
        postimage.push('\n');
    }
    Ok(postimage)
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
