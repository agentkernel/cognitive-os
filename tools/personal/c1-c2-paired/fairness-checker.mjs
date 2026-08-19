/**
 * Execution-plan §2.3 fairness checker. Observability only; not B0.
 */

export const FAIRNESS_AXES = Object.freeze([
  "pi_package_version_sri",
  "node_version",
  "provider_base_url_model",
  "system_task_prompt_bytes",
  "task_input_digest",
  "sampling_parameters",
  "timeout_retry0_max_turn",
  "visible_tool_set_schema",
  "workspace_snapshot",
  "network_policy",
  "cpu_memory_cwd_fs",
  "oracle_version",
  "warm_cold_stratum",
]);

export const DECLARED_DIFFERENCES = Object.freeze([
  "p_skips_cognitiveos",
  "o_uses_extension_daemon_governed_surface",
]);

function axisValue(arm, axis) {
  if (arm == null || typeof arm !== "object" || !Object.hasOwn(arm, axis)) {
    return undefined;
  }
  return arm[axis];
}

export function checkFairness({ p, o, declared_differences = DECLARED_DIFFERENCES } = {}) {
  const axes = [];
  let failed = 0;
  for (const axis of FAIRNESS_AXES) {
    const left = axisValue(p, axis);
    const right = axisValue(o, axis);
    const present = left !== undefined && right !== undefined;
    const equal = present && JSON.stringify(left) === JSON.stringify(right);
    const status = !present ? "fail_missing" : equal ? "pass" : "fail_mismatch";
    if (status !== "pass") {
      failed += 1;
    }
    axes.push({ axis, status });
  }
  return Object.freeze({
    kind: "c1-c2-fairness-record",
    retry: 0,
    counted_sample: false,
    b0: false,
    declared_differences: [...declared_differences],
    axes,
    result: failed === 0 ? "pass" : "fail",
    failed_axes: failed,
  });
}
