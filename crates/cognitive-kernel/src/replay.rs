//! Projection replay over the committed event history (REQ-STATE-002,
//! REQ-EVT-003 family; `docs/standards/event-audit-watch.md` section 3).
//!
//! Projections are derived and disposable: replaying the committed,
//! append-only event log reproduces a status projection with a stable
//! canonical digest. Replay consumes only committed history — it never
//! infers transitions from process state, timeouts, or receipts — and any
//! inconsistency in the log (unknown committed event type, version gap,
//! state mismatch) is a recovery barrier, not a guess
//! (`state-and-transition-contract.md` section 6).

use crate::engine::{EVENT_TYPE_OBJECT_ADMITTED, EVENT_TYPE_TRANSITION_COMMITTED};
use crate::ports::{AuthorityStore, StorePortError};

/// Event type appended when an Intent is durably persisted (M4;
/// audit/provenance event — it advances no object state and replay folds
/// it as provenance only).
pub const EVENT_TYPE_INTENT_PERSISTED: &str = "cognitiveos.intent.persisted";
use cognitive_contracts::canonical;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Fixed projection version (REQ-STATE-002: a fixed projection version over
/// the same ordered event set yields the same digest). Also used as the
/// digest domain label. Implementation-scoped pending a registered
/// projection contract (recorded as an M2 contract gap for Lane-CTR).
pub const PROJECTION_VERSION: &str = "cognitiveos.impl.execution-status-projection/0.1";

/// Replay failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReplayError {
    /// Reading committed history failed (fail closed).
    #[error(transparent)]
    Store(#[from] StorePortError),
    /// Committed history is internally inconsistent: recovery barrier.
    #[error("replay-barrier: {0}")]
    Barrier(String),
    /// Canonical encoding of the projection failed.
    #[error("replay-encoding: {0}")]
    Encoding(String),
}

/// A replayed status projection with its canonical identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayedProjection {
    /// The projection value.
    pub value: Value,
    /// RFC 8785 canonical bytes of the projection value.
    pub canonical_bytes: Vec<u8>,
    /// Domain-separated digest of the canonical bytes.
    pub digest: String,
    /// Highest event sequence folded in.
    pub high_watermark: i64,
    /// Number of events folded in.
    pub event_count: u64,
}

#[derive(Debug, Clone)]
struct FoldedObject {
    domain: String,
    state: String,
    version: i64,
}

fn field<'v>(event: &'v Value, name: &str, sequence: i64) -> Result<&'v Value, ReplayError> {
    event.get(name).ok_or_else(|| {
        ReplayError::Barrier(format!("event at sequence {sequence} misses field {name}"))
    })
}

fn str_field(event: &Value, name: &str, sequence: i64) -> Result<String, ReplayError> {
    Ok(field(event, name, sequence)?
        .as_str()
        .ok_or_else(|| {
            ReplayError::Barrier(format!(
                "event at sequence {sequence} field {name} is not a string"
            ))
        })?
        .to_owned())
}

fn int_field(event: &Value, name: &str, sequence: i64) -> Result<i64, ReplayError> {
    field(event, name, sequence)?.as_i64().ok_or_else(|| {
        ReplayError::Barrier(format!(
            "event at sequence {sequence} field {name} is not an integer"
        ))
    })
}

/// Replay the full committed event history into a status projection.
/// Deterministic: the same ordered event set always yields byte-identical
/// canonical bytes and the same digest.
pub fn replay_projection<S: AuthorityStore>(store: &S) -> Result<ReplayedProjection, ReplayError> {
    const PAGE: usize = 256;
    let mut objects: BTreeMap<String, FoldedObject> = BTreeMap::new();
    let mut after = 0i64;
    let mut high_watermark = 0i64;
    let mut event_count = 0u64;

    loop {
        let page = store.read_events(after, PAGE)?;
        if page.is_empty() {
            break;
        }
        for committed in &page {
            let sequence = committed.sequence;
            if sequence <= high_watermark {
                return Err(ReplayError::Barrier(format!(
                    "event sequence {sequence} not strictly increasing"
                )));
            }
            high_watermark = sequence;
            event_count += 1;

            // Fold from the committed event VALUE (the event is the truth;
            // the row columns are derived copies checked against it).
            let event: Value = serde_json::from_str(&committed.canonical_json).map_err(|err| {
                ReplayError::Barrier(format!("event at sequence {sequence} unparseable: {err}"))
            })?;
            let event_type = str_field(&event, "event_type", sequence)?;
            let object_id = str_field(&event, "object_id", sequence)?;
            let domain = str_field(&event, "domain", sequence)?;
            if object_id != committed.object_id.as_str()
                || domain != committed.domain.as_str()
                || event_type != committed.event_type
            {
                return Err(ReplayError::Barrier(format!(
                    "event at sequence {sequence}: log columns diverge from event value"
                )));
            }

            // Provenance-only events fold no state (the intent event is
            // keyed by the intent's own identity).
            if event_type == EVENT_TYPE_INTENT_PERSISTED {
                str_field(&event, "idempotency_key", sequence)?;
                continue;
            }

            let after_state = str_field(&event, "after_state", sequence)?;
            let after_version = int_field(&event, "after_version", sequence)?;
            if after_version != committed.object_version.get() {
                return Err(ReplayError::Barrier(format!(
                    "event at sequence {sequence}: log columns diverge from event value"
                )));
            }

            match event_type.as_str() {
                EVENT_TYPE_OBJECT_ADMITTED => {
                    if after_version != 1 {
                        return Err(ReplayError::Barrier(format!(
                            "admission at sequence {sequence} claims version {after_version}"
                        )));
                    }
                    if objects
                        .insert(
                            object_id.clone(),
                            FoldedObject {
                                domain,
                                state: after_state,
                                version: after_version,
                            },
                        )
                        .is_some()
                    {
                        return Err(ReplayError::Barrier(format!(
                            "duplicate admission of {object_id} at sequence {sequence}"
                        )));
                    }
                }
                EVENT_TYPE_TRANSITION_COMMITTED => {
                    let before_state = str_field(&event, "before_state", sequence)?;
                    let entry = objects.get_mut(&object_id).ok_or_else(|| {
                        ReplayError::Barrier(format!(
                            "transition of unknown object {object_id} at sequence {sequence}"
                        ))
                    })?;
                    if entry.domain != domain {
                        return Err(ReplayError::Barrier(format!(
                            "domain change of {object_id} at sequence {sequence}"
                        )));
                    }
                    if entry.state != before_state {
                        return Err(ReplayError::Barrier(format!(
                            "before_state {before_state} at sequence {sequence} but replayed state is {}",
                            entry.state
                        )));
                    }
                    if after_version != entry.version + 1 {
                        return Err(ReplayError::Barrier(format!(
                            "version gap for {object_id} at sequence {sequence}: {} -> {after_version}",
                            entry.version
                        )));
                    }
                    entry.state = after_state;
                    entry.version = after_version;
                }
                other => {
                    return Err(ReplayError::Barrier(format!(
                        "unknown committed event type {other} at sequence {sequence}"
                    )));
                }
            }
        }
        after = high_watermark;
        if page.len() < PAGE {
            break;
        }
    }

    let object_values: Vec<Value> = objects
        .iter()
        .map(|(object_id, folded)| {
            json!({
                "object_id": object_id,
                "domain": folded.domain,
                "state": folded.state,
                "version": folded.version,
            })
        })
        .collect();
    let value = json!({
        "projection_version": PROJECTION_VERSION,
        "source": {
            "event_count": event_count,
            "high_watermark": high_watermark,
        },
        "objects": object_values,
    });
    let canonical_bytes = canonical::canonical_bytes_of_value(&value)
        .map_err(|err| ReplayError::Encoding(err.to_string()))?;
    let digest = canonical::digest(&canonical_bytes, PROJECTION_VERSION)
        .map_err(|err| ReplayError::Encoding(err.to_string()))?;
    Ok(ReplayedProjection {
        value,
        canonical_bytes,
        digest,
        high_watermark,
        event_count,
    })
}
