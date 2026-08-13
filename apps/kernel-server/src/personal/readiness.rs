//! Personal readiness / status / doctor projection (P1-T05).
//!
//! CLI, Pi, and future UI share this fact source. The service reports
//! component facts, durations, and overall `blocked` / `degraded` / `ready`
//! without claiming G0, B01-B12, or Profile conformance. Static analysis
//! success is never rewritten as runtime ready.
//!
//! Ownership: lives in the Personal composition root (`kernel-server`) so
//! Personal does not take Lane-RUN ownership of `cognitive-management`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cognitive_secret::{
    ProviderConfig, ProviderConfigError, ProviderConfigRepository, SecretError, SecretStore,
    SecretStoreAvailability, SecretStoreClass, SelectedModelRepository,
    select_production_secret_store,
};
use cognitive_store::PersonalDataLayout;
use serde_json::{Value, json};

use super::headless_vault_doctor::{
    HeadlessVaultDoctorPath, HeadlessVaultPathObservation, HeadlessVaultPathStatus,
    evaluate_headless_vault_doctor, headless_vault_doctor_projection_json,
};
use super::operability_doctor::{
    OperabilityDoctorObservation, OperabilityDoctorStatus, OperabilityDoctorTopic,
    evaluate_operability_doctor, operability_doctor_projection_json,
};
use super::pi_runtime::{PINNED_PI_VERSION, PiRuntimeObservation, observe_pi_runtime};
use super::six_resource_doctor::{
    SixResourceFamily, SixResourceHealthFact, SixResourceHealthObservation,
    SixResourceHealthStatus, evaluate_six_resource_doctor_health,
    six_resource_doctor_projection_json,
};

/// Product-local schema version for readiness projections (not a registry schema).
pub const PERSONAL_READINESS_SCHEMA_VERSION: u32 = 1;

/// Coarse overall readiness. Distinct from M6 `MANAGEMENT_READY` grades and
/// from Profile / Gate claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverallReadiness {
    /// At least one required component blocks first-conversation progress.
    Blocked,
    /// Required components are present but at least one is reduced.
    Degraded,
    /// All required components report runtime-ready facts.
    Ready,
}

impl OverallReadiness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Ready => "ready",
        }
    }
}

/// Per-component status. Optional components may use `NotConfigured`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Ready,
    Degraded,
    Blocked,
    NotConfigured,
}

impl ComponentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Blocked => "blocked",
            Self::NotConfigured => "not_configured",
        }
    }
}

/// One non-secret observation about a component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessFact {
    pub key: &'static str,
    pub value: String,
}

/// Result of checking a single component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentCheck {
    pub component: &'static str,
    pub status: ComponentStatus,
    pub required: bool,
    pub source: &'static str,
    pub duration_ms: u64,
    pub observed_at_unix_ms: u64,
    pub error_class: Option<&'static str>,
    pub facts: Vec<ReadinessFact>,
}

/// Full readiness / doctor report shared by status and doctor routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessReport {
    pub overall: OverallReadiness,
    pub first_conversation_ready: bool,
    pub components: Vec<ComponentCheck>,
    pub evaluated_at_unix_ms: u64,
}

/// Inputs collected by the daemon or by hermetic tests.
#[derive(Clone)]
pub struct ReadinessEvaluationContext {
    pub layout: PersonalDataLayout,
    pub daemon_listening: bool,
    pub session_count: usize,
    /// Optional override for SecretStore probe (tests inject; production None).
    pub secret_probe_override: Option<SecretProbeObservation>,
    /// Optional override for Provider config path (defaults to layout config).
    pub provider_config_path_override: Option<PathBuf>,
    /// Optional override for the Provider secret-ref resolution observation
    /// (tests inject; production None).
    pub provider_secret_resolution_override: Option<ProviderSecretResolution>,
    /// Optional SecretStore used to exercise the production resolution path
    /// against the exact config snapshot already loaded above.
    pub provider_secret_store_override: Option<Arc<dyn SecretStore + Send + Sync>>,
    /// Optional override for the Pi runtime observation (tests inject; production None).
    pub pi_observation_override: Option<PiRuntimeObservation>,
}

/// Non-secret SecretStore probe observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecretProbeObservation {
    pub class: SecretStoreClass,
    pub availability: SecretStoreAvailability,
}

/// Whether the configured Provider `secret_ref` actually resolves.
///
/// A reachable SecretStore does not imply that the reference recorded in
/// `provider.json` still points at a stored item: a cleanup can remove the item
/// and leave the reference behind. Readiness must distinguish the two, so this
/// observation is derived from a real resolution attempt. It carries no secret
/// material and the resolved bytes are dropped immediately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSecretResolution {
    /// The configured ref resolved to stored material.
    Resolves,
    /// The backend answered, and the configured ref has no stored item.
    Missing,
    /// The backend could not answer, so resolvability is unknown.
    Unavailable,
}

/// Evaluate Personal readiness from filesystem and probe facts.
pub fn evaluate_personal_readiness(context: &ReadinessEvaluationContext) -> ReadinessReport {
    let evaluated_at_unix_ms = unix_now_ms();
    let components = vec![
        check_system(&context.layout, evaluated_at_unix_ms),
        check_database(&context.layout, evaluated_at_unix_ms),
        check_secret(context, evaluated_at_unix_ms),
        check_provider(context, evaluated_at_unix_ms),
        check_daemon(context, evaluated_at_unix_ms),
        check_pi(context, evaluated_at_unix_ms),
    ];

    let overall = aggregate_overall(&components);
    let first_conversation_ready = overall == OverallReadiness::Ready
        && components.iter().all(|component| {
            if component.component == "pi" {
                // `pi` is optional for the required-set aggregate (ADR-0023),
                // but a first conversation genuinely needs it, so it is
                // required here and only here. P1-T07 made this a real
                // observation; the rule itself is unchanged.
                component.status == ComponentStatus::Ready
            } else if component.required {
                component.status == ComponentStatus::Ready
            } else {
                true
            }
        });

    ReadinessReport {
        overall,
        first_conversation_ready,
        components,
        evaluated_at_unix_ms,
    }
}

/// Compact status projection for `GET /personal/status`.
pub fn status_projection_json(report: &ReadinessReport) -> Value {
    json!({
        "schema_version": PERSONAL_READINESS_SCHEMA_VERSION,
        "surface": "personal-status",
        "overall": report.overall.as_str(),
        "first_conversation_ready": report.first_conversation_ready,
        "evaluated_at_unix_ms": report.evaluated_at_unix_ms,
        "components": report.components.iter().map(component_summary_json).collect::<Vec<_>>(),
        "static_check_is_not_runtime_ready": true,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false
    })
}

/// Detailed doctor projection for `GET /personal/doctor`.
pub fn doctor_projection_json(report: &ReadinessReport) -> Value {
    json!({
        "schema_version": PERSONAL_READINESS_SCHEMA_VERSION,
        "surface": "personal-doctor",
        "overall": report.overall.as_str(),
        "first_conversation_ready": report.first_conversation_ready,
        "evaluated_at_unix_ms": report.evaluated_at_unix_ms,
        "components": report.components.iter().map(component_detail_json).collect::<Vec<_>>(),
        "six_resource": default_six_resource_doctor_section(),
        "headless_vault": default_headless_vault_doctor_section(),
        "operability": default_operability_doctor_section(),
        "static_check_is_not_runtime_ready": true,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false,
        "guidance": doctor_guidance(report)
    })
}

fn default_operability_doctor_section() -> Value {
    let observations: Vec<OperabilityDoctorObservation> = OperabilityDoctorTopic::ALL
        .iter()
        .map(|topic| OperabilityDoctorObservation {
            topic: *topic,
            status: OperabilityDoctorStatus::NotConfigured,
            error_code: Some("OPERABILITY_TOPIC_NOT_PROBED"),
            recovery_hint: Some("await supported operability probe"),
            facts: vec![("probe".to_owned(), "not_run".to_owned())],
        })
        .collect();
    match evaluate_operability_doctor(&observations) {
        Ok(report) => operability_doctor_projection_json(&report),
        Err(_) => json!({
            "schema": "personal-operability-doctor",
            "surface": "personal-doctor-operability",
            "overall": "blocked",
            "gate_claim": "not-claimed",
            "profile_claim": "not-claimed",
            "error_code": "OPERABILITY_DOCTOR_INTERNAL",
            "topics": [],
        }),
    }
}

fn default_headless_vault_doctor_section() -> Value {
    let observations: Vec<HeadlessVaultPathObservation> = HeadlessVaultDoctorPath::ALL
        .iter()
        .map(|path| HeadlessVaultPathObservation {
            path: *path,
            status: HeadlessVaultPathStatus::NotConfigured,
            error_code: Some("VAULT_PATH_NOT_PROBED"),
            recovery_hint: Some("await supported vault path probe"),
            facts: vec![("probe".to_owned(), "not_run".to_owned())],
        })
        .collect();
    match evaluate_headless_vault_doctor(&observations) {
        Ok(report) => headless_vault_doctor_projection_json(&report),
        Err(_) => json!({
            "schema": "personal-headless-vault-doctor",
            "surface": "personal-doctor-headless-vault",
            "overall": "unavailable",
            "gate_claim": "not-claimed",
            "profile_claim": "not-claimed",
            "error_code": "HEADLESS_VAULT_DOCTOR_INTERNAL",
            "paths": [],
        }),
    }
}

fn default_six_resource_doctor_section() -> Value {
    let observations: Vec<SixResourceHealthObservation> = SixResourceFamily::ALL
        .iter()
        .map(|family| SixResourceHealthObservation {
            family: *family,
            status: SixResourceHealthStatus::NotConfigured,
            error_code: Some("RESOURCE_HEALTH_NOT_PROBED"),
            recovery_hint: Some("await resource-specific doctor probe"),
            facts: vec![SixResourceHealthFact {
                key: "probe",
                value: "not_run".to_owned(),
            }],
        })
        .collect();
    match evaluate_six_resource_doctor_health(&observations) {
        Ok(report) => six_resource_doctor_projection_json(&report),
        Err(_) => json!({
            "schema": "personal-six-resource-doctor",
            "schema_version": 1,
            "surface": "personal-doctor-six-resource",
            "overall": "blocked",
            "gate_claim": "not-claimed",
            "profile_claim": "not-claimed",
            "error_code": "SIX_RESOURCE_DOCTOR_INTERNAL",
            "families": [],
        }),
    }
}

fn component_summary_json(check: &ComponentCheck) -> Value {
    json!({
        "component": check.component,
        "status": check.status.as_str(),
        "required": check.required,
        "error_class": check.error_class,
        "duration_ms": check.duration_ms
    })
}

fn component_detail_json(check: &ComponentCheck) -> Value {
    let facts: Vec<Value> = check
        .facts
        .iter()
        .map(|fact| {
            json!({
                "key": fact.key,
                "value": fact.value
            })
        })
        .collect();
    json!({
        "component": check.component,
        "status": check.status.as_str(),
        "required": check.required,
        "source": check.source,
        "duration_ms": check.duration_ms,
        "observed_at_unix_ms": check.observed_at_unix_ms,
        "error_class": check.error_class,
        "facts": facts
    })
}

fn doctor_guidance(report: &ReadinessReport) -> Vec<&'static str> {
    let mut guidance = Vec::new();
    for component in &report.components {
        match (component.component, component.status) {
            ("system", ComponentStatus::Blocked) => {
                guidance.push("create Personal XDG layout directories before continuing");
            }
            ("database", ComponentStatus::Blocked) => {
                guidance
                    .push("run personal database prepare / migration before first conversation");
            }
            ("secret", ComponentStatus::Blocked) => {
                guidance.push("configure a native SecretStore backend and store the Provider key");
            }
            ("provider", ComponentStatus::Blocked)
                if matches!(
                    component.error_class,
                    Some(
                        "provider_selected_model_missing"
                            | "provider_selected_model_unusable"
                            | "provider_selected_model_digest_mismatch"
                    )
                ) =>
            {
                guidance.push(
                    "rerun cognitive init with the configured Provider and exact model so a successful probe can persist matching selected-model state",
                );
            }
            ("provider", ComponentStatus::Blocked) => {
                guidance.push("write provider.json via cognitive init with an opaque secret_ref");
            }
            ("provider", ComponentStatus::Degraded) => {
                guidance
                    .push("run provider discovery probe to persist a capability snapshot digest");
            }
            ("daemon", ComponentStatus::Blocked) => {
                guidance.push("start kernel-server --personal and confirm loopback listen");
            }
            ("pi", ComponentStatus::NotConfigured) => {
                guidance.push(
                    "write pi.json with an absolute Pi executable path and CognitiveOS Extension entry path; first conversation stays blocked until Pi is configured",
                );
            }
            ("pi", ComponentStatus::Blocked) => {
                guidance.push(
                    "install the pinned Pi version and the built CognitiveOS Extension at the paths recorded in pi.json; see the pi component facts for which check failed",
                );
            }
            _ => {}
        }
    }
    if guidance.is_empty() {
        guidance.push("all required runtime checks currently report ready facts");
    }
    guidance
}

fn aggregate_overall(components: &[ComponentCheck]) -> OverallReadiness {
    let mut saw_degraded = false;
    for component in components {
        if !component.required {
            continue;
        }
        match component.status {
            ComponentStatus::Blocked => return OverallReadiness::Blocked,
            ComponentStatus::Degraded | ComponentStatus::NotConfigured => saw_degraded = true,
            ComponentStatus::Ready => {}
        }
    }
    if saw_degraded {
        OverallReadiness::Degraded
    } else {
        OverallReadiness::Ready
    }
}

fn check_system(layout: &PersonalDataLayout, observed_at_unix_ms: u64) -> ComponentCheck {
    let started = Instant::now();
    let mut facts = Vec::new();
    let mut missing = Vec::new();
    for (label, path) in [
        ("config_dir", layout.config_dir()),
        ("data_dir", layout.data_dir()),
        ("state_dir", layout.state_dir()),
        ("cache_dir", layout.cache_dir()),
        ("runtime_dir", layout.runtime_dir()),
    ] {
        let exists = path.is_dir();
        facts.push(ReadinessFact {
            key: label,
            value: if exists {
                "present".to_owned()
            } else {
                "missing".to_owned()
            },
        });
        if !exists {
            missing.push(label);
        }
    }
    let (status, error_class) = if missing.is_empty() {
        (ComponentStatus::Ready, None)
    } else {
        (ComponentStatus::Blocked, Some("layout_missing"))
    };
    ComponentCheck {
        component: "system",
        status,
        required: true,
        source: "filesystem:xdg-layout",
        duration_ms: elapsed_ms(started),
        observed_at_unix_ms,
        error_class,
        facts,
    }
}

fn check_database(layout: &PersonalDataLayout, observed_at_unix_ms: u64) -> ComponentCheck {
    let started = Instant::now();
    let authority = layout.authority_database_path();
    let installation = layout.installation_database_path();
    let authority_present = authority.is_file();
    let installation_present = installation.is_file();
    let mut facts = vec![
        ReadinessFact {
            key: "authority_database",
            value: presence_token(authority_present),
        },
        ReadinessFact {
            key: "installation_database",
            value: presence_token(installation_present),
        },
    ];
    // File presence is a runtime fact, not SQLite integrity proof. Integrity
    // campaigns remain out of scope for this projection.
    facts.push(ReadinessFact {
        key: "integrity_claim",
        value: "not-claimed".to_owned(),
    });
    let (status, error_class) = if authority_present && installation_present {
        (ComponentStatus::Ready, None)
    } else {
        (ComponentStatus::Blocked, Some("database_not_prepared"))
    };
    ComponentCheck {
        component: "database",
        status,
        required: true,
        source: "filesystem:sqlite-paths",
        duration_ms: elapsed_ms(started),
        observed_at_unix_ms,
        error_class,
        facts,
    }
}

fn check_secret(context: &ReadinessEvaluationContext, observed_at_unix_ms: u64) -> ComponentCheck {
    let started = Instant::now();
    let observation = context
        .secret_probe_override
        .unwrap_or_else(probe_production_secret_store);
    let mut facts = vec![
        ReadinessFact {
            key: "backend_class",
            value: secret_class_token(observation.class).to_owned(),
        },
        ReadinessFact {
            key: "availability",
            value: secret_availability_token(observation.availability).to_owned(),
        },
    ];
    // Never embed secret material or SecretRef contents from get().
    facts.push(ReadinessFact {
        key: "secret_material_exposed",
        value: "false".to_owned(),
    });
    let (status, error_class) = match (observation.class, observation.availability) {
        (SecretStoreClass::Native, SecretStoreAvailability::Available) => {
            (ComponentStatus::Ready, None)
        }
        (SecretStoreClass::EphemeralTestDouble, SecretStoreAvailability::Available) => {
            // Allowed only in hermetic tests; production never selects this class.
            (ComponentStatus::Ready, None)
        }
        (_, SecretStoreAvailability::Locked) => {
            (ComponentStatus::Blocked, Some("secret_store_locked"))
        }
        (_, SecretStoreAvailability::PromptUnavailable) => {
            (ComponentStatus::Blocked, Some("secret_prompt_unavailable"))
        }
        (_, SecretStoreAvailability::Unavailable) | (SecretStoreClass::Unavailable, _) => {
            (ComponentStatus::Blocked, Some("secret_store_unavailable"))
        }
    };
    ComponentCheck {
        component: "secret",
        status,
        required: true,
        source: "secret-store:probe",
        duration_ms: elapsed_ms(started),
        observed_at_unix_ms,
        error_class,
        facts,
    }
}

fn check_provider(
    context: &ReadinessEvaluationContext,
    observed_at_unix_ms: u64,
) -> ComponentCheck {
    let started = Instant::now();
    let config_path = context
        .provider_config_path_override
        .clone()
        .unwrap_or_else(|| {
            context
                .layout
                .config_dir()
                .join(cognitive_secret::PROVIDER_CONFIG_FILE_NAME)
        });
    let repository = ProviderConfigRepository::from_file_path(&config_path);
    match repository.load() {
        Ok(config) => {
            let resolution = context
                .provider_secret_resolution_override
                .unwrap_or_else(|| {
                    context.provider_secret_store_override.as_ref().map_or_else(
                        || resolve_production_provider_secret(&config),
                        |store| resolve_provider_secret_from_snapshot(&config, store.as_ref()),
                    )
                });
            let selected_snapshot_digest = config.selected_snapshot_digest();
            let digest_present = selected_snapshot_digest.is_some();
            let mut facts = vec![
                ReadinessFact {
                    key: "provider_id",
                    value: config.provider_id().to_owned(),
                },
                ReadinessFact {
                    key: "base_url_scheme",
                    value: if config.base_url().starts_with("https://") {
                        "https".to_owned()
                    } else {
                        "non-https".to_owned()
                    },
                },
                ReadinessFact {
                    key: "secret_ref_present",
                    value: "true".to_owned(),
                },
                ReadinessFact {
                    key: "secret_ref_resolves",
                    value: provider_secret_resolution_token(resolution).to_owned(),
                },
                ReadinessFact {
                    key: "selected_snapshot_digest_present",
                    value: if digest_present {
                        "true".to_owned()
                    } else {
                        "false".to_owned()
                    },
                },
            ];
            // Opaque ref identity must not be printed.
            facts.push(ReadinessFact {
                key: "secret_ref_redacted",
                value: "true".to_owned(),
            });
            let (status, error_class) = match selected_snapshot_digest {
                None => (
                    ComponentStatus::Degraded,
                    Some("provider_snapshot_digest_missing"),
                ),
                Some(expected_digest) => {
                    let selected_model_repository =
                        SelectedModelRepository::under_config_dir(repository.config_dir());
                    match selected_model_repository.load() {
                        Ok(Some(selected_model))
                            if selected_model.selected_snapshot_digest() == expected_digest =>
                        {
                            facts.push(ReadinessFact {
                                key: "selected_model_present",
                                value: "true".to_owned(),
                            });
                            facts.push(ReadinessFact {
                                key: "selected_model_digest_matches",
                                value: "true".to_owned(),
                            });
                            (ComponentStatus::Ready, None)
                        }
                        Ok(Some(_)) => {
                            facts.push(ReadinessFact {
                                key: "selected_model_present",
                                value: "true".to_owned(),
                            });
                            facts.push(ReadinessFact {
                                key: "selected_model_digest_matches",
                                value: "false".to_owned(),
                            });
                            (
                                ComponentStatus::Blocked,
                                Some("provider_selected_model_digest_mismatch"),
                            )
                        }
                        Ok(None) => {
                            facts.push(ReadinessFact {
                                key: "selected_model_present",
                                value: "false".to_owned(),
                            });
                            (
                                ComponentStatus::Blocked,
                                Some("provider_selected_model_missing"),
                            )
                        }
                        Err(_) => {
                            facts.push(ReadinessFact {
                                key: "selected_model_present",
                                value: "unusable".to_owned(),
                            });
                            (
                                ComponentStatus::Blocked,
                                Some("provider_selected_model_unusable"),
                            )
                        }
                    }
                }
            };
            // A configured Provider whose secret_ref no longer resolves cannot
            // serve a single request, so it must never read as ready. This
            // outranks the snapshot-digest verdict above.
            let (status, error_class) = match resolution {
                ProviderSecretResolution::Missing => (
                    ComponentStatus::Blocked,
                    Some("provider_secret_unresolvable"),
                ),
                ProviderSecretResolution::Unavailable => (
                    ComponentStatus::Blocked,
                    Some("provider_secret_store_unavailable"),
                ),
                ProviderSecretResolution::Resolves => (status, error_class),
            };
            ComponentCheck {
                component: "provider",
                status,
                required: true,
                source: "filesystem:provider-config+secret-store:resolve",
                duration_ms: elapsed_ms(started),
                observed_at_unix_ms,
                error_class,
                facts,
            }
        }
        Err(ProviderConfigError::NotFound) => ComponentCheck {
            component: "provider",
            status: ComponentStatus::Blocked,
            required: true,
            source: "filesystem:provider-config",
            duration_ms: elapsed_ms(started),
            observed_at_unix_ms,
            error_class: Some("provider_config_missing"),
            facts: vec![ReadinessFact {
                key: "provider_config",
                value: "missing".to_owned(),
            }],
        },
        Err(ProviderConfigError::Corrupt { .. }) => ComponentCheck {
            component: "provider",
            status: ComponentStatus::Blocked,
            required: true,
            source: "filesystem:provider-config",
            duration_ms: elapsed_ms(started),
            observed_at_unix_ms,
            error_class: Some("provider_config_corrupt"),
            facts: vec![ReadinessFact {
                key: "provider_config",
                value: "corrupt".to_owned(),
            }],
        },
        Err(ProviderConfigError::Invalid { .. }) | Err(ProviderConfigError::Io { .. }) => {
            ComponentCheck {
                component: "provider",
                status: ComponentStatus::Blocked,
                required: true,
                source: "filesystem:provider-config",
                duration_ms: elapsed_ms(started),
                observed_at_unix_ms,
                error_class: Some("provider_config_unreadable"),
                facts: vec![ReadinessFact {
                    key: "provider_config",
                    value: "unreadable".to_owned(),
                }],
            }
        }
    }
}

fn check_daemon(context: &ReadinessEvaluationContext, observed_at_unix_ms: u64) -> ComponentCheck {
    let started = Instant::now();
    let lock_present = context.layout.daemon_lock_path().is_file();
    let bootstrap_present = context.layout.local_bootstrap_secret_path().is_file();
    let facts = vec![
        ReadinessFact {
            key: "listening",
            value: if context.daemon_listening {
                "true".to_owned()
            } else {
                "false".to_owned()
            },
        },
        ReadinessFact {
            key: "daemon_lock",
            value: presence_token(lock_present),
        },
        ReadinessFact {
            key: "bootstrap_secret",
            value: presence_token(bootstrap_present),
        },
        ReadinessFact {
            key: "session_count",
            value: context.session_count.to_string(),
        },
    ];
    let (status, error_class) = if context.daemon_listening && lock_present && bootstrap_present {
        (ComponentStatus::Ready, None)
    } else if context.daemon_listening {
        (ComponentStatus::Degraded, Some("daemon_runtime_partial"))
    } else {
        (ComponentStatus::Blocked, Some("daemon_not_listening"))
    };
    ComponentCheck {
        component: "daemon",
        status,
        required: true,
        source: "runtime:personal-daemon",
        duration_ms: elapsed_ms(started),
        observed_at_unix_ms,
        error_class,
        facts,
    }
}

fn check_pi(context: &ReadinessEvaluationContext, observed_at_unix_ms: u64) -> ComponentCheck {
    // P1-T07 replaced the hard-coded placeholder with a real observation. The
    // component stays optional so ADR-0023's aggregation rules are unchanged;
    // only its status is now fact-derived. Absence is still reported as
    // absence: an unconfigured host reads `not_configured`, exactly as before.
    let started = Instant::now();
    let observation = match &context.pi_observation_override {
        Some(observation) => observation.clone(),
        None => observe_pi_runtime(context.layout.config_dir()),
    };

    let status = match observation {
        PiRuntimeObservation::Ready => ComponentStatus::Ready,
        PiRuntimeObservation::NotConfigured => ComponentStatus::NotConfigured,
        _ => ComponentStatus::Blocked,
    };

    let mut facts = vec![
        ReadinessFact {
            key: "package_status",
            value: observation.package_status().to_owned(),
        },
        ReadinessFact {
            key: "pinned_version",
            value: PINNED_PI_VERSION.to_owned(),
        },
    ];
    if let Some(observed_version) = observation.observed_version() {
        facts.push(ReadinessFact {
            key: "observed_version",
            value: observed_version.to_owned(),
        });
    }
    // The Extension is a client surface; a ready Pi component never implies a
    // Gate, a sandbox, or a governed AgentInstallation.
    facts.push(ReadinessFact {
        key: "containment_claim",
        value: "not-claimed".to_owned(),
    });

    ComponentCheck {
        component: "pi",
        status,
        required: false,
        source: "product:pi-package",
        duration_ms: elapsed_ms(started),
        observed_at_unix_ms,
        error_class: observation.error_class(),
        facts,
    }
}

fn probe_production_secret_store() -> SecretProbeObservation {
    let backend = select_production_secret_store();
    let class = backend.class();
    let availability = backend
        .probe()
        .unwrap_or(SecretStoreAvailability::Unavailable);
    SecretProbeObservation {
        class,
        availability,
    }
}

/// Attempt to resolve the configured Provider secret ref against the production
/// SecretStore. The resolved material is dropped immediately and never enters a
/// fact, a log, or the report; only the three-way outcome is retained.
fn resolve_production_provider_secret(config: &ProviderConfig) -> ProviderSecretResolution {
    let backend = select_production_secret_store();
    resolve_provider_secret_from_snapshot(config, backend.as_secret_store())
}

fn resolve_provider_secret_from_snapshot<S: SecretStore + ?Sized>(
    config: &ProviderConfig,
    store: &S,
) -> ProviderSecretResolution {
    match store.get(config.secret_ref()) {
        Ok(_material) => ProviderSecretResolution::Resolves,
        Err(SecretError::NotFound) => ProviderSecretResolution::Missing,
        Err(_) => ProviderSecretResolution::Unavailable,
    }
}

fn provider_secret_resolution_token(resolution: ProviderSecretResolution) -> &'static str {
    match resolution {
        ProviderSecretResolution::Resolves => "true",
        ProviderSecretResolution::Missing => "false",
        ProviderSecretResolution::Unavailable => "unknown",
    }
}

fn presence_token(present: bool) -> String {
    if present {
        "present".to_owned()
    } else {
        "missing".to_owned()
    }
}

fn secret_class_token(class: SecretStoreClass) -> &'static str {
    match class {
        SecretStoreClass::Native => "native",
        SecretStoreClass::EphemeralTestDouble => "ephemeral_test_double",
        SecretStoreClass::Unavailable => "unavailable",
    }
}

fn secret_availability_token(availability: SecretStoreAvailability) -> &'static str {
    match availability {
        SecretStoreAvailability::Available => "available",
        SecretStoreAvailability::Locked => "locked",
        SecretStoreAvailability::PromptUnavailable => "prompt_unavailable",
        SecretStoreAvailability::Unavailable => "unavailable",
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    let duration = started.elapsed();
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis()
        .try_into()
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_secret::{
        ProviderConfig, SecretAttributes, SecretLabel, SecretMaterial, SecretRef, SelectedModel,
        SelectedModelRepository,
    };
    use std::fs;
    use std::sync::Mutex;

    struct ConfigSwappingSecretStore {
        config_path: PathBuf,
        replacement: ProviderConfig,
        expected_ref: SecretRef,
        observed_refs: Mutex<Vec<String>>,
    }

    impl SecretStore for ConfigSwappingSecretStore {
        fn class(&self) -> SecretStoreClass {
            SecretStoreClass::EphemeralTestDouble
        }

        fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
            Ok(SecretStoreAvailability::Available)
        }

        fn put(
            &self,
            _label: &SecretLabel,
            _attributes: &SecretAttributes,
            _material: SecretMaterial,
        ) -> Result<SecretRef, SecretError> {
            Err(SecretError::Backend {
                detail: "test store is read-only",
            })
        }

        fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
            self.observed_refs
                .lock()
                .map_err(|_| SecretError::Backend {
                    detail: "test observation lock poisoned",
                })?
                .push(secret_ref.as_str().to_owned());
            ProviderConfigRepository::from_file_path(&self.config_path)
                .store(&self.replacement)
                .map_err(|_| SecretError::Backend {
                    detail: "test config swap failed",
                })?;
            if secret_ref == &self.expected_ref {
                SecretMaterial::from_bytes(b"snapshot-a-material".to_vec())
            } else {
                Err(SecretError::NotFound)
            }
        }

        fn delete(&self, _secret_ref: &SecretRef) -> Result<(), SecretError> {
            Err(SecretError::Backend {
                detail: "test store is read-only",
            })
        }
    }

    fn touch_personal_database_files(layout: &PersonalDataLayout) -> std::io::Result<()> {
        touch_file(&layout.authority_database_path())?;
        touch_file(&layout.installation_database_path())?;
        Ok(())
    }

    fn touch_file(path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)
            .map(|_| ())
    }

    fn temp_layout(label: &str) -> PersonalDataLayout {
        let root = std::env::temp_dir().join(format!(
            "cos-p1t05-{}-{}-{}",
            label,
            std::process::id(),
            unix_now_ms()
        ));
        let _ = fs::remove_dir_all(&root);
        let layout = PersonalDataLayout::from_xdg_roots(&root, &root, &root, &root, &root);
        layout.ensure_directories().expect("dirs");
        layout
    }

    fn available_secret_probe() -> SecretProbeObservation {
        SecretProbeObservation {
            class: SecretStoreClass::EphemeralTestDouble,
            availability: SecretStoreAvailability::Available,
        }
    }

    fn write_provider_config(layout: &PersonalDataLayout, with_digest: bool) {
        let secret_ref = SecretRef::from_opaque("secretref:test-provider-p1t05").expect("ref");
        let digest = if with_digest {
            Some("fnv1a64:0123456789abcdef".to_owned())
        } else {
            None
        };
        let config = ProviderConfig::new(
            "deepseek",
            "https://api.deepseek.com/v1",
            secret_ref,
            digest,
        )
        .expect("config");
        ProviderConfigRepository::under_config_dir(layout.config_dir())
            .store(&config)
            .expect("store config");
    }

    fn write_selected_model(layout: &PersonalDataLayout, digest: &str) {
        let selected_model =
            SelectedModel::new("test-chat-model", digest, true).expect("selected model");
        SelectedModelRepository::under_config_dir(layout.config_dir())
            .store(&selected_model)
            .expect("store selected model");
    }

    #[test]
    fn missing_databases_and_provider_are_blocked_not_ready() {
        let layout = temp_layout("blocked");
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 0,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: None,
        });
        assert_eq!(report.overall, OverallReadiness::Blocked);
        assert!(!report.first_conversation_ready);
        let database = report
            .components
            .iter()
            .find(|component| component.component == "database")
            .expect("database component");
        assert_eq!(database.status, ComponentStatus::Blocked);
        assert_eq!(database.error_class, Some("database_not_prepared"));
        let status_json = status_projection_json(&report);
        assert_eq!(status_json["overall"], "blocked");
        assert_eq!(status_json["profile_claim"], "not-claimed");
        assert_eq!(status_json["static_check_is_not_runtime_ready"], true);
    }

    #[test]
    fn provider_without_snapshot_digest_is_degraded() {
        let layout = temp_layout("degraded");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, false);
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 1,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: None,
        });
        // daemon lock/bootstrap may be missing → degraded or blocked; force
        // only provider path by requiring provider degraded and overall not ready.
        let provider = report
            .components
            .iter()
            .find(|component| component.component == "provider")
            .expect("provider");
        assert_eq!(provider.status, ComponentStatus::Degraded);
        assert_eq!(
            provider.error_class,
            Some("provider_snapshot_digest_missing")
        );
        assert_ne!(report.overall, OverallReadiness::Ready);
        assert!(!report.first_conversation_ready);
    }

    #[test]
    fn selected_model_must_match_the_successful_provider_snapshot() {
        let layout = temp_layout("selected-model-required");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        fs::write(layout.daemon_lock_path(), b"lock").unwrap();
        fs::write(layout.local_bootstrap_secret_path(), b"bootstrap").unwrap();

        let missing_selected_model = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout: layout.clone(),
            daemon_listening: true,
            session_count: 1,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: Some(PiRuntimeObservation::Ready),
        });
        assert_eq!(missing_selected_model.overall, OverallReadiness::Blocked);
        assert!(!missing_selected_model.first_conversation_ready);
        let missing_provider = missing_selected_model
            .components
            .iter()
            .find(|component| component.component == "provider")
            .expect("provider component");
        assert_eq!(missing_provider.status, ComponentStatus::Blocked);
        assert_eq!(
            missing_provider.error_class,
            Some("provider_selected_model_missing")
        );

        write_selected_model(&layout, "fnv1a64:different-snapshot");
        let mismatched_selected_model = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 1,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: Some(PiRuntimeObservation::Ready),
        });
        assert_eq!(mismatched_selected_model.overall, OverallReadiness::Blocked);
        assert!(!mismatched_selected_model.first_conversation_ready);
        let mismatched_provider = mismatched_selected_model
            .components
            .iter()
            .find(|component| component.component == "provider")
            .expect("provider component");
        assert_eq!(mismatched_provider.status, ComponentStatus::Blocked);
        assert_eq!(
            mismatched_provider.error_class,
            Some("provider_selected_model_digest_mismatch")
        );
    }

    /// PERSONAL-PERF-EVAL-002 observed 80/80 Provider requests refused with
    /// `PERSONAL_PROVIDER_SECRET_UNAVAILABLE` while `status` and `doctor`
    /// reported `provider: ready`. A reachable backend is not a resolvable ref.
    #[test]
    fn dangling_provider_secret_ref_never_reads_as_ready() {
        let layout = temp_layout("dangling-secret-ref");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        write_selected_model(&layout, "fnv1a64:0123456789abcdef");
        fs::write(layout.daemon_lock_path(), b"lock").unwrap();
        fs::write(layout.local_bootstrap_secret_path(), b"bootstrap").unwrap();
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 1,
            // The backend itself is reachable; only the referenced item is gone.
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Missing),
            provider_secret_store_override: None,
            pi_observation_override: Some(PiRuntimeObservation::Ready),
        });
        let provider = report
            .components
            .iter()
            .find(|component| component.component == "provider")
            .expect("provider component");
        assert_eq!(provider.status, ComponentStatus::Blocked);
        assert_eq!(provider.error_class, Some("provider_secret_unresolvable"));
        assert!(
            provider
                .facts
                .iter()
                .any(|fact| fact.key == "secret_ref_resolves" && fact.value == "false"),
            "readiness must publish that the configured ref does not resolve"
        );
        assert_eq!(report.overall, OverallReadiness::Blocked);
        assert!(
            !report.first_conversation_ready,
            "a first conversation cannot be ready without a resolvable Provider key"
        );
    }

    /// An unreachable backend must not be reported as a resolvable ref either;
    /// unknown is its own answer, not an optimistic one.
    #[test]
    fn unresolvable_provider_secret_store_blocks_rather_than_assumes() {
        let layout = temp_layout("secret-store-unknown");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        write_selected_model(&layout, "fnv1a64:0123456789abcdef");
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 0,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Unavailable),
            provider_secret_store_override: None,
            pi_observation_override: Some(PiRuntimeObservation::Ready),
        });
        let provider = report
            .components
            .iter()
            .find(|component| component.component == "provider")
            .expect("provider component");
        assert_eq!(provider.status, ComponentStatus::Blocked);
        assert_eq!(
            provider.error_class,
            Some("provider_secret_store_unavailable")
        );
        assert!(
            provider
                .facts
                .iter()
                .any(|fact| fact.key == "secret_ref_resolves" && fact.value == "unknown")
        );
    }

    #[test]
    fn provider_secret_resolution_uses_the_already_loaded_config_snapshot() {
        let layout = temp_layout("provider-snapshot");
        touch_personal_database_files(&layout).unwrap();
        let config_path = layout
            .config_dir()
            .join(cognitive_secret::PROVIDER_CONFIG_FILE_NAME);
        let ref_a = SecretRef::from_opaque("secretref:snapshot-a").expect("snapshot A ref");
        let config_a = ProviderConfig::new(
            "provider-a",
            "https://provider-a.example/v1",
            ref_a.clone(),
            Some("fnv1a64:aaaaaaaaaaaaaaaa".to_owned()),
        )
        .expect("snapshot A");
        ProviderConfigRepository::from_file_path(&config_path)
            .store(&config_a)
            .expect("store snapshot A");
        write_selected_model(&layout, "fnv1a64:aaaaaaaaaaaaaaaa");
        let config_b = ProviderConfig::new(
            "provider-b",
            "https://provider-b.example/v1",
            SecretRef::from_opaque("secretref:snapshot-b").expect("snapshot B ref"),
            Some("fnv1a64:bbbbbbbbbbbbbbbb".to_owned()),
        )
        .expect("snapshot B");
        let store = Arc::new(ConfigSwappingSecretStore {
            config_path: config_path.clone(),
            replacement: config_b,
            expected_ref: ref_a.clone(),
            observed_refs: Mutex::new(Vec::new()),
        });
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 0,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: Some(config_path.clone()),
            provider_secret_resolution_override: None,
            provider_secret_store_override: Some(store.clone()),
            pi_observation_override: None,
        });
        let provider = report
            .components
            .iter()
            .find(|component| component.component == "provider")
            .expect("provider component");
        assert_eq!(provider.status, ComponentStatus::Ready);
        assert!(
            provider
                .facts
                .iter()
                .any(|fact| fact.key == "provider_id" && fact.value == "provider-a")
        );
        assert!(
            provider
                .facts
                .iter()
                .any(|fact| fact.key == "secret_ref_resolves" && fact.value == "true")
        );
        assert_eq!(
            store
                .observed_refs
                .lock()
                .expect("observed refs")
                .as_slice(),
            &[ref_a.as_str().to_owned()]
        );
        assert_eq!(
            ProviderConfigRepository::from_file_path(&config_path)
                .load()
                .expect("load swapped snapshot")
                .provider_id(),
            "provider-b",
            "the fake store must swap the file during resolution"
        );
    }

    #[test]
    fn full_runtime_facts_yield_ready_overall_but_first_conversation_blocked_without_pi() {
        let layout = temp_layout("ready");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        write_selected_model(&layout, "fnv1a64:0123456789abcdef");
        // Simulate daemon runtime artifacts without starting a server.
        fs::write(layout.daemon_lock_path(), b"lock").unwrap();
        fs::write(layout.local_bootstrap_secret_path(), b"bootstrap").unwrap();
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 2,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: None,
        });
        assert_eq!(report.overall, OverallReadiness::Ready);
        assert!(
            !report.first_conversation_ready,
            "Pi remains not_configured until P1-T07"
        );
        let pi = report
            .components
            .iter()
            .find(|component| component.component == "pi")
            .expect("pi");
        assert_eq!(pi.status, ComponentStatus::NotConfigured);
        assert_eq!(pi.error_class, Some("pi_not_configured"));
        let doctor = doctor_projection_json(&report);
        assert_eq!(doctor["overall"], "ready");
        assert_eq!(doctor["first_conversation_ready"], false);
        assert_eq!(doctor["gate_claim"], "not-claimed");
        assert_eq!(
            doctor["six_resource"]["surface"],
            "personal-doctor-six-resource"
        );
        assert_eq!(doctor["six_resource"]["gate_claim"], "not-claimed");
        assert_eq!(
            doctor["six_resource"]["families"]
                .as_array()
                .expect("six families")
                .len(),
            6
        );
        assert_eq!(
            doctor["headless_vault"]["surface"],
            "personal-doctor-headless-vault"
        );
        assert_eq!(doctor["headless_vault"]["gate_claim"], "not-claimed");
        assert_eq!(
            doctor["operability"]["surface"],
            "personal-doctor-operability"
        );
        assert_eq!(doctor["operability"]["gate_claim"], "not-claimed");
        let guidance = doctor["guidance"].as_array().expect("guidance array");
        assert!(
            guidance.iter().any(|entry| entry
                .as_str()
                .is_some_and(|text| text.contains("write pi.json"))),
            "an unconfigured Pi must produce actionable guidance"
        );
    }

    #[test]
    fn a_ready_pi_observation_unblocks_first_conversation_without_changing_aggregation() {
        let layout = temp_layout("pi-ready");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        write_selected_model(&layout, "fnv1a64:0123456789abcdef");
        fs::write(layout.daemon_lock_path(), b"lock").unwrap();
        fs::write(layout.local_bootstrap_secret_path(), b"bootstrap").unwrap();
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 1,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: Some(PiRuntimeObservation::Ready),
        });
        assert_eq!(report.overall, OverallReadiness::Ready);
        assert!(report.first_conversation_ready);
        let pi = report
            .components
            .iter()
            .find(|component| component.component == "pi")
            .expect("pi");
        assert_eq!(pi.status, ComponentStatus::Ready);
        assert!(!pi.required, "ADR-0023 keeps the pi component optional");
        assert_eq!(pi.error_class, None);
        let doctor = doctor_projection_json(&report);
        // A ready Pi is still not a Gate, sandbox or Profile claim.
        assert_eq!(doctor["gate_claim"], "not-claimed");
        assert_eq!(doctor["profile_claim"], "not-claimed");
        assert_eq!(doctor["static_check_is_not_runtime_ready"], true);
        let facts = doctor["components"]
            .as_array()
            .expect("components")
            .iter()
            .find(|component| component["component"] == "pi")
            .expect("pi component")["facts"]
            .as_array()
            .expect("facts")
            .clone();
        assert!(
            facts
                .iter()
                .any(|fact| fact["key"] == "containment_claim" && fact["value"] == "not-claimed")
        );
        assert!(facts
            .iter()
            .any(|fact| fact["key"] == "observed_version" && fact["value"] == PINNED_PI_VERSION));
    }

    #[test]
    fn a_broken_pi_observation_blocks_the_component_without_blocking_overall() {
        let cases = [
            (
                PiRuntimeObservation::ExecutableMissing,
                "pi_executable_missing",
            ),
            (
                PiRuntimeObservation::ExtensionMissing,
                "pi_extension_missing",
            ),
            (
                PiRuntimeObservation::VersionMismatch {
                    observed: "0.82.0".to_owned(),
                },
                "pi_version_mismatch",
            ),
            (PiRuntimeObservation::ProbeTimedOut, "pi_probe_timeout"),
            (
                PiRuntimeObservation::ConfigUnusable { detail: "bad" },
                "pi_config_unusable",
            ),
        ];
        for (index, (observation, expected_error_class)) in cases.into_iter().enumerate() {
            let layout = temp_layout(&format!("pi-broken-{index}"));
            touch_personal_database_files(&layout).unwrap();
            write_provider_config(&layout, true);
            write_selected_model(&layout, "fnv1a64:0123456789abcdef");
            fs::write(layout.daemon_lock_path(), b"lock").unwrap();
            fs::write(layout.local_bootstrap_secret_path(), b"bootstrap").unwrap();
            let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
                layout,
                daemon_listening: true,
                session_count: 0,
                secret_probe_override: Some(available_secret_probe()),
                provider_config_path_override: None,
                provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
                provider_secret_store_override: None,
                pi_observation_override: Some(observation),
            });
            // `pi` is optional, so a broken Pi never rewrites the required-set
            // aggregate. It only keeps the first conversation blocked.
            assert_eq!(report.overall, OverallReadiness::Ready);
            assert!(!report.first_conversation_ready);
            let pi = report
                .components
                .iter()
                .find(|component| component.component == "pi")
                .expect("pi");
            assert_eq!(pi.status, ComponentStatus::Blocked);
            assert_eq!(pi.error_class, Some(expected_error_class));
        }
    }

    #[test]
    fn secret_store_locked_is_blocked_and_does_not_expose_material() {
        let layout = temp_layout("secret-locked");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        fs::write(layout.daemon_lock_path(), b"lock").unwrap();
        let bootstrap_material = "bootstrap-secret-material-p1t05-redaction";
        fs::write(
            layout.local_bootstrap_secret_path(),
            bootstrap_material.as_bytes(),
        )
        .unwrap();
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 0,
            secret_probe_override: Some(SecretProbeObservation {
                class: SecretStoreClass::Native,
                availability: SecretStoreAvailability::Locked,
            }),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: None,
        });
        assert_eq!(report.overall, OverallReadiness::Blocked);
        let secret = report
            .components
            .iter()
            .find(|component| component.component == "secret")
            .expect("secret");
        assert_eq!(secret.status, ComponentStatus::Blocked);
        assert_eq!(secret.error_class, Some("secret_store_locked"));
        let doctor_text = doctor_projection_json(&report).to_string();
        assert!(!doctor_text.contains(bootstrap_material));
        assert!(!doctor_text.contains("test-material-not-a-real-key"));
    }

    #[test]
    fn opaque_secret_ref_is_not_serialized_in_projections() {
        let layout = temp_layout("redaction");
        touch_personal_database_files(&layout).unwrap();
        write_provider_config(&layout, true);
        let config = ProviderConfigRepository::under_config_dir(layout.config_dir())
            .load()
            .expect("load");
        let secret_ref = config.secret_ref().as_str().to_owned();
        fs::write(layout.daemon_lock_path(), b"lock").unwrap();
        fs::write(layout.local_bootstrap_secret_path(), b"bootstrap").unwrap();
        let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
            layout,
            daemon_listening: true,
            session_count: 0,
            secret_probe_override: Some(available_secret_probe()),
            provider_config_path_override: None,
            provider_secret_resolution_override: Some(ProviderSecretResolution::Resolves),
            provider_secret_store_override: None,
            pi_observation_override: None,
        });
        let doctor_text = doctor_projection_json(&report).to_string();
        let status_text = status_projection_json(&report).to_string();
        assert!(
            !doctor_text.contains(&secret_ref),
            "doctor must not embed opaque secret_ref"
        );
        assert!(
            !status_text.contains(&secret_ref),
            "status must not embed opaque secret_ref"
        );
    }
}
