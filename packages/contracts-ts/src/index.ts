/**
 * `@cognitiveos/contracts-ts`: machine-contract layer of the CognitiveOS
 * reference implementation (TypeScript side).
 *
 * Scope (M0): canonical JSON encoding, domain-separated digests and
 * signature preimages, shared golden fixtures with the Rust twin
 * (`crates/cognitive-contracts`). Schema-generated TypeScript bindings
 * arrive with the codegen pipeline (Lane-CTR, M1; see
 * `docs/adr/0006-code-generation-policy.md`).
 */

export * from "./canonical.js";

/** Encoding profile implemented by this package. */
export const ENCODING_PROFILE = "cognitiveos.canonical-json/0.1";
