//! Emit `{fixture id -> digest}` for every positive golden fixture, plus the
//! live schema-bundle manifest digest of `specs/schemas/`, as canonical JSON
//! on stdout. CI runs this and the TypeScript twin
//! (`packages/contracts-ts/dist/emit-golden.js`) and asserts the outputs are
//! byte-identical (cross-language digest equality gate).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cognitive_contracts::bundle::{
    BundleAsset, MEDIA_TYPE_SCHEMA_JSON, SCHEMA_BUNDLE_DOMAIN, SPEC_SUITE_VERSION, manifest_digest,
};
use cognitive_contracts::canonical::{canonical_bytes, canonicalize, digest, parse_strict};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Live schema-bundle manifest digest over the current specs/schemas suite
/// (registered section-13 procedure; twin logic in emit-golden.ts).
fn live_schema_bundle_digest(repo_root: &Path) -> String {
    let dir = repo_root.join("specs").join("schemas");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("specs/schemas must exist")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    let assets: Vec<BundleAsset> = files
        .iter()
        .map(|path| {
            let raw = std::fs::read_to_string(path).expect("schema readable");
            let value = parse_strict(&raw).expect("schema parses strictly");
            let bytes = canonical_bytes(&value).expect("schema canonicalizes");
            BundleAsset {
                id: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                version: SPEC_SUITE_VERSION.to_owned(),
                media_type: MEDIA_TYPE_SCHEMA_JSON.to_owned(),
                content_digest: digest(&bytes, SCHEMA_BUNDLE_DOMAIN).expect("asset digest"),
            }
        })
        .collect();
    manifest_digest(&assets, SCHEMA_BUNDLE_DOMAIN).expect("manifest digest")
}

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let path = repo_root
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
    entries.push((
        "live:schema-bundle-manifest".to_owned(),
        live_schema_bundle_digest(&repo_root),
    ));
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
