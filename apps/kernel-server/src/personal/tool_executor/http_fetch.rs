#![allow(unused, unused_imports)]

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{
    NativeOperationFamily, NativeToolDescriptor, ToolRisk, validate_read_only_http_fetch,
};
use cognitive_kernel::{
    authz::AuthorizationGrant,
    effects::{EffectError, EffectProtocol, GovernanceCurrency, WriterLease},
    engine::CommittedTransition,
    executor::{
        DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    },
    ports::PortFailure,
};
use cognitive_provider_transport::{
    ReadOnlyFetchError, ReadOnlyFetchMethod, ReadOnlyFetchRequest, ReadOnlyFetchTransport,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedHttpFetchRequest {
    parameters_digest: String,
    target: String,
    output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedHttpFetch {
    redacted_output: Vec<u8>,
}

const HTTP_FETCH_STATE_NAMESPACE: &str = "http-fetch";
const HTTP_FETCH_STATE_SCHEMA: &str = "native-http-fetch-state/0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HttpFetchAttemptStatus {
    Staged,
    Attempted,
    NotExecuted,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HttpFetchStateRecord {
    schema: String,
    idempotency_key: String,
    parameters_digest: String,
    target: String,
    output_limit_bytes: usize,
    status: HttpFetchAttemptStatus,
    receipt_ref: Option<String>,
}

impl HttpFetchStateRecord {
    fn staged(idempotency_key: &str, request: &StagedHttpFetchRequest) -> Self {
        Self {
            schema: HTTP_FETCH_STATE_SCHEMA.to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            parameters_digest: request.parameters_digest.clone(),
            target: request.target.clone(),
            output_limit_bytes: request.output_limit_bytes,
            status: HttpFetchAttemptStatus::Staged,
            receipt_ref: None,
        }
    }
}

fn http_state_matches(
    record: &HttpFetchStateRecord,
    idempotency_key: &str,
    request: &StagedHttpFetchRequest,
) -> bool {
    record.schema == HTTP_FETCH_STATE_SCHEMA
        && record.idempotency_key == idempotency_key
        && record.parameters_digest == request.parameters_digest
        && record.target == request.target
        && record.output_limit_bytes == request.output_limit_bytes
}

/// Daemon-private read-only HTTP fetch sink.
///
/// The network policy is not restated here: staging calls the same registered
/// `validate_read_only_http_fetch` pre-executor validator that P2-T05 defined,
/// so the origin allowlist, HTTPS requirement, absent userinfo, absent query
/// and fragment, and timeout ceiling all come from one source. The sink adds
/// only what the Effect boundary needs — digest-bound staging, fencing,
/// original-key idempotency and bounded redacted retention.
///
/// This MVP issues `GET` only. The registered validator also admits `HEAD`, and
/// the transport supports it, but no caller needs it yet and there is no
/// registered parameter channel for choosing a verb, so inventing one here
/// would be an unregistered micro-contract. A request body is refused outright:
/// a read-only fetch has none.
pub(crate) struct NativeHttpFetchReadOnlyExecutor<T> {
    trusted_fencing_epoch: i64,
    transport: Arc<T>,
    allowed_origins: Vec<String>,
    timeout_ms: u32,
    staged_requests: Mutex<BTreeMap<String, StagedHttpFetchRequest>>,
    completed_fetches: Mutex<BTreeMap<String, CompletedHttpFetch>>,
    state_store: Arc<DurableExecutorStateStore>,
    #[cfg(test)]
    fetch_count: std::sync::atomic::AtomicUsize,
}

impl<T> NativeHttpFetchReadOnlyExecutor<T>
where
    T: ReadOnlyFetchTransport,
{
    pub(crate) fn new(
        trusted_fencing_epoch: i64,
        transport: Arc<T>,
        allowed_origins: Vec<String>,
        timeout_ms: u32,
        state_store: Arc<DurableExecutorStateStore>,
    ) -> Self {
        Self {
            trusted_fencing_epoch,
            transport,
            allowed_origins,
            timeout_ms,
            staged_requests: Mutex::new(BTreeMap::new()),
            completed_fetches: Mutex::new(BTreeMap::new()),
            state_store,
            #[cfg(test)]
            fetch_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        request: &ValidatedNativeToolRequest,
    ) -> Result<(), NativeToolExecutionError> {
        validate_descriptor(&request.descriptor)?;
        if request.descriptor.family != NativeOperationFamily::HttpFetchReadOnly {
            return Err(NativeToolExecutionError::UnsupportedExecutionFamily);
        }
        if idempotency_key.is_empty() || parameters_digest.is_empty() {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "idempotency key and parameters digest are required".to_owned(),
            ));
        }
        if !request.input.is_empty() {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "a read-only HTTP fetch does not accept a request body".to_owned(),
            ));
        }
        validate_read_only_http_fetch(
            "GET",
            &request.target,
            &self.allowed_origins,
            u64::from(self.timeout_ms),
        )
        .map_err(NativeToolExecutionError::InvalidDescriptor)?;
        let staged_request = StagedHttpFetchRequest {
            parameters_digest,
            target: request.target.clone(),
            output_limit_bytes: request.descriptor.output_limit_bytes,
        };
        let state_guard = self
            .state_store
            .lock_key(HTTP_FETCH_STATE_NAMESPACE, &idempotency_key)
            .map_err(|error| {
                NativeToolExecutionError::ExecutorUnavailable(format!(
                    "durable fetch state lock failed: {error}"
                ))
            })?;
        match state_guard
            .read::<HttpFetchStateRecord>()
            .map_err(|error| {
                NativeToolExecutionError::ExecutorUnavailable(format!(
                    "durable fetch state read failed: {error}"
                ))
            })? {
            Some(existing) if !http_state_matches(&existing, &idempotency_key, &staged_request) => {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            Some(_) => {}
            None => {
                if state_guard.key_previously_seen() {
                    return Err(NativeToolExecutionError::ExecutorUnavailable(
                        "durable fetch state is missing for a previously seen key".to_owned(),
                    ));
                }
                state_guard
                    .write(&HttpFetchStateRecord::staged(
                        &idempotency_key,
                        &staged_request,
                    ))
                    .map_err(|error| {
                        NativeToolExecutionError::ExecutorUnavailable(format!(
                            "durable fetch state write failed: {error}"
                        ))
                    })?
            }
        }
        let mut staged_requests = self.staged_requests.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "staged fetch store is poisoned".to_owned(),
            )
        })?;
        if let Some(existing_request) = staged_requests.get(&idempotency_key) {
            if existing_request != &staged_request {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_requests.insert(idempotency_key, staged_request);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn completed_output(&self, idempotency_key: &str) -> Option<Vec<u8>> {
        self.completed_fetches
            .lock()
            .ok()
            .and_then(|completed| completed.get(idempotency_key).cloned())
            .map(|completed| completed.redacted_output)
    }

    #[cfg(test)]
    pub(crate) fn fetch_count(&self) -> usize {
        self.fetch_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn remove_durable_state(&self, idempotency_key: &str) -> std::io::Result<bool> {
        self.state_store
            .lock_key(HTTP_FETCH_STATE_NAMESPACE, idempotency_key)?
            .remove_record()
    }

    fn fetch_staged_target(
        &self,
        call: &ExecutorCall,
        staged_request: &StagedHttpFetchRequest,
    ) -> Result<DispatchOutcome, PortFailure> {
        if call.action != "fetch"
            || call.target != staged_request.target
            || call.parameters_digest != staged_request.parameters_digest
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "dispatch does not match the daemon-staged read-only fetch".to_owned(),
            });
        }
        let state_guard = self
            .state_store
            .lock_key(HTTP_FETCH_STATE_NAMESPACE, &call.idempotency_key)
            .map_err(|error| PortFailure {
                detail: format!("durable fetch state lock failed: {error}"),
            })?;
        let Some(mut state) = state_guard
            .read::<HttpFetchStateRecord>()
            .map_err(|error| PortFailure {
                detail: format!("durable fetch state read failed: {error}"),
            })?
        else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no durable daemon-staged fetch for idempotency key".to_owned(),
            });
        };
        if !http_state_matches(&state, &call.idempotency_key, staged_request) {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "durable fetch binding does not match dispatch".to_owned(),
            });
        }
        match state.status {
            HttpFetchAttemptStatus::Completed => {
                let Some(receipt_ref) = state.receipt_ref else {
                    return Ok(DispatchOutcome::Unknown {
                        detail: "completed fetch state has no key-bound receipt".to_owned(),
                    });
                };
                if receipt_ref != format!("tool-receipt://http-fetch/{}", call.idempotency_key) {
                    return Ok(DispatchOutcome::Unknown {
                        detail: "completed fetch receipt is bound to a different key".to_owned(),
                    });
                }
                return Ok(DispatchOutcome::Executed { receipt_ref });
            }
            HttpFetchAttemptStatus::Attempted => {
                return Ok(DispatchOutcome::Unknown {
                    detail: "a prior fetch attempt has no durable terminal receipt".to_owned(),
                });
            }
            HttpFetchAttemptStatus::Staged | HttpFetchAttemptStatus::NotExecuted => {}
        }
        state.status = HttpFetchAttemptStatus::Attempted;
        state.receipt_ref = None;
        state_guard.write(&state).map_err(|error| PortFailure {
            detail: format!("persist fetch attempt before egress: {error}"),
        })?;
        #[cfg(test)]
        self.fetch_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let response = match self.transport.fetch(&ReadOnlyFetchRequest {
            method: ReadOnlyFetchMethod::Get,
            url: staged_request.target.clone(),
            timeout_ms: self.timeout_ms,
            maximum_response_bytes: staged_request.output_limit_bytes,
        }) {
            Ok(response) => response,
            // A policy refusal happens before egress, so non-execution is a
            // fact rather than an inference.
            Err(ReadOnlyFetchError::Policy { detail }) => {
                state.status = HttpFetchAttemptStatus::NotExecuted;
                if let Err(error) = state_guard.write(&state) {
                    return Ok(DispatchOutcome::Unknown {
                        detail: format!(
                            "fetch was refused before egress but durable disposition failed: {error}"
                        ),
                    });
                }
                return Ok(DispatchOutcome::NotExecuted {
                    reason: format!("read-only fetch refused before egress: {detail}"),
                });
            }
            // An oversized body did reach this process, but nothing external
            // changed and nothing was retained, so this Effect executed no
            // observable work.
            Err(ReadOnlyFetchError::ResponseTooLarge) => {
                state.status = HttpFetchAttemptStatus::NotExecuted;
                if let Err(error) = state_guard.write(&state) {
                    return Ok(DispatchOutcome::Unknown {
                        detail: format!(
                            "oversize fetch retained nothing but durable disposition failed: {error}"
                        ),
                    });
                }
                return Ok(DispatchOutcome::NotExecuted {
                    reason: "read-only fetch response exceeded the registered output bound; nothing retained"
                        .to_owned(),
                });
            }
            // A timeout or transport fault may or may not have reached the
            // origin. That is exactly the uncertain outcome class.
            Err(ReadOnlyFetchError::Timeout) => {
                return Ok(DispatchOutcome::Unknown {
                    detail: "read-only fetch reached its bounded deadline".to_owned(),
                });
            }
            Err(ReadOnlyFetchError::Network { detail }) => {
                return Ok(DispatchOutcome::Unknown {
                    detail: format!("read-only fetch outcome is uncertain: {detail}"),
                });
            }
        };
        let rendered_response = format!(
            "{}\n{}",
            response.status,
            String::from_utf8_lossy(&response.body)
        );
        let redacted_response = redact_sensitive_output(&rendered_response);
        let redacted_output =
            truncate_utf8_to_bytes(&redacted_response, staged_request.output_limit_bytes)
                .to_owned();
        let receipt_ref = format!("tool-receipt://http-fetch/{}", call.idempotency_key);
        state.status = HttpFetchAttemptStatus::Completed;
        state.receipt_ref = Some(receipt_ref.clone());
        if let Err(error) = state_guard.write(&state) {
            return Ok(DispatchOutcome::Unknown {
                detail: format!(
                    "fetch completed but its key-bound durable receipt could not be stored: {error}"
                ),
            });
        }
        self.completed_fetches
            .lock()
            .map_err(|_| PortFailure {
                detail: "completed fetch store is poisoned".to_owned(),
            })?
            .insert(
                call.idempotency_key.clone(),
                CompletedHttpFetch {
                    redacted_output: redacted_output.into_bytes(),
                },
            );
        Ok(DispatchOutcome::Executed { receipt_ref })
    }
}

fn truncate_utf8_to_bytes(value: &str, maximum_bytes: usize) -> &str {
    if value.len() <= maximum_bytes {
        return value;
    }
    let mut boundary = maximum_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.get(..boundary).unwrap_or_default()
}

impl<T> EffectExecutor for NativeHttpFetchReadOnlyExecutor<T>
where
    T: ReadOnlyFetchTransport,
{
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        }
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        if call.fencing_epoch != self.trusted_fencing_epoch {
            return Ok(DispatchOutcome::FencedStaleEpoch {
                sink_epoch: self.trusted_fencing_epoch,
            });
        }
        let staged_request = self
            .staged_requests
            .lock()
            .map_err(|_| PortFailure {
                detail: "staged fetch store is poisoned".to_owned(),
            })?
            .get(&call.idempotency_key)
            .cloned();
        let Some(staged_request) = staged_request else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged read-only fetch for idempotency key".to_owned(),
            });
        };
        self.fetch_staged_target(call, &staged_request)
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let state_guard = self
            .state_store
            .lock_key(HTTP_FETCH_STATE_NAMESPACE, idempotency_key)
            .map_err(|error| PortFailure {
                detail: format!("durable fetch state lock failed: {error}"),
            })?;
        let state = state_guard
            .read::<HttpFetchStateRecord>()
            .map_err(|error| PortFailure {
                detail: format!("durable fetch state read failed: {error}"),
            })?;
        let Some(state) = state else {
            return Ok(ExecutorQueryResult::Indeterminate);
        };
        if state.schema != HTTP_FETCH_STATE_SCHEMA || state.idempotency_key != idempotency_key {
            return Ok(ExecutorQueryResult::Indeterminate);
        }
        let expected_receipt = format!("tool-receipt://http-fetch/{idempotency_key}");
        Ok(match state.status {
            HttpFetchAttemptStatus::Completed
                if state.receipt_ref.as_deref() == Some(expected_receipt.as_str()) =>
            {
                ExecutorQueryResult::ExecutedWithOriginalKey
            }
            HttpFetchAttemptStatus::Completed | HttpFetchAttemptStatus::Attempted => {
                ExecutorQueryResult::Indeterminate
            }
            HttpFetchAttemptStatus::Staged | HttpFetchAttemptStatus::NotExecuted => {
                ExecutorQueryResult::NotExecuted
            }
        })
    }
}

/// Drive an already staged read-only fetch through the durable Effect protocol.
pub(crate) fn dispatch_staged_http_fetch_effect<S, C, G, T>(
    effect_protocol: &EffectProtocol<'_, S, C, G>,
    effect_object_id: &ObjectId,
    expected_effect_version: Version,
    grant: &AuthorizationGrant,
    governance_currency: &GovernanceCurrency,
    executor: &NativeHttpFetchReadOnlyExecutor<T>,
    writer_lease: &WriterLease,
) -> Result<CommittedTransition, EffectError>
where
    S: cognitive_kernel::ports::AuthorityStore + cognitive_kernel::ports::ProtocolStore,
    C: cognitive_kernel::ports::Clock,
    G: cognitive_kernel::ports::IdGenerator,
    T: ReadOnlyFetchTransport,
{
    let authorized = effect_protocol.authorize_effect(
        effect_object_id,
        expected_effect_version,
        grant,
        governance_currency,
        writer_lease,
    )?;
    let (dispatched, outcome) = effect_protocol.dispatch_effect(
        effect_object_id,
        authorized.after_version,
        grant,
        governance_currency,
        executor,
        writer_lease,
    )?;
    effect_protocol.record_outcome(
        effect_object_id,
        dispatched.after_version,
        &outcome,
        writer_lease,
    )
}
