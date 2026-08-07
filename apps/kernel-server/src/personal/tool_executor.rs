//! Daemon-private native Tool execution admission and bounded request shape.
//!
//! This module is intentionally the first execution slice for P2-T06. It
//! does not grant authority and does not replace Intent/Effect persistence;
//! it converts an already daemon-bound Tool descriptor into a request that a
//! later persist-before-dispatch caller can safely execute.

#![allow(unused)] // The next task slice wires this boundary to Effect dispatch.

use cognitive_kernel::tool_registry::{NativeOperationFamily, NativeToolDescriptor, ToolRisk};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

/// Bounded daemon-private input for one native Tool operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeToolExecutionRequest {
    pub descriptor: NativeToolDescriptor,
    pub target: String,
    pub input: Vec<u8>,
    pub workspace_root: Option<PathBuf>,
}

/// Validated execution input. The validated target is still an observation
/// boundary; authority, budget, fencing, and Effect state remain server-side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedNativeToolRequest {
    pub descriptor: NativeToolDescriptor,
    pub target: String,
    pub input: Vec<u8>,
    pub resolved_workspace_path: Option<PathBuf>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum NativeToolExecutionError {
    #[error("descriptor binding is invalid: {0}")]
    InvalidDescriptor(String),
    #[error("tool input exceeds descriptor limit: {observed} > {limit}")]
    InputTooLarge { observed: usize, limit: usize },
    #[error("workspace target is required for this operation")]
    WorkspaceTargetRequired,
    #[error("workspace target escapes the daemon-approved workspace root")]
    WorkspaceTargetEscapesRoot,
    #[error("workspace mutation requires a non-empty input")]
    MutationInputRequired,
    #[error("network target must use HTTPS")]
    NetworkTargetMustUseHttps,
    #[error("network target contains credentials")]
    NetworkTargetContainsCredentials,
    #[error("process target is not a bounded process identifier")]
    InvalidProcessTarget,
}

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

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use cognitive_kernel::tool_registry::{BUILTIN_TOOL_CATALOG, ToolAvailability};

    fn request_for(family: NativeOperationFamily) -> NativeToolExecutionRequest {
        let descriptor = BUILTIN_TOOL_CATALOG
            .iter()
            .find(|descriptor| descriptor.family == family)
            .cloned()
            .unwrap_or_else(|| panic!("catalog family missing: {family:?}"));
        NativeToolExecutionRequest {
            descriptor,
            target: "workspace://notes/today.txt".to_owned(),
            input: b"bounded input".to_vec(),
            workspace_root: Some(PathBuf::from("/tmp/cognitiveos-workspace")),
        }
    }

    #[test]
    fn workspace_target_cannot_escape_approved_root() {
        let mut request = request_for(NativeOperationFamily::WorkspaceRead);
        request.target = "workspace://../secrets.txt".to_owned();
        assert_eq!(
            validate_native_tool_request(&request),
            Err(NativeToolExecutionError::WorkspaceTargetEscapesRoot)
        );
    }

    #[test]
    fn disabled_descriptor_fails_before_execution() {
        let mut request = request_for(NativeOperationFamily::WorkspaceRead);
        request.descriptor.availability = ToolAvailability::Disabled;
        assert!(matches!(
            validate_native_tool_request(&request),
            Err(NativeToolExecutionError::InvalidDescriptor(_))
        ));
    }

    #[test]
    fn mutation_requires_bounded_input() {
        let mut request = request_for(NativeOperationFamily::WorkspaceWrite);
        request.input.clear();
        assert_eq!(
            validate_native_tool_request(&request),
            Err(NativeToolExecutionError::MutationInputRequired)
        );
    }

    #[test]
    fn network_target_rejects_credentials_and_plain_http() {
        let mut request = request_for(NativeOperationFamily::HttpFetchReadOnly);
        request.target = "http://example.test/data".to_owned();
        assert_eq!(
            validate_native_tool_request(&request),
            Err(NativeToolExecutionError::NetworkTargetMustUseHttps)
        );
        request.target = "https://user:secret@example.test/data".to_owned();
        assert_eq!(
            validate_native_tool_request(&request),
            Err(NativeToolExecutionError::NetworkTargetContainsCredentials)
        );
    }
}
