//! Emit `{fixture id -> digest}` for every positive golden fixture as
//! canonical JSON on stdout. CI runs this and the TypeScript twin
//! (`packages/contracts-ts/dist/emit-golden.js`) and asserts the outputs are
//! byte-identical (cross-language digest equality gate).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cognitive_contracts::canonical::{canonical_bytes, canonicalize, digest, parse_strict};
use serde_json::Value;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("golden")
        .join("canonical-json-fixtures.json");
    let raw = std::fs::read_to_string(path).expect("fixture file must exist");
    let fixtures: Value = serde_json::from_str(&raw).expect("fixture file must be valid JSON");
    let domain = fixtures["digest_domain"].as_str().expect("digest_domain");

    let mut entries: Vec<(String, String)> = fixtures["positive"]
        .as_array()
        .expect("positive array")
        .iter()
        .map(|fixture| {
            let id = fixture["id"].as_str().expect("id").to_owned();
            let canonical = canonicalize(fixture["input_json"].as_str().expect("input_json"))
                .expect("fixture must canonicalize");
            (id, digest(&canonical, domain).expect("digest"))
        })
        .collect();
    entries.sort();

    let map: serde_json::Map<String, Value> = entries
        .into_iter()
        .map(|(id, dg)| (id, Value::String(dg)))
        .collect();
    // Re-canonicalize so both languages emit byte-identical output.
    let value = parse_strict(&serde_json::to_string(&Value::Object(map)).unwrap()).unwrap();
    let out = canonical_bytes(&value).unwrap();
    println!("{}", String::from_utf8(out).unwrap());
}
