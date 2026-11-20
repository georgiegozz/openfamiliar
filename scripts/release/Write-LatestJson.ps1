#Requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$DownloadBaseUrl,
    [Parameter(Mandatory = $true)]
    [string]$BundleRoot,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-UpdaterAsset {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Directory,
        [Parameter(Mandatory = $true)]
        [string]$Filter
    )
    if (-not (Test-Path -LiteralPath $Directory)) {
        return $null
    }
    $file = Get-ChildItem -LiteralPath $Directory -File -Filter $Filter |
        Sort-Object Name |
        Select-Object -First 1
    if (-not $file) {
        return $null
    }
    $sigPath = "$($file.FullName).sig"
    if (-not (Test-Path -LiteralPath $sigPath)) {
        throw "Missing signature file for $($file.Name)"
    }
    [pscustomobject]@{
        Name = $file.Name
        Signature = (Get-Content -LiteralPath $sigPath -Raw).Trim()
    }
}

$nsis = Get-UpdaterAsset -Directory (Join-Path $BundleRoot "nsis") -Filter "*.exe"
$msi = Get-UpdaterAsset -Directory (Join-Path $BundleRoot "msi") -Filter "*.msi"
if (-not $nsis -and -not $msi) {
    throw "No signed NSIS or MSI updater artifacts were found under $BundleRoot"
}

# Prefer the NSIS installer for in-app updates; MSI is a fallback.
$asset = if ($nsis) { $nsis } else { $msi }
$platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
        signature = $asset.Signature
        url = "$($DownloadBaseUrl.TrimEnd('/'))/$($asset.Name)"
    }
}

$payload = [ordered]@{
    version = $Version.TrimStart('v')
    notes = "OpenFamiliar $Version"
    pub_date = [DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = $platforms
}

$json = $payload | ConvertTo-Json -Depth 8
$dir = Split-Path -Parent $OutputPath
if ($dir -and -not (Test-Path -LiteralPath $dir)) {
    New-Item -ItemType Directory -Path $dir | Out-Null
}
Set-Content -LiteralPath $OutputPath -Value $json -Encoding utf8
Write-Output "Wrote $OutputPath for $($asset.Name)"
