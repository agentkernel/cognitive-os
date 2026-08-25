/**
 * `@cognitiveos/sdk-ts`: client SDK of the CognitiveOS reference
 * implementation (Lane-TSC M5: HttpSseTransport binds kernel-server
 * `/management/*`, `/shell/*`, `GET /task/watch`).
 *
 * Hard rules (see `.cursor/rules/11-typescript-clients.mdc`): clients are
 * never an authority; every displayed state is an authority projection; the
 * task channel and the management channel keep separate credentials and
 * caches and are never mixed in one client instance.
 */

import { ENCODING_PROFILE } from "@cognitiveos/contracts-ts";

export * from "./channel.js";
export * from "./client.js";
export * from "./envelope.js";
export * from "./errors.js";
export * from "./fixtures.js";
export * from "./transport.js";
export * from "./views.js";
export * from "./watch.js";

/** Encoding profile shared with the contracts layer. */
export const SDK_ENCODING_PROFILE: string = ENCODING_PROFILE;
