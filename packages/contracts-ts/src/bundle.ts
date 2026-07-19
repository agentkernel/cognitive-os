/**
 * Set and bundle digests (registered procedure), TypeScript twin of
 * `crates/cognitive-contracts/src/bundle.rs`.
 *
 * Implements docs/standards/canonical-encoding-and-digest.md section 13: a
 * specification set or schema bundle is digested over a CANONICAL LOGICAL
 * MANIFEST — one entry per logical asset carrying asset ID, version, media
 * type, and full content digest, in deterministic sorted order — never over
 * a variable archive representation. Nested (per-asset) digests are computed
 * first; the manifest digest then pins the whole set.
 *
 * Cross-language byte identity with the Rust twin is held by the golden
 * fixtures (`tests/golden/`) and the emit-golden CI gate.
 */

import { canonicalize, digest } from "./canonical.js";

/** Registered digest domain for schema bundles (standard section 9/13). */
export const SCHEMA_BUNDLE_DOMAIN = "schema-bundle/0.1";

/** Registered digest domain for specification sets (standard section 9/13). */
export const SPEC_SET_DOMAIN = "spec-set/0.1";

/** Media type recorded for JSON Schema assets. */
export const MEDIA_TYPE_SCHEMA_JSON = "application/schema+json";

/** Media type recorded for plain JSON assets (transition tables). */
export const MEDIA_TYPE_JSON = "application/json";

/**
 * Media type recorded for YAML registry assets (digested over their
 * canonical JSON projection).
 */
export const MEDIA_TYPE_YAML = "application/yaml";

/**
 * Suite-level SemVer applied to every asset of the v0.1 draft suite
 * (drift D-011: no per-asset SemVer is registered yet).
 */
export const SPEC_SUITE_VERSION = "0.1.0-draft.1";

/** One logical asset of a set or bundle manifest. */
export interface BundleAsset {
  /** Logical asset ID: schema file name (== `$id`) or repo-relative path. */
  readonly id: string;
  /** Asset SemVer (currently SPEC_SUITE_VERSION for every asset). */
  readonly version: string;
  /** Asset media type. */
  readonly media_type: string;
  /** Full domain-separated content digest of the asset's canonical bytes. */
  readonly content_digest: string;
}

/** Manifest construction failure (duplicate asset ID / empty bundle). */
export class BundleError extends Error {
  constructor(
    readonly category: "duplicate-asset-id" | "empty-bundle",
    detail: string,
  ) {
    super(`${category}: ${detail}`);
    this.name = "BundleError";
  }
}

/**
 * Build the canonical logical manifest value: `{"assets": [...]}` with one
 * `{content_digest, id, media_type, version}` entry per asset, sorted by
 * asset ID (deterministic sorted order per standard section 13).
 */
export function manifestValue(assets: ReadonlyArray<BundleAsset>): {
  assets: ReadonlyArray<Record<string, string>>;
} {
  if (assets.length === 0) {
    throw new BundleError("empty-bundle", "a manifest must cover at least one asset");
  }
  const sorted = [...assets].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  for (let i = 1; i < sorted.length; i += 1) {
    const current = sorted[i];
    const previous = sorted[i - 1];
    if (current !== undefined && previous !== undefined && current.id === previous.id) {
      throw new BundleError("duplicate-asset-id", current.id);
    }
  }
  return {
    assets: sorted.map((asset) => ({
      id: asset.id,
      version: asset.version,
      media_type: asset.media_type,
      content_digest: asset.content_digest,
    })),
  };
}

/** Canonical bytes of the manifest (RFC 8785 over the manifest value). */
export function manifestCanonicalBytes(assets: ReadonlyArray<BundleAsset>): Uint8Array {
  return canonicalize(JSON.stringify(manifestValue(assets)));
}

/** Manifest digest under the given registered bundle domain. */
export function manifestDigest(assets: ReadonlyArray<BundleAsset>, domain: string): string {
  return digest(manifestCanonicalBytes(assets), domain);
}

/**
 * Per-asset content digest: canonical bytes of the parsed asset value under
 * the bundle's own domain (nested digest, verified first per section 13).
 */
export function assetContentDigest(assetValue: unknown, domain: string): string {
  return digest(canonicalize(JSON.stringify(assetValue)), domain);
}
