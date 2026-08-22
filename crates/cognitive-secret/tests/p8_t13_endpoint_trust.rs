//! Public endpoint-trust surface for P8-T13 (std-only; no Secret Store).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use cognitive_secret::{
    EndpointTrustError, EndpointTrustGrant, ProviderKind, TrustedEndpoint, reject_caller_headers,
};

fn public_grant() -> EndpointTrustGrant {
    EndpointTrustGrant {
        allow_private_network: false,
        allow_insecure_http: false,
    }
}

#[test]
fn public_api_refuses_anthropic_compatible_http_private_and_header_injection() {
    assert_eq!(
        ProviderKind::parse("anthropic_compatible"),
        Err(EndpointTrustError::UnsupportedAnthropicCompatible)
    );
    assert_eq!(
        TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("http://api.example.test/v1"),
            public_grant()
        ),
        Err(EndpointTrustError::InsecureHttpRequiresGrant)
    );
    assert_eq!(
        TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("https://127.0.0.1/v1"),
            public_grant()
        ),
        Err(EndpointTrustError::PrivateNetworkRequiresGrant)
    );
    assert_eq!(
        reject_caller_headers(
            &[("X-Api-Key".to_owned(), "secret".to_owned())],
            ProviderKind::OpenaiCompatible
        ),
        Err(EndpointTrustError::ArbitraryHeaderForbidden)
    );
}
