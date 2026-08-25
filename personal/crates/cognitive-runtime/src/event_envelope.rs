//! D-018 publication-boundary assembler: committed fact plus its durable
//! governance header.
use cognitive_contracts::generated::{event, governed_object_header::GovernedObjectHeader};
use cognitive_kernel::ports::{CommittedEvent, GovernanceObjectStore};
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
pub enum EventEnvelopeError {
    #[error("invalid committed event: {0}")]
    Invalid(String),
}
/// Assemble a publication envelope from the authority store's immutable
/// governance record. A runtime caller cannot substitute a header from an
/// untrusted request or projection.
pub fn assemble_persisted_event<S>(
    store: &S,
    committed: &CommittedEvent,
    ingest_time: &str,
) -> Result<Value, EventEnvelopeError>
where
    S: GovernanceObjectStore,
{
    let header = store
        .load_governed_object_header(&committed.object_id)
        .map_err(|err| EventEnvelopeError::Invalid(format!("governance header lookup: {err}")))?
        .ok_or_else(|| EventEnvelopeError::Invalid("governance header missing".to_owned()))?;
    assemble_with_header(committed, &header, ingest_time)
}

fn assemble_with_header(
    committed: &CommittedEvent,
    header: &GovernedObjectHeader,
    ingest_time: &str,
) -> Result<Value, EventEnvelopeError> {
    let internal: Value = serde_json::from_str(&committed.canonical_json)
        .map_err(|e| EventEnvelopeError::Invalid(e.to_string()))?;
    let event_time = internal
        .get("event_time")
        .and_then(Value::as_str)
        .ok_or_else(|| EventEnvelopeError::Invalid("event_time missing".to_owned()))?;
    let correlation = internal
        .pointer("/causation/correlation_id")
        .or_else(|| internal.get("correlation_id"))
        .and_then(Value::as_str)
        .ok_or_else(|| EventEnvelopeError::Invalid("correlation_id missing".to_owned()))?;
    let causation = internal
        .pointer("/causation/causation_id")
        .or_else(|| internal.get("causation_id"))
        .and_then(Value::as_str);
    let payload = internal
        .get("payload")
        .cloned()
        .unwrap_or_else(|| internal.clone());
    Ok(
        json!({"header":header,"event_type":committed.event_type,"source":"authority://cognitiveos/state","subject":format!("{}://{}",committed.domain.as_str(),committed.object_id),"correlation_id":correlation,"causation_id":causation,"event_time":event_time,"ingest_time":ingest_time,"schema_digest":event::SCHEMA_DIGEST,"deadline":null,"delivery_class":"at_least_once","ack":{"mode":"explicit","consumer_offset":"required"},"backpressure":{"mode":"bounded_block","overflow":"reject"},"payload":payload,"immutable":true}),
    )
}
