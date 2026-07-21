//! Generated-binding integration with the canonical encoding layer
//! (ADR-0006 acceptance: generated types compile, round-trip real contract
//! instances, and preserve canonical bytes/digests).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::canonical;
use cognitive_contracts::generated::agent_installation::AgentInstallation;
use cognitive_contracts::generated::agent_package_manifest::AgentPackageManifest;
use cognitive_contracts::generated::effect::{Effect, EffectState};
use cognitive_contracts::generated::intent_interpretation::IntentInterpretation;
use cognitive_contracts::generated::management_action_proposal::ManagementActionProposal;
use cognitive_contracts::generated::object_reference::{ObjectReference, StrongReference};
use cognitive_contracts::generated::performance_report::PerformanceReport;
use cognitive_contracts::generated::privileged_management_session::PrivilegedManagementSession;
use cognitive_contracts::generated::profile_manifest::ProfileManifest;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// The migrated positive Effect instance: the legacy-strongref vector object
/// with its reference migrated to the ObjectReference strong shape (the same
/// instance the schema contract tests accept).
fn positive_effect_json() -> Value {
    let path = repo_root()
        .join("conformance")
        .join("vectors")
        .join("governed-object-legacy-strongref-001.json");
    let raw = fs::read_to_string(&path).unwrap();
    let vector: Value = serde_json::from_str(&raw).unwrap();
    let mut object = vector["input"]["object"].clone();
    object["intent_ref"] = serde_json::json!({
        "kind": "strong",
        "id": "01890a5d-ac96-774b-bcce-b302099a805d",
        "object_version": 1,
        "content_digest":
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });
    object
}

#[test]
fn generated_effect_round_trips_with_identical_canonical_digest() {
    let source = positive_effect_json();
    let typed: Effect = serde_json::from_value(source.clone()).expect("Effect must deserialize");
    assert_eq!(typed.state, EffectState::Proposed);
    assert_eq!(typed.header.schema_version, "cognitiveos.effect/0.2");

    let reserialized = serde_json::to_value(&typed).expect("Effect must serialize");
    let domain = "governed-object-content/0.1";
    let source_digest = canonical::digest(
        &canonical::canonical_bytes_of_value(&source).unwrap(),
        domain,
    )
    .unwrap();
    let typed_digest = canonical::digest(
        &canonical::canonical_bytes_of_value(&reserialized).unwrap(),
        domain,
    )
    .unwrap();
    assert_eq!(
        source_digest, typed_digest,
        "generated Effect binding must preserve canonical bytes (missing vs null, field set)"
    );
}

#[test]
fn generated_bindings_reject_unknown_and_legacy_shapes() {
    // Unknown member: deny_unknown_fields mirrors additionalProperties:false.
    let mut with_unknown = positive_effect_json();
    with_unknown["not_in_contract"] = Value::Bool(true);
    assert!(
        serde_json::from_value::<Effect>(with_unknown).is_err(),
        "unknown member must be rejected by the generated binding"
    );
    // Legacy strongRef shape must not deserialize as an ObjectReference.
    let legacy = serde_json::json!({
        "id": "intent://tenant-a/payments/int-0001",
        "version": 4,
        "digest": "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });
    assert!(
        serde_json::from_value::<ObjectReference>(legacy.clone()).is_err(),
        "legacy strongRef shape must not parse as ObjectReference"
    );
    assert!(
        serde_json::from_value::<StrongReference>(legacy).is_err(),
        "legacy strongRef shape must not parse as StrongReference"
    );
}

#[test]
fn effect_state_enum_is_exhaustive_against_transition_table() {
    let path = repo_root()
        .join("specs")
        .join("transitions")
        .join("effect.transitions.json");
    let table: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for state in table["states"].as_array().unwrap() {
        let literal = Value::String(state.as_str().unwrap().to_owned());
        let parsed: EffectState = serde_json::from_value(literal.clone())
            .unwrap_or_else(|_| panic!("state {state} missing from generated EffectState"));
        assert_eq!(serde_json::to_value(parsed).unwrap(), literal);
    }
}

#[test]
fn generated_headers_pin_current_schema_digests() {
    // Every schema-sourced generated file's schema_digest header line must
    // match the digest of the current source schema (ADR-0006 item 4,
    // verified without regenerating). The errors.yaml-sourced module is
    // covered by `error_registry_matches_errors_yaml` below.
    let generated_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated");
    let schema_dir = repo_root().join("specs").join("schemas");
    let mut checked = 0usize;
    for entry in fs::read_dir(&generated_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs")
            || path.file_name().is_some_and(|n| n == "mod.rs")
        {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let source_line = text
            .lines()
            .find(|l| l.starts_with("//! source: "))
            .unwrap_or_else(|| panic!("{} missing source header", path.display()));
        let Some(schema_rel) = source_line.strip_prefix("//! source: specs/schemas/") else {
            continue; // registry-sourced module (error_registry.rs)
        };
        let digest_line = text
            .lines()
            .find(|l| l.starts_with("//! schema_digest: "))
            .unwrap_or_else(|| panic!("{} missing digest header", path.display()));
        let raw = fs::read_to_string(schema_dir.join(schema_rel)).unwrap();
        let doc = canonical::parse_strict(&raw).unwrap();
        let expected = canonical::digest(
            &canonical::canonical_bytes(&doc).unwrap(),
            "schema-bundle/0.1",
        )
        .unwrap();
        assert!(
            digest_line.contains(&expected),
            "{}: header digest is stale (run contracts-codegen + cargo fmt)",
            path.display()
        );
        checked += 1;
    }
    assert!(checked >= 40, "generated module coverage shrank: {checked}");
}

#[test]
fn schema_digest_constants_match_live_schemas() {
    // Gap 5 of the 20260720 lane-tsc handoff: the digest is a RUNTIME
    // constant, not just a header comment. The aggregate table and the
    // per-module constants must equal the re-derived digest of the live
    // schema file (same recipe as the schema-bundle manifest per-asset
    // digest, so clients can pin envelope `schema_digest` without
    // re-deriving).
    let schema_dir = repo_root().join("specs").join("schemas");
    for (file, pinned) in cognitive_contracts::generated::SCHEMA_DIGESTS {
        let raw = fs::read_to_string(schema_dir.join(file)).unwrap();
        let doc = canonical::parse_strict(&raw).unwrap();
        let live = canonical::digest(
            &canonical::canonical_bytes(&doc).unwrap(),
            "schema-bundle/0.1",
        )
        .unwrap();
        assert_eq!(pinned, live, "{file}: SCHEMA_DIGESTS entry is stale");
    }
    assert_eq!(
        cognitive_contracts::generated::SCHEMA_DIGESTS.len(),
        40,
        "generated schema module count drifted"
    );
    // Per-module constants are the same values (spot checks across families).
    use cognitive_contracts::generated as g;
    let by_file: std::collections::BTreeMap<&str, &str> =
        g::SCHEMA_DIGESTS.iter().copied().collect();
    assert_eq!(
        g::effect::SCHEMA_DIGEST,
        by_file["effect.schema.json"],
        "per-module constant diverged from the aggregate"
    );
    assert_eq!(g::effect::SCHEMA_ID, "effect.schema.json");
    assert_eq!(
        g::akp_request_envelope::SCHEMA_DIGEST,
        by_file["akp-request-envelope.schema.json"]
    );
    assert_eq!(
        g::shell_action_proposal::SCHEMA_DIGEST,
        by_file["shell-action-proposal.schema.json"]
    );
}

#[test]
fn m5_consumer_bindings_are_exported_with_required_members() {
    let missing_required = serde_json::json!({});
    assert!(serde_json::from_value::<IntentInterpretation>(missing_required.clone()).is_err());
    assert!(
        serde_json::from_value::<PrivilegedManagementSession>(missing_required.clone()).is_err()
    );
    assert!(serde_json::from_value::<ManagementActionProposal>(missing_required).is_err());

    use cognitive_contracts::generated as g;
    let by_file: std::collections::BTreeMap<&str, &str> =
        g::SCHEMA_DIGESTS.iter().copied().collect();
    assert_eq!(
        g::intent_interpretation::SCHEMA_DIGEST,
        by_file[g::intent_interpretation::SCHEMA_ID]
    );
    assert_eq!(
        g::privileged_management_session::SCHEMA_DIGEST,
        by_file[g::privileged_management_session::SCHEMA_ID]
    );
    assert_eq!(
        g::management_action_proposal::SCHEMA_DIGEST,
        by_file[g::management_action_proposal::SCHEMA_ID]
    );
}

#[test]
fn m6_consumer_bindings_are_exported_with_required_members() {
    let missing_required = serde_json::json!({});
    assert!(serde_json::from_value::<AgentPackageManifest>(missing_required.clone()).is_err());
    assert!(serde_json::from_value::<AgentInstallation>(missing_required.clone()).is_err());
    assert!(serde_json::from_value::<PerformanceReport>(missing_required.clone()).is_err());
    assert!(serde_json::from_value::<ProfileManifest>(missing_required).is_err());

    use cognitive_contracts::generated as g;
    let by_file: std::collections::BTreeMap<&str, &str> =
        g::SCHEMA_DIGESTS.iter().copied().collect();
    for id in [
        g::agent_package_manifest::SCHEMA_ID,
        g::agent_installation::SCHEMA_ID,
        g::agent_compatibility_report::SCHEMA_ID,
        g::performance_report::SCHEMA_ID,
        g::profile_manifest::SCHEMA_ID,
    ] {
        assert!(
            by_file.contains_key(id),
            "M6 schema {id} missing from SCHEMA_DIGESTS"
        );
    }
    assert_eq!(
        g::agent_package_manifest::SCHEMA_DIGEST,
        by_file[g::agent_package_manifest::SCHEMA_ID]
    );
    assert_eq!(
        g::performance_report::SCHEMA_DIGEST,
        by_file[g::performance_report::SCHEMA_ID]
    );
}

#[test]
fn error_registry_matches_errors_yaml() {
    // Gap 2 of the 20260720 lane-tsc handoff: the generated registry table
    // must match specs/registry/errors.yaml entry by entry, and the pinned
    // REGISTRY_DIGEST must equal the spec-set manifest per-asset recipe
    // (canonical JSON projection of the parsed YAML, domain spec-set/0.1).
    use cognitive_contracts::bundle;
    use cognitive_contracts::generated::error_registry::{
        REGISTERED_ERRORS, REGISTRY_DIGEST, RegisteredErrorCode,
    };

    let raw = fs::read_to_string(
        repo_root()
            .join("specs")
            .join("registry")
            .join("errors.yaml"),
    )
    .unwrap();
    let value: Value = serde_yaml::from_str(&raw).unwrap();
    let live_digest = bundle::asset_content_digest(&value, bundle::SPEC_SET_DOMAIN).unwrap();
    assert_eq!(
        REGISTRY_DIGEST, live_digest,
        "REGISTRY_DIGEST is stale (run contracts-codegen + cargo fmt)"
    );

    let entries = value["errors"].as_array().unwrap();
    assert_eq!(
        REGISTERED_ERRORS.len(),
        entries.len(),
        "registry table count drifted from errors.yaml"
    );
    for (generated, registered) in REGISTERED_ERRORS.iter().zip(entries) {
        let code = registered["code"].as_str().unwrap();
        assert_eq!(generated.code.as_str(), code, "code order drifted");
        assert_eq!(
            serde_json::to_value(generated.category).unwrap(),
            registered["category"],
            "{code}: category drifted"
        );
        assert_eq!(
            generated.retryable,
            registered["retryable"].as_bool().unwrap(),
            "{code}: retryable drifted"
        );
        assert_eq!(
            generated.description,
            registered["description"]
                .as_str()
                .unwrap()
                .trim_end()
                .replace('\n', " "),
            "{code}: description drifted"
        );
        // Round trips: enum <-> wire string <-> table entry.
        assert_eq!(RegisteredErrorCode::parse(code), Some(generated.code));
        assert_eq!(generated.code.entry(), generated);
    }
    assert_eq!(RegisteredErrorCode::parse("NOT_A_REGISTERED_CODE"), None);
    // Contract-driven retry classification stays registry truth
    // (docs/standards/error-contract.md section 3).
    let retryable = |code: &str| {
        RegisteredErrorCode::parse(code)
            .map(|c| c.entry().retryable)
            .unwrap()
    };
    assert!(retryable("STATE_CONFLICT"));
    assert!(retryable("EFFECT_OUTCOME_UNKNOWN"));
    assert!(!retryable("EFFECT_IDEMPOTENCY_CONFLICT"));
    assert!(!retryable("CONTEXT_AUTH_DENIED"));
}
