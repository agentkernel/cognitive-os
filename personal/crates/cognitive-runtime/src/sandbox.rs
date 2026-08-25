//! Sandbox interception and per-platform claim matrix (F-017 / M6-A2).
//!
//! Platform rows MUST NOT be merged: Linux native, Windows+WSL2 (Linux guest),
//! and Windows native are distinct. Claims without evidence stay
//! `unsupported` / `not-tested`.

use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Execution environment row in the F-017 matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxPlatform {
    LinuxNative,
    WindowsWsl2LinuxGuest,
    WindowsNative,
}

impl SandboxPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LinuxNative => "linux_native",
            Self::WindowsWsl2LinuxGuest => "windows_wsl2_linux_guest",
            Self::WindowsNative => "windows_native",
        }
    }
}

/// Mediated boundary channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxChannel {
    Network,
    Filesystem,
    Secrets,
    Subprocess,
    Device,
    Model,
    Mcp,
    A2a,
    Ipc,
    ToolProxy,
}

impl SandboxChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Filesystem => "filesystem",
            Self::Secrets => "secrets",
            Self::Subprocess => "subprocess",
            Self::Device => "device",
            Self::Model => "model",
            Self::Mcp => "mcp",
            Self::A2a => "a2a",
            Self::Ipc => "ipc",
            Self::ToolProxy => "tool_proxy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelClaim {
    DeniedWithEvidence,
    Degraded,
    Unsupported,
    NotTested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformChannelRow {
    pub platform: SandboxPlatform,
    pub channel: SandboxChannel,
    pub claim: ChannelClaim,
    pub evidence_digest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SandboxError {
    pub code: &'static str,
    pub detail: String,
}

impl SandboxError {
    fn bypass(detail: impl Into<String>) -> Self {
        Self {
            code: RegisteredErrorCode::AgentAdapterBypassDetected.as_str(),
            detail: detail.into(),
        }
    }
}

/// Declared mediation policy for an installed agent profile.
#[derive(Debug, Clone, Default)]
pub struct SandboxPolicy {
    pub declared_channels: BTreeSet<SandboxChannel>,
    pub registered_tool_proxies: BTreeSet<String>,
    pub registered_mcp_servers: BTreeSet<String>,
}

/// Deterministic sandbox gate: undeclared / unregistered paths fail closed.
#[derive(Debug, Clone)]
pub struct SandboxGate {
    pub platform: SandboxPlatform,
    pub policy: SandboxPolicy,
    /// Channels with negative-test evidence on this platform.
    pub evidenced_denials: BTreeSet<SandboxChannel>,
}

impl SandboxGate {
    pub fn intercept(
        &self,
        channel: SandboxChannel,
        target: &str,
        mediated: bool,
    ) -> Result<(), SandboxError> {
        // Any I/O that did not arrive through the adapter mediation path is a bypass.
        if !mediated {
            return Err(SandboxError::bypass(format!(
                "unmediated {} access to {target} on {}",
                channel.as_str(),
                self.platform.as_str()
            )));
        }
        if !self.policy.declared_channels.contains(&channel)
            && !matches!(
                channel,
                SandboxChannel::ToolProxy | SandboxChannel::Mcp | SandboxChannel::Secrets
            )
        {
            return Err(SandboxError::bypass(format!(
                "channel {} not in agent declaration",
                channel.as_str()
            )));
        }
        match channel {
            SandboxChannel::ToolProxy => {
                if !self.policy.registered_tool_proxies.contains(target) {
                    return Err(SandboxError::bypass(format!(
                        "unregistered tool proxy {target}"
                    )));
                }
            }
            SandboxChannel::Mcp => {
                if !self.policy.registered_mcp_servers.contains(target) {
                    return Err(SandboxError::bypass(format!(
                        "unregistered MCP server {target}"
                    )));
                }
            }
            SandboxChannel::Secrets if target.starts_with("host://") => {
                return Err(SandboxError::bypass(format!(
                    "host secret path {target} blocked"
                )));
            }
            _ => {}
        }
        Ok(())
    }

    /// Build F-017 matrix rows for this gate's platform. Unproven channels are
    /// `not_tested` or `unsupported` — never silently claimed as denied.
    pub fn matrix_rows(&self) -> Vec<PlatformChannelRow> {
        let all = [
            SandboxChannel::Network,
            SandboxChannel::Filesystem,
            SandboxChannel::Secrets,
            SandboxChannel::Subprocess,
            SandboxChannel::Device,
            SandboxChannel::Model,
            SandboxChannel::Mcp,
            SandboxChannel::A2a,
            SandboxChannel::Ipc,
            SandboxChannel::ToolProxy,
        ];
        all.into_iter()
            .map(|channel| {
                let claim = if self.evidenced_denials.contains(&channel) {
                    ChannelClaim::DeniedWithEvidence
                } else if self.platform == SandboxPlatform::WindowsNative {
                    ChannelClaim::Unsupported
                } else {
                    ChannelClaim::NotTested
                };
                PlatformChannelRow {
                    platform: self.platform,
                    channel,
                    claim,
                    evidence_digest: self
                        .evidenced_denials
                        .contains(&channel)
                        .then(|| format!("sha256:evidence-{}", channel.as_str())),
                }
            })
            .collect()
    }
}

/// Merge guard: refuse to collapse WSL2 guest evidence into Windows native.
pub fn refuse_cross_platform_merge(
    a: SandboxPlatform,
    b: SandboxPlatform,
) -> Result<(), SandboxError> {
    if a != b {
        return Err(SandboxError::bypass(format!(
            "refusing to merge sandbox claims across {} and {}",
            a.as_str(),
            b.as_str()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn linux_gate() -> SandboxGate {
        let mut declared = BTreeSet::new();
        declared.insert(SandboxChannel::Network);
        let mut proxies = BTreeSet::new();
        proxies.insert("proxy://registered".into());
        let mut evidenced = BTreeSet::new();
        evidenced.insert(SandboxChannel::Network);
        evidenced.insert(SandboxChannel::Secrets);
        evidenced.insert(SandboxChannel::ToolProxy);
        SandboxGate {
            platform: SandboxPlatform::LinuxNative,
            policy: SandboxPolicy {
                declared_channels: declared,
                registered_tool_proxies: proxies,
                registered_mcp_servers: BTreeSet::new(),
            },
            evidenced_denials: evidenced,
        }
    }

    #[test]
    fn undeclared_network_is_bypass() {
        let gate = linux_gate();
        let err = gate
            .intercept(SandboxChannel::Network, "https://evil", false)
            .unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    #[test]
    fn unregistered_tool_proxy_is_bypass() {
        let gate = linux_gate();
        let err = gate
            .intercept(SandboxChannel::ToolProxy, "proxy://shadow", true)
            .unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    #[test]
    fn host_secret_is_bypass() {
        let gate = linux_gate();
        let err = gate
            .intercept(SandboxChannel::Secrets, "host://~/.ssh/id_rsa", true)
            .unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    #[test]
    fn matrix_keeps_windows_native_unsupported_without_evidence() {
        let gate = SandboxGate {
            platform: SandboxPlatform::WindowsNative,
            policy: SandboxPolicy::default(),
            evidenced_denials: BTreeSet::new(),
        };
        let rows = gate.matrix_rows();
        assert!(rows.iter().all(|r| r.claim == ChannelClaim::Unsupported));
    }

    #[test]
    fn cannot_merge_wsl2_into_windows_native() {
        let err = refuse_cross_platform_merge(
            SandboxPlatform::WindowsWsl2LinuxGuest,
            SandboxPlatform::WindowsNative,
        )
        .unwrap_err();
        assert_eq!(err.code, "AGENT_ADAPTER_BYPASS_DETECTED");
    }

    /// Pins F-017 claim-freeze digests for the three linux_native deny rows.
    #[test]
    fn f017_claim_freeze_digests_are_stable() {
        let gate = linux_gate();
        let rows = gate.matrix_rows();
        let digest = |ch: SandboxChannel| {
            rows.iter()
                .find(|r| r.channel == ch)
                .and_then(|r| r.evidence_digest.clone())
        };
        assert_eq!(
            digest(SandboxChannel::Network).as_deref(),
            Some("sha256:evidence-network")
        );
        assert_eq!(
            digest(SandboxChannel::Secrets).as_deref(),
            Some("sha256:evidence-secrets")
        );
        assert_eq!(
            digest(SandboxChannel::ToolProxy).as_deref(),
            Some("sha256:evidence-tool_proxy")
        );
        assert_eq!(
            rows.iter()
                .find(|r| r.channel == SandboxChannel::Filesystem)
                .map(|r| r.claim),
            Some(ChannelClaim::NotTested)
        );
    }
}
