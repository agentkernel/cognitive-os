#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P1-T03 focused tests: Provider model discovery, capability probes, snapshot.
//!
//! Uses a hermetic injectable transport. Synthetic non-production secret
//! markers only. Never claims G0, B01-B12, or Profile conformance.

use cognitive_secret::{
    EphemeralSecretStore, ModelSelection, ProbeErrorClass, ProviderConfigRepository,
    ProviderDiscoveryService, ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse,
    ProviderKeyService, ProviderProbeError, ProviderProbeOptions, ProviderTransport,
    ProviderTransportError, SecretMaterial, redacted_headers,
};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug)]
enum Scenario {
    HappyPath,
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
    ServerError,
    ToolCapabilityMissing,
    TimeoutOnChat,
    EmptyCatalog,
}

#[derive(Clone)]
struct MockTransport {
    scenario: Scenario,
    captured: Arc<Mutex<Vec<ProviderHttpRequest>>>,
}

impl MockTransport {
    fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            captured: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn captured_requests(&self) -> Vec<ProviderHttpRequest> {
        self.captured.lock().expect("lock").clone()
    }
}

impl ProviderTransport for MockTransport {
    fn exchange(
        &self,
        request: &ProviderHttpRequest,
    ) -> Result<ProviderHttpResponse, ProviderTransportError> {
        self.captured.lock().expect("lock").push(request.clone());

        if request.cancel_requested {
            return Err(ProviderTransportError::Timeout);
        }

        if matches!(self.scenario, Scenario::TimeoutOnChat)
            && request.url.contains("/chat/completions")
            && !request.cancel_requested
        {
            return Err(ProviderTransportError::Timeout);
        }

        if request.method == ProviderHttpMethod::Get && request.url.ends_with("/models") {
            return match self.scenario {
                Scenario::Unauthorized => Ok(ProviderHttpResponse {
                    status: 401,
                    body: br#"{"error":"unauthorized"}"#.to_vec(),
                }),
                Scenario::Forbidden => Ok(ProviderHttpResponse {
                    status: 403,
                    body: br#"{"error":"forbidden"}"#.to_vec(),
                }),
                Scenario::NotFound => Ok(ProviderHttpResponse {
                    status: 404,
                    body: br#"{"error":"not found"}"#.to_vec(),
                }),
                Scenario::RateLimited => Ok(ProviderHttpResponse {
                    status: 429,
                    body: br#"{"error":"rate limited"}"#.to_vec(),
                }),
                Scenario::ServerError => Ok(ProviderHttpResponse {
                    status: 503,
                    body: br#"{"error":"unavailable"}"#.to_vec(),
                }),
                Scenario::EmptyCatalog => Ok(ProviderHttpResponse {
                    status: 200,
                    body: br#"{"object":"list","data":[]}"#.to_vec(),
                }),
                _ => Ok(ProviderHttpResponse {
                    status: 200,
                    body: br#"{"object":"list","data":[{"id":"deepseek-v4-flash","object":"model"},{"id":"deepseek-v4-pro","object":"model"}]}"#.to_vec(),
                }),
            };
        }

        // chat/completions probes
        let body_text = request
            .body
            .as_ref()
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or("");
        let is_stream = body_text.contains("\"stream\":true");
        let wants_tools = body_text.contains("\"tools\"");

        if wants_tools {
            if matches!(self.scenario, Scenario::ToolCapabilityMissing) {
                return Ok(ProviderHttpResponse {
                    status: 200,
                    body: br#"{"choices":[{"message":{"role":"assistant","content":"no tools"}}]}"#
                        .to_vec(),
                });
            }
            return Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{"choices":[{"message":{"role":"assistant","tool_calls":[{"id":"c1","type":"function","function":{"name":"cognitiveos_probe_noop","arguments":"{}"}}]}}]}"#.to_vec(),
            });
        }

        if is_stream {
            return Ok(ProviderHttpResponse {
                status: 200,
                body: b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n".to_vec(),
            });
        }

        Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#.to_vec(),
        })
    }
}

fn hermetic_config_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cognitiveos-p1-t03-{label}-{nanos}"));
    fs::create_dir_all(&directory).expect("temp config dir");
    directory.join("provider.json")
}

fn sample_material(marker: &str) -> SecretMaterial {
    SecretMaterial::from_bytes(format!("poc-provider-material-{marker}").into_bytes())
        .expect("material")
}

fn configured_service(label: &str) -> (ProviderKeyService<EphemeralSecretStore>, PathBuf) {
    let config_path = hermetic_config_path(label);
    let service = ProviderKeyService::new(
        EphemeralSecretStore::default(),
        ProviderConfigRepository::from_file_path(&config_path),
    );
    service
        .configure_provider(
            "deepseek",
            "https://api.deepseek.com",
            sample_material(label),
            None,
        )
        .expect("configure");
    (service, config_path)
}

#[test]
fn happy_path_discovers_probes_and_persists_snapshot_digest() {
    let (service, config_path) = configured_service("happy");
    let transport = MockTransport::new(Scenario::HappyPath);
    let discovery = ProviderDiscoveryService::new(&service, transport.clone());

    let readiness = discovery
        .discover_probe_and_persist(&ProviderProbeOptions::default())
        .expect("probe campaign");

    assert!(readiness.snapshot.is_minimally_ready());
    assert!(readiness.snapshot.capabilities().chat);
    assert!(readiness.snapshot.capabilities().stream);
    assert!(readiness.snapshot.capabilities().tool_call);
    assert!(readiness.snapshot.capabilities().cancel);
    assert_eq!(readiness.snapshot.selected_model(), "deepseek-v4-flash");
    assert!(readiness.snapshot_digest.starts_with("fnv1a64:"));

    let on_disk = fs::read_to_string(&config_path).expect("config");
    assert!(on_disk.contains(&readiness.snapshot_digest));
    assert!(!on_disk.contains("poc-provider-material"));
    assert!(!on_disk.contains("Bearer "));

    let requests = transport.captured_requests();
    assert!(requests.iter().any(|request| {
        request.method == ProviderHttpMethod::Get && request.url.ends_with("/models")
    }));
    assert!(requests.iter().any(|request| request.cancel_requested));

    // Authorization is present on the wire request object but redacted in Debug.
    let first = &requests[0];
    assert!(
        first
            .headers
            .iter()
            .any(|(name, value)| name == "Authorization" && value.starts_with("Bearer "))
    );
    let debug = format!("{first:?}");
    assert!(debug.contains("<redacted-authorization"));
    assert!(!debug.contains("poc-provider-material"));
}

#[test]
fn list_models_classifies_http_negatives() {
    for (scenario, class) in [
        (Scenario::Unauthorized, ProbeErrorClass::Unauthorized),
        (Scenario::Forbidden, ProbeErrorClass::Forbidden),
        (Scenario::NotFound, ProbeErrorClass::NotFound),
        (Scenario::RateLimited, ProbeErrorClass::RateLimited),
        (Scenario::ServerError, ProbeErrorClass::ServerError),
    ] {
        let (service, _) = configured_service(&format!("neg-{class:?}"));
        let discovery = ProviderDiscoveryService::new(&service, MockTransport::new(scenario));
        let error = discovery.list_models().expect_err("must fail");
        match error {
            ProviderProbeError::Classified {
                class: observed, ..
            } => assert_eq!(observed, class),
            other => panic!("unexpected error for {class:?}: {other:?}"),
        }
    }
}

#[test]
fn alias_drift_rejects_missing_catalog_model() {
    let (service, _) = configured_service("alias");
    let discovery =
        ProviderDiscoveryService::new(&service, MockTransport::new(Scenario::HappyPath));
    let options = ProviderProbeOptions {
        selection: ModelSelection::ExactCatalog {
            model_id: "missing-model-alias".into(),
        },
        ..ProviderProbeOptions::default()
    };
    let error = discovery
        .discover_probe_and_persist(&options)
        .expect_err("alias drift");
    match error {
        ProviderProbeError::Classified {
            class: ProbeErrorClass::AliasDrift,
            ..
        } => {}
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn manual_fallback_works_when_catalog_empty() {
    let (service, config_path) = configured_service("manual");
    let discovery =
        ProviderDiscoveryService::new(&service, MockTransport::new(Scenario::EmptyCatalog));
    let options = ProviderProbeOptions {
        selection: ModelSelection::ManualFallback {
            model_id: "operator-chosen-model".into(),
        },
        ..ProviderProbeOptions::default()
    };
    let readiness = discovery
        .discover_probe_and_persist(&options)
        .expect("manual fallback");
    assert!(readiness.snapshot.manual_model_fallback());
    assert_eq!(readiness.snapshot.selected_model(), "operator-chosen-model");
    assert!(readiness.snapshot.is_minimally_ready());
    let on_disk = fs::read_to_string(config_path).expect("config");
    assert!(on_disk.contains(&readiness.snapshot_digest));
}

#[test]
fn tool_probe_marks_capability_missing_on_http_200_without_tool_calls() {
    let (service, _) = configured_service("tool-missing");
    let discovery = ProviderDiscoveryService::new(
        &service,
        MockTransport::new(Scenario::ToolCapabilityMissing),
    );
    let readiness = discovery
        .discover_probe_and_persist(&ProviderProbeOptions::default())
        .expect("campaign completes with partial capabilities");
    assert!(readiness.snapshot.capabilities().chat);
    assert!(!readiness.snapshot.capabilities().tool_call);
    match readiness.snapshot.tool_call() {
        cognitive_secret::ProbeOutcome::Failed {
            class: ProbeErrorClass::CapabilityMissing,
            ..
        } => {}
        other => panic!("unexpected tool outcome: {other:?}"),
    }
}

#[test]
fn chat_timeout_is_classified_without_leaking_secret() {
    let (service, _) = configured_service("timeout");
    let discovery =
        ProviderDiscoveryService::new(&service, MockTransport::new(Scenario::TimeoutOnChat));
    let readiness = discovery
        .discover_probe_and_persist(&ProviderProbeOptions::default())
        .expect("campaign records timeout outcomes");
    assert!(!readiness.snapshot.capabilities().chat);
    match readiness.snapshot.chat() {
        cognitive_secret::ProbeOutcome::Failed {
            class: ProbeErrorClass::Timeout,
            ..
        } => {}
        other => panic!("unexpected chat outcome: {other:?}"),
    }
    let debug = format!("{readiness:?}");
    assert!(!debug.contains("poc-provider-material"));
    assert!(!debug.contains("Bearer "));
}

#[test]
fn redacted_headers_helper_masks_authorization() {
    let headers = vec![
        ("Authorization".into(), "Bearer super-secret-token".into()),
        ("Accept".into(), "application/json".into()),
    ];
    let redacted = redacted_headers(&headers);
    assert!(redacted[0].1.contains("<redacted-authorization"));
    assert!(!redacted[0].1.contains("super-secret-token"));
    assert_eq!(redacted[1].1, "application/json");
}
