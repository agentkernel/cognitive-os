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
    io::Read,
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
    relative_workspace_path: PathBuf,
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
/// The approved root is held as a directory capability. Every descendant is
/// opened relative to its already-open parent with no-follow semantics and its
/// type is verified from the opened handle, including Windows reparse-point
/// rejection. A file or directory swap cannot redirect the later read.
pub(crate) struct NativeWorkspaceSearchExecutor {
    trusted_fencing_epoch: i64,
    bounds: WorkspaceSearchBounds,
    staged_requests: Mutex<BTreeMap<String, StagedWorkspaceSearchRequest>>,
    completed_searches: Mutex<BTreeMap<String, CompletedWorkspaceSearch>>,
    #[cfg(test)]
    scan_count: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    enumerated_entry_count: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    before_search_hook: Mutex<Option<Box<dyn Fn() + Send>>>,
    #[cfg(test)]
    before_entry_open_hook: Mutex<Option<BeforeEntryOpenHook>>,
}

#[cfg(test)]
type BeforeEntryOpenHook = Box<dyn Fn(&Path) + Send>;

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
            enumerated_entry_count: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            before_search_hook: Mutex::new(None),
            #[cfg(test)]
            before_entry_open_hook: Mutex::new(None),
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
        validate_descriptor(&request.descriptor)?;
        if request.descriptor.family != NativeOperationFamily::WorkspaceSearch {
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
            relative_workspace_path: relative_workspace_path.clone(),
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
    pub(crate) fn enumerated_entry_count(&self) -> usize {
        self.enumerated_entry_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn install_before_search_hook(&self, hook: impl Fn() + Send + 'static) {
        let mut before_search_hook = match self.before_search_hook.lock() {
            Ok(before_search_hook) => before_search_hook,
            Err(poisoned_before_search_hook) => poisoned_before_search_hook.into_inner(),
        };
        *before_search_hook = Some(Box::new(hook));
    }

    #[cfg(test)]
    pub(crate) fn install_before_entry_open_hook(&self, hook: impl Fn(&Path) + Send + 'static) {
        let mut before_entry_open_hook = match self.before_entry_open_hook.lock() {
            Ok(before_entry_open_hook) => before_entry_open_hook,
            Err(poisoned) => poisoned.into_inner(),
        };
        *before_entry_open_hook = Some(Box::new(hook));
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
        let workspace =
            AnchoredWorkspace::open(&staged_request.approved_workspace_root).map_err(|error| {
                PortFailure {
                    detail: format!("workspace root handle open failed: {error}"),
                }
            })?;
        let search_root = match workspace
            .open_entry(&staged_request.relative_workspace_path)
            .map_err(|error| PortFailure {
                detail: format!("workspace search root handle open failed: {error}"),
            })? {
            SecureEntry::File(file) => SecureEntry::File(file),
            SecureEntry::Directory(directory) => SecureEntry::Directory(directory),
            SecureEntry::Absent | SecureEntry::Rejected => {
                return Ok(DispatchOutcome::NotExecuted {
                    reason:
                        "workspace search root is absent, linked, reparsed, or not a regular entry"
                            .to_owned(),
                });
            }
        };
        let mut hooks = WorkspaceSearchHooks {
            #[cfg(test)]
            before_entry_open: self
                .before_entry_open_hook
                .lock()
                .map_err(|_| PortFailure {
                    detail: "before-entry-open hook store is poisoned".to_owned(),
                })?
                .take(),
        };
        let scan = scan_contained_workspace_tree(
            search_root,
            &staged_request.relative_workspace_path,
            &staged_request.query,
            &self.bounds,
            staged_request.output_limit_bytes,
            &mut hooks,
        )?;
        #[cfg(test)]
        self.enumerated_entry_count
            .store(scan.enumerated_entries, std::sync::atomic::Ordering::SeqCst);
        let redacted_output = redact_sensitive_output(&scan.rendered_matches)
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

/// Walk a subtree through opened directory handles and render bounded matches.
///
/// Enumeration itself consumes the visit budget: a huge directory is never
/// collected in full merely so it can later be truncated. Every child is
/// opened no-follow relative to its already-open parent and its type is checked
/// from the resulting handle before reading or descending.
fn scan_contained_workspace_tree(
    search_root: SecureEntry,
    relative_search_root: &Path,
    query: &str,
    bounds: &WorkspaceSearchBounds,
    output_limit_bytes: usize,
    hooks: &mut WorkspaceSearchHooks,
) -> Result<WorkspaceScanResult, PortFailure> {
    let mut state = WorkspaceScanState {
        query,
        bounds,
        output_limit_bytes,
        visited_entries: 0,
        enumerated_entries: 0,
        retained_matches: 0,
        rendered_matches: String::new(),
        hooks,
    };
    scan_secure_entry(search_root, relative_search_root, true, &mut state)?;
    Ok(WorkspaceScanResult {
        rendered_matches: state.rendered_matches,
        enumerated_entries: state.enumerated_entries,
    })
}

struct WorkspaceSearchHooks {
    #[cfg(test)]
    before_entry_open: Option<BeforeEntryOpenHook>,
}

impl WorkspaceSearchHooks {
    fn before_entry_open(&mut self, relative_path: &Path) {
        #[cfg(test)]
        if let Some(hook) = self.before_entry_open.take() {
            hook(relative_path);
        }
        #[cfg(not(test))]
        let _ = relative_path;
    }
}

struct WorkspaceScanState<'a> {
    query: &'a str,
    bounds: &'a WorkspaceSearchBounds,
    output_limit_bytes: usize,
    visited_entries: usize,
    enumerated_entries: usize,
    retained_matches: usize,
    rendered_matches: String,
    hooks: &'a mut WorkspaceSearchHooks,
}

struct WorkspaceScanResult {
    rendered_matches: String,
    enumerated_entries: usize,
}

fn scan_secure_entry(
    entry: SecureEntry,
    relative_path: &Path,
    count_visit: bool,
    state: &mut WorkspaceScanState<'_>,
) -> Result<(), PortFailure> {
    if scan_is_complete(state)
        || (count_visit && state.visited_entries >= state.bounds.maximum_visited_entries)
    {
        return Ok(());
    }
    if count_visit {
        state.visited_entries += 1;
    }
    match entry {
        SecureEntry::Absent | SecureEntry::Rejected => Ok(()),
        SecureEntry::File(mut file) => {
            let metadata = file.metadata().map_err(|error| PortFailure {
                detail: format!("workspace search file metadata failed: {error}"),
            })?;
            if metadata.len() > state.bounds.maximum_file_bytes {
                return Ok(());
            }
            let mut file_bytes = Vec::new();
            file.by_ref()
                .take(state.bounds.maximum_file_bytes.saturating_add(1))
                .read_to_end(&mut file_bytes)
                .map_err(|error| PortFailure {
                    detail: format!("workspace search file read failed: {error}"),
                })?;
            if u64::try_from(file_bytes.len()).unwrap_or(u64::MAX) > state.bounds.maximum_file_bytes
            {
                return Ok(());
            }
            let rendered_path = relative_path.to_string_lossy().replace('\\', "/");
            for (line_index, line) in String::from_utf8_lossy(&file_bytes).lines().enumerate() {
                if scan_is_complete(state) {
                    break;
                }
                if !line.contains(state.query) {
                    continue;
                }
                state.retained_matches += 1;
                state.rendered_matches.push_str(&format!(
                    "{rendered_path}:{}:{}\n",
                    line_index + 1,
                    bounded_prefix(line, state.bounds.maximum_line_bytes)
                ));
            }
            Ok(())
        }
        SecureEntry::Directory(directory) => scan_secure_directory(directory, relative_path, state),
    }
}

fn scan_secure_directory(
    directory: cap_std::fs::Dir,
    relative_path: &Path,
    state: &mut WorkspaceScanState<'_>,
) -> Result<(), PortFailure> {
    let initial_remaining = state
        .bounds
        .maximum_visited_entries
        .saturating_sub(state.visited_entries);
    if initial_remaining == 0 || scan_is_complete(state) {
        return Ok(());
    }
    let mut names = Vec::with_capacity(initial_remaining.min(256));
    let mut entries = match directory.entries() {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    while state.visited_entries < state.bounds.maximum_visited_entries {
        let Some(next_entry) = entries.next() else {
            break;
        };
        state.enumerated_entries += 1;
        state.visited_entries += 1;
        let Ok(next_entry) = next_entry else {
            continue;
        };
        names.push(next_entry.file_name());
    }
    names.sort();
    for name in names {
        if scan_is_complete(state) {
            break;
        }
        let child_path = relative_path.join(&name);
        state.hooks.before_entry_open(&child_path);
        let child = match open_entry_at(&directory, &name) {
            Ok(child) => child,
            Err(_) => continue,
        };
        scan_secure_entry(child, &child_path, false, state)?;
    }
    Ok(())
}

fn scan_is_complete(state: &WorkspaceScanState<'_>) -> bool {
    state.retained_matches >= state.bounds.maximum_matches
        || state.rendered_matches.len() >= state.output_limit_bytes
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
