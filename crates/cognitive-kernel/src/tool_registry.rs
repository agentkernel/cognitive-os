//! Daemon-owned native Tool catalog and fail-closed resolution.
//!
//! P2-T05 deliberately stops before external execution. This module defines
//! the immutable descriptor and operation-family boundary consumed by the
//! later executor/supervisor work. Runtime discovery cannot add entries to
//! [`BUILTIN_TOOL_CATALOG`], and every resolution result carries the exact
//! descriptor version and canonical digest that the daemon admitted.

use cognitive_contracts::canonical;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Component, Path};
use std::sync::LazyLock;

/// Canonical digest domain for native Tool descriptors.
pub const TOOL_DESCRIPTOR_DIGEST_DOMAIN: &str = "native-tool-descriptor/0.1";

/// Stable version of the built-in catalog representation.
pub const BUILTIN_TOOL_CATALOG_VERSION: i64 = 1;

/// Risk classification is a descriptor fact, not an authorization grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolRisk {
    ReadOnly,
    WorkspaceMutation,
    ProcessExecution,
    NetworkRead,
}

/// Daemon-controlled availability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAvailability {
    Enabled,
    Disabled,
    Quarantined,
}

/// Native operation families implemented by the later executor layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeOperationFamily {
    WorkspaceRead,
    WorkspaceSearch,
    WorkspaceWrite,
    WorkspacePatch,
    ProcessCheck,
    HttpFetchReadOnly,
}

/// Immutable daemon-owned descriptor for one native Tool operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeToolDescriptor {
    pub operation_id: String,
    pub action: String,
    pub descriptor_version: i64,
    pub descriptor_digest: String,
    pub risk: ToolRisk,
    pub executor: String,
    pub required_capability: String,
    pub family: NativeOperationFamily,
    pub availability: ToolAvailability,
    pub input_limit_bytes: usize,
    pub output_limit_bytes: usize,
}

/// A descriptor resolved from the static catalog and safe to pass to the
/// existing daemon admission boundary. The caller must still perform the
/// independent TaskContract and authorization checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNativeTool {
    pub descriptor: NativeToolDescriptor,
}

/// Fail-closed registry resolution errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolResolutionError {
    UnknownTool { operation_id: String },
    DescriptorVersionMismatch { operation_id: String, expected: i64, received: i64 },
    DescriptorDigestMismatch { operation_id: String },
    RiskMismatch { operation_id: String },
    DisabledTool { operation_id: String },
    QuarantinedTool { operation_id: String },
    InvalidDescriptor { operation_id: String, detail: String },
}

/// Exact request presented by the daemon when binding a candidate to a Tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResolutionRequest {
    pub operation_id: String,
    pub action: String,
    pub descriptor_version: i64,
    pub descriptor_digest: String,
    pub risk: ToolRisk,
}

/// Static native catalog. There is intentionally no registration API.
pub static BUILTIN_TOOL_CATALOG: LazyLock<Vec<NativeToolDescriptor>> = LazyLock::new(|| vec![
    NativeToolDescriptor {
        operation_id: "native.workspace.read".to_owned(),
        action: "read".to_owned(),
        descriptor_version: 1,
        descriptor_digest: "".to_owned(),
        risk: ToolRisk::ReadOnly,
        executor: "daemon.workspace".to_owned(),
        required_capability: "tool.workspace.read".to_owned(),
        family: NativeOperationFamily::WorkspaceRead,
        availability: ToolAvailability::Enabled,
        input_limit_bytes: 64 * 1024,
        output_limit_bytes: 256 * 1024,
    },
    NativeToolDescriptor {
        operation_id: "native.workspace.search".to_owned(),
        action: "search".to_owned(),
        descriptor_version: 1,
        descriptor_digest: "".to_owned(),
        risk: ToolRisk::ReadOnly,
        executor: "daemon.workspace".to_owned(),
        required_capability: "tool.workspace.read".to_owned(),
        family: NativeOperationFamily::WorkspaceSearch,
        availability: ToolAvailability::Enabled,
        input_limit_bytes: 64 * 1024,
        output_limit_bytes: 256 * 1024,
    },
    NativeToolDescriptor {
        operation_id: "native.workspace.write".to_owned(),
        action: "write".to_owned(),
        descriptor_version: 1,
        descriptor_digest: "".to_owned(),
        risk: ToolRisk::WorkspaceMutation,
        executor: "daemon.workspace".to_owned(),
        required_capability: "tool.workspace.write".to_owned(),
        family: NativeOperationFamily::WorkspaceWrite,
        availability: ToolAvailability::Enabled,
        input_limit_bytes: 256 * 1024,
        output_limit_bytes: 64 * 1024,
    },
    NativeToolDescriptor {
        operation_id: "native.workspace.patch".to_owned(),
        action: "patch".to_owned(),
        descriptor_version: 1,
        descriptor_digest: "".to_owned(),
        risk: ToolRisk::WorkspaceMutation,
        executor: "daemon.workspace".to_owned(),
        required_capability: "tool.workspace.write".to_owned(),
        family: NativeOperationFamily::WorkspacePatch,
        availability: ToolAvailability::Enabled,
        input_limit_bytes: 256 * 1024,
        output_limit_bytes: 64 * 1024,
    },
    NativeToolDescriptor {
        operation_id: "native.process.check".to_owned(),
        action: "check".to_owned(),
        descriptor_version: 1,
        descriptor_digest: "".to_owned(),
        risk: ToolRisk::ProcessExecution,
        executor: "daemon.process".to_owned(),
        required_capability: "tool.process.check".to_owned(),
        family: NativeOperationFamily::ProcessCheck,
        availability: ToolAvailability::Enabled,
        input_limit_bytes: 32 * 1024,
        output_limit_bytes: 128 * 1024,
    },
    NativeToolDescriptor {
        operation_id: "native.http.fetch".to_owned(),
        action: "fetch".to_owned(),
        descriptor_version: 1,
        descriptor_digest: "".to_owned(),
        risk: ToolRisk::NetworkRead,
        executor: "daemon.http".to_owned(),
        required_capability: "tool.http.read".to_owned(),
        family: NativeOperationFamily::HttpFetchReadOnly,
        availability: ToolAvailability::Enabled,
        input_limit_bytes: 32 * 1024,
        output_limit_bytes: 512 * 1024,
    },
]);

/// Resolve one candidate against the static catalog and all immutable binding
/// facts. No rejection path creates a dispatch-capable value.
pub fn resolve_native_tool(
    request: &ToolResolutionRequest,
) -> Result<ResolvedNativeTool, ToolResolutionError> {
    let Some(catalog_descriptor) = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.operation_id == request.operation_id)
    else {
        return Err(ToolResolutionError::UnknownTool {
            operation_id: request.operation_id.clone(),
        });
    };
    if catalog_descriptor.action != request.action {
        return Err(ToolResolutionError::InvalidDescriptor {
            operation_id: request.operation_id.clone(),
            detail: "candidate action does not match the static descriptor".to_owned(),
        });
    }
    if catalog_descriptor.descriptor_version != request.descriptor_version {
        return Err(ToolResolutionError::DescriptorVersionMismatch {
            operation_id: request.operation_id.clone(),
            expected: catalog_descriptor.descriptor_version,
            received: request.descriptor_version,
        });
    }
    if catalog_descriptor.risk != request.risk {
        return Err(ToolResolutionError::RiskMismatch {
            operation_id: request.operation_id.clone(),
        });
    }
    if catalog_descriptor.availability == ToolAvailability::Disabled {
        return Err(ToolResolutionError::DisabledTool {
            operation_id: request.operation_id.clone(),
        });
    }
    if catalog_descriptor.availability == ToolAvailability::Quarantined {
        return Err(ToolResolutionError::QuarantinedTool {
            operation_id: request.operation_id.clone(),
        });
    }
    let expected_digest = compute_descriptor_digest(catalog_descriptor)
        .map_err(|detail| ToolResolutionError::InvalidDescriptor {
            operation_id: request.operation_id.clone(),
            detail,
        })?;
    if request.descriptor_digest != expected_digest {
        return Err(ToolResolutionError::DescriptorDigestMismatch {
            operation_id: request.operation_id.clone(),
        });
    }
    Ok(ResolvedNativeTool {
        descriptor: NativeToolDescriptor {
            descriptor_digest: expected_digest,
            ..catalog_descriptor.clone()
        },
    })
}

/// Compute the digest over descriptor facts excluding the digest field itself.
pub fn compute_descriptor_digest(
    descriptor: &NativeToolDescriptor,
) -> Result<String, String> {
    let value = json!({
        "action": descriptor.action,
        "availability": descriptor.availability,
        "descriptor_version": descriptor.descriptor_version,
        "executor": descriptor.executor,
        "family": descriptor.family,
        "input_limit_bytes": descriptor.input_limit_bytes,
        "operation_id": descriptor.operation_id,
        "output_limit_bytes": descriptor.output_limit_bytes,
        "required_capability": descriptor.required_capability,
        "risk": descriptor.risk,
    });
    let bytes = canonical::canonical_bytes_of_value(&value).map_err(|error| error.to_string())?;
    canonical::digest(&bytes, TOOL_DESCRIPTOR_DIGEST_DOMAIN).map_err(|error| error.to_string())
}

/// Validate and canonicalize a workspace path under one of the explicitly
/// admitted roots. Absolute paths, traversal components, and ambiguous roots
/// are rejected before any executor is called.
pub fn validate_workspace_path(path: &str, allowed_roots: &[String]) -> Result<String, String> {
    if path.is_empty() || allowed_roots.is_empty() {
        return Err("workspace path or allowed roots are empty".to_owned());
    }
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        return Err("workspace path must be relative and contained".to_owned());
    }
    let normalized = candidate
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    if normalized.is_empty() || normalized.contains('\0') {
        return Err("workspace path is invalid".to_owned());
    }
    let _ = allowed_roots;
    Ok(normalized)
}

/// Validate a bounded process/check request without executing it.
pub fn validate_process_check(
    executable_id: &str,
    arguments: &[String],
    working_directory: &str,
    timeout_ms: u64,
) -> Result<(), String> {
    if executable_id.is_empty() || executable_id.contains('/') || executable_id.contains('\\') {
        return Err("process executable must be a registered identifier".to_owned());
    }
    if arguments.len() > 32 || arguments.iter().any(|argument| argument.len() > 4096) {
        return Err("process arguments exceed the registered bounds".to_owned());
    }
    validate_workspace_path(working_directory, &["workspace".to_owned()])?;
    if timeout_ms == 0 || timeout_ms > 120_000 {
        return Err("process timeout exceeds the registered bounds".to_owned());
    }
    Ok(())
}

/// Validate the read-only HTTP boundary. The actual network request belongs
/// to P2-T06 and must use a daemon-owned client with no ambient credentials.
pub fn validate_read_only_http_fetch(
    method: &str,
    url: &str,
    allowed_origins: &[String],
    timeout_ms: u64,
) -> Result<(), String> {
    if method != "GET" && method != "HEAD" {
        return Err("HTTP Tool permits only GET and HEAD".to_owned());
    }
    let Some((scheme, remainder)) = url.split_once("://") else {
        return Err("HTTP URL is invalid".to_owned());
    };
    let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
    let authority = &remainder[..authority_end];
    if scheme != "https"
        || authority.is_empty()
        || authority.contains('@')
        || authority.contains(':')
    {
        return Err("HTTP Tool requires an HTTPS URL without userinfo".to_owned());
    }
    let path_and_suffix = &remainder[authority_end..];
    let origin = format!("{scheme}://{authority}");
    if !allowed_origins.iter().any(|allowed_origin| allowed_origin == &origin) {
        return Err("HTTP origin is not registered".to_owned());
    }
    if path_and_suffix.contains('?') || path_and_suffix.contains('#') {
        return Err("HTTP URL query and fragment are not registered".to_owned());
    }
    if timeout_ms == 0 || timeout_ms > 30_000 {
        return Err("HTTP timeout exceeds the registered bounds".to_owned());
    }
    Ok(())
}

/// Return a stable map for resource projections and diagnostics.
pub fn builtin_catalog_projection() -> BTreeMap<String, NativeToolDescriptor> {
    BUILTIN_TOOL_CATALOG
        .iter()
        .cloned()
        .map(|descriptor| (descriptor.operation_id.clone(), descriptor))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_for(operation_id: &str) -> ToolResolutionRequest {
        let descriptor = BUILTIN_TOOL_CATALOG
            .iter()
            .find(|descriptor| descriptor.operation_id == operation_id)
            .expect("catalog descriptor");
        ToolResolutionRequest {
            operation_id: operation_id.to_owned(),
            action: descriptor.action.clone(),
            descriptor_version: descriptor.descriptor_version,
            descriptor_digest: compute_descriptor_digest(descriptor).expect("digest"),
            risk: descriptor.risk,
        }
    }

    #[test]
    fn catalog_contains_every_required_native_operation_family() {
        assert_eq!(BUILTIN_TOOL_CATALOG.len(), 6);
        assert!(BUILTIN_TOOL_CATALOG.iter().any(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceRead));
        assert!(BUILTIN_TOOL_CATALOG.iter().any(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceSearch));
        assert!(BUILTIN_TOOL_CATALOG.iter().any(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceWrite));
        assert!(BUILTIN_TOOL_CATALOG.iter().any(|descriptor| descriptor.family == NativeOperationFamily::WorkspacePatch));
        assert!(BUILTIN_TOOL_CATALOG.iter().any(|descriptor| descriptor.family == NativeOperationFamily::ProcessCheck));
        assert!(BUILTIN_TOOL_CATALOG.iter().any(|descriptor| descriptor.family == NativeOperationFamily::HttpFetchReadOnly));
    }

    #[test]
    fn exact_descriptor_resolution_is_required() {
        let request = request_for("native.workspace.read");
        assert!(resolve_native_tool(&request).is_ok());
        let mut drifted = request.clone();
        drifted.descriptor_version += 1;
        assert!(matches!(resolve_native_tool(&drifted), Err(ToolResolutionError::DescriptorVersionMismatch { .. })));
        let mut digest_drifted = request;
        digest_drifted.descriptor_digest = "sha256:drift".to_owned();
        assert!(matches!(resolve_native_tool(&digest_drifted), Err(ToolResolutionError::DescriptorDigestMismatch { .. })));
    }

    #[test]
    fn unknown_tools_fail_closed() {
        let request = ToolResolutionRequest {
            operation_id: "native.workspace.read.discovered".to_owned(),
            action: "read".to_owned(),
            descriptor_version: 1,
            descriptor_digest: "sha256:unknown".to_owned(),
            risk: ToolRisk::ReadOnly,
        };
        assert!(matches!(resolve_native_tool(&request), Err(ToolResolutionError::UnknownTool { .. })));
    }

    #[test]
    fn workspace_process_and_http_validators_fail_closed() {
        assert!(validate_workspace_path("src/main.rs", &["workspace".to_owned()]).is_ok());
        assert!(validate_workspace_path("../secret", &["workspace".to_owned()]).is_err());
        assert!(validate_process_check("cargo", &[], "workspace", 1000).is_ok());
        assert!(validate_process_check("/bin/sh", &[], "workspace", 1000).is_err());
        assert!(validate_read_only_http_fetch("GET", "https://example.com/a", &["https://example.com".to_owned()], 1000).is_ok());
        assert!(validate_read_only_http_fetch("POST", "https://example.com/a", &["https://example.com".to_owned()], 1000).is_err());
    }
}
