//! Generated-binding integration with the canonical encoding layer
//! (ADR-0006 acceptance: generated types compile, round-trip real contract
//! instances, and preserve canonical bytes/digests).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::canonical;
use cognitive_contracts::generated::effect::{Effect, EffectState};
use cognitive_contracts::generated::object_reference::{ObjectReference, StrongReference};
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
    // Every generated file's schema_digest header line must match the digest
    // of the current source schema (ADR-0006 item 4, verified without
    // regenerating).
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
        let digest_line = text
            .lines()
            .find(|l| l.starts_with("//! schema_digest: "))
            .unwrap_or_else(|| panic!("{} missing digest header", path.display()));
        let schema_rel = source_line.trim_start_matches("//! source: specs/schemas/");
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
    assert!(checked >= 19, "generated module coverage shrank: {checked}");
}
