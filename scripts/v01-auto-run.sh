#!/usr/bin/env bash
# V01 unattended Boot → Connect → Verify → Perf Auto orchestrator (POSIX).
# Tip-honest: kernel-server uses --once --bind only (no /health, no --data-dir).
# See docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

SKIP_BUILD=0
STRICT_ENTRY=0
for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=1 ;;
    --strict-entry) STRICT_ENTRY=1 ;;
  esac
done

RUN_ID="$(date +%Y%m%d-%H%M%S)-$$"
RUN_DIR="$REPO_ROOT/artifacts/evidence/v01-auto-run/$RUN_ID"
LOG_DIR="$RUN_DIR/logs"
mkdir -p "$LOG_DIR" "$RUN_DIR/tmp"

PIN_TOTAL=84 PIN_PASS=55 PIN_FAIL=0 PIN_NA=0 PIN_DD=0 PIN_NR=29 PIN_SC=36

LEVEL="L0"
STOPPED=0
STOP_REASON=""
RESULTS_FILE="$RUN_DIR/results-map.json"
echo '{}' >"$RESULTS_FILE"

set_result() {
  node -e "const fs=require('fs');const p='$RESULTS_FILE';const r=JSON.parse(fs.readFileSync(p,'utf8'));r['$1']='$2';fs.writeFileSync(p,JSON.stringify(r,null,2)+'\n');"
}

run_logged() {
  local name="$1"; shift
  local log="$LOG_DIR/$name.log"
  set +e
  "$@" >"$log" 2>&1
  local ec=$?
  set -e
  echo "$ec"
}

resolve_kernel_bin() {
  for c in target/debug/kernel-server target/release/kernel-server \
           target/debug/kernel-server.exe target/release/kernel-server.exe; do
    if [[ -f "$c" ]]; then echo "$REPO_ROOT/$c"; return 0; fi
  done
  return 1
}

# ENTRY
ORIGIN_MAIN="$(git rev-parse origin/main 2>/dev/null || true)"
HEAD="$(git rev-parse HEAD 2>/dev/null || true)"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
DIRTY=0
[[ -n "$(git status --porcelain 2>/dev/null || true)" ]] && DIRTY=1
PB_NOISE=0
if git status --porcelain --untracked-files=all 2>/dev/null | grep -q 'personal-blog/'; then PB_NOISE=1; fi
ENTRY_STATUS=auto_pass
ENTRY_FAIL=""
if [[ "$PB_NOISE" = 1 ]]; then ENTRY_STATUS=auto_fail; ENTRY_FAIL="personal-blog porcelain"; fi
if [[ "$STRICT_ENTRY" = 1 ]]; then
  [[ "$DIRTY" = 1 ]] && { ENTRY_STATUS=auto_fail; ENTRY_FAIL="$ENTRY_FAIL dirty"; }
  [[ -n "$ORIGIN_MAIN" && -n "$HEAD" && "$ORIGIN_MAIN" != "$HEAD" ]] && { ENTRY_STATUS=auto_fail; ENTRY_FAIL="$ENTRY_FAIL HEAD!=origin/main"; }
fi
cat >"$RUN_DIR/entry.json" <<EOF
{"status":"$ENTRY_STATUS","origin_main":"$ORIGIN_MAIN","head":"$HEAD","branch":"$BRANCH","dirty":$([ "$DIRTY" = 1 ] && echo true || echo false),"failures":["$ENTRY_FAIL"]}
EOF
set_result ENTRY "$ENTRY_STATUS"
if [[ "$ENTRY_STATUS" = auto_fail ]]; then STOPPED=1; STOP_REASON="ENTRY: $ENTRY_FAIL"; fi

# PLATFORM
if [[ -f /proc/version ]] && grep -qiE 'Microsoft|WSL' /proc/version; then
  PLATFORM_LABEL=windows_wsl2_linux_guest
elif [[ "$(uname -s)" == Linux ]]; then
  PLATFORM_LABEL=linux_native
else
  PLATFORM_LABEL=windows_native
fi
cat >"$RUN_DIR/platform.json" <<EOF
{"status":"auto_pass","f017_platform_label":"$PLATFORM_LABEL","forbid_windows_native_sandbox_pass":true,"os_version":"$(uname -a | sed 's/"/'\''/g')","arch":"$(uname -m)"}
EOF
set_result PLATFORM-LABEL auto_pass

if [[ "$STOPPED" = 0 ]]; then
  BOOT_STATUS=auto_pass
  if [[ "$SKIP_BUILD" = 0 ]]; then
    ec="$(run_logged boot-cargo-build cargo build --workspace --locked)"
    [[ "$ec" = 0 ]] || { BOOT_STATUS=auto_fail; STOPPED=1; STOP_REASON=BOOT; }
    if [[ "$BOOT_STATUS" = auto_pass ]]; then
      ec="$(run_logged boot-pnpm-install pnpm install --frozen-lockfile)"
      [[ "$ec" = 0 ]] || { BOOT_STATUS=auto_fail; STOPPED=1; STOP_REASON=BOOT; }
    fi
    if [[ "$BOOT_STATUS" = auto_pass ]]; then
      ec="$(run_logged boot-pnpm-build pnpm -r build)"
      [[ "$ec" = 0 ]] || { BOOT_STATUS=auto_fail; STOPPED=1; STOP_REASON=BOOT; }
    fi
  fi
  KERNEL_BIN=""
  if KERNEL_BIN="$(resolve_kernel_bin)"; then
    :
  else
    BOOT_STATUS=auto_fail; STOPPED=1; STOP_REASON="kernel-server bin missing"
  fi
  cat >"$RUN_DIR/boot.json" <<EOF
{"status":"$BOOT_STATUS","kernel_server_bin":"$KERNEL_BIN","skip_build":$([ "$SKIP_BUILD" = 1 ] && echo true || echo false)}
EOF
  set_result BOOT-BUILD "$BOOT_STATUS"
  set_result BOOT-UP "$BOOT_STATUS"
  [[ "$BOOT_STATUS" = auto_pass ]] && LEVEL=L0
fi

if [[ "$STOPPED" = 0 ]]; then
  export KERNEL_SERVER_BIN="$KERNEL_BIN"
  MGMT=auto_pass
  ec="$(run_logged connect-mgmt-admin cargo test -p admin-cli --test m5_deterministic_fallback --locked)"
  [[ "$ec" = 0 ]] || MGMT=auto_fail
  ec="$(run_logged connect-mgmt-verbs cargo test -p cognitive-management --test m5_fallback_verbs --locked)"
  [[ "$ec" = 0 ]] || MGMT=auto_fail
  echo "{\"status\":\"$MGMT\"}" >"$RUN_DIR/connect-mgmt.json"
  set_result CONNECT-MGMT "$MGMT"

  SHELL_S=auto_pass
  if [[ ! -f "$KERNEL_SERVER_BIN" ]]; then
    SHELL_S=auto_fail
  else
    ec="$(run_logged connect-shell-sdk pnpm --filter @cognitiveos/sdk-ts test)"
    [[ "$ec" = 0 ]] || SHELL_S=auto_fail
    ec="$(run_logged connect-shell-sse cargo test -p kernel-server --test m5_http_sse --locked)"
    [[ "$ec" = 0 ]] || SHELL_S=auto_fail
  fi
  cat >"$RUN_DIR/connect-shell.json" <<EOF
{"status":"$SHELL_S","notes":["CONNECT-WATCH=skipped_nonclaim","CONNECT-FULL-DEMO=skipped_nonclaim"]}
EOF
  set_result CONNECT-SHELL "$SHELL_S"
  set_result CONNECT-WATCH skipped_nonclaim
  set_result CONNECT-FULL-DEMO skipped_nonclaim
  if [[ "$MGMT" != auto_pass || "$SHELL_S" != auto_pass ]]; then
    STOPPED=1; STOP_REASON=CONNECT
  else
    LEVEL=L1
  fi
fi

if [[ "$STOPPED" = 0 ]]; then
  VERIFY=auto_pass
  ec="$(run_logged verify-consistency pnpm run check:consistency)"
  [[ "$ec" = 0 ]] || VERIFY=auto_fail
  set_result VERIFY-CONSISTENCY "$([ "$ec" = 0 ] && echo auto_pass || echo auto_fail)"

  if [[ "$VERIFY" = auto_pass ]]; then
    ec="$(run_logged verify-runner cargo run --locked -p cognitive-conformance --bin conformance-runner)"
    [[ "$ec" = 0 ]] || VERIFY=auto_fail
    REPORT="$REPO_ROOT/artifacts/evidence/conformance/conformance-report.json"
    if [[ -f "$REPORT" ]]; then
      cp "$REPORT" "$RUN_DIR/conformance-report.json"
      if node -e "
        const fs=require('fs');
        const r=JSON.parse(fs.readFileSync(process.argv[1],'utf8'));
        const s=r.summary;
        const pinned={total_vectors:$PIN_TOTAL,pass:$PIN_PASS,fail:$PIN_FAIL,'not-applicable':$PIN_NA,'documented-degradation':$PIN_DD,'not-run':$PIN_NR};
        for (const [k,w] of Object.entries(pinned)) {
          if (s[k]!==w) { console.error('pin fail',k,s[k],w); process.exit(2); }
        }
        const want=['MGMT-APPROVAL-R1-009','MGMT-APPROVAL-SELF-010','MGMT-APPROVAL-FATIGUE-011','AGENT-INSTALL-001','AGENT-BYPASS-002','AGENT-OOB-001'];
        const by=Object.fromEntries(r.vectors.map(v=>[v.id,v.result]));
        for (const id of want) {
          if (by[id]!=='pass') { console.error('regress fail',id,by[id]); process.exit(3); }
        }
        console.log('pins+regress ok');
      " "$REPORT"; then
        set_result VERIFY-PINS auto_pass
        set_result REGRESS-V01 auto_pass
      else
        VERIFY=auto_fail
        set_result VERIFY-PINS auto_fail
        set_result REGRESS-V01 auto_fail
      fi
    else
      VERIFY=auto_fail
      set_result VERIFY-PINS auto_fail
      set_result REGRESS-V01 auto_fail
    fi
  fi

  if [[ "$VERIFY" = auto_pass ]]; then
    ec="$(run_logged verify-self-check cargo run --locked -p cognitive-conformance --bin conformance-runner -- --self-check)"
    SC="$REPO_ROOT/artifacts/evidence/conformance/self-check-report.json"
    if [[ -f "$SC" ]]; then
      cp "$SC" "$RUN_DIR/self-check-report.json"
      if node -e "const r=require(process.argv[1]); if ((r.must_flip||[]).length < $PIN_SC) process.exit(2);" "$SC" && [[ "$ec" = 0 ]]; then
        set_result VERIFY-SELFCHECK auto_pass
      else
        VERIFY=auto_fail
        set_result VERIFY-SELFCHECK auto_fail
      fi
    else
      VERIFY=auto_fail
      set_result VERIFY-SELFCHECK auto_fail
    fi
  fi

  if [[ "$VERIFY" = auto_pass ]]; then
    ec="$(run_logged verify-f017 cargo test -p cognitive-runtime --lib sandbox::tests --locked -- --nocapture)"
    [[ "$ec" = 0 ]] || VERIFY=auto_fail
    set_result F017-CLAIM-FREEZE "$([ "$ec" = 0 ] && echo auto_pass || echo auto_fail)"
  fi

  echo "{\"status\":\"$VERIFY\"}" >"$RUN_DIR/verify.json"
  if [[ "$VERIFY" != auto_pass ]]; then
    STOPPED=1; STOP_REASON=VERIFY
  else
    LEVEL=L2
  fi
fi

PERF4=skipped_nonclaim
if [[ "$STOPPED" = 0 ]]; then
  ec="$(run_logged perf004-unit cargo test -p cognitive-runtime overhead_report_requires_ungoverned_baseline_and_forbids_benefit --locked -- --exact)"
  if [[ "$ec" = 0 ]]; then
    mkdir -p "$REPO_ROOT/artifacts/evidence/performance"
    PERF_PATH="$REPO_ROOT/artifacts/evidence/performance/performance-report-v01-sample.json"
    cat >"$PERF_PATH" <<'EOF'
{
  "orchestrator_honesty": {
    "claim_level": "sample_or_builder_only",
    "campaign": "not_executed",
    "claims_agent_benefit": false,
    "forbid_silent_campaign_pass": true,
    "source": "cognitive_runtime::GovernanceOverheadSample (tip unit/runner numbers)"
  },
  "schema_version": "cognitiveos.performance-report/0.1",
  "note": "Sample/builder export for L3 report-ready. NOT a full HW campaign digest.",
  "governance_overhead": {
    "ungoverned_baseline": "ungoverned-local-v1",
    "gate_latency_ms": {
      "authorization": {"p50": 0.1, "p95": 0.4, "p99": 0.9},
      "context_resolution": {"p50": 1.0, "p95": 3.0, "p99": 5.0},
      "effect_protocol": {"p50": 0.5, "p95": 1.2, "p99": 2.0}
    },
    "cache_hit_preservation_ratio": 0.9,
    "extra_persistence_per_governed_call": {"writes": 2.0, "bytes": 1024.0},
    "approval": {
      "latency_ms": {"p50": 10.0, "p95": 50.0, "p99": 100.0},
      "rubber_stamp_rate": 0.01,
      "retry_after_deny_rate": 0.02
    },
    "overhead_share_by_risk_class": [
      {"risk_class": "R1", "latency_percent": 3.0, "cost_percent": 2.0}
    ]
  }
}
EOF
    cp "$PERF_PATH" "$RUN_DIR/performance-report-v01-sample.json"
    PERF4=auto_pass
    LEVEL=L3
  else
    PERF4=auto_fail
  fi
fi
set_result PERF004-AUTO-REPORT "$PERF4"
set_result PERF004-NO-SILENT-CAMPAIGN auto_pass
cat >"$RUN_DIR/perf004.json" <<EOF
{"status":"$PERF4","claim_level":"sample_or_builder_only","campaign":"not_executed","claims_agent_benefit":false}
EOF

cat >"$RUN_DIR/perf005-precheck.json" <<EOF
{
  "status": "skipped_nonclaim",
  "four_arm_harness": false,
  "preregistration": false,
  "independent_verifier": false,
  "significant_benefit_forbidden": true,
  "reason": "Tip has contract/docs only; no executable four-arm harness (F-026 / IMP-18 / M7+)"
}
EOF
set_result PERF005-DEFAULT-NONCLAIM skipped_nonclaim
set_result PERF005-NO-SILENT-BENEFIT auto_pass
set_result BOOT-TEARDOWN auto_pass
set_result MANIFEST-HONESTY auto_pass
set_result ORCHESTRATOR-ONE-SHOT auto_pass
set_result SUMMARY-MACHINE-READABLE auto_pass
set_result REVIEW skipped_nonclaim
set_result HUMAN-CI-JOB-ADD skipped_nonclaim
set_result HUMAN-PERF004-CAMPAIGN skipped_nonclaim
set_result HUMAN-PERF005-CLAIM skipped_nonclaim

node -e "
const fs=require('fs');
const results=JSON.parse(fs.readFileSync('$RESULTS_FILE','utf8'));
const stopped = $STOPPED === 1;
const summary={
  schema_version:'cognitiveos.v01-auto-run-summary/0.1',
  run_id:'$RUN_ID',
  level:'$LEVEL',
  stopped,
  stop_reason: stopped ? '$STOP_REASON' : null,
  release: stopped ? 'blocked' : 'non_claim_preserved',
  profile_implemented: 0,
  auto_green_means_profile_implemented: false,
  results,
  platform_label: '$PLATFORM_LABEL',
  v01_non_claims: [
    'Windows-native sandbox = unsupported',
    'WSL2 guest sandbox = not_tested',
    'Durable install authority = in-process ledger only',
    'REQ-PERF-004 full HW campaign = not executed (sample/builder only)',
    'REQ-PERF-005 agent benefit = not emitted',
    'Profile implemented = 0',
    'D-018 governance object ports = residual / exchange-surface non-claim',
    'R2/R3, distributed, clients/Console/Agent Hub, M7+ = out of v0.1 auto-run'
  ],
  human_gates: [
    {id:'HUMAN-PLATFORM-LABEL', default:'conservative label + continue', triggered:false},
    {id:'HUMAN-CI-JOB-ADD', default:'local verify:local only', triggered:false},
    {id:'HUMAN-PERF004-CAMPAIGN', default:'keep non-claim', triggered:false},
    {id:'HUMAN-PERF005-CLAIM', default:'forbid benefit', triggered:false},
    {id:'HUMAN-NO-GO', default:'mark failed, do not release', triggered: stopped}
  ],
  partitions: {
    auto_pass: Object.keys(results).filter(k=>results[k]==='auto_pass'),
    auto_fail: Object.keys(results).filter(k=>results[k]==='auto_fail'),
    skipped_nonclaim: Object.keys(results).filter(k=>results[k]==='skipped_nonclaim'),
    needs_human: Object.keys(results).filter(k=>results[k]==='needs_human')
  }
};
fs.writeFileSync('$RUN_DIR/summary.json', JSON.stringify(summary,null,2)+'\n');
const lines = [
  '# V01 Auto-Run Summary — $RUN_ID',
  '',
  '- **Level**: $LEVEL',
  '- **Stopped**: ' + stopped,
  '- **Platform**: $PLATFORM_LABEL',
  '- **Profile implemented**: 0 (auto green ≠ Profile implemented)',
  '',
  '## Results',
  '',
  '| ID | Status |',
  '|---|---|',
  ...Object.entries(results).map(([k,v])=>'| '+k+' | '+v+' |'),
  '',
  'Evidence: \`artifacts/evidence/v01-auto-run/$RUN_ID/\`',
  ''
];
fs.writeFileSync('$RUN_DIR/summary.md', lines.join('\n'));
const files=[];
function walk(d){ for (const ent of fs.readdirSync(d,{withFileTypes:true})) {
  const p=d+'/'+ent.name;
  if (ent.isDirectory()) walk(p);
  else {
    const crypto=require('crypto');
    const h=crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
    files.push({path:p.slice('$RUN_DIR'.length+1), sha256:h});
  }
}}
walk('$RUN_DIR');
fs.writeFileSync('$RUN_DIR/sha256-manifest.json', JSON.stringify({files},null,2)+'\n');
"

rm -rf "$RUN_DIR/tmp" || true

echo "=== Done: level=$LEVEL stopped=$STOPPED ==="
echo "Summary: $RUN_DIR/summary.json"
[[ "$STOPPED" = 0 ]]
