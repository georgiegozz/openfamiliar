# Windows development notes

## Toolchain

1. **Node.js 20+** and **pnpm 9+**
2. **Rust stable** (`rustup`)
3. **Visual Studio Build Tools 2022** with workload **Desktop development with C++**
   (`link.exe` / MSVC). Required for `x86_64-pc-windows-msvc`.
4. **WebView2** (usually preinstalled on Windows 11)
5. Optional: **Ollama** for local models

## Bootstrap

```powershell
cd "C:\Users\jorge gonzalez\Music\proyects\openfamiliar"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
.\scripts\bootstrap.ps1
.\scripts\test-all.ps1
```

If `cargo` fails with `linker link.exe not found`, finish installing VS Build Tools
and open a **new** PowerShell so the environment is refreshed. You can also run:

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64
```

## Desktop

```powershell
pnpm --filter @openfamiliar/desktop dev          # Vite UI only
pnpm --filter @openfamiliar/desktop exec tauri dev  # full shell
```

## CLI

```powershell
cargo run -p familiar-cli -- pack validate mascots/perrito-tech
cargo run -p familiar-cli -- pack build mascots/blank-template
```
