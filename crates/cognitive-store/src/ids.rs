//! UUIDv7 identifier source (RFC 9562, ADR-0005), adapter layer.
//!
//! Uses the `uuid` crate's v7 generator: cryptographically secure random
//! bits from the OS, RFC 9562 timestamp layout, lowercase canonical text
//! form. UUIDv7 timestamp bits remain an ID-generation hint only; nothing
//! in the kernel reads them back as time or order proof.

use cognitive_kernel::ports::{IdGenerator, PortFailure};

/// RFC 9562 UUIDv7 generator.
#[derive(Debug, Default, Clone, Copy)]
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        Ok(uuid::Uuid::now_v7().as_hyphenated().to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn generates_lowercase_canonical_v7_ids() {
        let generated = UuidV7Generator.next_uuid_v7().unwrap();
        // The domain newtype enforces the lowercase canonical UUIDv7 form.
        assert!(
            cognitive_domain::ObjectId::parse(&generated).is_ok(),
            "{generated}"
        );
    }

    #[test]
    fn ids_are_unique_across_a_burst() {
        let mut seen = BTreeSet::new();
        for _ in 0..1_000 {
            assert!(seen.insert(UuidV7Generator.next_uuid_v7().unwrap()));
        }
    }
}
