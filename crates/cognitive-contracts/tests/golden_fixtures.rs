//! Golden fixture verification (docs/standards/canonical-encoding-and-digest.md
//! section 14). The same fixture file is verified by the TypeScript twin in
//! `packages/contracts-ts/src/golden.test.ts`; CI additionally diffs the
//! emitted digest maps of both implementations.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::canonical::{
    canonicalize, digest, parse_strict, parse_strict_bytes, signature_input,
};
use serde_json::Value;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("golden")
        .join("canonical-json-fixtures.json")
}

fn load_fixtures() -> Value {
    let raw = std::fs::read_to_string(fixture_path()).expect("fixture file must exist");
    serde_json::from_str(&raw).expect("fixture file must be valid JSON")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn positive_fixtures_produce_identical_canonical_bytes_and_digests() {
    let fixtures = load_fixtures();
    assert_eq!(
        fixtures["encoding_profile"].as_str().unwrap(),
        "cognitiveos.canonical-json/0.1"
    );
    let domain = fixtures["digest_domain"].as_str().unwrap();
    let positive = fixtures["positive"].as_array().unwrap();
    assert!(positive.len() >= 10, "fixture coverage shrank");
    for fixture in positive {
        let id = fixture["id"].as_str().unwrap();
        let input = fixture["input_json"].as_str().unwrap();
        let canonical = canonicalize(input)
            .unwrap_or_else(|err| panic!("fixture {id} failed to canonicalize: {err}"));
        assert_eq!(
            String::from_utf8(canonical.clone()).unwrap(),
            fixture["expected_canonical_text"].as_str().unwrap(),
            "canonical text mismatch for {id}"
        );
        assert_eq!(
            digest(&canonical, domain).unwrap(),
            fixture["expected_digest"].as_str().unwrap(),
            "digest mismatch for {id}"
        );
        if let Some(signature) = fixture.get("signature") {
            let sig_input = signature_input(
                &canonical,
                signature["domain"].as_str().unwrap(),
                signature["algorithm"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(
                hex(&sig_input),
                signature["expected_signature_input_hex"].as_str().unwrap(),
                "signature input mismatch for {id}"
            );
        }
    }
}

#[test]
fn negative_fixtures_are_rejected_with_the_expected_category() {
    let fixtures = load_fixtures();
    let negative = fixtures["negative"].as_array().unwrap();
    assert!(negative.len() >= 6, "fixture coverage shrank");
    for fixture in negative {
        let id = fixture["id"].as_str().unwrap();
        let expected = fixture["expected_rejection"].as_str().unwrap();
        let result = if let Some(bytes_hex) = fixture.get("input_bytes_hex").and_then(Value::as_str)
        {
            let bytes: Vec<u8> = (0..bytes_hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&bytes_hex[i..i + 2], 16).unwrap())
                .collect();
            parse_strict_bytes(&bytes).map(|_| ())
        } else {
            parse_strict(fixture["input_json"].as_str().unwrap()).map(|_| ())
        };
        let err = result.expect_err(&format!("fixture {id} was not rejected"));
        assert_eq!(err.category(), expected, "category mismatch for {id}");
    }
}
