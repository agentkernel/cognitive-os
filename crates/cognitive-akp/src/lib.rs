//! `cognitive-akp`: Agent Kernel Protocol envelope and transport profile of
//! the CognitiveOS reference implementation.
//!
//! Scope (M5, per `docs/plan/DEVELOPMENT-PLAN.md`): the AKP envelope with
//! pinned schema digests, unknown-critical-extension fail-closed handling,
//! and the single-node HTTP JSON + SSE watch transport profile
//! (`docs/adr/0003-json-http-sse.md`, reference implementation decision).
//!
//! Normative source: `specs/akp/README.md` and the registered AKP error
//! codes in `specs/registry/errors.yaml`.

use cognitive_contracts::generated::akp_request_envelope::AkpRequestEnvelope;
use serde_json::{Value, json};

const VERSION: &str = "cognitiveos.akp/0.2";
const PAYLOAD_DOMAIN: &str = "akp-payload/0.2";

#[derive(Debug, thiserror::Error)]
pub enum AkpError {
    #[error("{detail} ({code})")]
    Registered { code: &'static str, detail: String },
    #[error("malformed envelope: {0}")]
    Malformed(String),
}
impl AkpError {
    pub fn code(&self) -> &str {
        match self {
            Self::Registered { code, .. } => code,
            Self::Malformed(_) => "SCHEMA_MISMATCH",
        }
    }
}

pub fn parse_request(
    bytes: &[u8],
    expected_schema_digest: &str,
) -> Result<AkpRequestEnvelope, AkpError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|e| AkpError::Malformed(e.to_string()))?;
    if value.get("protocol_version").and_then(Value::as_str) != Some(VERSION) {
        return Err(reg(
            "VERSION_UNSUPPORTED",
            "unsupported AKP protocol version",
        ));
    }
    if value
        .get("extensions")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("critical") == Some(&Value::Bool(true)))
        })
    {
        return Err(reg(
            "CRITICAL_EXTENSION_UNKNOWN",
            "unknown critical extension",
        ));
    }
    if value.get("schema_digest").and_then(Value::as_str) != Some(expected_schema_digest) {
        return Err(reg(
            "PROTOCOL_SCHEMA_DIGEST_MISMATCH",
            "payload schema pin differs",
        ));
    }
    let envelope: AkpRequestEnvelope =
        serde_json::from_value(value).map_err(|e| AkpError::Malformed(e.to_string()))?;
    if envelope.payload.is_some() == envelope.payload_ref.is_some() {
        return Err(AkpError::Malformed(
            "exactly one payload source required".to_owned(),
        ));
    }
    if let (Some(payload), Some(declared)) = (&envelope.payload, &envelope.payload_digest) {
        let actual = digest(payload)?;
        if actual != declared.0 {
            return Err(reg("DIGEST_MISMATCH", "inline payload digest differs"));
        }
    }
    Ok(envelope)
}

pub fn result_ok(request: &AkpRequestEnvelope, result: Value) -> Result<Value, AkpError> {
    let result_digest = digest(&result)?;
    Ok(
        json!({"in_reply_to":request.message_id,"correlation_id":request.correlation_id,"protocol_version":VERSION,"status":"ok","result":result,"result_digest":result_digest}),
    )
}

fn digest(value: &Value) -> Result<String, AkpError> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(value)
        .map_err(|e| AkpError::Malformed(e.to_string()))?;
    cognitive_contracts::canonical::digest(&bytes, PAYLOAD_DOMAIN)
        .map_err(|e| AkpError::Malformed(e.to_string()))
}
fn reg(code: &'static str, detail: &str) -> AkpError {
    AkpError::Registered {
        code,
        detail: detail.to_owned(),
    }
}

#[derive(Debug, Clone)]
struct WatchEntry {
    sequence: i64,
    payload: Value,
}
#[derive(Debug)]
pub struct WatchLog {
    stream_id: String,
    retention: usize,
    minimum_cursor: i64,
    next: i64,
    entries: Vec<WatchEntry>,
}
impl WatchLog {
    pub fn new(stream_id: &str, retention: usize) -> Self {
        Self {
            stream_id: stream_id.to_owned(),
            retention: retention.max(1),
            minimum_cursor: 0,
            next: 1,
            entries: Vec::new(),
        }
    }
    pub fn append(&mut self, payload: Value) -> Result<i64, AkpError> {
        let sequence = self.next;
        self.next += 1;
        self.entries.push(WatchEntry { sequence, payload });
        Ok(sequence)
    }
    pub fn compact_through(&mut self, cursor: i64) {
        self.entries.retain(|entry| entry.sequence > cursor);
        self.minimum_cursor = self.minimum_cursor.max(cursor);
        while self.entries.len() > self.retention {
            let removed = self.entries.remove(0);
            self.minimum_cursor = self.minimum_cursor.max(removed.sequence);
        }
    }
    pub fn open(&self, snapshot: Value) -> Result<Vec<Value>, AkpError> {
        let mut frames = vec![frame(
            &self.stream_id,
            0,
            "snapshot",
            snapshot,
            Some(self.next - 1),
        )?];
        for entry in &self.entries {
            frames.push(frame(
                &self.stream_id,
                entry.sequence,
                "delta",
                entry.payload.clone(),
                None,
            )?);
        }
        Ok(frames)
    }
    pub fn resume(&self, cursor: i64) -> Result<Vec<Value>, AkpError> {
        if cursor < self.minimum_cursor {
            return Err(reg(
                "WATCH_CURSOR_STALE",
                "cursor continuity was compacted; fresh snapshot required",
            ));
        }
        self.entries
            .iter()
            .filter(|entry| entry.sequence > cursor)
            .map(|entry| {
                frame(
                    &self.stream_id,
                    entry.sequence,
                    "delta",
                    entry.payload.clone(),
                    None,
                )
            })
            .collect()
    }
}
fn frame(
    stream: &str,
    sequence: i64,
    kind: &str,
    payload: Value,
    snapshot_version: Option<i64>,
) -> Result<Value, AkpError> {
    let payload_digest = digest(&payload)?;
    Ok(
        json!({"stream_id":stream,"sequence":sequence,"kind":kind,"payload":payload,"payload_digest":payload_digest,"snapshot_version":snapshot_version}),
    )
}
/// Transport profile implemented by the single-node reference deployment.
pub const TRANSPORT_PROFILE: &str = "http-json+sse (planned, M5)";

#[cfg(test)]
mod tests {
    #[test]
    fn envelope_layer_builds_on_contracts() {
        assert_eq!(
            cognitive_contracts::ENCODING_PROFILE,
            "cognitiveos.canonical-json/0.1"
        );
    }
}
