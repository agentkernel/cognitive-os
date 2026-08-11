//! Post-1.0 MCP Tool adapter qualification — private MVP (P5-T03).
//!
//! MCP initialize/capability/version/timeout establish transport only. They
//! never grant CognitiveOS capability, Intent/Effect authority, or Task
//! completion. Native Tool Registry remains the sole operation catalog;
//! MCP tool-list changes produce candidates that require re-qualification.

use crate::sandbox::{SandboxChannel, SandboxGate, SandboxPolicy};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Fixture MCP server id for P5-T03 qualification (non-authoritative).
pub const MCP_FIXTURE_SERVER_ID: &str = "fixture.mcp.filesystem";

/// MCP protocol pin referenced by the research ledger (MCP-01).
pub const MCP_PROTOCOL_VERSION_PIN: &str = "2025-06-18";

/// Upper bound for initialize timeout in the MVP fixture path (milliseconds).
pub const MCP_INITIALIZE_TIMEOUT_BUDGET_MS: u64 = 5_000;

/// Transport profile for the MCP fixture (stdio JSON-RPC style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransportProfile {
    StdioJsonRpc,
}

impl McpTransportProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StdioJsonRpc => "stdio_jsonrpc",
        }
    }
}

/// Digest-bound MCP server package/manifest identity (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerManifest {
    pub server_id: String,
    pub protocol_version: String,
    pub package_digest: String,
    pub manifest_digest: String,
    pub transport: McpTransportProfile,
    pub authority_writer: bool,
}

/// Transport-only MCP session after initialize. Never a CognitiveOS capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpTransportSession {
    pub server_id: String,
    pub protocol_version: String,
    pub manifest_digest: String,
    pub session_digest: String,
    pub transport_only: bool,
    pub cognitiveos_capability_granted: bool,
    pub mcp_capability_names: Vec<String>,
}

/// Candidate produced when an MCP server advertises a tool-list change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpToolListCandidate {
    pub server_id: String,
    pub list_digest: String,
    pub enabled: bool,
    pub requires_requalification: bool,
}

/// Fixed-denominator MCP adapter qualification observation (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpQualificationObservation {
    pub server_id: String,
    pub manifest_digest: String,
    pub session_digest: String,
    pub claim_scope: &'static str,
    pub observations: Vec<&'static str>,
    pub report_digest: String,
}

/// Fail-closed MCP adapter qualification errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum McpToolAdapterError {
    #[error("MCP package/manifest identity is missing required material")]
    MissingIdentity,
    #[error("MCP protocol version does not match the pinned fixture version")]
    ProtocolVersionMismatch,
    #[error("MCP manifest digest drifted from the admitted fixture")]
    ManifestDigestDrift,
    #[error("MCP initialize timeout is missing, zero, or exceeds the fixture budget")]
    InitializeTimeoutInvalid,
    #[error("MCP must not declare CognitiveOS authority-writer capability")]
    AuthorityWriterForbidden,
    #[error("MCP transport completion must not grant CognitiveOS capability")]
    CognitiveOsCapabilityForbidden,
    #[error("MCP tool-list change cannot auto-enable without re-qualification")]
    AutoEnableForbidden,
    #[error("MCP direct endpoint bypass is forbidden")]
    DirectBypassForbidden,
    #[error("MCP qualification rejects Gate/authority-shaped claims")]
    AuthorityShapedClaimForbidden,
    #[error("MCP sandbox mediation rejected the request: {0}")]
    SandboxBypass(String),
}

const NON_CLAIM: &str = "non-claim";
const REQUIRED_OBSERVATIONS: &[&str] = &[
    "manifest_identity_bound",
    "transport_only_initialize",
    "manifest_drift_and_timeout_fail_closed",
    "direct_bypass_rejected",
];

/// Bind a fixture MCP server package/manifest identity.
pub fn bind_mcp_fixture_manifest(
    server_id: &str,
    protocol_version: &str,
    package_digest: &str,
) -> Result<McpServerManifest, McpToolAdapterError> {
    if server_id.trim().is_empty() || package_digest.trim().is_empty() {
        return Err(McpToolAdapterError::MissingIdentity);
    }
    if protocol_version != MCP_PROTOCOL_VERSION_PIN {
        return Err(McpToolAdapterError::ProtocolVersionMismatch);
    }
    if server_id != MCP_FIXTURE_SERVER_ID {
        return Err(McpToolAdapterError::MissingIdentity);
    }

    let mut hasher = Sha256::new();
    hasher.update(server_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(protocol_version.as_bytes());
    hasher.update(b"\0");
    hasher.update(package_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(McpTransportProfile::StdioJsonRpc.as_str().as_bytes());
    Ok(McpServerManifest {
        server_id: server_id.to_owned(),
        protocol_version: protocol_version.to_owned(),
        package_digest: package_digest.to_owned(),
        manifest_digest: format!("{:x}", hasher.finalize()),
        transport: McpTransportProfile::StdioJsonRpc,
        authority_writer: false,
    })
}

/// Initialize MCP transport only. Protocol capabilities never become Cos grants.
pub fn initialize_mcp_transport(
    manifest: &McpServerManifest,
    mcp_capability_names: &[&str],
    timeout_ms: u64,
    declares_authority_writer: bool,
    grants_cognitiveos_capability: bool,
) -> Result<McpTransportSession, McpToolAdapterError> {
    if manifest.server_id != MCP_FIXTURE_SERVER_ID
        || manifest.manifest_digest.trim().is_empty()
        || manifest.package_digest.trim().is_empty()
    {
        return Err(McpToolAdapterError::MissingIdentity);
    }
    if manifest.protocol_version != MCP_PROTOCOL_VERSION_PIN {
        return Err(McpToolAdapterError::ProtocolVersionMismatch);
    }
    if manifest.authority_writer || declares_authority_writer {
        return Err(McpToolAdapterError::AuthorityWriterForbidden);
    }
    if grants_cognitiveos_capability {
        return Err(McpToolAdapterError::CognitiveOsCapabilityForbidden);
    }
    if timeout_ms == 0 || timeout_ms > MCP_INITIALIZE_TIMEOUT_BUDGET_MS {
        return Err(McpToolAdapterError::InitializeTimeoutInvalid);
    }

    let caps: Vec<String> = mcp_capability_names
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(manifest.manifest_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(timeout_ms.to_string().as_bytes());
    hasher.update(b"\0");
    for name in &caps {
        hasher.update(name.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(b"transport_only");
    Ok(McpTransportSession {
        server_id: manifest.server_id.clone(),
        protocol_version: manifest.protocol_version.clone(),
        manifest_digest: manifest.manifest_digest.clone(),
        session_digest: format!("{:x}", hasher.finalize()),
        transport_only: true,
        cognitiveos_capability_granted: false,
        mcp_capability_names: caps,
    })
}

/// Re-check an active transport session against the admitted manifest.
pub fn verify_mcp_manifest_current(
    manifest: &McpServerManifest,
    session: &McpTransportSession,
    observed_protocol_version: &str,
    observed_manifest_digest: &str,
) -> Result<(), McpToolAdapterError> {
    if session.server_id != manifest.server_id
        || session.manifest_digest != manifest.manifest_digest
    {
        return Err(McpToolAdapterError::MissingIdentity);
    }
    if observed_protocol_version != MCP_PROTOCOL_VERSION_PIN
        || observed_protocol_version != manifest.protocol_version
    {
        return Err(McpToolAdapterError::ProtocolVersionMismatch);
    }
    if observed_manifest_digest != manifest.manifest_digest
        || observed_manifest_digest != session.manifest_digest
    {
        return Err(McpToolAdapterError::ManifestDigestDrift);
    }
    Ok(())
}

/// Tool-list changes produce disabled candidates that require re-qualification.
pub fn plan_mcp_tool_list_candidate(
    session: &McpTransportSession,
    list_digest: &str,
    auto_enable: bool,
) -> Result<McpToolListCandidate, McpToolAdapterError> {
    if !session.transport_only || session.cognitiveos_capability_granted {
        return Err(McpToolAdapterError::CognitiveOsCapabilityForbidden);
    }
    if list_digest.trim().is_empty() {
        return Err(McpToolAdapterError::MissingIdentity);
    }
    if auto_enable {
        return Err(McpToolAdapterError::AutoEnableForbidden);
    }
    Ok(McpToolListCandidate {
        server_id: session.server_id.clone(),
        list_digest: list_digest.to_owned(),
        enabled: false,
        requires_requalification: true,
    })
}

/// Register the fixture MCP server id into a sandbox policy (mediation only).
pub fn register_mcp_server_in_policy(
    mut policy: SandboxPolicy,
    server_id: &str,
) -> Result<SandboxPolicy, McpToolAdapterError> {
    if server_id != MCP_FIXTURE_SERVER_ID {
        return Err(McpToolAdapterError::MissingIdentity);
    }
    policy.declared_channels.insert(SandboxChannel::Mcp);
    policy.registered_mcp_servers.insert(server_id.to_owned());
    Ok(policy)
}

/// Mediated MCP access through the sandbox gate; unmediated paths fail closed.
pub fn mediate_mcp_access(
    gate: &SandboxGate,
    server_id: &str,
    mediated: bool,
) -> Result<(), McpToolAdapterError> {
    if !mediated {
        return Err(McpToolAdapterError::DirectBypassForbidden);
    }
    gate.intercept(SandboxChannel::Mcp, server_id, true)
        .map_err(|err| McpToolAdapterError::SandboxBypass(err.detail))
}

/// Build a fixed-denominator non-claim MCP adapter qualification report.
pub fn build_mcp_qualification_report(
    manifest: &McpServerManifest,
    session: &McpTransportSession,
    observations: &[&str],
    authority_claim_labels: &[&str],
) -> Result<McpQualificationObservation, McpToolAdapterError> {
    if manifest.server_id != MCP_FIXTURE_SERVER_ID
        || session.server_id != MCP_FIXTURE_SERVER_ID
        || manifest.manifest_digest.trim().is_empty()
        || session.session_digest.trim().is_empty()
        || !session.transport_only
        || session.cognitiveos_capability_granted
    {
        return Err(McpToolAdapterError::MissingIdentity);
    }
    for label in authority_claim_labels {
        let normalized = label.trim().to_ascii_lowercase();
        if matches!(
            normalized.as_str(),
            "gate" | "release" | "profile" | "b10" | "pass" | "passed" | "gmvp-linux"
        ) {
            return Err(McpToolAdapterError::AuthorityShapedClaimForbidden);
        }
    }
    let mut sorted_required: Vec<&str> = REQUIRED_OBSERVATIONS.to_vec();
    sorted_required.sort_unstable();
    let mut sorted_actual: Vec<&str> = observations.to_vec();
    sorted_actual.sort_unstable();
    if sorted_actual != sorted_required {
        return Err(McpToolAdapterError::MissingIdentity);
    }

    let mut hasher = Sha256::new();
    hasher.update(manifest.server_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(manifest.manifest_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(session.session_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(NON_CLAIM.as_bytes());
    for observation in sorted_required.iter() {
        hasher.update(observation.as_bytes());
        hasher.update(b"\0");
    }
    Ok(McpQualificationObservation {
        server_id: manifest.server_id.clone(),
        manifest_digest: manifest.manifest_digest.clone(),
        session_digest: session.session_digest.clone(),
        claim_scope: NON_CLAIM,
        observations: REQUIRED_OBSERVATIONS.to_vec(),
        report_digest: format!("{:x}", hasher.finalize()),
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::sandbox::SandboxPlatform;
    use std::collections::BTreeSet;

    fn fixture_digest() -> String {
        format!("sha256:{}", "a".repeat(64))
    }

    fn bound_manifest() -> McpServerManifest {
        bind_mcp_fixture_manifest(
            MCP_FIXTURE_SERVER_ID,
            MCP_PROTOCOL_VERSION_PIN,
            &fixture_digest(),
        )
        .expect("bind")
    }

    fn transport_session(manifest: &McpServerManifest) -> McpTransportSession {
        initialize_mcp_transport(manifest, &["tools"], 1_000, false, false).expect("init")
    }

    #[test]
    fn binds_mcp_fixture_and_initializes_transport_only() {
        let manifest = bound_manifest();
        assert_eq!(manifest.server_id, MCP_FIXTURE_SERVER_ID);
        assert_eq!(manifest.protocol_version, MCP_PROTOCOL_VERSION_PIN);
        assert!(!manifest.authority_writer);
        assert_eq!(manifest.manifest_digest.len(), 64);

        let session = transport_session(&manifest);
        assert!(session.transport_only);
        assert!(!session.cognitiveos_capability_granted);
        assert_eq!(session.mcp_capability_names, vec!["tools".to_owned()]);
        assert_eq!(session.session_digest.len(), 64);
    }

    #[test]
    fn rejects_authority_surface_on_bind_and_initialize() {
        assert_eq!(
            bind_mcp_fixture_manifest("other.server", MCP_PROTOCOL_VERSION_PIN, &fixture_digest())
                .unwrap_err(),
            McpToolAdapterError::MissingIdentity
        );
        assert_eq!(
            bind_mcp_fixture_manifest(MCP_FIXTURE_SERVER_ID, "2024-11-05", &fixture_digest())
                .unwrap_err(),
            McpToolAdapterError::ProtocolVersionMismatch
        );
        assert_eq!(
            bind_mcp_fixture_manifest(MCP_FIXTURE_SERVER_ID, MCP_PROTOCOL_VERSION_PIN, "  ")
                .unwrap_err(),
            McpToolAdapterError::MissingIdentity
        );

        let manifest = bound_manifest();
        assert_eq!(
            initialize_mcp_transport(&manifest, &["tools"], 1_000, true, false).unwrap_err(),
            McpToolAdapterError::AuthorityWriterForbidden
        );
        assert_eq!(
            initialize_mcp_transport(&manifest, &["tools"], 1_000, false, true).unwrap_err(),
            McpToolAdapterError::CognitiveOsCapabilityForbidden
        );
    }

    #[test]
    fn rejects_manifest_drift_timeout_and_auto_enable() {
        let manifest = bound_manifest();
        let session = transport_session(&manifest);

        assert_eq!(
            initialize_mcp_transport(&manifest, &["tools"], 0, false, false).unwrap_err(),
            McpToolAdapterError::InitializeTimeoutInvalid
        );
        assert_eq!(
            initialize_mcp_transport(
                &manifest,
                &["tools"],
                MCP_INITIALIZE_TIMEOUT_BUDGET_MS + 1,
                false,
                false
            )
            .unwrap_err(),
            McpToolAdapterError::InitializeTimeoutInvalid
        );

        verify_mcp_manifest_current(
            &manifest,
            &session,
            MCP_PROTOCOL_VERSION_PIN,
            &manifest.manifest_digest,
        )
        .expect("current");
        assert_eq!(
            verify_mcp_manifest_current(
                &manifest,
                &session,
                "2024-11-05",
                &manifest.manifest_digest,
            )
            .unwrap_err(),
            McpToolAdapterError::ProtocolVersionMismatch
        );
        assert_eq!(
            verify_mcp_manifest_current(
                &manifest,
                &session,
                MCP_PROTOCOL_VERSION_PIN,
                "sha256:drifted",
            )
            .unwrap_err(),
            McpToolAdapterError::ManifestDigestDrift
        );

        let candidate =
            plan_mcp_tool_list_candidate(&session, "sha256:list1", false).expect("candidate");
        assert!(!candidate.enabled);
        assert!(candidate.requires_requalification);
        assert_eq!(
            plan_mcp_tool_list_candidate(&session, "sha256:list1", true).unwrap_err(),
            McpToolAdapterError::AutoEnableForbidden
        );
    }

    #[test]
    fn rejects_direct_bypass_and_builds_non_claim_report() {
        let manifest = bound_manifest();
        let session = transport_session(&manifest);
        let policy = register_mcp_server_in_policy(SandboxPolicy::default(), MCP_FIXTURE_SERVER_ID)
            .expect("policy");
        let gate = SandboxGate {
            platform: SandboxPlatform::LinuxNative,
            policy,
            evidenced_denials: BTreeSet::new(),
        };

        mediate_mcp_access(&gate, MCP_FIXTURE_SERVER_ID, true).expect("mediated");
        assert_eq!(
            mediate_mcp_access(&gate, MCP_FIXTURE_SERVER_ID, false).unwrap_err(),
            McpToolAdapterError::DirectBypassForbidden
        );
        assert!(matches!(
            mediate_mcp_access(&gate, "unregistered.mcp", true).unwrap_err(),
            McpToolAdapterError::SandboxBypass(_)
        ));

        let report =
            build_mcp_qualification_report(&manifest, &session, REQUIRED_OBSERVATIONS, &[])
                .expect("report");
        assert_eq!(report.claim_scope, NON_CLAIM);
        assert_eq!(report.observations.len(), 4);
        assert_eq!(report.report_digest.len(), 64);
        assert_eq!(
            build_mcp_qualification_report(&manifest, &session, REQUIRED_OBSERVATIONS, &["B10"],)
                .unwrap_err(),
            McpToolAdapterError::AuthorityShapedClaimForbidden
        );
        assert_eq!(
            build_mcp_qualification_report(&manifest, &session, &["incomplete"], &[]).unwrap_err(),
            McpToolAdapterError::MissingIdentity
        );
    }
}
