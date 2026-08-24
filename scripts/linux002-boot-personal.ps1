<#
.SYNOPSIS
  One-shot operator entry point: sync + boot CognitiveOS Personal on linux-002
  (guest B01-Desktop-Linux-002) and bring up the installed dsh and Pi agents.

.DESCRIPTION
  Owner-ops development convenience tooling. Run this ONE script from local
  Windows PowerShell (5.1 compatible); it performs every jump-host and guest
  step itself over SSH. No second device-specific playbook is needed.

  What one invocation does, in order:

    1. Verifies the jump host is reachable (BatchMode SSH, no prompts).
    2. Optionally (-EnsureGuestRunning) checks the libvirt domain state on the
       jump host with `virsh -c qemu:///system` and starts the guest domain if
       it is shut off, then waits for guest SSH.
    3. Verifies guest SSH over the verified route:
         local -> wuz@192.168.1.2 (hal9000) -> ProxyJump -> hal9001@192.168.123.160
    4. Resolves the requested pushed Git revision (default: origin/main tip)
       in a script-managed clone on the jump host, builds `cognitive` +
       `kernel-server` (release, --locked, Rust pinned by rust-toolchain.toml),
       and installs them on the guest under a script-managed, cleanable root
       with a REVISION marker and a `current` symlink. Only pushed/exact
       revisions are ever used; uncommitted local trees are never copied.
       SyncMode auto skips the build when the guest already has the exact
       target revision installed by this script.
    5. Ensures the Personal daemon is healthy on the owner-ops bind
       (default 127.0.0.1:48681, runtime root /home/hal9001/p8t13-owner-ops/runtime).
       A healthy already-running daemon is reused, never fought; a stale
       daemon.lock (dead pid) is removed before a fresh `cognitive daemon start`.
    6. Ensures dsh is configured against the installed Cos dsh tree
       (Path B: dsh -> AKP -> daemon; Path A direct-Flash is measurement-only
       and is never started by this script) and, in the default `web` mode,
       starts the native dsh web panel on loopback (default 127.0.0.1:3080)
       if it is not already serving. Existing healthy panels are reused.
    7. Ensures Pi is configured (existing valid pi.json is reused; otherwise
       known installed pin anchors under the guest home are discovered and
       `cognitive pi configure` is run). Pi launch is interactive by design:
       by default the script leaves Pi ready and prints the exact launch
       command; -LaunchPi opens the interactive session in this terminal.
    8. Prints a redacted local status summary: daemon health, `cognitive
       status` / `cognitive doctor` readiness flags, `cognitive dsh status`,
       and Pi readiness facts.

.PARAMETER JumpHost
  SSH destination of the jump host. Default: wuz@192.168.1.2

.PARAMETER GuestHost
  SSH destination of the guest, reached via ProxyJump through the jump host.
  Default: hal9001@192.168.123.160

.PARAMETER GuestDomain
  libvirt domain name of the guest on the jump host (system URI). Default:
  B01-Desktop-Linux-002. The retired B01-Clean-Linux-001 is hard-refused.

.PARAMETER EnsureGuestRunning
  Check/start the guest domain via jump-host `virsh -c qemu:///system` before
  connecting. Without this switch a down guest is a fail-closed error.

.PARAMETER Revision
  Pushed Git revision to sync: `origin/<branch>` or an exact SHA already
  present on origin. Default: origin/main.

.PARAMETER SyncMode
  auto (default): build+install only when the guest's script-managed install
  does not already match the resolved target revision. always: rebuild and
  reinstall. never: reuse existing guest binaries only (no jump-host build).

.PARAMETER RepoUrl
  Git remote the jump host clones/fetches from.
  Default: https://github.com/agentkernel/cognitive-os.git

.PARAMETER JumpWorkDir
  Script-managed, cleanable working directory on the jump host.
  Default: /home/wuz/linux002-boot (clone lives at <dir>/cognitive-os).

.PARAMETER GuestInstallRoot
  Script-managed, cleanable install root on the guest.
  Default: /home/hal9001/linux002-boot (per-revision bin dirs + `current`).

.PARAMETER RuntimeRoot
  Personal runtime root on the guest (owner-ops root from P8-T13/T15).
  Default: /home/hal9001/p8t13-owner-ops/runtime

.PARAMETER Bind
  Loopback daemon bind. Default: 127.0.0.1:48681 (owner-ops daemon on this
  guest; the product default elsewhere is 127.0.0.1:48181).

.PARAMETER CognitiveBin
  Explicit absolute path of the `cognitive` binary on the guest. Overrides
  discovery (script-managed `current` first, then newest /home/hal9001/p8t*-*/bin).

.PARAMETER KernelServerBin
  Explicit absolute path of `kernel-server` on the guest. Default: sibling of
  the selected `cognitive`.

.PARAMETER RestartDaemon
  Stop a healthy running daemon and restart it on the freshly selected
  kernel-server binary. Default: a healthy daemon is reused as-is.

.PARAMETER DshRoot
  Installed Cos dsh tree on the guest (Path B). Default:
  /home/hal9001/p8t10-a17edfad/dsh

.PARAMETER AdapterRoot
  Installed AKP adapter root on the guest. Default:
  /home/hal9001/p8t15-0376e942/adapter

.PARAMETER DshRevision
  dsh pin; must equal the CLI's compiled exact pin.
  Default: 528c682e061696f5a160f363f236ecbf53cbd006

.PARAMETER DshMode
  web (default): ensure the loopback dsh web panel is serving.
  headless: configure + status only, no panel.

.PARAMETER DshWebPort
  Loopback port for the dsh web panel. Default: 3080.

.PARAMETER ReconfigureDsh
  Force `cognitive dsh configure` even when dsh.json already matches.

.PARAMETER SkipDsh
  Skip the dsh stage entirely.

.PARAMETER SkipPi
  Skip the Pi stage entirely.

.PARAMETER PiExecutable
  Explicit Pi executable path on the guest (absolute). Overrides pi.json reuse
  and discovery.

.PARAMETER PiExtensionEntry
  Explicit CognitiveOS Extension entry path on the guest (absolute).

.PARAMETER LaunchPi
  After everything is up, open the interactive Pi session in this terminal
  (`ssh -t ... cognitive pi launch`). Default: leave ready and print the command.

.PARAMETER DshSmokeTask
  Optional one-shot non-interactive dsh Path B prompt (`dsh launch --print
  --path b --task <prompt>`). Consumes a real Provider call; off by default.

.PARAMETER GuestBootTimeoutSec
  How long to wait for guest SSH after a virsh start. Default: 240.

.PARAMETER ConnectTimeoutSec
  SSH ConnectTimeout for every hop. Default: 10.

.PARAMETER DryRun
  Read-only probes plus a printed action plan. No jump-host or guest mutation.

.EXAMPLE
  # CANONICAL PASTE - works from ANY PowerShell prompt (C:\Windows\system32
  # included). Step 1: cd into your local cognitive-os clone. Step 2: run the
  # script via a path that exists under that clone. Optional switches such as
  # -EnsureGuestRunning and -LaunchPi are real parameters and belong on the
  # SAME command line, never on separate '#' comment lines.
  cd C:\path\to\your\cognitive-os
  powershell -ExecutionPolicy Bypass -File .\scripts\linux002-boot-personal.ps1 -EnsureGuestRunning -LaunchPi

.EXAMPLE
  # No cd needed: give -File the ABSOLUTE path to the script instead
  powershell -ExecutionPolicy Bypass -File C:\path\to\your\cognitive-os\scripts\linux002-boot-personal.ps1 -EnsureGuestRunning

.EXAMPLE
  # From the clone root a same-name launcher forwards every parameter here
  .\linux002-boot-personal.ps1 -EnsureGuestRunning -LaunchPi

.EXAMPLE
  # Default full boot (guest already running): sync origin/main, ensure
  # daemon + dsh web panel, leave Pi ready with a printed launch command
  .\scripts\linux002-boot-personal.ps1

.EXAMPLE
  # Reuse whatever is installed (no build), restart daemon on it, headless dsh
  .\scripts\linux002-boot-personal.ps1 -SyncMode never -RestartDaemon -DshMode headless

.EXAMPLE
  # Pin an exact pushed revision and hand me an interactive Pi at the end
  .\scripts\linux002-boot-personal.ps1 -Revision 562d2a5d0000000000000000000000000000dead -LaunchPi

.NOTES
  TROUBLESHOOTING - "The argument '...' to the -File parameter does not exist":
    PowerShell resolved a RELATIVE path against your CURRENT directory (for
    example C:\Windows\system32) and found nothing. One of two things is true:
    (a) You are not inside a cognitive-os clone. Fix: `cd` into your clone
        first, or pass the absolute path to -File (see the examples above).
    (b) Your checkout does not contain this script yet. Until PR #268
        (https://github.com/agentkernel/cognitive-os/pull/268) is merged, the
        script ships only on its branch. Fetch it into your clone:
          git fetch origin cursor/linux002-boot-personal-ops-script-ac2d
          git checkout cursor/linux002-boot-personal-ops-script-ac2d
        After PR #268 is merged to main, use instead:
          git checkout main
          git pull origin main
    Also: lines starting with '#' are comments, not parameters. Put switches
    such as -EnsureGuestRunning / -LaunchPi on the same command line as -File.

  Exit codes:
    0  success
    2  parameter / local prerequisite error (ssh missing, unsafe path chars)
    3  jump host unreachable
    4  guest domain down / did not boot in time
    5  guest SSH unreachable
    6  revision resolve or jump-host build failure
    7  binary transfer / guest install failure
    8  Personal daemon did not become healthy
    9  dsh configure / web panel failure
   10  Pi not configured / not ready

  Secret boundary (hard rules):
    - This script NEVER reads, prints, or transfers Provider API keys,
      bootstrap tokens, bearer material, or SecretStore content.
    - Secrets stay only in the approved guest Secret Store / keyring; nothing
      here writes keys into argv, env files, git, logs, or chat transcripts.
    - Remote status output comes only from the product's redacted surfaces
      (`cognitive status|doctor|dsh status`, /personal/health) and daemon.log
      tails, which the product keeps secret-free by contract.

  Non-claims (governance):
    - Owner-ops development convenience only. Running this script creates NO
      Gate, release, Profile, B01, EVAL, or Agent-benefit claim, and does not
      change any campaign or attempt ledger.
    - linux-002 state changed by this script is ordinary owner-directed ops on
      the owner's instruction; it is not a preregistered B01 campaign action.
    - The retired guest B01-Clean-Linux-001 is never touched (hard refusal).

  Prerequisites on the operator machine: Windows OpenSSH client (ssh.exe) with
  working key auth + known_hosts for both hops (run one manual
  `ssh -J wuz@192.168.1.2 hal9001@192.168.123.160 true` first if needed).
  Jump host needs git + rustup cargo (Rust pin auto-selected by
  rust-toolchain.toml). BatchMode is used everywhere: the script fails closed
  instead of prompting.
#>
[CmdletBinding()]
param(
  [string]$JumpHost = 'wuz@192.168.1.2',
  [string]$GuestHost = 'hal9001@192.168.123.160',
  [string]$GuestDomain = 'B01-Desktop-Linux-002',
  [switch]$EnsureGuestRunning,
  [string]$Revision = 'origin/main',
  [ValidateSet('auto', 'always', 'never')]
  [string]$SyncMode = 'auto',
  [string]$RepoUrl = 'https://github.com/agentkernel/cognitive-os.git',
  [string]$JumpWorkDir = '/home/wuz/linux002-boot',
  [string]$GuestInstallRoot = '/home/hal9001/linux002-boot',
  [string]$RuntimeRoot = '/home/hal9001/p8t13-owner-ops/runtime',
  [string]$Bind = '127.0.0.1:48681',
  [string]$CognitiveBin = '',
  [string]$KernelServerBin = '',
  [switch]$RestartDaemon,
  [string]$DshRoot = '/home/hal9001/p8t10-a17edfad/dsh',
  [string]$AdapterRoot = '/home/hal9001/p8t15-0376e942/adapter',
  [string]$DshRevision = '528c682e061696f5a160f363f236ecbf53cbd006',
  [ValidateSet('web', 'headless')]
  [string]$DshMode = 'web',
  [int]$DshWebPort = 3080,
  [switch]$ReconfigureDsh,
  [switch]$SkipDsh,
  [switch]$SkipPi,
  [string]$PiExecutable = '',
  [string]$PiExtensionEntry = '',
  [switch]$LaunchPi,
  [string]$DshSmokeTask = '',
  [int]$GuestBootTimeoutSec = 240,
  [int]$ConnectTimeoutSec = 10,
  [switch]$DryRun
)

Set-StrictMode -Version 2.0
$ErrorActionPreference = 'Stop'

# Fail closed, with a human-readable pointer, when this code was not started
# as an on-disk .ps1 file (for example pasted or piped into powershell).
# Relative-path guidance and script-root anchoring are meaningless in that
# mode, so refuse early instead of failing later with a confusing error.
if ([string]::IsNullOrEmpty($PSCommandPath)) {
  Write-Host ''
  Write-Host 'ERROR (2): linux002-boot-personal.ps1 was not started as a script file.' -ForegroundColor Red
  Write-Host '  Run the on-disk .ps1 from your cognitive-os clone:'
  Write-Host '    cd C:\path\to\your\cognitive-os'
  Write-Host '    powershell -ExecutionPolicy Bypass -File .\scripts\linux002-boot-personal.ps1 -EnsureGuestRunning -LaunchPi'
  Write-Host '  If -File reports the path does not exist, see TROUBLESHOOTING in the'
  Write-Host '  help header of scripts/linux002-boot-personal.ps1: either your current'
  Write-Host '  directory is not a clone containing the script, or your checkout'
  Write-Host '  predates PR #268 (branch cursor/linux002-boot-personal-ops-script-ac2d).'
  exit 2
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Write-Stage { param([string]$Message) Write-Host "==> $Message" -ForegroundColor Cyan }
function Write-Info { param([string]$Message) Write-Host "    $Message" }
function Write-Warn2 { param([string]$Message) Write-Host "    WARNING: $Message" -ForegroundColor Yellow }

function Stop-Boot {
  param([int]$Code, [string]$Message)
  Write-Host ""
  Write-Host "ERROR ($Code): $Message" -ForegroundColor Red
  exit $Code
}

# Single-quote a value for a remote POSIX shell.
function ConvertTo-ShellArg {
  param([string]$Value)
  return "'" + ($Value -replace "'", "'\''") + "'"
}

# Remote path/identifier safety: values travel through PowerShell native
# argument quoting, ssh, and bash. Refuse anything outside a conservative set
# so no quoting layer can be broken. Fail closed.
function Test-SafeRemoteValue {
  param([string]$Value, [string]$Name, [switch]$AllowEmpty)
  if ([string]::IsNullOrEmpty($Value)) {
    if ($AllowEmpty) { return }
    Stop-Boot 2 "parameter $Name must not be empty"
  }
  if ($Value -notmatch '^[A-Za-z0-9._/:@+-]+$') {
    Stop-Boot 2 "parameter $Name contains characters unsafe for remote quoting: $Value"
  }
}

$script:RemoteExitCode = 0

# Run a literal bash payload on a remote host by piping it to `bash -s`.
# Payload stdout is captured and returned; payload/ssh stderr flows to this
# console so the operator sees live diagnostics. CR characters are stripped so
# Windows line endings never reach bash.
function Invoke-RemoteBash {
  param(
    [string[]]$SshArgs,
    [string]$Payload,
    [string[]]$PayloadArgs = @()
  )
  $remoteCommand = 'bash -s --'
  foreach ($argument in $PayloadArgs) {
    $remoteCommand += ' ' + (ConvertTo-ShellArg $argument)
  }
  $cleanPayload = $Payload -replace "`r", ''
  $output = @($cleanPayload | & ssh @SshArgs $remoteCommand)
  $script:RemoteExitCode = $LASTEXITCODE
  return $output
}

function Invoke-JumpBash {
  param([string]$Payload, [string[]]$PayloadArgs = @())
  $sshArgs = @('-o', 'BatchMode=yes', '-o', "ConnectTimeout=$ConnectTimeoutSec", $JumpHost)
  return Invoke-RemoteBash -SshArgs $sshArgs -Payload $Payload -PayloadArgs $PayloadArgs
}

function Invoke-GuestBash {
  param([string]$Payload, [string[]]$PayloadArgs = @())
  $sshArgs = @('-o', 'BatchMode=yes', '-o', "ConnectTimeout=$ConnectTimeoutSec", '-J', $JumpHost, $GuestHost)
  return Invoke-RemoteBash -SshArgs $sshArgs -Payload $Payload -PayloadArgs $PayloadArgs
}

# Extract "NAME=value" tokens emitted by remote payloads on stdout.
function Get-Token {
  param([string[]]$Lines, [string]$Name)
  foreach ($line in @($Lines)) {
    $text = "$line"
    if ($text.StartsWith("$Name=")) { return $text.Substring($Name.Length + 1) }
  }
  return $null
}

# ---------------------------------------------------------------------------
# Parameter validation (fail closed before any network use)
# ---------------------------------------------------------------------------

if ($GuestDomain -eq 'B01-Clean-Linux-001') {
  Stop-Boot 2 'B01-Clean-Linux-001 is retired and isolated; this script refuses to touch it'
}
if (-not (Get-Command ssh -ErrorAction SilentlyContinue)) {
  Stop-Boot 2 'ssh.exe not found; install the Windows OpenSSH client feature first'
}

Test-SafeRemoteValue $JumpHost 'JumpHost'
Test-SafeRemoteValue $GuestHost 'GuestHost'
Test-SafeRemoteValue $GuestDomain 'GuestDomain'
Test-SafeRemoteValue $Revision 'Revision'
Test-SafeRemoteValue $RepoUrl 'RepoUrl'
Test-SafeRemoteValue $JumpWorkDir 'JumpWorkDir'
Test-SafeRemoteValue $GuestInstallRoot 'GuestInstallRoot'
Test-SafeRemoteValue $RuntimeRoot 'RuntimeRoot'
Test-SafeRemoteValue $Bind 'Bind'
Test-SafeRemoteValue $CognitiveBin 'CognitiveBin' -AllowEmpty
Test-SafeRemoteValue $KernelServerBin 'KernelServerBin' -AllowEmpty
Test-SafeRemoteValue $DshRoot 'DshRoot'
Test-SafeRemoteValue $AdapterRoot 'AdapterRoot'
Test-SafeRemoteValue $DshRevision 'DshRevision'
Test-SafeRemoteValue $PiExecutable 'PiExecutable' -AllowEmpty
Test-SafeRemoteValue $PiExtensionEntry 'PiExtensionEntry' -AllowEmpty

if ($Bind -notmatch '^127\.[0-9.]+:[0-9]+$') {
  Stop-Boot 2 "Bind must be a loopback host:port (got: $Bind)"
}
if ($DshSmokeTask -match '["`\\]') {
  Stop-Boot 2 'DshSmokeTask must not contain double quotes, backticks, or backslashes'
}

$summary = New-Object System.Collections.Specialized.OrderedDictionary
$summary['guest'] = "$GuestHost (domain $GuestDomain via $JumpHost)"

Write-Host ''
Write-Host 'linux002-boot-personal: CognitiveOS Personal owner-ops boot' -ForegroundColor Green
Write-Info "script         $PSCommandPath"
Write-Info "route          local -> $JumpHost -> $GuestHost"
Write-Info "revision       $Revision (SyncMode=$SyncMode)"
Write-Info "runtime root   $RuntimeRoot"
Write-Info "daemon bind    $Bind"
Write-Info "dsh            mode=$DshMode root=$DshRoot"
Write-Info "pi             skip=$($SkipPi.IsPresent) launch=$($LaunchPi.IsPresent)"
if ($DryRun) { Write-Warn2 'DRY-RUN: read-only probes only; no remote mutation' }
Write-Host ''

# ---------------------------------------------------------------------------
# Stage 1: jump host reachability
# ---------------------------------------------------------------------------

Write-Stage "[1/7] probing jump host $JumpHost"
$jumpProbe = Invoke-JumpBash -Payload 'echo JUMP_OK'
if ($script:RemoteExitCode -ne 0 -or (($jumpProbe -join ' ') -notmatch 'JUMP_OK')) {
  Stop-Boot 3 "jump host $JumpHost is unreachable over BatchMode SSH; check VPN/LAN, key auth, and known_hosts (run: ssh $JumpHost true)"
}
Write-Info 'jump host reachable'

# ---------------------------------------------------------------------------
# Stage 2: optional guest domain ensure (virsh on the jump host)
# ---------------------------------------------------------------------------

$virshPayload = @'
set -eu
domain="$1"
dry_run="$2"
if [ "$domain" = "B01-Clean-Linux-001" ]; then
  echo "DOMAIN=refused"
  exit 97
fi
state=$(virsh -c qemu:///system domstate "$domain" 2>/dev/null | sed -n '1p' || true)
echo "DOMSTATE=$state"
if [ "$state" != "running" ]; then
  if [ "$dry_run" = "yes" ]; then
    echo "DOMSTART=skipped-dry-run"
  else
    virsh -c qemu:///system start "$domain" 1>&2
    echo "DOMSTART=issued"
  fi
fi
'@

if ($EnsureGuestRunning) {
  Write-Stage "[2/7] ensuring libvirt domain $GuestDomain is running"
  $dryFlag = 'no'
  if ($DryRun) { $dryFlag = 'yes' }
  $virshOut = Invoke-JumpBash -Payload $virshPayload -PayloadArgs @($GuestDomain, $dryFlag)
  if ($script:RemoteExitCode -ne 0) {
    Stop-Boot 4 "virsh could not read/start domain $GuestDomain on the jump host (see output above)"
  }
  $domState = Get-Token $virshOut 'DOMSTATE'
  $domStart = Get-Token $virshOut 'DOMSTART'
  Write-Info "domain state: $domState"
  if ($domStart -eq 'issued') {
    Write-Info "start issued; waiting up to $GuestBootTimeoutSec s for guest SSH"
    $deadline = [DateTime]::UtcNow.AddSeconds($GuestBootTimeoutSec)
    $guestUp = $false
    while ([DateTime]::UtcNow -lt $deadline) {
      Start-Sleep -Seconds 10
      $probe = Invoke-GuestBash -Payload 'echo GUEST_OK'
      if ($script:RemoteExitCode -eq 0 -and (($probe -join ' ') -match 'GUEST_OK')) { $guestUp = $true; break }
    }
    if (-not $guestUp) {
      Stop-Boot 4 "guest did not accept SSH within $GuestBootTimeoutSec s after virsh start"
    }
  }
} else {
  Write-Stage '[2/7] guest domain ensure skipped (-EnsureGuestRunning not set)'
}

# ---------------------------------------------------------------------------
# Stage 3: guest reachability
# ---------------------------------------------------------------------------

Write-Stage "[3/7] probing guest $GuestHost via ProxyJump"
$guestProbe = Invoke-GuestBash -Payload 'echo GUEST_OK; uname -sr'
if ($script:RemoteExitCode -ne 0 -or (($guestProbe -join ' ') -notmatch 'GUEST_OK')) {
  $hint = 'check the guest is running (rerun with -EnsureGuestRunning), key auth, and known_hosts'
  Stop-Boot 5 "guest $GuestHost is unreachable via -J $JumpHost; $hint"
}
Write-Info ("guest reachable: " + (@($guestProbe) | Select-Object -Last 1))

# ---------------------------------------------------------------------------
# Stage 4: resolve revision, build on the jump host, install on the guest
# ---------------------------------------------------------------------------

$resolvePayload = @'
set -eu
repo_url="$1"
work_dir="$2"
revision="$3"
dry_run="$4"
export PATH="$HOME/.cargo/bin:$PATH"
clone_dir="$work_dir/cognitive-os"
if [ ! -d "$clone_dir/.git" ]; then
  if [ "$dry_run" = "yes" ]; then
    ref="$revision"
    case "$ref" in
      origin/*) ref="refs/heads/${ref#origin/}" ;;
    esac
    sha=$(git ls-remote "$repo_url" "$ref" | awk '{print $1}' | sed -n '1p')
    if [ -n "$sha" ]; then
      echo "RESOLVED_SHA=$sha"
    else
      echo "RESOLVED_SHA=$revision"
    fi
    echo "CLONE=absent"
    exit 0
  fi
  git clone --no-checkout "$repo_url" "$clone_dir" 1>&2
fi
cd "$clone_dir"
if [ "$dry_run" != "yes" ]; then
  git fetch --prune origin 1>&2
fi
sha=$(git rev-parse --verify "$revision^{commit}")
echo "RESOLVED_SHA=$sha"
echo "CLONE=present"
'@

$buildPayload = @'
set -eu
work_dir="$1"
sha="$2"
export PATH="$HOME/.cargo/bin:$PATH"
cd "$work_dir/cognitive-os"
git checkout --force --detach "$sha" 1>&2
if ! command -v cargo >/dev/null 2>&1; then
  echo "BUILD=cargo-missing"
  exit 96
fi
cargo build --release --locked -p admin-cli -p kernel-server 1>&2
test -x target/release/cognitive
test -x target/release/kernel-server
echo "BUILD=ok"
echo "SHA_COGNITIVE=$(sha256sum target/release/cognitive | awk '{print $1}')"
echo "SHA_KERNEL=$(sha256sum target/release/kernel-server | awk '{print $1}')"
'@

$guestInstalledPayload = @'
set -eu
install_root="$1"
if [ -f "$install_root/current/REVISION" ]; then
  echo "INSTALLED_SHA=$(sed -n '1p' "$install_root/current/REVISION")"
else
  echo "INSTALLED_SHA=none"
fi
'@

$guestPreparePayload = @'
set -eu
dest_bin="$1"
mkdir -p "$dest_bin"
echo "PREPARE=ok"
'@

$jumpScpPayload = @'
set -eu
work_dir="$1"
guest="$2"
dest_bin="$3"
scp -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new -q \
  "$work_dir/cognitive-os/target/release/cognitive" \
  "$work_dir/cognitive-os/target/release/kernel-server" \
  "$guest:$dest_bin/"
echo "SCP=ok"
'@

$guestFinalizePayload = @'
set -eu
install_root="$1"
short="$2"
sha="$3"
expect_cognitive="$4"
expect_kernel="$5"
dest="$install_root/$short"
chmod 0755 "$dest/bin/cognitive" "$dest/bin/kernel-server"
got_cognitive=$(sha256sum "$dest/bin/cognitive" | awk '{print $1}')
got_kernel=$(sha256sum "$dest/bin/kernel-server" | awk '{print $1}')
if [ "$got_cognitive" != "$expect_cognitive" ] || [ "$got_kernel" != "$expect_kernel" ]; then
  echo "FINALIZE=digest-mismatch"
  exit 91
fi
printf '%s\n' "$sha" > "$dest/REVISION"
ln -sfn "$dest" "$install_root/current"
echo "FINALIZE=ok"
'@

$guestDiscoverPayload = @'
set -eu
install_root="$1"
explicit_cognitive="$2"
explicit_kernel="$3"
cognitive=""
if [ -n "$explicit_cognitive" ]; then
  if [ -x "$explicit_cognitive" ]; then
    cognitive="$explicit_cognitive"
  else
    echo "COGNITIVE=missing-explicit"
    exit 95
  fi
elif [ -x "$install_root/current/bin/cognitive" ]; then
  cognitive="$install_root/current/bin/cognitive"
else
  newest=$(ls -1dt "$HOME"/p8t*-*/bin/cognitive 2>/dev/null | sed -n '1p' || true)
  if [ -n "$newest" ] && [ -x "$newest" ]; then
    cognitive="$newest"
  fi
fi
if [ -z "$cognitive" ]; then
  echo "COGNITIVE=none"
  exit 0
fi
echo "COGNITIVE=$cognitive"
kernel=""
if [ -n "$explicit_kernel" ]; then
  if [ -x "$explicit_kernel" ]; then
    kernel="$explicit_kernel"
  else
    echo "KERNEL=missing-explicit"
    exit 95
  fi
else
  sibling="$(dirname "$cognitive")/kernel-server"
  if [ -x "$sibling" ]; then kernel="$sibling"; fi
fi
if [ -n "$kernel" ]; then
  echo "KERNEL=$kernel"
else
  echo "KERNEL=none"
fi
'@

Write-Stage "[4/7] resolving revision $Revision and syncing binaries (SyncMode=$SyncMode)"
$dryFlag = 'no'
if ($DryRun) { $dryFlag = 'yes' }

$targetSha = $null
if ($SyncMode -ne 'never') {
  $resolveOut = Invoke-JumpBash -Payload $resolvePayload -PayloadArgs @($RepoUrl, $JumpWorkDir, $Revision, $dryFlag)
  if ($script:RemoteExitCode -ne 0) {
    Stop-Boot 6 "could not resolve revision '$Revision' on the jump host (clone/fetch/rev-parse failed; is the revision pushed?)"
  }
  $targetSha = Get-Token $resolveOut 'RESOLVED_SHA'
  if (-not $targetSha) { Stop-Boot 6 'revision resolution returned no SHA' }
  Write-Info "resolved target revision: $targetSha"
} else {
  Write-Info 'SyncMode=never: no revision resolve, reusing existing guest binaries'
}
$summary['target_revision'] = "$targetSha"

$installedOut = Invoke-GuestBash -Payload $guestInstalledPayload -PayloadArgs @($GuestInstallRoot)
$installedSha = Get-Token $installedOut 'INSTALLED_SHA'
Write-Info "guest script-managed install: $installedSha"

$needBuild = $false
if ($SyncMode -eq 'always') { $needBuild = $true }
elseif ($SyncMode -eq 'auto') { $needBuild = ($installedSha -ne $targetSha) }

if ($needBuild) {
  if ($DryRun) {
    Write-Warn2 "[dry-run] would build $targetSha on the jump host and install to $GuestInstallRoot"
  } else {
    Write-Info 'building cognitive + kernel-server on the jump host (release, --locked; first build may take a while)'
    $buildOut = Invoke-JumpBash -Payload $buildPayload -PayloadArgs @($JumpWorkDir, $targetSha)
    if ($script:RemoteExitCode -ne 0 -or (Get-Token $buildOut 'BUILD') -ne 'ok') {
      if ((Get-Token $buildOut 'BUILD') -eq 'cargo-missing') {
        Stop-Boot 6 'cargo is not available on the jump host ($HOME/.cargo/bin); install rustup there first'
      }
      Stop-Boot 6 'jump-host build failed (see cargo output above)'
    }
    $expectCognitive = Get-Token $buildOut 'SHA_COGNITIVE'
    $expectKernel = Get-Token $buildOut 'SHA_KERNEL'
    $shortSha = $targetSha.Substring(0, 8)
    $destBin = "$GuestInstallRoot/$shortSha/bin"

    $prepOut = Invoke-GuestBash -Payload $guestPreparePayload -PayloadArgs @($destBin)
    if ($script:RemoteExitCode -ne 0) { Stop-Boot 7 "could not create $destBin on the guest" }

    Write-Info "transferring binaries to guest $destBin"
    $scpOut = Invoke-JumpBash -Payload $jumpScpPayload -PayloadArgs @($JumpWorkDir, $GuestHost, $destBin)
    if ($script:RemoteExitCode -ne 0 -or (Get-Token $scpOut 'SCP') -ne 'ok') {
      Write-Warn2 'direct jump->guest scp failed; falling back to a local relay pipe (tar over two SSH hops)'
      $sshOptsFlat = "-o BatchMode=yes -o ConnectTimeout=$ConnectTimeoutSec"
      $releaseDir = "$JumpWorkDir/cognitive-os/target/release"
      $pipeline = "ssh $sshOptsFlat $JumpHost tar -C $releaseDir -czf - cognitive kernel-server | ssh $sshOptsFlat -J $JumpHost $GuestHost tar -C $destBin -xzf -"
      & cmd.exe /d /c $pipeline
      if ($LASTEXITCODE -ne 0) {
        Stop-Boot 7 'binary transfer failed on both routes (jump->guest scp and local relay pipe)'
      }
    }

    $finalizeOut = Invoke-GuestBash -Payload $guestFinalizePayload -PayloadArgs @($GuestInstallRoot, $shortSha, $targetSha, $expectCognitive, $expectKernel)
    if ($script:RemoteExitCode -ne 0 -or (Get-Token $finalizeOut 'FINALIZE') -ne 'ok') {
      Stop-Boot 7 'guest install finalize failed (digest mismatch or filesystem error); nothing was activated'
    }
    Write-Info "installed $targetSha as $GuestInstallRoot/current (digest-verified)"
    $installedSha = $targetSha
  }
} else {
  Write-Info 'guest already has the exact target revision (or SyncMode=never); no build needed'
}

$discoverOut = Invoke-GuestBash -Payload $guestDiscoverPayload -PayloadArgs @($GuestInstallRoot, $CognitiveBin, $KernelServerBin)
if ($script:RemoteExitCode -ne 0) {
  Stop-Boot 7 'explicitly requested guest binary path is missing or not executable'
}
$cognitivePath = Get-Token $discoverOut 'COGNITIVE'
$kernelPath = Get-Token $discoverOut 'KERNEL'
if (-not $cognitivePath -or $cognitivePath -eq 'none') {
  if ($DryRun) {
    Write-Warn2 '[dry-run] no cognitive binary on the guest yet; a real run would have installed one above'
    $cognitivePath = "$GuestInstallRoot/current/bin/cognitive"
    $kernelPath = "$GuestInstallRoot/current/bin/kernel-server"
  } else {
    Stop-Boot 7 "no cognitive binary found on the guest (checked -CognitiveBin, $GuestInstallRoot/current, ~/p8t*-*/bin); rerun with SyncMode=auto|always"
  }
}
if (-not $kernelPath -or $kernelPath -eq 'none') {
  Stop-Boot 7 "no kernel-server binary found next to $cognitivePath; pass -KernelServerBin explicitly"
}
Write-Info "cognitive:     $cognitivePath"
Write-Info "kernel-server: $kernelPath"
$summary['installed_revision'] = "$installedSha"
$summary['cognitive'] = "$cognitivePath"
$summary['kernel_server'] = "$kernelPath"

# ---------------------------------------------------------------------------
# Stage 5: ensure the Personal daemon
# ---------------------------------------------------------------------------

$daemonPayload = @'
set -eu
runtime_root="$1"
bind="$2"
cognitive="$3"
kernel_server="$4"
restart="$5"
dry_run="$6"
health_url="http://$bind/personal/health"
lock_path="$runtime_root/runtime/cognitiveos/daemon.lock"
log_path="$runtime_root/state/cognitiveos/daemon.log"

code=$(curl -fsS -o /dev/null -w '%{http_code}' --max-time 5 "$health_url" 2>/dev/null || true)
if [ "$code" = "200" ] && [ "$restart" != "yes" ]; then
  echo "DAEMON=healthy-reused"
  if [ -f "$lock_path" ]; then
    pid=$(tr -dc '0-9' < "$lock_path" | cut -c1-12)
    echo "DAEMON_PID=$pid"
    echo "DAEMON_EXE=$(readlink -f "/proc/$pid/exe" 2>/dev/null || echo unknown)"
  fi
  exit 0
fi
if [ "$dry_run" = "yes" ]; then
  echo "DAEMON=dry-run-would-start"
  exit 0
fi
if [ "$code" = "200" ] && [ "$restart" = "yes" ]; then
  echo "DAEMON_RESTART=stopping" 1>&2
  "$cognitive" daemon stop --runtime-root "$runtime_root" 1>&2
  sleep 1
fi
status_json=$("$cognitive" daemon status --runtime-root "$runtime_root" 2>/dev/null || true)
if printf '%s' "$status_json" | grep -q '"lock_exists": *true' \
   && printf '%s' "$status_json" | grep -q '"process_alive": *false'; then
  rm -f "$lock_path"
  echo "DAEMON_STALE_LOCK_REMOVED=yes"
fi
"$cognitive" daemon start --runtime-root "$runtime_root" --bind "$bind" --kernel-server "$kernel_server" 1>&2
for _ in $(seq 1 50); do
  code=$(curl -fsS -o /dev/null -w '%{http_code}' --max-time 5 "$health_url" 2>/dev/null || true)
  if [ "$code" = "200" ]; then
    echo "DAEMON=started-healthy"
    exit 0
  fi
  sleep 0.4
done
echo "DAEMON=unhealthy"
echo "---- daemon.log tail (product log is secret-free by contract) ----" 1>&2
tail -n 25 "$log_path" 1>&2 || true
exit 98
'@

Write-Stage "[5/7] ensuring Personal daemon on $Bind (runtime root $RuntimeRoot)"
$restartFlag = 'no'
if ($RestartDaemon) { $restartFlag = 'yes' }
$daemonOut = Invoke-GuestBash -Payload $daemonPayload -PayloadArgs @($RuntimeRoot, $Bind, $cognitivePath, $kernelPath, $restartFlag, $dryFlag)
$daemonState = Get-Token $daemonOut 'DAEMON'
if ($script:RemoteExitCode -ne 0 -or $daemonState -eq 'unhealthy' -or -not $daemonState) {
  Stop-Boot 8 "Personal daemon is not healthy on $Bind; inspect the daemon.log tail above (common causes: wrong bind, runtime permissions, migration lock)"
}
Write-Info "daemon: $daemonState"
$daemonPid = Get-Token $daemonOut 'DAEMON_PID'
$daemonExe = Get-Token $daemonOut 'DAEMON_EXE'
if ($daemonPid) { Write-Info "daemon pid $daemonPid ($daemonExe)" }
if ($daemonState -eq 'healthy-reused' -and $targetSha -and $daemonExe -and $daemonExe -ne 'unknown' -and $daemonExe -notlike "*$($targetSha.Substring(0,8))*") {
  Write-Warn2 "healthy daemon reused on $daemonExe; a newer revision is installed - rerun with -RestartDaemon to switch"
}
$summary['daemon'] = "$daemonState ($Bind)"

# ---------------------------------------------------------------------------
# Stage 6: ensure dsh (installed Cos tree, Path B) and the web panel
# ---------------------------------------------------------------------------

$dshPayload = @'
set -eu
runtime_root="$1"
cognitive="$2"
dsh_root="$3"
adapter_root="$4"
dsh_rev="$5"
mode="$6"
port="$7"
reconfigure="$8"
dry_run="$9"
cfg="$runtime_root/config/cognitiveos/dsh.json"

need=yes
if [ "$reconfigure" != "yes" ] && [ -f "$cfg" ]; then
  if grep -q "\"dsh_root\"[[:space:]]*:[[:space:]]*\"$dsh_root\"" "$cfg" \
     && grep -q "\"adapter_root\"[[:space:]]*:[[:space:]]*\"$adapter_root\"" "$cfg"; then
    need=no
  fi
fi
if [ "$need" = "yes" ]; then
  if [ ! -d "$dsh_root" ]; then
    echo "DSH=missing_dsh_root"
    exit 93
  fi
  if [ ! -d "$adapter_root" ]; then
    echo "DSH=missing_adapter_root"
    exit 93
  fi
  if [ "$dry_run" = "yes" ]; then
    echo "DSH_CONFIGURED=dry-run-would-configure"
  else
    "$cognitive" dsh configure --runtime-root "$runtime_root" --dsh-root "$dsh_root" --adapter-root "$adapter_root" --revision "$dsh_rev" 1>&2
    echo "DSH_CONFIGURED=fresh"
  fi
else
  echo "DSH_CONFIGURED=reused"
fi

if [ "$mode" = "web" ]; then
  code=$(curl -fsS -o /dev/null -w '%{http_code}' --max-time 5 "http://127.0.0.1:$port/" 2>/dev/null || true)
  if [ "$code" = "200" ]; then
    echo "DSH_WEB=already-listening"
  elif [ "$dry_run" = "yes" ]; then
    echo "DSH_WEB=dry-run-would-start"
  else
    log="$runtime_root/state/cognitiveos/dsh-web.boot.log"
    mkdir -p "$(dirname "$log")"
    setsid nohup "$cognitive" dsh web --runtime-root "$runtime_root" --host 127.0.0.1 --port "$port" --no-open >>"$log" 2>&1 < /dev/null &
    code=""
    for _ in $(seq 1 60); do
      code=$(curl -fsS -o /dev/null -w '%{http_code}' --max-time 3 "http://127.0.0.1:$port/" 2>/dev/null || true)
      if [ "$code" = "200" ]; then break; fi
      sleep 0.5
    done
    if [ "$code" = "200" ]; then
      echo "DSH_WEB=started"
    else
      echo "DSH_WEB=failed"
      echo "---- dsh-web.boot.log tail ----" 1>&2
      tail -n 20 "$log" 1>&2 || true
      exit 92
    fi
  fi
fi
if [ "$dry_run" != "yes" ]; then
  echo "DSH_STATUS_JSON_BEGIN"
  "$cognitive" dsh status --runtime-root "$runtime_root" || true
fi
'@

if ($SkipDsh) {
  Write-Stage '[6/7] dsh stage skipped (-SkipDsh)'
  $summary['dsh'] = 'skipped'
} else {
  Write-Stage "[6/7] ensuring dsh (Path B against installed Cos tree; mode=$DshMode)"
  $reconfFlag = 'no'
  if ($ReconfigureDsh) { $reconfFlag = 'yes' }
  $dshOut = Invoke-GuestBash -Payload $dshPayload -PayloadArgs @($RuntimeRoot, $cognitivePath, $DshRoot, $AdapterRoot, $DshRevision, $DshMode, "$DshWebPort", $reconfFlag, $dryFlag)
  if ($script:RemoteExitCode -ne 0) {
    $dshErr = Get-Token $dshOut 'DSH'
    if ($dshErr -eq 'missing_dsh_root') { Stop-Boot 9 "installed Cos dsh tree not found at $DshRoot; pass -DshRoot or reinstall (P8-T10 layout)" }
    if ($dshErr -eq 'missing_adapter_root') { Stop-Boot 9 "AKP adapter root not found at $AdapterRoot; pass -AdapterRoot (P8-T15 layout)" }
    Stop-Boot 9 'dsh configure or web panel start failed (see output above)'
  }
  $dshConfigured = Get-Token $dshOut 'DSH_CONFIGURED'
  $dshWeb = Get-Token $dshOut 'DSH_WEB'
  Write-Info "dsh configuration: $dshConfigured"
  if ($DshMode -eq 'web') {
    Write-Info "dsh web panel: $dshWeb (http://127.0.0.1:$DshWebPort/ on the guest)"
    $summary['dsh_web'] = "$dshWeb (127.0.0.1:$DshWebPort on guest)"
  }
  $summary['dsh'] = "$dshConfigured (root $DshRoot)"
  $statusLines = @()
  $inStatus = $false
  foreach ($line in @($dshOut)) {
    if ("$line" -eq 'DSH_STATUS_JSON_BEGIN') { $inStatus = $true; continue }
    if ($inStatus) { $statusLines += "$line" }
  }
  if ($statusLines.Count -gt 0) {
    Write-Info 'cognitive dsh status (redacted product projection):'
    foreach ($line in $statusLines) { Write-Host "      $line" }
  }

  if ($DshSmokeTask -and -not $DryRun) {
    Write-Info 'running opt-in dsh Path B smoke task (consumes one real Provider call)'
    $smokePayload = @'
set -eu
runtime_root="$1"
cognitive="$2"
task="$3"
"$cognitive" dsh launch --runtime-root "$runtime_root" --print --path b --task "$task"
'@
    $smokeOut = Invoke-GuestBash -Payload $smokePayload -PayloadArgs @($RuntimeRoot, $cognitivePath, $DshSmokeTask)
    if ($script:RemoteExitCode -ne 0) {
      Write-Warn2 'dsh smoke task failed (panel/daemon may still be healthy; see output above)'
      $summary['dsh_smoke'] = 'failed'
    } else {
      foreach ($line in @($smokeOut)) { Write-Host "      $line" }
      $summary['dsh_smoke'] = 'completed'
    }
  }
}

# ---------------------------------------------------------------------------
# Stage 7: ensure Pi is configured and ready
# ---------------------------------------------------------------------------

$piPayload = @'
set -eu
runtime_root="$1"
cognitive="$2"
pi_exe="$3"
pi_ext="$4"
dry_run="$5"
cfg="$runtime_root/config/cognitiveos/pi.json"

json_field() {
  sed -n 's/.*"'"$2"'"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | sed -n '1p'
}

if [ -f "$cfg" ] && [ -z "$pi_exe" ] && [ -z "$pi_ext" ]; then
  exe=$(json_field "$cfg" executable_path)
  ext=$(json_field "$cfg" extension_entry_path)
  if [ -n "$exe" ] && [ -f "$exe" ] && [ -n "$ext" ] && [ -f "$ext" ]; then
    echo "PI=configured-reused"
    echo "PI_EXE=$exe"
    echo "PI_EXT=$ext"
    exit 0
  fi
  echo "existing pi.json points at missing files; rediscovering" 1>&2
fi

if [ -z "$pi_exe" ]; then
  pi_exe=$(ls -1dt "$HOME"/*/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js 2>/dev/null | sed -n '1p' || true)
fi
if [ -z "$pi_ext" ] && [ -n "$pi_exe" ]; then
  base=$(printf '%s' "$pi_exe" | sed 's#/pi-runtime/node_modules/.*##')
  if [ -f "$base/pi-cognitiveos/dist/index.js" ]; then
    pi_ext="$base/pi-cognitiveos/dist/index.js"
  fi
fi
if [ -z "$pi_ext" ]; then
  pi_ext=$(ls -1dt "$HOME"/*/pi-cognitiveos/dist/index.js 2>/dev/null | sed -n '1p' || true)
fi
if [ -z "$pi_exe" ] || [ ! -f "$pi_exe" ] || [ -z "$pi_ext" ] || [ ! -f "$pi_ext" ]; then
  echo "PI=not_configured"
  exit 94
fi
if [ "$dry_run" = "yes" ]; then
  echo "PI=dry-run-would-configure"
else
  "$cognitive" pi configure --runtime-root "$runtime_root" --executable "$pi_exe" --extension-entry "$pi_ext" 1>&2
  echo "PI=configured-fresh"
fi
echo "PI_EXE=$pi_exe"
echo "PI_EXT=$pi_ext"
'@

if ($SkipPi) {
  Write-Stage '[7/7] Pi stage skipped (-SkipPi)'
  $summary['pi'] = 'skipped'
} else {
  Write-Stage '[7/7] ensuring Pi configuration (pin 0.81.1 installed tree reuse)'
  $piOut = Invoke-GuestBash -Payload $piPayload -PayloadArgs @($RuntimeRoot, $cognitivePath, $PiExecutable, $PiExtensionEntry, $dryFlag)
  $piState = Get-Token $piOut 'PI'
  if ($script:RemoteExitCode -ne 0 -or $piState -eq 'not_configured' -or -not $piState) {
    $hint = "no valid pi.json and no installed Pi anchors found under the guest home; " +
      "pass -PiExecutable/-PiExtensionEntry (pin @earendil-works/pi-coding-agent@0.81.1) " +
      "or install Pi per the handbook, then rerun (or use -SkipPi)"
    Stop-Boot 10 "Pi is not ready: $hint"
  }
  $piExe = Get-Token $piOut 'PI_EXE'
  $piExt = Get-Token $piOut 'PI_EXT'
  Write-Info "pi: $piState"
  Write-Info "pi executable: $piExe"
  Write-Info "pi extension:  $piExt"
  $summary['pi'] = "$piState"
}

# ---------------------------------------------------------------------------
# Final redacted status summary
# ---------------------------------------------------------------------------

$statusPayload = @'
set -eu
runtime_root="$1"
cognitive="$2"
if [ ! -x "$cognitive" ]; then
  echo "STATUS=binary-missing"
  exit 0
fi
echo "---- cognitive status ----"
"$cognitive" status --runtime-root "$runtime_root" || true
echo "---- cognitive doctor ----"
doctor_output=$("$cognitive" doctor --runtime-root "$runtime_root" 2>&1 || true)
printf '%s\n' "$doctor_output"
if printf '%s' "$doctor_output" | grep -q '"first_conversation_ready": *true'; then
  echo "FIRST_CONVERSATION_READY=true"
else
  echo "FIRST_CONVERSATION_READY=false"
fi
'@

Write-Stage 'final redacted status'
if (-not $DryRun) {
  $statusOut = Invoke-GuestBash -Payload $statusPayload -PayloadArgs @($RuntimeRoot, $cognitivePath)
  foreach ($line in @($statusOut)) {
    if ("$line" -like 'FIRST_CONVERSATION_READY=*') { continue }
    Write-Host "      $line"
  }
  $fcr = Get-Token $statusOut 'FIRST_CONVERSATION_READY'
  $summary['first_conversation_ready'] = "$fcr"
}

Write-Host ''
Write-Host 'linux002-boot-personal: summary' -ForegroundColor Green
foreach ($key in $summary.Keys) {
  Write-Host ("    {0,-24} {1}" -f $key, $summary[$key])
}
Write-Host ''
Write-Host '    Interactive Pi session (run from this machine):'
Write-Host ("      ssh -t -o BatchMode=yes -J {0} {1} {2} pi launch --runtime-root {3}" -f $JumpHost, $GuestHost, $cognitivePath, $RuntimeRoot)
if ($DshMode -eq 'web' -and -not $SkipDsh) {
  Write-Host '    dsh web panel (loopback on the guest; tunnel if needed):'
  Write-Host ("      ssh -L {0}:127.0.0.1:{0} -J {1} {2}   # then open http://127.0.0.1:{0}/" -f $DshWebPort, $JumpHost, $GuestHost)
}
Write-Host ''
Write-Host '    Non-claims: owner-ops convenience only; no Gate / release / Profile /'
Write-Host '    B01 / EVAL / Agent-benefit claim is created by this script.'
Write-Host ''

if ($LaunchPi -and -not $DryRun -and -not $SkipPi) {
  Write-Stage 'opening interactive Pi session (Ctrl+C / exit to leave; agents stay up)'
  & ssh -t -o BatchMode=yes -o "ConnectTimeout=$ConnectTimeoutSec" -J $JumpHost $GuestHost "$cognitivePath pi launch --runtime-root $RuntimeRoot"
}

exit 0
