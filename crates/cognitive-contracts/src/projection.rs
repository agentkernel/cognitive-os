//! Digest projections, content-digest verification, canonical timestamp and
//! digest-string validation, and the unknown-critical-extension gate.
//!
//! Implements the schema-bound layers of
//! `docs/standards/canonical-encoding-and-digest.md` sitting above the
//! encoding module:
//!
//! - section 6: canonical RFC 3339 UTC timestamp FORM (uppercase `T`/`Z`,
//!   no offset, no local time, no leap second, 1-9 fraction digits without
//!   trailing zeros, zero fraction omitted);
//! - section 8: machine digest string form `sha256:<64 lowercase hex>`;
//! - section 10: content-digest exact input — remove ONLY the JSON Pointer
//!   paths a contract explicitly declares `digest_excluded`, then canonical
//!   bytes, then the domain-separated digest. Verification recomputes from
//!   the received semantic value and fails closed on mismatch;
//! - section 3 / AKP envelope: unknown critical extensions are rejected
//!   before any payload processing (`CRITICAL_EXTENSION_UNKNOWN` semantics).
//!
//! The TypeScript twin is `packages/contracts-ts/src/projection.ts`;
//! cross-language behavior is pinned by
//! `tests/golden/digest-and-projection-fixtures.json`.

use crate::canonical;
use serde_json::Value;

/// Rejection categories shared with the TypeScript twin and the golden
/// fixtures (string forms are fixture contract).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProjectionError {
    /// Digest string violates `sha256:<64 lowercase hex>` (section 8).
    #[error("invalid-digest: {0}")]
    InvalidDigest(String),
    /// Timestamp violates the canonical RFC 3339 UTC form (section 6).
    #[error("invalid-timestamp: {0}")]
    InvalidTimestamp(String),
    /// A declared `digest_excluded` entry is not a usable JSON Pointer.
    #[error("invalid-pointer: {0}")]
    InvalidPointer(String),
    /// No digest value at the declared self-digest pointer.
    #[error("missing-digest: {0}")]
    MissingDigest(String),
    /// Recomputed projection digest differs from the declared digest.
    #[error("digest-mismatch: declared {declared}, computed {computed}")]
    DigestMismatch { declared: String, computed: String },
    /// An unknown (or unverifiable) critical extension was present.
    #[error("critical-extension-unknown: {0}")]
    CriticalExtensionUnknown(String),
    /// Canonicalization/digest failure (propagated).
    #[error(transparent)]
    Canonical(#[from] canonical::CanonicalError),
}

impl ProjectionError {
    /// Stable machine category used by golden fixtures.
    pub fn category(&self) -> &'static str {
        match self {
            ProjectionError::InvalidDigest(_) => "invalid-digest",
            ProjectionError::InvalidTimestamp(_) => "invalid-timestamp",
            ProjectionError::InvalidPointer(_) => "invalid-pointer",
            ProjectionError::MissingDigest(_) => "missing-digest",
            ProjectionError::DigestMismatch { .. } => "digest-mismatch",
            ProjectionError::CriticalExtensionUnknown(_) => "critical-extension-unknown",
            ProjectionError::Canonical(err) => err.category(),
        }
    }
}

/// Validate the machine digest string form (section 8): exactly
/// `sha256:` + 64 lowercase hexadecimal digits.
pub fn validate_digest_string(digest: &str) -> Result<(), ProjectionError> {
    let rest = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| ProjectionError::InvalidDigest(digest.to_owned()))?;
    let ok = rest.len() == 64
        && rest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if ok {
        Ok(())
    } else {
        Err(ProjectionError::InvalidDigest(digest.to_owned()))
    }
}

/// Validate the canonical RFC 3339 UTC timestamp FORM (section 6):
/// `YYYY-MM-DDTHH:MM:SS[.fraction]Z`, uppercase `T`/`Z`, no offset or local
/// time, no leap second, fraction 1-9 digits, no trailing zeros, zero
/// fraction omitted.
pub fn validate_canonical_timestamp(value: &str) -> Result<(), ProjectionError> {
    let err = || ProjectionError::InvalidTimestamp(value.to_owned());
    let bytes = value.as_bytes();
    if bytes.len() < 20 {
        return Err(err());
    }
    let digits = |range: std::ops::Range<usize>| -> Result<u32, ProjectionError> {
        let mut out: u32 = 0;
        for &b in bytes.get(range).ok_or_else(err)? {
            if !b.is_ascii_digit() {
                return Err(err());
            }
            out = out * 10 + u32::from(b - b'0');
        }
        Ok(out)
    };
    let sep = |index: usize, expected: u8| -> Result<(), ProjectionError> {
        if bytes.get(index) == Some(&expected) {
            Ok(())
        } else {
            Err(err())
        }
    };
    digits(0..4)?;
    sep(4, b'-')?;
    let month = digits(5..7)?;
    sep(7, b'-')?;
    let day = digits(8..10)?;
    sep(10, b'T')?;
    let hour = digits(11..13)?;
    sep(13, b':')?;
    let minute = digits(14..16)?;
    sep(16, b':')?;
    let second = digits(17..19)?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(err());
    }
    let mut index = 19;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let fraction = &value[start..index];
        if fraction.is_empty()
            || fraction.len() > 9
            || fraction.ends_with('0')
            || fraction.bytes().all(|b| b == b'0')
        {
            return Err(err());
        }
    }
    if bytes.get(index) != Some(&b'Z') || index + 1 != bytes.len() {
        return Err(err());
    }
    Ok(())
}

fn unescape_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

/// Remove one JSON Pointer path from `value` if present. Declared exclusions
/// that do not exist are a no-op (self fields like `signature` are excluded
/// "if present", section 10). Returns an error only for unusable pointers.
fn remove_pointer(value: &mut Value, pointer: &str) -> Result<(), ProjectionError> {
    if pointer.is_empty() || !pointer.starts_with('/') {
        return Err(ProjectionError::InvalidPointer(pointer.to_owned()));
    }
    let segments: Vec<String> = pointer
        .split('/')
        .skip(1)
        .map(unescape_pointer_segment)
        .collect();
    let (last, parents) = segments
        .split_last()
        .ok_or_else(|| ProjectionError::InvalidPointer(pointer.to_owned()))?;
    let mut current = value;
    for segment in parents {
        current = match current {
            Value::Object(map) => match map.get_mut(segment.as_str()) {
                Some(next) => next,
                None => return Ok(()),
            },
            Value::Array(items) => {
                let index: usize = match segment.parse() {
                    Ok(index) => index,
                    Err(_) => return Err(ProjectionError::InvalidPointer(pointer.to_owned())),
                };
                match items.get_mut(index) {
                    Some(next) => next,
                    None => return Ok(()),
                }
            }
            _ => return Ok(()),
        };
    }
    match current {
        Value::Object(map) => {
            map.remove(last.as_str());
            Ok(())
        }
        Value::Array(_) => Err(ProjectionError::InvalidPointer(pointer.to_owned())),
        _ => Ok(()),
    }
}

/// Digest projection (section 10): the value with ONLY the declared
/// `digest_excluded` paths removed. No other path may be dropped.
pub fn digest_projection(
    value: &Value,
    digest_excluded: &[&str],
) -> Result<Value, ProjectionError> {
    let mut projected = value.clone();
    for pointer in digest_excluded {
        remove_pointer(&mut projected, pointer)?;
    }
    Ok(projected)
}

/// Canonical bytes of the digest projection.
pub fn projection_canonical_bytes(
    value: &Value,
    digest_excluded: &[&str],
) -> Result<Vec<u8>, ProjectionError> {
    let projected = digest_projection(value, digest_excluded)?;
    Ok(canonical::canonical_bytes_of_value(&projected)?)
}

/// Content digest of the projection under the contract's domain.
pub fn projection_digest(
    value: &Value,
    digest_excluded: &[&str],
    domain: &str,
) -> Result<String, ProjectionError> {
    let bytes = projection_canonical_bytes(value, digest_excluded)?;
    Ok(canonical::digest(&bytes, domain)?)
}

/// Verify a self-referential content digest: read the declared digest at
/// `digest_pointer`, recompute the projection digest from the received
/// semantic value, and fail closed on any mismatch (sections 10 and 15).
pub fn verify_content_digest(
    value: &Value,
    digest_excluded: &[&str],
    domain: &str,
    digest_pointer: &str,
) -> Result<(), ProjectionError> {
    let declared = value
        .pointer(digest_pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| ProjectionError::MissingDigest(digest_pointer.to_owned()))?
        .to_owned();
    validate_digest_string(&declared)?;
    let computed = projection_digest(value, digest_excluded, domain)?;
    if declared == computed {
        Ok(())
    } else {
        Err(ProjectionError::DigestMismatch { declared, computed })
    }
}

/// Reject unknown critical extensions before any payload processing
/// (section 3; AKP envelope `extensions`; `CRITICAL_EXTENSION_UNKNOWN`).
/// An extension entry is `{id: string, critical: boolean}`; a malformed
/// entry cannot be verified and therefore fails closed as critical.
pub fn assert_no_unknown_critical_extensions(
    value: &Value,
    supported_ids: &[&str],
) -> Result<(), ProjectionError> {
    let Some(extensions) = value.get("extensions") else {
        return Ok(());
    };
    let Some(items) = extensions.as_array() else {
        return Err(ProjectionError::CriticalExtensionUnknown(
            "extensions is not an array".to_owned(),
        ));
    };
    for item in items {
        let id = item.get("id").and_then(Value::as_str);
        let critical = item.get("critical").and_then(Value::as_bool);
        match (id, critical) {
            (Some(id), Some(true)) if !supported_ids.contains(&id) => {
                return Err(ProjectionError::CriticalExtensionUnknown(id.to_owned()));
            }
            (Some(_), Some(_)) => {}
            // Fail closed: unverifiable entries are treated as critical.
            _ => {
                return Err(ProjectionError::CriticalExtensionUnknown(
                    "malformed extension entry".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn digest_string_form() {
        assert!(validate_digest_string(&format!("sha256:{}", "a".repeat(64))).is_ok());
        for bad in [
            format!("SHA256:{}", "a".repeat(64)),
            format!("sha256:{}", "A".repeat(64)),
            format!("sha256:{}", "a".repeat(63)),
            format!("sha512:{}", "a".repeat(64)),
            "a".repeat(64),
        ] {
            assert_eq!(
                validate_digest_string(&bad).unwrap_err().category(),
                "invalid-digest",
                "{bad} must be rejected"
            );
        }
    }

    #[test]
    fn timestamp_canonical_form() {
        for good in [
            "2026-07-19T11:02:03Z",
            "2026-07-19T11:02:03.1234Z",
            "0001-01-01T00:00:00Z",
            "2026-12-31T23:59:59.999999999Z",
        ] {
            assert!(validate_canonical_timestamp(good).is_ok(), "{good}");
        }
        for bad in [
            "2026-07-19T11:02:03+02:00",
            "2026-07-19 11:02:03Z",
            "2026-07-19T11:02:03",
            "2026-07-19t11:02:03Z",
            "2026-07-19T11:02:03z",
            "2026-07-19T23:59:60Z",
            "2026-07-19T11:02:03.1230Z",
            "2026-07-19T11:02:03.000Z",
            "2026-07-19T11:02:03.Z",
            "2026-07-19T11:02:03.1234567890Z",
        ] {
            assert_eq!(
                validate_canonical_timestamp(bad).unwrap_err().category(),
                "invalid-timestamp",
                "{bad} must be rejected"
            );
        }
    }

    #[test]
    fn projection_removes_only_declared_paths() {
        let value = serde_json::json!({
            "header": {"content_digest": "sha256:x", "id": "a"},
            "signature": "sig",
            "body": {"kept": true}
        });
        let projected =
            digest_projection(&value, &["/header/content_digest", "/signature"]).unwrap();
        assert_eq!(
            projected,
            serde_json::json!({"header": {"id": "a"}, "body": {"kept": true}})
        );
        // Absent declared path is a no-op; undeclared paths stay.
        let unchanged = digest_projection(&value, &["/not_present"]).unwrap();
        assert_eq!(unchanged, value);
    }

    #[test]
    fn verify_content_digest_round_trip_and_mismatch() {
        let excluded = ["/header/content_digest"];
        let domain = "governed-object-content/0.1";
        let mut value = serde_json::json!({
            "header": {"content_digest": "sha256:temp", "id": "a"},
            "body": 1
        });
        let digest = projection_digest(&value, &excluded, domain).unwrap();
        value["header"]["content_digest"] = Value::String(digest);
        assert!(verify_content_digest(&value, &excluded, domain, "/header/content_digest").is_ok());
        value["body"] = Value::from(2);
        assert_eq!(
            verify_content_digest(&value, &excluded, domain, "/header/content_digest")
                .unwrap_err()
                .category(),
            "digest-mismatch"
        );
    }

    #[test]
    fn critical_extension_gate() {
        let ok = serde_json::json!({
            "extensions": [
                {"id": "x-supported", "critical": true},
                {"id": "x-unknown-noncritical", "critical": false}
            ]
        });
        assert!(assert_no_unknown_critical_extensions(&ok, &["x-supported"]).is_ok());
        let unknown_critical = serde_json::json!({
            "extensions": [{"id": "x-unknown", "critical": true}]
        });
        assert_eq!(
            assert_no_unknown_critical_extensions(&unknown_critical, &["x-supported"])
                .unwrap_err()
                .category(),
            "critical-extension-unknown"
        );
        let malformed = serde_json::json!({"extensions": [{"id": "x-a"}]});
        assert_eq!(
            assert_no_unknown_critical_extensions(&malformed, &["x-a"])
                .unwrap_err()
                .category(),
            "critical-extension-unknown"
        );
    }
}
