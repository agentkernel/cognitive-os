//! Set and bundle digests (registered procedure).
//!
//! Implements `docs/standards/canonical-encoding-and-digest.md` section 13:
//! a specification set or schema bundle is digested over a CANONICAL LOGICAL
//! MANIFEST — one entry per logical asset carrying asset ID, version, media
//! type, and full content digest, in deterministic sorted order — never over
//! a variable archive representation. Nested (per-asset) digests are
//! computed first; the manifest digest then pins the whole set.
//!
//! Domains: `schema-bundle/0.1` for the schema bundle, `spec-set/0.1` for
//! specification sets (section 9 registered examples). Per-asset content
//! digests use their bundle's own domain (documented in
//! `docs/standards/conformance-evidence.md` section 6).
//!
//! This replaces the provisional M0 digest recipe that hashed a bare
//! `{id, content_digest}` list. The TypeScript twin lives in
//! `packages/contracts-ts/src/bundle.ts`; cross-language byte identity is
//! held by the golden fixtures (`tests/golden/`) and the emit-golden CI gate.

use crate::canonical;
use serde_json::Value;

/// Registered digest domain for schema bundles (standard section 9/13).
pub const SCHEMA_BUNDLE_DOMAIN: &str = "schema-bundle/0.1";

/// Registered digest domain for specification sets (standard section 9/13).
pub const SPEC_SET_DOMAIN: &str = "spec-set/0.1";

/// Media type recorded for JSON Schema assets.
pub const MEDIA_TYPE_SCHEMA_JSON: &str = "application/schema+json";

/// Media type recorded for plain JSON assets (transition tables).
pub const MEDIA_TYPE_JSON: &str = "application/json";

/// Media type recorded for YAML registry assets (digested over their
/// canonical JSON projection).
pub const MEDIA_TYPE_YAML: &str = "application/yaml";

/// Suite-level SemVer applied to every asset of the v0.1 draft suite.
/// The registered assets do not yet carry per-asset SemVer (drift D-011,
/// findings ledger); the suite machine version is applied per asset until a
/// per-asset version is registered.
pub const SPEC_SUITE_VERSION: &str = "0.1.0-draft.1";

/// One logical asset of a set or bundle manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleAsset {
    /// Logical asset ID. Schema assets use the schema file name (== `$id`);
    /// other spec assets use their repo-relative path.
    pub id: String,
    /// Asset SemVer (currently [`SPEC_SUITE_VERSION`] for every asset).
    pub version: String,
    /// Asset media type.
    pub media_type: String,
    /// Full domain-separated content digest of the asset's canonical bytes.
    pub content_digest: String,
}

/// Errors of the manifest construction procedure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BundleError {
    /// Two assets share one logical ID: the manifest would be ambiguous.
    #[error("duplicate-asset-id: {0}")]
    DuplicateAssetId(String),
    /// The manifest must cover at least one asset.
    #[error("empty-bundle")]
    EmptyBundle,
    /// Canonicalization or digest failure (propagated).
    #[error(transparent)]
    Canonical(#[from] canonical::CanonicalError),
}

/// Build the canonical logical manifest value: `{"assets": [...]}` with one
/// `{content_digest, id, media_type, version}` entry per asset, sorted by
/// asset ID (deterministic sorted order per standard section 13).
pub fn manifest_value(assets: &[BundleAsset]) -> Result<Value, BundleError> {
    if assets.is_empty() {
        return Err(BundleError::EmptyBundle);
    }
    let mut sorted: Vec<&BundleAsset> = assets.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    for pair in sorted.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(BundleError::DuplicateAssetId(pair[0].id.clone()));
        }
    }
    let entries: Vec<Value> = sorted
        .iter()
        .map(|asset| {
            serde_json::json!({
                "id": asset.id,
                "version": asset.version,
                "media_type": asset.media_type,
                "content_digest": asset.content_digest,
            })
        })
        .collect();
    Ok(serde_json::json!({ "assets": entries }))
}

/// Canonical bytes of the manifest (RFC 8785 over the manifest value).
pub fn manifest_canonical_bytes(assets: &[BundleAsset]) -> Result<Vec<u8>, BundleError> {
    let value = manifest_value(assets)?;
    Ok(canonical::canonical_bytes_of_value(&value)?)
}

/// Manifest digest under the given registered bundle domain.
pub fn manifest_digest(assets: &[BundleAsset], domain: &str) -> Result<String, BundleError> {
    let bytes = manifest_canonical_bytes(assets)?;
    Ok(canonical::digest(&bytes, domain)?)
}

/// Per-asset content digest: canonical bytes of the parsed asset value under
/// the bundle's own domain (nested digest, verified first per section 13).
pub fn asset_content_digest(asset_value: &Value, domain: &str) -> Result<String, BundleError> {
    let bytes = canonical::canonical_bytes_of_value(asset_value)?;
    Ok(canonical::digest(&bytes, domain)?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn asset(id: &str, digest: &str) -> BundleAsset {
        BundleAsset {
            id: id.to_owned(),
            version: SPEC_SUITE_VERSION.to_owned(),
            media_type: MEDIA_TYPE_SCHEMA_JSON.to_owned(),
            content_digest: digest.to_owned(),
        }
    }

    #[test]
    fn manifest_is_sorted_and_deterministic() {
        let d1 = format!("sha256:{}", "1".repeat(64));
        let d2 = format!("sha256:{}", "2".repeat(64));
        let forward = manifest_digest(
            &[asset("a.schema.json", &d1), asset("b.schema.json", &d2)],
            SCHEMA_BUNDLE_DOMAIN,
        )
        .unwrap();
        let reversed = manifest_digest(
            &[asset("b.schema.json", &d2), asset("a.schema.json", &d1)],
            SCHEMA_BUNDLE_DOMAIN,
        )
        .unwrap();
        assert_eq!(forward, reversed, "input order must not matter");
    }

    #[test]
    fn duplicate_ids_and_empty_bundles_rejected() {
        let d = format!("sha256:{}", "3".repeat(64));
        let err = manifest_digest(
            &[asset("x.schema.json", &d), asset("x.schema.json", &d)],
            SCHEMA_BUNDLE_DOMAIN,
        )
        .unwrap_err();
        assert!(matches!(err, BundleError::DuplicateAssetId(_)));
        assert!(matches!(
            manifest_digest(&[], SCHEMA_BUNDLE_DOMAIN).unwrap_err(),
            BundleError::EmptyBundle
        ));
    }

    #[test]
    fn domains_separate_schema_bundle_and_spec_set() {
        let d = format!("sha256:{}", "4".repeat(64));
        let assets = [asset("a.schema.json", &d)];
        let bundle = manifest_digest(&assets, SCHEMA_BUNDLE_DOMAIN).unwrap();
        let set = manifest_digest(&assets, SPEC_SET_DOMAIN).unwrap();
        assert_ne!(bundle, set);
    }

    #[test]
    fn manifest_entry_order_and_shape_are_exact() {
        let d = format!("sha256:{}", "5".repeat(64));
        let bytes = manifest_canonical_bytes(&[asset("a.schema.json", &d)]).unwrap();
        let expected = format!(
            "{{\"assets\":[{{\"content_digest\":\"{d}\",\"id\":\"a.schema.json\",\
             \"media_type\":\"application/schema+json\",\"version\":\"0.1.0-draft.1\"}}]}}"
        );
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }
}
