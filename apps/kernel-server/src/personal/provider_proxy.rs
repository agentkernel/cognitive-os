//! Daemon-owned Provider proxy for the Pi Personal surface.
//!
//! The proxy is deliberately not an authority writer. It only attaches the
//! daemon-resolved Provider credential to a bounded outbound HTTPS request.
//! Public `stream:true` requests are forwarded as SSE; private-candidate and
//! Pi conversation clients remain unary.

use std::time::Instant;

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

/// Daemon-private Provider response metadata that contains no request,
/// response, credential, or authority data.
pub struct TimedProviderResponse {
    pub response: ProviderHttpResponse,
    /// Config load, selected-model load and SecretStore resolution, measured
    /// separately so the Pi client's loopback wait is not read as network cost.
    pub preflight_elapsed_nanos: u128,
    pub provider_network_elapsed_nanos: u128,
}

/// Streaming public-proxy outcome. Body bytes are delivered only through the
/// caller's chunk callback.
pub struct TimedStreamedProviderResponse {
    pub status: u16,
    pub preflight_elapsed_nanos: u128,
    pub first_byte_nanos: u128,
    pub provider_network_elapsed_nanos: u128,
    pub body_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedChatRequest {
    model: String,
    stream: bool,
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
    #[cfg(test)]
    pub fn forward_chat_completion(
        &self,
        request_body: &[u8],
    ) -> Result<ProviderHttpResponse, ProviderProxyError> {
        self.forward_chat_completion_with_timing(request_body)
            .map(|timed_response| timed_response.response)
    }

    /// Forward one non-streaming completion and retain the duration of only
    /// the daemon-owned Provider transport exchange for campaign telemetry.
    pub fn forward_chat_completion_with_timing(
        &self,
        request_body: &[u8],
    ) -> Result<TimedProviderResponse, ProviderProxyError> {
        let validated = validate_chat_request(request_body)?;
        if validated.stream {
            return Err(ProviderProxyError::StreamingUnsupported);
        }
        self.forward_selected_chat_completion(validated.model, request_body)
    }

    /// Forward one public `stream:true` completion, flushing upstream SSE bytes
    /// through the caller's callbacks without waiting for a unary JSON body.
    ///
    /// `on_preflight` runs after SecretStore/selected-model work and before any
    /// upstream byte. `on_status` runs after the upstream status is known and
    /// before the first body chunk.
    pub fn forward_streaming_chat_completion(
        &self,
        request_body: &[u8],
        on_preflight: &mut dyn FnMut(u128) -> Result<(), ProviderProxyError>,
        on_status: &mut dyn FnMut(u16) -> Result<(), ProviderProxyError>,
        on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderProxyError>,
    ) -> Result<TimedStreamedProviderResponse, ProviderProxyError> {
        let validated = validate_chat_request(request_body)?;
        if !validated.stream {
            return Err(ProviderProxyError::InvalidRequest);
        }
        let (request, preflight_elapsed_nanos) =
            self.prepare_selected_request(validated.model, request_body, true)?;
        on_preflight(preflight_elapsed_nanos)?;
        let streamed = self
            .transport
            .exchange_stream(
                &request,
                &mut |status| on_status(status).map_err(proxy_callback_to_transport),
                &mut |chunk| on_chunk(chunk).map_err(proxy_callback_to_transport),
            )
            .map_err(map_transport_error)?;
        Ok(TimedStreamedProviderResponse {
            status: streamed.status,
            preflight_elapsed_nanos,
            first_byte_nanos: streamed.first_byte_nanos,
            provider_network_elapsed_nanos: streamed.provider_network_elapsed_nanos,
            body_bytes: streamed.body_bytes,
        })
    }

    /// Forward one private Pi candidate completion and reject any upstream
    /// response shape that could carry a tool call or multiple candidates.
    /// The Provider credential remains confined to this daemon-owned service.
    #[cfg(unix)]
    pub fn forward_private_candidate_completion(
        &self,
        request_body: &[u8],
    ) -> Result<ProviderHttpResponse, ProviderProxyError> {
        let sanitized_body = sanitize_private_candidate_request(request_body)?;
        let validated = validate_chat_request(&sanitized_body)?;
        if validated.stream {
            return Err(ProviderProxyError::StreamingUnsupported);
        }
        let timed_response =
            self.forward_selected_chat_completion(validated.model, &sanitized_body)?;
        validate_private_candidate_response(&timed_response.response)?;
        Ok(timed_response.response)
    }

    fn forward_selected_chat_completion(
        &self,
        requested_model: String,
        request_body: &[u8],
    ) -> Result<TimedProviderResponse, ProviderProxyError> {
        let (request, preflight_elapsed_nanos) =
            self.prepare_selected_request(requested_model, request_body, false)?;
        let provider_network_started_at = Instant::now();
        let response = self
            .transport
            .exchange(&request)
            .map_err(map_transport_error)?;
        Ok(TimedProviderResponse {
            response,
            preflight_elapsed_nanos,
            provider_network_elapsed_nanos: provider_network_started_at.elapsed().as_nanos().max(1),
        })
    }

    fn prepare_selected_request(
        &self,
        requested_model: String,
        request_body: &[u8],
        stream: bool,
    ) -> Result<(ProviderHttpRequest, u128), ProviderProxyError> {
        let preflight_started_at = Instant::now();
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
        let mut headers = vec![
            ("Authorization".to_owned(), authorization_value),
            ("Content-Type".to_owned(), "application/json".to_owned()),
        ];
        if stream {
            headers.push(("Accept".to_owned(), "text/event-stream".to_owned()));
        }
        let request = ProviderHttpRequest {
            method: ProviderHttpMethod::Post,
            url: format!(
                "{}{}",
                provider_config.base_url().trim_end_matches('/'),
                PROVIDER_CHAT_COMPLETIONS_PATH
            ),
            headers,
            body: Some(request_body.to_vec()),
            timeout_ms: 60_000,
            cancel_requested: false,
        };
        Ok((request, preflight_started_at.elapsed().as_nanos().max(1)))
    }
}

#[cfg(unix)]
fn sanitize_private_candidate_request(request_body: &[u8]) -> Result<Vec<u8>, ProviderProxyError> {
    let mut request_json: serde_json::Value =
        serde_json::from_slice(request_body).map_err(|_| ProviderProxyError::InvalidRequest)?;
    if let Some(object) = request_json.as_object_mut() {
        object.remove("tools");
        object.remove("tool_choice");
        object.remove("functions");
        object.remove("function_call");
    }
    serde_json::to_vec(&request_json).map_err(|_| ProviderProxyError::InvalidRequest)
}

#[cfg(unix)]
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
    if choices.len() != 1 {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    }
    let Some(message) = choices[0]
        .get("message")
        .and_then(serde_json::Value::as_object)
    else {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    };
    if !private_candidate_message_is_text_only(message) {
        return Err(ProviderProxyError::UpstreamRequestFailed);
    }
    Ok(())
}

#[cfg(unix)]
fn private_candidate_message_is_text_only(
    message: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    if !message
        .get("content")
        .is_some_and(serde_json::Value::is_string)
    {
        return false;
    }
    if message.contains_key("tool_calls") || message.contains_key("function_call") {
        return false;
    }
    match message.get("role") {
        None => true,
        Some(role) => role.as_str() == Some("assistant"),
    }
}

fn validate_chat_request(request_body: &[u8]) -> Result<ValidatedChatRequest, ProviderProxyError> {
    let request_json: serde_json::Value =
        serde_json::from_slice(request_body).map_err(|_| ProviderProxyError::InvalidRequest)?;
    let request_object = request_json
        .as_object()
        .ok_or(ProviderProxyError::InvalidRequest)?;
    let stream = request_object.get("stream") == Some(&serde_json::Value::Bool(true));
    let model = request_object
        .get("model")
        .and_then(serde_json::Value::as_str)
        .filter(|model_id| !model_id.is_empty())
        .map(str::to_owned)
        .ok_or(ProviderProxyError::InvalidRequest)?;
    Ok(ValidatedChatRequest { model, stream })
}

fn proxy_callback_to_transport(_error: ProviderProxyError) -> ProviderTransportError {
    ProviderTransportError::Network {
        detail: "streaming callback failed",
    }
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
        ProviderProxyError, ProviderProxyService, ValidatedChatRequest, validate_chat_request,
    };
    #[cfg(unix)]
    use super::{sanitize_private_candidate_request, validate_private_candidate_response};
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

    struct DelayedStreamTransport {
        delay: std::time::Duration,
        chunks: Vec<Vec<u8>>,
    }

    impl ProviderTransport for DelayedStreamTransport {
        fn exchange(
            &self,
            _request: &ProviderHttpRequest,
        ) -> Result<ProviderHttpResponse, ProviderTransportError> {
            Err(ProviderTransportError::Policy {
                detail: "delayed stream transport is streaming-only",
            })
        }

        fn exchange_stream(
            &self,
            _request: &ProviderHttpRequest,
            on_status: &mut dyn FnMut(u16) -> Result<(), ProviderTransportError>,
            on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
        ) -> Result<cognitive_secret::StreamedProviderExchange, ProviderTransportError> {
            let started = std::time::Instant::now();
            on_status(200)?;
            let mut body_bytes = 0_usize;
            let mut first_byte_nanos = None;
            for (index, chunk) in self.chunks.iter().enumerate() {
                if index > 0 {
                    std::thread::sleep(self.delay);
                }
                if first_byte_nanos.is_none() && !chunk.is_empty() {
                    first_byte_nanos = Some(started.elapsed().as_nanos().max(1));
                }
                on_chunk(chunk)?;
                body_bytes += chunk.len();
            }
            let provider_network_elapsed_nanos = started.elapsed().as_nanos().max(1);
            Ok(cognitive_secret::StreamedProviderExchange {
                status: 200,
                first_byte_nanos: first_byte_nanos.unwrap_or(provider_network_elapsed_nanos),
                provider_network_elapsed_nanos,
                body_bytes,
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
    fn streaming_requests_are_validated_without_rejecting_stream_true() {
        assert_eq!(
            validate_chat_request(br#"{"stream":true,"messages":[]}"#),
            Err(ProviderProxyError::InvalidRequest)
        );
        assert_eq!(
            validate_chat_request(br#"{"model":"test-model","stream":true,"messages":[]}"#),
            Ok(ValidatedChatRequest {
                model: "test-model".to_owned(),
                stream: true,
            })
        );
        assert_eq!(
            validate_chat_request(b"not-json"),
            Err(ProviderProxyError::InvalidRequest)
        );
    }

    #[test]
    fn unary_forward_still_refuses_stream_true() {
        let transport = CapturingTransport::default();
        let config_path = temporary_provider_config_path();
        let store = EphemeralSecretStore::default();
        let service = ProviderProxyService::new(
            &store,
            ProviderConfigRepository::from_file_path(&config_path),
            &transport,
        );
        assert_eq!(
            service
                .forward_chat_completion_with_timing(
                    br#"{"model":"test-model","stream":true,"messages":[]}"#
                )
                .expect_err("unary path must refuse stream:true"),
            ProviderProxyError::StreamingUnsupported
        );
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn streaming_forward_flushes_the_first_chunk_before_the_delayed_last() {
        let config_path = temporary_provider_config_path();
        let config_repository = ProviderConfigRepository::from_file_path(&config_path);
        let secret_store = EphemeralSecretStore::default();
        let provider_key_service =
            ProviderKeyService::new(&secret_store, config_repository.clone());
        provider_key_service
            .configure_provider(
                "deepseek",
                "https://provider.example.invalid/v1",
                SecretMaterial::from_bytes(b"synthetic-provider-key-p8-t11".to_vec())
                    .expect("synthetic material"),
                None,
            )
            .expect("provider configuration");
        provider_key_service
            .selected_model_repository()
            .store(&SelectedModel::new("test-model", "fnv1a64:test", true).expect("selected model"))
            .expect("selected model store");
        let transport = DelayedStreamTransport {
            delay: std::time::Duration::from_millis(250),
            chunks: vec![
                b"data: {\"delta\":\"first\"}\n\n".to_vec(),
                b"data: [DONE]\n\n".to_vec(),
            ],
        };
        let proxy = ProviderProxyService::new(&secret_store, config_repository, &transport);
        let chunk_times = std::sync::Arc::new(Mutex::new(Vec::new()));
        let started = std::time::Instant::now();
        let mut on_preflight = |_nanos: u128| Ok(());
        let mut on_status = |_status: u16| Ok(());
        let mut on_chunk = {
            let chunk_times = std::sync::Arc::clone(&chunk_times);
            move |chunk: &[u8]| {
                if !chunk.is_empty() {
                    chunk_times
                        .lock()
                        .expect("chunk times")
                        .push(started.elapsed());
                }
                Ok(())
            }
        };
        let streamed = proxy
            .forward_streaming_chat_completion(
                br#"{"model":"test-model","stream":true,"messages":[]}"#,
                &mut on_preflight,
                &mut on_status,
                &mut on_chunk,
            )
            .expect("streaming forward");
        assert_eq!(streamed.status, 200);
        assert!(streamed.first_byte_nanos > 0);
        assert!(streamed.first_byte_nanos <= streamed.provider_network_elapsed_nanos);
        let times = chunk_times.lock().expect("chunk times");
        assert!(times.len() >= 2, "expected at least two flushed chunks");
        assert!(
            times[times.len() - 1].saturating_sub(times[0])
                >= std::time::Duration::from_millis(150),
            "first chunk must flush before the delayed last chunk"
        );
        std::fs::remove_dir_all(
            config_path
                .parent()
                .expect("temporary config parent directory"),
        )
        .expect("temporary config cleanup");
    }

    #[test]
    #[cfg(unix)]
    fn private_candidate_provider_response_requires_one_text_choice() {
        let valid_response = ProviderHttpResponse {
            status: 200,
            body: br#"{"choices":[{"message":{"content":"candidate"}}]}"#.to_vec(),
        };
        assert_eq!(validate_private_candidate_response(&valid_response), Ok(()));

        let role_response = ProviderHttpResponse {
            status: 200,
            body: br#"{"choices":[{"index":0,"message":{"role":"assistant","content":"candidate"},"finish_reason":"stop"}]}"#
                .to_vec(),
        };
        assert_eq!(validate_private_candidate_response(&role_response), Ok(()));

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
    #[cfg(unix)]
    fn private_candidate_request_strips_tool_surfaces_before_forwarding() {
        let sanitized = sanitize_private_candidate_request(
            br#"{"model":"test-model","stream":false,"messages":[],"tools":[{"type":"function"}],"tool_choice":"auto"}"#,
        )
        .expect("request with tools must still parse");
        let parsed: serde_json::Value =
            serde_json::from_slice(&sanitized).expect("sanitized request is JSON");
        let object = parsed.as_object().expect("sanitized request is an object");
        assert_eq!(
            object.get("model").and_then(serde_json::Value::as_str),
            Some("test-model")
        );
        assert!(!object.contains_key("tools"));
        assert!(!object.contains_key("tool_choice"));
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
    fn timed_forward_splits_preflight_from_the_provider_network_without_changing_the_body() {
        // preflight 覆盖配置/selected-model/SecretStore；网络阶段只覆盖 transport
        // exchange。两者都必须为正，且完成 body 必须与 transport 返回值逐字节相同。
        let config_path = temporary_provider_config_path();
        let config_repository = ProviderConfigRepository::from_file_path(&config_path);
        let secret_store = EphemeralSecretStore::default();
        let provider_key_service =
            ProviderKeyService::new(&secret_store, config_repository.clone());
        provider_key_service
            .configure_provider(
                "deepseek",
                "https://provider.example.invalid/v1",
                SecretMaterial::from_bytes(b"synthetic-provider-key-p9-t07".to_vec())
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

        let timed = proxy
            .forward_chat_completion_with_timing(request_body)
            .expect("timed daemon-owned provider forwarding");

        assert_eq!(timed.response.status, 200);
        assert_eq!(timed.response.body, br#"{"id":"synthetic-completion"}"#);
        assert!(
            timed.preflight_elapsed_nanos >= 1,
            "preflight must not report zero"
        );
        assert!(
            timed.provider_network_elapsed_nanos >= 1,
            "provider network must not report zero"
        );
        assert_eq!(transport.requests().len(), 1);

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
