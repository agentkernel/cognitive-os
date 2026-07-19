/**
 * `@cognitiveos/sdk-ts`: client SDK of the CognitiveOS reference
 * implementation (M5 delivery per `docs/plan/DEVELOPMENT-PLAN.md`; M0
 * skeleton only).
 *
 * Hard rules (see `.cursor/rules/11-typescript-clients.mdc`): clients are
 * never an authority; every displayed state is an authority projection; the
 * task channel and the management channel keep separate credentials and
 * caches and are never mixed in one client instance.
 */

import { ENCODING_PROFILE } from "@cognitiveos/contracts-ts";

/** The two isolated client channels. A client instance binds exactly one. */
export const CLIENT_CHANNELS = ["task", "management"] as const;
export type ClientChannel = (typeof CLIENT_CHANNELS)[number];

/** Encoding profile shared with the contracts layer. */
export const SDK_ENCODING_PROFILE: string = ENCODING_PROFILE;
