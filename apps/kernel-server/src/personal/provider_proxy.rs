//! Daemon-owned, non-streaming Provider proxy for the Pi Personal surface.
//!
//! The proxy is deliberately not an authority writer. It only attaches the
//! daemon-resolved Provider credential to a bounded outbound HTTPS request.

pub use cognitive_provider_transport::RustlsProviderTransport;
use cognitive_secret::{
    ProviderConfigRepository, ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse,
    ProviderKeyService, ProviderKeyServiceError, ProviderTransport, ProviderTransportError,
    SecretStore, SelectedModelRepository, bearer_authorization_header_value,
};

const PROVIDER_CHAT_COMPLETIONS_PATH: &str = "/chat/completions";

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
        self.forward_selected_chat_completion(requested_model, request_body)
    }

    /// Forward one private Pi candidate completion and reject any upstream
    /// response shape that could carry a tool call or multiple candidates.
    /// The Provider credential remains confined to this daemon-owned service.
    pub fn forward_private_candidate_completion(
        &self,
        request_body: &[u8],
    ) -> Result<ProviderHttpResponse, ProviderProxyError> {
        let requested_model = validate_chat_request(request_body)?;
        let response = self.forward_selected_chat_completion(requested_model, request_body)?;
        validate_private_candidate_response(&response)?;
        Ok(response)
    }

    fn forward_selected_chat_completion(
        &self,
        requested_model: String,
        request_body: &[u8],
    ) -> Result<ProviderHttpResponse, ProviderProxyError> {
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

fn validate_private_candidate_response(
    response: &ProviderHttpResponse,
) -> Result<(), ProviderProxyError> {
    if response.status != 200 {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    }
    let response_json: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|_| ProviderProxyError::UpstreamRequestFailed)?;
    let Some(choices) = response_json
        .get("choices")
        .and_then(serde_json::Value::as_array)
    else {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    };
    let Some(choice) = choices.first() else {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    };
    if choices.len() != 1
        || choice
            .get("message")
            .and_then(serde_json::Value::as_object)
            .is_none_or(|message| {
                message.len() != 1
                    || !message
                        .get("content")
                        .is_some_and(serde_json::Value::is_string)
            })
    {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    }
    Ok(())
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

// Test doubles and temporary fixture setup intentionally use direct assertion
// failures; production proxy code remains subject to the strict lint set.
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        ProviderProxyError, ProviderProxyService, validate_chat_request,
        validate_private_candidate_response,
    };
    use cognitive_secret::{
        EphemeralSecretStore, ProviderConfigRepository, ProviderHttpRequest, ProviderHttpResponse,
        ProviderKeyService, ProviderTransport, ProviderTransportError, SecretMaterial,
        SelectedModel,
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
    fn private_candidate_provider_response_requires_one_text_choice() {
        let valid_response = ProviderHttpResponse {
            status: 200,
            body: br#"{"choices":[{"message":{"content":"candidate"}}]}"#.to_vec(),
        };
        assert_eq!(validate_private_candidate_response(&valid_response), Ok(()));

        let tool_response = ProviderHttpResponse {
            status: 200,
            body: br#"{"choices":[{"message":{"content":"candidate","tool_calls":[]}}]}"#.to_vec(),
        };
        assert_eq!(
            validate_private_candidate_response(&tool_response),
            Err(ProviderProxyError::UpstreamRequestFailed)
        );

        let multiple_choice_response = ProviderHttpResponse {
            status: 200,
            body: br#"{"choices":[{"message":{"content":"one"}},{"message":{"content":"two"}}]}"#
                .to_vec(),
        };
        assert_eq!(
            validate_private_candidate_response(&multiple_choice_response),
            Err(ProviderProxyError::UpstreamRequestFailed)
        );
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
