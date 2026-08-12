//! Measurement-only L0/L1 campaign runner admission for P9-T04.
//!
//! The runner accepts only registered measurement arguments, refuses
//! secret-shaped process input, and assembles a redacted hypothesis-level
//! report around the [`crate::performance_campaign`] envelope. It never opens
//! an authority store, never reaches a Provider, and never echoes an argument
//! value it rejected.

use crate::performance_campaign::{
    CampaignCorrelationId, CampaignMeasurementEnvelope, CampaignStage, CampaignStageTiming,
    PerformanceCampaignError, ProviderUsage,
};
use serde::Serialize;
use serde_json::Value;

/// The single preregistered campaign identity from ADR-0051.
pub const CAMPAIGN_ID: &str = "P9-T04-comprehensive-performance-001";
pub const CAMPAIGN_RUNNER_REPORT_KIND: &str = "p9-t04-l0-l1-campaign/0.1";
pub const CAMPAIGN_RUNNER_CLAIM_LEVEL: &str = "hypothesis";

const SOURCE_REVISION_ARGUMENT: &str = "--source-revision";
const CORRELATION_ID_ARGUMENT: &str = "--correlation-id";
const SAMPLES_ARGUMENT: &str = "--samples";
const DEFAULT_SAMPLE_COUNT: usize = 25;
const MAXIMUM_SAMPLE_COUNT: usize = 100_000;

/// Environment families that may legitimately carry Provider or daemon
/// credentials. The runner refuses to start while any of them is visible so a
/// campaign process cannot inherit secret material it must never observe.
const CREDENTIAL_BEARING_ENVIRONMENT_PREFIXES: [&str; 5] = [
    "COGNITIVEOS_",
    "COGNITIVE_",
    "DEEPSEEK",
    "OPENAI",
    "PROVIDER_",
];
const SECRET_SHAPED_ENVIRONMENT_MARKERS: [&str; 7] = [
    "KEY",
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSPHRASE",
    "CREDENTIAL",
    "BEARER",
];

/// Report keys that would turn redacted measurement evidence into Provider
/// content, credential material, or an authority record.
const FORBIDDEN_OBSERVATION_KEYS: [&str; 12] = [
    "api_key",
    "authorization",
    "bearer",
    "content",
    "credential",
    "messages",
    "password",
    "prompt",
    "response",
    "secret",
    "secret_ref",
    "session_token",
];

/// A validated measurement-only campaign invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CampaignRunRequest {
    pub source_revision: String,
    pub correlation_id: CampaignCorrelationId,
    pub sample_count: usize,
}

/// L0 records whether measurement may begin at all. Every field is a decision
/// fact; none of them is an authority outcome or a benefit claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct L0EligibilityFacts {
    pub source_revision_pinned: bool,
    pub correlation_id_opaque: bool,
    pub secret_shaped_input_present: bool,
    pub provider_enabled: bool,
    pub provider_retry_budget: u32,
    pub authority_writer: bool,
}

/// The redacted L0/L1 runner report. It carries identifiers, digests,
/// durations, counts, and registered error classes only.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CampaignRunnerReport {
    pub report_kind: &'static str,
    pub claim_level: &'static str,
    pub campaign_id: &'static str,
    pub layers: Vec<&'static str>,
    pub request: CampaignRunRequest,
    pub l0: L0EligibilityFacts,
    pub l1_module_observation: Value,
    pub envelope: CampaignMeasurementEnvelope,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CampaignRunnerError {
    #[error("campaign runner accepts only registered measurement-only arguments")]
    UnregisteredArgument,
    #[error("a required campaign runner argument is missing")]
    MissingArgument,
    #[error("a campaign runner argument value is missing")]
    MissingArgumentValue,
    #[error("campaign runner arguments must not be declared more than once")]
    DuplicateArgument,
    #[error("campaign sample count must be a positive integer within the registered bound")]
    InvalidSampleCount,
    #[error("secret-shaped campaign environment input is not accepted")]
    SecretShapedEnvironmentInput,
    #[error("campaign observations must not carry Provider content or credential material")]
    UnredactedObservation,
    #[error("campaign observations must remain hypothesis-level and non-authoritative")]
    ClaimShapedObservation,
    #[error(transparent)]
    Envelope(#[from] PerformanceCampaignError),
}

/// Admit one campaign invocation from process arguments and the names of the
/// visible environment variables. Values are deliberately not accepted: the
/// runner has no reason to read them and must not be able to log one.
pub fn parse_campaign_run_request<Arguments, EnvironmentNames>(
    arguments: Arguments,
    environment_names: EnvironmentNames,
) -> Result<CampaignRunRequest, CampaignRunnerError>
where
    Arguments: IntoIterator<Item = String>,
    EnvironmentNames: IntoIterator<Item = String>,
{
    if environment_names
        .into_iter()
        .any(|name| is_secret_shaped_environment_name(&name))
    {
        return Err(CampaignRunnerError::SecretShapedEnvironmentInput);
    }

    let mut source_revision: Option<String> = None;
    let mut correlation_id: Option<String> = None;
    let mut sample_count: Option<String> = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let slot = match argument.as_str() {
            SOURCE_REVISION_ARGUMENT => &mut source_revision,
            CORRELATION_ID_ARGUMENT => &mut correlation_id,
            SAMPLES_ARGUMENT => &mut sample_count,
            _ => return Err(CampaignRunnerError::UnregisteredArgument),
        };
        if slot.is_some() {
            return Err(CampaignRunnerError::DuplicateArgument);
        }
        *slot = Some(
            arguments
                .next()
                .ok_or(CampaignRunnerError::MissingArgumentValue)?,
        );
    }

    let source_revision = source_revision.ok_or(CampaignRunnerError::MissingArgument)?;
    let correlation_id = correlation_id.ok_or(CampaignRunnerError::MissingArgument)?;
    let sample_count = match sample_count {
        None => DEFAULT_SAMPLE_COUNT,
        Some(declared) => declared
            .parse::<usize>()
            .ok()
            .filter(|count| (1..=MAXIMUM_SAMPLE_COUNT).contains(count))
            .ok_or(CampaignRunnerError::InvalidSampleCount)?,
    };

    Ok(CampaignRunRequest {
        source_revision: validated_source_revision(source_revision)?,
        correlation_id: CampaignCorrelationId::parse(correlation_id)?,
        sample_count,
    })
}

/// Assemble the redacted report once L0 admission and the L1 module
/// observation have completed.
pub fn build_campaign_runner_report(
    request: CampaignRunRequest,
    l0_elapsed_nanos: u128,
    l1_elapsed_nanos: u128,
    l1_module_observation: Value,
) -> Result<CampaignRunnerReport, CampaignRunnerError> {
    reject_unredacted_observation(&l1_module_observation)?;
    reject_claim_shaped_observation(&l1_module_observation)?;
    let envelope = CampaignMeasurementEnvelope::new(
        CAMPAIGN_ID,
        request.correlation_id.clone(),
        request.source_revision.clone(),
        vec![
            CampaignStageTiming {
                stage: CampaignStage::SessionMint,
                elapsed_nanos: l0_elapsed_nanos,
            },
            CampaignStageTiming {
                stage: CampaignStage::Context,
                elapsed_nanos: l1_elapsed_nanos,
            },
        ],
        ProviderUsage::NotAvailable,
    )?;
    Ok(CampaignRunnerReport {
        report_kind: CAMPAIGN_RUNNER_REPORT_KIND,
        claim_level: CAMPAIGN_RUNNER_CLAIM_LEVEL,
        campaign_id: CAMPAIGN_ID,
        layers: vec!["L0", "L1"],
        l0: L0EligibilityFacts {
            source_revision_pinned: true,
            correlation_id_opaque: true,
            secret_shaped_input_present: false,
            provider_enabled: false,
            provider_retry_budget: 0,
            authority_writer: false,
        },
        request,
        l1_module_observation,
        envelope,
    })
}

fn validated_source_revision(value: String) -> Result<String, CampaignRunnerError> {
    if value.len() == 40 && value.bytes().all(|character| character.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(CampaignRunnerError::Envelope(
            PerformanceCampaignError::InvalidSourceRevision,
        ))
    }
}

fn is_secret_shaped_environment_name(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    CREDENTIAL_BEARING_ENVIRONMENT_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
        && SECRET_SHAPED_ENVIRONMENT_MARKERS
            .iter()
            .any(|marker| name.contains(marker))
}

fn reject_unredacted_observation(observation: &Value) -> Result<(), CampaignRunnerError> {
    match observation {
        Value::Object(members) => {
            for (key, value) in members {
                if FORBIDDEN_OBSERVATION_KEYS.contains(&key.to_ascii_lowercase().as_str()) {
                    return Err(CampaignRunnerError::UnredactedObservation);
                }
                reject_unredacted_observation(value)?;
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(reject_unredacted_observation),
        _ => Ok(()),
    }
}

fn reject_claim_shaped_observation(observation: &Value) -> Result<(), CampaignRunnerError> {
    let declared_claim_level = observation.get("claim_level").and_then(Value::as_str);
    if declared_claim_level.is_some_and(|level| level != CAMPAIGN_RUNNER_CLAIM_LEVEL) {
        return Err(CampaignRunnerError::ClaimShapedObservation);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    const VALID_REVISION: &str = "9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c";
    const VALID_CORRELATION_ID: &str = "campaign-0123456789abcdef0123456789abcdef";

    fn registered_arguments() -> Vec<String> {
        vec![
            SOURCE_REVISION_ARGUMENT.to_owned(),
            VALID_REVISION.to_owned(),
            CORRELATION_ID_ARGUMENT.to_owned(),
            VALID_CORRELATION_ID.to_owned(),
        ]
    }

    fn module_observation() -> Value {
        json!({
            "report_kind": "p7-t04-d01-module-observation/0.1",
            "claim_level": "hypothesis",
            "benchmarks": [{"benchmark_id": "context-cache-full-key-hit", "p50": 12_u64}],
        })
    }

    #[test]
    fn registered_arguments_admit_the_default_sample_count() {
        let request =
            parse_campaign_run_request(registered_arguments(), Vec::<String>::new()).unwrap();
        assert_eq!(request.source_revision, VALID_REVISION);
        assert_eq!(request.correlation_id.as_str(), VALID_CORRELATION_ID);
        assert_eq!(request.sample_count, DEFAULT_SAMPLE_COUNT);
    }

    #[test]
    fn secret_shaped_environment_input_blocks_measurement_before_any_argument_parse() {
        let error =
            parse_campaign_run_request(Vec::<String>::new(), vec!["DEEPSEEK_API_KEY".to_owned()])
                .unwrap_err();
        assert_eq!(error, CampaignRunnerError::SecretShapedEnvironmentInput);
        assert_eq!(
            parse_campaign_run_request(
                registered_arguments(),
                vec!["COGNITIVEOS_PROVIDER_TOKEN".to_owned()],
            )
            .unwrap_err(),
            CampaignRunnerError::SecretShapedEnvironmentInput
        );
    }

    #[test]
    fn ordinary_environment_names_remain_eligible() {
        let request = parse_campaign_run_request(
            registered_arguments(),
            vec![
                "PATH".to_owned(),
                "COGNITIVEOS_BENCHMARK_SAMPLES".to_owned(),
            ],
        )
        .unwrap();
        assert_eq!(request.sample_count, DEFAULT_SAMPLE_COUNT);
    }

    #[test]
    fn unregistered_argument_is_rejected_without_echoing_its_value() {
        let mut arguments = registered_arguments();
        arguments.extend([
            "--api-key".to_owned(),
            "sk-live-not-a-real-value".to_owned(),
        ]);
        let error = parse_campaign_run_request(arguments, Vec::<String>::new()).unwrap_err();
        assert_eq!(error, CampaignRunnerError::UnregisteredArgument);
        assert!(!error.to_string().contains("sk-live"));
    }

    #[test]
    fn missing_or_duplicated_registered_arguments_fail_closed() {
        assert_eq!(
            parse_campaign_run_request(
                vec![
                    SOURCE_REVISION_ARGUMENT.to_owned(),
                    VALID_REVISION.to_owned()
                ],
                Vec::<String>::new(),
            )
            .unwrap_err(),
            CampaignRunnerError::MissingArgument
        );
        assert_eq!(
            parse_campaign_run_request(
                vec![CORRELATION_ID_ARGUMENT.to_owned()],
                Vec::<String>::new()
            )
            .unwrap_err(),
            CampaignRunnerError::MissingArgumentValue
        );
        let mut duplicated = registered_arguments();
        duplicated.extend([
            SOURCE_REVISION_ARGUMENT.to_owned(),
            VALID_REVISION.to_owned(),
        ]);
        assert_eq!(
            parse_campaign_run_request(duplicated, Vec::<String>::new()).unwrap_err(),
            CampaignRunnerError::DuplicateArgument
        );
    }

    #[test]
    fn non_positive_or_unbounded_sample_counts_fail_closed() {
        for declared in ["0", "-1", "abc", "100001"] {
            let mut arguments = registered_arguments();
            arguments.extend([SAMPLES_ARGUMENT.to_owned(), declared.to_owned()]);
            assert_eq!(
                parse_campaign_run_request(arguments, Vec::<String>::new()).unwrap_err(),
                CampaignRunnerError::InvalidSampleCount,
                "sample count {declared} must not be admitted"
            );
        }
    }

    #[test]
    fn abbreviated_or_non_hexadecimal_revisions_fail_closed() {
        let error = parse_campaign_run_request(
            vec![
                SOURCE_REVISION_ARGUMENT.to_owned(),
                "9fbd390".to_owned(),
                CORRELATION_ID_ARGUMENT.to_owned(),
                VALID_CORRELATION_ID.to_owned(),
            ],
            Vec::<String>::new(),
        )
        .unwrap_err();
        assert_eq!(
            error,
            CampaignRunnerError::Envelope(PerformanceCampaignError::InvalidSourceRevision)
        );
    }

    #[test]
    fn provider_shaped_correlation_input_fails_closed() {
        let error = parse_campaign_run_request(
            vec![
                SOURCE_REVISION_ARGUMENT.to_owned(),
                VALID_REVISION.to_owned(),
                CORRELATION_ID_ARGUMENT.to_owned(),
                "Bearer task-session-value".to_owned(),
            ],
            Vec::<String>::new(),
        )
        .unwrap_err();
        assert_eq!(
            error,
            CampaignRunnerError::Envelope(PerformanceCampaignError::InvalidCorrelationId)
        );
    }

    #[test]
    fn report_records_non_provider_non_authority_l0_facts() {
        let request =
            parse_campaign_run_request(registered_arguments(), Vec::<String>::new()).unwrap();
        let report =
            build_campaign_runner_report(request, 4_096, 8_192, module_observation()).unwrap();
        assert_eq!(report.claim_level, CAMPAIGN_RUNNER_CLAIM_LEVEL);
        assert!(!report.l0.provider_enabled);
        assert!(!report.l0.authority_writer);
        assert_eq!(report.l0.provider_retry_budget, 0);
        assert_eq!(report.envelope.provider_usage, ProviderUsage::NotAvailable);
        let serialized = serde_json::to_string(&report).unwrap();
        assert!(!serialized.contains("prompt"));
    }

    #[test]
    fn observation_carrying_provider_content_is_not_reportable() {
        let request =
            parse_campaign_run_request(registered_arguments(), Vec::<String>::new()).unwrap();
        let contaminated = json!({
            "claim_level": "hypothesis",
            "benchmarks": [{"benchmark_id": "context-cache-full-key-hit", "prompt": "user text"}],
        });
        assert_eq!(
            build_campaign_runner_report(request.clone(), 1, 1, contaminated).unwrap_err(),
            CampaignRunnerError::UnredactedObservation
        );
        let credentialed = json!({"claim_level": "hypothesis", "authorization": "Bearer value"});
        assert_eq!(
            build_campaign_runner_report(request, 1, 1, credentialed).unwrap_err(),
            CampaignRunnerError::UnredactedObservation
        );
    }

    #[test]
    fn observation_promoting_itself_above_hypothesis_is_not_reportable() {
        let request =
            parse_campaign_run_request(registered_arguments(), Vec::<String>::new()).unwrap();
        let promoted = json!({"claim_level": "release", "benchmarks": []});
        assert_eq!(
            build_campaign_runner_report(request, 1, 1, promoted).unwrap_err(),
            CampaignRunnerError::ClaimShapedObservation
        );
    }

    #[test]
    fn zero_duration_layers_cannot_be_reported_as_measured() {
        let request =
            parse_campaign_run_request(registered_arguments(), Vec::<String>::new()).unwrap();
        assert_eq!(
            build_campaign_runner_report(request, 0, 8_192, module_observation()).unwrap_err(),
            CampaignRunnerError::Envelope(PerformanceCampaignError::InvalidTiming)
        );
    }
}
