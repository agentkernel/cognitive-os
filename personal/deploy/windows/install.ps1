# CognitiveOS Personal Windows bootstrap template.
#
# Release automation renders every CognitiveOS policy placeholder below into a
# reviewed, version-specific script before publication. The source template
# deliberately fails closed and must not be represented as a usable release
# installer. It mirrors personal/deploy/linux/install.sh: policy validation before any
# network or filesystem action, a private owned temporary directory, bounded
# HTTPS-only downloads through the absolute System32 curl.exe, digest
# verification of the downloaded installer, and delegation to that verified
# installer. This bootstrap never reads, writes, or receives secret material.
#
# P11-T02 host layout policy (inspectable, not a rendered installer):
# Personal Home/app/ is replaceable application bytes; Personal Home/data/ is
# retained across upgrades. This template is not a second credential plane
# (Windows Credential Manager remains the approved backend). DSH web is not
# the host shell. A tray icon is not proof of work. Close may ask
# background-or-pause only if the daemon can honor it. Same-disk automatic
# versions are local restore points, not disaster backups. GNU/WSL/Linux
# evidence is not a Windows product host.

param([string]$RequestedVersion)

Set-StrictMode -Version 2.0
$ErrorActionPreference = 'Stop'

$script:ReleaseVersion = '@COGNITIVEOS_RELEASE_VERSION@'
$script:ReleaseObjectDirectory = '@COGNITIVEOS_RELEASE_OBJECT_DIRECTORY@'
$script:AllowedRedirectHost = '@COGNITIVEOS_ALLOWED_REDIRECT_HOST@'
$script:InstallerSha256 = '@COGNITIVEOS_INSTALLER_SHA256@'
$script:TrustedKeyringVersion = '@COGNITIVEOS_TRUSTED_KEYRING_VERSION@'
$script:TrustedKeyId = '@COGNITIVEOS_TRUSTED_KEY_ID@'
$script:TrustedPublicKeyBase64Url = '@COGNITIVEOS_TRUSTED_PUBLIC_KEY_BASE64URL@'
$script:ExpectedPiVersion = '@COGNITIVEOS_EXPECTED_PI_VERSION@'
$script:ExpectedPiIntegrity = '@COGNITIVEOS_EXPECTED_PI_INTEGRITY@'

$script:InstallerFilename = 'cognitiveos-windows-bundle-installer.exe'
$script:ManifestFilename = 'manifest.json'
$script:ArtifactFilename = 'cognitiveos-windows-x86_64.zip'
$script:StatementFilename = 'attestation.statement.json'
$script:SignatureFilename = 'attestation.signature.json'
$script:MaxInstallerBytes = 33554432
$script:MaxMetadataBytes = 65536
$script:MaxArtifactBytes = 536870912
$script:ConnectTimeoutSeconds = 10
$script:TransferTimeoutSeconds = 120
$script:RetryCount = 2

$script:TempDirectory = $null
$script:TempOwnerMarker = $null
$script:BundleDirectory = $null
$script:InstallerPath = $null

function Write-BootstrapError([string]$message) {
    [Console]::Error.WriteLine("CognitiveOS Windows bootstrap failed: $message")
}

function Test-UnrenderedTemplate([string]$value) {
    return ($value -match '@COGNITIVEOS_[A-Z0-9_]+@')
}

function Test-ControlCharacter([string]$value) {
    foreach ($character in $value.ToCharArray()) {
        if ([char]::IsControl($character)) { return $true }
    }
    return $false
}

function Assert-RenderedValue([string]$value) {
    if ([string]::IsNullOrEmpty($value) -or (Test-UnrenderedTemplate $value) -or (Test-ControlCharacter $value)) {
        Write-BootstrapError 'release policy is not rendered'
        exit 64
    }
}

function Assert-ReleasePolicy {
    Assert-RenderedValue $script:ReleaseVersion
    Assert-RenderedValue $script:ReleaseObjectDirectory
    Assert-RenderedValue $script:AllowedRedirectHost
    Assert-RenderedValue $script:InstallerSha256
    Assert-RenderedValue $script:TrustedKeyringVersion
    Assert-RenderedValue $script:TrustedKeyId
    Assert-RenderedValue $script:TrustedPublicKeyBase64Url
    Assert-RenderedValue $script:ExpectedPiVersion
    Assert-RenderedValue $script:ExpectedPiIntegrity

    if ($script:ReleaseVersion -notmatch '^[A-Za-z0-9._-]+$') {
        Write-BootstrapError 'release policy version is invalid'
        exit 64
    }
    if ($script:ReleaseObjectDirectory -notmatch '^https://.+/.+' -or
        $script:ReleaseObjectDirectory.Contains('@') -or
        $script:ReleaseObjectDirectory.Contains('?') -or
        $script:ReleaseObjectDirectory.Contains('#')) {
        Write-BootstrapError 'release policy URL is invalid'
        exit 64
    }
    if ($script:AllowedRedirectHost -notmatch '^[A-Za-z0-9.-]+$' -or
        $script:AllowedRedirectHost.StartsWith('.') -or
        $script:AllowedRedirectHost.Contains('..')) {
        Write-BootstrapError 'redirect host policy is invalid'
        exit 64
    }
    if ($script:InstallerSha256 -cnotmatch '^sha256:[0-9a-f]{64}$') {
        Write-BootstrapError 'installer digest policy is invalid'
        exit 64
    }
}

function Get-CurlExecutable {
    $curlPath = Join-Path $env:SystemRoot 'System32\curl.exe'
    if (-not (Test-Path -LiteralPath $curlPath -PathType Leaf)) {
        Write-BootstrapError 'bounded system downloader is unavailable'
        exit 70
    }
    return $curlPath
}

function New-PrivateTemporaryDirectory {
    $temporaryBase = $env:TEMP
    if ([string]::IsNullOrEmpty($temporaryBase) -or -not (Test-Path -LiteralPath $temporaryBase -PathType Container)) {
        Write-BootstrapError 'temporary directory is unavailable'
        exit 70
    }
    $randomSuffix = [System.IO.Path]::GetRandomFileName().Replace('.', '')
    $script:TempDirectory = Join-Path $temporaryBase "cognitiveos-bootstrap.$randomSuffix"
    try {
        [void](New-Item -ItemType Directory -Path $script:TempDirectory -ErrorAction Stop)
    } catch {
        Write-BootstrapError 'private temporary directory could not be created'
        exit 70
    }
    $script:TempOwnerMarker = Join-Path $script:TempDirectory '.cognitiveos-bootstrap-owner'
    Set-Content -LiteralPath $script:TempOwnerMarker -Value 'owned' -NoNewline
}

function Remove-TemporaryDirectory {
    if ($null -ne $script:TempDirectory -and
        $null -ne $script:TempOwnerMarker -and
        (Test-Path -LiteralPath $script:TempOwnerMarker -PathType Leaf) -and
        ([System.IO.Path]::GetFileName($script:TempDirectory) -like 'cognitiveos-bootstrap.*')) {
        Remove-Item -LiteralPath $script:TempDirectory -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Invoke-DownloadOnce([string]$downloadUrl, [string]$partialPath, [string]$headerPath, [long]$maximumBytes) {
    $curlPath = Get-CurlExecutable
    $httpStatus = & $curlPath --disable --silent --show-error --fail --globoff --proto '=https' `
        --connect-timeout $script:ConnectTimeoutSeconds `
        --max-time $script:TransferTimeoutSeconds `
        --retry $script:RetryCount --retry-delay 1 --max-filesize $maximumBytes `
        --dump-header $headerPath --output $partialPath `
        --write-out '%{http_code}' --url $downloadUrl
    if ($LASTEXITCODE -ne 0) {
        return $null
    }
    return "$httpStatus".Trim()
}

function Get-RedirectLocation([string]$headerPath) {
    if (-not (Test-Path -LiteralPath $headerPath -PathType Leaf)) { return $null }
    $location = $null
    foreach ($line in Get-Content -LiteralPath $headerPath) {
        if ($line -match '^(?i)Location:\s*(.+?)\s*$') {
            $location = $Matches[1]
        }
    }
    return $location
}

function Test-AllowedRedirect([string]$redirectUrl) {
    return $redirectUrl.StartsWith("https://$($script:AllowedRedirectHost)/")
}

function Invoke-DownloadFile([string]$objectFilename, [long]$maximumBytes) {
    $finalPath = Join-Path $script:BundleDirectory $objectFilename
    $partialPath = "$finalPath.partial"
    $headerPath = Join-Path $script:TempDirectory "$objectFilename.headers"
    $initialUrl = "$($script:ReleaseObjectDirectory)/$objectFilename"

    $httpStatus = Invoke-DownloadOnce $initialUrl $partialPath $headerPath $maximumBytes
    if ($null -eq $httpStatus) {
        Write-BootstrapError 'download failed'
        exit 69
    }
    if ($httpStatus -eq '200') {
        Move-Item -LiteralPath $partialPath -Destination $finalPath -Force
        return
    }
    if ($httpStatus -notin @('301', '302', '303', '307', '308')) {
        Write-BootstrapError 'download returned an unsupported response'
        exit 69
    }
    $redirectUrl = Get-RedirectLocation $headerPath
    if ([string]::IsNullOrEmpty($redirectUrl) -or -not (Test-AllowedRedirect $redirectUrl)) {
        Write-BootstrapError 'download redirect is not allowed'
        exit 69
    }

    Remove-Item -LiteralPath $partialPath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $headerPath -Force -ErrorAction SilentlyContinue
    $redirectedStatus = Invoke-DownloadOnce $redirectUrl $partialPath $headerPath $maximumBytes
    if ($null -eq $redirectedStatus) {
        Write-BootstrapError 'redirected download failed'
        exit 69
    }
    if ($redirectedStatus -ne '200') {
        Write-BootstrapError 'redirected download did not complete'
        exit 69
    }
    Move-Item -LiteralPath $partialPath -Destination $finalPath -Force
}

function Assert-InstallerDigest {
    $actualDigest = $null
    try {
        $actualDigest = (Get-FileHash -LiteralPath $script:InstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
    } catch {
        Write-BootstrapError 'bootstrap installer digest could not be computed'
        exit 69
    }
    if ("sha256:$actualDigest" -cne $script:InstallerSha256) {
        Write-BootstrapError 'bootstrap installer digest does not match release policy'
        exit 69
    }
}

function Invoke-LocalInstaller {
    & $script:InstallerPath `
        --bundle-directory $script:BundleDirectory `
        --expected-release-version $script:ReleaseVersion `
        --expected-pi-version $script:ExpectedPiVersion `
        --expected-pi-integrity $script:ExpectedPiIntegrity `
        --keyring-version $script:TrustedKeyringVersion `
        --key-id $script:TrustedKeyId `
        --public-key-base64url $script:TrustedPublicKeyBase64Url
    exit $LASTEXITCODE
}

function Invoke-Main {
    if ($env:OS -ne 'Windows_NT' -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
        Write-BootstrapError 'this bootstrap supports Windows x86_64 only'
        exit 64
    }
    if ($script:ExtraArgumentCount -gt 0) {
        Write-BootstrapError 'unsupported extra arguments were provided'
        exit 64
    }
    if ($script:RequestedVersionProvided) {
        if ($RequestedVersion -cne $script:ReleaseVersion) {
            Write-BootstrapError 'requested version does not match inspected release policy'
            exit 64
        }
    }

    Assert-ReleasePolicy
    New-PrivateTemporaryDirectory
    $script:BundleDirectory = Join-Path $script:TempDirectory 'bundle'
    $script:InstallerPath = Join-Path $script:TempDirectory $script:InstallerFilename
    [void](New-Item -ItemType Directory -Path $script:BundleDirectory)

    Invoke-DownloadFile $script:InstallerFilename $script:MaxInstallerBytes
    Move-Item -LiteralPath (Join-Path $script:BundleDirectory $script:InstallerFilename) -Destination $script:InstallerPath -Force
    Assert-InstallerDigest
    Invoke-DownloadFile $script:ManifestFilename $script:MaxMetadataBytes
    Invoke-DownloadFile $script:StatementFilename $script:MaxMetadataBytes
    Invoke-DownloadFile $script:SignatureFilename $script:MaxMetadataBytes
    Invoke-DownloadFile $script:ArtifactFilename $script:MaxArtifactBytes
    Invoke-LocalInstaller
}

# Captured at script scope: inside functions $PSBoundParameters and $args
# would describe the function invocation, not this script invocation.
$script:RequestedVersionProvided = $PSBoundParameters.ContainsKey('RequestedVersion') -or -not [string]::IsNullOrEmpty($RequestedVersion)
$script:ExtraArgumentCount = $args.Count

try {
    Invoke-Main
} finally {
    Remove-TemporaryDirectory
}
