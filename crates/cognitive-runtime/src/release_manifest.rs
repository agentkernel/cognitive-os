//! Personal production release-manifest authority path (P7-T01/D01).
//!
//! Verifies a signed six-resource release manifest against caller-fixed pins.
//! This module does not generate SBOM bytes, publish GitHub Releases, hold a
//! production signing key, or claim Gate/release/Profile outcomes.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const RELEASE_MANIFEST_SCHEMA: &str = "cognitiveos.personal.release-manifest";
const SIGNATURE_SCHEMA: &str = "cognitiveos.personal.release-manifest-signature";
const ED25519_ALGORITHM: &str = "Ed25519";
const EXPECTED_PRODUCT_ID: &str = "cognitiveos-personal";
const EXPECTED_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
const EXPECTED_LICENSE: &str = "Apache-2.0";
const NON_CLAIM: &str = "not-claimed";
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_SIGNATURE_BYTES: usize = 16 * 1024;

/// Exact six Personal cognitive-resource families required by ADR-0037 / P7-T01.
pub const REQUIRED_RESOURCE_FAMILIES: [&str; 6] =
    ["Memory", "Skill", "Tool", "Context", "Task", "Runtime"];

/// One family pin: schema identity, version, and content digest.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceFamilyPin {
    pub schema_id: String,
    pub schema_version: String,
    pub content_digest: String,
}

/// Production release manifest fixing six-family pins and distribution trust facts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PersonalReleaseManifest {
    pub schema: String,
    pub schema_version: u32,
    pub product_id: String,
    pub version: String,
    pub target_triple: String,
    pub artifact_digest: String,
    pub license_spdx: String,
    pub notice_ref: String,
    pub third_party_inventory_ref: String,
    pub sbom_digest: String,
    pub attestation_ref: String,
    pub pi_pin: String,
    pub pi_integrity: String,
    pub resource_families: BTreeMap<String, ResourceFamilyPin>,
    pub sidecar_protocol_digest: String,
    pub adapter_digest: String,
    pub skill_package_digest: String,
    pub tool_catalog_digest: String,
    pub profile_claim: String,
    pub gate_claim: String,
}

/// Caller-fixed pins the manifest is never allowed to redefine unilaterally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedReleasePins {
    pub resource_families: BTreeMap<String, ResourceFamilyPin>,
    pub sidecar_protocol_digest: String,
    pub adapter_digest: String,
    pub skill_package_digest: String,
    pub tool_catalog_digest: String,
    pub pi_pin: String,
    pub pi_integrity: String,
}

/// Verified manifest plus its canonical content digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPersonalReleaseManifest {
    pub manifest: PersonalReleaseManifest,
    pub manifest_digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseManifestSignatureEnvelope {
    schema: String,
    schema_version: u32,
    key_id: String,
    algorithm: String,
    signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseTrustedKeyStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTrustedKeyInput {
    pub key_id: String,
    pub algorithm: String,
    pub public_key_base64url: String,
    pub status: ReleaseTrustedKeyStatus,
}

#[derive(Clone)]
struct ReleaseTrustedKey {
    verifying_key: VerifyingKey,
    status: ReleaseTrustedKeyStatus,
}

/// Production-trust keyring used only to verify release-manifest signatures.
#[derive(Clone)]
pub struct ReleaseTrustedKeyring {
    keys: BTreeMap<String, ReleaseTrustedKey>,
}

/// Fail-closed errors for the release-manifest authority path.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReleaseManifestError {
    #[error("release manifest is invalid: {0}")]
    InvalidManifest(&'static str),
    #[error("release manifest is missing required resource family pin: {0}")]
    MissingResourceFamily(String),
    #[error("release manifest resource-family pin does not match expected authority pin")]
    ResourceFamilyPinMismatch,
    #[error("release manifest sidecar or adapter pin does not match expected authority pin")]
    SidecarPinMismatch,
    #[error("release manifest Skill/Tool pin does not match expected authority pin")]
    SkillOrToolPinMismatch,
    #[error("release manifest Pi compatibility pin does not match the expected pin")]
    PiCompatibilityMismatch,
    #[error("release manifest platform or product identity is unsupported")]
    UnsupportedIdentity,
    #[error("release manifest claim fields must remain not-claimed")]
    AuthorityClaimRejected,
    #[error("release manifest attestation reference is missing or unsupported")]
    InvalidAttestationReference,
    #[error("release manifest signature envelope is malformed: {0}")]
    MalformedSignature(&'static str),
    #[error("release manifest signature version or algorithm is unsupported: {0}")]
    UnsupportedSignature(&'static str),
    #[error("release manifest references an unknown or untrusted key")]
    UnknownOrUntrustedKey,
    #[error("release manifest trusted keyring is invalid: {0}")]
    InvalidTrustedKeyring(&'static str),
    #[error("release manifest signature does not verify")]
    SignatureMismatch,
    #[error("release manifest bytes are not canonical")]
    NonCanonicalManifest,
}

impl ReleaseTrustedKeyring {
    pub fn new(
        _keyring_id: impl Into<String>,
        keys: Vec<ReleaseTrustedKeyInput>,
    ) -> Result<Self, ReleaseManifestError> {
        if keys.is_empty() {
            return Err(ReleaseManifestError::InvalidTrustedKeyring(
                "keyring must contain at least one key",
            ));
        }
        let mut mapped = BTreeMap::new();
        for key in keys {
            if key.key_id.is_empty()
                || key.key_id.len() > 128
                || !key
                    .key_id
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
            {
                return Err(ReleaseManifestError::InvalidTrustedKeyring(
                    "key_id is invalid",
                ));
            }
            if key.algorithm != ED25519_ALGORITHM {
                return Err(ReleaseManifestError::InvalidTrustedKeyring(
                    "only Ed25519 keys are supported",
                ));
            }
            let public_key =
                decode_canonical_base64url(&key.public_key_base64url).map_err(|_| {
                    ReleaseManifestError::InvalidTrustedKeyring("public key encoding is invalid")
                })?;
            let public_key_array: [u8; 32] = public_key.try_into().map_err(|_| {
                ReleaseManifestError::InvalidTrustedKeyring("public key length is invalid")
            })?;
            let verifying_key = VerifyingKey::from_bytes(&public_key_array).map_err(|_| {
                ReleaseManifestError::InvalidTrustedKeyring("public key is not a valid Ed25519 key")
            })?;
            if mapped
                .insert(
                    key.key_id,
                    ReleaseTrustedKey {
                        verifying_key,
                        status: key.status,
                    },
                )
                .is_some()
            {
                return Err(ReleaseManifestError::InvalidTrustedKeyring(
                    "duplicate key_id",
                ));
            }
        }
        Ok(Self { keys: mapped })
    }
}

/// Verify signed release-manifest bytes against caller-fixed six-family pins.
pub fn verify_personal_release_manifest(
    manifest_bytes: &[u8],
    signature_envelope_bytes: &[u8],
    expected: &ExpectedReleasePins,
    trusted_keyring: &ReleaseTrustedKeyring,
) -> Result<VerifiedPersonalReleaseManifest, ReleaseManifestError> {
    if manifest_bytes.is_empty() || manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(ReleaseManifestError::InvalidManifest(
            "manifest size is out of bounds",
        ));
    }
    if signature_envelope_bytes.is_empty() || signature_envelope_bytes.len() > MAX_SIGNATURE_BYTES {
        return Err(ReleaseManifestError::MalformedSignature(
            "signature envelope size is out of bounds",
        ));
    }

    let manifest_text = std::str::from_utf8(manifest_bytes)
        .map_err(|_| ReleaseManifestError::InvalidManifest("manifest must be UTF-8 JSON"))?;
    let manifest: PersonalReleaseManifest = deserialize_strict_json(manifest_text)
        .map_err(|_| ReleaseManifestError::InvalidManifest("strict JSON parsing failed"))?;

    let canonical_manifest = serde_json_canonicalizer::to_vec(&manifest)
        .map_err(|_| ReleaseManifestError::InvalidManifest("manifest cannot be canonicalized"))?;
    if canonical_manifest != manifest_bytes {
        return Err(ReleaseManifestError::NonCanonicalManifest);
    }

    validate_identity_and_claims(&manifest)?;
    validate_digest_shaped_fields(&manifest)?;
    validate_resource_families(&manifest, expected)?;
    validate_sidecar_and_package_pins(&manifest, expected)?;
    if manifest.pi_pin != expected.pi_pin || manifest.pi_integrity != expected.pi_integrity {
        return Err(ReleaseManifestError::PiCompatibilityMismatch);
    }
    if !is_strict_https_reference(&manifest.attestation_ref) {
        return Err(ReleaseManifestError::InvalidAttestationReference);
    }

    verify_signature(manifest_bytes, signature_envelope_bytes, trusted_keyring)?;

    Ok(VerifiedPersonalReleaseManifest {
        manifest_digest: sha256_digest(manifest_bytes),
        manifest,
    })
}

fn validate_identity_and_claims(
    manifest: &PersonalReleaseManifest,
) -> Result<(), ReleaseManifestError> {
    if manifest.schema != RELEASE_MANIFEST_SCHEMA || manifest.schema_version != 1 {
        return Err(ReleaseManifestError::InvalidManifest(
            "schema or schema_version is unsupported",
        ));
    }
    if manifest.product_id != EXPECTED_PRODUCT_ID
        || manifest.target_triple != EXPECTED_TARGET_TRIPLE
        || manifest.license_spdx != EXPECTED_LICENSE
        || manifest.version.trim().is_empty()
        || manifest.notice_ref.trim().is_empty()
        || manifest.third_party_inventory_ref.trim().is_empty()
    {
        return Err(ReleaseManifestError::UnsupportedIdentity);
    }
    if manifest.profile_claim != NON_CLAIM || manifest.gate_claim != NON_CLAIM {
        return Err(ReleaseManifestError::AuthorityClaimRejected);
    }
    Ok(())
}

fn validate_digest_shaped_fields(
    manifest: &PersonalReleaseManifest,
) -> Result<(), ReleaseManifestError> {
    for value in [
        manifest.artifact_digest.as_str(),
        manifest.sbom_digest.as_str(),
        manifest.sidecar_protocol_digest.as_str(),
        manifest.adapter_digest.as_str(),
        manifest.skill_package_digest.as_str(),
        manifest.tool_catalog_digest.as_str(),
        manifest.pi_integrity.as_str(),
    ] {
        if !is_sha_digest(value) {
            return Err(ReleaseManifestError::InvalidManifest(
                "digest-shaped field is missing or malformed",
            ));
        }
    }
    if manifest.pi_pin.trim().is_empty() {
        return Err(ReleaseManifestError::InvalidManifest(
            "pi_pin must be non-empty",
        ));
    }
    Ok(())
}

fn validate_resource_families(
    manifest: &PersonalReleaseManifest,
    expected: &ExpectedReleasePins,
) -> Result<(), ReleaseManifestError> {
    let required: BTreeSet<&str> = REQUIRED_RESOURCE_FAMILIES.into_iter().collect();
    let present: BTreeSet<&str> = manifest
        .resource_families
        .keys()
        .map(String::as_str)
        .collect();
    if present != required {
        for family in REQUIRED_RESOURCE_FAMILIES {
            if !present.contains(family) {
                return Err(ReleaseManifestError::MissingResourceFamily(
                    family.to_owned(),
                ));
            }
        }
        return Err(ReleaseManifestError::InvalidManifest(
            "resource_families must contain exactly the six Personal families",
        ));
    }
    if expected.resource_families.keys().collect::<BTreeSet<_>>()
        != manifest.resource_families.keys().collect::<BTreeSet<_>>()
    {
        return Err(ReleaseManifestError::ResourceFamilyPinMismatch);
    }
    for (family, pin) in &manifest.resource_families {
        if !is_sha_digest(&pin.content_digest)
            || pin.schema_id.trim().is_empty()
            || pin.schema_version.trim().is_empty()
        {
            return Err(ReleaseManifestError::InvalidManifest(
                "resource family pin is incomplete",
            ));
        }
        let expected_pin = expected
            .resource_families
            .get(family)
            .ok_or(ReleaseManifestError::ResourceFamilyPinMismatch)?;
        if pin != expected_pin {
            return Err(ReleaseManifestError::ResourceFamilyPinMismatch);
        }
    }
    Ok(())
}

fn validate_sidecar_and_package_pins(
    manifest: &PersonalReleaseManifest,
    expected: &ExpectedReleasePins,
) -> Result<(), ReleaseManifestError> {
    if manifest.sidecar_protocol_digest != expected.sidecar_protocol_digest
        || manifest.adapter_digest != expected.adapter_digest
    {
        return Err(ReleaseManifestError::SidecarPinMismatch);
    }
    if manifest.skill_package_digest != expected.skill_package_digest
        || manifest.tool_catalog_digest != expected.tool_catalog_digest
    {
        return Err(ReleaseManifestError::SkillOrToolPinMismatch);
    }
    Ok(())
}

fn verify_signature(
    manifest_bytes: &[u8],
    signature_envelope_bytes: &[u8],
    trusted_keyring: &ReleaseTrustedKeyring,
) -> Result<(), ReleaseManifestError> {
    let signature_text = std::str::from_utf8(signature_envelope_bytes)
        .map_err(|_| ReleaseManifestError::MalformedSignature("signature must be UTF-8"))?;
    let envelope: ReleaseManifestSignatureEnvelope = deserialize_strict_json(signature_text)
        .map_err(|_| ReleaseManifestError::MalformedSignature("signature JSON is invalid"))?;
    if envelope.schema != SIGNATURE_SCHEMA || envelope.schema_version != 1 {
        return Err(ReleaseManifestError::UnsupportedSignature(
            "signature envelope version",
        ));
    }
    if envelope.algorithm != ED25519_ALGORITHM {
        return Err(ReleaseManifestError::UnsupportedSignature(
            "signature algorithm",
        ));
    }
    validate_key_id(&envelope.key_id)?;
    let trusted_key = trusted_keyring
        .keys
        .get(&envelope.key_id)
        .filter(|key| key.status == ReleaseTrustedKeyStatus::Active)
        .ok_or(ReleaseManifestError::UnknownOrUntrustedKey)?;
    let signature_bytes = decode_canonical_base64url(&envelope.signature)
        .map_err(|_| ReleaseManifestError::MalformedSignature("signature encoding is invalid"))?;
    let signature_array: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| ReleaseManifestError::MalformedSignature("signature length is invalid"))?;
    let signature = Signature::from_bytes(&signature_array);
    trusted_key
        .verifying_key
        .verify_strict(manifest_bytes, &signature)
        .map_err(|_| ReleaseManifestError::SignatureMismatch)?;
    Ok(())
}

fn deserialize_strict_json<'de, Value>(input: &'de str) -> Result<Value, serde_json::Error>
where
    Value: Deserialize<'de>,
{
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let value = Value::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

fn validate_key_id(key_id: &str) -> Result<(), ReleaseManifestError> {
    if key_id.is_empty()
        || key_id.len() > 128
        || !key_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(ReleaseManifestError::MalformedSignature(
            "key_id is invalid",
        ));
    }
    Ok(())
}

fn decode_canonical_base64url(encoded: &str) -> Result<Vec<u8>, ()> {
    if encoded.is_empty() || encoded.contains('=') {
        return Err(());
    }
    URL_SAFE_NO_PAD.decode(encoded).map_err(|_| ())
}

fn is_strict_https_reference(reference: &str) -> bool {
    let Ok(url) = url::Url::parse(reference) else {
        return false;
    };
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.has_host()
}

fn is_sha_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const TEST_KEY_ID: &str = "release-manifest-unit-test-key";
    const PI_PIN: &str = "0.81.1";
    const PI_INTEGRITY: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn digest(label: &str) -> String {
        format!("sha256:{:x}", Sha256::digest(label.as_bytes()))
    }

    fn family_pin(family: &str) -> ResourceFamilyPin {
        ResourceFamilyPin {
            schema_id: format!("cognitiveos.{}/0.1", family.to_ascii_lowercase()),
            schema_version: "0.1".to_owned(),
            content_digest: digest(&format!("family:{family}")),
        }
    }

    fn expected_pins() -> ExpectedReleasePins {
        let mut resource_families = BTreeMap::new();
        for family in REQUIRED_RESOURCE_FAMILIES {
            resource_families.insert(family.to_owned(), family_pin(family));
        }
        ExpectedReleasePins {
            resource_families,
            sidecar_protocol_digest: digest("sidecar-protocol"),
            adapter_digest: digest("adapter"),
            skill_package_digest: digest("skill-package"),
            tool_catalog_digest: digest("tool-catalog"),
            pi_pin: PI_PIN.to_owned(),
            pi_integrity: PI_INTEGRITY.to_owned(),
        }
    }

    fn valid_manifest(expected: &ExpectedReleasePins) -> PersonalReleaseManifest {
        PersonalReleaseManifest {
            schema: RELEASE_MANIFEST_SCHEMA.to_owned(),
            schema_version: 1,
            product_id: EXPECTED_PRODUCT_ID.to_owned(),
            version: "1.0.0-rc.0".to_owned(),
            target_triple: EXPECTED_TARGET_TRIPLE.to_owned(),
            artifact_digest: digest("artifact"),
            license_spdx: EXPECTED_LICENSE.to_owned(),
            notice_ref: "NOTICE".to_owned(),
            third_party_inventory_ref: "docs/legal/THIRD-PARTY-NOTICES.md".to_owned(),
            sbom_digest: digest("sbom"),
            attestation_ref: "https://example.invalid/attestations/release/v1".to_owned(),
            pi_pin: expected.pi_pin.clone(),
            pi_integrity: expected.pi_integrity.clone(),
            resource_families: expected.resource_families.clone(),
            sidecar_protocol_digest: expected.sidecar_protocol_digest.clone(),
            adapter_digest: expected.adapter_digest.clone(),
            skill_package_digest: expected.skill_package_digest.clone(),
            tool_catalog_digest: expected.tool_catalog_digest.clone(),
            profile_claim: NON_CLAIM.to_owned(),
            gate_claim: NON_CLAIM.to_owned(),
        }
    }

    fn test_signing_key() -> SigningKey {
        SigningKey::from_bytes(&[0x51; 32])
    }

    fn test_keyring() -> ReleaseTrustedKeyring {
        let signing_key = test_signing_key();
        ReleaseTrustedKeyring::new(
            "release-manifest-unit-test-keyring-v1",
            vec![ReleaseTrustedKeyInput {
                key_id: TEST_KEY_ID.to_owned(),
                algorithm: ED25519_ALGORITHM.to_owned(),
                public_key_base64url: URL_SAFE_NO_PAD
                    .encode(signing_key.verifying_key().to_bytes()),
                status: ReleaseTrustedKeyStatus::Active,
            }],
        )
        .unwrap()
    }

    fn sign_manifest(manifest_bytes: &[u8]) -> Vec<u8> {
        let signature = test_signing_key().sign(manifest_bytes);
        serde_json::to_vec(&serde_json::json!({
            "schema": SIGNATURE_SCHEMA,
            "schema_version": 1,
            "key_id": TEST_KEY_ID,
            "algorithm": ED25519_ALGORITHM,
            "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        }))
        .unwrap()
    }

    fn canonical_manifest_bytes(manifest: &PersonalReleaseManifest) -> Vec<u8> {
        serde_json_canonicalizer::to_vec(manifest).unwrap()
    }

    #[test]
    fn accepts_signed_six_family_release_manifest() {
        let expected = expected_pins();
        let manifest_bytes = canonical_manifest_bytes(&valid_manifest(&expected));
        let signature = sign_manifest(&manifest_bytes);
        let verified = verify_personal_release_manifest(
            &manifest_bytes,
            &signature,
            &expected,
            &test_keyring(),
        )
        .expect("valid release manifest must verify");
        assert_eq!(verified.manifest.resource_families.len(), 6);
        assert_eq!(verified.manifest_digest, sha256_digest(&manifest_bytes));
        assert_eq!(verified.manifest.profile_claim, NON_CLAIM);
        assert_eq!(verified.manifest.gate_claim, NON_CLAIM);
    }

    #[test]
    fn rejects_missing_resource_family() {
        let expected = expected_pins();
        let mut manifest = valid_manifest(&expected);
        manifest.resource_families.remove("Memory");
        let manifest_bytes = canonical_manifest_bytes(&manifest);
        let signature = sign_manifest(&manifest_bytes);
        let err = verify_personal_release_manifest(
            &manifest_bytes,
            &signature,
            &expected,
            &test_keyring(),
        )
        .expect_err("missing Memory pin must fail closed");
        assert!(matches!(
            err,
            ReleaseManifestError::MissingResourceFamily(_)
        ));
    }

    #[test]
    fn rejects_resource_family_digest_drift() {
        let expected = expected_pins();
        let mut manifest = valid_manifest(&expected);
        manifest
            .resource_families
            .get_mut("Context")
            .unwrap()
            .content_digest = digest("drifted-context");
        let manifest_bytes = canonical_manifest_bytes(&manifest);
        let signature = sign_manifest(&manifest_bytes);
        let err = verify_personal_release_manifest(
            &manifest_bytes,
            &signature,
            &expected,
            &test_keyring(),
        )
        .expect_err("family digest drift must fail closed");
        assert_eq!(err, ReleaseManifestError::ResourceFamilyPinMismatch);
    }

    #[test]
    fn rejects_sidecar_adapter_and_skill_tool_pin_drift() {
        let expected = expected_pins();
        let mut sidecar_drift = valid_manifest(&expected);
        sidecar_drift.adapter_digest = digest("wrong-adapter");
        let sidecar_bytes = canonical_manifest_bytes(&sidecar_drift);
        assert_eq!(
            verify_personal_release_manifest(
                &sidecar_bytes,
                &sign_manifest(&sidecar_bytes),
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::SidecarPinMismatch
        );

        let mut skill_drift = valid_manifest(&expected);
        skill_drift.tool_catalog_digest = digest("wrong-tool-catalog");
        let skill_bytes = canonical_manifest_bytes(&skill_drift);
        assert_eq!(
            verify_personal_release_manifest(
                &skill_bytes,
                &sign_manifest(&skill_bytes),
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::SkillOrToolPinMismatch
        );
    }

    #[test]
    fn rejects_gate_or_profile_claim_and_bad_signature() {
        let expected = expected_pins();
        let mut claimed = valid_manifest(&expected);
        claimed.gate_claim = "pass".to_owned();
        let claimed_bytes = canonical_manifest_bytes(&claimed);
        assert_eq!(
            verify_personal_release_manifest(
                &claimed_bytes,
                &sign_manifest(&claimed_bytes),
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::AuthorityClaimRejected
        );

        let manifest_bytes = canonical_manifest_bytes(&valid_manifest(&expected));
        let mut bad_signature = sign_manifest(&manifest_bytes);
        // Flip one signature payload byte after signing a different message.
        let other = sign_manifest(b"{\"tampered\":true}");
        bad_signature = other;
        assert_eq!(
            verify_personal_release_manifest(
                &manifest_bytes,
                &bad_signature,
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::SignatureMismatch
        );
    }

    #[test]
    fn rejects_pi_drift_non_https_attestation_and_non_canonical_bytes() {
        let expected = expected_pins();
        let mut pi_drift = valid_manifest(&expected);
        pi_drift.pi_pin = "9.9.9".to_owned();
        let pi_bytes = canonical_manifest_bytes(&pi_drift);
        assert_eq!(
            verify_personal_release_manifest(
                &pi_bytes,
                &sign_manifest(&pi_bytes),
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::PiCompatibilityMismatch
        );

        let mut bad_attestation = valid_manifest(&expected);
        bad_attestation.attestation_ref = "http://insecure.example/attest".to_owned();
        let attestation_bytes = canonical_manifest_bytes(&bad_attestation);
        assert_eq!(
            verify_personal_release_manifest(
                &attestation_bytes,
                &sign_manifest(&attestation_bytes),
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::InvalidAttestationReference
        );

        let mut pretty = canonical_manifest_bytes(&valid_manifest(&expected));
        pretty.push(b'\n');
        assert_eq!(
            verify_personal_release_manifest(
                &pretty,
                &sign_manifest(&pretty),
                &expected,
                &test_keyring()
            )
            .unwrap_err(),
            ReleaseManifestError::NonCanonicalManifest
        );
    }
}
