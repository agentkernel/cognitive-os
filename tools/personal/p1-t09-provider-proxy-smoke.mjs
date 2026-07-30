/**
 * Bounded, redacted P1-T09 Provider-proxy smoke probe.
 *
 * This invokes the same PersonalDaemonClient used by the Pi Extension. It does
 * not accept credentials, read Provider config, or print a bearer, SecretRef,
 * selected-model digest, or model response. The daemon remains the only code
 * that resolves the provider secret and dispatches the Provider request.
 */
import { PersonalDaemonClient } from "../../packages/pi-cognitiveos/dist/daemon-client.js";

const smokePrompt = "Reply exactly: cognitiveos-provider-smoke-ok";
const daemonClient = new PersonalDaemonClient({ requestTimeoutMs: 30_000 });

try {
  const selectedModelProjection = await daemonClient.fetchSelectedModel();
  const completion = await daemonClient.completeChat(selectedModelProjection.selectedModel, [
    { role: "user", content: smokePrompt },
  ]);
  const expectedReplyObserved = completion.content.includes("cognitiveos-provider-smoke-ok");

  console.log(JSON.stringify({
    status: "ok",
    authority_side_effects: false,
    finish_reason: completion.finishReason,
    expected_reply_observed: expectedReplyObserved,
    response_received: completion.content.length > 0,
  }));
} catch (error) {
  console.log(JSON.stringify({
    status: "error",
    authority_side_effects: false,
    error_class: error instanceof Error ? error.name : "unknown",
    error_code: error instanceof Error && "code" in error ? error.code : undefined,
  }));
  process.exitCode = 1;
}
