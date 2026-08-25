//! Fail-closed admission for a Linux-native, candidate-only Pi process.
//!
//! This module intentionally does not provide a host-process fallback. A caller
//! can receive a permit only on a Linux-native host with a registered, healthy
//! sandbox adapter and the exact policy/adapter/compatibility bindings. The
//! permit is not an authorization, capability, Effect, or task transition.

use cognitive_contracts::generated::error_registry::RegisteredErrorCode;

/// The platform classification recorded by the platform probe.
///
/// WSL2 is deliberately distinct from both Linux-native and Windows-native.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiPlatformClass {
    LinuxNative,
    WindowsWsl2LinuxGuest,
    WindowsNative,
}

/// Health and registration state reported by the Linux sandbox adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiSandboxAdapterState {
    Ready,
    NotRegistered,
    Unavailable,
    Faulted,
}

/// All bindings required before a governed Pi process may be considered for
/// launch. None of these inputs grant authority to Pi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiLaunchRequest {
    pub observed_platform: PiPlatformClass,
    pub registered_policy_digest: String,
    pub supplied_policy_digest: String,
    pub expected_sandbox_adapter_digest: String,
    pub supplied_sandbox_adapter_digest: String,
    pub expected_compatibility_digest: String,
    pub supplied_compatibility_digest: String,
    pub sandbox_adapter: PiSandboxAdapterState,
    pub model_egress_proxy: Option<String>,
    pub allowed_model_endpoint: Option<String>,
}

/// Why a launch was refused. Every variant maps to the registered
/// `AGENT_ADAPTER_BYPASS_DETECTED` error; this module introduces no error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiLaunchFailure {
    WindowsNativeUnsupported,
    Wsl2SeparatePlatform,
    NotLinuxNativeHost,
    PolicyDigestMismatch,
    SandboxAdapterDigestMismatch,
    CompatibilityDigestMismatch,
    SandboxAdapterNotRegistered,
    SandboxAdapterUnavailable,
    SandboxAdapterFaulted,
    ModelEgressProxyMissing,
    ModelEgressProxyInvalid,
    ModelEndpointNotAllowed,
}

impl PiLaunchFailure {
    pub fn code(self) -> &'static str {
        RegisteredErrorCode::AgentAdapterBypassDetected.as_str()
    }
}

/// An opaque pre-launch token. It records no authority and can only be created
/// by [`admit_pi_launch`]. A concrete Linux sandbox adapter is still required
/// to execute a process; this crate ships no permissive adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiLaunchPermit {
    policy_digest: String,
    sandbox_adapter_digest: String,
    compatibility_digest: String,
}

impl PiLaunchPermit {
    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    pub fn sandbox_adapter_digest(&self) -> &str {
        &self.sandbox_adapter_digest
    }

    pub fn compatibility_digest(&self) -> &str {
        &self.compatibility_digest
    }
}

/// Admit a candidate-only Pi launch on the governed Linux-native boundary.
///
/// Direct sockets and an arbitrary model endpoint have no route through this
/// API: the only admitted egress shape is a registered HTTPS proxy forwarding
/// to the one pre-registered DeepSeek endpoint. A missing or faulty dependency
/// is a refusal before a Pi process can be started.
pub fn admit_pi_launch(request: &PiLaunchRequest) -> Result<PiLaunchPermit, PiLaunchFailure> {
    match request.observed_platform {
        PiPlatformClass::WindowsNative => return Err(PiLaunchFailure::WindowsNativeUnsupported),
        PiPlatformClass::WindowsWsl2LinuxGuest => {
            return Err(PiLaunchFailure::Wsl2SeparatePlatform);
        }
        PiPlatformClass::LinuxNative => {}
    }

    // A request label is never enough to turn a Windows or other host into a
    // Linux-native claim. Linux behavior evidence must be produced on Linux.
    if !cfg!(target_os = "linux") {
        return Err(PiLaunchFailure::NotLinuxNativeHost);
    }

    if !is_sha256_digest(&request.registered_policy_digest)
        || !is_sha256_digest(&request.supplied_policy_digest)
        || request.registered_policy_digest != request.supplied_policy_digest
    {
        return Err(PiLaunchFailure::PolicyDigestMismatch);
    }
    if !is_sha256_digest(&request.expected_sandbox_adapter_digest)
        || !is_sha256_digest(&request.supplied_sandbox_adapter_digest)
        || request.expected_sandbox_adapter_digest != request.supplied_sandbox_adapter_digest
    {
        return Err(PiLaunchFailure::SandboxAdapterDigestMismatch);
    }
    if !is_sha256_digest(&request.expected_compatibility_digest)
        || !is_sha256_digest(&request.supplied_compatibility_digest)
        || request.expected_compatibility_digest != request.supplied_compatibility_digest
    {
        return Err(PiLaunchFailure::CompatibilityDigestMismatch);
    }
    match request.sandbox_adapter {
        PiSandboxAdapterState::Ready => {}
        PiSandboxAdapterState::NotRegistered => {
            return Err(PiLaunchFailure::SandboxAdapterNotRegistered);
        }
        PiSandboxAdapterState::Unavailable => {
            return Err(PiLaunchFailure::SandboxAdapterUnavailable);
        }
        PiSandboxAdapterState::Faulted => return Err(PiLaunchFailure::SandboxAdapterFaulted),
    }

    let proxy = request
        .model_egress_proxy
        .as_deref()
        .ok_or(PiLaunchFailure::ModelEgressProxyMissing)?;
    if !proxy.starts_with("https://") || proxy.contains('@') {
        return Err(PiLaunchFailure::ModelEgressProxyInvalid);
    }
    if request.allowed_model_endpoint.as_deref() != Some("https://api.deepseek.com") {
        return Err(PiLaunchFailure::ModelEndpointNotAllowed);
    }

    Ok(PiLaunchPermit {
        policy_digest: request.supplied_policy_digest.clone(),
        sandbox_adapter_digest: request.supplied_sandbox_adapter_digest.clone(),
        compatibility_digest: request.supplied_compatibility_digest.clone(),
    })
}

fn is_sha256_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn request() -> PiLaunchRequest {
        PiLaunchRequest {
            observed_platform: PiPlatformClass::LinuxNative,
            registered_policy_digest: format!("sha256:{}", "1".repeat(64)),
            supplied_policy_digest: format!("sha256:{}", "1".repeat(64)),
            expected_sandbox_adapter_digest: format!("sha256:{}", "2".repeat(64)),
            supplied_sandbox_adapter_digest: format!("sha256:{}", "2".repeat(64)),
            expected_compatibility_digest: format!("sha256:{}", "3".repeat(64)),
            supplied_compatibility_digest: format!("sha256:{}", "3".repeat(64)),
            sandbox_adapter: PiSandboxAdapterState::Ready,
            model_egress_proxy: Some("https://model-proxy.internal".into()),
            allowed_model_endpoint: Some("https://api.deepseek.com".into()),
        }
    }

    #[test]
    fn proxy_is_the_only_admitted_model_egress_shape() {
        let mut value = request();
        value.model_egress_proxy = Some("http://api.deepseek.com".into());
        #[cfg(target_os = "linux")]
        assert_eq!(
            admit_pi_launch(&value),
            Err(PiLaunchFailure::ModelEgressProxyInvalid)
        );
        #[cfg(not(target_os = "linux"))]
        assert_eq!(
            admit_pi_launch(&value),
            Err(PiLaunchFailure::NotLinuxNativeHost),
            "Windows test hosts must not manufacture a Linux-native launch claim"
        );
    }

    #[test]
    fn all_failures_use_the_registered_adapter_bypass_code() {
        assert_eq!(
            PiLaunchFailure::SandboxAdapterFaulted.code(),
            "AGENT_ADAPTER_BYPASS_DETECTED"
        );
    }
}
