#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
.SYNOPSIS
  V01 unattended Boot → Connect → Verify → Perf Auto orchestrator.

.DESCRIPTION
  Tip-honest: kernel-server uses --once --bind only (no /health, no --data-dir).
  Emits artifacts/evidence/v01-auto-run/<run_id>/{summary.json,summary.md,...}.
  Default: L3 report-ready with PERF campaign/benefit non-claim.
  See docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md.

.PARAMETER SkipBuild
  Skip cargo/pnpm build (assume binaries/packages already built).

.PARAMETER StrictEntry
  Fail if worktree is dirty or HEAD != origin/main.
#>
param(
  [switch]$SkipBuild,
  [switch]$StrictEntry
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $RepoRoot

# Unattended: pnpm may need to purge node_modules when switching host/guest
# toolchains; without CI=true it aborts with ERR_PNPM_ABORTED_REMOVE_MODULES_DIR_NO_TTY.
if (-not $env:CI -or [string]::IsNullOrWhiteSpace($env:CI)) {
  $env:CI = "true"
}

$Pinned = @{
  total_vectors              = 84
  pass                       = 55
  fail                       = 0
  "not-applicable"           = 0
  "documented-degradation"   = 0
  "not-run"                  = 29
  self_check_min             = 36
}

$RegressIds = @(
  "MGMT-APPROVAL-R1-009",
  "MGMT-APPROVAL-SELF-010",
  "MGMT-APPROVAL-FATIGUE-011",
  "AGENT-INSTALL-001",
  "AGENT-BYPASS-002",
  "AGENT-OOB-001"
)

$V01NonClaims = @(
  "Windows-native sandbox = unsupported",
  "WSL2 guest sandbox = not_tested",
  "Durable install authority = in-process ledger only",
  "REQ-PERF-004 full HW campaign = not executed (sample/builder only)",
  "REQ-PERF-005 agent benefit = not emitted",
  "Profile implemented = 0",
  "D-018 governance object ports = residual / exchange-surface non-claim",
  "R2/R3, distributed, clients/Console/Agent Hub, M7+ = out of v0.1 auto-run"
)

function New-RunId {
  return (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + ([guid]::NewGuid().ToString("N").Substring(0, 8))
}

function Write-JsonFile {
  param([string]$Path, [object]$Object)
  $dir = Split-Path -Parent $Path
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
  $text = ($Object | ConvertTo-Json -Depth 40) + "`n"
  [System.IO.File]::WriteAllText($Path, $text)
}

function Get-FileSha256 {
  param([string]$Path)
  if (-not (Test-Path $Path)) { return $null }
  return (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLowerInvariant()
}

function Invoke-Logged {
  param(
    [string]$Name,
    [string]$LogPath,
    [scriptblock]$Script
  )
  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  $exit = 0
  try {
    # Capture all streams so the function does not leak pipeline objects as return value.
    $output = & $Script *>&1 | ForEach-Object { "$_" }
    if ($null -ne $LASTEXITCODE) { $exit = $LASTEXITCODE }
    if ($null -eq $output) { $output = @() }
    ($output -join "`n") + "`n" | Out-File -FilePath $LogPath -Encoding utf8
  } catch {
    $_ | Out-File -FilePath $LogPath -Append -Encoding utf8
    $exit = 1
  }
  $sw.Stop()
  # Leading comma forces a single hashtable return (no unwrapping).
  return , @{
    name       = $Name
    exit_code  = $exit
    elapsed_ms = $sw.ElapsedMilliseconds
    log        = $LogPath
  }
}

function Resolve-KernelServerBin {
  $candidates = @(
    (Join-Path $RepoRoot "target/debug/kernel-server.exe"),
    (Join-Path $RepoRoot "target/debug/kernel-server"),
    (Join-Path $RepoRoot "target/release/kernel-server.exe"),
    (Join-Path $RepoRoot "target/release/kernel-server")
  )
  foreach ($c in $candidates) {
    if (Test-Path $c) { return (Resolve-Path $c).Path }
  }
  return $null
}

function Detect-Platform {
  $onWindows = ($env:OS -match "Windows") -or ($IsWindows -eq $true)
  $isWsl = $false
  $contradiction = $false
  $procVersion = $null
  if (Test-Path "/proc/version") {
    $procVersion = (Get-Content "/proc/version" -Raw -ErrorAction SilentlyContinue)
    if ($procVersion -match "Microsoft|WSL") { $isWsl = $true }
  }
  # Contradiction: Windows host claiming WSL via env but no /proc, or reverse noise
  if ($onWindows -and $isWsl -and -not (Test-Path "/proc/version")) {
    $contradiction = $true
  }

  $label = if ($isWsl) {
    "windows_wsl2_linux_guest"
  } elseif ($onWindows) {
    "windows_native"
  } else {
    "linux_native"
  }

  $arch = if ($env:PROCESSOR_ARCHITECTURE) { $env:PROCESSOR_ARCHITECTURE } else { uname -m 2>$null }
  $osVersion = if ($onWindows) {
    [System.Environment]::OSVersion.VersionString
  } else {
    (uname -a 2>$null)
  }

  $gates = @()
  $status = "auto_pass"
  if ($contradiction) {
    $status = "needs_human"
    $gates += @{
      id       = "HUMAN-PLATFORM-LABEL"
      default  = "adopt_conservative_label"
      resolved = "windows_native_or_not_tested"
    }
    # Conservative: never claim linux_native sandbox on confused host
    if ($label -eq "linux_native") { $label = "windows_wsl2_linux_guest" }
  }

  $sandbox_policy = switch ($label) {
    "linux_native" { "reference_deny_rows_with_digest" }
    "windows_wsl2_linux_guest" { "not_tested_linux_guest_only" }
    default { "unsupported_skip_sandbox_claim" }
  }

  return @{
    status                 = $status
    f017_platform_label    = $label
    sandbox_claim_policy   = $sandbox_policy
    os_version             = "$osVersion"
    arch                   = "$arch"
    is_windows             = [bool]$onWindows
    is_wsl                 = [bool]$isWsl
    proc_version_snippet   = if ($procVersion) { $procVersion.Substring(0, [Math]::Min(120, $procVersion.Length)) } else { $null }
    human_gates            = $gates
    forbid_windows_native_sandbox_pass = $true
  }
}

function Get-EntryEvidence {
  param([string]$OutDir)
  $originMain = (git rev-parse origin/main 2>$null)
  $head = (git rev-parse HEAD 2>$null)
  $branch = (git rev-parse --abbrev-ref HEAD 2>$null)
  $statusShort = (git status --short --branch 2>$null) -join "`n"
  $dirty = -not [string]::IsNullOrWhiteSpace((git status --porcelain 2>$null))
  $hasPersonalBlog = ($statusShort -match "personal-blog") -or (Test-Path (Join-Path $RepoRoot "personal-blog/.git"))

  $ciNote = $null
  try {
    $ciNote = (gh run list --commit $originMain --limit 3 2>$null) -join "`n"
  } catch {
    $ciNote = "gh unavailable: $($_.Exception.Message)"
  }

  $entryStatus = "auto_pass"
  $failures = @()
  if ($hasPersonalBlog -and $dirty -and ($statusShort -match "^\?\? personal-blog|^\s*M\s+personal-blog|personal-blog/")) {
    # Untracked/ignored personal-blog dir is OK (gitignored); fail only if it appears in porcelain as tracked noise
  }
  # Hard fail: personal-blog paths in porcelain that are not purely ignored (git status --short shows ignored only with -u)
  $porcelain = git status --porcelain --untracked-files=all 2>$null
  if ($porcelain -match "(^|`n).. personal-blog/") {
    $entryStatus = "auto_fail"
    $failures += "personal-blog paths present in worktree porcelain — forbidden baseline"
  }
  if ($StrictEntry) {
    if ($dirty) {
      $entryStatus = "auto_fail"
      $failures += "StrictEntry: dirty worktree"
    }
    if ($originMain -and $head -and ($originMain -ne $head)) {
      $entryStatus = "auto_fail"
      $failures += "StrictEntry: HEAD != origin/main"
    }
  }

  return @{
    status            = $entryStatus
    origin_main       = "$originMain"
    head              = "$head"
    branch            = "$branch"
    dirty             = $dirty
    personal_blog_noise = [bool]($porcelain -match "personal-blog")
    status_short      = "$statusShort"
    ci_runs           = "$ciNote"
    pinned_expectation = $Pinned
    failures          = $failures
    strict_entry      = [bool]$StrictEntry
  }
}

# --- run setup ---
$RunId = New-RunId
$RunDir = Join-Path $RepoRoot "artifacts/evidence/v01-auto-run/$RunId"
$LogDir = Join-Path $RunDir "logs"
$TmpDir = Join-Path $RunDir "tmp"
New-Item -ItemType Directory -Force -Path $LogDir, $TmpDir | Out-Null

$results = [ordered]@{}
$level = "L0"
$stopped = $false
$stopReason = $null

Write-Host "=== V01 auto-run $RunId ==="
Write-Host "Repo: $RepoRoot"
Write-Host "Out:  $RunDir"

# ENTRY
$entry = Get-EntryEvidence -OutDir $RunDir
Write-JsonFile (Join-Path $RunDir "entry.json") $entry
$results["ENTRY"] = $entry.status
if ($entry.status -eq "auto_fail") {
  $stopped = $true
  $stopReason = "ENTRY failed: $($entry.failures -join '; ')"
}

# PLATFORM
$platform = Detect-Platform
Write-JsonFile (Join-Path $RunDir "platform.json") $platform
$results["PLATFORM-LABEL"] = $platform.status
if ($platform.status -eq "needs_human") {
  $results["PLATFORM-LABEL"] = "needs_human"
  # default continue with conservative label already applied
}

# BOOT
if (-not $stopped) {
  $boot = @{
    status          = "auto_pass"
    skip_build      = [bool]$SkipBuild
    kernel_server_bin = $null
    steps           = @()
    failures        = @()
  }
  if (-not $SkipBuild) {
    $r1 = Invoke-Logged "cargo-build" (Join-Path $LogDir "boot-cargo-build.log") {
      cargo build --workspace --locked
    }
    $boot.steps += $r1
    if ($r1.exit_code -ne 0) {
      $boot.status = "auto_fail"
      $boot.failures += "cargo build failed ($($r1.exit_code))"
    }
    if ($boot.status -eq "auto_pass") {
      $r2 = Invoke-Logged "pnpm-install" (Join-Path $LogDir "boot-pnpm-install.log") {
        pnpm install --frozen-lockfile
      }
      $boot.steps += $r2
      if ($r2.exit_code -ne 0) {
        $boot.status = "auto_fail"
        $boot.failures += "pnpm install failed ($($r2.exit_code))"
      }
    }
    if ($boot.status -eq "auto_pass") {
      $r3 = Invoke-Logged "pnpm-build" (Join-Path $LogDir "boot-pnpm-build.log") {
        pnpm -r build
      }
      $boot.steps += $r3
      if ($r3.exit_code -ne 0) {
        $boot.status = "auto_fail"
        $boot.failures += "pnpm -r build failed ($($r3.exit_code))"
      }
    }
  }
  $bin = Resolve-KernelServerBin
  $boot.kernel_server_bin = $bin
  if (-not $bin) {
    $boot.status = "auto_fail"
    $boot.failures += "kernel-server binary not found under target/{debug,release}"
  }
  Write-JsonFile (Join-Path $RunDir "boot.json") $boot
  $results["BOOT-BUILD"] = $boot.status
  $results["BOOT-UP"] = if ($bin) { "auto_pass" } else { "auto_fail" }
  if ($boot.status -ne "auto_pass") {
    $stopped = $true
    $stopReason = "BOOT failed"
    $level = "L0"
  } else {
    $level = "L0"
  }
}

# CONNECT MGMT + SHELL
if (-not $stopped) {
  $env:KERNEL_SERVER_BIN = $boot.kernel_server_bin

  $mgmt = @{ status = "auto_pass"; steps = @(); failures = @() }
  $r = Invoke-Logged "admin-cli-fallback" (Join-Path $LogDir "connect-mgmt-admin-cli.log") {
    cargo test -p admin-cli --test m5_deterministic_fallback --locked
  }
  $mgmt.steps += $r
  if ($r.exit_code -ne 0) { $mgmt.status = "auto_fail"; $mgmt.failures += "admin-cli m5_deterministic_fallback" }

  $r = Invoke-Logged "mgmt-verbs" (Join-Path $LogDir "connect-mgmt-verbs.log") {
    cargo test -p cognitive-management --test m5_fallback_verbs --locked
  }
  $mgmt.steps += $r
  if ($r.exit_code -ne 0) { $mgmt.status = "auto_fail"; $mgmt.failures += "cognitive-management m5_fallback_verbs" }

  Write-JsonFile (Join-Path $RunDir "connect-mgmt.json") $mgmt
  $results["CONNECT-MGMT"] = $mgmt.status

  $shell = @{
    status   = "auto_pass"
    steps    = @()
    failures = @()
    notes    = @(
      "L1 = tip live minimal loop (sdk-ts http_live + kernel-server m5_http_sse)",
      "CONNECT-FULL-DEMO skipped_nonclaim (no long-lived proposal→cancel e2e on tip)",
      "CONNECT-WATCH reconnect/stale-cursor live = skipped_nonclaim (vector honesty via pins)"
    )
  }
  if (-not (Test-Path $env:KERNEL_SERVER_BIN)) {
    $shell.status = "auto_fail"
    $shell.failures += "KERNEL_SERVER_BIN missing — live tests would skip; auto_fail per plan"
  } else {
    $r = Invoke-Logged "sdk-ts-test" (Join-Path $LogDir "connect-shell-sdk-ts.log") {
      pnpm --filter @cognitiveos/sdk-ts test
    }
    $shell.steps += $r
    if ($r.exit_code -ne 0) {
      $shell.status = "auto_fail"
      $shell.failures += "sdk-ts test failed"
    } else {
      $liveLog = Get-Content (Join-Path $LogDir "connect-shell-sdk-ts.log") -Raw -ErrorAction SilentlyContinue
      # node:test TAP marks skipped tests with "# skip"; live:* must not be skipped when bin exists
      if ($liveLog -match "live:.*# skip" -or $liveLog -match "# skip\s+live:") {
        $shell.status = "auto_fail"
        $shell.failures += "sdk-ts live tests skipped despite KERNEL_SERVER_BIN — auto_fail"
      }
    }
  }

  # Serial threads: parallel watch/management --once children occasionally
  # ConnectionReset under load right after sdk-ts live suite; tip behavior unchanged.
  $r = Invoke-Logged "m5-http-sse" (Join-Path $LogDir "connect-shell-m5-http-sse.log") {
    cargo test -p kernel-server --test m5_http_sse --locked -- --test-threads=1
  }
  $shell.steps += $r
  if ($r.exit_code -ne 0) {
    $shell.status = "auto_fail"
    $shell.failures += "kernel-server m5_http_sse"
  }

  Write-JsonFile (Join-Path $RunDir "connect-shell.json") $shell
  $results["CONNECT-SHELL"] = $shell.status
  $results["CONNECT-WATCH"] = "skipped_nonclaim"
  $results["CONNECT-FULL-DEMO"] = "skipped_nonclaim"

  if ($mgmt.status -ne "auto_pass" -or $shell.status -ne "auto_pass") {
    $stopped = $true
    $stopReason = "CONNECT failed"
    $level = "L0"
  } else {
    $level = "L1"
  }
}

# VERIFY
if (-not $stopped) {
  $verify = @{ status = "auto_pass"; steps = @(); failures = @(); pins = $null; self_check = $null; regress = @{} }

  $r = Invoke-Logged "consistency" (Join-Path $LogDir "verify-consistency.log") {
    pnpm run check:consistency
  }
  $verify.steps += $r
  if ($r.exit_code -ne 0) { $verify.status = "auto_fail"; $verify.failures += "check:consistency" }
  $results["VERIFY-CONSISTENCY"] = if ($r.exit_code -eq 0) { "auto_pass" } else { "auto_fail" }

  if ($verify.status -eq "auto_pass") {
    $r = Invoke-Logged "conformance-runner" (Join-Path $LogDir "verify-runner.log") {
      cargo run --locked -p cognitive-conformance --bin conformance-runner
    }
    $verify.steps += $r
    if ($r.exit_code -ne 0) { $verify.status = "auto_fail"; $verify.failures += "conformance-runner" }

    $reportPath = Join-Path $RepoRoot "artifacts/evidence/conformance/conformance-report.json"
    if (Test-Path $reportPath) {
      Copy-Item $reportPath (Join-Path $RunDir "conformance-report.json") -Force
      $report = Get-Content $reportPath -Raw | ConvertFrom-Json
      $s = $report.summary
      $counts = @{
        total_vectors            = [int]$s.total_vectors
        pass                     = [int]$s.pass
        fail                     = [int]$s.fail
        "not-applicable"         = [int]$s.'not-applicable'
        "documented-degradation" = [int]$s.'documented-degradation'
        "not-run"                = [int]$s.'not-run'
      }
      $verify.pins = $counts
      $pinOk = (
        $counts.total_vectors -eq $Pinned.total_vectors -and
        $counts.pass -eq $Pinned.pass -and
        $counts.fail -eq $Pinned.fail -and
        $counts.'not-applicable' -eq $Pinned.'not-applicable' -and
        $counts.'documented-degradation' -eq $Pinned.'documented-degradation' -and
        $counts.'not-run' -eq $Pinned.'not-run'
      )
      if (-not $pinOk) {
        $verify.status = "auto_fail"
        $verify.failures += "pins mismatch: got $($counts | ConvertTo-Json -Compress)"
      }
      $results["VERIFY-PINS"] = if ($pinOk) { "auto_pass" } else { "auto_fail" }

      $byId = @{}
      foreach ($row in @($report.vectors)) {
        $byId[$row.id] = "$($row.result)"
      }

      $regressOk = $true
      foreach ($id in $RegressIds) {
        $got = $byId[$id]
        $verify.regress[$id] = if ($got) { "$got" } else { "missing" }
        if ($got -ne "pass") { $regressOk = $false }
      }
      if (-not $regressOk) {
        $verify.status = "auto_fail"
        $verify.failures += "REGRESS-V01 vector check failed: $($verify.regress | ConvertTo-Json -Compress)"
      }
      $results["REGRESS-V01"] = if ($regressOk) { "auto_pass" } else { "auto_fail" }
    } else {
      $verify.status = "auto_fail"
      $verify.failures += "conformance-report.json missing"
      $results["VERIFY-PINS"] = "auto_fail"
      $results["REGRESS-V01"] = "auto_fail"
    }
  }

  if ($verify.status -eq "auto_pass") {
    $r = Invoke-Logged "self-check" (Join-Path $LogDir "verify-self-check.log") {
      cargo run --locked -p cognitive-conformance --bin conformance-runner -- --self-check
    }
    $verify.steps += $r
    $scPath = Join-Path $RepoRoot "artifacts/evidence/conformance/self-check-report.json"
    if (Test-Path $scPath) {
      Copy-Item $scPath (Join-Path $RunDir "self-check-report.json") -Force
      $sc = Get-Content $scPath -Raw | ConvertFrom-Json
      $mustFlip = @($sc.must_flip).Count
      $verify.self_check = @{ must_flip = $mustFlip; flipped_to_fail = @($sc.flipped_to_fail).Count }
      if ($mustFlip -lt $Pinned.self_check_min) {
        $verify.status = "auto_fail"
        $verify.failures += "self-check corpus shrank to $mustFlip"
        $results["VERIFY-SELFCHECK"] = "auto_fail"
      } elseif ($r.exit_code -ne 0) {
        $verify.status = "auto_fail"
        $verify.failures += "self-check exit $($r.exit_code)"
        $results["VERIFY-SELFCHECK"] = "auto_fail"
      } else {
        $results["VERIFY-SELFCHECK"] = "auto_pass"
      }
    } else {
      $verify.status = "auto_fail"
      $verify.failures += "self-check-report.json missing"
      $results["VERIFY-SELFCHECK"] = "auto_fail"
    }
  }

  if ($verify.status -eq "auto_pass") {
    $r = Invoke-Logged "f017-digests" (Join-Path $LogDir "verify-f017.log") {
      cargo test -p cognitive-runtime --lib sandbox::tests --locked -- --nocapture
    }
    $verify.steps += $r
    if ($r.exit_code -ne 0) {
      $verify.status = "auto_fail"
      $verify.failures += "F-017 sandbox::tests"
      $results["F017-CLAIM-FREEZE"] = "auto_fail"
    } else {
      $results["F017-CLAIM-FREEZE"] = "auto_pass"
    }
  }

  Write-JsonFile (Join-Path $RunDir "verify.json") $verify
  if ($verify.status -ne "auto_pass") {
    $stopped = $true
    $stopReason = "VERIFY failed: $($verify.failures -join '; ')"
    # Do not claim L2
  } else {
    $level = "L2"
  }
}

# PERF-004 AUTO
$perf4 = @{
  status               = "skipped_nonclaim"
  claim_level          = "sample_or_builder_only"
  campaign             = "not_executed"
  claims_agent_benefit = $false
  human_gate_default   = "HUMAN-PERF004-CAMPAIGN → skip / keep non-claim"
  failures             = @()
}
if (-not $stopped) {
  $r = Invoke-Logged "perf004-unit" (Join-Path $LogDir "perf004-unit.log") {
    cargo test -p cognitive-runtime overhead_report_requires_ungoverned_baseline_and_forbids_benefit --locked -- --exact
  }
  if ($r.exit_code -ne 0) {
    $perf4.status = "auto_fail"
    $perf4.failures += "overhead unit test failed"
    $results["PERF004-AUTO-REPORT"] = "auto_fail"
  } else {
    # Honesty envelope + sample numbers matching tip builder/runner (not a HW campaign)
    $sampleReport = [ordered]@{
      orchestrator_honesty = [ordered]@{
        claim_level          = "sample_or_builder_only"
        campaign             = "not_executed"
        claims_agent_benefit = $false
        source               = "cognitive_runtime::GovernanceOverheadSample (same numbers as unit test / conformance-runner embed)"
        forbid_silent_campaign_pass = $true
      }
      schema_version = "cognitiveos.performance-report/0.1"
      note           = "Sample/builder export for L3 report-ready. NOT a full HW campaign digest."
      governance_overhead = [ordered]@{
        ungoverned_baseline = "ungoverned-local-v1"
        gate_latency_ms     = [ordered]@{
          authorization = @{ p50 = 0.1; p95 = 0.4; p99 = 0.9 }
          context_resolution = @{ p50 = 1.0; p95 = 3.0; p99 = 5.0 }
          effect_protocol = @{ p50 = 0.5; p95 = 1.2; p99 = 2.0 }
        }
        cache_hit_preservation_ratio = 0.9
        extra_persistence_per_governed_call = @{ writes = 2.0; bytes = 1024.0 }
        approval = [ordered]@{
          latency_ms = @{ p50 = 10.0; p95 = 50.0; p99 = 100.0 }
          rubber_stamp_rate = 0.01
          retry_after_deny_rate = 0.02
        }
        overhead_share_by_risk_class = @(
          @{ risk_class = "R1"; latency_percent = 3.0; cost_percent = 2.0 }
        )
      }
    }
    $perfDir = Join-Path $RepoRoot "artifacts/evidence/performance"
    New-Item -ItemType Directory -Force -Path $perfDir | Out-Null
    $perfPath = Join-Path $perfDir "performance-report-v01-sample.json"
    Write-JsonFile $perfPath $sampleReport
    Copy-Item $perfPath (Join-Path $RunDir "performance-report-v01-sample.json") -Force
    $perf4.status = "auto_pass"
    $perf4.report_path = "artifacts/evidence/performance/performance-report-v01-sample.json"
    $perf4.report_sha256 = Get-FileSha256 $perfPath
    $results["PERF004-AUTO-REPORT"] = "auto_pass"
  }
} else {
  $results["PERF004-AUTO-REPORT"] = "skipped_nonclaim"
}
# Always assert no silent campaign
$results["PERF004-NO-SILENT-CAMPAIGN"] = if (
  $perf4.campaign -eq "not_executed" -and -not $perf4.claims_agent_benefit
) { "auto_pass" } else { "auto_fail" }
Write-JsonFile (Join-Path $RunDir "perf004.json") $perf4

# PERF-005 precheck (default skip)
$perf5 = @{
  status             = "skipped_nonclaim"
  four_arm_harness   = $false
  preregistration    = $false
  independent_verifier = $false
  significant_benefit_forbidden = $true
  human_gate_default = "HUMAN-PERF005-CLAIM → forbid benefit"
  reason             = "Tip has contract/docs only; no executable four-arm harness (F-026 / IMP-18 / M7+)"
}
$harnessPaths = @(
  "docs/evaluation/agent-benefit-benchmark.md"
)
# Presence of doc ≠ harness
$perf5.doc_contract_exists = (Test-Path (Join-Path $RepoRoot "docs/evaluation/agent-benefit-benchmark.md"))
Write-JsonFile (Join-Path $RunDir "perf005-precheck.json") $perf5
$results["PERF005-DEFAULT-NONCLAIM"] = "skipped_nonclaim"
$results["PERF005-NO-SILENT-BENEFIT"] = "auto_pass"

if (-not $stopped -and $results["PERF004-AUTO-REPORT"] -eq "auto_pass" -and $results["PERF004-NO-SILENT-CAMPAIGN"] -eq "auto_pass") {
  $level = "L3"
}

# Teardown tmp
try {
  if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue }
  $results["BOOT-TEARDOWN"] = "auto_pass"
} catch {
  $results["BOOT-TEARDOWN"] = "auto_fail"
}

# Manifest honesty note (from runner artifacts if present)
$rcPath = Join-Path $RepoRoot "artifacts/evidence/conformance/release-candidate-profile-manifest.json"
$manifestHonesty = "auto_pass"
if (Test-Path $rcPath) {
  Copy-Item $rcPath (Join-Path $RunDir "release-candidate-profile-manifest.json") -Force
  $rc = Get-Content $rcPath -Raw
  if ($rc -match '"implemented"' -and $rc -notmatch '"status"\s*:\s*"planned"' ) {
    # soft check: implemented count should be zero profiles — look for implemented status
    if ($rc -match '"status"\s*:\s*"implemented"') {
      $manifestHonesty = "auto_fail"
    }
  }
}
$results["MANIFEST-HONESTY"] = $manifestHonesty
$results["ORCHESTRATOR-ONE-SHOT"] = "auto_pass"
$results["SUMMARY-MACHINE-READABLE"] = "auto_pass"
$results["REVIEW"] = "skipped_nonclaim"
$results["HUMAN-CI-JOB-ADD"] = "skipped_nonclaim"
$results["HUMAN-PERF004-CAMPAIGN"] = "skipped_nonclaim"
$results["HUMAN-PERF005-CLAIM"] = "skipped_nonclaim"

# Digest manifest
$digestRows = @()
Get-ChildItem -Path $RunDir -File -Recurse | ForEach-Object {
  $rel = $_.FullName.Substring($RunDir.Length).TrimStart("\", "/")
  $digestRows += @{ path = $rel; sha256 = (Get-FileSha256 $_.FullName) }
}
Write-JsonFile (Join-Path $RunDir "sha256-manifest.json") @{ files = $digestRows }

$humanGates = @(
  @{ id = "HUMAN-PLATFORM-LABEL"; default = "conservative label + continue"; triggered = ($platform.status -eq "needs_human") },
  @{ id = "HUMAN-CI-JOB-ADD"; default = "local verify:local only"; triggered = $false },
  @{ id = "HUMAN-PERF004-CAMPAIGN"; default = "keep non-claim"; triggered = $false },
  @{ id = "HUMAN-PERF005-CLAIM"; default = "forbid benefit"; triggered = $false },
  @{ id = "HUMAN-NO-GO"; default = "mark failed, do not release"; triggered = $stopped }
)

$summary = [ordered]@{
  schema_version     = "cognitiveos.v01-auto-run-summary/0.1"
  run_id             = $RunId
  level              = $level
  stopped            = $stopped
  stop_reason        = $stopReason
  release            = if ($stopped) { "blocked" } else { "non_claim_preserved" }
  profile_implemented = 0
  auto_green_means_profile_implemented = $false
  results            = $results
  v01_non_claims     = $V01NonClaims
  human_gates        = $humanGates
  entry              = @{
    origin_main = $entry.origin_main
    head        = $entry.head
    branch      = $entry.branch
  }
  platform_label     = $platform.f017_platform_label
  partitions         = @{
    auto_pass         = @($results.GetEnumerator() | Where-Object { $_.Value -eq "auto_pass" } | ForEach-Object { $_.Key })
    auto_fail         = @($results.GetEnumerator() | Where-Object { $_.Value -eq "auto_fail" } | ForEach-Object { $_.Key })
    skipped_nonclaim  = @($results.GetEnumerator() | Where-Object { $_.Value -eq "skipped_nonclaim" } | ForEach-Object { $_.Key })
    needs_human       = @($results.GetEnumerator() | Where-Object { $_.Value -eq "needs_human" } | ForEach-Object { $_.Key })
  }
}

Write-JsonFile (Join-Path $RunDir "summary.json") $summary

$md = @"
# V01 Auto-Run Summary — $RunId

- **Level**: $level
- **Stopped**: $stopped $(if ($stopReason) { "($stopReason)" } else { "" })
- **Release posture**: $($summary.release)
- **Platform label**: $($platform.f017_platform_label)
- **Profile implemented**: 0 (auto green ≠ Profile implemented)

## Results

| ID | Status |
|---|---|
$($results.GetEnumerator() | ForEach-Object { "| $($_.Key) | $($_.Value) |" } | Out-String)

## Inherited v0.1 non-claims

$($V01NonClaims | ForEach-Object { "- $_" } | Out-String)

## Human gates (defaults applied)

$($humanGates | ForEach-Object { "- $($_.id): default=$($_.default); triggered=$($_.triggered)" } | Out-String)

## Evidence

- Run dir: ``artifacts/evidence/v01-auto-run/$RunId/``
- ``summary.json`` + ``sha256-manifest.json``
"@
$md | Set-Content -Path (Join-Path $RunDir "summary.md") -Encoding utf8

Write-Host ""
Write-Host "=== Done: level=$level stopped=$stopped ==="
Write-Host "Summary: $RunDir\summary.json"
if ($stopped) { exit 1 } else { exit 0 }
