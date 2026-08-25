use cognitive_runtime::{
    PiLaunchFailure, PiLaunchRequest, PiPlatformClass, PiSandboxAdapterState, admit_pi_launch,
};

fn request() -> PiLaunchRequest {
    PiLaunchRequest {
        observed_platform: PiPlatformClass::LinuxNative,
        registered_policy_digest:
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        supplied_policy_digest:
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        expected_sandbox_adapter_digest:
            "sha256:2222222222222222222222222222222222222222222222222222222222222222".into(),
        supplied_sandbox_adapter_digest:
            "sha256:2222222222222222222222222222222222222222222222222222222222222222".into(),
        expected_compatibility_digest:
            "sha256:3333333333333333333333333333333333333333333333333333333333333333".into(),
        supplied_compatibility_digest:
            "sha256:3333333333333333333333333333333333333333333333333333333333333333".into(),
        sandbox_adapter: PiSandboxAdapterState::Ready,
        model_egress_proxy: Some("https://model-proxy.internal:8443".into()),
        allowed_model_endpoint: Some("https://api.deepseek.com".into()),
    }
}

#[test]
fn windows_native_is_unsupported_even_with_complete_configuration() {
    let mut value = request();
    value.observed_platform = PiPlatformClass::WindowsNative;

    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::WindowsNativeUnsupported)
    );
}

#[test]
fn wsl2_linux_guest_is_not_a_linux_native_claim() {
    let mut value = request();
    value.observed_platform = PiPlatformClass::WindowsWsl2LinuxGuest;

    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::Wsl2SeparatePlatform)
    );
}

#[test]
fn missing_proxy_fails_closed_before_any_pi_process_can_start() {
    let mut value = request();
    value.model_egress_proxy = None;

    #[cfg(target_os = "linux")]
    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::ModelEgressProxyMissing)
    );
    #[cfg(not(target_os = "linux"))]
    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::NotLinuxNativeHost)
    );
}

#[test]
fn unavailable_sandbox_adapter_fails_closed_before_any_pi_process_can_start() {
    let mut value = request();
    value.sandbox_adapter = PiSandboxAdapterState::Unavailable;

    #[cfg(target_os = "linux")]
    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::SandboxAdapterUnavailable)
    );
    #[cfg(not(target_os = "linux"))]
    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::NotLinuxNativeHost)
    );
}

#[test]
fn empty_policy_binding_cannot_be_an_exact_match() {
    let mut value = request();
    value.registered_policy_digest.clear();
    value.supplied_policy_digest.clear();

    #[cfg(target_os = "linux")]
    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::PolicyDigestMismatch)
    );
    #[cfg(not(target_os = "linux"))]
    assert_eq!(
        admit_pi_launch(&value),
        Err(PiLaunchFailure::NotLinuxNativeHost)
    );
}
