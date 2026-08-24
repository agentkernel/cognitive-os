<#
.SYNOPSIS
  Repo-root launcher for scripts\linux002-boot-personal.ps1. Identical
  parameters; every argument is forwarded unchanged.

.DESCRIPTION
  Convenience entry point so the canonical operator paste works right after
  cd-ing into the clone root, without remembering the scripts\ prefix:

    cd C:\path\to\your\cognitive-os
    powershell -ExecutionPolicy Bypass -File .\linux002-boot-personal.ps1 -EnsureGuestRunning -LaunchPi

  See scripts\linux002-boot-personal.ps1 for the full parameter reference,
  examples, exit codes, the secret boundary, and the TROUBLESHOOTING section
  covering the "-File ... does not exist" failure (wrong current directory,
  or a checkout that predates PR #268 /
  branch cursor/linux002-boot-personal-ops-script-ac2d).
#>

# No param() block on purpose: every argument (including switches such as
# -EnsureGuestRunning and -LaunchPi) is forwarded verbatim via @args, so this
# launcher can never drift out of sync with the real parameter set.

Set-StrictMode -Version 2.0

if ([string]::IsNullOrEmpty($PSCommandPath)) {
  Write-Host ''
  Write-Host 'ERROR (2): linux002-boot-personal.ps1 was not started as a script file.' -ForegroundColor Red
  Write-Host '  Run the on-disk .ps1 from your cognitive-os clone:'
  Write-Host '    cd C:\path\to\your\cognitive-os'
  Write-Host '    powershell -ExecutionPolicy Bypass -File .\linux002-boot-personal.ps1 -EnsureGuestRunning -LaunchPi'
  exit 2
}

$inner = Join-Path $PSScriptRoot 'scripts\linux002-boot-personal.ps1'
if (-not (Test-Path -LiteralPath $inner -PathType Leaf)) {
  Write-Host ''
  Write-Host "ERROR (2): expected the real script at $inner but it is missing." -ForegroundColor Red
  Write-Host '  This launcher only works inside a cognitive-os clone that contains'
  Write-Host '  scripts\linux002-boot-personal.ps1. Until PR #268 is merged, fetch it:'
  Write-Host '    git fetch origin cursor/linux002-boot-personal-ops-script-ac2d'
  Write-Host '    git checkout cursor/linux002-boot-personal-ops-script-ac2d'
  Write-Host '  After PR #268 is merged to main: git checkout main; git pull origin main'
  exit 2
}

& $inner @args
exit $LASTEXITCODE
