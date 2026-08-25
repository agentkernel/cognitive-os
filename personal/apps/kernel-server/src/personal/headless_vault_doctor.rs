//! Headless vault doctor diagnostics (P7-T03/D02).
//!
//! Reports redacted diagnostic facts for desktop Secret Service and headless
//! encrypted-vault locked / TTY / unattended paths. Secret material must never
//! appear in unit, env, argv, or doctor facts. This module does not unlock
//! vaults, import keys, or claim Gate/release/Profile outcomes.

use serde_json::{Value, json};

const FORBIDDEN_VAULT_MARKERS: &[&str] = &[
    "secret",
    "credential",
    "api_key",
    "api-key",
    "private_key",
    "ssv1:",
    "sk-",
    "bearer ",
    "password=",
    "ENVIRONMENT=",
    "argv=",
];

/// Diagnostic path covered by the headless vault doctor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessVaultDoctorPath {
    DesktopSecretService,
    HeadlessLocked,
    HeadlessTtyUnlock,
    HeadlessUnattended,
}

impl HeadlessVaultDoctorPath {
    pub const ALL: [Self; 4] = [
        Self::DesktopSecretService,
        Self::HeadlessLocked,
        Self::HeadlessTtyUnlock,
        Self::HeadlessUnattended,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DesktopSecretService => "desktop_secret_service",
            Self::HeadlessLocked => "headless_locked",
            Self::HeadlessTtyUnlock => "headless_tty_unlock",
            Self::HeadlessUnattended => "headless_unattended",
        }
    }
}

/// Coarse redacted status for one vault diagnostic path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessVaultPathStatus {
    Ready,
    Locked,
    Unavailable,
    NotConfigured,
}

impl HeadlessVaultPathStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Locked => "locked",
            Self::Unavailable => "unavailable",
            Self::NotConfigured => "not_configured",
        }
    }
}

/// Caller-supplied non-secret observation for one vault path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessVaultPathObservation {
    pub path: HeadlessVaultDoctorPath,
    pub status: HeadlessVaultPathStatus,
    pub error_code: Option<&'static str>,
    pub recovery_hint: Option<&'static str>,
    /// Redacted facts only. Values must not contain secret markers or
    /// unit/env/argv contamination.
    pub facts: Vec<(String, String)>,
}

/// Redacted vault doctor report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessVaultDoctorReport {
    pub paths: Vec<HeadlessVaultPathObservation>,
    pub overall: HeadlessVaultPathStatus,
    pub gate_claim: &'static str,
    pub profile_claim: &'static str,
}

/// Fail-closed vault doctor errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadlessVaultDoctorError {
    MissingPath(&'static str),
    DuplicatePath(&'static str),
    SecretContamination,
    InvalidErrorCode,
}

/// Evaluate redacted diagnostics for all required vault paths.
pub fn evaluate_headless_vault_doctor(
    observations: &[HeadlessVaultPathObservation],
) -> Result<HeadlessVaultDoctorReport, HeadlessVaultDoctorError> {
    let mut seen = [false; 4];
    let mut paths = Vec::with_capacity(4);

    for observation in observations {
        let index = path_index(observation.path);
        if seen[index] {
            return Err(HeadlessVaultDoctorError::DuplicatePath(
                observation.path.as_str(),
            ));
        }
        seen[index] = true;
        validate_observation(observation)?;
        paths.push(observation.clone());
    }

    for (index, present) in seen.iter().enumerate() {
        if !present {
            return Err(HeadlessVaultDoctorError::MissingPath(
                HeadlessVaultDoctorPath::ALL[index].as_str(),
            ));
        }
    }

    paths.sort_by_key(|observation| path_index(observation.path));
    Ok(HeadlessVaultDoctorReport {
        overall: aggregate_overall(&paths),
        paths,
        gate_claim: "not-claimed",
        profile_claim: "not-claimed",
    })
}

/// JSON projection for doctor/support surfaces.
pub fn headless_vault_doctor_projection_json(report: &HeadlessVaultDoctorReport) -> Value {
    json!({
        "schema": "personal-headless-vault-doctor",
        "surface": "personal-doctor-headless-vault",
        "overall": report.overall.as_str(),
        "gate_claim": report.gate_claim,
        "profile_claim": report.profile_claim,
        "paths": report.paths.iter().map(|path| {
            json!({
                "path": path.path.as_str(),
                "status": path.status.as_str(),
                "error_code": path.error_code,
                "recovery_hint": path.recovery_hint,
                "facts": path.facts.iter().map(|(key, value)| {
                    json!({ "key": key, "value": value })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn validate_observation(
    observation: &HeadlessVaultPathObservation,
) -> Result<(), HeadlessVaultDoctorError> {
    if let Some(code) = observation.error_code {
        if !is_stable_error_code(code) {
            return Err(HeadlessVaultDoctorError::InvalidErrorCode);
        }
        if text_is_contaminated(code) {
            return Err(HeadlessVaultDoctorError::SecretContamination);
        }
    }
    if let Some(hint) = observation.recovery_hint
        && text_is_contaminated(hint)
    {
        return Err(HeadlessVaultDoctorError::SecretContamination);
    }
    for (key, value) in &observation.facts {
        if text_is_contaminated(key) || text_is_contaminated(value) {
            return Err(HeadlessVaultDoctorError::SecretContamination);
        }
    }
    Ok(())
}

fn aggregate_overall(paths: &[HeadlessVaultPathObservation]) -> HeadlessVaultPathStatus {
    if paths
        .iter()
        .any(|path| path.status == HeadlessVaultPathStatus::Unavailable)
    {
        return HeadlessVaultPathStatus::Unavailable;
    }
    if paths
        .iter()
        .any(|path| path.status == HeadlessVaultPathStatus::Locked)
    {
        return HeadlessVaultPathStatus::Locked;
    }
    if paths
        .iter()
        .any(|path| path.status == HeadlessVaultPathStatus::NotConfigured)
    {
        return HeadlessVaultPathStatus::NotConfigured;
    }
    HeadlessVaultPathStatus::Ready
}

fn path_index(path: HeadlessVaultDoctorPath) -> usize {
    match path {
        HeadlessVaultDoctorPath::DesktopSecretService => 0,
        HeadlessVaultDoctorPath::HeadlessLocked => 1,
        HeadlessVaultDoctorPath::HeadlessTtyUnlock => 2,
        HeadlessVaultDoctorPath::HeadlessUnattended => 3,
    }
}

fn is_stable_error_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn text_is_contaminated(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    FORBIDDEN_VAULT_MARKERS
        .iter()
        .any(|marker| lowered.contains(&marker.to_ascii_lowercase()))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn complete_observations() -> Vec<HeadlessVaultPathObservation> {
        HeadlessVaultDoctorPath::ALL
            .iter()
            .map(|path| HeadlessVaultPathObservation {
                path: *path,
                status: HeadlessVaultPathStatus::NotConfigured,
                error_code: Some("VAULT_PATH_NOT_PROBED"),
                recovery_hint: Some("await supported vault path probe"),
                facts: vec![("probe".to_owned(), "not_run".to_owned())],
            })
            .collect()
    }

    #[test]
    fn evaluates_complete_headless_vault_doctor_report() {
        let report =
            evaluate_headless_vault_doctor(&complete_observations()).expect("complete paths");
        assert_eq!(report.paths.len(), 4);
        assert_eq!(report.gate_claim, "not-claimed");
        let json = headless_vault_doctor_projection_json(&report);
        assert_eq!(json["surface"], "personal-doctor-headless-vault");
    }

    #[test]
    fn rejects_secret_in_unit_env_or_argv_shaped_facts() {
        let mut observations = complete_observations();
        observations[1].facts.push((
            "unit_snippet".to_owned(),
            "Environment=PROVIDER_API_KEY=sk-demo".to_owned(),
        ));
        assert_eq!(
            evaluate_headless_vault_doctor(&observations).unwrap_err(),
            HeadlessVaultDoctorError::SecretContamination
        );

        observations = complete_observations();
        observations[2].facts = vec![("argv".to_owned(), "--token=ssv1:abc".to_owned())];
        assert_eq!(
            evaluate_headless_vault_doctor(&observations).unwrap_err(),
            HeadlessVaultDoctorError::SecretContamination
        );
    }

    #[test]
    fn rejects_missing_path_and_records_locked_overall() {
        let mut observations = complete_observations();
        observations.pop();
        assert_eq!(
            evaluate_headless_vault_doctor(&observations).unwrap_err(),
            HeadlessVaultDoctorError::MissingPath("headless_unattended")
        );

        observations = complete_observations();
        observations[1].status = HeadlessVaultPathStatus::Locked;
        observations[1].error_code = Some("HEADLESS_VAULT_LOCKED");
        observations[1].recovery_hint = Some("unlock over SSH TTY then retry doctor");
        let report = evaluate_headless_vault_doctor(&observations).unwrap();
        assert_eq!(report.overall, HeadlessVaultPathStatus::Locked);
    }
}
