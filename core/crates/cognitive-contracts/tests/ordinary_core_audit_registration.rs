//! Registration-stage static checks for the Ordinary Core `status.inspect`
//! AUDIT contract.  These are schema/binding checks only, never conformance
//! behavior execution.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use cognitive_contracts::canonical;
use jsonschema::{Retrieve, Uri};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const DECISION: &str = "privileged-read-decision.schema.json";
const RECEIPT: &str = "audit-commit-receipt.schema.json";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn schemas() -> HashMap<String, Value> {
    let dir = repo_root().join("specs").join("schemas");
    let mut docs = HashMap::new();
    for entry in fs::read_dir(&dir).expect("read schemas") {
        let path = entry.expect("schema entry").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let name = path
                .file_name()
                .expect("schema name")
                .to_string_lossy()
                .into_owned();
            docs.insert(
                name,
                serde_json::from_slice(&fs::read(path).expect("schema bytes"))
                    .expect("schema JSON"),
            );
        }
    }
    docs
}

struct Retriever {
    schemas: HashMap<String, Value>,
}

impl Retrieve for Retriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let path = uri.path().as_str();
        let name = path.rsplit('/').next().unwrap_or(path);
        self.schemas
            .get(name)
            .cloned()
            .ok_or_else(|| format!("missing schema {name}").into())
    }
}

fn validator(docs: &HashMap<String, Value>, name: &str) -> jsonschema::Validator {
    jsonschema::options()
        .with_retriever(Retriever {
            schemas: docs.clone(),
        })
        .should_validate_formats(true)
        .build(
            docs.get(name)
                .unwrap_or_else(|| panic!("{name} must be registered")),
        )
        .unwrap_or_else(|error| panic!("{name} must compile: {error}"))
}

#[derive(Deserialize)]
struct ErrorRegistry {
    errors: Vec<ErrorEntry>,
}
#[derive(Deserialize)]
struct ErrorEntry {
    code: String,
}

fn registered_error_codes() -> BTreeSet<String> {
    serde_yaml::from_slice::<ErrorRegistry>(
        &fs::read(repo_root().join("specs/registry/errors.yaml")).expect("errors registry"),
    )
    .expect("parse errors registry")
    .errors
    .into_iter()
    .map(|entry| entry.code)
    .collect()
}

fn decision_fixture(outcome: &str) -> Value {
    let mut value = json!({
        "record_kind": "privileged_read_decision",
        "record_id": "01890a5d-ac96-774b-bcce-b302099a805d",
        "request_digest": format!("sha256:{}", "a".repeat(64)),
        "outcome": outcome,
        "observed_at": "2026-07-23T00:00:00Z"
    });
    if outcome == "success" {
        value["result_digest"] = json!(format!("sha256:{}", "b".repeat(64)));
    } else {
        value["safe_reason"] = json!("CONTEXT_AUTH_DENIED");
    }
    value
}

#[test]
fn registered_audit_schemas_are_closed_and_candidate_equivalent() {
    let docs = schemas();
    for name in [DECISION, RECEIPT] {
        let schema = docs
            .get(name)
            .unwrap_or_else(|| panic!("{name} must exist"));
        assert_eq!(schema["$id"], name, "{name} $id policy");
        assert_eq!(
            schema["additionalProperties"], false,
            "{name} must reject unknown fields"
        );
    }
    let decision = docs.get(DECISION).unwrap();
    let candidate: Value = serde_json::from_slice(&fs::read(repo_root().join("docs/plan/candidates/v02-ordinary-core-audit/privileged-read-decision.candidate.schema.json")).unwrap()).unwrap();
    assert_eq!(decision["required"], candidate["required"]);
    assert_eq!(decision["properties"], candidate["properties"]);
    assert_eq!(decision["allOf"], candidate["allOf"]);
    let receipt = docs.get(RECEIPT).unwrap();
    let candidate_receipt: Value = serde_json::from_slice(&fs::read(repo_root().join("docs/plan/candidates/v02-ordinary-core-audit/audit-commit-receipt.candidate.schema.json")).unwrap()).unwrap();
    assert_eq!(receipt["required"], candidate_receipt["required"]);
    assert_eq!(receipt["properties"], candidate_receipt["properties"]);
}

#[test]
fn registered_audit_decision_and_receipt_reject_negative_shapes() {
    let docs = schemas();
    let decision = validator(&docs, DECISION);
    let receipt = validator(&docs, RECEIPT);
    let safe_reason: BTreeSet<String> = docs[DECISION]["properties"]["safe_reason"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        safe_reason,
        registered_error_codes(),
        "safe_reason must close the public registry exactly"
    );
    for outcome in ["success", "denied", "error"] {
        assert!(decision.is_valid(&decision_fixture(outcome)));
    }
    let mut success_without_digest = decision_fixture("success");
    success_without_digest
        .as_object_mut()
        .unwrap()
        .remove("result_digest");
    assert!(!decision.is_valid(&success_without_digest));
    let mut success_with_reason = decision_fixture("success");
    success_with_reason["safe_reason"] = json!("CONTEXT_AUTH_DENIED");
    assert!(!decision.is_valid(&success_with_reason));
    let mut denied_with_result = decision_fixture("denied");
    denied_with_result["result_digest"] = json!(format!("sha256:{}", "c".repeat(64)));
    assert!(!decision.is_valid(&denied_with_result));
    let mut unregistered = decision_fixture("error");
    unregistered["safe_reason"] = json!("UNREGISTERED_AUDIT_REASON");
    assert!(!decision.is_valid(&unregistered));
    for forbidden in ["domain", "object_id", "tenant", "exists", "unexpected"] {
        let mut value = decision_fixture("error");
        value[forbidden] = json!("must-reject");
        assert!(!decision.is_valid(&value), "{forbidden}");
    }
    let valid_receipt = json!({"record_id":"01890a5d-ac96-774b-bcce-b302099a805d","record_digest":format!("sha256:{}", "d".repeat(64)),"request_digest":format!("sha256:{}", "a".repeat(64)),"sequence":1,"writer_epoch":1,"committed_at":"2026-07-23T00:00:01Z"});
    assert!(receipt.is_valid(&valid_receipt));
    for field in ["record_id", "record_digest", "request_digest"] {
        let mut value = valid_receipt.clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(!receipt.is_valid(&value));
    }
    for field in ["sequence", "writer_epoch"] {
        let mut value = valid_receipt.clone();
        value[field] = json!(0);
        assert!(!receipt.is_valid(&value));
    }
    let mut unknown = valid_receipt;
    unknown["unexpected"] = json!(true);
    assert!(!receipt.is_valid(&unknown));
}

#[test]
fn registered_audit_schema_digests_are_reproducible() {
    for name in [DECISION, RECEIPT] {
        let raw = fs::read_to_string(repo_root().join("specs/schemas").join(name)).unwrap();
        let parsed = canonical::parse_strict(&raw).unwrap();
        let digest = canonical::digest(
            &canonical::canonical_bytes(&parsed).unwrap(),
            "schema-bundle/0.1",
        )
        .unwrap();
        assert!(digest.starts_with("sha256:"));
    }
}
