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
