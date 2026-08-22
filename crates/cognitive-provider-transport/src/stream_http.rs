//! HTTP/1.1 SSE streaming over Rustls.
//!
//! reqwest's blocking `Read` waits for later body bytes before delivering the
//! first SSE event (observed ~250 ms on the delayed-sse fixture). This path
//! parses headers, then forwards leftover and subsequent TLS records immediately.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::{CertificateDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use cognitive_secret::{
    ProviderHttpMethod, ProviderHttpRequest, ProviderTransportError, StreamedProviderExchange,
};

use crate::{MAX_PROVIDER_RESPONSE_BYTES, is_provider_stream_closed};

const MAX_HEADER_BYTES: usize = 65_536;
const STREAM_READ_BYTES: usize = 4_096;

pub(crate) fn exchange_stream(
    additional_root_certificates_der: &[Vec<u8>],
    request: &ProviderHttpRequest,
    on_status: &mut dyn FnMut(u16) -> Result<(), ProviderTransportError>,
    on_chunk: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
) -> Result<StreamedProviderExchange, ProviderTransportError> {
    let timeout = Duration::from_millis(u64::from(request.timeout_ms));
    let (host, port, path) = split_https_url(&request.url)?;
    let mut tls = connect_tls(additional_root_certificates_der, &host, port, timeout)?;
    tls.conn.set_buffer_limit(Some(1));
    write_http_request(&mut tls, &host, port, path.as_str(), request)?;
    flush_tls(&mut tls)?;

    let network_started_at = Instant::now();
    let mut reader = TlsHttp {
        tls,
        leftover: Vec::new(),
    };
    let (status, chunked, content_length) = read_status_and_headers(&mut reader)?;
    on_status(status)?;

    let mut body_bytes = 0_usize;
    let mut first_byte_nanos = None;
    let mut deliver = |chunk: &[u8]| -> Result<(), ProviderTransportError> {
        if chunk.is_empty() {
            return Ok(());
        }
        if body_bytes.saturating_add(chunk.len()) > MAX_PROVIDER_RESPONSE_BYTES {
            return Err(ProviderTransportError::Policy {
                detail: "Provider response exceeds local limit",
            });
        }
        if first_byte_nanos.is_none() {
            first_byte_nanos = Some(network_started_at.elapsed().as_nanos().max(1));
        }
        on_chunk(chunk)?;
        body_bytes += chunk.len();
        Ok(())
    };

    if chunked {
        read_chunked_body(&mut reader, &mut deliver)?;
    } else if let Some(length) = content_length {
        read_exact_body(&mut reader, length, &mut deliver)?;
    } else {
        read_until_close(&mut reader, &mut deliver)?;
    }

    let provider_network_elapsed_nanos = network_started_at.elapsed().as_nanos().max(1);
    Ok(StreamedProviderExchange {
        status,
        first_byte_nanos: first_byte_nanos.unwrap_or(provider_network_elapsed_nanos),
        provider_network_elapsed_nanos,
        body_bytes,
    })
}

fn split_https_url(url: &str) -> Result<(String, u16, String), ProviderTransportError> {
    let without_scheme = url
        .strip_prefix("https://")
        .ok_or(ProviderTransportError::Policy {
            detail: "Provider request URL must be credential-free HTTPS",
        })?;
    let (authority, path) = match without_scheme.split_once('/') {
        Some((authority, rest)) => (authority, format!("/{rest}")),
        None => (without_scheme, "/".to_owned()),
    };
    if authority.is_empty() || authority.contains('@') || authority.contains('[') {
        return Err(ProviderTransportError::Policy {
            detail: "Provider request URL must be credential-free HTTPS",
        });
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port_text))
            if !host.is_empty() && port_text.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            let port: u16 = port_text
                .parse()
                .map_err(|_| ProviderTransportError::Policy {
                    detail: "Provider request URL port is invalid",
                })?;
            (host.to_owned(), port)
        }
        _ => (authority.to_owned(), 443),
    };
    Ok((host, port, path))
}

fn connect_tls(
    additional_root_certificates_der: &[Vec<u8>],
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<StreamOwned<ClientConnection, TcpStream>, ProviderTransportError> {
    let mut roots = RootCertStore::empty();
    let native = rustls_native_certs::load_native_certs();
    for certificate in native.certs {
        let _ = roots.add(certificate);
    }
    for certificate_der in additional_root_certificates_der {
        roots
            .add(CertificateDer::from(certificate_der.clone()))
            .map_err(|_| ProviderTransportError::Policy {
                detail: "additional Provider root certificate is invalid",
            })?;
    }
    if roots.is_empty() {
        return Err(ProviderTransportError::Policy {
            detail: "Provider TLS root store is empty",
        });
    }
    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let server_name =
        ServerName::try_from(host.to_owned()).map_err(|_| ProviderTransportError::Policy {
            detail: "Provider TLS server name is invalid",
        })?;
    let addresses =
        (host, port)
            .to_socket_addrs()
            .map_err(|_| ProviderTransportError::Network {
                detail: "Provider HTTPS exchange failed",
            })?;
    let mut last_error = None;
    let mut tcp = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => {
                tcp = Some(stream);
                break;
            }
            Err(error) => last_error = Some(error),
        }
    }
    let Some(tcp) = tcp else {
        return Err(last_error
            .map(map_io_error)
            .unwrap_or(ProviderTransportError::Network {
                detail: "Provider HTTPS exchange failed",
            }));
    };
    tcp.set_nodelay(true)
        .map_err(|_| ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        })?;
    tcp.set_read_timeout(Some(timeout))
        .map_err(|_| ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        })?;
    tcp.set_write_timeout(Some(timeout))
        .map_err(|_| ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        })?;
    let connection = ClientConnection::new(Arc::new(config), server_name).map_err(|_| {
        ProviderTransportError::Backend {
            detail: "failed to construct Rustls Provider transport",
        }
    })?;
    Ok(StreamOwned::new(connection, tcp))
}

fn write_http_request(
    tls: &mut StreamOwned<ClientConnection, TcpStream>,
    host: &str,
    port: u16,
    path: &str,
    request: &ProviderHttpRequest,
) -> Result<(), ProviderTransportError> {
    let method = match request.method {
        ProviderHttpMethod::Get => "GET",
        ProviderHttpMethod::Post => "POST",
    };
    let host_header = if port == 443 {
        host.to_owned()
    } else {
        format!("{host}:{port}")
    };
    let body = request.body.as_deref().unwrap_or(&[]);
    let mut message =
        format!("{method} {path} HTTP/1.1\r\nHost: {host_header}\r\nConnection: close\r\n");
    let mut has_accept = false;
    let mut has_content_length = false;
    for (name, value) in &request.headers {
        if name.eq_ignore_ascii_case("accept") {
            has_accept = true;
        }
        if name.eq_ignore_ascii_case("content-length") {
            has_content_length = true;
        }
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("connection") {
            continue;
        }
        message.push_str(name);
        message.push_str(": ");
        message.push_str(value);
        message.push_str("\r\n");
    }
    if !has_accept {
        message.push_str("Accept: text/event-stream\r\n");
    }
    if !has_content_length {
        message.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    message.push_str("\r\n");
    tls.write_all(message.as_bytes())
        .map_err(|_| ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        })?;
    if !body.is_empty() {
        tls.write_all(body)
            .map_err(|_| ProviderTransportError::Network {
                detail: "Provider HTTPS exchange failed",
            })?;
    }
    Ok(())
}

fn flush_tls(
    tls: &mut StreamOwned<ClientConnection, TcpStream>,
) -> Result<(), ProviderTransportError> {
    while tls.conn.wants_write() {
        tls.conn
            .complete_io(&mut tls.sock)
            .map_err(|_| ProviderTransportError::Network {
                detail: "Provider HTTPS exchange failed",
            })?;
    }
    tls.sock
        .flush()
        .map_err(|_| ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        })
}

struct TlsHttp {
    tls: StreamOwned<ClientConnection, TcpStream>,
    leftover: Vec<u8>,
}

impl TlsHttp {
    fn read_more(&mut self) -> Result<usize, ProviderTransportError> {
        let mut buffer = [0_u8; STREAM_READ_BYTES];
        let read = match self.tls.read(&mut buffer) {
            Ok(bytes_read) => bytes_read,
            Err(error) if is_provider_stream_closed(&error) => 0,
            Err(error) => return Err(map_io_error(error)),
        };
        if read > 0 {
            self.leftover.extend_from_slice(&buffer[..read]);
        }
        Ok(read)
    }

    fn take_prefix(&mut self, count: usize) -> Vec<u8> {
        self.leftover.drain(..count).collect()
    }
}

fn read_status_and_headers(
    reader: &mut TlsHttp,
) -> Result<(u16, bool, Option<usize>), ProviderTransportError> {
    let header_end = loop {
        if let Some(index) = find_header_end(&reader.leftover) {
            break index;
        }
        if reader.leftover.len() >= MAX_HEADER_BYTES {
            return Err(ProviderTransportError::Policy {
                detail: "Provider response exceeds local limit",
            });
        }
        if reader.read_more()? == 0 {
            return Err(ProviderTransportError::Network {
                detail: "failed to read Provider stream",
            });
        }
    };
    let header_bytes = reader.take_prefix(header_end);
    let header_text =
        std::str::from_utf8(&header_bytes).map_err(|_| ProviderTransportError::Network {
            detail: "failed to read Provider stream",
        })?;
    let mut lines = header_text.split("\r\n");
    let status_line = lines.next().ok_or(ProviderTransportError::Network {
        detail: "failed to read Provider stream",
    })?;
    let mut status_parts = status_line.split_whitespace();
    let _http = status_parts.next();
    let status: u16 = status_parts
        .next()
        .and_then(|text| text.parse().ok())
        .ok_or(ProviderTransportError::Network {
            detail: "failed to read Provider stream",
        })?;
    let mut chunked = false;
    let mut content_length = None;
    for line in lines.filter(|line| !line.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("transfer-encoding")
            && value
                .split(',')
                .any(|item| item.trim().eq_ignore_ascii_case("chunked"))
        {
            chunked = true;
        }
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.trim().parse().ok();
        }
    }
    Ok((status, chunked, content_length))
}

fn read_chunked_body(
    reader: &mut TlsHttp,
    deliver: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
) -> Result<(), ProviderTransportError> {
    loop {
        let size_line = read_line(reader)?.ok_or(ProviderTransportError::Network {
            detail: "failed to read Provider stream",
        })?;
        let size_text = size_line
            .split(';')
            .next()
            .unwrap_or(size_line.as_str())
            .trim();
        let size =
            usize::from_str_radix(size_text, 16).map_err(|_| ProviderTransportError::Network {
                detail: "failed to read Provider stream",
            })?;
        if size == 0 {
            let _ = read_line(reader)?;
            return Ok(());
        }
        ensure_available(reader, size + 2)?;
        let data = reader.take_prefix(size);
        deliver(&data)?;
        if reader.leftover.len() < 2 || &reader.leftover[..2] != b"\r\n" {
            return Err(ProviderTransportError::Network {
                detail: "failed to read Provider stream",
            });
        }
        let _ = reader.take_prefix(2);
    }
}

fn read_exact_body(
    reader: &mut TlsHttp,
    length: usize,
    deliver: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
) -> Result<(), ProviderTransportError> {
    let mut remaining = length;
    while remaining > 0 {
        if reader.leftover.is_empty() && reader.read_more()? == 0 {
            return Err(ProviderTransportError::Network {
                detail: "failed to read Provider stream",
            });
        }
        let take = remaining.min(reader.leftover.len());
        let data = reader.take_prefix(take);
        remaining -= take;
        deliver(&data)?;
    }
    Ok(())
}

fn read_until_close(
    reader: &mut TlsHttp,
    deliver: &mut dyn FnMut(&[u8]) -> Result<(), ProviderTransportError>,
) -> Result<(), ProviderTransportError> {
    loop {
        if !reader.leftover.is_empty() {
            let data = std::mem::take(&mut reader.leftover);
            deliver(&data)?;
        }
        if reader.read_more()? == 0 {
            return Ok(());
        }
    }
}

fn read_line(reader: &mut TlsHttp) -> Result<Option<String>, ProviderTransportError> {
    loop {
        if let Some(index) = reader
            .leftover
            .windows(2)
            .position(|window| window == b"\r\n")
        {
            let mut line = reader.take_prefix(index + 2);
            line.truncate(line.len() - 2);
            let text = String::from_utf8(line).map_err(|_| ProviderTransportError::Network {
                detail: "failed to read Provider stream",
            })?;
            return Ok(Some(text));
        }
        if reader.leftover.len() >= MAX_HEADER_BYTES {
            return Err(ProviderTransportError::Policy {
                detail: "Provider response exceeds local limit",
            });
        }
        if reader.read_more()? == 0 {
            return if reader.leftover.is_empty() {
                Ok(None)
            } else {
                Err(ProviderTransportError::Network {
                    detail: "failed to read Provider stream",
                })
            };
        }
    }
}

fn ensure_available(reader: &mut TlsHttp, needed: usize) -> Result<(), ProviderTransportError> {
    while reader.leftover.len() < needed {
        if reader.read_more()? == 0 {
            return Err(ProviderTransportError::Network {
                detail: "failed to read Provider stream",
            });
        }
    }
    Ok(())
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn map_io_error(error: std::io::Error) -> ProviderTransportError {
    if error.kind() == std::io::ErrorKind::TimedOut
        || error.kind() == std::io::ErrorKind::WouldBlock
    {
        ProviderTransportError::Timeout
    } else {
        ProviderTransportError::Network {
            detail: "Provider HTTPS exchange failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::split_https_url;

    #[test]
    fn split_https_url_accepts_localhost_with_port() {
        let (host, port, path) =
            split_https_url("https://localhost:9443/v1/chat/completions").expect("url");
        assert_eq!(host, "localhost");
        assert_eq!(port, 9443);
        assert_eq!(path, "/v1/chat/completions");
    }
}
