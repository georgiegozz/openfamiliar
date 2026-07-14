#Requires -Version 5.1
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$output = Join-Path $root "docs\legal\dependency-licenses.json"
Push-Location $root
try {
    $cargoRaw = cargo metadata --format-version 1 --locked | Out-String
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed" }
    $cargo = $cargoRaw | ConvertFrom-Json
    $rust = @(
        $cargo.packages |
            Sort-Object name, version |
            ForEach-Object {
                [ordered]@{
                    name = $_.name
                    version = $_.version
                    license = if ([string]::IsNullOrWhiteSpace($_.license)) { "NOASSERTION" } else { $_.license }
                }
            }
    )

    $pnpmRaw = pnpm licenses list --prod --json | Out-String
    if ($LASTEXITCODE -ne 0) { throw "pnpm licenses failed" }
    $pnpm = $pnpmRaw | ConvertFrom-Json
    $javascript = @(
        foreach ($licenseGroup in $pnpm.PSObject.Properties | Sort-Object Name) {
            foreach ($dependency in $licenseGroup.Value | Sort-Object name) {
                foreach ($version in $dependency.versions | Sort-Object) {
                    [ordered]@{
                        name = $dependency.name
                        version = $version
                        license = $dependency.license
                    }
                }
            }
        }
    )

    $document = [ordered]@{
        schemaVersion = 1
        source = [ordered]@{
            rust = "Cargo.lock via cargo metadata --locked"
            javascript = "pnpm-lock.yaml via pnpm licenses list --prod"
        }
        rust = $rust
        javascriptProduction = $javascript
    }
    $json = $document | ConvertTo-Json -Depth 8
    $directory = Split-Path -Parent $output
    if (-not (Test-Path -LiteralPath $directory)) {
        New-Item -ItemType Directory -Path $directory | Out-Null
    }
    [System.IO.File]::WriteAllText($output, $json + [Environment]::NewLine, [System.Text.UTF8Encoding]::new($false))
    & pnpm exec prettier --write $output
    if ($LASTEXITCODE -ne 0) { throw "prettier failed for dependency inventory" }
    Write-Host "Wrote $output"
}
finally {
    Pop-Location
}
