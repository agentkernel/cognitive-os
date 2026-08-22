//! Deterministic loopback-only HTTPS Provider used by P1-T09 integration tests.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

const MAX_FIXTURE_REQUEST_BYTES: usize = 65_536;
const OVERSIZED_RESPONSE_BYTES: usize = 1_048_577;
const FIXTURE_MODEL_ID: &str = "p1-t09-deterministic-chat-model";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixtureScenario {
    Ready,
    MalformedModels,
    Unauthorized,
    NonChatCapable,
    Timeout,
    Oversized,
    Redirect,
    DelayedSse,
}

impl FixtureScenario {
    fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "ready" => Ok(Self::Ready),
            "malformed-models" => Ok(Self::MalformedModels),
            "unauthorized" => Ok(Self::Unauthorized),
            "non-chat-capable" => Ok(Self::NonChatCapable),
            "timeout" => Ok(Self::Timeout),
            "oversized" => Ok(Self::Oversized),
            "redirect" => Ok(Self::Redirect),
            "delayed-sse" => Ok(Self::DelayedSse),
            _ => Err("unsupported deterministic Provider fixture scenario"),
        }
    }
}

#[derive(Debug)]
struct FixtureOptions {
    scenario: FixtureScenario,
    certificate_output: PathBuf,
    observations_output: PathBuf,
}

#[derive(Debug)]
struct FixtureRequest {
    method: String,
    path: String,
    authorization_present: bool,
    body: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options()?;
    let CertifiedKey {
        cert: certificate,
        signing_key,
    } = generate_simple_self_signed(vec!["localhost".to_owned()])?;
    let certificate_der = certificate.der().clone();
    let private_key_der = PrivatePkcs8KeyDer::from(signing_key.serialize_der());
    let server_configuration = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certificate_der.clone()],
            PrivateKeyDer::Pkcs8(private_key_der),
        )?;

    if let Some(parent_directory) = options.certificate_output.parent() {
        fs::create_dir_all(parent_directory)?;
    }
    fs::write(&options.certificate_output, certificate_der.as_ref())?;
    fs::write(&options.observations_output, [])?;

    let listener = TcpListener::bind("127.0.0.1:0")?;
    println!("https://localhost:{}/v1", listener.local_addr()?.port());
    std::io::stdout().flush()?;

    let shared_server_configuration = Arc::new(server_configuration);
    for incoming_connection in listener.incoming() {
        let tcp_stream = incoming_connection?;
        handle_connection(
            tcp_stream,
            Arc::clone(&shared_server_configuration),
            &options,
        )?;
    }
    Ok(())
}

fn parse_options() -> Result<FixtureOptions, &'static str> {
    let mut arguments = std::env::args().skip(1);
    let mut scenario = None;
    let mut certificate_output = None;
    let mut observations_output = None;

    while let Some(argument) = arguments.next() {
        let value = arguments
            .next()
            .ok_or("fixture option requires an explicit value")?;
        match argument.as_str() {
            "--scenario" => scenario = Some(FixtureScenario::parse(&value)?),
            "--certificate-output" => certificate_output = Some(PathBuf::from(value)),
            "--observations-output" => observations_output = Some(PathBuf::from(value)),
            _ => return Err("fixture received an unsupported option"),
        }
    }

    Ok(FixtureOptions {
        scenario: scenario.ok_or("fixture scenario is required")?,
        certificate_output: certificate_output.ok_or("certificate output is required")?,
        observations_output: observations_output.ok_or("observations output is required")?,
    })
}

fn handle_connection(
    tcp_stream: TcpStream,
    server_configuration: Arc<ServerConfig>,
    options: &FixtureOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    tcp_stream.set_nodelay(true)?;
    tcp_stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    tcp_stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let server_connection = ServerConnection::new(server_configuration)?;
    let mut tls_stream = StreamOwned::new(server_connection, tcp_stream);
    let request = read_request(&mut tls_stream)?;
    record_observation(&request, &options.observations_output)?;

    if options.scenario == FixtureScenario::DelayedSse {
        write_delayed_sse(&mut tls_stream)?;
        return Ok(());
    }

    let response = fixture_response(options.scenario, &request);
    if options.scenario == FixtureScenario::Timeout {
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = tls_stream.write_all(&response);
    let _ = tls_stream.flush();
    Ok(())
}

fn write_delayed_sse(
    tls_stream: &mut StreamOwned<ServerConnection, TcpStream>,
) -> Result<(), Box<dyn std::error::Error>> {
    let header = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
    tls_stream.write_all(header)?;
    tls_stream.flush()?;
    tls_stream.write_all(b"data: {\"choices\":[{\"delta\":{\"content\":\"pong\"}}]}\n\n")?;
    tls_stream.flush()?;
    std::thread::sleep(Duration::from_millis(250));
    tls_stream.write_all(b"data: [DONE]\n\n")?;
    tls_stream.flush()?;
    Ok(())
}

fn read_request(
    tls_stream: &mut StreamOwned<ServerConnection, TcpStream>,
) -> Result<FixtureRequest, Box<dyn std::error::Error>> {
    let mut request_bytes = Vec::new();
    let header_end = loop {
        if request_bytes.len() >= MAX_FIXTURE_REQUEST_BYTES {
            return Err("fixture request exceeds local limit".into());
        }
        let mut next_byte = [0_u8; 1];
        tls_stream.read_exact(&mut next_byte)?;
        request_bytes.push(next_byte[0]);
        if request_bytes.ends_with(b"\r\n\r\n") {
            break request_bytes.len();
        }
    };

    let header_text = std::str::from_utf8(&request_bytes[..header_end])?;
    let mut header_lines = header_text.split("\r\n");
    let request_line = header_lines
        .next()
        .ok_or("fixture request line is absent")?;
    let mut request_line_parts = request_line.split_whitespace();
    let method = request_line_parts
        .next()
        .ok_or("fixture request method is absent")?
        .to_owned();
    let path = request_line_parts
        .next()
        .ok_or("fixture request path is absent")?
        .to_owned();
    let mut content_length = 0_usize;
    let mut authorization_present = false;
    for header_line in header_lines.filter(|line| !line.is_empty()) {
        let (header_name, header_value) = header_line
            .split_once(':')
            .ok_or("fixture request header is malformed")?;
        if header_name.eq_ignore_ascii_case("content-length") {
            content_length = header_value.trim().parse()?;
        }
        if header_name.eq_ignore_ascii_case("authorization") {
            authorization_present = true;
        }
    }
    if header_end + content_length > MAX_FIXTURE_REQUEST_BYTES {
        return Err("fixture request body exceeds local limit".into());
    }
    let mut body = vec![0_u8; content_length];
    tls_stream.read_exact(&mut body)?;

    Ok(FixtureRequest {
        method,
        path,
        authorization_present,
        body,
    })
}

fn record_observation(
    request: &FixtureRequest,
    observations_output: &PathBuf,
) -> Result<(), std::io::Error> {
    let authorization_state = if request.authorization_present {
        "present"
    } else {
        "absent"
    };
    let mut observations_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(observations_output)?;
    writeln!(
        observations_file,
        "{} {} authorization={authorization_state}",
        request.method, request.path
    )
}

fn fixture_response(scenario: FixtureScenario, request: &FixtureRequest) -> Vec<u8> {
    if scenario == FixtureScenario::Timeout {
        return http_response(200, "OK", &[], b"{}");
    }
    if scenario == FixtureScenario::Oversized {
        return http_response(200, "OK", &[], &vec![b'x'; OVERSIZED_RESPONSE_BYTES]);
    }
    if scenario == FixtureScenario::Redirect {
        return http_response(
            302,
            "Found",
            &[("Location", "https://example.invalid/must-not-be-followed")],
            b"{}",
        );
    }
    if scenario == FixtureScenario::Unauthorized {
        return http_response(401, "Unauthorized", &[], br#"{"error":"unauthorized"}"#);
    }
    if request.method == "GET" && request.path == "/v1/models" {
        let body: &[u8] = if scenario == FixtureScenario::MalformedModels {
            b"not-json"
        } else {
            br#"{"object":"list","data":[{"id":"p1-t09-deterministic-chat-model","object":"model"}]}"#
        };
        return http_response(200, "OK", &[], body);
    }
    if request.method == "POST" && request.path == "/v1/chat/completions" {
        if scenario == FixtureScenario::NonChatCapable {
            return http_response(200, "OK", &[], b"{}");
        }
        let body_text = std::str::from_utf8(&request.body).unwrap_or_default();
        if body_text.contains("\"tools\"") {
            return http_response(
                200,
                "OK",
                &[],
                br#"{"choices":[{"message":{"role":"assistant","tool_calls":[{"id":"fixture-call","type":"function","function":{"name":"cognitiveos_probe_noop","arguments":"{}"}}]}}]}"#,
            );
        }
        if body_text.contains("\"stream\":true") {
            return http_response(
                200,
                "OK",
                &[],
                b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
            );
        }
        return http_response(
            200,
            "OK",
            &[],
            br#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#,
        );
    }
    http_response(404, "Not Found", &[], br#"{"error":"not found"}"#)
}

fn http_response(
    status: u16,
    reason: &str,
    additional_headers: &[(&str, &str)],
    body: &[u8],
) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )
    .into_bytes();
    for (header_name, header_value) in additional_headers {
        response.extend_from_slice(format!("{header_name}: {header_value}\r\n").as_bytes());
    }
    response.extend_from_slice(b"\r\n");
    response.extend_from_slice(body);
    response
}

#[allow(dead_code)]
fn _fixture_model_id_is_stable() -> &'static str {
    FIXTURE_MODEL_ID
}
