//! Daemon-owned native Tool catalog and fail-closed resolution.
//!
//! P2-T05 deliberately stops before external execution. This module defines
//! the immutable descriptor and operation-family boundary consumed by the
//! later executor/supervisor work. Runtime discovery cannot add entries to
//! [`BUILTIN_TOOL_CATALOG`], and every resolution result carries the exact
//! descriptor version and canonical digest that the daemon admitted.

use crate::effects::{EffectClass, OperationDescriptor};
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
    UnknownTool {
        operation_id: String,
    },
    DescriptorVersionMismatch {
        operation_id: String,
        expected: i64,
        received: i64,
    },
    DescriptorDigestMismatch {
        operation_id: String,
    },
    RiskMismatch {
        operation_id: String,
    },
    DisabledTool {
        operation_id: String,
    },
    QuarantinedTool {
        operation_id: String,
    },
    InvalidDescriptor {
        operation_id: String,
        detail: String,
    },
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
pub static BUILTIN_TOOL_CATALOG: LazyLock<Vec<NativeToolDescriptor>> = LazyLock::new(|| {
    let mut catalog = vec![
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
    ];
    for descriptor in &mut catalog {
        descriptor.descriptor_digest = compute_descriptor_digest(descriptor)
            .expect("built-in Tool descriptor must have a canonical digest");
    }
    catalog
});

/// Resolve one candidate against the static catalog and all immutable binding
/// facts. No rejection path creates a dispatch-capable value.
pub fn resolve_native_tool(
    request: &ToolResolutionRequest,
) -> Result<ResolvedNativeTool, ToolResolutionError> {
    resolve_native_tool_from_catalog(&BUILTIN_TOOL_CATALOG, request)
}

/// Resolve against an explicit immutable catalog slice. Production always
/// supplies [`BUILTIN_TOOL_CATALOG`]; keeping the lookup pure lets tests prove
/// that disabled and quarantined descriptors cannot become dispatch-capable.
fn resolve_native_tool_from_catalog(
    catalog: &[NativeToolDescriptor],
    request: &ToolResolutionRequest,
) -> Result<ResolvedNativeTool, ToolResolutionError> {
    let Some(catalog_descriptor) = catalog
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
    let expected_digest = compute_descriptor_digest(catalog_descriptor).map_err(|detail| {
        ToolResolutionError::InvalidDescriptor {
            operation_id: request.operation_id.clone(),
            detail,
        }
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

/// Bind an already-persisted daemon descriptor to the static native catalog.
/// This is the integration form used by admission: it prevents a native Tool
/// from silently changing executor, effect class, or recovery capabilities
/// while retaining the existing descriptor table as the durable source.
pub fn resolve_persisted_native_descriptor(
    descriptor: &OperationDescriptor,
) -> Result<ResolvedNativeTool, ToolResolutionError> {
    let Some(catalog_descriptor) = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|catalog| catalog.operation_id == descriptor.operation_id)
    else {
        return Err(ToolResolutionError::UnknownTool {
            operation_id: descriptor.operation_id.clone(),
        });
    };
    if catalog_descriptor.action != descriptor.action
        || catalog_descriptor.descriptor_version != descriptor.descriptor_version
        || catalog_descriptor.executor != descriptor.executor
    {
        return Err(ToolResolutionError::InvalidDescriptor {
            operation_id: descriptor.operation_id.clone(),
            detail: "persisted descriptor drifted from the native catalog".to_owned(),
        });
    }
    let recovery_capabilities_match = match catalog_descriptor.risk {
        ToolRisk::ReadOnly | ToolRisk::NetworkRead => {
            descriptor.effect_class == EffectClass::Pure
                && descriptor.capabilities.queryable
                && descriptor.capabilities.idempotent
        }
        ToolRisk::WorkspaceMutation => {
            descriptor.effect_class == EffectClass::GovernedExternal
                && descriptor.capabilities.queryable
        }
        ToolRisk::ProcessExecution => {
            descriptor.effect_class == EffectClass::LocalEphemeral
                && descriptor.capabilities.queryable
        }
    };
    if !recovery_capabilities_match {
        return Err(ToolResolutionError::InvalidDescriptor {
            operation_id: descriptor.operation_id.clone(),
            detail: "persisted descriptor recovery or effect facts drifted".to_owned(),
        });
    }
    if catalog_descriptor.availability != ToolAvailability::Enabled {
        return match catalog_descriptor.availability {
            ToolAvailability::Disabled => Err(ToolResolutionError::DisabledTool {
                operation_id: descriptor.operation_id.clone(),
            }),
            ToolAvailability::Quarantined => Err(ToolResolutionError::QuarantinedTool {
                operation_id: descriptor.operation_id.clone(),
            }),
            ToolAvailability::Enabled => unreachable!(),
        };
    }
    let descriptor_digest = compute_descriptor_digest(catalog_descriptor).map_err(|detail| {
        ToolResolutionError::InvalidDescriptor {
            operation_id: descriptor.operation_id.clone(),
            detail,
        }
    })?;
    Ok(ResolvedNativeTool {
        descriptor: NativeToolDescriptor {
            descriptor_digest,
            ..catalog_descriptor.clone()
        },
    })
}

/// Compute the digest over descriptor facts excluding the digest field itself.
pub fn compute_descriptor_digest(descriptor: &NativeToolDescriptor) -> Result<String, String> {
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
    if allowed_roots
        .iter()
        .any(|root| !is_workspace_root_identifier(root))
        || allowed_roots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != allowed_roots.len()
    {
        return Err("workspace roots are invalid or ambiguous".to_owned());
    }
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
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
    let root = normalized
        .split('/')
        .next()
        .expect("non-empty path has a root");
    if !allowed_roots
        .iter()
        .any(|allowed_root| allowed_root == root)
    {
        return Err("workspace path is outside an admitted root".to_owned());
    }
    Ok(normalized)
}

fn is_workspace_root_identifier(root: &str) -> bool {
    !root.is_empty()
        && !root.contains('\0')
        && !root.contains(['/', '\\'])
        && root != "."
        && root != ".."
}

/// Validate one workspace operation's bounded input without touching the
/// filesystem. The executor later resolves the returned relative path under
/// the daemon's admitted Standard Workspace or Extended Home root.
pub fn validate_workspace_operation(
    family: NativeOperationFamily,
    path: &str,
    payload: &str,
    allowed_roots: &[String],
) -> Result<String, String> {
    let normalized_path = validate_workspace_path(path, allowed_roots)?;
    match family {
        NativeOperationFamily::WorkspaceRead => {
            if !payload.is_empty() {
                return Err("workspace read does not accept a write payload".to_owned());
            }
        }
        NativeOperationFamily::WorkspaceSearch => {
            if payload.is_empty() || payload.len() > 4096 {
                return Err("workspace search query exceeds the registered bounds".to_owned());
            }
        }
        NativeOperationFamily::WorkspaceWrite => {
            if payload.len() > 256 * 1024 {
                return Err("workspace write payload exceeds the registered bounds".to_owned());
            }
        }
        NativeOperationFamily::WorkspacePatch => {
            if payload.is_empty() || payload.len() > 256 * 1024 {
                return Err("workspace patch payload exceeds the registered bounds".to_owned());
            }
            if !payload.lines().any(|line| {
                line.starts_with("@@") || line.starts_with("+") || line.starts_with("-")
            }) {
                return Err("workspace patch payload is not a bounded patch".to_owned());
            }
        }
        NativeOperationFamily::ProcessCheck | NativeOperationFamily::HttpFetchReadOnly => {
            return Err("workspace validator received a non-workspace family".to_owned());
        }
    }
    Ok(normalized_path)
}

/// Validate a bounded process/check request without executing it.
pub fn validate_process_check(
    executable_id: &str,
    arguments: &[String],
    working_directory: &str,
    admitted_workspace_roots: &[String],
    registered_executable_ids: &[String],
    timeout_ms: u64,
) -> Result<(), String> {
    if executable_id.is_empty() || executable_id.contains('/') || executable_id.contains('\\') {
        return Err("process executable must be a registered identifier".to_owned());
    }
    if !registered_executable_ids
        .iter()
        .any(|registered_id| registered_id == executable_id)
    {
        return Err("process executable is not registered".to_owned());
    }
    if arguments.len() > 32 || arguments.iter().any(|argument| argument.len() > 4096) {
        return Err("process arguments exceed the registered bounds".to_owned());
    }
    validate_workspace_path(working_directory, admitted_workspace_roots)?;
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
    if !allowed_origins
        .iter()
        .any(|allowed_origin| allowed_origin == &origin)
    {
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
        let Some(descriptor) = BUILTIN_TOOL_CATALOG
            .iter()
            .find(|descriptor| descriptor.operation_id == operation_id)
        else {
            panic!("catalog descriptor must exist");
        };
        let descriptor_digest = match compute_descriptor_digest(descriptor) {
            Ok(descriptor_digest) => descriptor_digest,
            Err(_) => panic!("catalog descriptor must have a digest"),
        };
        ToolResolutionRequest {
            operation_id: operation_id.to_owned(),
            action: descriptor.action.clone(),
            descriptor_version: descriptor.descriptor_version,
            descriptor_digest,
            risk: descriptor.risk,
        }
    }

    fn persisted_descriptor_for(operation_id: &str) -> OperationDescriptor {
        let Some(descriptor) = BUILTIN_TOOL_CATALOG
            .iter()
            .find(|descriptor| descriptor.operation_id == operation_id)
        else {
            panic!("catalog descriptor must exist");
        };
        OperationDescriptor {
            operation_id: descriptor.operation_id.clone(),
            action: descriptor.action.clone(),
            effect_class: EffectClass::Pure,
            executor: descriptor.executor.clone(),
            capabilities: crate::executor::ExecutorCapabilities {
                queryable: true,
                idempotent: true,
            },
            descriptor_version: descriptor.descriptor_version,
        }
    }

    #[test]
    fn catalog_contains_every_required_native_operation_family() {
        assert_eq!(BUILTIN_TOOL_CATALOG.len(), 6);
        assert!(
            BUILTIN_TOOL_CATALOG
                .iter()
                .any(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceRead)
        );
        assert!(
            BUILTIN_TOOL_CATALOG
                .iter()
                .any(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceSearch)
        );
        assert!(
            BUILTIN_TOOL_CATALOG
                .iter()
                .any(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceWrite)
        );
        assert!(
            BUILTIN_TOOL_CATALOG
                .iter()
                .any(|descriptor| descriptor.family == NativeOperationFamily::WorkspacePatch)
        );
        assert!(
            BUILTIN_TOOL_CATALOG
                .iter()
                .any(|descriptor| descriptor.family == NativeOperationFamily::ProcessCheck)
        );
        assert!(
            BUILTIN_TOOL_CATALOG
                .iter()
                .any(|descriptor| descriptor.family == NativeOperationFamily::HttpFetchReadOnly)
        );
    }

    #[test]
    fn exact_descriptor_resolution_is_required() {
        let request = request_for("native.workspace.read");
        assert!(resolve_native_tool(&request).is_ok());
        let mut action_drifted = request.clone();
        action_drifted.action = "write".to_owned();
        assert!(matches!(
            resolve_native_tool(&action_drifted),
            Err(ToolResolutionError::InvalidDescriptor { .. })
        ));
        let mut drifted = request.clone();
        drifted.descriptor_version += 1;
        assert!(matches!(
            resolve_native_tool(&drifted),
            Err(ToolResolutionError::DescriptorVersionMismatch { .. })
        ));
        let mut digest_drifted = request;
        digest_drifted.descriptor_digest = "sha256:drift".to_owned();
        assert!(matches!(
            resolve_native_tool(&digest_drifted),
            Err(ToolResolutionError::DescriptorDigestMismatch { .. })
        ));
    }

    #[test]
    fn descriptor_digest_binds_each_immutable_descriptor_fact() {
        let descriptor = BUILTIN_TOOL_CATALOG
            .iter()
            .find(|descriptor| descriptor.operation_id == "native.workspace.read")
            .expect("workspace read descriptor");
        let digest = compute_descriptor_digest(descriptor).expect("baseline digest");

        let mut action_drift = descriptor.clone();
        action_drift.action = "other".to_owned();
        let mut version_drift = descriptor.clone();
        version_drift.descriptor_version += 1;
        let mut risk_drift = descriptor.clone();
        risk_drift.risk = ToolRisk::WorkspaceMutation;
        let mut executor_drift = descriptor.clone();
        executor_drift.executor = "daemon.other".to_owned();
        let mut capability_drift = descriptor.clone();
        capability_drift.required_capability = "tool.other".to_owned();
        let mut family_drift = descriptor.clone();
        family_drift.family = NativeOperationFamily::WorkspaceSearch;
        let mut input_bound_drift = descriptor.clone();
        input_bound_drift.input_limit_bytes += 1;
        let mut output_bound_drift = descriptor.clone();
        output_bound_drift.output_limit_bytes += 1;

        for drifted_descriptor in [
            action_drift,
            version_drift,
            risk_drift,
            executor_drift,
            capability_drift,
            family_drift,
            input_bound_drift,
            output_bound_drift,
        ] {
            assert_ne!(
                compute_descriptor_digest(&drifted_descriptor).expect("drift digest"),
                digest
            );
        }
    }

    #[test]
    fn disabled_and_quarantined_tools_fail_before_resolution() {
        let request = request_for("native.workspace.read");
        let mut disabled = BUILTIN_TOOL_CATALOG[0].clone();
        disabled.availability = ToolAvailability::Disabled;
        assert!(matches!(
            resolve_native_tool_from_catalog(&[disabled], &request),
            Err(ToolResolutionError::DisabledTool { .. })
        ));

        let mut quarantined = BUILTIN_TOOL_CATALOG[0].clone();
        quarantined.availability = ToolAvailability::Quarantined;
        assert!(matches!(
            resolve_native_tool_from_catalog(&[quarantined], &request),
            Err(ToolResolutionError::QuarantinedTool { .. })
        ));
    }

    #[test]
    fn persisted_native_descriptor_must_match_effect_and_recovery_facts() {
        let descriptor = BUILTIN_TOOL_CATALOG
            .iter()
            .find(|descriptor| descriptor.operation_id == "native.workspace.read")
            .expect("workspace read descriptor");
        let persisted = persisted_descriptor_for(&descriptor.operation_id);
        assert!(resolve_persisted_native_descriptor(&persisted).is_ok());

        let mut drifted = persisted;
        drifted.executor = "daemon.unknown".to_owned();
        assert!(matches!(
            resolve_persisted_native_descriptor(&drifted),
            Err(ToolResolutionError::InvalidDescriptor { .. })
        ));
        let mut effect_drifted = persisted_descriptor_for("native.workspace.read");
        effect_drifted.effect_class = EffectClass::GovernedExternal;
        assert!(resolve_persisted_native_descriptor(&effect_drifted).is_err());
        let mut recovery_drifted = persisted_descriptor_for("native.workspace.read");
        recovery_drifted.capabilities.idempotent = false;
        assert!(resolve_persisted_native_descriptor(&recovery_drifted).is_err());
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
        assert!(matches!(
            resolve_native_tool(&request),
            Err(ToolResolutionError::UnknownTool { .. })
        ));
    }

    #[test]
    fn workspace_process_and_http_validators_fail_closed() {
        let roots = ["workspace".to_owned(), "extended-home".to_owned()];
        let registered_executables = ["cargo".to_owned()];
        assert!(validate_workspace_path("workspace/src/main.rs", &roots).is_ok());
        assert!(validate_workspace_path("extended-home/docs/readme.md", &roots).is_ok());
        assert!(validate_workspace_path("other-root/src/main.rs", &roots).is_err());
        assert!(validate_workspace_path("/workspace/src/main.rs", &roots).is_err());
        assert!(validate_workspace_path("workspace/../secret", &roots).is_err());
        assert!(validate_workspace_path("", &roots).is_err());
        assert!(validate_workspace_path("workspace/\0secret", &roots).is_err());
        assert!(validate_workspace_path("../secret", &["workspace".to_owned()]).is_err());
        assert!(
            validate_workspace_operation(
                NativeOperationFamily::WorkspaceRead,
                "workspace/src/main.rs",
                "",
                &roots
            )
            .is_ok()
        );
        assert!(
            validate_workspace_operation(
                NativeOperationFamily::WorkspacePatch,
                "workspace/src/main.rs",
                "@@ -1 +1 @@\n-old\n+new",
                &roots
            )
            .is_ok()
        );
        assert!(
            validate_workspace_operation(
                NativeOperationFamily::WorkspaceRead,
                "workspace/src/main.rs",
                "unexpected payload",
                &roots
            )
            .is_err()
        );
        assert!(
            validate_process_check(
                "cargo",
                &[],
                "workspace",
                &roots,
                &registered_executables,
                1000
            )
            .is_ok()
        );
        assert!(
            validate_process_check(
                "/bin/sh",
                &[],
                "workspace",
                &roots,
                &registered_executables,
                1000
            )
            .is_err()
        );
        assert!(
            validate_process_check(
                "sh",
                &[],
                "workspace",
                &roots,
                &registered_executables,
                1000
            )
            .is_err()
        );
        assert!(
            validate_read_only_http_fetch(
                "GET",
                "https://example.com/a",
                &["https://example.com".to_owned()],
                1000
            )
            .is_ok()
        );
        assert!(
            validate_read_only_http_fetch(
                "POST",
                "https://example.com/a",
                &["https://example.com".to_owned()],
                1000
            )
            .is_err()
        );
    }
}
