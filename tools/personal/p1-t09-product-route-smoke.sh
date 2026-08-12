#!/usr/bin/env bash
# P1-T09 reproducible first-response route probe.
#
# This runner accepts only executable and Extension paths. It configures the
# non-secret product Pi client, verifies redacted daemon readiness, and asks
# the pinned Pi binary for one bounded response through the deployed Extension.
# It never prints the doctor document, Pi output, Provider configuration,
# secrets, SecretRefs, SQLite paths, model identifiers, or response contents.

set -euo pipefail

readonly DEFAULT_TIMEOUT_SECONDS=90
readonly DEFAULT_EXPECTED_MARKER="cognitiveos-first-response-ok"
readonly READINESS_RETRY_ATTEMPTS=10

cognitive_executable=""
pi_executable=""
extension_entry=""
timeout_seconds="$DEFAULT_TIMEOUT_SECONDS"
expected_marker="$DEFAULT_EXPECTED_MARKER"

print_usage() {
    cat <<'EOF'
Usage:
  p1-t09-product-route-smoke.sh \
    --cognitive <absolute-path> \
    --pi <absolute-path> \
    --extension <absolute-path> \
    [--timeout-seconds <positive-integer>] \
    [--expected-marker <non-secret-marker>]
EOF
}

fail_with_usage() {
    printf '%s\n' "$1" >&2
    print_usage >&2
    exit 2
}

require_absolute_regular_file() {
    local description="$1"
    local candidate_path="$2"
    if [[ "$candidate_path" != /* ]] || [[ ! -f "$candidate_path" ]]; then
        printf '{"status":"error","phase":"preflight","error_class":"invalid_%s","authority_side_effects":false}\n' "$description"
        exit 2
    fi
}

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --cognitive)
            [[ "$#" -ge 2 ]] || fail_with_usage "--cognitive requires a value"
            cognitive_executable="$2"
            shift 2
            ;;
        --pi)
            [[ "$#" -ge 2 ]] || fail_with_usage "--pi requires a value"
            pi_executable="$2"
            shift 2
            ;;
        --extension)
            [[ "$#" -ge 2 ]] || fail_with_usage "--extension requires a value"
            extension_entry="$2"
            shift 2
            ;;
        --timeout-seconds)
            [[ "$#" -ge 2 ]] || fail_with_usage "--timeout-seconds requires a value"
            timeout_seconds="$2"
            shift 2
            ;;
        --expected-marker)
            [[ "$#" -ge 2 ]] || fail_with_usage "--expected-marker requires a value"
            expected_marker="$2"
            shift 2
            ;;
        --help|-h)
            print_usage
            exit 0
            ;;
        *)
            fail_with_usage "unexpected argument"
            ;;
    esac
done

[[ -n "$cognitive_executable" ]] || fail_with_usage "--cognitive is required"
[[ -n "$pi_executable" ]] || fail_with_usage "--pi is required"
[[ -n "$extension_entry" ]] || fail_with_usage "--extension is required"
[[ "$timeout_seconds" =~ ^[1-9][0-9]*$ ]] || fail_with_usage "--timeout-seconds must be positive"
[[ "$expected_marker" =~ ^[A-Za-z0-9._-]+$ ]] || fail_with_usage "--expected-marker must be a non-secret token"

require_absolute_regular_file "cognitive_executable" "$cognitive_executable"
require_absolute_regular_file "pi_executable" "$pi_executable"
require_absolute_regular_file "extension_entry" "$extension_entry"

run_directory="$(mktemp -d)"
trap 'rm -rf "$run_directory"' EXIT

configuration_output="$run_directory/configure.json"
doctor_output="$run_directory/doctor.json"
pi_version_output="$run_directory/pi-version.txt"
pi_response_output="$run_directory/pi-response.txt"

if ! "$cognitive_executable" pi configure \
    --executable "$pi_executable" \
    --extension-entry "$extension_entry" >"$configuration_output" 2>&1; then
    printf '{"status":"error","phase":"configure","error_class":"cognitive_configure_failed","authority_side_effects":false}\n'
    exit 3
fi

first_conversation_ready=false
for readiness_attempt in $(seq 1 "$READINESS_RETRY_ATTEMPTS"); do
    if ! "$cognitive_executable" doctor >"$doctor_output" 2>&1; then
        printf '{"status":"error","phase":"doctor","error_class":"cognitive_doctor_failed","authority_side_effects":false}\n'
        exit 4
    fi

    if node -e '
const fs = require("node:fs");
const document = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
if (document.overall !== "ready" || document.first_conversation_ready !== true) {
  process.exit(1);
}
' "$doctor_output"; then
        first_conversation_ready=true
        break
    fi

    if [[ "$readiness_attempt" -lt "$READINESS_RETRY_ATTEMPTS" ]]; then
        sleep 1
    fi
done

if [[ "$first_conversation_ready" != true ]]; then
    printf '{"status":"error","phase":"doctor","error_class":"first_conversation_not_ready","authority_side_effects":false}\n'
    exit 5
fi

if ! timeout 5 env -i \
    HOME="$HOME" \
    LOGNAME="${LOGNAME:-}" \
    PATH="$PATH" \
    TMPDIR="${TMPDIR:-/tmp}" \
    USER="${USER:-}" \
    XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}" \
    XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}" \
    XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}" \
    XDG_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}" \
    XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
    "$pi_executable" --version >"$pi_version_output" 2>&1; then
    printf '{"status":"error","phase":"pi_version","error_class":"pi_version_probe_failed","authority_side_effects":false}\n'
    exit 6
fi

if ! grep -Eq '(^|[^0-9])0\.81\.1([^0-9]|$)' "$pi_version_output"; then
    printf '{"status":"error","phase":"pi_version","error_class":"pi_version_mismatch","authority_side_effects":false}\n'
    exit 7
fi

start_nanoseconds="$(date +%s%N)"
set +e
timeout "$timeout_seconds" env -i \
    HOME="$HOME" \
    LOGNAME="${LOGNAME:-}" \
    PATH="$PATH" \
    TMPDIR="${TMPDIR:-/tmp}" \
    USER="${USER:-}" \
    XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}" \
    XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}" \
    XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}" \
    XDG_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}" \
    XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
    "$pi_executable" \
        --extension "$extension_entry" \
        --provider cognitiveos \
        --print "Reply exactly: $expected_marker" \
        </dev/null >"$pi_response_output" 2>&1
pi_exit_code="$?"
set -e
end_nanoseconds="$(date +%s%N)"
duration_milliseconds="$(( (end_nanoseconds - start_nanoseconds) / 1000000 ))"

response_received=false
expected_reply_observed=false
if [[ -s "$pi_response_output" ]]; then
    response_received=true
fi
if grep -Fqx "$expected_marker" "$pi_response_output"; then
    expected_reply_observed=true
fi

if [[ "$pi_exit_code" -eq 0 && "$expected_reply_observed" == true ]]; then
    printf '{"status":"ok","phase":"first_response","duration_ms":%s,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}\n' "$duration_milliseconds"
    exit 0
fi

error_class="pi_nonzero_exit"
if [[ "$pi_exit_code" -eq 124 ]]; then
    error_class="pi_timeout"
elif [[ "$pi_exit_code" -eq 0 ]]; then
    error_class="unexpected_pi_response"
fi
printf '{"status":"error","phase":"first_response","error_class":"%s","duration_ms":%s,"expected_reply_observed":%s,"response_received":%s,"authority_side_effects":false}\n' \
    "$error_class" "$duration_milliseconds" "$expected_reply_observed" "$response_received"
exit 8
