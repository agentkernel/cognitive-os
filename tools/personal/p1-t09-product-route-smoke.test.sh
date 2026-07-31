#!/usr/bin/env bash
# Focused negatives for the redacted P1-T09 route runner.

set -euo pipefail

script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
runner="$script_directory/p1-t09-product-route-smoke.sh"
fixture_directory="$(mktemp -d)"
trap 'rm -rf "$fixture_directory"' EXIT

assert_failure() {
    local expected_exit_code="$1"
    local expected_error_class="$2"
    shift 2
    set +e
    output="$(bash "$runner" "$@" 2>&1)"
    actual_exit_code="$?"
    set -e
    [[ "$actual_exit_code" -eq "$expected_exit_code" ]]
    [[ "$output" == *"$expected_error_class"* ]]
}

assert_failure 2 "--cognitive is required" \
    --pi /missing/pi --extension /missing/extension
assert_failure 2 "invalid_cognitive_executable" \
    --cognitive relative-cognitive --pi /missing/pi --extension /missing/extension
assert_failure 2 "--timeout-seconds must be positive" \
    --cognitive /missing/cognitive --pi /missing/pi --extension /missing/extension \
    --timeout-seconds 0
assert_failure 2 "--expected-marker must be a non-secret token" \
    --cognitive /missing/cognitive --pi /missing/pi --extension /missing/extension \
    --expected-marker "not safe"

fake_cognitive="$fixture_directory/cognitive"
fake_pi="$fixture_directory/pi"
fake_extension="$fixture_directory/index.js"
fake_pi_environment="$fixture_directory/pi-environment"
fake_pi_arguments="$fixture_directory/pi-arguments"
fake_pi_stdin="$fixture_directory/pi-stdin"
fake_node="$fixture_directory/node"
cat > "$fake_cognitive" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == "pi" && "$2" == "configure" ]]; then
    printf '{"status":"ok"}\n'
elif [[ "$1" == "doctor" ]]; then
    doctor_counter_file="$TMPDIR/doctor-count"
    doctor_invocation_count=0
    if [[ -f "$doctor_counter_file" ]]; then
        doctor_invocation_count="$(<"$doctor_counter_file")"
    fi
    doctor_invocation_count="$((doctor_invocation_count + 1))"
    printf '%s\n' "$doctor_invocation_count" > "$doctor_counter_file"
    if [[ "$doctor_invocation_count" -lt 2 ]]; then
        printf '{"status":"ok","overall":"degraded","first_conversation_ready":false}\n'
    else
        printf '{"overall":"ready","first_conversation_ready":true}\n'
    fi
else
    exit 1
fi
EOF
cat > "$fake_pi" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == "--version" ]]; then
    printf 'pi 0.81.1\n'
    exit 0
fi
env | sort > "$TMPDIR/pi-environment"
printf '%s\n' "$@" > "$TMPDIR/pi-arguments"
if [[ "$(readlink /proc/self/fd/0)" != "/dev/null" ]]; then
    printf 'stdin must be closed for Pi print mode\n' >&2
    exit 91
fi
printf 'closed\n' > "$TMPDIR/pi-stdin"
printf 'cognitiveos-first-response-ok\n'
EOF
cat > "$fake_node" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
validation_script="$2"
doctor_document="$3"
if [[ "$validation_script" == *"document.status"* ]]; then
    exit 1
fi
if [[ "$(<"$doctor_document")" == *'"overall":"ready"'* && "$(<"$doctor_document")" == *'"first_conversation_ready":true'* ]]; then
    exit 0
fi
exit 1
EOF
printf 'export {};\n' > "$fake_extension"
chmod 700 "$fake_cognitive" "$fake_pi" "$fake_node"

successful_output="$(PATH="$fixture_directory:/usr/bin:/bin" PROVIDER_API_KEY=must-not-reach-pi TMPDIR="$fixture_directory" \
    bash "$runner" \
        --cognitive "$fake_cognitive" \
        --pi "$fake_pi" \
        --extension "$fake_extension" \
        --timeout-seconds 1)"
[[ "$successful_output" == *'"status":"ok"'* ]]
[[ "$successful_output" == *'"expected_reply_observed":true'* ]]
[[ ! -s "$fake_pi_environment" || "$(<"$fake_pi_environment")" != *"PROVIDER_API_KEY="* ]]
[[ "$(<"$fixture_directory/doctor-count")" -ge 2 ]]
[[ "$(<"$fake_pi_arguments")" == *$'--provider\ncognitiveos'* ]]
[[ "$(<"$fake_pi_stdin")" == "closed" ]]

printf 'p1-t09-product-route-smoke focused negatives: PASS\n'
