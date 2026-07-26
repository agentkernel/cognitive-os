/**
 * Presentation mapping from daemon facts to Pi UI text.
 *
 * Every string here is derived from a projection the daemon actually returned,
 * or from an explicit failure. Nothing in this file infers readiness, softens a
 * `blocked` result, or omits the standing non-claim: a Personal readiness
 * projection is not a Gate, Profile or release claim, and this Extension never
 * upgrades one into the other.
 */

import type { ReadinessProjection } from "./daemon-client.js";
import { DaemonClientError, isDaemonUnavailable } from "./errors.js";

/** Short line shown in the Pi status bar. */
export function statusLineFromProjection(projection: ReadinessProjection): string {
  const conversation = projection.firstConversationReady
    ? "first conversation ready"
    : "first conversation blocked";
  return `CognitiveOS ${projection.overall}: ${conversation}; governed mode, Pi tools disabled`;
}

/** Short line shown in the Pi status bar when the daemon could not be read. */
export function statusLineFromFailure(error: unknown): string {
  if (isDaemonUnavailable(error)) {
    return "CognitiveOS unavailable: daemon not reachable; governed mode, Pi tools disabled";
  }
  if (error instanceof DaemonClientError) {
    return `CognitiveOS unavailable: ${error.code}; governed mode, Pi tools disabled`;
  }
  return "CognitiveOS unavailable: unexpected client failure; governed mode, Pi tools disabled";
}

/** Operator-facing detail for the `/cognitive-status` command. */
export function statusDetailFromProjection(projection: ReadinessProjection): string {
  const lines: string[] = [
    `CognitiveOS Personal status: ${projection.overall}`,
    `first conversation ready: ${projection.firstConversationReady ? "yes" : "no"}`,
  ];

  for (const component of projection.components) {
    const requirement = component.required ? "required" : "optional";
    const reason = component.errorClass === undefined ? "" : ` (${component.errorClass})`;
    lines.push(`- ${component.component}: ${component.status} [${requirement}]${reason}`);
  }

  lines.push(
    `claims: profile=${projection.profileClaim}, gate=${projection.gateClaim}`,
    "this is a static readiness projection, not a Gate, Profile or release claim",
    "Pi runs as a non-authority client: direct bash/write/edit are disabled",
  );
  return lines.join("\n");
}

/** Operator-facing detail for `/cognitive-status` when the daemon could not be read. */
export function statusDetailFromFailure(error: unknown): string {
  if (error instanceof DaemonClientError) {
    const httpStatus = error.httpStatus === undefined ? "" : ` (HTTP ${error.httpStatus})`;
    const daemonCode =
      error.daemonErrorCode === undefined ? "" : `\ndaemon error code: ${error.daemonErrorCode}`;
    return [
      `CognitiveOS Personal status unavailable: ${error.code}${httpStatus}`,
      error.message,
      daemonCode,
      "no readiness is assumed while the daemon cannot be read",
    ]
      .filter((line) => line.length > 0)
      .join("\n");
  }
  return [
    "CognitiveOS Personal status unavailable: unexpected client failure",
    "no readiness is assumed while the daemon cannot be read",
  ].join("\n");
}
