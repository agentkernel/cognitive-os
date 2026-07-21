//! C0/C1 adapter family ports (six families) and compatibility reporting.
//!
//! Completion outputs are always `CANDIDATE_COMPLETE` — never authority
//! COMPLETED. Batch tool calls require a registered proxy endpoint (IMP-12).

use crate::sandbox::{SandboxChannel, SandboxError, SandboxGate};
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityProfile {
    C0,
    C1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureStatus {
    Supported,
    Degraded,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct AdapterError {
    pub code: &'static str,
    pub detail: String,
}

impl AdapterError {
    fn bypass(detail: impl Into<String>) -> Self {
        Self {
            code: RegisteredErrorCode::AgentAdapterBypassDetected.as_str(),
            detail: detail.into(),
        }
    }

    fn degraded(detail: impl Into<String>) -> Self {
        Self {
            code: RegisteredErrorCode::AgentCompatibilityDegraded.as_str(),
            detail: detail.into(),
        }
    }
}

/// Identity adapter: never trusts agent self-reported user/role.
#[derive(Debug, Default)]
pub struct IdentityAdapter;

impl IdentityAdapter {
    pub fn resolve(
        &self,
        claimed_user: Option<&str>,
        authority_principal: &str,
    ) -> Result<String, AdapterError> {
        if let Some(claimed) = claimed_user {
            if claimed != authority_principal {
                return Err(AdapterError::bypass(format!(
                    "agent self-reported identity {claimed} rejected; authority={authority_principal}"
                )));
            }
        }
        Ok(authority_principal.to_owned())
    }
}

/// Memory adapter maps search/get/add/update/delete → resolve/expand/propose/…
#[derive(Debug, Default)]
pub struct MemoryAdapter;

impl MemoryAdapter {
    pub fn map_op(&self, op: &str) -> Result<&'static str, AdapterError> {
        Ok(match op {
            "search" => "resolve",
            "get" => "expand",
            "add" | "update" => "propose",
            "delete" => "invalidate",
            other => {
                return Err(AdapterError::bypass(format!(
                    "unknown memory op {other}"
                )));
            }
        })
    }
}

/// Tool adapter: list/call → discover/describe/bind + Intent/Effect.
#[derive(Debug, Default)]
pub struct ToolAdapter {
    pub registered_batch_proxy: Option<String>,
}

impl ToolAdapter {
    pub fn call_via_proxy(
        &self,
        proxy_endpoint: &str,
        calls: &[Value],
        sandbox: &SandboxGate,
    ) -> Result<Vec<Value>, AdapterError> {
        sandbox
            .intercept(SandboxChannel::ToolProxy, proxy_endpoint, true)
            .map_err(|e| AdapterError {
                code: e.code,
                detail: e.detail,
            })?;
        match &self.registered_batch_proxy {
            Some(reg) if reg == proxy_endpoint => {}
            _ => {
                return Err(AdapterError::bypass(
                    "batch tool calls require registered proxy endpoint (IMP-12)",
                ));
            }
        }
        // Each call retains its own authz/audit slot (represented as receipts).
        Ok(calls
            .iter()
            .enumerate()
            .map(|(i, c)| {
                json!({
                    "index": i,
                    "authorized": true,
                    "audited": true,
                    "call": c,
                })
            })
            .collect())
    }
}

/// Completion adapter: only `CANDIDATE_COMPLETE`.
#[derive(Debug, Default)]
pub struct CompletionAdapter;

impl CompletionAdapter {
    pub fn complete(&self, agent_said_done: bool) -> Result<&'static str, AdapterError> {
        if !agent_said_done {
            return Err(AdapterError::bypass(
                "completion without agent terminal signal",
            ));
        }
        Ok("CANDIDATE_COMPLETE")
    }

    pub fn reject_authority_completed(agent_text: &str) -> Result<(), AdapterError> {
        if agent_text.contains("COMPLETED") || agent_text.contains("task_complete") {
            return Err(AdapterError::bypass(
                "agent self-completion cannot advance Task to COMPLETED",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct CheckpointAdapter;

impl CheckpointAdapter {
    pub fn save_facts(&self, pending_effects: usize) -> Value {
        json!({
            "action_level_facts": true,
            "pending_effects": pending_effects,
        })
    }
}

#[derive(Debug, Default)]
pub struct SandboxAdapterFamily;

impl SandboxAdapterFamily {
    pub fn mediate(
        gate: &SandboxGate,
        channel: SandboxChannel,
        target: &str,
    ) -> Result<(), SandboxError> {
        gate.intercept(channel, target, false)
    }
}

/// Build a compatibility feature matrix for C0/C1.
pub fn compatibility_matrix(
    profile: CompatibilityProfile,
    features: BTreeMap<String, FeatureStatus>,
) -> Value {
    let feature_json: BTreeMap<String, &str> = features
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                match v {
                    FeatureStatus::Supported => "supported",
                    FeatureStatus::Degraded => "degraded",
                    FeatureStatus::Unsupported => "unsupported",
                },
            )
        })
        .collect();
    json!({
        "profile": match profile {
            CompatibilityProfile::C0 => "C0",
            CompatibilityProfile::C1 => "C1",
        },
        "features": feature_json,
        "max_verified_risk": match profile {
            CompatibilityProfile::C0 => "R0",
            CompatibilityProfile::C1 => "R1",
        },
    })
}

/// Adapter failure must fail closed or degrade the claim (REQ-AGENT-ADAPTER-001).
pub fn on_adapter_failure(fail_closed: bool) -> Result<(), AdapterError> {
    if fail_closed {
        Err(AdapterError::bypass("adapter failure fail-closed"))
    } else {
        Err(AdapterError::degraded(
            "adapter failure lowers compatibility declaration",
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::sandbox::{SandboxPlatform, SandboxPolicy};
    use std::collections::BTreeSet;

    #[test]
    fn identity_rejects_self_report() {
        let err = IdentityAdapter
            .resolve(Some("agent-said-admin"), "user://alice")
            .unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    #[test]
    fn unregistered_batch_proxy_rejected() {
        let tool = ToolAdapter {
            registered_batch_proxy: Some("proxy://ok".into()),
        };
        let mut proxies = BTreeSet::new();
        proxies.insert("proxy://ok".into());
        let gate = SandboxGate {
            platform: SandboxPlatform::LinuxNative,
            policy: SandboxPolicy {
                declared_channels: BTreeSet::new(),
                registered_tool_proxies: proxies,
                registered_mcp_servers: BTreeSet::new(),
            },
            evidenced_denials: BTreeSet::new(),
        };
        let err = tool
            .call_via_proxy("proxy://shadow", &[json!({"op":"x"})], &gate)
            .unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    #[test]
    fn completion_is_candidate_only() {
        assert_eq!(
            CompletionAdapter.complete(true).unwrap(),
            "CANDIDATE_COMPLETE"
        );
        let err = CompletionAdapter::reject_authority_completed("mark COMPLETED now").unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    #[test]
    fn adapter_crash_fail_closed() {
        let err = on_adapter_failure(true).unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }
}
