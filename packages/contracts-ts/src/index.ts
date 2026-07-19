/**
 * `@cognitiveos/contracts-ts`: machine-contract layer of the CognitiveOS
 * reference implementation (TypeScript side).
 *
 * Scope: canonical JSON encoding, domain-separated digests and signature
 * preimages, shared golden fixtures with the Rust twin
 * (`crates/cognitive-contracts`), and the schema-generated TypeScript
 * bindings for the IMP-08 minimal core object set (`./generated/`, emitted
 * by the committed `contracts-codegen` binary per
 * `docs/adr/0006-code-generation-policy.md`; CI regenerates and diffs).
 */

export * from "./bundle.js";
export * from "./canonical.js";
export * from "./generated/index.js";

/** Encoding profile implemented by this package. */
export const ENCODING_PROFILE = "cognitiveos.canonical-json/0.1";
