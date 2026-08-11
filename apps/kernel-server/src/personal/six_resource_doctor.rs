//! Six-resource doctor health projection (P7-T03/D01).
//!
//! Builds a redacted, non-claim six-family health report for Personal doctor
//! surfaces. Secrets, bootstrap material, and authority-shaped completion
//! claims are rejected. This module does not unlock Secret Stores, mutate
//! Tasks, or claim Gate/release/Profile outcomes.

use serde_json::{Value, json};

/// Stable product-local schema version for six-resource doctor reports.
pub const SIX_RESOURCE_DOCTOR_SCHEMA_VERSION: u32 = 1;

/// The six Personal resource families covered by doctor health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SixResourceFamily {
    Memory,
    Skill,
    Tool,
    Context,
    Task,
    Runtime,
}

impl SixResourceFamily {
    pub const ALL: [Self; 6] = [
        Self::Memory,
        Self::Skill,
        Self::Tool,
        Self::Context,
        Self::Task,
        Self::Runtime,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Skill => "skill",
            Self::Tool => "tool",
            Self::Context => "context",
            Self::Task => "task",
            Self::Runtime => "runtime",
        }
    }
}

/// Coarse redacted health for one resource family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SixResourceHealthStatus {
    Ready,
    Degraded,
    Blocked,
    NotConfigured,
}

impl SixResourceHealthStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Blocked => "blocked",
            Self::NotConfigured => "not_configured",
        }
    }
}

/// One redacted fact about a resource family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SixResourceHealthFact {
    pub key: &'static str,
    pub value: String,
}

/// Caller-supplied non-secret observation for one family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SixResourceHealthObservation {
    pub family: SixResourceFamily,
    pub status: SixResourceHealthStatus,
    pub error_code: Option<&'static str>,
    pub recovery_hint: Option<&'static str>,
    pub facts: Vec<SixResourceHealthFact>,
}

/// Redacted six-resource doctor report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SixResourceDoctorReport {
    pub schema_version: u32,
    pub families: Vec<SixResourceHealthObservation>,
    pub overall: SixResourceHealthStatus,
    pub gate_claim: &'static str,
    pub profile_claim: &'static str,
}

/// Fail-closed doctor health errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SixResourceDoctorError {
    MissingFamily(&'static str),
    DuplicateFamily(&'static str),
    SecretContamination,
    AuthorityClaimRejected,
    InvalidErrorCode,
}

const FORBIDDEN_DOCTOR_MARKERS: &[&str] = &[
    "secret",
    "credential",
    "api_key",
    "api-key",
    "private_key",
    "ssv1:",
    "sk-",
    "local-bootstrap.secret",
    "bearer ",
    "authorization:",
];

const FORBIDDEN_CLAIM_KEYS: &[&str] = &[
    "gate_pass",
    "profile_implemented",
    "task_completed",
    "release_ready",
    "gmvp_linux_pass",
];

/// Evaluate a complete six-family redacted doctor health report.
pub fn evaluate_six_resource_doctor_health(
    observations: &[SixResourceHealthObservation],
) -> Result<SixResourceDoctorReport, SixResourceDoctorError> {
    let mut seen = [false; 6];
    let mut families = Vec::with_capacity(6);

    for observation in observations {
        let index = family_index(observation.family);
        if seen[index] {
            return Err(SixResourceDoctorError::DuplicateFamily(
                observation.family.as_str(),
            ));
        }
        seen[index] = true;
        validate_observation(observation)?;
        families.push(observation.clone());
    }

    for (index, present) in seen.iter().enumerate() {
        if !present {
            return Err(SixResourceDoctorError::MissingFamily(
                SixResourceFamily::ALL[index].as_str(),
            ));
        }
    }

    families.sort_by_key(|observation| family_index(observation.family));
    let overall = aggregate_overall(&families);
    Ok(SixResourceDoctorReport {
        schema_version: SIX_RESOURCE_DOCTOR_SCHEMA_VERSION,
        families,
        overall,
        gate_claim: "not-claimed",
        profile_claim: "not-claimed",
    })
}

/// JSON projection for doctor/support surfaces.
pub fn six_resource_doctor_projection_json(report: &SixResourceDoctorReport) -> Value {
    json!({
        "schema": "personal-six-resource-doctor",
        "schema_version": report.schema_version,
        "surface": "personal-doctor-six-resource",
        "overall": report.overall.as_str(),
        "gate_claim": report.gate_claim,
        "profile_claim": report.profile_claim,
        "families": report.families.iter().map(|family| {
            json!({
                "family": family.family.as_str(),
                "status": family.status.as_str(),
                "error_code": family.error_code,
                "recovery_hint": family.recovery_hint,
                "facts": family.facts.iter().map(|fact| {
                    json!({
                        "key": fact.key,
                        "value": fact.value,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn validate_observation(
    observation: &SixResourceHealthObservation,
) -> Result<(), SixResourceDoctorError> {
    if let Some(code) = observation.error_code {
        if code.trim().is_empty() || !is_stable_error_code(code) {
            return Err(SixResourceDoctorError::InvalidErrorCode);
        }
        if text_is_contaminated(code) {
            return Err(SixResourceDoctorError::SecretContamination);
        }
    }
    if let Some(hint) = observation.recovery_hint {
        if text_is_contaminated(hint) {
            return Err(SixResourceDoctorError::SecretContamination);
        }
    }
    for fact in &observation.facts {
        if FORBIDDEN_CLAIM_KEYS.contains(&fact.key) {
            return Err(SixResourceDoctorError::AuthorityClaimRejected);
        }
        if text_is_contaminated(fact.key) || text_is_contaminated(&fact.value) {
            return Err(SixResourceDoctorError::SecretContamination);
        }
    }
    Ok(())
}

fn aggregate_overall(families: &[SixResourceHealthObservation]) -> SixResourceHealthStatus {
    if families
        .iter()
        .any(|family| family.status == SixResourceHealthStatus::Blocked)
    {
        return SixResourceHealthStatus::Blocked;
    }
    if families
        .iter()
        .any(|family| family.status == SixResourceHealthStatus::Degraded)
    {
        return SixResourceHealthStatus::Degraded;
    }
    if families
        .iter()
        .any(|family| family.status == SixResourceHealthStatus::NotConfigured)
    {
        return SixResourceHealthStatus::NotConfigured;
    }
    SixResourceHealthStatus::Ready
}

fn family_index(family: SixResourceFamily) -> usize {
    match family {
        SixResourceFamily::Memory => 0,
        SixResourceFamily::Skill => 1,
        SixResourceFamily::Tool => 2,
        SixResourceFamily::Context => 3,
        SixResourceFamily::Task => 4,
        SixResourceFamily::Runtime => 5,
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
    FORBIDDEN_DOCTOR_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn complete_observations() -> Vec<SixResourceHealthObservation> {
        SixResourceFamily::ALL
            .iter()
            .map(|family| SixResourceHealthObservation {
                family: *family,
                status: SixResourceHealthStatus::Ready,
                error_code: None,
                recovery_hint: Some("no action required"),
                facts: vec![SixResourceHealthFact {
                    key: "digest_bound",
                    value: "true".to_owned(),
                }],
            })
            .collect()
    }

    #[test]
    fn evaluates_complete_six_resource_doctor_report() {
        let report = evaluate_six_resource_doctor_health(&complete_observations())
            .expect("complete observations must succeed");
        assert_eq!(report.families.len(), 6);
        assert_eq!(report.overall, SixResourceHealthStatus::Ready);
        assert_eq!(report.gate_claim, "not-claimed");
        let json = six_resource_doctor_projection_json(&report);
        assert_eq!(json["surface"], "personal-doctor-six-resource");
        assert_eq!(json["profile_claim"], "not-claimed");
    }

    #[test]
    fn rejects_missing_and_duplicate_families() {
        let mut observations = complete_observations();
        observations.pop();
        assert_eq!(
            evaluate_six_resource_doctor_health(&observations).unwrap_err(),
            SixResourceDoctorError::MissingFamily("runtime")
        );

        observations = complete_observations();
        observations.push(observations[0].clone());
        assert_eq!(
            evaluate_six_resource_doctor_health(&observations).unwrap_err(),
            SixResourceDoctorError::DuplicateFamily("memory")
        );
    }

    #[test]
    fn rejects_secret_contamination_and_authority_claims() {
        let mut observations = complete_observations();
        observations[0].facts[0].value = "ssv1:not-a-real-secret".to_owned();
        assert_eq!(
            evaluate_six_resource_doctor_health(&observations).unwrap_err(),
            SixResourceDoctorError::SecretContamination
        );

        observations = complete_observations();
        observations[1].facts[0].key = "gate_pass";
        assert_eq!(
            evaluate_six_resource_doctor_health(&observations).unwrap_err(),
            SixResourceDoctorError::AuthorityClaimRejected
        );
    }

    #[test]
    fn aggregates_blocked_overall_and_requires_stable_error_codes() {
        let mut observations = complete_observations();
        observations[2].status = SixResourceHealthStatus::Blocked;
        observations[2].error_code = Some("TOOL_CATALOG_UNAVAILABLE");
        observations[2].recovery_hint = Some("rebuild daemon Tool catalog then retry doctor");
        let report = evaluate_six_resource_doctor_health(&observations).unwrap();
        assert_eq!(report.overall, SixResourceHealthStatus::Blocked);

        observations[2].error_code = Some("bad-code");
        assert_eq!(
            evaluate_six_resource_doctor_health(&observations).unwrap_err(),
            SixResourceDoctorError::InvalidErrorCode
        );
    }
}
