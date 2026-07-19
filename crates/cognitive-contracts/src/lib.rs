//! `cognitive-contracts`: machine-contract layer of the CognitiveOS reference
//! implementation.
//!
//! Scope (M0): canonical JSON encoding, domain-separated digests and signature
//! preimages, strict parsing, and the shared golden fixtures under
//! `tests/golden/`. Schema-generated Rust bindings arrive with the codegen
//! pipeline (Lane-CTR, M1; see `docs/adr/0006-code-generation-policy.md`).
//!
//! Normative sources: `docs/standards/canonical-encoding-and-digest.md`
//! (`cognitiveos.canonical-json/0.1`), ADR-0004, ADR-0005.

pub mod canonical;

/// Encoding profile implemented by [`canonical`].
pub const ENCODING_PROFILE: &str = "cognitiveos.canonical-json/0.1";
