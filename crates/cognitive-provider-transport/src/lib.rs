//! Shared bounded HTTPS transport for Personal Provider egress.
//!
//! This crate is intentionally an adapter: secret resolution and Provider
//! discovery policy remain in `cognitive-secret`, while composition roots use
//! this implementation for the single allowed Rustls HTTP boundary.

use std::io::Read;
use std::time::Duration;

use cognitive_secret::{
    ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse, ProviderTransport,
    ProviderTransportError,
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::validate_transport_request;
    use cognitive_secret::{ProviderHttpMethod, ProviderHttpRequest, ProviderTransportError};

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
