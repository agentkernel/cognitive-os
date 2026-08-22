/**
 * Shared Personal daemon helpers for P8-T09 harnesses.
 * Prints and returns redacted task facts only — no bearer, bootstrap, or Provider key.
 */
export function uuid7Like(kind) {
  const n = (Date.now() ^ process.pid ^ kind.length) >>> 0;
  const suffix = n.toString(16).padStart(12, "0").slice(-12);
  const variant = kind === "budget" ? "8" : "9";
  return `00000000-0000-7000-${variant}000-${suffix}`;
}

export async function httpJson(origin, method, path, token, body) {
  const headers = {};
  if (token) headers.authorization = `Bearer ${token}`;
  const init = { method, headers };
  if (body !== undefined) {
    headers["content-type"] = "application/json";
    init.body = typeof body === "string" ? body : JSON.stringify(body);
  }
  const response = await fetch(`${origin}${path}`, init);
  const text = await response.text();
  let json;
  try {
    json = JSON.parse(text);
  } catch {
    json = { parse_error: true, http_status: response.status, body_bytes: Buffer.byteLength(text, "utf8") };
  }
  return { status: response.status, json };
}

export async function issueToken(origin, bootstrap, channel) {
  const { json } = await httpJson(origin, "POST", "/local/session", "", {
    channel,
    principal_id: "principal://local/owner",
    bootstrap_secret: bootstrap,
  });
  const token = json.token;
  if (typeof token !== "string" || !token) {
    throw new Error(`session token missing for ${channel}`);
  }
  return token;
}

export async function admitTask(origin, token, spec) {
  const recorded = await httpJson(origin, "POST", "/task/intent.record", token, {
    conversation_or_scope_ref: spec.conversation,
    raw_expression: spec.objective,
    schema_version: "cognitiveos.task-intent-record-request/0.1",
  });
  const interpreted = await httpJson(origin, "POST", "/task/intent.interpret", token, {
    schema_version: "cognitiveos.task-intent-interpret-request/0.1",
    user_intent_record_id: recorded.json.user_intent_record_id,
    candidate: {
      objectives: [spec.objective],
      constraints: [],
      forbidden: ["bash", "edit", "write"],
      assumptions: [],
      ambiguities: [],
      information_gaps: [],
    },
  });
  const draft = {
    allowed_state_domains: ["task", "effect"],
    allowed_tools: [spec.tool],
    budget: { semantic_calls: 4, tool_calls: 4 },
    budget_id: uuid7Like(`budget-${spec.family}`),
    conditions: [
      {
        description: "independent fixed-effect verification",
        id: "acceptance",
        kind: "acceptance",
        verifier_ref: "verifier://personal/fixed-effect",
      },
    ],
    deadline: "2027-12-31T00:00:00Z",
    loop_object_id: uuid7Like(`loop-${spec.family}`),
    max_iterations: 4,
    max_retries: 0,
    objective: spec.objective,
    scope: {
      in_scope: [`workspace ${spec.family}`],
      out_of_scope: ["bash", "edit", "write"],
    },
    task_ref: spec.taskRef,
  };
  const previewed = await httpJson(origin, "POST", "/task/preview", token, {
    schema_version: "cognitiveos.task-preview-request/0.1",
    task_contract_draft: draft,
  });
  const admitted = await httpJson(origin, "POST", "/task/admit", token, {
    schema_version: "cognitiveos.task-admit-request/0.1",
    expected_current_epoch: 0,
    preview_digest: previewed.json.preview_digest,
    task_contract_draft: draft,
    acceptance: {
      accepted_by: "principal://local/owner",
      accepted_digest: interpreted.json.interpretation_digest,
      interpretation_id: interpreted.json.interpretation_id,
    },
  });
  if (admitted.json.task_ref !== spec.taskRef) {
    throw new Error(`admit failed for ${spec.family}: ${JSON.stringify(admitted.json)}`);
  }
  return spec.taskRef;
}

export async function waitLifecycle(origin, token, taskRef, options = {}) {
  const want = options.want ?? "COMPLETED";
  const attempts = options.attempts ?? 120;
  const delayMs = options.delayMs ?? 250;
  const encoded = encodeURIComponent(taskRef);
  let lifecycle = "absent";
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const evidence = await httpJson(origin, "GET", `/task/evidence?task_ref=${encoded}`, token);
    lifecycle = evidence.json?.lifecycle?.current_state ?? "absent";
    if (lifecycle === want) return lifecycle;
    if (lifecycle === "FAILED" || lifecycle === "REJECTED" || lifecycle === "CANCELLED") {
      return lifecycle;
    }
    await new Promise((resolve) => setTimeout(resolve, delayMs));
  }
  return lifecycle;
}
