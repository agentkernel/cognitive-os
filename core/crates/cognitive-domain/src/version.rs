//! Logical object versions (`logical_version` clock domain, ADR-0005).
//!
//! Authority ordering uses logical versions, never wall timestamps or
//! UUIDv7 lexical order. CAS rules (REQ-STATE-003,
//! `state-and-transition-contract.md` section 3): a transition request fixes
//! `expected_version`; the authoritative commit advances the version by
//! exactly one; a mismatch is `STATE_CONFLICT`, never last-write-wins.

use crate::error::DomainError;
use cognitive_contracts::canonical::MAX_SAFE_INTEGER;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Authoritative logical version of a governed object. The first committed
/// version of an admitted object is 1 (`after_version` minimum in
/// `state-transition-request.schema.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Version(i64);

impl Version {
    /// Version of the initial admitted object state.
    pub const INITIAL: Version = Version(1);

    /// Parse a version from an integer; valid versions are `1..=2^53-1`
    /// (I-JSON safe range, canonical standard section 5).
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if (1..=MAX_SAFE_INTEGER).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidVersion(value))
        }
    }

    /// The integer value.
    pub fn get(self) -> i64 {
        self.0
    }

    /// The version an accepted transition commits: exactly `self + 1`.
    /// Fails instead of leaving the I-JSON safe integer range.
    pub fn next(self) -> Result<Self, DomainError> {
        if self.0 >= MAX_SAFE_INTEGER {
            Err(DomainError::VersionOverflow(self.0))
        } else {
            Ok(Self(self.0 + 1))
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn version_range_is_positive_safe_integers() {
        assert!(Version::new(1).is_ok());
        assert!(Version::new(MAX_SAFE_INTEGER).is_ok());
        assert!(Version::new(0).is_err());
        assert!(Version::new(-1).is_err());
        assert!(Version::new(MAX_SAFE_INTEGER + 1).is_err());
    }

    #[test]
    fn next_advances_by_exactly_one_and_fails_at_the_safe_bound() {
        assert_eq!(Version::INITIAL.next().unwrap().get(), 2);
        let at_bound = Version::new(MAX_SAFE_INTEGER).unwrap();
        assert_eq!(
            at_bound.next().unwrap_err(),
            DomainError::VersionOverflow(MAX_SAFE_INTEGER)
        );
    }
}
