//! Canonical JSON (`cognitiveos.canonical-json/0.1`) and domain-separated
//! digest / signature preimages.
//!
//! Implements `docs/standards/canonical-encoding-and-digest.md` sections 2-9
//! and 12 for the encoding layer: UTF-8 without BOM, I-JSON, RFC 8785 JCS
//! output, duplicate-member rejection, unsafe-integer rejection, and the
//! `CognitiveOS-Digest-V1` / `CognitiveOS-Signature-V1` preimages.
//!
//! Schema-bound rules (timestamp field contracts, digest projections, set
//! manifests) are enforced one level up, next to schema validation; they are
//! not part of this encoding-layer module.

use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use sha2::{Digest, Sha256};
use std::fmt;

/// I-JSON safe integer bound (RFC 7493): 2^53 - 1.
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// Exact ASCII prefix of every digest preimage (standard section 9).
pub const DIGEST_PREIMAGE_PREFIX: &[u8] = b"CognitiveOS-Digest-V1\n";

/// Exact ASCII prefix of every signature input (standard section 12).
pub const SIGNATURE_PREIMAGE_PREFIX: &[u8] = b"CognitiveOS-Signature-V1\n";

/// Rejection categories shared with the TypeScript implementation and the
/// golden fixtures under `tests/golden/`. String forms are fixture contract.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CanonicalError {
    /// Input bytes are not well-formed shortest-form UTF-8.
    #[error("invalid-utf8")]
    InvalidUtf8,
    /// Input begins with a U+FEFF byte order mark.
    #[error("bom")]
    Bom,
    /// An object contains the same member name twice.
    #[error("duplicate-member-name: {0}")]
    DuplicateMemberName(String),
    /// An exact integer literal outside the I-JSON safe range used a JSON number.
    #[error("unsafe-integer: {0}")]
    UnsafeInteger(String),
    /// Any other JSON grammar violation (NaN/Infinity literals, lone
    /// surrogates in escapes, trailing content, truncation, ...).
    #[error("invalid-json: {0}")]
    InvalidJson(String),
    /// Digest/signature domain label violates the registered grammar.
    #[error("invalid-domain: {0}")]
    InvalidDomain(String),
    /// Signature algorithm identifier is empty or not ASCII.
    #[error("invalid-algorithm: {0}")]
    InvalidAlgorithm(String),
    /// RFC 8785 serialization failed (non-finite number reached the encoder).
    #[error("canonicalization-failed: {0}")]
    CanonicalizationFailed(String),
}

impl CanonicalError {
    /// Stable machine category used by golden fixtures (`expected_rejection`).
    pub fn category(&self) -> &'static str {
        match self {
            CanonicalError::InvalidUtf8 => "invalid-utf8",
            CanonicalError::Bom => "bom",
            CanonicalError::DuplicateMemberName(_) => "duplicate-member-name",
            CanonicalError::UnsafeInteger(_) => "unsafe-integer",
            CanonicalError::InvalidJson(_) => "invalid-json",
            CanonicalError::InvalidDomain(_) => "invalid-domain",
            CanonicalError::InvalidAlgorithm(_) => "invalid-algorithm",
            CanonicalError::CanonicalizationFailed(_) => "canonicalization-failed",
        }
    }
}

// Marker prefixes smuggled through serde's custom error channel from the
// strict visitor, then mapped back to typed variants in `parse_strict`.
const DUP_MARKER: &str = "__COS_DUPLICATE__:";
const UNSAFE_MARKER: &str = "__COS_UNSAFE_INT__:";

/// Strict JSON value: preserves document member order, rejects duplicate
/// member names and unsafe integers at parse time.
#[derive(Debug, Clone, PartialEq)]
pub enum StrictValue {
    Null,
    Bool(bool),
    /// Exact integer within the I-JSON safe range.
    Int(i64),
    /// Any other finite binary64 number.
    Float(f64),
    String(String),
    Array(Vec<StrictValue>),
    /// Members in document order; JCS ordering is applied at serialization.
    Object(Vec<(String, StrictValue)>),
}

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictVisitor;

        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any strict I-JSON value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<StrictValue, E> {
                Ok(StrictValue::Bool(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<StrictValue, E> {
                if v.unsigned_abs() > MAX_SAFE_INTEGER as u64 {
                    return Err(de::Error::custom(format!("{UNSAFE_MARKER}{v}")));
                }
                Ok(StrictValue::Int(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<StrictValue, E> {
                if v > MAX_SAFE_INTEGER as u64 {
                    return Err(de::Error::custom(format!("{UNSAFE_MARKER}{v}")));
                }
                Ok(StrictValue::Int(v as i64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<StrictValue, E> {
                Ok(StrictValue::Float(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<StrictValue, E> {
                Ok(StrictValue::String(v.to_owned()))
            }

            fn visit_string<E>(self, v: String) -> Result<StrictValue, E> {
                Ok(StrictValue::String(v))
            }

            fn visit_unit<E>(self) -> Result<StrictValue, E> {
                Ok(StrictValue::Null)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<StrictValue, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element::<StrictValue>()? {
                    items.push(item);
                }
                Ok(StrictValue::Array(items))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<StrictValue, A::Error> {
                let mut members: Vec<(String, StrictValue)> = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    if members.iter().any(|(existing, _)| existing == &key) {
                        return Err(de::Error::custom(format!("{DUP_MARKER}{key}")));
                    }
                    let value = map.next_value::<StrictValue>()?;
                    members.push((key, value));
                }
                Ok(StrictValue::Object(members))
            }
        }

        deserializer.deserialize_any(StrictVisitor)
    }
}

impl Serialize for StrictValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            StrictValue::Null => serializer.serialize_unit(),
            StrictValue::Bool(b) => serializer.serialize_bool(*b),
            StrictValue::Int(i) => serializer.serialize_i64(*i),
            StrictValue::Float(f) => serializer.serialize_f64(*f),
            StrictValue::String(s) => serializer.serialize_str(s),
            StrictValue::Array(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            StrictValue::Object(members) => {
                let mut map = serializer.serialize_map(Some(members.len()))?;
                for (key, value) in members {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

/// Strictly parse UTF-8 bytes into a [`StrictValue`] (standard sections 3-5).
pub fn parse_strict_bytes(input: &[u8]) -> Result<StrictValue, CanonicalError> {
    let text = std::str::from_utf8(input).map_err(|_| CanonicalError::InvalidUtf8)?;
    parse_strict(text)
}

/// Reject exact-integer literals outside the I-JSON safe range before the
/// main parse. serde_json silently widens integer literals beyond u64/i64 to
/// binary64, which would violate standard section 5, so this lexical scan
/// runs first. String bodies (including escapes) are skipped; in valid JSON
/// every digit run outside a string belongs to a number literal.
fn reject_unsafe_integer_literals(input: &str) -> Result<(), CanonicalError> {
    const SAFE_DIGITS: usize = 16; // "9007199254740991".len()
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            match b {
                b'\\' => i += 2,
                b'"' => {
                    in_string = false;
                    i += 1;
                }
                _ => i += 1,
            }
            continue;
        }
        match b {
            b'"' => {
                in_string = true;
                i += 1;
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                while i < bytes.len()
                    && matches!(bytes[i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    i += 1;
                }
                let literal = &input[start..i];
                let is_integer_form = !literal.bytes().any(|c| matches!(c, b'.' | b'e' | b'E'));
                if is_integer_form {
                    let digits = literal.strip_prefix('-').unwrap_or(literal);
                    let too_big = digits.len() > SAFE_DIGITS
                        || (digits.len() == SAFE_DIGITS && digits > "9007199254740991");
                    if too_big {
                        return Err(CanonicalError::UnsafeInteger(literal.to_owned()));
                    }
                }
            }
            _ => i += 1,
        }
    }
    Ok(())
}

/// Strictly parse a JSON text into a [`StrictValue`].
pub fn parse_strict(input: &str) -> Result<StrictValue, CanonicalError> {
    if input.starts_with('\u{FEFF}') {
        return Err(CanonicalError::Bom);
    }
    reject_unsafe_integer_literals(input)?;
    serde_json::from_str::<StrictValue>(input).map_err(|err| {
        let msg = err.to_string();
        if let Some(pos) = msg.find(DUP_MARKER) {
            let rest = &msg[pos + DUP_MARKER.len()..];
            let name = rest.split(" at line").next().unwrap_or(rest);
            return CanonicalError::DuplicateMemberName(name.to_owned());
        }
        if let Some(pos) = msg.find(UNSAFE_MARKER) {
            let rest = &msg[pos + UNSAFE_MARKER.len()..];
            let lit = rest.split(" at line").next().unwrap_or(rest);
            return CanonicalError::UnsafeInteger(lit.to_owned());
        }
        CanonicalError::InvalidJson(msg)
    })
}

/// Produce RFC 8785 canonical bytes for an already strict-parsed value.
pub fn canonical_bytes(value: &StrictValue) -> Result<Vec<u8>, CanonicalError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|err| CanonicalError::CanonicalizationFailed(err.to_string()))
}

/// Strict-parse a JSON text and return its RFC 8785 canonical bytes.
pub fn canonicalize(input: &str) -> Result<Vec<u8>, CanonicalError> {
    canonical_bytes(&parse_strict(input)?)
}

/// Canonicalize an in-memory `serde_json::Value`. Duplicate member names are
/// unrepresentable in `Value`; unsafe integers are still rejected so a value
/// is never silently rounded before a digest (standard section 5).
pub fn canonical_bytes_of_value(value: &serde_json::Value) -> Result<Vec<u8>, CanonicalError> {
    reject_unsafe_value_numbers(value)?;
    serde_json_canonicalizer::to_vec(value)
        .map_err(|err| CanonicalError::CanonicalizationFailed(err.to_string()))
}

fn reject_unsafe_value_numbers(value: &serde_json::Value) -> Result<(), CanonicalError> {
    match value {
        serde_json::Value::Number(number) => {
            let unsafe_int = number
                .as_i64()
                .map(|i| i.unsigned_abs() > MAX_SAFE_INTEGER as u64)
                .or_else(|| number.as_u64().map(|u| u > MAX_SAFE_INTEGER as u64))
                .unwrap_or(false);
            if unsafe_int {
                return Err(CanonicalError::UnsafeInteger(number.to_string()));
            }
            Ok(())
        }
        serde_json::Value::Array(items) => items.iter().try_for_each(reject_unsafe_value_numbers),
        serde_json::Value::Object(members) => {
            members.values().try_for_each(reject_unsafe_value_numbers)
        }
        _ => Ok(()),
    }
}

/// Validate a digest/signature domain label (standard section 9).
pub fn validate_domain(domain: &str) -> Result<(), CanonicalError> {
    const FORBIDDEN: [&str; 3] = ["generic", "object", "payload"];
    let bytes = domain.as_bytes();
    let valid_head = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let valid_tail = |b: u8| valid_head(b) || b == b'.' || b == b'_' || b == b'/' || b == b'-';
    let grammar_ok = match bytes.split_first() {
        Some((head, tail)) => {
            bytes.len() <= 128 && valid_head(*head) && tail.iter().all(|b| valid_tail(*b))
        }
        None => false,
    };
    if !grammar_ok || FORBIDDEN.contains(&domain) {
        return Err(CanonicalError::InvalidDomain(domain.to_owned()));
    }
    Ok(())
}

/// Exact digest preimage: `"CognitiveOS-Digest-V1\n" || domain || 0x00 || C`.
pub fn digest_preimage(canonical: &[u8], domain: &str) -> Result<Vec<u8>, CanonicalError> {
    validate_domain(domain)?;
    let mut preimage =
        Vec::with_capacity(DIGEST_PREIMAGE_PREFIX.len() + domain.len() + 1 + canonical.len());
    preimage.extend_from_slice(DIGEST_PREIMAGE_PREFIX);
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0x00);
    preimage.extend_from_slice(canonical);
    Ok(preimage)
}

/// Domain-separated digest: `"sha256:" || lowercase_hex(SHA-256(preimage))`.
pub fn digest(canonical: &[u8], domain: &str) -> Result<String, CanonicalError> {
    let preimage = digest_preimage(canonical, domain)?;
    Ok(format!(
        "sha256:{}",
        lowercase_hex(&Sha256::digest(&preimage))
    ))
}

/// Exact signature input (standard section 12):
/// `"CognitiveOS-Signature-V1\n" || domain || 0x00 || algorithm || 0x00 || C`.
pub fn signature_input(
    canonical: &[u8],
    domain: &str,
    algorithm: &str,
) -> Result<Vec<u8>, CanonicalError> {
    validate_domain(domain)?;
    if algorithm.is_empty() || !algorithm.is_ascii() {
        return Err(CanonicalError::InvalidAlgorithm(algorithm.to_owned()));
    }
    let mut input = Vec::with_capacity(
        SIGNATURE_PREIMAGE_PREFIX.len() + domain.len() + algorithm.len() + 2 + canonical.len(),
    );
    input.extend_from_slice(SIGNATURE_PREIMAGE_PREFIX);
    input.extend_from_slice(domain.as_bytes());
    input.push(0x00);
    input.extend_from_slice(algorithm.as_bytes());
    input.push(0x00);
    input.extend_from_slice(canonical);
    Ok(input)
}

/// Convenience: strict parse + canonicalize + domain-separated digest.
pub fn digest_json(input: &str, domain: &str) -> Result<String, CanonicalError> {
    digest(&canonicalize(input)?, domain)
}

fn lowercase_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use fmt::Write;
        // Writing to a String cannot fail; ignore the Infallible result.
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_member_names() {
        let err = parse_strict(r#"{"a":1,"a":2}"#).unwrap_err();
        assert_eq!(err.category(), "duplicate-member-name");
    }

    #[test]
    fn rejects_unsafe_integers() {
        let err = parse_strict("9007199254740993").unwrap_err();
        assert_eq!(err.category(), "unsafe-integer");
        assert!(parse_strict("9007199254740991").is_ok());
        assert!(parse_strict("-9007199254740991").is_ok());
    }

    #[test]
    fn rejects_bom_and_invalid_utf8() {
        assert_eq!(parse_strict("\u{FEFF}{}").unwrap_err().category(), "bom");
        assert_eq!(
            parse_strict_bytes(&[0xFF, 0xFE]).unwrap_err().category(),
            "invalid-utf8"
        );
    }

    #[test]
    fn canonical_key_ordering_and_unicode() {
        let bytes = canonicalize(r#"{"b":2,"a":1,"\u20ac":"euro"}"#).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            "{\"a\":1,\"b\":2,\"\u{20ac}\":\"euro\"}"
        );
    }

    #[test]
    fn digest_uses_domain_separation() {
        let canonical = canonicalize("{}").unwrap();
        let d1 = digest(&canonical, "conformance-fixture/0.1").unwrap();
        let d2 = digest(&canonical, "schema-bundle/0.1").unwrap();
        assert_ne!(d1, d2);
        assert!(d1.starts_with("sha256:") && d1.len() == 7 + 64);
    }

    #[test]
    fn forbidden_domains_rejected() {
        let canonical = canonicalize("{}").unwrap();
        for domain in ["", "generic", "object", "payload", "UPPER", "-lead"] {
            assert_eq!(
                digest(&canonical, domain).unwrap_err().category(),
                "invalid-domain",
                "domain {domain:?} must be rejected"
            );
        }
    }
}
