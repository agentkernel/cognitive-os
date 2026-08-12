//! Transport-only stage decomposition for the loopback daemon front door.
//!
//! `P9-T01` established that `effect_persistence` aggregates SQLite open,
//! admission, persist and reload, so it must never be read as HTTP, watch or
//! sidecar transport cost. This collector measures the transport window
//! separately and refuses to publish an observation that does not disclaim
//! those other attributions.
//!
//! The recorder is thread-local because the personal daemon serves one
//! connection per thread. It stores durations and byte counts only: no request
//! line, header, bearer, or body ever enters an observation.

use serde::Serialize;
use std::cell::RefCell;
use std::time::Instant;

/// Attributions this collector explicitly does not measure. A transport
/// observation is only publishable while it carries all of them.
pub const EXCLUDED_TRANSPORT_ATTRIBUTIONS: [&str; 4] = [
    "effect_persistence",
    "provider_network",
    "pi_process_launch",
    "scheduler_wait",
];

const TRANSPORT_CLAIM_LEVEL: &str = "hypothesis";

/// Disjoint sub-windows of one loopback connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopbackTransportStage {
    /// Read-timeout configuration plus connection and in-flight accounting.
    ConnectionAdmission,
    /// Bounded request-line, header and body read.
    RequestRead,
    /// Front-door header checks performed before any route runs.
    HeaderAdmission,
    /// Route match and handler execution, with response writes subtracted.
    RouteDispatch,
    /// Accumulated socket write time for the response.
    ResponseWrite,
}

/// One measured or explicitly omitted transport stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LoopbackTransportSample {
    pub stage: LoopbackTransportStage,
    pub duration_nanos: u128,
    pub omitted: bool,
}

/// A redacted transport observation for one loopback connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoopbackTransportObservation {
    pub claim_level: &'static str,
    pub connection_elapsed_nanos: u128,
    pub stages: Vec<LoopbackTransportSample>,
    pub request_bytes: u64,
    pub response_bytes: u64,
    pub excluded_attributions: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LoopbackTransportError {
    #[error("loopback transport observations must remain hypothesis-only")]
    ClaimShapedObservation,
    #[error("an omitted transport stage must not carry a measured duration")]
    OmittedStageWithDuration,
    #[error("transport observations must disclaim the attributions they do not measure")]
    MissingExcludedAttribution,
    #[error("transport stages must be disjoint windows within the measured connection")]
    StageDurationExceedsConnection,
    #[error("a transport stage must not be recorded more than once per connection")]
    DuplicateStage,
}

#[derive(Debug, Default)]
struct TransportRecorder {
    connection_started_at: Option<Instant>,
    samples: Vec<LoopbackTransportSample>,
    request_bytes: u64,
    response_bytes: u64,
    write_nanos: u128,
    last_observation: Option<LoopbackTransportObservation>,
}

thread_local! {
    static RECORDER: RefCell<TransportRecorder> = RefCell::new(TransportRecorder::default());
}

/// Start a new connection window, discarding any partial previous state.
pub fn begin_connection() {
    RECORDER.with_borrow_mut(|recorder| {
        *recorder = TransportRecorder {
            connection_started_at: Some(Instant::now()),
            ..TransportRecorder::default()
        };
    });
}

/// Record one measured transport stage. Calls outside a connection window are
/// dropped so instrumentation can never invent a stage of its own.
pub fn record_stage(stage: LoopbackTransportStage, duration_nanos: u128) {
    RECORDER.with_borrow_mut(|recorder| {
        if recorder.connection_started_at.is_none() {
            return;
        }
        recorder.samples.push(LoopbackTransportSample {
            stage,
            duration_nanos,
            omitted: false,
        });
    });
}

/// Record a stage the daemon did not measure on this path.
pub fn record_omitted_stage(stage: LoopbackTransportStage) {
    RECORDER.with_borrow_mut(|recorder| {
        if recorder.connection_started_at.is_none() {
            return;
        }
        recorder.samples.push(LoopbackTransportSample {
            stage,
            duration_nanos: 0,
            omitted: true,
        });
    });
}

/// Accumulate one socket write. Writes happen inside route handlers, so they
/// are collected separately and subtracted from the dispatch window.
pub fn add_response_write(duration_nanos: u128, bytes: u64) {
    RECORDER.with_borrow_mut(|recorder| {
        if recorder.connection_started_at.is_none() {
            return;
        }
        recorder.write_nanos = recorder.write_nanos.saturating_add(duration_nanos);
        recorder.response_bytes = recorder.response_bytes.saturating_add(bytes);
    });
}

/// Record the bounded request size actually read from the socket.
pub fn add_request_bytes(bytes: u64) {
    RECORDER.with_borrow_mut(|recorder| {
        if recorder.connection_started_at.is_none() {
            return;
        }
        recorder.request_bytes = recorder.request_bytes.saturating_add(bytes);
    });
}

/// Subtract accumulated write time from a dispatch window so response writes
/// are attributed to `ResponseWrite` instead of route work.
pub fn record_route_dispatch(elapsed_nanos: u128) {
    let write_nanos = RECORDER.with_borrow(|recorder| recorder.write_nanos);
    record_stage(
        LoopbackTransportStage::RouteDispatch,
        elapsed_nanos.saturating_sub(write_nanos),
    );
    record_stage(LoopbackTransportStage::ResponseWrite, write_nanos);
}

/// Close the connection window and retain the validated observation.
pub fn finish_connection() -> Option<LoopbackTransportObservation> {
    let observation = RECORDER.with_borrow_mut(|recorder| {
        let started_at = recorder.connection_started_at.take()?;
        let mut samples = std::mem::take(&mut recorder.samples);
        samples.sort_by_key(|sample| sample.stage);
        Some(LoopbackTransportObservation {
            claim_level: TRANSPORT_CLAIM_LEVEL,
            connection_elapsed_nanos: started_at.elapsed().as_nanos(),
            stages: samples,
            request_bytes: recorder.request_bytes,
            response_bytes: recorder.response_bytes,
            excluded_attributions: EXCLUDED_TRANSPORT_ATTRIBUTIONS.to_vec(),
        })
    })?;
    if validate_loopback_transport_observation(&observation).is_err() {
        return None;
    }
    RECORDER.with_borrow_mut(|recorder| {
        recorder.last_observation = Some(observation.clone());
    });
    Some(observation)
}

/// The most recent validated observation recorded on this thread.
pub fn last_observation() -> Option<LoopbackTransportObservation> {
    RECORDER.with_borrow(|recorder| recorder.last_observation.clone())
}

/// Reject observations that fabricate stages, promote themselves above a
/// hypothesis, or drop the attributions they are not allowed to absorb.
pub fn validate_loopback_transport_observation(
    observation: &LoopbackTransportObservation,
) -> Result<(), LoopbackTransportError> {
    if observation.claim_level != TRANSPORT_CLAIM_LEVEL {
        return Err(LoopbackTransportError::ClaimShapedObservation);
    }
    for excluded in EXCLUDED_TRANSPORT_ATTRIBUTIONS {
        if !observation.excluded_attributions.contains(&excluded) {
            return Err(LoopbackTransportError::MissingExcludedAttribution);
        }
    }
    let mut measured_total = 0_u128;
    for (index, sample) in observation.stages.iter().enumerate() {
        if sample.omitted && sample.duration_nanos != 0 {
            return Err(LoopbackTransportError::OmittedStageWithDuration);
        }
        if observation.stages[..index]
            .iter()
            .any(|earlier| earlier.stage == sample.stage)
        {
            return Err(LoopbackTransportError::DuplicateStage);
        }
        measured_total = measured_total.saturating_add(sample.duration_nanos);
    }
    if measured_total > observation.connection_elapsed_nanos {
        return Err(LoopbackTransportError::StageDurationExceedsConnection);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn observation_with(
        stages: Vec<LoopbackTransportSample>,
        connection_elapsed_nanos: u128,
    ) -> LoopbackTransportObservation {
        LoopbackTransportObservation {
            claim_level: TRANSPORT_CLAIM_LEVEL,
            connection_elapsed_nanos,
            stages,
            request_bytes: 128,
            response_bytes: 256,
            excluded_attributions: EXCLUDED_TRANSPORT_ATTRIBUTIONS.to_vec(),
        }
    }

    fn measured(stage: LoopbackTransportStage, duration_nanos: u128) -> LoopbackTransportSample {
        LoopbackTransportSample {
            stage,
            duration_nanos,
            omitted: false,
        }
    }

    #[test]
    fn recorded_connection_separates_write_time_from_route_dispatch() {
        begin_connection();
        record_stage(LoopbackTransportStage::ConnectionAdmission, 100);
        record_stage(LoopbackTransportStage::RequestRead, 200);
        record_stage(LoopbackTransportStage::HeaderAdmission, 50);
        add_request_bytes(512);
        add_response_write(400, 1_024);
        record_route_dispatch(1_000);
        let observation = finish_connection().unwrap();
        validate_loopback_transport_observation(&observation).unwrap();
        let dispatch = observation
            .stages
            .iter()
            .find(|sample| sample.stage == LoopbackTransportStage::RouteDispatch)
            .unwrap();
        let write = observation
            .stages
            .iter()
            .find(|sample| sample.stage == LoopbackTransportStage::ResponseWrite)
            .unwrap();
        assert_eq!(dispatch.duration_nanos, 600);
        assert_eq!(write.duration_nanos, 400);
        assert_eq!(observation.request_bytes, 512);
        assert_eq!(observation.response_bytes, 1_024);
        assert_eq!(last_observation().unwrap(), observation);
    }

    #[test]
    fn stages_recorded_outside_a_connection_window_are_not_observations() {
        begin_connection();
        finish_connection().unwrap();
        record_stage(LoopbackTransportStage::RequestRead, 9_999);
        assert!(finish_connection().is_none());
    }

    #[test]
    fn transport_observation_carries_no_request_content() {
        begin_connection();
        record_stage(LoopbackTransportStage::RequestRead, 10);
        add_request_bytes(64);
        record_route_dispatch(20);
        let observation = finish_connection().unwrap();
        let serialized = serde_json::to_string(&observation).unwrap();
        for forbidden in ["Authorization", "Bearer", "Host", "prompt", "body"] {
            assert!(
                !serialized.contains(forbidden),
                "transport observation must not carry {forbidden}"
            );
        }
    }

    #[test]
    fn observation_absorbing_effect_persistence_is_not_publishable() {
        let mut observation = observation_with(
            vec![measured(LoopbackTransportStage::RouteDispatch, 10)],
            1_000,
        );
        observation
            .excluded_attributions
            .retain(|attribution| *attribution != "effect_persistence");
        assert_eq!(
            validate_loopback_transport_observation(&observation).unwrap_err(),
            LoopbackTransportError::MissingExcludedAttribution
        );
    }

    #[test]
    fn stage_longer_than_its_connection_is_not_publishable() {
        let observation = observation_with(
            vec![measured(LoopbackTransportStage::RouteDispatch, 5_000)],
            1_000,
        );
        assert_eq!(
            validate_loopback_transport_observation(&observation).unwrap_err(),
            LoopbackTransportError::StageDurationExceedsConnection
        );
    }

    #[test]
    fn duplicated_or_self_promoted_stages_are_not_publishable() {
        let duplicated = observation_with(
            vec![
                measured(LoopbackTransportStage::RequestRead, 10),
                measured(LoopbackTransportStage::RequestRead, 10),
            ],
            1_000,
        );
        assert_eq!(
            validate_loopback_transport_observation(&duplicated).unwrap_err(),
            LoopbackTransportError::DuplicateStage
        );
        let mut promoted = observation_with(
            vec![measured(LoopbackTransportStage::RequestRead, 10)],
            1_000,
        );
        promoted.claim_level = "tested-local";
        assert_eq!(
            validate_loopback_transport_observation(&promoted).unwrap_err(),
            LoopbackTransportError::ClaimShapedObservation
        );
    }

    #[test]
    fn omitted_stage_must_stay_zero_duration() {
        begin_connection();
        record_omitted_stage(LoopbackTransportStage::ResponseWrite);
        let mut observation = finish_connection().unwrap();
        assert!(
            observation
                .stages
                .iter()
                .any(|sample| sample.omitted && sample.duration_nanos == 0)
        );
        observation.stages[0].duration_nanos = 7;
        assert_eq!(
            validate_loopback_transport_observation(&observation).unwrap_err(),
            LoopbackTransportError::OmittedStageWithDuration
        );
    }
}
