//! Validated identifier and primitive newtypes (REQ-GOBJ-ID-001, ADR-0005).
//!
//! Stable object identity is lowercase canonical UUIDv7 (`8-4-4-4-12`, no
//! braces, no `urn:uuid:`). IDs are identity, not authorization, and are
//! treated as opaque beyond format validation: UUIDv7 timestamp bits are
//! never read back as event time, order proof, or freshness (ADR-0005).
//!
//! Reference and actor fields of transition requests/records are URI
//! references (`state-transition-request.schema.json` uses
//! `common-defs.schema.json#/$defs/uriRef`), kept as a distinct newtype so
//! they can never be confused with object identity.

use crate::error::DomainError;
use cognitive_contracts::generated::object_reference::UuidV7;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Validate lowercase canonical UUIDv7 text form:
/// `xxxxxxxx-xxxx-7xxx-[89ab]xxx-xxxxxxxxxxxx` (RFC 9562; the same pattern
/// enforced by `object-reference.schema.json#/$defs/uuidV7`).
fn is_canonical_uuid_v7(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            14 => {
                if *byte != b'7' {
                    return false;
                }
            }
            19 => {
                if !matches!(*byte, b'8' | b'9' | b'a' | b'b') {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(byte) {
                    return false;
                }
            }
        }
    }
    true
}

macro_rules! uuid_v7_newtype {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Parse and validate the lowercase canonical UUIDv7 form.
            pub fn parse(value: &str) -> Result<Self, DomainError> {
                if is_canonical_uuid_v7(value) {
                    Ok(Self(value.to_owned()))
                } else {
                    Err(DomainError::InvalidUuidV7(value.to_owned()))
                }
            }

            /// Canonical string form.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Convert into the schema-generated transparent newtype.
            pub fn to_generated(&self) -> UuidV7 {
                UuidV7(self.0.clone())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<&UuidV7> for $name {
            type Error = DomainError;

            fn try_from(value: &UuidV7) -> Result<Self, DomainError> {
                Self::parse(&value.0)
            }
        }
    };
}

uuid_v7_newtype!(
    /// Stable identity of one governed object (REQ-GOBJ-ID-001).
    ObjectId
);
uuid_v7_newtype!(
    /// Identity of one committed event-log entry.
    EventId
);
uuid_v7_newtype!(
    /// Identity of one committed state-transition record.
    RecordId
);
uuid_v7_newtype!(
    /// Identity of one hard-budget ledger row.
    BudgetId
);

/// Non-empty URI reference (`common-defs.schema.json#/$defs/uriRef`), used
/// for `request_id`, `subject_ref`, `actor_ref`, `authority_ref`,
/// `causation_id` and `correlation_id` fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UriRef(String);

impl UriRef {
    /// Accept any non-empty URI reference without control characters or
    /// spaces. Full RFC 3986 parsing is intentionally not re-implemented
    /// here; schema-level `format: uri-reference` stays with the validator.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let ok = !value.is_empty() && value.bytes().all(|b| b.is_ascii_graphic() || !b.is_ascii());
        if ok {
            Ok(Self(value.to_owned()))
        } else {
            Err(DomainError::InvalidUriRef(value.to_owned()))
        }
    }

    /// The underlying reference text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UriRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// State name in table grammar `^[A-Z][A-Z0-9_]*$`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StateName(String);

impl StateName {
    /// Parse and validate the uppercase state-name grammar.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        if is_upper_snake(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(DomainError::InvalidStateName(value.to_owned()))
        }
    }

    /// The state name text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StateName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Structured reason code in grammar `^[A-Z][A-Z0-9_]*$`. Reason codes are
/// transition-table data, not registry error codes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReasonCode(String);

impl ReasonCode {
    /// Parse and validate the uppercase reason-code grammar.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        if is_upper_snake(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(DomainError::InvalidReasonCode(value.to_owned()))
        }
    }

    /// The reason code text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_upper_snake(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes.split_first() {
        Some((head, tail)) => {
            head.is_ascii_uppercase()
                && tail
                    .iter()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || *b == b'_')
        }
        None => false,
    }
}

/// Canonical RFC 3339 UTC wall timestamp (`wall_clock` domain, ADR-0005;
/// canonical FORM per `canonical-encoding-and-digest.md` section 6).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WallTimestamp(String);

impl WallTimestamp {
    /// Parse and validate the canonical UTC timestamp form.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        cognitive_contracts::projection::validate_canonical_timestamp(value)
            .map_err(|_| DomainError::InvalidWallTimestamp(value.to_owned()))?;
        Ok(Self(value.to_owned()))
    }

    /// The timestamp text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WallTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    const GOOD: &str = "018f4d2e-7c1a-7d3b-9a2f-0123456789ab";

    #[test]
    fn accepts_lowercase_canonical_uuid_v7() {
        let id = ObjectId::parse(GOOD).unwrap();
        assert_eq!(id.as_str(), GOOD);
        assert_eq!(id.to_generated().0, GOOD);
    }

    #[test]
    fn rejects_non_v7_uppercase_braced_and_urn_forms() {
        for bad in [
            "",
            "not-a-uuid",
            "018F4D2E-7C1A-7D3B-9A2F-0123456789AB",
            "{018f4d2e-7c1a-7d3b-9a2f-0123456789ab}",
            "urn:uuid:018f4d2e-7c1a-7d3b-9a2f-0123456789ab",
            "018f4d2e-7c1a-4d3b-9a2f-0123456789ab",
            "018f4d2e-7c1a-7d3b-ca2f-0123456789ab",
            "018f4d2e7c1a7d3b9a2f0123456789ab",
        ] {
            assert!(ObjectId::parse(bad).is_err(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn uri_ref_rejects_empty_and_whitespace() {
        assert!(UriRef::parse("task://tenant-a/task-42").is_ok());
        assert!(UriRef::parse("").is_err());
        assert!(UriRef::parse("has space").is_err());
        assert!(UriRef::parse("has\ttab").is_err());
    }

    #[test]
    fn state_and_reason_grammar() {
        assert!(StateName::parse("OUTCOME_UNKNOWN").is_ok());
        assert!(StateName::parse("outcome").is_err());
        assert!(StateName::parse("_X").is_err());
        assert!(ReasonCode::parse("AUTHORIZATION_GRANTED").is_ok());
        assert!(ReasonCode::parse("granted").is_err());
    }

    #[test]
    fn wall_timestamp_requires_canonical_utc_form() {
        assert!(WallTimestamp::parse("2026-07-20T05:00:00Z").is_ok());
        assert!(WallTimestamp::parse("2026-07-20T05:00:00.123Z").is_ok());
        assert!(WallTimestamp::parse("2026-07-20T05:00:00+02:00").is_err());
        assert!(WallTimestamp::parse("2026-07-20T05:00:00.120Z").is_err());
    }
}
