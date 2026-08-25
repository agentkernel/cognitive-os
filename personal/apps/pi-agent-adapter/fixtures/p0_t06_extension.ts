import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const BLOCKED_BUILT_IN_TOOLS = new Set(["write", "edit", "bash"]);

/**
 * P0-T06 compatibility fixture for Pi 0.81.1.
 *
 * This extension is intentionally a narrow, non-authoritative compatibility
 * probe. It rejects project trust, blocks Pi's mutating built-in tools, and
 * only displays session-local status. It has no durable state, provider
 * credential, network, or CognitiveOS authority access.
 */
export default function registerCognitiveOsCompatibilityProbe(pi: ExtensionAPI) {
  pi.on("project_trust", async () => ({ trusted: "no" }));

  pi.on("tool_call", async (event) => {
    if (BLOCKED_BUILT_IN_TOOLS.has(event.toolName)) {
      return {
        block: true,
        reason: "CognitiveOS candidate compatibility probe blocks Pi mutating tools",
      };
    }
  });

  pi.on("session_start", async (_event, context) => {
    context.ui.setStatus(
      "cognitiveos-p0-t06",
      "candidate-only: project trust denied; write/edit/bash blocked",
    );
  });

  pi.registerCommand("cognitiveos-p0-t06-status", {
    description: "Show the non-authoritative P0-T06 compatibility probe status",
    handler: async (_arguments, context) => {
      context.ui.notify(
        "CognitiveOS P0-T06 compatibility probe is candidate-only",
        "info",
      );
    },
  });
}
