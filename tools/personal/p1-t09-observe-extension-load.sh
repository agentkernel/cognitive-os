#!/usr/bin/env bash
# P1-T09 exact-Pi-extension-load observation probe.
#
# Runs entirely on the qualified Linux-native experimental host and produces
# only redacted, session-local diagnostics. No Provider config, secret,
# SecretRef, SQLite path, selected-model, capability, Effect, Verification or
# authority side effect is created or read.
#
# Success criterion for the observation is not that Pi produces a completion:
# the isolated environment has no daemon endpoint, so the CognitiveOS
# daemon-provider MUST fail closed during Extension registration. What we do
# need to see is:
#   1) the built ESM module imports cleanly under Node.
#   2) Pi 0.81.1 invokes a session-local wrapper passed through `--extension`.
#      The wrapper leaves a marker before delegating directly to the actual
#      CognitiveOS default export. That export registers the production hooks
#      and command before its deliberate fail-closed daemon-provider check.
#
set -u
set +e

extension_path="${1:-/tmp/cognitiveos-p1-t09-ext-index.js}"
if [[ ! -f "$extension_path" ]]; then
    echo "MISSING_EXTENSION_FILE: $extension_path" >&2
    exit 2
fi
export EXT_PATH="$extension_path"

echo "=== step1: node ESM import of built extension ==="
node --input-type=module -e "
 import(process.env.EXT_PATH).then((mod) => {
   const exportNames = Object.keys(mod).sort().join(',');
   console.log('default_type=' + typeof mod.default);
   console.log('default_name=' + (mod.default && mod.default.name));
   console.log('has_registerCognitiveOsExtension=' + (typeof mod.registerCognitiveOsExtension === 'function'));
   console.log('has_PROJECT_TRUST_DECISION=' + (mod.PROJECT_TRUST_DECISION !== undefined));
   console.log('exports=' + exportNames);
 }).catch((err) => {
   console.log('IMPORT_ERROR code=' + err.code + ' name=' + err.name + ' message=' + err.message);
   process.exit(3);
 });
" || {
    echo "===STEP1_FAILED==="
    exit 3
}

echo "=== step2: real pi 0.81.1 --extension invocation observation ==="
run_root=$(mktemp -d)
mkdir -p "$run_root/runtime" "$run_root/cfg" "$run_root/data" "$run_root/state"
marker_file="$run_root/pi-extension-default-export-invoked"
wrapper_file="$run_root/observe-cognitiveos-extension.mjs"

# The wrapper is session-local instrumentation. It has no CognitiveOS authority
# and delegates immediately to the actual built extension default export.
cat > "$wrapper_file" <<'EOF'
import { writeFileSync } from "node:fs";

const cognitiveOsExtension = await import(process.env.COGNITIVEOS_REAL_EXTENSION_PATH);

export default async function observeCognitiveOsExtension(pi) {
  writeFileSync(process.env.COGNITIVEOS_INVOCATION_MARKER, "invoked\n", { mode: 0o600 });
  return cognitiveOsExtension.default(pi);
}
EOF

EXT_PATH="$extension_path" timeout 45 env -i \
    HOME="$HOME" \
    PATH="$PATH" \
    EXT_PATH="$extension_path" \
    COGNITIVEOS_REAL_EXTENSION_PATH="$extension_path" \
    COGNITIVEOS_INVOCATION_MARKER="$marker_file" \
    TMPDIR="$run_root" \
    PI_OFFLINE=1 \
    XDG_CONFIG_HOME="$run_root/cfg" \
    XDG_DATA_HOME="$run_root/data" \
    XDG_STATE_HOME="$run_root/state" \
    XDG_RUNTIME_DIR="$run_root/runtime" \
    npx --yes @earendil-works/pi-coding-agent@0.81.1 \
        --verbose \
        --no-session \
        --no-tools \
        --no-extensions \
        --no-skills \
        --no-prompt-templates \
        --no-themes \
        --no-context-files \
        --offline \
        --extension "$wrapper_file" \
        --print "load-observation-only" 2>&1 | head -n 80
pi_exit=${PIPESTATUS[0]}
echo "===PI_EXIT=$pi_exit==="

if [[ -f "$marker_file" ]] && [[ "$(<"$marker_file")" == "invoked" ]]; then
    echo "===PI_EXTENSION_DEFAULT_EXPORT_INVOKED==="
else
    echo "===PI_EXTENSION_DEFAULT_EXPORT_NOT_OBSERVED==="
fi

rm -rf "$run_root"
exit 0
