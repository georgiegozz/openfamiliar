#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
Set-Location $PSScriptRoot\..
pnpm --filter @openfamiliar/desktop dev