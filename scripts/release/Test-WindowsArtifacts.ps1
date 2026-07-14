#Requires -Version 5.1

[CmdletBinding()]
param(
    [Parameter()]
    [string[]]$Path = @(),

    [Parameter()]
    [switch]$RequireSignature
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$resolvedArtifacts = @()

if ($Path.Count -gt 0) {
    foreach ($item in $Path) {
        $resolvedArtifacts += Get-Item -LiteralPath $item -ErrorAction Stop
    }
}
else {
    $appExecutable = Join-Path $repoRoot 'target\release\openfamiliar-desktop.exe'
    if (Test-Path -LiteralPath $appExecutable -PathType Leaf) {
        $resolvedArtifacts += Get-Item -LiteralPath $appExecutable
    }
    $bundleRoot = Join-Path $repoRoot 'target\release\bundle'
    if (Test-Path -LiteralPath $bundleRoot -PathType Container) {
        $resolvedArtifacts += @(
            Get-ChildItem -LiteralPath $bundleRoot -Recurse -File |
                Where-Object { $_.Extension -in '.exe', '.msi' }
        )
    }
}

if ($resolvedArtifacts.Count -eq 0) {
    throw 'No Windows installer artifacts were found. Build the MSI/NSIS bundles first or pass -Path.'
}

$results = foreach ($artifact in $resolvedArtifacts) {
    if ($artifact.Extension -notin '.exe', '.msi') {
        throw "Unsupported Windows artifact: $($artifact.FullName)"
    }

    $signature = Get-AuthenticodeSignature -LiteralPath $artifact.FullName
    $hash = Get-FileHash -LiteralPath $artifact.FullName -Algorithm SHA256
    [pscustomobject]@{
        File = $artifact.FullName
        Bytes = $artifact.Length
        SHA256 = $hash.Hash
        SignatureStatus = $signature.Status.ToString()
        Publisher = if ($null -ne $signature.SignerCertificate) {
            $signature.SignerCertificate.Subject
        }
        else {
            $null
        }
        Timestamped = $null -ne $signature.TimeStamperCertificate
    }
}

$results | ConvertTo-Json -Depth 3

if ($RequireSignature -and @($results | Where-Object SignatureStatus -ne 'Valid').Count -gt 0) {
    throw 'At least one artifact does not have a valid trusted Authenticode signature.'
}
