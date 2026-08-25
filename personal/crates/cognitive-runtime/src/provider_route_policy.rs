//! `L3` Provider-route denominator and outcome policy for P9-T04.
//!
//! ADR-0051 forbids three specific ways a Provider route report can lie: it may
//! not report a time to first token without streaming timestamps, it may not
//! turn a missing usage counter into a measured zero, and it may not quietly
//! retry a completion and report the retry as the sample. This module makes
//! each of those a rejection rather than a convention.

use crate::performance_campaign::ProviderUsage;
use serde::Serialize;

/// Every outcome a started Provider request can have. All of them stay in the
/// denominator; none of them may be discarded and re-run in its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderRouteOutcome {
    CompleteResponse,
    Timeout,
    RateLimited,
    UpstreamFailure,
    DeniedBeforeDispatch,
    OutcomeUnknown,
}

/// One started Provider request. Duration is the local wall time the daemon
/// observed; it is never split into a fabricated first-token component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderRouteSample {
    pub outcome: ProviderRouteOutcome,
    pub local_preflight_nanos: u128,
    pub provider_network_nanos: u128,
    pub usage: ProviderUsage,
}

/// Time to first token is only reportable when the transport actually produced
/// streaming timestamps. The current proxy is non-streaming, so this is
/// `NotStreaming` and the report carries no TTFT at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum FirstTokenTiming {
    NotStreaming,
    Streamed { first_token_nanos: u128 },
}

/// One `L3` scenario cell, for example `R1` Provider proxy marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderRouteObservation {
    pub claim_level: &'static str,
    pub scenario_id: String,
    pub selected_model: String,
    pub retry_budget: u32,
    pub started_requests: u64,
    pub samples: Vec<ProviderRouteSample>,
    pub first_token_timing: FirstTokenTiming,
    pub cost_available: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProviderRoutePolicyError {
    #[error("Provider route observations must remain hypothesis-only")]
    ClaimShapedObservation,
    #[error("a Provider completion must not be retried; the retry budget is zero")]
    RetryBudgetNotZero,
    #[error("every started Provider request must be retained as a classified sample")]
    IncompleteDenominator,
    #[error("a scenario cell must declare its scenario identity and selected model")]
    MissingScenarioIdentity,
    #[error("a complete response must record a positive Provider network duration")]
    ImplausibleCompleteResponse,
    #[error("a request that never reached the Provider must not report network time")]
    NetworkTimeWithoutDispatch,
    #[error("a failed or unreached request must not report measured Provider usage")]
    UsageWithoutCompleteResponse,
    #[error("time to first token requires real streaming timestamps")]
    FabricatedFirstTokenTiming,
    #[error("cost is only reportable against a preregistered pricing snapshot")]
    FabricatedCost,
}

const PROVIDER_ROUTE_CLAIM_LEVEL: &str = "hypothesis";

/// Assemble one scenario cell under the ADR-0051 Provider route rules.
pub fn build_provider_route_observation(
    scenario_id: impl Into<String>,
    selected_model: impl Into<String>,
    started_requests: u64,
    samples: Vec<ProviderRouteSample>,
    first_token_timing: FirstTokenTiming,
    cost_available: bool,
) -> Result<ProviderRouteObservation, ProviderRoutePolicyError> {
    let observation = ProviderRouteObservation {
        claim_level: PROVIDER_ROUTE_CLAIM_LEVEL,
        scenario_id: scenario_id.into(),
        selected_model: selected_model.into(),
        retry_budget: 0,
        started_requests,
        samples,
        first_token_timing,
        cost_available,
    };
    validate_provider_route_observation(&observation)?;
    Ok(observation)
}

/// Reject Provider route observations that fabricate a claim the transport
/// cannot support.
pub fn validate_provider_route_observation(
    observation: &ProviderRouteObservation,
) -> Result<(), ProviderRoutePolicyError> {
    if observation.claim_level != PROVIDER_ROUTE_CLAIM_LEVEL {
        return Err(ProviderRoutePolicyError::ClaimShapedObservation);
    }
    if observation.retry_budget != 0 {
        return Err(ProviderRoutePolicyError::RetryBudgetNotZero);
    }
    if observation.scenario_id.trim().is_empty() || observation.selected_model.trim().is_empty() {
        return Err(ProviderRoutePolicyError::MissingScenarioIdentity);
    }
    if u64::try_from(observation.samples.len()).unwrap_or(u64::MAX) != observation.started_requests
    {
        return Err(ProviderRoutePolicyError::IncompleteDenominator);
    }
    if matches!(
        observation.first_token_timing,
        FirstTokenTiming::Streamed { .. }
    ) {
        return Err(ProviderRoutePolicyError::FabricatedFirstTokenTiming);
    }
    if observation.cost_available {
        return Err(ProviderRoutePolicyError::FabricatedCost);
    }
    for sample in &observation.samples {
        let reached_provider = sample.outcome != ProviderRouteOutcome::DeniedBeforeDispatch;
        if !reached_provider && sample.provider_network_nanos != 0 {
            return Err(ProviderRoutePolicyError::NetworkTimeWithoutDispatch);
        }
        if sample.outcome == ProviderRouteOutcome::CompleteResponse
            && sample.provider_network_nanos == 0
        {
            return Err(ProviderRoutePolicyError::ImplausibleCompleteResponse);
        }
        if sample.outcome != ProviderRouteOutcome::CompleteResponse
            && sample.usage != ProviderUsage::NotAvailable
        {
            return Err(ProviderRoutePolicyError::UsageWithoutCompleteResponse);
        }
    }
    Ok(())
}

/// Count one outcome class without collapsing the denominator.
pub fn count_outcome(observation: &ProviderRouteObservation, outcome: ProviderRouteOutcome) -> u64 {
    u64::try_from(
        observation
            .samples
            .iter()
            .filter(|sample| sample.outcome == outcome)
            .count(),
    )
    .unwrap_or(u64::MAX)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn complete(network_nanos: u128, usage: ProviderUsage) -> ProviderRouteSample {
        ProviderRouteSample {
            outcome: ProviderRouteOutcome::CompleteResponse,
            local_preflight_nanos: 1_000,
            provider_network_nanos: network_nanos,
            usage,
        }
    }

    fn failed(outcome: ProviderRouteOutcome, network_nanos: u128) -> ProviderRouteSample {
        ProviderRouteSample {
            outcome,
            local_preflight_nanos: 900,
            provider_network_nanos: network_nanos,
            usage: ProviderUsage::NotAvailable,
        }
    }

    fn build(
        started: u64,
        samples: Vec<ProviderRouteSample>,
    ) -> Result<ProviderRouteObservation, ProviderRoutePolicyError> {
        build_provider_route_observation(
            "R1-provider-proxy-marker",
            "deepseek-v4-flash",
            started,
            samples,
            FirstTokenTiming::NotStreaming,
            false,
        )
    }

    #[test]
    fn every_started_request_stays_classified_in_the_denominator() {
        let observation = build(
            4,
            vec![
                complete(12_000_000, ProviderUsage::NotAvailable),
                failed(ProviderRouteOutcome::Timeout, 60_000_000_000),
                failed(ProviderRouteOutcome::RateLimited, 8_000_000),
                failed(ProviderRouteOutcome::OutcomeUnknown, 30_000_000),
            ],
        )
        .expect("publishable observation");
        assert_eq!(observation.retry_budget, 0);
        assert_eq!(
            count_outcome(&observation, ProviderRouteOutcome::CompleteResponse),
            1
        );
        assert_eq!(
            count_outcome(&observation, ProviderRouteOutcome::Timeout),
            1
        );
        assert_eq!(
            count_outcome(&observation, ProviderRouteOutcome::OutcomeUnknown),
            1
        );
    }

    #[test]
    fn a_discarded_failure_is_not_reportable() {
        assert_eq!(
            build(3, vec![complete(9_000_000, ProviderUsage::NotAvailable)]).unwrap_err(),
            ProviderRoutePolicyError::IncompleteDenominator
        );
    }

    #[test]
    fn time_to_first_token_requires_real_streaming_timestamps() {
        let error = build_provider_route_observation(
            "R1-provider-proxy-marker",
            "deepseek-v4-flash",
            1,
            vec![complete(9_000_000, ProviderUsage::NotAvailable)],
            FirstTokenTiming::Streamed {
                first_token_nanos: 3_000_000,
            },
            false,
        )
        .unwrap_err();
        assert_eq!(error, ProviderRoutePolicyError::FabricatedFirstTokenTiming);
    }

    #[test]
    fn cost_is_not_reportable_without_a_pricing_snapshot() {
        let error = build_provider_route_observation(
            "R1-provider-proxy-marker",
            "deepseek-v4-flash",
            1,
            vec![complete(9_000_000, ProviderUsage::NotAvailable)],
            FirstTokenTiming::NotStreaming,
            true,
        )
        .unwrap_err();
        assert_eq!(error, ProviderRoutePolicyError::FabricatedCost);
    }

    #[test]
    fn a_request_denied_before_dispatch_cannot_report_network_time() {
        assert_eq!(
            build(
                1,
                vec![failed(ProviderRouteOutcome::DeniedBeforeDispatch, 5_000)]
            )
            .unwrap_err(),
            ProviderRoutePolicyError::NetworkTimeWithoutDispatch
        );
        build(
            1,
            vec![failed(ProviderRouteOutcome::DeniedBeforeDispatch, 0)],
        )
        .expect("a fail-closed denial is a valid retained sample");
    }

    #[test]
    fn a_complete_response_without_network_time_is_not_plausible() {
        assert_eq!(
            build(1, vec![complete(0, ProviderUsage::NotAvailable)]).unwrap_err(),
            ProviderRoutePolicyError::ImplausibleCompleteResponse
        );
    }

    #[test]
    fn a_failed_request_cannot_carry_measured_usage() {
        let measured = ProviderUsage::Measured {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        let sample = ProviderRouteSample {
            outcome: ProviderRouteOutcome::Timeout,
            local_preflight_nanos: 100,
            provider_network_nanos: 60_000_000_000,
            usage: measured,
        };
        assert_eq!(
            build(1, vec![sample]).unwrap_err(),
            ProviderRoutePolicyError::UsageWithoutCompleteResponse
        );
    }

    #[test]
    fn a_retried_completion_and_a_self_promoted_claim_fail_closed() {
        let mut observation = build(1, vec![complete(7_000_000, ProviderUsage::NotAvailable)])
            .expect("publishable observation");
        observation.retry_budget = 1;
        assert_eq!(
            validate_provider_route_observation(&observation).unwrap_err(),
            ProviderRoutePolicyError::RetryBudgetNotZero
        );
        observation.retry_budget = 0;
        observation.claim_level = "tested-local";
        assert_eq!(
            validate_provider_route_observation(&observation).unwrap_err(),
            ProviderRoutePolicyError::ClaimShapedObservation
        );
    }

    #[test]
    fn a_scenario_without_identity_or_model_fails_closed() {
        assert_eq!(
            build_provider_route_observation(
                "  ",
                "deepseek-v4-flash",
                1,
                vec![complete(7_000_000, ProviderUsage::NotAvailable)],
                FirstTokenTiming::NotStreaming,
                false,
            )
            .unwrap_err(),
            ProviderRoutePolicyError::MissingScenarioIdentity
        );
        assert_eq!(
            build_provider_route_observation(
                "R1-provider-proxy-marker",
                "",
                1,
                vec![complete(7_000_000, ProviderUsage::NotAvailable)],
                FirstTokenTiming::NotStreaming,
                false,
            )
            .unwrap_err(),
            ProviderRoutePolicyError::MissingScenarioIdentity
        );
    }
}
