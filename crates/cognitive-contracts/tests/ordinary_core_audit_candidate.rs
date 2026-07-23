//! Review-only Ordinary Core AUDIT candidate verification.
//!
//! This test deliberately reads `docs/plan/candidates/`, never `specs/`. It is
//! a freeze reproducibility check, not a conformance runner or machine-asset
//! registration path.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use cognitive_contracts::canonical;
use jsonschema::Validator;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const CANDIDATE_DOMAIN: &str = "ordinary-core-audit-candidate-file/0.2";

fn candidate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("plan")
        .join("candidates")
        .join("v02-ordinary-core-audit")
}

fn json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn validator(path: &Path) -> Validator {
    jsonschema::options()
        .should_validate_formats(true)
        .build(&json(path))
        .unwrap_or_else(|e| panic!("compile {}: {e}", path.display()))
}

#[test]
fn ordinary_core_audit_candidate_schemas_close_real_fields_and_terminal_outcomes() {
    let root = candidate_root();
    let decision = validator(&root.join("privileged-read-decision.candidate.schema.json"));
    let receipt = validator(&root.join("audit-commit-receipt.candidate.schema.json"));
    let fixtures = json(&root.join("fixtures.json"));

    for case in ["success", "denied", "error"] {
        assert!(
            decision.is_valid(&fixtures[case]["record"]),
            "{case} record must validate"
        );
        assert!(
            receipt.is_valid(&fixtures[case]["receipt"]),
            "{case} receipt must validate"
        );
    }

    let mut success = fixtures["success"]["record"].clone();
    success["safe_reason"] = Value::String("CONTEXT_AUTH_DENIED".to_owned());
    assert!(
        !decision.is_valid(&success),
        "success must not expose a denial/error reason"
    );

    let mut denied = fixtures["denied"]["record"].clone();
    denied["result_digest"] = Value::String(
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
    );
    assert!(
        !decision.is_valid(&denied),
        "denied must not include a result digest"
    );

    let mut error = fixtures["error"]["record"].clone();
    error["object_id"] = Value::String("must-not-appear".to_owned());
    assert!(
        !decision.is_valid(&error),
        "raw selectors/object identity must be rejected"
    );
}

#[test]
fn ordinary_core_audit_candidate_manifest_is_reproducible() {
    let root = candidate_root();
    let manifest = json(&root.join("candidate-manifest.json"));
    assert_eq!(manifest["status"], "review-only-candidate");
    assert_eq!(manifest["machine_registration"], "no");

    for entry in manifest["files"].as_array().expect("files array") {
        let relative = entry["path"].as_str().expect("path");
        let bytes =
            fs::read(root.join(relative)).unwrap_or_else(|e| panic!("read {relative}: {e}"));
        let raw_sha = format!("sha256:{:x}", Sha256::digest(&bytes));

        let value: Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|e| panic!("candidate JSON {relative}: {e}"));
        let canonical = canonical::canonical_bytes_of_value(&value)
            .unwrap_or_else(|e| panic!("canonicalize {relative}: {e}"));
        let digest = canonical::digest(&canonical, CANDIDATE_DOMAIN)
            .unwrap_or_else(|e| panic!("digest {relative}: {e}"));
        eprintln!(
            "candidate {relative}: bytes={} raw_sha256={raw_sha} canonical_digest={digest}",
            bytes.len()
        );
        assert_eq!(
            entry["byte_length"].as_u64(),
            Some(bytes.len() as u64),
            "length {relative}"
        );
        assert_eq!(
            entry["raw_sha256"].as_str(),
            Some(raw_sha.as_str()),
            "raw SHA-256 {relative}"
        );
        assert_eq!(
            entry["canonical_digest"].as_str(),
            Some(digest.as_str()),
            "canonical digest {relative}"
        );
        assert_eq!(entry["digest_domain"].as_str(), Some(CANDIDATE_DOMAIN));
    }
}
