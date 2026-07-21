//! M6 behavioral vector execution: package verification, adapter/sandbox
//! bypass denial, and OOB reconciliation — against public RUN surfaces
//! (`cognitive-runtime` installer / sandbox / adapters / oob).
//!
//! Deliberately wrong: accept invalid signatures, allow unmediated network,
//! and silently adopt out-of-band workspace bytes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_runtime::{
    CompletionAdapter, InstallationLedger, OobReconciler, PackageInstallRequest, ProjectionObject,
    RejectingSignaturePort, SandboxChannel, SandboxGate, SandboxPlatform, SandboxPolicy,
    install_package,
};
use serde_json::json;
use std::collections::BTreeSet;

const REFERENCE_IMPLEMENTATION: &str = "cognitive-runtime installer + SandboxGate + \
     CompletionAdapter + OobReconciler (real M6 RUN surfaces)";
const WRONG_IMPLEMENTATION: &str = "install/sandbox/oob anti-pattern implementation \
     (deliberately wrong: accept bad signature, allow unmediated I/O, silent OOB adopt)";

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

fn implementation_label(kind: ImplementationKind) -> Option<&'static str> {
    Some(match kind {
        ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
        ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
    })
}

fn registered(ctx: &AssetContext, code: &str) -> Result<(), ExecError> {
    ctx.registered_error(code)
        .ok_or_else(|| env_err(format!("code {code} not registered")))?;
    Ok(())
}

fn valid_request(artifact: &[u8], digest: &str) -> PackageInstallRequest {
    PackageInstallRequest {
        package_id: "pkg://cfr-m6-install-01".into(),
        publisher: "cognitiveos-reference".into(),
        package_version: "0.1.0".into(),
        artifact: artifact.to_vec(),
        declared_artifact_digest: digest.to_owned(),
        signature_ref: "sig://invalid".into(),
        provenance_ref: "prov://invalid".into(),
        adapter_digest: "sha256:adapter".into(),
        sandbox_digest: "sha256:sandbox".into(),
        compatibility_digest: "sha256:compat".into(),
        expected_adapter_digest: "sha256:adapter".into(),
        expected_sandbox_digest: "sha256:sandbox".into(),
        expected_compatibility_digest: "sha256:compat".into(),
    }
}

/// AGENT-INSTALL-001 — invalid signature prevents installation commit.
pub(super) fn agent_install_001_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    registered(ctx, "AGENT_PACKAGE_VERIFICATION_FAILED")?;
    let artifact = b"tampered-agent-package";
    // Digest matching bytes so only the signature port fails.
    let digest = {
        let hex: String = artifact.iter().map(|b| format!("{b:02x}")).collect();
        let value = serde_json::Value::String(hex);
        let canonical = cognitive_contracts::canonical::canonical_bytes_of_value(&value)
            .map_err(|e| env_err(e.to_string()))?;
        cognitive_contracts::canonical::digest(&canonical, "agent-package-artifact/0.1")
            .map_err(|e| env_err(e.to_string()))?
    };
    let req = valid_request(artifact, &digest);
    let ledger = InstallationLedger::new();

    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        // Wrong: pretend verification passed and commit capability.
        return Ok(GateOutput {
            actual: json!({
                "outcome": "committed",
                "error_code": "OK",
                "authority_unchanged": false,
                "capability_expanded": true
            }),
            grounding: vec!["deliberately-wrong: accept invalid signature / provenance".to_owned()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({
                "committed_count": 1,
                "capability_grants": 1
            }),
        });
    }

    let err = install_package(&ledger, &req, &RejectingSignaturePort, None)
        .expect_err("invalid signature must fail");
    assert_eq!(err.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(ledger.committed_view().is_empty());
    assert_eq!(ledger.capability_grants(), 0);

    Ok(GateOutput {
        actual: json!({
            "outcome": "denied_or_controlled_fallback",
            "error_code": "AGENT_PACKAGE_VERIFICATION_FAILED",
            "authority_unchanged": true,
            "capability_expanded": false
        }),
        grounding: vec![
            "cognitive-runtime::install_package + RejectingSignaturePort".to_owned(),
            "REQ-AGENT-INSTALL-001".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "committed_count": ledger.committed_view().len(),
            "capability_grants": ledger.capability_grants(),
            "detail": err.detail
        }),
    })
}

/// AGENT-BYPASS-002 — network/secret/tool bypass and self-completion rejected.
pub(super) fn agent_bypass_002_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    registered(ctx, "AGENT_ADAPTER_BYPASS_DETECTED")?;
    let mut declared = BTreeSet::new();
    declared.insert(SandboxChannel::Network);
    let gate = SandboxGate {
        platform: SandboxPlatform::LinuxNative,
        policy: SandboxPolicy {
            declared_channels: declared,
            registered_tool_proxies: BTreeSet::new(),
            registered_mcp_servers: BTreeSet::new(),
        },
        evidenced_denials: BTreeSet::from([
            SandboxChannel::Network,
            SandboxChannel::Secrets,
            SandboxChannel::ToolProxy,
        ]),
    };

    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "outcome": "allowed",
                "error_code": "OK",
                "authority_unchanged": false,
                "capability_expanded": true
            }),
            grounding: vec![
                "deliberately-wrong: allow unmediated network + self-completion".to_owned(),
            ],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"bypass_accepted": true}),
        });
    }

    let net = gate
        .intercept(SandboxChannel::Network, "https://evil.example", false)
        .expect_err("unmediated network must deny");
    let secret = gate
        .intercept(SandboxChannel::Secrets, "host://~/.ssh/id_rsa", true)
        .expect_err("host secret must deny");
    let tool = gate
        .intercept(SandboxChannel::ToolProxy, "proxy://shadow", true)
        .expect_err("unregistered proxy must deny");
    let completion = CompletionAdapter::reject_authority_completed("task_complete COMPLETED")
        .expect_err("self-completion must deny");

    assert_eq!(net.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    assert_eq!(secret.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    assert_eq!(tool.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    assert_eq!(completion.code, "AGENT_ADAPTER_BYPASS_DETECTED");

    Ok(GateOutput {
        actual: json!({
            "outcome": "denied_or_controlled_fallback",
            "error_code": "AGENT_ADAPTER_BYPASS_DETECTED",
            "authority_unchanged": true,
            "capability_expanded": false
        }),
        grounding: vec![
            "cognitive-runtime::SandboxGate::intercept".to_owned(),
            "cognitive-runtime::CompletionAdapter::reject_authority_completed".to_owned(),
            "REQ-AGENT-ADAPTER-001 / REQ-AGENT-SANDBOX-001 / REQ-AGENT-COMPLETE-001".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "cases": ["undeclared_network", "host_secret", "unregistered_tool_proxy", "self_completion"],
            "platform": "linux_native",
            "windows_native_claim": "unsupported_without_evidence"
        }),
    })
}

/// AGENT-OOB-001 — digest drift ingests candidate; no silent overwrite.
pub(super) fn agent_oob_001_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = ctx;
    let authority = b"authority-projection-v1";
    let pinned = format!(
        "sha256:len{}-b0-{:02x}",
        authority.len(),
        authority.first().copied().unwrap_or(0)
    );
    let mut reconciler = OobReconciler::default();
    reconciler.pin(ProjectionObject {
        path: "workspace/memory.json".into(),
        pinned_digest: pinned,
        authority_bytes: authority.to_vec(),
    });
    let on_disk = b"user-edited-out-of-band";

    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "outcome": "adopted",
                "digest_drift_detected": false,
                "edit_ingested_as_candidate": false,
                "silent_overwrite_either_side": true,
                "authority_unchanged": false,
                "capability_expanded": true
            }),
            grounding: vec!["deliberately-wrong: silent adopt of OOB bytes".to_owned()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"silent_overwrite": true}),
        });
    }

    let result = reconciler
        .first_read_after_edit("workspace/memory.json", on_disk)
        .map_err(|e| env_err(e.detail))?;
    assert_eq!(result["digest_drift_detected"], true);
    assert_eq!(result["edit_ingested_as_candidate"], true);
    assert_eq!(result["silent_overwrite_either_side"], false);
    assert_eq!(reconciler.silent_overwrite_count(), 0);
    assert_eq!(
        reconciler.authority_bytes("workspace/memory.json"),
        Some(authority.as_slice())
    );
    assert_eq!(reconciler.candidates().len(), 1);

    Ok(GateOutput {
        actual: json!({
            "outcome": "denied_or_controlled_fallback",
            "digest_drift_detected": true,
            "edit_ingested_as_candidate": true,
            "silent_overwrite_either_side": false,
            "authority_unchanged": true,
            "capability_expanded": false
        }),
        grounding: vec![
            "cognitive-runtime::OobReconciler::first_read_after_edit".to_owned(),
            "REQ-AGENT-OOB-001".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: result,
    })
}
