#![allow(unused, unused_imports)]

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{NativeOperationFamily, NativeToolDescriptor, ToolRisk};
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

    let resolved_workspace_path = match request.descriptor.family {
        NativeOperationFamily::WorkspaceRead
        | NativeOperationFamily::WorkspaceSearch
        | NativeOperationFamily::WorkspaceWrite
        | NativeOperationFamily::WorkspacePatch => Some(validate_workspace_target(
            &request.target,
            request.workspace_root.as_deref(),
            request.descriptor.risk,
            request.input.is_empty(),
        )?),
        NativeOperationFamily::ProcessCheck => {
            validate_process_target(&request.target)?;
            None
        }
        NativeOperationFamily::HttpFetchReadOnly => {
            validate_network_target(&request.target)?;
            None
        }
    };

    Ok(ValidatedNativeToolRequest {
        descriptor: request.descriptor.clone(),
        target: request.target.clone(),
        input: request.input.clone(),
        approved_workspace_root: request.workspace_root.clone(),
        resolved_workspace_path,
    })
}

fn validate_descriptor(descriptor: &NativeToolDescriptor) -> Result<(), NativeToolExecutionError> {
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
    Ok(())
}

fn validate_workspace_target(
    target: &str,
    workspace_root: Option<&Path>,
    risk: ToolRisk,
    input_is_empty: bool,
) -> Result<PathBuf, NativeToolExecutionError> {
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
    Ok(root.join(relative_path))
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
