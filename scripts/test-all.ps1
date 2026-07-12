#Requires -Version 5.1
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path

pnpm test
cargo test --workspace
cargo run -q -p familiar-cli -- pack validate mascots/perrito-tech
cargo run -q -p familiar-cli -- pack validate mascots/blank-template
Write-Host "All local checks finished"