#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use cognitive_provider_transport::RustlsProviderTransport;
use cognitive_secret::{
    EphemeralSecretStore, ModelSelection, ProbeErrorClass, ProviderConfigRepository,
    ProviderDiscoveryService, ProviderHttpMethod, ProviderHttpRequest, ProviderKeyService,
    ProviderProbeError, ProviderProbeOptions, ProviderTransport, ProviderTransportError,
    SecretMaterial,
};

const FIXTURE_MODEL_ID: &str = "p1-t09-deterministic-chat-model";
const SECRET_MARKER: &str = "p1t09-secret-material-must-not-leak";

struct RunningProviderFixture {
    child: Child,
    base_url: String,
    certificate_path: PathBuf,
    observations_path: PathBuf,
}

impl RunningProviderFixture {
    fn spawn(scenario: &str) -> Self {
        let fixture_root = unique_temporary_directory(scenario);
        let certificate_path = fixture_root.join("fixture-ca.der");
        let observations_path = fixture_root.join("observations.txt");
        let fixture_binary_path = std::env::var_os("CARGO_BIN_EXE_p1-t09-provider-fixture")
            .expect("Cargo publishes the deterministic Provider fixture path");
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
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("deterministic Provider fixture starts");
        let stdout = child.stdout.take().expect("fixture stdout is captured");
        let mut startup_reader = BufReader::new(stdout);
        let mut base_url = String::new();
        startup_reader
            .read_line(&mut base_url)
            .expect("fixture publishes its HTTPS base URL");
        let base_url = base_url.trim().to_owned();
        assert!(
            base_url.starts_with("https://localhost:"),
            "fixture must publish only an HTTPS loopback endpoint"
        );
        assert!(certificate_path.is_file(), "fixture CA must be published");

        Self {
            child,
            base_url,
            certificate_path,
            observations_path,
        }
    }

    fn transport(&self) -> RustlsProviderTransport {
        let certificate_der = fs::read(&self.certificate_path).expect("read fixture CA");
        RustlsProviderTransport::with_additional_root_certificate_der(certificate_der)
            .expect("fixture CA is a valid additional Rustls root")
    }

    fn observations(&self) -> String {
        fs::read_to_string(&self.observations_path).unwrap_or_default()
    }
}

impl Drop for RunningProviderFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn unique_temporary_directory(label: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cognitiveos-p1-t09-provider-fixture-{label}-{unique_suffix}"
    ));
    fs::create_dir_all(&directory).expect("create fixture test directory");
    directory
}

fn configured_key_service(
    fixture: &RunningProviderFixture,
    label: &str,
) -> (ProviderKeyService<EphemeralSecretStore>, PathBuf) {
    let state_directory = unique_temporary_directory(label);
    let provider_config_path = state_directory.join("provider.json");
    let key_service = ProviderKeyService::new(
        EphemeralSecretStore::default(),
        ProviderConfigRepository::from_file_path(&provider_config_path),
    );
    key_service
        .configure_provider(
            "deterministic-fixture",
            &fixture.base_url,
            SecretMaterial::from_bytes(SECRET_MARKER.as_bytes().to_vec())
                .expect("synthetic test secret is valid"),
            None,
        )
        .expect("configure fixture Provider");
    (key_service, provider_config_path)
}

fn exact_model_options() -> ProviderProbeOptions {
    ProviderProbeOptions {
        budget_ms: 3_000,
        exchange_timeout_ms: 500,
        selection: ModelSelection::ExactCatalog {
            model_id: FIXTURE_MODEL_ID.to_owned(),
        },
    }
}

fn direct_request(url: String, timeout_ms: u32) -> ProviderHttpRequest {
    ProviderHttpRequest {
        method: ProviderHttpMethod::Get,
        url,
        headers: Vec::new(),
        body: None,
        timeout_ms,
        cancel_requested: false,
    }
}

fn assert_persisted_state_is_secret_free(provider_config_path: &Path) {
    let state_directory = provider_config_path
        .parent()
        .expect("provider config has a parent directory");
    for state_file_name in ["provider.json", "selected-model.json"] {
        let state = fs::read_to_string(state_directory.join(state_file_name))
            .expect("expected Provider state file is readable");
        assert!(!state.contains(SECRET_MARKER));
        assert!(!state.contains("Bearer "));
    }
}

#[test]
fn binary_fixture_drives_real_rustls_discovery_without_leaking_provider_material() {
    let fixture = RunningProviderFixture::spawn("ready");
    let (key_service, provider_config_path) = configured_key_service(&fixture, "ready");
    let discovery = ProviderDiscoveryService::new(&key_service, fixture.transport());

    let readiness = discovery
        .discover_probe_and_persist(&exact_model_options())
        .expect("deterministic Provider discovery succeeds");

    assert!(readiness.snapshot.is_minimally_ready());
    assert_eq!(readiness.snapshot.selected_model(), FIXTURE_MODEL_ID);
    assert!(readiness.snapshot.capabilities().chat);
    assert!(readiness.snapshot.capabilities().stream);
    assert!(readiness.snapshot.capabilities().tool_call);
    assert!(readiness.snapshot.capabilities().cancel);
    assert_persisted_state_is_secret_free(&provider_config_path);

    let observations = fixture.observations();
    assert_eq!(observations.matches("GET /v1/models").count(), 1);
    assert_eq!(observations.matches("POST /v1/chat/completions").count(), 3);
    assert!(
        observations
            .lines()
            .all(|line| line.contains("authorization=present"))
    );
    assert!(!observations.contains(SECRET_MARKER));
    assert!(!format!("{readiness:?}").contains(SECRET_MARKER));
}

#[test]
fn malformed_error_and_non_chat_responses_fail_closed_without_selection() {
    for (scenario, expected_error_class) in [
        ("malformed-models", Some(ProbeErrorClass::AliasDrift)),
        ("unauthorized", Some(ProbeErrorClass::Unauthorized)),
        ("non-chat-capable", None),
    ] {
        let fixture = RunningProviderFixture::spawn(scenario);
        let (key_service, _) = configured_key_service(&fixture, scenario);
        let discovery = ProviderDiscoveryService::new(&key_service, fixture.transport());
        let outcome = discovery.discover_probe_and_persist(&exact_model_options());

        if let Some(expected_error_class) = expected_error_class {
            match outcome.expect_err("invalid discovery response must fail") {
                ProviderProbeError::Classified { class, .. } => {
                    assert_eq!(class, expected_error_class)
                }
                other => panic!("unexpected probe error: {other:?}"),
            }
        } else {
            let readiness = outcome.expect("non-chat capability is recorded as non-ready");
            assert!(!readiness.snapshot.is_minimally_ready());
            assert!(!readiness.snapshot.capabilities().chat);
        }
        assert!(
            key_service
                .selected_model_repository()
                .load()
                .expect("selected-model repository remains readable")
                .is_none(),
            "failed or non-chat discovery must not persist a selected model"
        );
    }
}

#[test]
fn timeout_oversize_and_redirect_are_bounded_and_fail_closed() {
    let timeout_fixture = RunningProviderFixture::spawn("timeout");
    let timeout_error = timeout_fixture
        .transport()
        .exchange(&direct_request(
            format!("{}/models", timeout_fixture.base_url),
            50,
        ))
        .expect_err("fixture delay must hit the transport timeout");
    assert_eq!(timeout_error, ProviderTransportError::Timeout);

    let oversized_fixture = RunningProviderFixture::spawn("oversized");
    let oversized_error = oversized_fixture
        .transport()
        .exchange(&direct_request(
            format!("{}/models", oversized_fixture.base_url),
            1_000,
        ))
        .expect_err("oversized fixture response must be rejected");
    assert!(matches!(
        oversized_error,
        ProviderTransportError::Policy { .. }
    ));

    let redirect_fixture = RunningProviderFixture::spawn("redirect");
    let redirect_response = redirect_fixture
        .transport()
        .exchange(&direct_request(
            format!("{}/models", redirect_fixture.base_url),
            1_000,
        ))
        .expect("redirect response is returned without being followed");
    assert_eq!(redirect_response.status, 302);
    assert_eq!(redirect_fixture.observations().lines().count(), 1);
}
