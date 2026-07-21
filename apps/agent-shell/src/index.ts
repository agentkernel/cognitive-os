/**
 * `@cognitiveos/agent-shell`: task Shell client of the CognitiveOS reference
 * implementation (M5 delivery per `docs/plan/DEVELOPMENT-PLAN.md`; M0
 * skeleton only). Coexists with `apps/cognitiveos-console` (separate planned
 * product; see its PRODUCT-DESIGN.md).
 *
 * Hard rules (whitepaper Shell semantics; vectors `shell-*.json`): the Shell
 * is a client, never an authority; detaching or exiting the Shell does not
 * cancel a Task (`shell-detach-attach-004`); cancel is a request whose
 * closure is decided by Effect state, not by the Shell
 * (`shell-cancel-semantics-005`).
 */

import { CLIENT_CHANNELS } from "@cognitiveos/sdk-ts";

export * from "./live.js";
export * from "./session.js";

/** Shell verbs (the CLI front end binds them at M5). */
export const SHELL_VERBS = ["propose", "preview", "attach", "detach", "cancel", "watch"] as const;

/** The Shell binds the task channel only; management uses admin-cli/Console. */
export const SHELL_CHANNEL: (typeof CLIENT_CHANNELS)[number] = "task";
