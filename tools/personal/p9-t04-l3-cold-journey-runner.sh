#!/usr/bin/env bash
# P9-T04 `L3` `R3` cold daemon journey runner.
#
# Measures the full cold path per sample: daemon stop, cache reset, daemon
# start, readiness poll, then one real Provider completion. Every started
# sample is retained, including a sample whose daemon never became ready or
# whose Provider request failed. Nothing is retried.
#
# It prints only durations, counts and registered status words. It never reads
# Provider configuration, never handles a credential, and never prints a
# response.
set -u

RUNTIME_ROOT=${1:?runtime root required}
BIN_DIR=${2:?binary directory required}
PROBE_DIR=${3:?probe directory required}
SOURCE_REVISION=${4:?source revision required}
SAMPLE_COUNT=${5:-20}
READINESS_TIMEOUT_MS=30000

export XDG_CONFIG_HOME="$RUNTIME_ROOT/config"
export XDG_DATA_HOME="$RUNTIME_ROOT/data"
export XDG_STATE_HOME="$RUNTIME_ROOT/state"
export XDG_CACHE_HOME="$RUNTIME_ROOT/cache"
export XDG_RUNTIME_DIR="$RUNTIME_ROOT"

now_ns() { date +%s%N; }

printf '{\n  "report_kind": "p9-t04-l3-cold-journey/0.1",\n'
printf '  "claim_level": "hypothesis",\n'
printf '  "scenario_id": "R3-cold-daemon-first-response",\n'
printf '  "source_revision": "%s",\n' "$SOURCE_REVISION"
printf '  "retry_budget": 0,\n'
printf '  "started_samples": %s,\n' "$SAMPLE_COUNT"
printf '  "samples": [\n'

sample=0
while [ "$sample" -lt "$SAMPLE_COUNT" ]; do
  sample=$((sample + 1))

  "$BIN_DIR/cognitive" daemon stop --runtime-root "$RUNTIME_ROOT" >/dev/null 2>&1
  rm -rf "$RUNTIME_ROOT/cache"

  start_ns=$(now_ns)
  if "$BIN_DIR/cognitive" daemon start --runtime-root "$RUNTIME_ROOT" \
      --kernel-server "$BIN_DIR/kernel-server" >/dev/null 2>&1; then
    start_status=started
  else
    start_status=start_failed
  fi

  ready_status=not_ready
  deadline_ns=$(( $(now_ns) + READINESS_TIMEOUT_MS * 1000000 ))
  while [ "$(now_ns)" -lt "$deadline_ns" ]; do
    if "$BIN_DIR/cognitive" status --runtime-root "$RUNTIME_ROOT" >/dev/null 2>&1; then
      ready_status=ready
      break
    fi
    sleep 0.1
  done
  ready_ns=$(now_ns)

  if [ "$ready_status" = ready ]; then
    completion_json=$(node "$PROBE_DIR/tools/personal/p9-t04-l3-provider-route-runner.mjs" \
      --source-revision "$SOURCE_REVISION" --scenario R3-cold-inner --samples 1 2>/dev/null)
    if [ -n "$completion_json" ]; then
      first_outcome=$(printf '%s' "$completion_json" \
        | python3 -c 'import json,sys; print(json.load(sys.stdin)["samples"][0]["outcome"])' 2>/dev/null)
    else
      first_outcome=outcome_unknown
    fi
  else
    first_outcome=not_attempted
  fi
  completed_ns=$(now_ns)

  separator=","
  [ "$sample" -eq "$SAMPLE_COUNT" ] && separator=""
  printf '    {"sample": %s, "start_status": "%s", "ready_status": "%s", "first_response_outcome": "%s", "startup_to_ready_nanos": %s, "ready_to_first_response_nanos": %s, "total_cold_journey_nanos": %s}%s\n' \
    "$sample" "$start_status" "$ready_status" "$first_outcome" \
    "$((ready_ns - start_ns))" "$((completed_ns - ready_ns))" "$((completed_ns - start_ns))" "$separator"
done

printf '  ]\n}\n'
