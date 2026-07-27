//! Daemon-owned, non-streaming Provider proxy for the Pi Personal surface.
//!
//! The proxy is deliberately not an authority writer. It only attaches the
//! daemon-resolved Provider credential to a bounded outbound HTTPS request.

use std::io::Read;
use std::time::Duration;

use cognitive_secret::{
    ProviderConfigRepository, ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse,
    ProviderKeyService, ProviderKeyServiceError, ProviderTransport, ProviderTransportError,
    SecretStore, SelectedModelRepository, bearer_authorization_header_value,
};

const PROVIDER_CHAT_COMPLETIONS_PATH: &str = "/chat/completions";
const MAX_PROVIDER_RESPONSE_BYTES: usize = 1_048_576;
const MAX_PROVIDER_RESPONSE_READ_BYTES: u64 = 1_048_577;

/// Stable failures exposed by the local Provider proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderProxyError {
    NotConfigured,
    SecretUnavailable,
    StreamingUnsupported,
    InvalidRequest,
    SelectedModelUnavailable,
    SelectedModelMismatch,
    TransportUnavailable,
    UpstreamRequestFailed,
}

impl ProviderProxyError {
    /// Stable wire error code. Never includes Provider or credential detail.
    pub fn code(self) -> &'static str {
        match self {
            Self::NotConfigured => "PERSONAL_PROVIDER_NOT_CONFIGURED",
            Self::SecretUnavailable => "PERSONAL_PROVIDER_SECRET_UNAVAILABLE",
            Self::StreamingUnsupported => "PERSONAL_PROVIDER_STREAMING_UNSUPPORTED",
            Self::InvalidRequest => "PERSONAL_PROVIDER_REQUEST_INVALID",
            Self::SelectedModelUnavailable => "PERSONAL_PROVIDER_SELECTED_MODEL_UNAVAILABLE",
            Self::SelectedModelMismatch => "PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH",
            Self::TransportUnavailable => "PERSONAL_PROVIDER_TRANSPORT_UNAVAILABLE",
            Self::UpstreamRequestFailed => "PERSONAL_PROVIDER_UPSTREAM_REQUEST_FAILED",
        }
    }

    /// HTTP status returned by the bounded local front door.
    pub fn status_code(self) -> u16 {
        match self {
            Self::NotConfigured
            | Self::SecretUnavailable
            | Self::SelectedModelUnavailable
            | Self::TransportUnavailable => 503,
            Self::StreamingUnsupported | Self::InvalidRequest | Self::SelectedModelMismatch => 400,
            Self::UpstreamRequestFailed => 502,
        }
    }
}

/// Daemon-side request service. The extension and Pi never receive secret bytes.
pub struct ProviderProxyService<'transport, T: ProviderTransport + ?Sized> {
    secret_store: &'transport dyn SecretStore,
    config_repository: ProviderConfigRepository,
    transport: &'transport T,
}

impl<'transport, T: ProviderTransport + ?Sized> ProviderProxyService<'transport, T> {
    /// Build a proxy around the daemon-owned secret backend and transport.
    pub fn new(
        secret_store: &'transport dyn SecretStore,
        config_repository: ProviderConfigRepository,
        transport: &'transport T,
    ) -> Self {
        Self {
            secret_store,
            config_repository,
            transport,
        }
    }

    /// Forward one non-streaming OpenAI-compatible chat completion request.
    pub fn forward_chat_completion(
        &self,
        request_body: &[u8],
    ) -> Result<ProviderHttpResponse, ProviderProxyError> {
        let requested_model = validate_chat_request(request_body)?;

        let provider_key_service =
            ProviderKeyService::new(self.secret_store, self.config_repository.clone());
        let provider_config = provider_key_service
            .load_config()
            .map_err(map_provider_key_error)?
            .ok_or(ProviderProxyError::NotConfigured)?;
        let selected_model =
            SelectedModelRepository::under_config_dir(self.config_repository.config_dir())
                .load()
                .map_err(|_| ProviderProxyError::SelectedModelUnavailable)?
                .ok_or(ProviderProxyError::SelectedModelUnavailable)?;
        if requested_model != selected_model.model_id() {
            return Err(ProviderProxyError::SelectedModelMismatch);
        }
        let provider_material = provider_key_service
            .resolve_provider_material()
            .map_err(map_provider_key_error)?;
        let authorization_value =
            bearer_authorization_header_value(provider_material.expose_bytes())
                .map_err(|_| ProviderProxyError::SecretUnavailable)?;
        let request = ProviderHttpRequest {
            method: ProviderHttpMethod::Post,
            url: format!(
                "{}{}",
                provider_config.base_url().trim_end_matches('/'),
                PROVIDER_CHAT_COMPLETIONS_PATH
            ),
            headers: vec![
                ("Authorization".to_owned(), authorization_value),
                ("Content-Type".to_owned(), "application/json".to_owned()),
            ],
            body: Some(request_body.to_vec()),
            timeout_ms: 60_000,
            cancel_requested: false,
        };
        self.transport
            .exchange(&request)
            .map_err(map_transport_error)
    }
}

/// Production HTTPS transport. The only TLS implementation is Rustls; no
/// Provider credential is put in a subprocess argument or inherited environment.
#[derive(Debug, Default)]
pub struct RustlsProviderTransport;

impl ProviderTransport for RustlsProviderTransport {
    fn exchange(
        &self,
        request: &ProviderHttpRequest,
    ) -> Result<ProviderHttpResponse, ProviderTransportError> {
        validate_transport_request(request)?;
        if request.cancel_requested {
            return Err(ProviderTransportError::Timeout);
        }

        let timeout = Duration::from_millis(u64::from(request.timeout_ms));
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .use_rustls_tls()
            .build()
            .map_err(|_| ProviderTransportError::Backend {
                detail: "failed to construct Rustls Provider transport",
            })?;
        let method = match request.method {
            ProviderHttpMethod::Get => reqwest::Method::GET,
            ProviderHttpMethod::Post => reqwest::Method::POST,
        };
        let mut request_builder = client.request(method, &request.url);
        for (header_name, header_value) in &request.headers {
            request_builder = request_builder.header(header_name, header_value);
        }
        if let Some(request_body) = &request.body {
            request_builder = request_builder.body(request_body.clone());
        }

        let mut response = request_builder.send().map_err(map_reqwest_error)?;
        let mut response_body = Vec::new();
        let bytes_read = response
            .by_ref()
            .take(MAX_PROVIDER_RESPONSE_READ_BYTES)
            .read_to_end(&mut response_body)
            .map_err(|_| ProviderTransportError::Network {
                detail: "failed to read Provider response",
            })?;
        if bytes_read > MAX_PROVIDER_RESPONSE_BYTES {
            return Err(ProviderTransportError::Policy {
                detail: "Provider response exceeds local limit",
            });
        }
        Ok(ProviderHttpResponse {
            status: response.status().as_u16(),
            body: response_body,
        })
    }
}

fn validate_chat_request(request_body: &[u8]) -> Result<String, ProviderProxyError> {
    let request_json: serde_json::Value =
        serde_json::from_slice(request_body).map_err(|_| ProviderProxyError::InvalidRequest)?;
    let request_object = request_json
        .as_object()
        .ok_or(ProviderProxyError::InvalidRequest)?;
    if request_object.get("stream") == Some(&serde_json::Value::Bool(true)) {
        return Err(ProviderProxyError::StreamingUnsupported);
    }
    request_object
        .get("model")
        .and_then(serde_json::Value::as_str)
        .filter(|model_id| !model_id.is_empty())
        .map(str::to_owned)
        .ok_or(ProviderProxyError::InvalidRequest)
}

fn map_provider_key_error(error: ProviderKeyServiceError) -> ProviderProxyError {
    match error {
        ProviderKeyServiceError::NotConfigured => ProviderProxyError::NotConfigured,
        ProviderKeyServiceError::SecretMissing | ProviderKeyServiceError::Secret(_) => {
            ProviderProxyError::SecretUnavailable
        }
        ProviderKeyServiceError::Config(_) | ProviderKeyServiceError::SelectedModel(_) => {
            ProviderProxyError::NotConfigured
        }
    }
}

fn map_transport_error(error: ProviderTransportError) -> ProviderProxyError {
    match error {
        ProviderTransportError::Policy { .. } | ProviderTransportError::Backend { .. } => {
            ProviderProxyError::TransportUnavailable
        }
        ProviderTransportError::Timeout | ProviderTransportError::Network { .. } => {
            ProviderProxyError::UpstreamRequestFailed
        }
    }
}

fn validate_transport_request(request: &ProviderHttpRequest) -> Result<(), ProviderTransportError> {
    if !request.url.starts_with("https://") || request.url.contains('@') {
        return Err(ProviderTransportError::Policy {
            detail: "Provider request URL must be credential-free HTTPS",
        });
    }
    if request.timeout_ms == 0 {
        return Err(ProviderTransportError::Policy {
            detail: "Provider request timeout must be non-zero",
        });
    }
    if request
        .headers
        .iter()
        .any(|(name, value)| name.contains(['\r', '\n']) || value.contains(['\r', '\n']))
    {
        return Err(ProviderTransportError::Policy {
            detail: "Provider request header contains an invalid line break",
        });
    }
    Ok(())
}

fn map_reqwest_error(error: reqwest::Error) -> ProviderTransportError {
    if error.is_timeout() {
        ProviderTransportError::Timeout
    } else {
        ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        }
    }
}

// Test doubles and temporary fixture setup intentionally use direct assertion
// failures; production proxy code remains subject to the strict lint set.
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        ProviderProxyError, ProviderProxyService, validate_chat_request, validate_transport_request,
    };
    use cognitive_secret::{
        EphemeralSecretStore, ProviderConfigRepository, ProviderHttpMethod, ProviderHttpRequest,
        ProviderHttpResponse, ProviderKeyService, ProviderTransport, ProviderTransportError,
        SecretMaterial, SelectedModel,
    };

    #[derive(Clone, Default)]
    struct CapturingTransport {
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
    }

    impl CapturingTransport {
        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().expect("capture lock").clone()
        }
    }

    impl ProviderTransport for CapturingTransport {
        fn exchange(
            &self,
            request: &ProviderHttpRequest,
        ) -> Result<ProviderHttpResponse, ProviderTransportError> {
            self.requests
                .lock()
                .expect("capture lock")
                .push(request.clone());
            Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{"id":"synthetic-completion"}"#.to_vec(),
            })
        }
    }

    fn temporary_provider_config_path() -> PathBuf {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "cognitiveos-p1-t07-provider-proxy-{timestamp_nanos}"
        ));
        std::fs::create_dir_all(&directory).expect("temporary config directory");
        directory.join("provider.json")
    }

    #[test]
    fn streaming_and_malformed_requests_are_rejected_before_secret_resolution() {
        assert_eq!(
            validate_chat_request(br#"{"stream":true,"messages":[]}"#),
            Err(ProviderProxyError::StreamingUnsupported)
        );
        assert_eq!(
            validate_chat_request(b"not-json"),
            Err(ProviderProxyError::InvalidRequest)
        );
    }

    #[test]
    fn transport_requires_https_and_rejects_header_injection() {
        let http_request = ProviderHttpRequest {
            method: ProviderHttpMethod::Post,
            url: "http://provider.invalid/v1/chat/completions".to_owned(),
            headers: Vec::new(),
            body: None,
            timeout_ms: 1,
            cancel_requested: false,
        };
        assert!(matches!(
            validate_transport_request(&http_request),
            Err(ProviderTransportError::Policy { .. })
        ));

        let injected_header_request = ProviderHttpRequest {
            url: "https://provider.invalid/v1/chat/completions".to_owned(),
            headers: vec![("X-Test".to_owned(), "safe\r\nunsafe".to_owned())],
            ..http_request
        };
        assert!(matches!(
            validate_transport_request(&injected_header_request),
            Err(ProviderTransportError::Policy { .. })
        ));
    }

    #[test]
    fn proxy_resolves_the_provider_key_only_for_daemon_owned_transport() {
        let config_path = temporary_provider_config_path();
        let config_repository = ProviderConfigRepository::from_file_path(&config_path);
        let secret_store = EphemeralSecretStore::default();
        let provider_key_service =
            ProviderKeyService::new(&secret_store, config_repository.clone());
        let synthetic_provider_key = "synthetic-provider-key-p1-t07";
        provider_key_service
            .configure_provider(
                "deepseek",
                "https://provider.example.invalid/v1",
                SecretMaterial::from_bytes(synthetic_provider_key.as_bytes().to_vec())
                    .expect("synthetic material"),
                None,
            )
            .expect("provider configuration");
        provider_key_service
            .selected_model_repository()
            .store(&SelectedModel::new("test-model", "fnv1a64:test", true).expect("selected model"))
            .expect("selected model store");
        let transport = CapturingTransport::default();
        let proxy = ProviderProxyService::new(&secret_store, config_repository, &transport);
        let request_body = br#"{"model":"test-model","stream":false,"messages":[]}"#;

        let response = proxy
            .forward_chat_completion(request_body)
            .expect("daemon-owned provider forwarding");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, br#"{"id":"synthetic-completion"}"#);
        let captured_requests = transport.requests();
        assert_eq!(captured_requests.len(), 1);
        let captured_request = &captured_requests[0];
        assert_eq!(
            captured_request.url,
            "https://provider.example.invalid/v1/chat/completions"
        );
        assert_eq!(
            captured_request.body.as_deref(),
            Some(request_body.as_slice())
        );
        assert_eq!(
            captured_request.headers,
            vec![
                (
                    "Authorization".to_owned(),
                    format!("Bearer {synthetic_provider_key}"),
                ),
                ("Content-Type".to_owned(), "application/json".to_owned()),
            ]
        );

        std::fs::remove_dir_all(
            config_path
                .parent()
                .expect("temporary config parent directory"),
        )
        .expect("temporary config cleanup");
    }

    #[test]
    fn proxy_rejects_an_unselected_model_before_provider_secret_resolution() {
        let config_path = temporary_provider_config_path();
        let config_repository = ProviderConfigRepository::from_file_path(&config_path);
        let secret_store = EphemeralSecretStore::default();
        let provider_key_service =
            ProviderKeyService::new(&secret_store, config_repository.clone());
        provider_key_service
            .configure_provider(
                "deepseek",
                "https://provider.example.invalid/v1",
                SecretMaterial::from_bytes(b"synthetic-provider-key-p1-t07".to_vec())
                    .expect("synthetic material"),
                None,
            )
            .expect("provider configuration");
        provider_key_service
            .selected_model_repository()
            .store(
                &SelectedModel::new("approved-model", "fnv1a64:approved", true)
                    .expect("selected model"),
            )
            .expect("selected model store");
        let transport = CapturingTransport::default();
        let proxy = ProviderProxyService::new(&secret_store, config_repository, &transport);

        assert!(matches!(
            proxy.forward_chat_completion(br#"{"model":"other-model","messages":[]}"#),
            Err(ProviderProxyError::SelectedModelMismatch)
        ));
        assert!(transport.requests().is_empty());

        std::fs::remove_dir_all(
            config_path
                .parent()
                .expect("temporary config parent directory"),
        )
        .expect("temporary config cleanup");
    }
}
