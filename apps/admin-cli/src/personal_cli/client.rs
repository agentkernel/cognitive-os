//! Minimal HTTP client for Personal daemon projections (std only).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use cognitive_store::PersonalDataLayout;
use serde_json::Value;

/// Failures talking to the Personal daemon as a non-authority client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalDaemonClientError {
    Connect { detail: String },
    Protocol { detail: String },
    Unauthorized { detail: String },
    Http { status: u16, body: String },
}

impl std::fmt::Display for PersonalDaemonClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect { detail } => write!(
                formatter,
                "unable to connect to Personal daemon: {detail}. Run `cognitive daemon start`."
            ),
            Self::Protocol { detail } => {
                write!(formatter, "Personal daemon protocol error: {detail}")
            }
            Self::Unauthorized { detail } => write!(
                formatter,
                "Personal daemon rejected management authentication: {detail}"
            ),
            Self::Http { status, body } => {
                write!(formatter, "Personal daemon HTTP {status}: {body}")
            }
        }
    }
}

impl std::error::Error for PersonalDaemonClientError {}

/// Authenticated client bound to one management session.
pub struct PersonalDaemonClient {
    endpoint: String,
    management_token: String,
}

impl PersonalDaemonClient {
    /// Connect, load bootstrap secret from the layout runtime dir, issue a
    /// management session, and return a client ready for status/doctor.
    pub fn connect(
        endpoint: &str,
        layout: &PersonalDataLayout,
    ) -> Result<Self, PersonalDaemonClientError> {
        let bootstrap = read_bootstrap_secret(layout)?;
        let token = issue_management_token(endpoint, &bootstrap)?;
        Ok(Self {
            endpoint: endpoint.to_owned(),
            management_token: token,
        })
    }

    /// `GET /personal/status` projection body.
    pub fn get_status(&self) -> Result<String, PersonalDaemonClientError> {
        self.get_authorized("/personal/status")
    }

    /// `GET /personal/doctor` projection body.
    pub fn get_doctor(&self) -> Result<String, PersonalDaemonClientError> {
        self.get_authorized("/personal/doctor")
    }

    fn get_authorized(&self, path: &str) -> Result<String, PersonalDaemonClientError> {
        let host = host_header_value(&self.endpoint);
        let wire = format!(
            "GET {path} HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {}\r\nConnection: close\r\n\r\n",
            self.management_token
        );
        let response = http_exchange(&self.endpoint, &wire)?;
        let (status, body) = split_http_response(&response)?;
        if status == 200 {
            return Ok(body);
        }
        if status == 401 || status == 403 {
            return Err(PersonalDaemonClientError::Unauthorized { detail: body });
        }
        Err(PersonalDaemonClientError::Http { status, body })
    }
}

fn read_bootstrap_secret(layout: &PersonalDataLayout) -> Result<String, PersonalDaemonClientError> {
    let path = layout.local_bootstrap_secret_path();
    std::fs::read_to_string(&path)
        .map(|contents| contents.trim().to_owned())
        .map_err(|error| PersonalDaemonClientError::Connect {
            detail: format!(
                "bootstrap secret missing at {} ({error}); is the daemon running?",
                path.display()
            ),
        })
}

fn issue_management_token(
    endpoint: &str,
    bootstrap_secret: &str,
) -> Result<String, PersonalDaemonClientError> {
    let body = serde_json::json!({
        "channel": "management",
        "principal_id": "principal://local/owner",
        "bootstrap_secret": bootstrap_secret
    })
    .to_string();
    let host = host_header_value(endpoint);
    let wire = format!(
        "POST /local/session HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = http_exchange(endpoint, &wire)?;
    let (status, response_body) = split_http_response(&response)?;
    if status != 200 {
        return Err(PersonalDaemonClientError::Unauthorized {
            detail: response_body,
        });
    }
    extract_json_string(&response_body, "token").ok_or_else(|| {
        PersonalDaemonClientError::Protocol {
            detail: "session response missing token field".to_owned(),
        }
    })
}

fn http_exchange(endpoint: &str, wire: &str) -> Result<String, PersonalDaemonClientError> {
    let address = endpoint
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let socket_address = resolve_socket_address(address)?;
    let mut stream =
        TcpStream::connect_timeout(&socket_address, Duration::from_secs(3)).map_err(|error| {
            PersonalDaemonClientError::Connect {
                detail: error.to_string(),
            }
        })?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    stream
        .write_all(wire.as_bytes())
        .map_err(|error| PersonalDaemonClientError::Protocol {
            detail: format!("write failed: {error}"),
        })?;
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut out = String::new();
    stream
        .read_to_string(&mut out)
        .map_err(|error| PersonalDaemonClientError::Protocol {
            detail: format!("read failed: {error}"),
        })?;
    Ok(out)
}

fn split_http_response(response: &str) -> Result<(u16, String), PersonalDaemonClientError> {
    let (header_block, body) =
        response
            .split_once("\r\n\r\n")
            .ok_or_else(|| PersonalDaemonClientError::Protocol {
                detail: "response missing header/body separator".to_owned(),
            })?;
    let status_line = header_block.lines().next().unwrap_or_default();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok())
        .ok_or_else(|| PersonalDaemonClientError::Protocol {
            detail: format!("unparseable status line: {status_line}"),
        })?;
    Ok((status, body.to_owned()))
}

fn extract_json_string(document: &str, field: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(document)
        && let Some(text) = value.get(field).and_then(Value::as_str)
    {
        return Some(text.to_owned());
    }
    let marker = format!("\"{field}\":\"");
    let start = document.find(&marker)? + marker.len();
    let end = document[start..].find('"')? + start;
    Some(document[start..end].to_owned())
}

fn host_header_value(endpoint: &str) -> String {
    endpoint
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .to_owned()
}

fn resolve_socket_address(
    endpoint: &str,
) -> Result<std::net::SocketAddr, PersonalDaemonClientError> {
    use std::net::ToSocketAddrs;
    endpoint
        .to_socket_addrs()
        .map_err(|error| PersonalDaemonClientError::Connect {
            detail: format!("unable to resolve endpoint `{endpoint}`: {error}"),
        })?
        .next()
        .ok_or_else(|| PersonalDaemonClientError::Connect {
            detail: format!("endpoint `{endpoint}` resolved to zero addresses"),
        })
}
