//! `cognitive-contracts`: machine-contract layer of the CognitiveOS reference
//! implementation.
//!
//! Scope: canonical JSON encoding, domain-separated digests and signature
//! preimages, strict parsing, the shared golden fixtures under
//! `tests/golden/`, and the schema-generated Rust bindings for the IMP-08
//! minimal core object set (`generated`, emitted by the committed
//! `contracts-codegen` binary per `docs/adr/0006-code-generation-policy.md`;
//! CI regenerates and diffs).
//!
//! Normative sources: `docs/standards/canonical-encoding-and-digest.md`
//! (`cognitiveos.canonical-json/0.1`), ADR-0004, ADR-0005, ADR-0006.

pub mod bundle;
pub mod canonical;
pub mod generated;
pub mod projection;

/// Encoding profile implemented by [`canonical`].
pub const ENCODING_PROFILE: &str = "cognitiveos.canonical-json/0.1";
