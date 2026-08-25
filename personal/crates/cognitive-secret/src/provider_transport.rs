//! Injectable Provider HTTP transport for discovery and capability probes.
//!
//! Production daemons inject an HTTPS-capable implementation. Tests inject a
//! hermetic mock. Authorization header values and response bodies are redacted
//! in Debug/Display. This module is not an authority writer.

use std::fmt;

/// HTTP method subset used by OpenAI-compatible Provider probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHttpMethod {
    /// GET (model catalog).
    Get,
    /// POST (chat/completions probes).
    Post,
}

impl ProviderHttpMethod {
    /// Upper-case HTTP method token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

/// One outbound Provider request. Secret material may appear only in headers
/// constructed by the discovery service for the duration of the exchange.
#[derive(Clone)]
pub struct ProviderHttpRequest {
    /// GET or POST.
    pub method: ProviderHttpMethod,
    /// Absolute HTTPS URL. Credentials in the URL are rejected by callers.
    pub url: String,
    /// Request headers. Authorization values must never be logged.
    pub headers: Vec<(String, String)>,
    /// Optional JSON body bytes.
    pub body: Option<Vec<u8>>,
    /// Soft timeout budget in milliseconds for this exchange.
    pub timeout_ms: u32,
    /// When true, the transport must abort the exchange (cancel probe path).
    pub cancel_requested: bool,
}

impl fmt::Debug for ProviderHttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderHttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &redacted_headers(&self.headers))
            .field(
                "body",
                &self
                    .body
                    .as_ref()
                    .map(|bytes| format!("<redacted-body len={}>", bytes.len())),
            )
            .field("timeout_ms", &self.timeout_ms)
            .field("cancel_requested", &self.cancel_requested)
            .finish()
    }
}

/// Raw Provider HTTP response. Body content is never shown by Debug/Display.
#[derive(Clone)]
pub struct ProviderHttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Raw response body bytes.
    pub body: Vec<u8>,
}

/// Metadata for one streaming Provider exchange. Body bytes are delivered only
/// through the caller's chunk callback and are never retained here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamedProviderExchange {
    /// HTTP status code from the upstream Provider.
    pub status: u16,
    /// Wall time from dispatch until the first non-empty body byte, or the
    /// full network duration when the body is empty.
    pub first_byte_nanos: u128,
    /// Wall time of the complete upstream exchange.
    pub provider_network_elapsed_nanos: u128,
    /// Bounded number of body bytes delivered to the caller.
    pub body_bytes: usize,
}

impl fmt::Debug for ProviderHttpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderHttpResponse")
            .field("status", &self.status)
            .field("body", &format!("<redacted-body len={}>", self.body.len()))
            .finish()
    }
}

impl fmt::Display for ProviderHttpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "ProviderHttpResponse(status={},body_len={})",
            self.status,
            self.body.len()
        )
    }
}

/// Failures raised by a transport before any Provider business interpretation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderTransportError {
    /// Soft timeout or cancel aborted the exchange.
    Timeout,
    /// Local policy rejected the request before network I/O.
    Policy { detail: &'static str },
    /// Network or TLS failure without embedding response bodies.
    Network { detail: &'static str },
    /// Transport implementation failed without secret content.
    Backend { detail: &'static str },
}

impl fmt::Display for ProviderTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(formatter, "provider transport timeout"),
            Self::Policy { detail } => write!(formatter, "provider transport policy: {detail}"),
            Self::Network { detail } => write!(formatter, "provider transport network: {detail}"),
            Self::Backend { detail } => write!(formatter, "provider transport backend: {detail}"),
        }
    }
}

impl std::error::Error for ProviderTransportError {}

/// Injectable HTTP transport used by discovery and capability probes.
///
/// Implementations must:
/// - never log Authorization header values or secret material
/// - honor `cancel_requested` by aborting without treating abort as success body
/// - enforce HTTPS at the composition boundary (daemon wiring)
pub trait ProviderTransport {
    /// Perform one Provider HTTP exchange.
    fn exchange(
        &self,
        request: &ProviderHttpRequest,
    ) -> Result<ProviderHttpResponse, ProviderTransportError>;

    /// Perform one streaming Provider HTTP exchange.
    ///
    /// `on_status` is invoked once the upstream status is known and before any
    /// body byte. `on_chunk` is invoked for each non-empty body fragment in
    /// arrival order. Implementations must not log Authorization values or
    /// body bytes. The default is a policy refusal so existing unary transports
    /// stay fail-closed.
    fn exchange_stream(
        &self,
        request: &ProviderHttpRequest,
        on_status: &mut dyn FnMut(u16) -> Result<(), ProviderTransportError>,
        on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
    ) -> Result<StreamedProviderExchange, ProviderTransportError> {
        let _ = (request, on_status, on_chunk);
        Err(ProviderTransportError::Policy {
            detail: "streaming Provider exchange is not implemented by this transport",
        })
    }
}

impl<T: ProviderTransport + ?Sized> ProviderTransport for &T {
    fn exchange(
        &self,
        request: &ProviderHttpRequest,
    ) -> Result<ProviderHttpResponse, ProviderTransportError> {
        (**self).exchange(request)
    }

    fn exchange_stream(
        &self,
        request: &ProviderHttpRequest,
        on_status: &mut dyn FnMut(u16) -> Result<(), ProviderTransportError>,
        on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
    ) -> Result<StreamedProviderExchange, ProviderTransportError> {
        (**self).exchange_stream(request, on_status, on_chunk)
    }
}

impl<T: ProviderTransport + ?Sized> ProviderTransport for std::sync::Arc<T> {
    fn exchange(
        &self,
        request: &ProviderHttpRequest,
    ) -> Result<ProviderHttpResponse, ProviderTransportError> {
        (**self).exchange(request)
    }

    fn exchange_stream(
        &self,
        request: &ProviderHttpRequest,
        on_status: &mut dyn FnMut(u16) -> Result<(), ProviderTransportError>,
        on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
    ) -> Result<StreamedProviderExchange, ProviderTransportError> {
        (**self).exchange_stream(request, on_status, on_chunk)
    }
}

/// Header name that may carry the Provider API key.
pub const AUTHORIZATION_HEADER_NAME: &str = "Authorization";
/// Official Anthropic native key header. Callers cannot supply this template.
pub const ANTHROPIC_API_KEY_HEADER_NAME: &str = "x-api-key";
/// Official Anthropic version header name.
pub const ANTHROPIC_VERSION_HEADER_NAME: &str = "anthropic-version";
/// Pinned Anthropic API version for official adapter requests.
pub const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Build a Bearer Authorization header value from UTF-8 secret material.
///
/// The returned string is secret-bearing for the lifetime of the request only.
/// Callers must not log it, persist it, or include it in errors.
pub fn bearer_authorization_header_value(
    material: &[u8],
) -> Result<String, ProviderTransportError> {
    let token = std::str::from_utf8(material).map_err(|_| ProviderTransportError::Policy {
        detail: "provider api key material is not valid utf-8",
    })?;
    if token.is_empty() {
        return Err(ProviderTransportError::Policy {
            detail: "provider api key material is empty",
        });
    }
    if token.chars().any(|character| character.is_control()) {
        return Err(ProviderTransportError::Policy {
            detail: "provider api key material contains control characters",
        });
    }
    Ok(format!("Bearer {token}"))
}

/// True when a header name is Authorization (ASCII case-insensitive).
pub fn is_authorization_header_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(AUTHORIZATION_HEADER_NAME)
        || name.eq_ignore_ascii_case(ANTHROPIC_API_KEY_HEADER_NAME)
}

/// Build the official Anthropic `x-api-key` header value from UTF-8 material.
///
/// Callers must not log, persist, or include this value in errors.
pub fn anthropic_api_key_header_value(material: &[u8]) -> Result<String, ProviderTransportError> {
    let token = std::str::from_utf8(material).map_err(|_| ProviderTransportError::Policy {
        detail: "provider api key material is not valid utf-8",
    })?;
    if token.is_empty() {
        return Err(ProviderTransportError::Policy {
            detail: "provider api key material is empty",
        });
    }
    if token.chars().any(|character| character.is_control()) {
        return Err(ProviderTransportError::Policy {
            detail: "provider api key material contains control characters",
        });
    }
    Ok(token.to_owned())
}

/// Redact header values that may carry credentials for Debug output.
pub fn redacted_headers(headers: &[(String, String)]) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            if is_authorization_header_name(name) {
                (
                    name.clone(),
                    format!("<redacted-authorization len={}>", value.len()),
                )
            } else {
                (name.clone(), value.clone())
            }
        })
        .collect()
}
