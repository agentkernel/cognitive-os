//! P8-T02/D03: Lane-CTR `agent-adapter-manifest` registration negatives.
//!
//! Schema/binding checks only — not conformance behavior execution and not a
//! Gate/release/Profile claim.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use jsonschema::{Retrieve, Uri};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "agent-adapter-manifest.schema.json";

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

fn validator(docs: &HashMap<String, Value>) -> jsonschema::Validator {
    jsonschema::options()
        .with_retriever(Retriever {
            schemas: docs.clone(),
        })
        .should_validate_formats(true)
        .build(
            docs.get(MANIFEST)
                .unwrap_or_else(|| panic!("{MANIFEST} must be registered")),
        )
        .unwrap_or_else(|error| panic!("{MANIFEST} must compile: {error}"))
}

fn valid_manifest() -> Value {
    json!({
        "schema_version": "cognitiveos.agent-adapter-manifest/0.1",
        "adapter_id": "adapter.example.cli",
        "protocol": "akp-http-json-sse",
        "candidate_only": true,
        "public_listener": false,
        "authority_writer": false,
        "discovery_card_digest": format!("sha256:{}", "a".repeat(64)),
        "declaration_digest": format!("sha256:{}", "b".repeat(64)),
        "discovery": {
            "name": "example",
            "description": "local discovery metadata only",
            "version": "0.1.0"
        }
    })
}

#[test]
fn agent_adapter_manifest_accepts_candidate_only_akp_shape() {
    let docs = schemas();
    let schema = docs
        .get(MANIFEST)
        .unwrap_or_else(|| panic!("{MANIFEST} must exist"));
    assert_eq!(schema["$id"], MANIFEST, "$id policy");
    assert_eq!(schema["additionalProperties"], false);

    let compiled = validator(&docs);
    assert!(
        compiled.is_valid(&valid_manifest()),
        "valid AKP candidate-only manifest must validate"
    );
}

#[test]
fn agent_adapter_manifest_rejects_public_listener_authority_writer_and_non_candidate() {
    let compiled = validator(&schemas());

    let mut public_listener = valid_manifest();
    public_listener["public_listener"] = json!(true);
    assert!(
        !compiled.is_valid(&public_listener),
        "public_listener=true must fail closed"
    );

    let mut authority_writer = valid_manifest();
    authority_writer["authority_writer"] = json!(true);
    assert!(
        !compiled.is_valid(&authority_writer),
        "authority_writer=true must fail closed"
    );

    let mut non_candidate = valid_manifest();
    non_candidate["candidate_only"] = json!(false);
    assert!(
        !compiled.is_valid(&non_candidate),
        "candidate_only=false must fail closed"
    );

    let mut wrong_protocol = valid_manifest();
    wrong_protocol["protocol"] = json!("a2a-public-http");
    assert!(
        !compiled.is_valid(&wrong_protocol),
        "non-AKP protocol must fail closed"
    );
}
