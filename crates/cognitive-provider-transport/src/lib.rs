//! Shared bounded HTTPS transport for Personal Provider egress.
//!
//! This crate is intentionally an adapter: secret resolution and Provider
//! discovery policy remain in `cognitive-secret`, while composition roots use
//! this implementation for the single allowed Rustls HTTP boundary.

use std::io::Read;
use std::time::Duration;

use cognitive_secret::{
    ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse, ProviderTransport,
    ProviderTransportError, StreamedProviderExchange,
};

const MAX_PROVIDER_RESPONSE_BYTES: usize = 1_048_576;
const MAX_PROVIDER_RESPONSE_READ_BYTES: u64 = 1_048_577;

/// Production HTTPS transport for OpenAI-compatible Provider requests.
///
/// The transport accepts only credential-free HTTPS request URLs, disables
/// redirects, uses Rustls, applies the caller's bounded timeout, and bounds
/// response bodies before returning them to a composition root.
#[derive(Debug, Default)]
pub struct RustlsProviderTransport {
    additional_root_certificates_der: Vec<Vec<u8>>,
}

impl RustlsProviderTransport {
    /// Construct a transport that additionally trusts one caller-supplied DER
    /// certificate while retaining every production transport policy.
    ///
    /// This seam supports hermetic HTTPS integration fixtures without adding a
    /// plaintext HTTP exception or changing the process-wide trust store.
    pub fn with_additional_root_certificate_der(
        certificate_der: Vec<u8>,
    ) -> Result<Self, ProviderTransportError> {
        reqwest::Certificate::from_der(&certificate_der).map_err(|_| {
            ProviderTransportError::Policy {
                detail: "additional Provider root certificate is invalid",
            }
        })?;
        Ok(Self {
            additional_root_certificates_der: vec![certificate_der],
        })
    }

    fn build_client(
        &self,
        timeout: Duration,
    ) -> Result<reqwest::blocking::Client, ProviderTransportError> {
        let mut client_builder = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .tcp_nodelay(true)
            .use_rustls_tls();
        for certificate_der in &self.additional_root_certificates_der {
            let certificate = reqwest::Certificate::from_der(certificate_der).map_err(|_| {
                ProviderTransportError::Policy {
                    detail: "additional Provider root certificate is invalid",
                }
            })?;
            client_builder = client_builder.add_root_certificate(certificate);
        }
        client_builder
            .build()
            .map_err(|_| ProviderTransportError::Backend {
                detail: "failed to construct Rustls Provider transport",
            })
    }
}

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
        let client = self.build_client(timeout)?;
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

    fn exchange_stream(
        &self,
        request: &ProviderHttpRequest,
        on_status: &mut dyn FnMut(u16) -> Result<(), ProviderTransportError>,
        on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
    ) -> Result<StreamedProviderExchange, ProviderTransportError> {
        validate_transport_request(request)?;
        if request.cancel_requested {
            return Err(ProviderTransportError::Timeout);
        }

        let timeout = Duration::from_millis(u64::from(request.timeout_ms));
        let client = self.build_client(timeout)?;
        let method = match request.method {
            ProviderHttpMethod::Get => reqwest::Method::GET,
            ProviderHttpMethod::Post => reqwest::Method::POST,
        };
        let mut request_builder = client.request(method, &request.url);
        let mut has_accept = false;
        for (header_name, header_value) in &request.headers {
            if header_name.eq_ignore_ascii_case("accept") {
                has_accept = true;
            }
            request_builder = request_builder.header(header_name, header_value);
        }
        if !has_accept {
            request_builder = request_builder.header("Accept", "text/event-stream");
        }
        if let Some(request_body) = &request.body {
            request_builder = request_builder.body(request_body.clone());
        }

        let network_started_at = std::time::Instant::now();
        let mut response = request_builder.send().map_err(map_reqwest_error)?;
        let status = response.status().as_u16();
        on_status(status)?;

        let mut body_bytes = 0_usize;
        let mut first_byte_nanos = None;
        let mut buffer = [0_u8; 4096];
        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|_| ProviderTransportError::Network {
                    detail: "failed to read Provider stream",
                })?;
            if read == 0 {
                break;
            }
            if body_bytes.saturating_add(read) > MAX_PROVIDER_RESPONSE_BYTES {
                return Err(ProviderTransportError::Policy {
                    detail: "Provider response exceeds local limit",
                });
            }
            if first_byte_nanos.is_none() {
                first_byte_nanos = Some(network_started_at.elapsed().as_nanos().max(1));
            }
            on_chunk(&buffer[..read])?;
            body_bytes += read;
        }

        let provider_network_elapsed_nanos = network_started_at.elapsed().as_nanos().max(1);
        Ok(StreamedProviderExchange {
            status,
            first_byte_nanos: first_byte_nanos.unwrap_or(provider_network_elapsed_nanos),
            provider_network_elapsed_nanos,
            body_bytes,
        })
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

/// Largest read-only fetch timeout the transport will accept.
pub const MAXIMUM_READ_ONLY_FETCH_TIMEOUT_MS: u32 = 30_000;

/// Verbs a read-only fetch may use. There is deliberately no request body and
/// no caller-supplied header, so this boundary cannot carry a credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyFetchMethod {
    Get,
    Head,
}

/// One bounded, credential-free read-only HTTP fetch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOnlyFetchRequest {
    pub method: ReadOnlyFetchMethod,
    pub url: String,
    pub timeout_ms: u32,
    pub maximum_response_bytes: usize,
}

/// A bounded read-only fetch response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOnlyFetchResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyFetchError {
    /// Refused before any egress.
    Policy { detail: &'static str },
    /// A request may have reached the network.
    Network { detail: &'static str },
    /// The bounded deadline expired.
    Timeout,
    /// A response arrived but exceeded the caller's bound, so none is returned.
    ResponseTooLarge,
}

/// The daemon's read-only outbound HTTP boundary.
///
/// It is separate from [`ProviderTransport`] on purpose: Provider egress
/// carries credentials to one configured endpoint, while this boundary carries
/// none and is used for caller-named origins. Keeping them apart means a Tool
/// fetch can never inherit Provider authorization.
pub trait ReadOnlyFetchTransport: Send + Sync {
    fn fetch(
        &self,
        request: &ReadOnlyFetchRequest,
    ) -> Result<ReadOnlyFetchResponse, ReadOnlyFetchError>;
}

/// Production read-only HTTPS fetch over the same Rustls stack as Provider
/// egress: no redirects, no proxy inherited from the ambient environment, no
/// request headers, no body, a bounded timeout and a bounded response.
#[derive(Debug, Default)]
pub struct RustlsReadOnlyFetchTransport {
    additional_root_certificates_der: Vec<Vec<u8>>,
}

impl RustlsReadOnlyFetchTransport {
    /// Trust one extra DER root so hermetic loopback fixtures can exercise the
    /// real TLS path without a plaintext exception or a process-wide change.
    pub fn with_additional_root_certificate_der(
        certificate_der: Vec<u8>,
    ) -> Result<Self, ReadOnlyFetchError> {
        reqwest::Certificate::from_der(&certificate_der).map_err(|_| {
            ReadOnlyFetchError::Policy {
                detail: "additional read-only fetch root certificate is invalid",
            }
        })?;
        Ok(Self {
            additional_root_certificates_der: vec![certificate_der],
        })
    }
}

impl ReadOnlyFetchTransport for RustlsReadOnlyFetchTransport {
    fn fetch(
        &self,
        request: &ReadOnlyFetchRequest,
    ) -> Result<ReadOnlyFetchResponse, ReadOnlyFetchError> {
        validate_read_only_fetch_request(request)?;
        let timeout = Duration::from_millis(u64::from(request.timeout_ms));
        let mut client_builder = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .timeout(timeout)
            .use_rustls_tls();
        for certificate_der in &self.additional_root_certificates_der {
            let certificate = reqwest::Certificate::from_der(certificate_der).map_err(|_| {
                ReadOnlyFetchError::Policy {
                    detail: "additional read-only fetch root certificate is invalid",
                }
            })?;
            client_builder = client_builder.add_root_certificate(certificate);
        }
        let client = client_builder
            .build()
            .map_err(|_| ReadOnlyFetchError::Network {
                detail: "failed to construct the read-only Rustls transport",
            })?;
        let method = match request.method {
            ReadOnlyFetchMethod::Get => reqwest::Method::GET,
            ReadOnlyFetchMethod::Head => reqwest::Method::HEAD,
        };
        let mut response = client
            .request(method, &request.url)
            .send()
            .map_err(map_read_only_fetch_error)?;
        let mut response_body = Vec::new();
        let read_ceiling = u64::try_from(request.maximum_response_bytes)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let bytes_read = response
            .by_ref()
            .take(read_ceiling)
            .read_to_end(&mut response_body)
            .map_err(|_| ReadOnlyFetchError::Network {
                detail: "failed to read the read-only fetch response",
            })?;
        if bytes_read > request.maximum_response_bytes {
            return Err(ReadOnlyFetchError::ResponseTooLarge);
        }
        Ok(ReadOnlyFetchResponse {
            status: response.status().as_u16(),
            body: response_body,
        })
    }
}

/// Refuse anything that must never reach the network, before it does.
pub fn validate_read_only_fetch_request(
    request: &ReadOnlyFetchRequest,
) -> Result<(), ReadOnlyFetchError> {
    if !request.url.starts_with("https://") {
        return Err(ReadOnlyFetchError::Policy {
            detail: "read-only fetch requires HTTPS",
        });
    }
    let authority = request
        .url
        .get("https://".len()..)
        .unwrap_or_default()
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    if authority.is_empty() || authority.contains('@') {
        return Err(ReadOnlyFetchError::Policy {
            detail: "read-only fetch URL must be credential-free",
        });
    }
    if request.timeout_ms == 0 || request.timeout_ms > MAXIMUM_READ_ONLY_FETCH_TIMEOUT_MS {
        return Err(ReadOnlyFetchError::Policy {
            detail: "read-only fetch timeout is outside the registered bound",
        });
    }
    if request.maximum_response_bytes == 0 {
        return Err(ReadOnlyFetchError::Policy {
            detail: "read-only fetch response bound must be positive",
        });
    }
    Ok(())
}

fn map_read_only_fetch_error(error: reqwest::Error) -> ReadOnlyFetchError {
    if error.is_timeout() {
        ReadOnlyFetchError::Timeout
    } else {
        ReadOnlyFetchError::Network {
            detail: "read-only HTTPS fetch failed",
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        ReadOnlyFetchError, ReadOnlyFetchMethod, ReadOnlyFetchRequest,
        validate_read_only_fetch_request, validate_transport_request,
    };
    use cognitive_secret::{ProviderHttpMethod, ProviderHttpRequest, ProviderTransportError};

    fn read_only_fetch_request(url: &str) -> ReadOnlyFetchRequest {
        ReadOnlyFetchRequest {
            method: ReadOnlyFetchMethod::Get,
            url: url.to_owned(),
            timeout_ms: 1_000,
            maximum_response_bytes: 4_096,
        }
    }

    #[test]
    fn read_only_fetch_refuses_unsafe_urls_and_bounds_before_egress() {
        for url in [
            "http://example.test/data",
            "https://user:secret@example.test/data",
            "https:///data",
        ] {
            assert!(
                matches!(
                    validate_read_only_fetch_request(&read_only_fetch_request(url)),
                    Err(ReadOnlyFetchError::Policy { .. })
                ),
                "unsafe read-only fetch URL must be refused before egress: {url}"
            );
        }

        let mut unbounded_timeout = read_only_fetch_request("https://example.test/data");
        unbounded_timeout.timeout_ms = super::MAXIMUM_READ_ONLY_FETCH_TIMEOUT_MS + 1;
        assert!(matches!(
            validate_read_only_fetch_request(&unbounded_timeout),
            Err(ReadOnlyFetchError::Policy { .. })
        ));

        let mut zero_timeout = read_only_fetch_request("https://example.test/data");
        zero_timeout.timeout_ms = 0;
        assert!(matches!(
            validate_read_only_fetch_request(&zero_timeout),
            Err(ReadOnlyFetchError::Policy { .. })
        ));

        let mut unbounded_response = read_only_fetch_request("https://example.test/data");
        unbounded_response.maximum_response_bytes = 0;
        assert!(matches!(
            validate_read_only_fetch_request(&unbounded_response),
            Err(ReadOnlyFetchError::Policy { .. })
        ));

        assert!(
            validate_read_only_fetch_request(&read_only_fetch_request("https://example.test/data"))
                .is_ok()
        );
    }

    fn transport_request(url: &str) -> ProviderHttpRequest {
        ProviderHttpRequest {
            method: ProviderHttpMethod::Get,
            url: url.to_owned(),
            headers: Vec::new(),
            body: None,
            timeout_ms: 1_000,
            cancel_requested: false,
        }
    }

    #[test]
    fn rejects_http_and_credential_bearing_urls_before_egress() {
        for url in [
            "http://provider.example/v1/models",
            "https://key@provider.example/v1/models",
        ] {
            let error = validate_transport_request(&transport_request(url))
                .expect_err("unsafe Provider URL must be rejected before egress");
            assert!(matches!(error, ProviderTransportError::Policy { .. }));
        }
    }

    #[test]
    fn rejects_header_injection_before_egress() {
        let mut request = transport_request("https://provider.example/v1/models");
        request.headers.push((
            "Authorization".to_owned(),
            "Bearer key\r\nX-Injected: true".to_owned(),
        ));

        let error = validate_transport_request(&request)
            .expect_err("line-break header must be rejected before egress");
        assert!(matches!(error, ProviderTransportError::Policy { .. }));
    }
}
