#![allow(unused, unused_imports)]

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{
    BUILTIN_TOOL_CATALOG, NativeOperationFamily, NativeToolDescriptor, ToolRisk,
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
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;

use super::*;

pub(crate) fn validate_native_tool_request(
    request: &NativeToolExecutionRequest,
) -> Result<ValidatedNativeToolRequest, NativeToolExecutionError> {
    validate_descriptor(&request.descriptor)?;
    if request.input.len() > request.descriptor.input_limit_bytes {
        return Err(NativeToolExecutionError::InputTooLarge {
            observed: request.input.len(),
            limit: request.descriptor.input_limit_bytes,
        });
    }

    let (resolved_workspace_path, relative_workspace_path) = match request.descriptor.family {
        NativeOperationFamily::WorkspaceRead
        | NativeOperationFamily::WorkspaceSearch
        | NativeOperationFamily::WorkspaceWrite
        | NativeOperationFamily::WorkspacePatch => {
            let (resolved, relative) = validate_workspace_target(
                &request.target,
                request.workspace_root.as_deref(),
                request.descriptor.risk,
                request.input.is_empty(),
            )?;
            (Some(resolved), Some(relative))
        }
        NativeOperationFamily::ProcessCheck => {
            validate_process_target(&request.target)?;
            (None, None)
        }
        NativeOperationFamily::HttpFetchReadOnly => {
            validate_network_target(&request.target)?;
            (None, None)
        }
        NativeOperationFamily::RegisteredCheckRun => {
            let check_id = request.target.strip_prefix("check://").ok_or_else(|| {
                NativeToolExecutionError::InvalidDescriptor(
                    "registered check target must be check://<check_id>".to_owned(),
                )
            })?;
            if check_id.is_empty() || request.workspace_root.is_none() {
                return Err(NativeToolExecutionError::WorkspaceTargetRequired);
            }
            (None, None)
        }
    };

    // A mutation that cannot name the state it replaces has no compare-and-swap
    // guard, so it can silently clobber a concurrently changed target. Refuse
    // to validate it rather than leaving the check to the sink.
    if matches!(
        request.descriptor.family,
        NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch
    ) && request.expected_preimage.is_none()
    {
        return Err(NativeToolExecutionError::MutationPreimageRequired);
    }

    Ok(ValidatedNativeToolRequest {
        descriptor: request.descriptor.clone(),
        target: request.target.clone(),
        input: request.input.clone(),
        approved_workspace_root: request.workspace_root.clone(),
        resolved_workspace_path,
        relative_workspace_path,
        expected_preimage: request.expected_preimage.clone(),
    })
}

pub(crate) fn validate_descriptor(
    descriptor: &NativeToolDescriptor,
) -> Result<(), NativeToolExecutionError> {
    if descriptor.operation_id.is_empty()
        || descriptor.action.is_empty()
        || descriptor.executor.is_empty()
        || descriptor.descriptor_version < 1
        || descriptor.descriptor_digest.is_empty()
    {
        return Err(NativeToolExecutionError::InvalidDescriptor(
            "descriptor identity and digest are required".to_owned(),
        ));
    }
    if !matches!(
        descriptor.availability,
        cognitive_kernel::tool_registry::ToolAvailability::Enabled
    ) {
        return Err(NativeToolExecutionError::InvalidDescriptor(
            "disabled or quarantined descriptor cannot execute".to_owned(),
        ));
    }
    let Some(catalog_descriptor) = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|candidate| candidate.operation_id == descriptor.operation_id)
    else {
        return Err(NativeToolExecutionError::InvalidDescriptor(
            "descriptor is not present in the immutable native Tool catalog".to_owned(),
        ));
    };
    if catalog_descriptor != descriptor {
        return Err(NativeToolExecutionError::InvalidDescriptor(
            "descriptor drifted from the immutable native Tool catalog".to_owned(),
        ));
    }
    Ok(())
}

fn validate_workspace_target(
    target: &str,
    workspace_root: Option<&Path>,
    risk: ToolRisk,
    input_is_empty: bool,
) -> Result<(PathBuf, PathBuf), NativeToolExecutionError> {
    let root = workspace_root.ok_or(NativeToolExecutionError::WorkspaceTargetRequired)?;
    let relative_target = target
        .strip_prefix("workspace://")
        .ok_or(NativeToolExecutionError::WorkspaceTargetEscapesRoot)?;
    let relative_path = Path::new(relative_target);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(NativeToolExecutionError::WorkspaceTargetEscapesRoot);
    }
    if matches!(risk, ToolRisk::WorkspaceMutation) && input_is_empty {
        return Err(NativeToolExecutionError::MutationInputRequired);
    }
    Ok((root.join(relative_path), relative_path.to_path_buf()))
}

fn validate_process_target(target: &str) -> Result<(), NativeToolExecutionError> {
    let process_id = target
        .strip_prefix("process://")
        .ok_or(NativeToolExecutionError::InvalidProcessTarget)?;
    if process_id.is_empty() || !process_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(NativeToolExecutionError::InvalidProcessTarget);
    }
    Ok(())
}

fn validate_network_target(target: &str) -> Result<(), NativeToolExecutionError> {
    let authority = target
        .strip_prefix("https://")
        .ok_or(NativeToolExecutionError::NetworkTargetMustUseHttps)?
        .split_once('/')
        .map_or(
            target.strip_prefix("https://").unwrap_or_default(),
            |(host, _)| host,
        );
    if authority.is_empty() || authority.contains('@') {
        return Err(NativeToolExecutionError::NetworkTargetContainsCredentials);
    }
    Ok(())
}
