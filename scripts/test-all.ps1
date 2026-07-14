#Requires -Version 5.1
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path

pnpm format:check
pnpm typecheck
pnpm test
pnpm assets:check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -q -p familiar-cli -- pack validate mascots/perrito-tech
cargo run -q -p familiar-cli -- pack validate mascots/blank-template
Write-Host "All local checks finished"
