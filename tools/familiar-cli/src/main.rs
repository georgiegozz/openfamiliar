//! `familiar` CLI — pack tooling for OpenFamiliar.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const MAX_PACK_BYTES: u64 = 50 * 1024 * 1024;
const ALLOWED_EXTS: &[&str] = &[
    "webp", "png", "jpg", "jpeg", "gif", "md", "json", "txt", "wav", "ogg", "mp3",
];

#[derive(Parser, Debug)]
#[command(name = "familiar", version, about = "OpenFamiliar pack CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Pack operations
    Pack {
        #[command(subcommand)]
        command: PackCommands,
    },
}

#[derive(Subcommand, Debug)]
enum PackCommands {
    /// Create a new pack scaffold
    Init {
        name: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Validate a pack directory
    Validate { path: PathBuf },
    /// Preview pack summary
    Preview { path: PathBuf },
    /// Build a .familiar zip archive
    Build {
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Inspect a pack dir or .familiar archive
    Inspect { path: PathBuf },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FamiliarManifest {
    #[serde(rename = "$schema", default)]
    schema: Option<String>,
    id: String,
    name: String,
    version: String,
    engine: String,
    author: String,
    license: String,
    #[serde(default)]
    homepage: Option<String>,
    personality: String,
    states: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    variants: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    asset_sources: Vec<String>,
    #[serde(default)]
    ai_generated: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Pack { command } => match command {
            PackCommands::Init { name, path } => pack_init(&name, &path)?,
            PackCommands::Validate { path } => {
                let report = validate_pack(&path)?;
                println!("{}", serde_json::to_string_pretty(&report)?);
                if !report.ok {
                    std::process::exit(1);
                }
            }
            PackCommands::Preview { path } => {
                let report = validate_pack(&path)?;
                println!("Pack: {} ({})", report.name, report.id);
                println!("Version: {}", report.version);
                println!("License: {}", report.license);
                println!("States: {}", report.states.join(", "));
                println!("OK: {}", report.ok);
                for w in report.warnings {
                    println!("WARN: {w}");
                }
                for e in report.errors {
                    println!("ERROR: {e}");
                }
            }
            PackCommands::Build { path, out } => {
                let out = out.unwrap_or_else(|| {
                    let id = validate_pack(&path)
                        .map(|r| r.id)
                        .unwrap_or_else(|_| "pack".into());
                    PathBuf::from(format!("{id}.familiar"))
                });
                build_pack(&path, &out)?;
                println!("Built {}", out.display());
            }
            PackCommands::Inspect { path } => {
                if path.extension().and_then(|e| e.to_str()) == Some("familiar") {
                    inspect_archive(&path)?;
                } else {
                    let report = validate_pack(&path)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
            }
        },
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    ok: bool,
    id: String,
    name: String,
    version: String,
    license: String,
    states: Vec<String>,
    variants: Vec<String>,
    file_hashes: Vec<(String, String)>,
    warnings: Vec<String>,
    errors: Vec<String>,
}

fn pack_init(name: &str, path: &Path) -> Result<()> {
    let dir = path.join(name);
    fs::create_dir_all(dir.join("assets"))?;
    fs::create_dir_all(dir.join("sounds"))?;
    fs::create_dir_all(dir.join("previews"))?;
    let id = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let manifest = serde_json::json!({
        "$schema": "https://openfamiliar.dev/schema/familiar-v1.json",
        "id": id,
        "name": name,
        "version": "0.1.0",
        "engine": ">=0.1.0",
        "author": "unknown",
        "license": "CC0-1.0",
        "personality": "personality.md",
        "states": {
            "idle": "assets/idle.webp",
            "thinking": "assets/thinking.webp",
            "working": "assets/working.webp",
            "approval": "assets/approval.webp",
            "success": "assets/success.webp",
            "error": "assets/error.webp"
        },
        "variants": {},
        "assetSources": [],
        "aiGenerated": false
    });
    fs::write(
        dir.join("familiar.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    fs::write(
        dir.join("personality.md"),
        format!("# {name}\n\nA friendly desktop familiar.\n"),
    )?;
    fs::write(
        dir.join("LICENSE"),
        "CC0-1.0 — dedicate this pack's assets to the public domain as appropriate.\n",
    )?;
    // placeholder SVG-like note files so validation can warn about missing webp
    for state in [
        "idle", "thinking", "working", "approval", "success", "error",
    ] {
        fs::write(
            dir.join("assets").join(format!("{state}.webp.txt")),
            format!("Replace with real {state}.webp asset\n"),
        )?;
    }
    println!("Initialized pack at {}", dir.display());
    Ok(())
}

fn validate_pack(path: &Path) -> Result<ValidationReport> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let manifest_path = path.join("familiar.json");
    if !manifest_path.exists() {
        bail!("familiar.json missing");
    }
    let raw = fs::read_to_string(&manifest_path)?;
    let manifest: FamiliarManifest = serde_json::from_str(&raw).context("parse familiar.json")?;

    if manifest.id.trim().is_empty() {
        errors.push("id is required".into());
    }
    if manifest.license.trim().is_empty() {
        errors.push("license is required".into());
        warnings.push("Unlicensed packs get NOASSERTION and cannot join gallery".into());
    }
    if manifest.license.eq_ignore_ascii_case("NOASSERTION") {
        warnings.push("license is NOASSERTION — local import only".into());
    }

    let personality = path.join(&manifest.personality);
    if !personality.exists() {
        errors.push(format!(
            "personality file missing: {}",
            manifest.personality
        ));
    }

    let required_states = [
        "idle", "thinking", "working", "approval", "success", "error",
    ];
    for s in required_states {
        if !manifest.states.contains_key(s) {
            errors.push(format!("missing state mapping: {s}"));
        }
    }

    let mut hashes = Vec::new();
    let mut total: u64 = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(path)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if rel.contains("..") {
            errors.push(format!("path traversal segment: {rel}"));
            continue;
        }
        if Path::new(&rel).is_absolute() {
            errors.push(format!("absolute path not allowed: {rel}"));
            continue;
        }
        let meta = entry.metadata()?;
        total += meta.len();
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if entry.path().extension().is_some() && !ALLOWED_EXTS.contains(&ext.as_str()) {
            // allow .txt placeholders for scaffolding and LICENSE without ext handling
            if ext != "txt" && !rel.eq_ignore_ascii_case("license") {
                warnings.push(format!("unexpected extension .{ext} for {rel}"));
            }
        }
        // no binaries: block common executable extensions
        if matches!(
            ext.as_str(),
            "exe" | "dll" | "so" | "dylib" | "bat" | "cmd" | "ps1" | "js"
        ) {
            errors.push(format!("executable/script not allowed in pack v1: {rel}"));
        }
        let mut file = fs::File::open(entry.path())?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        hashes.push((rel, hex::encode(hasher.finalize())));
    }
    if total > MAX_PACK_BYTES {
        errors.push(format!(
            "pack exceeds size limit ({total} > {MAX_PACK_BYTES})"
        ));
    }

    for (state, asset_rel) in &manifest.states {
        if asset_rel.contains("..") || Path::new(asset_rel).is_absolute() {
            errors.push(format!("invalid asset path for {state}: {asset_rel}"));
            continue;
        }
        let asset_path = path.join(asset_rel);
        if !asset_path.exists() {
            // allow placeholder .webp.txt during authoring
            let placeholder = path.join(format!("{asset_rel}.txt"));
            if placeholder.exists() {
                warnings.push(format!("state {state} uses placeholder for {asset_rel}"));
            } else {
                errors.push(format!("missing asset for {state}: {asset_rel}"));
            }
        }
    }

    for (variant, asset_rel) in &manifest.variants {
        if asset_rel.contains("..") || Path::new(asset_rel).is_absolute() {
            errors.push(format!(
                "invalid asset path for variant {variant}: {asset_rel}"
            ));
            continue;
        }
        if !path.join(asset_rel).exists() {
            errors.push(format!("missing asset for variant {variant}: {asset_rel}"));
        }
    }

    let ok = errors.is_empty();
    Ok(ValidationReport {
        ok,
        id: manifest.id,
        name: manifest.name,
        version: manifest.version,
        license: manifest.license,
        states: required_states.iter().map(|s| s.to_string()).collect(),
        variants: manifest.variants.keys().cloned().collect(),
        file_hashes: hashes,
        warnings,
        errors,
    })
}

fn build_pack(path: &Path, out: &Path) -> Result<()> {
    let report = validate_pack(path)?;
    if !report.ok {
        bail!("validation failed: {:?}", report.errors);
    }
    let file = fs::File::create(out)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut seen = HashSet::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(path)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if !seen.insert(rel.clone()) {
            continue;
        }
        zip.start_file(rel, options)?;
        let bytes = fs::read(entry.path())?;
        zip.write_all(&bytes)?;
    }
    zip.finish()?;
    Ok(())
}

fn inspect_archive(path: &Path) -> Result<()> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    println!("Archive: {} ({} files)", path.display(), archive.len());
    for i in 0..archive.len() {
        let f = archive.by_index(i)?;
        println!(" - {} ({} bytes)", f.name(), f.size());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_and_validate_warns_placeholders() {
        let dir = tempdir().unwrap();
        pack_init("buddy", dir.path()).unwrap();
        let report = validate_pack(&dir.path().join("buddy")).unwrap();
        assert!(report.ok, "{:?}", report.errors);
        assert!(!report.warnings.is_empty());
    }

    #[test]
    fn validate_rejects_missing_variant_asset() {
        let dir = tempdir().unwrap();
        pack_init("buddy", dir.path()).unwrap();
        let pack = dir.path().join("buddy");
        let manifest_path = pack.join("familiar.json");
        let raw = fs::read_to_string(&manifest_path).unwrap();
        let mut manifest: FamiliarManifest = serde_json::from_str(&raw).unwrap();
        manifest
            .variants
            .insert("missing".into(), "assets/missing.png".into());
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let report = validate_pack(&pack).unwrap();
        assert!(!report.ok);
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("missing asset for variant missing")));
    }
}
