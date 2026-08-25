/**
 * Pi compatibility pin, mirrored from the Rust `PiCompatibilityPin::expected()`
 * in `personal/apps/pi-agent-adapter/src/lib.rs`.
 *
 * The Rust constant is the single source of truth; `pin.test.ts` parses that
 * file and fails if any value here drifts from it. These are compatibility pins
 * only — an npm SRI is not trusted signature or provenance evidence, and this
 * package makes no supply-chain claim (that verifier is Pi P2, see
 * `docs/plan/archive/PI-AGENT-INTEGRATION-PLAN.md`).
 */

export interface PiCompatibilityPin {
  readonly packageName: string;
  readonly packageVersion: string;
  readonly npmIntegrity: string;
  readonly sourceCommit: string;
  readonly repositoryUrl: string;
  readonly repositoryDirectory: string;
  readonly nodeEngine: string;
}

export const PI_COMPATIBILITY_PIN: PiCompatibilityPin = {
  packageName: "@earendil-works/pi-coding-agent",
  packageVersion: "0.81.1",
  npmIntegrity:
    "sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==",
  sourceCommit: "20be4b18d4c57487f8993d2762bace129f0cf7c6",
  repositoryUrl: "https://github.com/earendil-works/pi.git",
  repositoryDirectory: "packages/coding-agent",
  nodeEngine: ">=22.19.0",
};

/** Status key this Extension owns in the Pi UI. */
export const COGNITIVEOS_STATUS_KEY = "cognitiveos";

/** Command name registered with Pi (Pi registers names without a leading slash). */
export const COGNITIVEOS_STATUS_COMMAND_NAME = "cognitive-status";

/** The slash command an operator types in the Pi TUI. */
export const COGNITIVEOS_STATUS_COMMAND = `/${COGNITIVEOS_STATUS_COMMAND_NAME}`;
