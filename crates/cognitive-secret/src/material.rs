//! Opaque secret material and references. Debug/Display never reveal bytes.

use crate::error::SecretError;
use std::fmt;
/// Stable opaque handle returned by put/replace. Safe to persist in config as a
/// reference only; it never contains the secret bytes.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SecretRef {
    value: String,
}

impl SecretRef {
    /// Builds a ref from a backend-local opaque token.
    pub fn from_opaque(value: impl Into<String>) -> Result<Self, SecretError> {
        let value = value.into();
        if value.is_empty() || value.len() > 256 {
            return Err(SecretError::InvalidAttributes {
                detail: "secret_ref must be 1..=256 bytes",
            });
        }
        if !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'/' | b'-' | b'_' | b'.')
        }) {
            return Err(SecretError::InvalidAttributes {
                detail: "secret_ref has unsupported characters",
            });
        }
        Ok(Self { value })
    }

    /// Backend-local opaque token. Not a secret, but also not a user display string.
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SecretRef({:?})", self.value)
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}

/// Non-secret label used by desktop secret collections for operator UX only.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretLabel {
    value: String,
}

impl SecretLabel {
    /// Validates a short non-empty label. Labels must not be used to carry secrets.
    pub fn new(value: impl Into<String>) -> Result<Self, SecretError> {
        let value = value.into();
        if value.is_empty() || value.len() > 128 {
            return Err(SecretError::InvalidAttributes {
                detail: "label must be 1..=128 bytes",
            });
        }
        if value.chars().any(|ch| ch.is_control()) {
            return Err(SecretError::InvalidAttributes {
                detail: "label must not contain control characters",
            });
        }
        Ok(Self { value })
    }

    /// Borrow the validated label text.
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for SecretLabel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SecretLabel({:?})", self.value)
    }
}

/// Non-secret lookup attributes that identify a secret item.
///
/// Attribute values are identifiers (provider id, purpose), never API keys.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SecretAttributes {
    pairs: Vec<(String, String)>,
}

impl SecretAttributes {
    /// Builds attributes from ordered key/value pairs after validation.
    pub fn from_pairs(pairs: Vec<(String, String)>) -> Result<Self, SecretError> {
        if pairs.is_empty() {
            return Err(SecretError::InvalidAttributes {
                detail: "at least one attribute pair is required",
            });
        }
        if pairs.len() > 16 {
            return Err(SecretError::InvalidAttributes {
                detail: "at most 16 attribute pairs are allowed",
            });
        }
        for (key, value) in &pairs {
            validate_attribute_token(key)?;
            validate_attribute_token(value)?;
        }
        Ok(Self { pairs })
    }

    /// Borrow the validated attribute pairs.
    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }
}

fn validate_attribute_token(token: &str) -> Result<(), SecretError> {
    if token.is_empty() || token.len() > 64 {
        return Err(SecretError::InvalidAttributes {
            detail: "attribute token length out of range",
        });
    }
    if !token
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(SecretError::InvalidAttributes {
            detail: "attribute token has unsupported characters",
        });
    }
    Ok(())
}

/// Zeroizing secret bytes. Debug and Display are always redacted.
#[derive(Clone)]
pub struct SecretMaterial {
    bytes: Vec<u8>,
}

impl Drop for SecretMaterial {
    fn drop(&mut self) {
        // Best-effort wipe without external crates. Not a formal side-channel claim.
        for byte in &mut self.bytes {
            *byte = 0;
        }
        self.bytes.clear();
    }
}

impl SecretMaterial {
    /// Captures secret bytes. Prefer constructing from a temporary buffer that
    /// the caller will drop; this type zeroizes on drop.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, SecretError> {
        let bytes = bytes.into();
        if bytes.is_empty() {
            return Err(SecretError::InvalidAttributes {
                detail: "secret material must be non-empty",
            });
        }
        if bytes.len() > 16 * 1024 {
            return Err(SecretError::InvalidAttributes {
                detail: "secret material exceeds 16 KiB limit",
            });
        }
        Ok(Self { bytes })
    }

    /// Controlled read access for backends that must write the secret once.
    pub fn expose_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Byte length only; never use this to reconstruct content.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// True when no bytes are held (should not occur for validated material).
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "SecretMaterial(redacted,len={})",
            self.bytes.len()
        )
    }
}

impl fmt::Display for SecretMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "<redacted-secret len={}>", self.bytes.len())
    }
}

impl PartialEq for SecretMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for SecretMaterial {}
