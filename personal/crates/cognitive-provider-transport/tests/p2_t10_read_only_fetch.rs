//! P2-T10/D03: the daemon's read-only outbound HTTP boundary, proven against a
//! real loopback TLS server rather than a mock.
#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use cognitive_provider_transport::{
    ReadOnlyFetchError, ReadOnlyFetchMethod, ReadOnlyFetchRequest, ReadOnlyFetchTransport,
    RustlsReadOnlyFetchTransport,
};

const FIXTURE_EXECUTION_ENVIRONMENT_ALLOWLIST: [&str; 8] = [
    "ComSpec",
    "PATH",
    "PATHEXT",
    "SystemRoot",
    "TEMP",
    "TMP",
    "TMPDIR",
    "WINDIR",
];

struct RunningFixture {
    child: Child,
    base_url: String,
    certificate_path: PathBuf,
    observations_path: PathBuf,
}

impl RunningFixture {
    fn spawn(scenario: &str) -> Self {
        let fixture_root = unique_temporary_directory(scenario);
        let certificate_path = fixture_root.join("fixture-ca.der");
        let observations_path = fixture_root.join("observations.txt");
        let fixture_binary_path = std::env::current_exe()
            .expect("test executable path is available")
            .parent()
            .and_then(Path::parent)
            .expect("test executable is under the Cargo target directory")
            .join(format!(
                "p1_t09_provider_fixture{}",
                std::env::consts::EXE_SUFFIX
            ));
        let mut child = Command::new(fixture_binary_path)
            .args([
                "--scenario",
                scenario,
                "--certificate-output",
                certificate_path
                    .to_str()
                    .expect("fixture certificate path is UTF-8"),
                "--observations-output",
                observations_path
                    .to_str()
                    .expect("fixture observations path is UTF-8"),
            ])
            .env_clear()
            .envs(fixture_execution_environment())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("deterministic loopback fixture starts");
        let stdout = child.stdout.take().expect("fixture stdout is captured");
        let mut startup_reader = BufReader::new(stdout);
        let mut base_url = String::new();
        startup_reader
            .read_line(&mut base_url)
            .expect("fixture publishes its HTTPS base URL");
        let base_url = base_url.trim().to_owned();
        assert!(base_url.starts_with("https://localhost:"));

        Self {
            child,
            base_url,
            certificate_path,
            observations_path,
        }
    }

    fn transport(&self) -> RustlsReadOnlyFetchTransport {
        let certificate_der = fs::read(&self.certificate_path).expect("read fixture CA");
        RustlsReadOnlyFetchTransport::with_additional_root_certificate_der(certificate_der)
            .expect("fixture CA is a valid additional Rustls root")
    }

    fn observations(&self) -> String {
        fs::read_to_string(&self.observations_path).unwrap_or_default()
    }
}

impl Drop for RunningFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn fixture_execution_environment() -> Vec<(String, std::ffi::OsString)> {
    FIXTURE_EXECUTION_ENVIRONMENT_ALLOWLIST
        .into_iter()
        .filter_map(|variable_name| {
            std::env::var_os(variable_name).map(|value| (variable_name.to_owned(), value))
        })
        .collect()
}

fn unique_temporary_directory(label: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cognitiveos-p2-t10-read-only-fetch-{label}-{unique_suffix}"
    ));
    fs::create_dir_all(&directory).expect("create fixture test directory");
    directory
}

fn fetch_request(
    url: String,
    timeout_ms: u32,
    maximum_response_bytes: usize,
) -> ReadOnlyFetchRequest {
    ReadOnlyFetchRequest {
        method: ReadOnlyFetchMethod::Get,
        url,
        timeout_ms,
        maximum_response_bytes,
    }
}

#[test]
fn read_only_fetch_returns_a_bounded_body_over_real_tls_and_sends_no_credential() {
    let fixture = RunningFixture::spawn("ready");
    let response = fixture
        .transport()
        .fetch(&fetch_request(
            format!("{}/models", fixture.base_url),
            5_000,
            65_536,
        ))
        .expect("loopback read-only fetch succeeds");

    assert_eq!(response.status, 200);
    assert!(
        String::from_utf8_lossy(&response.body).contains("p1-t09-deterministic-chat-model"),
        "the fetch must return the server's real body"
    );

    let observations = fixture.observations();
    assert_eq!(observations.lines().count(), 1, "exactly one request");
    assert!(observations.contains("GET /v1/models"));
    assert!(
        observations.contains("authorization=absent"),
        "a read-only Tool fetch must never carry a credential: {observations}"
    );
}

#[test]
fn read_only_fetch_returns_a_redirect_instead_of_following_it() {
    let fixture = RunningFixture::spawn("redirect");
    let response = fixture
        .transport()
        .fetch(&fetch_request(
            format!("{}/models", fixture.base_url),
            5_000,
            65_536,
        ))
        .expect("redirect status is returned to the caller");

    assert_eq!(response.status, 302);
    assert_eq!(
        fixture.observations().lines().count(),
        1,
        "the redirect target must never be requested"
    );
}

#[test]
fn read_only_fetch_refuses_an_oversized_response_rather_than_truncating_it() {
    let fixture = RunningFixture::spawn("oversized");
    let error = fixture
        .transport()
        .fetch(&fetch_request(
            format!("{}/models", fixture.base_url),
            5_000,
            4_096,
        ))
        .expect_err("an oversized body must be refused");

    assert_eq!(error, ReadOnlyFetchError::ResponseTooLarge);
}

#[test]
fn read_only_fetch_stops_at_its_bounded_deadline() {
    let fixture = RunningFixture::spawn("timeout");
    let error = fixture
        .transport()
        .fetch(&fetch_request(
            format!("{}/models", fixture.base_url),
            50,
            65_536,
        ))
        .expect_err("the fixture delay must hit the bounded deadline");

    assert_eq!(error, ReadOnlyFetchError::Timeout);
}
