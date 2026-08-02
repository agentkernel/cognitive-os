/**
 * `@cognitiveos/agent-shell`: reusable Task-channel client and session core for
 * CognitiveOS Personal Shell surfaces, including the Pi-hosted Agent Shell.
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

/** Task-channel verbs exposed to an interactive Shell adapter. */
export const SHELL_VERBS = ["propose", "preview", "attach", "detach", "cancel", "watch"] as const;

/** This client binds only the task channel; management uses separate clients. */
export const SHELL_CHANNEL: (typeof CLIENT_CHANNELS)[number] = "task";
