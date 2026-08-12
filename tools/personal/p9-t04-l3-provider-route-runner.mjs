/**
 * P9-T04 `L3` Provider-route runner.
 *
 * Drives the real Pi-to-daemon-to-Provider path through the same
 * `PersonalDaemonClient` the Pi Extension uses, and retains every started
 * request as a classified outcome. It never retries a completion, never prints
 * a bearer, prompt, response, header, or SecretRef, and never reports a time to
 * first token or a cost: the proxy is non-streaming and no pricing snapshot
 * exists. Provider usage stays `not_available` unless DeepSeek returned a
 * complete, internally consistent counter set.
 */
import { PersonalDaemonClient } from "../../packages/pi-cognitiveos/dist/daemon-client.js";

const MARKER = "cognitiveos-provider-smoke-ok";
const PROMPT = `Reply exactly: ${MARKER}`;
const REQUEST_TIMEOUT_MS = 60_000;

function requiredArgument(name, fallback) {
  const argv = process.argv.slice(2);
  const index = argv.indexOf(name);
  if (index === -1) {
    if (fallback !== undefined) return fallback;
    throw new Error(`missing required argument ${name}`);
  }
  const value = argv[index + 1];
  if (value === undefined) throw new Error(`missing value for ${name}`);
  return value;
}

/**
 * Map a failure onto the fixed outcome vocabulary. The classification uses the
 * registered error class only, never the message body.
 */
function classifyFailure(error) {
  const code = typeof error?.code === "string" ? error.code : "";
  const name = error instanceof Error ? error.name : "unknown";
  const text = `${code} ${name}`.toLowerCase();
  if (text.includes("timeout") || text.includes("abort")) return "timeout";
  if (text.includes("429") || text.includes("rate")) return "rate_limited";
  if (text.includes("401") || text.includes("403") || text.includes("denied")) {
    return "denied_before_dispatch";
  }
  if (text.includes("502") || text.includes("upstream") || text.includes("provider")) {
    return "upstream_failure";
  }
  return "outcome_unknown";
}

const startedRequests = Number.parseInt(requiredArgument("--samples", "30"), 10);
if (!Number.isSafeInteger(startedRequests) || startedRequests < 1) {
  throw new Error("--samples must be a positive integer");
}
const sourceRevision = requiredArgument("--source-revision");
if (!/^[0-9a-f]{40}$/.test(sourceRevision)) {
  throw new Error("--source-revision must be a full hexadecimal Git revision");
}

const scenarioId = requiredArgument("--scenario", "R1-provider-proxy-marker");
/**
 * `R5` deliberately requests a model the daemon did not select, so the request
 * must fail closed before any Provider dispatch. The override is a model
 * identifier only; it is never a credential.
 */
const modelOverride = requiredArgument("--model-override", "");

const daemonClient = new PersonalDaemonClient({ requestTimeoutMs: REQUEST_TIMEOUT_MS });
const selectedModel = (await daemonClient.fetchSelectedModel()).selectedModel;
const requestedModel = modelOverride === "" ? selectedModel : modelOverride;
const samples = [];

for (let sample = 0; sample < startedRequests; sample += 1) {
  const startedAt = process.hrtime.bigint();
  try {
    const completion = await daemonClient.completeChat(requestedModel, [
      { role: "user", content: PROMPT },
    ]);
    samples.push({
      outcome: "complete_response",
      total_elapsed_nanos: Number(process.hrtime.bigint() - startedAt),
      loopback_http_elapsed_nanos: completion.loopbackHttpElapsedNanos,
      provider_network_elapsed_nanos: completion.providerNetworkElapsedNanos ?? null,
      usage_availability: completion.providerUsage.availability,
      marker_observed: completion.content.includes(MARKER),
      response_characters: completion.content.length,
    });
  } catch (error) {
    // A failed sample is retained and classified. It is never retried and
    // never replaced by a fresh attempt.
    samples.push({
      outcome: classifyFailure(error),
      total_elapsed_nanos: Number(process.hrtime.bigint() - startedAt),
      loopback_http_elapsed_nanos: null,
      provider_network_elapsed_nanos: null,
      usage_availability: "not_available",
      marker_observed: false,
      response_characters: 0,
    });
  }
}

const outcomeCounts = samples.reduce((counts, sample) => {
  counts[sample.outcome] = (counts[sample.outcome] ?? 0) + 1;
  return counts;
}, {});

const networkDurations = samples
  .map((sample) => sample.provider_network_elapsed_nanos)
  .filter((value) => typeof value === "number")
  .sort((left, right) => left - right);

function percentile(fraction) {
  if (networkDurations.length === 0) return null;
  const index = Math.min(
    networkDurations.length - 1,
    Math.floor(fraction * networkDurations.length),
  );
  return networkDurations[index];
}

console.log(JSON.stringify({
  report_kind: "p9-t04-l3-provider-route/0.1",
  claim_level: "hypothesis",
  scenario_id: scenarioId,
  source_revision: sourceRevision,
  selected_model: selectedModel,
  requested_model: requestedModel,
  model_override_applied: modelOverride !== "",
  retry_budget: 0,
  first_token_timing: "not_streaming",
  cost_available: false,
  started_requests: startedRequests,
  retained_samples: samples.length,
  outcome_counts: outcomeCounts,
  provider_network_nanos: {
    measured_samples: networkDurations.length,
    p50: percentile(0.5),
    p95: percentile(0.95),
    minimum: networkDurations[0] ?? null,
    maximum: networkDurations[networkDurations.length - 1] ?? null,
  },
  samples,
}, null, 2));
