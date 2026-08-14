use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

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

use crate::personal::scheduler_authority::ResolvedNativeWorkerDispatch;

use super::{
    NativeToolExecutionError, NativeToolExecutionRequest, NativeWorkspaceReadExecutor,
    NativeWorkspaceSearchExecutor, validate_native_tool_request,
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
/// The first production caller supports the parameter-free WorkspaceRead
/// family. Families that need a separately governed payload or preimage remain
/// fail-closed until such an immutable carrier exists; the router never invents
/// input from a digest.
pub(crate) struct ProductionNativeToolExecutorRouter {
    workspace_root: PathBuf,
    workspace_read: NativeWorkspaceReadExecutor,
    workspace_search: NativeWorkspaceSearchExecutor,
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
        Ok(Self {
            workspace_root,
            workspace_read: NativeWorkspaceReadExecutor::new(trusted_fencing_epoch),
            workspace_search: NativeWorkspaceSearchExecutor::new(trusted_fencing_epoch),
            staged_families: Mutex::new(BTreeMap::new()),
        })
    }

    pub(crate) fn stage_resolved(
        &self,
        resolved: &ResolvedNativeWorkerDispatch,
    ) -> Result<(), NativeToolExecutionError> {
        let family = resolved.native_tool.descriptor.family;
        let input = match family {
            NativeOperationFamily::WorkspaceRead => Vec::new(),
            NativeOperationFamily::WorkspaceSearch => {
                workspace_search_query(&resolved.intent.canonical_json)?
            }
            _ => return Err(NativeToolExecutionError::UnsupportedExecutionFamily),
        };
        let request = validate_native_tool_request(&NativeToolExecutionRequest {
            descriptor: resolved.native_tool.descriptor.clone(),
            target: resolved.candidate.target.clone(),
            input,
            workspace_root: Some(self.workspace_root.clone()),
            expected_preimage: None,
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
