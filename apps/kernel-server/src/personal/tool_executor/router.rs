use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cognitive_domain::ObjectId;
use cognitive_kernel::effects::{EffectClass, OperationDescriptor};
use cognitive_kernel::executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
use cognitive_kernel::ports::{
    DaemonOperationDescriptorRow, PortFailure, WorkerAuthorizationStore,
};
use cognitive_kernel::tool_registry::{
    BUILTIN_TOOL_CATALOG, NativeOperationFamily, NativeToolDescriptor, ToolRisk,
};
use cognitive_provider_transport::RustlsReadOnlyFetchTransport;

use crate::personal::scheduler_authority::ResolvedNativeWorkerDispatch;

use super::{
    DaemonProcessSupervisor, DurableExecutorStateStore, FailClosedProcessObservationSource,
    NativeHttpFetchReadOnlyExecutor, NativeProcessCheckExecutor, NativeToolExecutionError,
    NativeToolExecutionRequest, NativeWorkspaceMutationExecutor, NativeWorkspaceReadExecutor,
    NativeWorkspaceSearchExecutor, WorkspacePreimage, validate_native_tool_request,
};

const NATIVE_DESCRIPTOR_IDS: [(&str, &str); 6] = [
    (
        "native.workspace.read",
        "00000000-0000-7000-8000-000000002001",
    ),
    (
        "native.workspace.search",
        "00000000-0000-7000-8000-000000002002",
    ),
    (
        "native.workspace.write",
        "00000000-0000-7000-8000-000000002003",
    ),
    (
        "native.workspace.patch",
        "00000000-0000-7000-8000-000000002004",
    ),
    (
        "native.process.check",
        "00000000-0000-7000-8000-000000002005",
    ),
    ("native.http.fetch", "00000000-0000-7000-8000-000000002006"),
];

pub(crate) fn builtin_native_descriptor_id(
    operation_id: &str,
) -> Result<ObjectId, NativeToolExecutionError> {
    let raw_id = NATIVE_DESCRIPTOR_IDS
        .iter()
        .find_map(|(candidate, identifier)| (*candidate == operation_id).then_some(*identifier))
        .ok_or_else(|| {
            NativeToolExecutionError::InvalidDescriptor(format!(
                "native operation is not in the immutable descriptor identity map: {operation_id}"
            ))
        })?;
    ObjectId::parse(raw_id)
        .map_err(|error| NativeToolExecutionError::InvalidDescriptor(error.to_string()))
}

fn persisted_operation_descriptor(descriptor: &NativeToolDescriptor) -> OperationDescriptor {
    let effect_class = match descriptor.risk {
        ToolRisk::ReadOnly | ToolRisk::NetworkRead => EffectClass::Pure,
        ToolRisk::WorkspaceMutation => EffectClass::GovernedExternal,
        ToolRisk::ProcessExecution => EffectClass::LocalEphemeral,
    };
    OperationDescriptor {
        operation_id: descriptor.operation_id.clone(),
        action: descriptor.action.clone(),
        effect_class,
        executor: descriptor.executor.clone(),
        capabilities: ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        },
        descriptor_version: descriptor.descriptor_version,
    }
}

pub(crate) fn ensure_builtin_native_descriptors<S>(
    store: &S,
) -> Result<(), NativeToolExecutionError>
where
    S: WorkerAuthorizationStore,
{
    for descriptor in BUILTIN_TOOL_CATALOG.iter() {
        let descriptor_id = builtin_native_descriptor_id(&descriptor.operation_id)?;
        let persisted = persisted_operation_descriptor(descriptor);
        let canonical_json = serde_json::json!({
            "descriptor_id": descriptor_id.as_str(),
            "operation_id": persisted.operation_id,
            "action": persisted.action,
            "effect_class": match persisted.effect_class {
                EffectClass::Pure => "pure",
                EffectClass::LocalEphemeral => "local_ephemeral",
                EffectClass::GovernedExternal => "governed_external",
                EffectClass::EmergencySafety => "emergency_safety",
            },
            "executor": persisted.executor,
            "queryable": persisted.capabilities.queryable,
            "idempotent": persisted.capabilities.idempotent,
            "descriptor_version": persisted.descriptor_version,
        })
        .to_string();
        let row = DaemonOperationDescriptorRow {
            descriptor_id: descriptor_id.clone(),
            descriptor: persisted,
            canonical_json,
        };
        match store
            .load_daemon_operation_descriptor(&descriptor_id)
            .map_err(|error| NativeToolExecutionError::ExecutorUnavailable(error.to_string()))?
        {
            Some(existing) if existing == row => {}
            Some(_) => {
                return Err(NativeToolExecutionError::InvalidDescriptor(format!(
                    "persisted native descriptor identity conflicts with catalog: {}",
                    descriptor.operation_id
                )));
            }
            None => store
                .append_daemon_operation_descriptor(&row)
                .map_err(|error| {
                    NativeToolExecutionError::ExecutorUnavailable(error.to_string())
                })?,
        }
    }
    Ok(())
}

/// Composition-root router for daemon-staged native Tool requests.
///
/// The production callers support WorkspaceRead, WorkspaceSearch,
/// WorkspaceWrite/Patch, ProcessCheck, and HttpFetchReadOnly. ProcessCheck and
/// HttpFetchReadOnly dispatch through fail-closed carriers until the daemon's
/// supervised-process registry or a registered origin set is wired; the router
/// never invents input from a digest.
pub(crate) struct ProductionNativeToolExecutorRouter {
    workspace_root: PathBuf,
    workspace_read: NativeWorkspaceReadExecutor,
    workspace_search: NativeWorkspaceSearchExecutor,
    workspace_mutation: NativeWorkspaceMutationExecutor,
    process_check:
        NativeProcessCheckExecutor<DaemonProcessSupervisor<FailClosedProcessObservationSource>>,
    http_fetch: NativeHttpFetchReadOnlyExecutor<RustlsReadOnlyFetchTransport>,
    staged_families: Mutex<BTreeMap<String, NativeOperationFamily>>,
}

impl ProductionNativeToolExecutorRouter {
    pub(crate) fn open(
        trusted_fencing_epoch: i64,
        workspace_root: PathBuf,
    ) -> Result<Self, NativeToolExecutionError> {
        std::fs::create_dir_all(&workspace_root).map_err(|error| {
            NativeToolExecutionError::ExecutorUnavailable(format!(
                "create daemon-approved workspace root: {error}"
            ))
        })?;
        let mutation_state_store = Arc::new(
            DurableExecutorStateStore::open(&executor_state_path(&workspace_root)).map_err(
                |error| {
                    NativeToolExecutionError::ExecutorUnavailable(format!(
                        "open durable executor state store: {error}"
                    ))
                },
            )?,
        );
        let process_supervisor = Arc::new(DaemonProcessSupervisor::fail_closed(
            trusted_fencing_epoch,
            Duration::from_secs(30),
            1024 * 1024,
        ));
        Ok(Self {
            workspace_root,
            workspace_read: NativeWorkspaceReadExecutor::new(trusted_fencing_epoch),
            workspace_search: NativeWorkspaceSearchExecutor::new(trusted_fencing_epoch),
            workspace_mutation: NativeWorkspaceMutationExecutor::new(
                trusted_fencing_epoch,
                mutation_state_store.clone(),
            ),
            process_check: NativeProcessCheckExecutor::new(
                trusted_fencing_epoch,
                process_supervisor,
                Duration::from_secs(30),
            ),
            http_fetch: NativeHttpFetchReadOnlyExecutor::new(
                trusted_fencing_epoch,
                Arc::new(RustlsReadOnlyFetchTransport::default()),
                Vec::new(),
                30_000,
                mutation_state_store,
            ),
            staged_families: Mutex::new(BTreeMap::new()),
        })
    }

    pub(crate) fn stage_resolved(
        &self,
        resolved: &ResolvedNativeWorkerDispatch,
    ) -> Result<(), NativeToolExecutionError> {
        let family = resolved.native_tool.descriptor.family;
        let (input, expected_preimage) = match family {
            NativeOperationFamily::WorkspaceRead
            | NativeOperationFamily::ProcessCheck
            | NativeOperationFamily::HttpFetchReadOnly => (Vec::new(), None),
            NativeOperationFamily::WorkspaceSearch => (
                workspace_search_query(&resolved.intent.canonical_json)?,
                None,
            ),
            NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch => {
                let (payload, preimage) =
                    workspace_mutation_parameters(&resolved.intent.canonical_json)?;
                (payload, Some(preimage))
            }
            _ => return Err(NativeToolExecutionError::UnsupportedExecutionFamily),
        };
        let workspace_root = match family {
            NativeOperationFamily::WorkspaceRead
            | NativeOperationFamily::WorkspaceSearch
            | NativeOperationFamily::WorkspaceWrite
            | NativeOperationFamily::WorkspacePatch => Some(self.workspace_root.clone()),
            NativeOperationFamily::ProcessCheck | NativeOperationFamily::HttpFetchReadOnly => None,
            _ => return Err(NativeToolExecutionError::UnsupportedExecutionFamily),
        };
        let request = validate_native_tool_request(&NativeToolExecutionRequest {
            descriptor: resolved.native_tool.descriptor.clone(),
            target: resolved.candidate.target.clone(),
            input,
            workspace_root,
            expected_preimage,
        })?;
        match family {
            NativeOperationFamily::WorkspaceRead => self.workspace_read.stage_request(
                resolved.intent.idempotency_key.clone(),
                resolved.intent.parameters_digest.clone(),
                &request,
            )?,
            NativeOperationFamily::WorkspaceSearch => self.workspace_search.stage_request(
                resolved.intent.idempotency_key.clone(),
                resolved.intent.parameters_digest.clone(),
                &request,
            )?,
            NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch => {
                self.workspace_mutation.stage_request(
                    resolved.intent.idempotency_key.clone(),
                    resolved.intent.parameters_digest.clone(),
                    &request,
                )?
            }
            NativeOperationFamily::ProcessCheck => self.process_check.stage_request(
                resolved.intent.idempotency_key.clone(),
                resolved.intent.parameters_digest.clone(),
                &request,
            )?,
            NativeOperationFamily::HttpFetchReadOnly => self.http_fetch.stage_request(
                resolved.intent.idempotency_key.clone(),
                resolved.intent.parameters_digest.clone(),
                &request,
            )?,
            _ => return Err(NativeToolExecutionError::UnsupportedExecutionFamily),
        }
        let mut staged_families = self.staged_families.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "native executor routing table is poisoned".to_owned(),
            )
        })?;
        if let Some(existing_family) = staged_families.get(&resolved.intent.idempotency_key) {
            if *existing_family != family {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_families.insert(resolved.intent.idempotency_key.clone(), family);
        Ok(())
    }

    fn staged_family(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<NativeOperationFamily>, PortFailure> {
        self.staged_families
            .lock()
            .map_err(|_| PortFailure {
                detail: "native executor routing table is poisoned".to_owned(),
            })
            .map(|families| families.get(idempotency_key).copied())
    }

    #[cfg(test)]
    pub(crate) fn install_workspace_read_before_io_hook(&self, hook: impl Fn() + Send + 'static) {
        self.workspace_read.install_before_read_hook(hook);
    }
}

impl EffectExecutor for ProductionNativeToolExecutorRouter {
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        }
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        match self.staged_family(&call.idempotency_key)? {
            Some(NativeOperationFamily::WorkspaceRead) => self.workspace_read.dispatch(call),
            Some(NativeOperationFamily::WorkspaceSearch) => self.workspace_search.dispatch(call),
            Some(NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch) => {
                self.workspace_mutation.dispatch(call)
            }
            Some(NativeOperationFamily::ProcessCheck) => self.process_check.dispatch(call),
            Some(NativeOperationFamily::HttpFetchReadOnly) => self.http_fetch.dispatch(call),
            Some(_) => Ok(DispatchOutcome::NotExecuted {
                reason: "native family has no production request carrier".to_owned(),
            }),
            None => Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged native request for idempotency key".to_owned(),
            }),
        }
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        match self.staged_family(idempotency_key)? {
            Some(NativeOperationFamily::WorkspaceRead) => {
                self.workspace_read.query_outcome(idempotency_key)
            }
            Some(NativeOperationFamily::WorkspaceSearch) => {
                self.workspace_search.query_outcome(idempotency_key)
            }
            Some(NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch) => {
                self.workspace_mutation.query_outcome(idempotency_key)
            }
            Some(NativeOperationFamily::ProcessCheck) => {
                self.process_check.query_outcome(idempotency_key)
            }
            Some(NativeOperationFamily::HttpFetchReadOnly) => {
                self.http_fetch.query_outcome(idempotency_key)
            }
            Some(_) | None => Ok(ExecutorQueryResult::NotExecuted),
        }
    }
}

/// Extract the governed search query from a persisted intent's canonical JSON.
///
/// The query is the `parameters.query` string of the intent value. A missing,
/// non-string, or unparseable query fails closed before any executor staging.
fn workspace_search_query(canonical_json: &str) -> Result<Vec<u8>, NativeToolExecutionError> {
    let value: serde_json::Value = serde_json::from_str(canonical_json).map_err(|error| {
        NativeToolExecutionError::InvalidDescriptor(format!(
            "intent canonical JSON is not parseable: {error}"
        ))
    })?;
    let query = value
        .get("parameters")
        .and_then(|parameters| parameters.get("query"))
        .and_then(|query| query.as_str())
        .ok_or_else(|| {
            NativeToolExecutionError::InvalidDescriptor(
                "workspace search query is missing from the governed intent parameters".to_owned(),
            )
        })?;
    Ok(query.as_bytes().to_vec())
}

/// Extract the governed mutation payload and preimage from a persisted intent's
/// canonical JSON.
///
/// The payload is `parameters.input_b64` (standard base64) and the preimage is
/// `parameters.preimage`, either `absent` or `digest:<sha256>`. Missing,
/// malformed, or unparseable parameters fail closed before any executor staging.
fn workspace_mutation_parameters(
    canonical_json: &str,
) -> Result<(Vec<u8>, WorkspacePreimage), NativeToolExecutionError> {
    let value: serde_json::Value = serde_json::from_str(canonical_json).map_err(|error| {
        NativeToolExecutionError::InvalidDescriptor(format!(
            "intent canonical JSON is not parseable: {error}"
        ))
    })?;
    let parameters = value.get("parameters").ok_or_else(|| {
        NativeToolExecutionError::InvalidDescriptor(
            "workspace mutation parameters are missing from the governed intent".to_owned(),
        )
    })?;
    let input_b64 = parameters
        .get("input_b64")
        .and_then(|input| input.as_str())
        .ok_or_else(|| {
            NativeToolExecutionError::InvalidDescriptor(
                "workspace mutation payload is missing from the governed intent parameters"
                    .to_owned(),
            )
        })?;
    let input = STANDARD.decode(input_b64).map_err(|error| {
        NativeToolExecutionError::InvalidDescriptor(format!(
            "workspace mutation payload is not valid base64: {error}"
        ))
    })?;
    let preimage = match parameters
        .get("preimage")
        .and_then(|preimage| preimage.as_str())
    {
        Some("absent") => WorkspacePreimage::Absent,
        Some(digest) if let Some(rest) = digest.strip_prefix("digest:") => {
            WorkspacePreimage::Digest(rest.to_owned())
        }
        _ => {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "workspace mutation preimage must be `absent` or `digest:<sha256>`".to_owned(),
            ));
        }
    };
    Ok((input, preimage))
}

/// Durable executor state lives outside the approved workspace so a mutation can
/// never write over its own attempt/receipt store.
fn executor_state_path(workspace_root: &Path) -> PathBuf {
    let parent = workspace_root.parent().unwrap_or_else(|| Path::new("."));
    let name = workspace_root
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("workspace"))
        .to_string_lossy();
    parent.join(format!(".{name}-executor-state"))
}
