# ADR-0028: Offline Linux Bundle Attestation Trust Boundary (P1-T08)

- Status: Accepted for the P1-T08 verifier foundation
- Date: 2026-07-28
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal distribution implementation decision. This ADR does
  not add or change a registry requirement, schema, transition, conformance
  vector, Profile claim, Gate claim, or release claim.
- Related: ADR-0004 canonical JSON, ADR-0025 Personal distribution,
  ADR-0029 bootstrap download trust, P1-T08 Linux installer, P7-T01
  release/SBOM/attestation production

## Context

The first P1-T08 slice validates an offline Linux x86_64 bundle manifest,
artifact SHA-256 digest, HTTPS attestation reference, and caller-fixed Pi pin
before staged activation. Its attestation reference is structural only. It
does not prove that a product-authorized release signer vouched for the
artifact and compatibility facts.

The next boundary must work without a network, external command, ambient
configuration, or bundle-provided trust anchor. There is no production signing
key or signing ceremony in the repository yet, so this decision must freeze a
verifier and rotation interface without fabricating production evidence.

## Decision

### 1. Mechanism and cryptographic library

CognitiveOS Personal Linux bundles use an **Ed25519 detached signature** over a
versioned attestation statement. Verification uses the audited Rust
`ed25519-dalek` implementation and its strict verification operation. The
runtime does not implement Ed25519 arithmetic and does not shell out to
`cosign`, `gh`, `curl`, OpenSSL, or another verifier.

Verification is strictly local. It performs no network access and does not
consult `PATH`, environment variables, the current directory, user
configuration, or a machine certificate store.

### 2. Signed statement and exact binding

The signed statement has the closed schema
`cognitiveos.personal.linux-bundle-attestation/1`. It binds all of:

- statement schema and version;
- product identity (`cognitiveos-personal`);
- target platform (`linux-x86_64`);
- release version;
- artifact filename;
- artifact SHA-256 digest;
- Pi version;
- Pi integrity; and
- the HTTPS provenance/attestation reference carried by the manifest.

The verifier strictly parses the local statement and compares every bound
release field with the already parsed manifest. A mismatch fails before
staging or active-version mutation.

The manifest provenance reference must parse as an absolute HTTPS URL with a
non-empty host, no user information, and no control characters. Artifact,
statement, and signature filenames must be safe, pairwise distinct local child
names and cannot replace `manifest.json`. Manifest, artifact, statement, and
signature inputs must be regular files rather than symbolic links, directories,
or special files; manifest and attestation metadata reads are size-bounded.

### 3. Bytes to sign

The signed bytes are the **exact original bytes** of the statement file, but
the verifier accepts those bytes only when they are already the RFC 8785 JCS
canonical encoding of the strictly parsed statement. The verification steps
are therefore:

1. read the statement as bounded local bytes;
2. require UTF-8 and strictly deserialize its closed typed shape;
3. serialize that typed value with the repository's registered RFC 8785 JCS
   canonicalizer;
4. require byte-for-byte equality with the original statement bytes; and
5. verify the detached Ed25519 signature over those exact original bytes.

Pretty JSON, insignificant whitespace, a UTF-8 BOM, alternate member order,
unknown members, duplicate members, or another semantically equivalent but
non-canonical encoding is rejected. No map iteration order or producer-specific
pretty printer participates in the signature boundary.

### 4. Signature envelope and encodings

The detached signature file is a closed JSON envelope with:

- envelope schema `cognitiveos.personal.linux-bundle-signature`;
- envelope version `1`;
- trusted key ID;
- algorithm exactly `Ed25519`; and
- signature encoded as unpadded URL-safe Base64.

The envelope is strictly parsed with duplicate and unknown fields rejected.
The decoder requires canonical unpadded URL-safe Base64 by decoding and then
re-encoding to the exact input. An Ed25519 signature must decode to exactly 64
bytes.

Trusted public key input uses the same canonical unpadded URL-safe Base64 rule
and must decode to exactly 32 bytes accepted by `ed25519-dalek`. Key IDs are
non-empty bounded ASCII identifiers; algorithms and versions are exact and
case-sensitive.

### 5. Product-owned trust root

Production acceptance is driven only by an explicitly supplied,
product-owned, versioned trusted keyring. The keyring constructor rejects:

- an empty keyring or empty keyring version;
- duplicate trusted key IDs;
- malformed or non-canonical key IDs or public-key encodings;
- unsupported key algorithms; and
- invalid Ed25519 public-key material.

The bundle manifest, statement, and signature envelope may reference an
allowed key ID, but cannot add a public key. Unknown and revoked key IDs fail
closed even when a bundle carries a mathematically valid signature and its own
public key. Unknown bundle fields used to smuggle a key are rejected by strict
parsing. No fallback trust source exists.

The repository intentionally contains no production trusted key or signing
key in this batch. Tests use keys explicitly marked test-only and never place a
private key in production configuration, evidence, logs, or release paths.

### 6. Rotation and revocation

The versioned keyring supports multiple distinct key IDs and an explicit
`active` or `revoked` status. Rotation is expressed by shipping a reviewed
product update whose keyring version contains the incoming active key and, as
needed, the outgoing key. Revocation is expressed by retaining the key ID with
`revoked` status or removing it; both states reject new verification under
that ID. Duplicate IDs are always invalid, regardless of status.

Dates, online revocation checks, transparency-log policy, threshold signing,
and signing ceremony controls are deferred to P7-T01. The verifier does not
infer trust from a timestamp or network service.

### 7. Failure-closed order and typed boundary

Only a value returned by complete verification may enter the staging API. The
verification sequence is:

1. strict manifest parsing and supported product/platform/version checks;
2. safe local child paths and forbidden Pi/Node payload checks;
3. artifact SHA-256 verification and caller-provided Pi pin comparison;
4. strict statement and signature-envelope parsing;
5. JCS canonical statement check;
6. manifest/statement binding check;
7. keyring validation and trusted key selection; and
8. strict cryptographic signature verification.

Failures are categorized without including artifact bytes, statement bytes,
signature bytes, key material, secrets, or user data. Attestation verification
is side-effect free and happens before staging or active-pointer mutation.
Staging re-reads and re-hashes the artifact immediately before writing the
candidate bytes, so mutation after verification fails before a staged
directory is created.

## Rejected alternatives

### Bundle-provided public key or certificate

Rejected because mathematical validity under a self-selected key is not
product trust. It permits an attacker to replace both artifact and signer.

### Environment/user/current-directory trust configuration

Rejected because ambient state silently changes production acceptance and is
not reviewable as part of the product trust root.

### Network or subprocess verifier

Rejected for this boundary. `curl`, `gh`, `cosign`, OpenSSL, and arbitrary
`PATH` commands introduce installation, version, network, and supply-chain
dependencies that are not frozen here and violate strict offline operation.

### Signing pretty JSON or a reserialized map

Rejected because whitespace and member iteration order are ambiguous.
Accepting non-canonical source while verifying regenerated bytes also hides
producer drift at a security boundary.

### Hand-written cryptography

Rejected because it expands audit surface and risks invalid-point,
malleability, and constant-time errors already handled by maintained libraries.

## Consequences and non-claims

This ADR closes only the P1-T08 **offline verifier foundation**. P7-T01 still
owns real SBOM generation, real attestation generation, production trusted-key
approval, production signing ceremony, release publication, and release
evidence.

This decision does not mean that a production signing key exists, a release
bundle or GitHub Release exists, provenance or an SBOM has been generated, or
P1-T08/P1-T09, B01, G1, any Profile, containment, RC, or release is complete.
The signed HTTPS provenance reference is a binding fact, not authorization to
dereference a URL. Downloader/inspected installer integration is governed by
ADR-0029; systemd user service, uninstall, cross-process installation lease,
interruption campaigns, and Linux-native Gate evidence remain separate work.
