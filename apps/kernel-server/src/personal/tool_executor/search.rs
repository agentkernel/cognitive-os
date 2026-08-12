#![allow(unused, unused_imports)]

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{
    MAXIMUM_WORKSPACE_SEARCH_QUERY_BYTES, NativeOperationFamily, NativeToolDescriptor, ToolRisk,
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
    path::{Path, PathBuf},
    sync::Mutex,
};

use super::*;

/// Fixed ceilings for one bounded workspace scan.
///
/// A search reads an unbounded number of files unless something stops it, so
/// every dimension that can grow is capped here rather than in the caller:
/// how many filesystem entries may be visited, how many matches may be
/// retained, how large a single file may be before it is skipped, and how much
/// of one matching line may be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkspaceSearchBounds {
    pub(crate) maximum_visited_entries: usize,
    pub(crate) maximum_matches: usize,
    pub(crate) maximum_file_bytes: u64,
    pub(crate) maximum_line_bytes: usize,
}

impl Default for WorkspaceSearchBounds {
    fn default() -> Self {
        Self {
            maximum_visited_entries: 4096,
            maximum_matches: 256,
            maximum_file_bytes: 1024 * 1024,
            maximum_line_bytes: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedWorkspaceSearchRequest {
    parameters_digest: String,
    target: String,
    approved_workspace_root: PathBuf,
    resolved_workspace_path: PathBuf,
    query: String,
    output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedWorkspaceSearch {
    receipt_ref: String,
    redacted_output: Vec<u8>,
}

/// Daemon-private, read-only workspace search sink.
///
/// It mirrors the `WorkspaceRead` boundary exactly: staging binds the request
/// to the sealed Intent's idempotency key and parameter digest, the Effect
/// protocol records `EXECUTING` before this adapter touches the filesystem,
/// and only bounded redacted bytes are retained under the original key. It has
/// no Task, progress, evidence, or completion input, so a search result can
/// never be mistaken for a Task outcome.
///
/// Containment is enforced twice. The staged search root is canonicalized and
/// must remain under the canonicalized approved root, and every entry reached
/// during the walk is inspected with `symlink_metadata` and skipped when it is
/// a symbolic link. The scan therefore cannot leave the approved root through
/// a link planted inside it.
pub(crate) struct NativeWorkspaceSearchExecutor {
    trusted_fencing_epoch: i64,
    bounds: WorkspaceSearchBounds,
    staged_requests: Mutex<BTreeMap<String, StagedWorkspaceSearchRequest>>,
    completed_searches: Mutex<BTreeMap<String, CompletedWorkspaceSearch>>,
    #[cfg(test)]
    scan_count: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    before_search_hook: Mutex<Option<Box<dyn Fn() + Send>>>,
}

impl NativeWorkspaceSearchExecutor {
    pub(crate) fn new(trusted_fencing_epoch: i64) -> Self {
        Self::with_bounds(trusted_fencing_epoch, WorkspaceSearchBounds::default())
    }

    pub(crate) fn with_bounds(trusted_fencing_epoch: i64, bounds: WorkspaceSearchBounds) -> Self {
        Self {
            trusted_fencing_epoch,
            bounds,
            staged_requests: Mutex::new(BTreeMap::new()),
            completed_searches: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            scan_count: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            before_search_hook: Mutex::new(None),
        }
    }

    /// Bind one validated workspace-search request to its durable Intent
    /// identity. The query arrives as the request's bounded input and is held
    /// here so a later dispatch cannot substitute a different one.
    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        request: &ValidatedNativeToolRequest,
    ) -> Result<(), NativeToolExecutionError> {
        if request.descriptor.family != NativeOperationFamily::WorkspaceSearch {
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
        if idempotency_key.is_empty() || parameters_digest.is_empty() {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "idempotency key and parameters digest are required".to_owned(),
            ));
        }
        let query = String::from_utf8(request.input.clone()).map_err(|_| {
            NativeToolExecutionError::InvalidDescriptor(
                "workspace search query is not valid UTF-8".to_owned(),
            )
        })?;
        // The registered pre-executor validator fixes the same bound; repeat it
        // at the sink so a caller that skipped validation cannot widen it.
        if query.is_empty() || query.len() > MAXIMUM_WORKSPACE_SEARCH_QUERY_BYTES {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "workspace search query exceeds the registered bounds".to_owned(),
            ));
        }
        let staged_request = StagedWorkspaceSearchRequest {
            parameters_digest,
            target: request.target.clone(),
            approved_workspace_root: approved_workspace_root.clone(),
            resolved_workspace_path: resolved_workspace_path.clone(),
            query,
            output_limit_bytes: request.descriptor.output_limit_bytes,
        };
        let mut staged_requests = self.staged_requests.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "staged search store is poisoned".to_owned(),
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
        self.completed_searches
            .lock()
            .ok()
            .and_then(|completed_searches| completed_searches.get(idempotency_key).cloned())
            .map(|completed_search| completed_search.redacted_output)
    }

    #[cfg(test)]
    pub(crate) fn scan_count(&self) -> usize {
        self.scan_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn install_before_search_hook(&self, hook: impl Fn() + Send + 'static) {
        let mut before_search_hook = match self.before_search_hook.lock() {
            Ok(before_search_hook) => before_search_hook,
            Err(poisoned_before_search_hook) => poisoned_before_search_hook.into_inner(),
        };
        *before_search_hook = Some(Box::new(hook));
    }

    fn search_staged_workspace_tree(
        &self,
        call: &ExecutorCall,
        staged_request: &StagedWorkspaceSearchRequest,
    ) -> Result<DispatchOutcome, PortFailure> {
        if call.action != "search"
            || call.target != staged_request.target
            || call.parameters_digest != staged_request.parameters_digest
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "dispatch does not match the daemon-staged workspace search".to_owned(),
            });
        }
        let canonical_workspace_root =
            std::fs::canonicalize(&staged_request.approved_workspace_root).map_err(|error| {
                PortFailure {
                    detail: format!("workspace root resolution failed: {error}"),
                }
            })?;
        let canonical_search_root = std::fs::canonicalize(&staged_request.resolved_workspace_path)
            .map_err(|error| PortFailure {
                detail: format!("workspace search root resolution failed: {error}"),
            })?;
        if !canonical_search_root.starts_with(&canonical_workspace_root) {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "workspace search root escaped the approved root after resolution"
                    .to_owned(),
            });
        }
        // Hold the completed-result ledger lock across the scan so concurrent
        // calls for one idempotency key cannot both execute it.
        let mut completed_searches = self.completed_searches.lock().map_err(|_| PortFailure {
            detail: "completed search store is poisoned".to_owned(),
        })?;
        if let Some(existing_search) = completed_searches.get(&call.idempotency_key) {
            return Ok(DispatchOutcome::Executed {
                receipt_ref: existing_search.receipt_ref.clone(),
            });
        }
        #[cfg(test)]
        let before_search_hook = self
            .before_search_hook
            .lock()
            .map_err(|_| PortFailure {
                detail: "before-search hook store is poisoned".to_owned(),
            })?
            .take();
        #[cfg(test)]
        if let Some(before_search_hook) = before_search_hook {
            before_search_hook();
        }
        #[cfg(test)]
        self.scan_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let rendered_matches = scan_contained_workspace_tree(
            &canonical_workspace_root,
            &canonical_search_root,
            &staged_request.query,
            &self.bounds,
            staged_request.output_limit_bytes,
        )?;
        let redacted_output = redact_sensitive_output(&rendered_matches)
            .into_bytes()
            .into_iter()
            .take(staged_request.output_limit_bytes)
            .collect::<Vec<_>>();
        let receipt_ref = format!("tool-receipt://workspace-search/{}", call.idempotency_key);
        completed_searches.insert(
            call.idempotency_key.clone(),
            CompletedWorkspaceSearch {
                receipt_ref: receipt_ref.clone(),
                redacted_output,
            },
        );
        Ok(DispatchOutcome::Executed { receipt_ref })
    }
}

impl EffectExecutor for NativeWorkspaceSearchExecutor {
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
                detail: "staged search store is poisoned".to_owned(),
            })?
            .get(&call.idempotency_key)
            .cloned();
        let Some(staged_request) = staged_request else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged search for idempotency key".to_owned(),
            });
        };
        self.search_staged_workspace_tree(call, &staged_request)
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let completed_searches = self.completed_searches.lock().map_err(|_| PortFailure {
            detail: "completed search store is poisoned".to_owned(),
        })?;
        Ok(if completed_searches.contains_key(idempotency_key) {
            ExecutorQueryResult::ExecutedWithOriginalKey
        } else {
            ExecutorQueryResult::NotExecuted
        })
    }
}

/// Walk a canonicalized subtree in a deterministic order and render bounded
/// matches. Symbolic links are never traversed and never opened, so the walk
/// cannot leave `canonical_workspace_root`.
fn scan_contained_workspace_tree(
    canonical_workspace_root: &Path,
    canonical_search_root: &Path,
    query: &str,
    bounds: &WorkspaceSearchBounds,
    output_limit_bytes: usize,
) -> Result<String, PortFailure> {
    let mut pending_entries = vec![canonical_search_root.to_path_buf()];
    let mut visited_entries = 0usize;
    let mut retained_matches = 0usize;
    let mut rendered_matches = String::new();

    while let Some(current_entry) = pending_entries.pop() {
        if visited_entries >= bounds.maximum_visited_entries
            || retained_matches >= bounds.maximum_matches
            || rendered_matches.len() >= output_limit_bytes
        {
            break;
        }
        visited_entries += 1;
        let entry_metadata = match std::fs::symlink_metadata(&current_entry) {
            Ok(entry_metadata) => entry_metadata,
            // A concurrently removed entry is not a search failure; the scan
            // reports what it could actually read.
            Err(_) => continue,
        };
        if entry_metadata.file_type().is_symlink() {
            continue;
        }
        if entry_metadata.is_dir() {
            let mut child_entries = Vec::new();
            let read_directory = match std::fs::read_dir(&current_entry) {
                Ok(read_directory) => read_directory,
                Err(_) => continue,
            };
            for child_entry in read_directory {
                let Ok(child_entry) = child_entry else {
                    continue;
                };
                child_entries.push(child_entry.path());
            }
            // Sort ascending, then reverse: the stack pops from the end, so
            // this yields a stable lexicographic visit order.
            child_entries.sort();
            child_entries.reverse();
            pending_entries.extend(child_entries);
            continue;
        }
        if !entry_metadata.is_file() || entry_metadata.len() > bounds.maximum_file_bytes {
            continue;
        }
        let Ok(file_bytes) = std::fs::read(&current_entry) else {
            continue;
        };
        let relative_entry = current_entry
            .strip_prefix(canonical_workspace_root)
            .unwrap_or(&current_entry)
            .to_string_lossy()
            .replace('\\', "/");
        for (line_index, line) in String::from_utf8_lossy(&file_bytes).lines().enumerate() {
            if retained_matches >= bounds.maximum_matches
                || rendered_matches.len() >= output_limit_bytes
            {
                break;
            }
            if !line.contains(query) {
                continue;
            }
            retained_matches += 1;
            rendered_matches.push_str(&format!(
                "{relative_entry}:{}:{}\n",
                line_index + 1,
                bounded_prefix(line, bounds.maximum_line_bytes)
            ));
        }
    }

    Ok(rendered_matches)
}

/// Truncate to at most `maximum_bytes`, never splitting a UTF-8 character.
fn bounded_prefix(value: &str, maximum_bytes: usize) -> &str {
    if value.len() <= maximum_bytes {
        return value;
    }
    let mut boundary = maximum_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.get(..boundary).unwrap_or_default()
}

/// Drive an already staged workspace search through the durable Effect
/// protocol. Staging binds the native request to the Intent's idempotency key
/// and parameter digest; this is the only adapter path that may invoke it.
pub(crate) fn dispatch_staged_workspace_search_effect<S, C, G>(
    effect_protocol: &EffectProtocol<'_, S, C, G>,
    effect_object_id: &ObjectId,
    expected_effect_version: Version,
    grant: &AuthorizationGrant,
    governance_currency: &GovernanceCurrency,
    executor: &NativeWorkspaceSearchExecutor,
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
