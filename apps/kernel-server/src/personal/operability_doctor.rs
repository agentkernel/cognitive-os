//! Operability doctor facts for sidecar/process/effect/migration (P7-T03/D03).
//!
//! Emits redacted diagnostic observations with stable error codes and recovery
//! hints. Rejects secret contamination and authority-shaped completion claims.
//! This module does not mutate Tasks, Effects, or SidecarSessions.

use serde_json::{Value, json};

const FORBIDDEN_MARKERS: &[&str] = &[
    "secret",
    "credential",
    "api_key",
    "api-key",
    "private_key",
    "ssv1:",
    "sk-",
    "bearer ",
];

const FORBIDDEN_CLAIM_KEYS: &[&str] = &[
    "gate_pass",
    "task_completed",
    "effect_succeeded_as_completion",
    "release_ready",
];

/// Operability doctor topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperabilityDoctorTopic {
    SidecarDrift,
    ProcessReconcile,
    EffectReconcile,
    Migration,
}

impl OperabilityDoctorTopic {
    pub const ALL: [Self; 4] = [
        Self::SidecarDrift,
        Self::ProcessReconcile,
        Self::EffectReconcile,
        Self::Migration,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SidecarDrift => "sidecar_drift",
            Self::ProcessReconcile => "process_reconcile",
            Self::EffectReconcile => "effect_reconcile",
            Self::Migration => "migration",
        }
    }
}

/// Coarse redacted status for one operability topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperabilityDoctorStatus {
    Ready,
    DriftDetected,
    ReconcileRequired,
    Blocked,
    NotConfigured,
}

impl OperabilityDoctorStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::DriftDetected => "drift_detected",
            Self::ReconcileRequired => "reconcile_required",
            Self::Blocked => "blocked",
            Self::NotConfigured => "not_configured",
        }
    }
}

/// Caller-supplied non-secret observation for one topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperabilityDoctorObservation {
    pub topic: OperabilityDoctorTopic,
    pub status: OperabilityDoctorStatus,
    pub error_code: Option<&'static str>,
    pub recovery_hint: Option<&'static str>,
    pub facts: Vec<(String, String)>,
}

/// Redacted operability doctor report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperabilityDoctorReport {
    pub topics: Vec<OperabilityDoctorObservation>,
    pub overall: OperabilityDoctorStatus,
    pub gate_claim: &'static str,
    pub profile_claim: &'static str,
}

/// Fail-closed operability doctor errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperabilityDoctorError {
    MissingTopic(&'static str),
    DuplicateTopic(&'static str),
    SecretContamination,
    AuthorityClaimRejected,
    InvalidErrorCode,
}

/// Evaluate redacted sidecar/process/effect/migration doctor facts.
pub fn evaluate_operability_doctor(
    observations: &[OperabilityDoctorObservation],
) -> Result<OperabilityDoctorReport, OperabilityDoctorError> {
    let mut seen = [false; 4];
    let mut topics = Vec::with_capacity(4);

    for observation in observations {
        let index = topic_index(observation.topic);
        if seen[index] {
            return Err(OperabilityDoctorError::DuplicateTopic(
                observation.topic.as_str(),
            ));
        }
        seen[index] = true;
        validate_observation(observation)?;
        topics.push(observation.clone());
    }

    for (index, present) in seen.iter().enumerate() {
        if !present {
            return Err(OperabilityDoctorError::MissingTopic(
                OperabilityDoctorTopic::ALL[index].as_str(),
            ));
        }
    }

    topics.sort_by_key(|observation| topic_index(observation.topic));
    Ok(OperabilityDoctorReport {
        overall: aggregate_overall(&topics),
        topics,
        gate_claim: "not-claimed",
        profile_claim: "not-claimed",
    })
}

/// JSON projection for doctor/support surfaces.
pub fn operability_doctor_projection_json(report: &OperabilityDoctorReport) -> Value {
    json!({
        "schema": "personal-operability-doctor",
        "surface": "personal-doctor-operability",
        "overall": report.overall.as_str(),
        "gate_claim": report.gate_claim,
        "profile_claim": report.profile_claim,
        "topics": report.topics.iter().map(|topic| {
            json!({
                "topic": topic.topic.as_str(),
                "status": topic.status.as_str(),
                "error_code": topic.error_code,
                "recovery_hint": topic.recovery_hint,
                "facts": topic.facts.iter().map(|(key, value)| {
                    json!({ "key": key, "value": value })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn validate_observation(
    observation: &OperabilityDoctorObservation,
) -> Result<(), OperabilityDoctorError> {
    if let Some(code) = observation.error_code {
        if !is_stable_error_code(code) {
            return Err(OperabilityDoctorError::InvalidErrorCode);
        }
        if text_is_contaminated(code) {
            return Err(OperabilityDoctorError::SecretContamination);
        }
    }
    if let Some(hint) = observation.recovery_hint
        && text_is_contaminated(hint)
    {
        return Err(OperabilityDoctorError::SecretContamination);
    }
    for (key, value) in &observation.facts {
        if FORBIDDEN_CLAIM_KEYS
            .iter()
            .any(|forbidden| *forbidden == key.as_str())
        {
            return Err(OperabilityDoctorError::AuthorityClaimRejected);
        }
        if text_is_contaminated(key) || text_is_contaminated(value) {
            return Err(OperabilityDoctorError::SecretContamination);
        }
    }
    Ok(())
}

fn aggregate_overall(topics: &[OperabilityDoctorObservation]) -> OperabilityDoctorStatus {
    if topics
        .iter()
        .any(|topic| topic.status == OperabilityDoctorStatus::Blocked)
    {
        return OperabilityDoctorStatus::Blocked;
    }
    if topics
        .iter()
        .any(|topic| topic.status == OperabilityDoctorStatus::ReconcileRequired)
    {
        return OperabilityDoctorStatus::ReconcileRequired;
    }
    if topics
        .iter()
        .any(|topic| topic.status == OperabilityDoctorStatus::DriftDetected)
    {
        return OperabilityDoctorStatus::DriftDetected;
    }
    if topics
        .iter()
        .any(|topic| topic.status == OperabilityDoctorStatus::NotConfigured)
    {
        return OperabilityDoctorStatus::NotConfigured;
    }
    OperabilityDoctorStatus::Ready
}

fn topic_index(topic: OperabilityDoctorTopic) -> usize {
    match topic {
        OperabilityDoctorTopic::SidecarDrift => 0,
        OperabilityDoctorTopic::ProcessReconcile => 1,
        OperabilityDoctorTopic::EffectReconcile => 2,
        OperabilityDoctorTopic::Migration => 3,
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
    FORBIDDEN_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn complete_observations() -> Vec<OperabilityDoctorObservation> {
        OperabilityDoctorTopic::ALL
            .iter()
            .map(|topic| OperabilityDoctorObservation {
                topic: *topic,
                status: OperabilityDoctorStatus::NotConfigured,
                error_code: Some("OPERABILITY_TOPIC_NOT_PROBED"),
                recovery_hint: Some("await supported operability probe"),
                facts: vec![("probe".to_owned(), "not_run".to_owned())],
            })
            .collect()
    }

    #[test]
    fn evaluates_complete_operability_doctor_report() {
        let report = evaluate_operability_doctor(&complete_observations()).expect("complete");
        assert_eq!(report.topics.len(), 4);
        assert_eq!(report.gate_claim, "not-claimed");
        let json = operability_doctor_projection_json(&report);
        assert_eq!(json["surface"], "personal-doctor-operability");
    }

    #[test]
    fn rejects_authority_claims_and_secret_contamination() {
        let mut observations = complete_observations();
        observations[0].facts = vec![("task_completed".to_owned(), "true".to_owned())];
        assert_eq!(
            evaluate_operability_doctor(&observations).unwrap_err(),
            OperabilityDoctorError::AuthorityClaimRejected
        );

        observations = complete_observations();
        observations[1].facts = vec![("note".to_owned(), "contains api_key material".to_owned())];
        assert_eq!(
            evaluate_operability_doctor(&observations).unwrap_err(),
            OperabilityDoctorError::SecretContamination
        );
    }

    #[test]
    fn aggregates_reconcile_required_and_rejects_missing_topics() {
        let mut observations = complete_observations();
        observations[2].status = OperabilityDoctorStatus::ReconcileRequired;
        observations[2].error_code = Some("EFFECT_OUTCOME_UNKNOWN");
        observations[2].recovery_hint = Some("reconcile Effect by original idempotency key");
        let report = evaluate_operability_doctor(&observations).unwrap();
        assert_eq!(report.overall, OperabilityDoctorStatus::ReconcileRequired);

        observations.pop();
        assert_eq!(
            evaluate_operability_doctor(&observations).unwrap_err(),
            OperabilityDoctorError::MissingTopic("migration")
        );
    }
}
