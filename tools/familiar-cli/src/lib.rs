//! Pack tooling shared by the `familiar` CLI and the desktop app.

use anyhow::{bail, Context, Result};
use familiar_pixel::{
    import_image, Layout, PixelOptions, DEFAULT_COLORS, FRAME_SIZE, MAX_COLORS, MIN_COLORS,
    SHEET_COLUMNS, SHEET_ROWS,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub const MAX_PACK_BYTES: u64 = 50 * 1024 * 1024;
pub const SPRITESHEET_REL: &str = "assets/spritesheet.png";
pub const MANIFEST_NAME: &str = "familiar.json";
pub const PERSONALITY_NAME: &str = "personality.md";
pub const ALLOWED_EXTS: &[&str] = &[
    "webp", "png", "jpg", "jpeg", "gif", "md", "json", "txt", "wav", "ogg", "mp3",
];

const REQUIRED_STATES: &[&str] = &[
    "idle", "thinking", "working", "approval", "success", "error",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetDefinition {
    pub asset: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub columns: u32,
    pub rows: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnimationDefinition {
    pub frames: Vec<u32>,
    pub fps: f32,
    #[serde(rename = "loop")]
    pub loop_playback: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FamiliarManifest {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine: String,
    pub author: String,
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub personality: String,
    pub states: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite_sheet: Option<SpriteSheetDefinition>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub variants: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub animations: BTreeMap<String, AnimationDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asset_sources: Vec<String>,
    #[serde(default)]
    pub ai_generated: bool,
}

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub ok: bool,
    pub id: String,
    pub name: String,
    pub version: String,
    pub license: String,
    pub states: Vec<String>,
    pub variants: Vec<String>,
    pub file_hashes: Vec<(String, String)>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ImportLayout {
    ContactSheet,
    SingleFrame,
    TiledMascot,
}

impl ImportLayout {
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "sheet" | "contact-sheet" => Ok(Self::ContactSheet),
            "frame" => Ok(Self::SingleFrame),
            "tiled" | "mascot" => Ok(Self::TiledMascot),
            other => bail!("unknown layout '{other}' (use sheet, frame, or tiled)"),
        }
    }

    fn to_pixel(self) -> Layout {
        match self {
            Self::ContactSheet => Layout::ContactSheet,
            Self::SingleFrame => Layout::SingleFrame,
            Self::TiledMascot => Layout::TiledMascot,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportImageRequest<'a> {
    pub input: &'a Path,
    pub out: &'a Path,
    pub colors: u32,
    pub layout: ImportLayout,
    pub force: bool,
    pub name: Option<&'a str>,
    pub author: Option<&'a str>,
    pub license: Option<&'a str>,
}

pub fn slugify(name: &str) -> Result<String> {
    let mut out = String::new();
    let mut dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty()
        || out.len() > 64
        || !out
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("invalid pack id derived from '{name}'");
    }
    // must match [a-z0-9]+(-[a-z0-9]+)*
    if out.starts_with('-') || out.ends_with('-') || out.contains("--") {
        bail!("invalid pack id '{out}'");
    }
    Ok(out)
}

pub fn pack_init(name: &str, path: &Path) -> Result<()> {
    let dir = path.join(name);
    fs::create_dir_all(dir.join("assets"))?;
    fs::create_dir_all(dir.join("sounds"))?;
    fs::create_dir_all(dir.join("previews"))?;
    let id = slugify(name).unwrap_or_else(|_| "pack".into());
    let manifest = serde_json::json!({
        "$schema": "https://openfamiliar.dev/schema/familiar-v1.json",
        "id": id,
        "name": name,
        "version": "0.1.0",
        "engine": ">=0.1.0",
        "author": "unknown",
        "license": "CC0-1.0",
        "personality": PERSONALITY_NAME,
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
        dir.join(MANIFEST_NAME),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    fs::write(
        dir.join(PERSONALITY_NAME),
        format!("# {name}\n\nA friendly desktop familiar.\n"),
    )?;
    fs::write(
        dir.join("LICENSE"),
        "CC0-1.0 — dedicate this pack's assets to the public domain as appropriate.\n",
    )?;
    for state in REQUIRED_STATES {
        fs::write(
            dir.join("assets").join(format!("{state}.webp.txt")),
            format!("Replace with real {state}.webp asset\n"),
        )?;
    }
    println!("Initialized pack at {}", dir.display());
    Ok(())
}

pub fn validate_pack(path: &Path) -> Result<ValidationReport> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let manifest_path = path.join(MANIFEST_NAME);
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

    for s in REQUIRED_STATES {
        if !manifest.states.contains_key(*s) {
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
        if entry.path().extension().is_some()
            && !ALLOWED_EXTS.contains(&ext.as_str())
            && ext != "txt"
            && !rel.eq_ignore_ascii_case("license")
        {
            warnings.push(format!("unexpected extension .{ext} for {rel}"));
        }
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
        states: REQUIRED_STATES.iter().map(|s| s.to_string()).collect(),
        variants: manifest.variants.keys().cloned().collect(),
        file_hashes: hashes,
        warnings,
        errors,
    })
}

pub fn build_pack(path: &Path, out: &Path) -> Result<()> {
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

pub fn inspect_archive(path: &Path) -> Result<()> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    println!("Archive: {} ({} files)", path.display(), archive.len());
    for i in 0..archive.len() {
        let f = archive.by_index(i)?;
        println!(" - {} ({} bytes)", f.name(), f.size());
    }
    Ok(())
}

pub fn import_image_to_pack(req: ImportImageRequest<'_>) -> Result<ValidationReport> {
    if !(MIN_COLORS..=MAX_COLORS).contains(&req.colors) {
        bail!("--colors must be between {MIN_COLORS} and {MAX_COLORS}");
    }
    if !req.input.is_file() {
        bail!("input image not found: {}", req.input.display());
    }
    if req.out.exists() && !req.force {
        bail!(
            "output path already exists (pass --force to overwrite): {}",
            req.out.display()
        );
    }
    let bytes = fs::read(req.input).context("read input image")?;
    let imported = import_image(
        &bytes,
        PixelOptions {
            colors: req.colors,
            layout: req.layout.to_pixel(),
        },
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    if req.out.exists() {
        fs::remove_dir_all(req.out).or_else(|_| fs::remove_file(req.out))?;
    }
    fs::create_dir_all(req.out.join("assets"))?;
    fs::write(req.out.join(SPRITESHEET_REL), &imported.png)?;

    let fallback_name = req
        .out
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("familiar");
    let name = req.name.unwrap_or(fallback_name);
    let id = slugify(name)?;
    let author = req.author.unwrap_or("unknown");
    let license = req.license.unwrap_or("CC0-1.0");

    let mut states = BTreeMap::new();
    let mut animations = BTreeMap::new();
    let frame = 0u32;
    for state in REQUIRED_STATES {
        states.insert((*state).to_string(), SPRITESHEET_REL.to_string());
        animations.insert(
            (*state).to_string(),
            AnimationDefinition {
                frames: vec![frame],
                fps: 1.0,
                loop_playback: true,
            },
        );
    }
    // Single-frame layout is still stored as a 64x64 PNG named spritesheet.png
    // so the pack has one fixed asset path. Contact/tiled layouts are 256x256.
    let (frame_w, frame_h, columns, rows) = match req.layout {
        ImportLayout::SingleFrame => (imported.width, imported.height, 1, 1),
        _ => (FRAME_SIZE, FRAME_SIZE, SHEET_COLUMNS, SHEET_ROWS),
    };

    let manifest = FamiliarManifest {
        schema: Some("https://openfamiliar.dev/schema/familiar-v1.json".into()),
        id,
        name: name.to_string(),
        version: "0.1.0".into(),
        engine: ">=0.1.0".into(),
        author: author.to_string(),
        license: license.to_string(),
        homepage: None,
        personality: PERSONALITY_NAME.into(),
        states,
        sprite_sheet: Some(SpriteSheetDefinition {
            asset: SPRITESHEET_REL.into(),
            frame_width: frame_w,
            frame_height: frame_h,
            columns,
            rows,
        }),
        variants: BTreeMap::new(),
        animations,
        asset_sources: vec![format!(
            "Imported locally from {} and quantized to a {}x{} sprite sheet.",
            req.input
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("image"),
            imported.width,
            imported.height
        )],
        ai_generated: false,
    };

    fs::write(
        req.out.join(MANIFEST_NAME),
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    fs::write(
        req.out.join(PERSONALITY_NAME),
        format!("# {name}\n\nA friendly desktop familiar.\n"),
    )?;
    validate_pack(req.out)
}

pub fn default_import_colors() -> u32 {
    DEFAULT_COLORS
}

pub const BUNDLED_MASCOT_ID: &str = "perrito-tech";
pub const MAX_IMPORT_BYTES: usize = familiar_pixel::MAX_INPUT_BYTES;

pub fn is_valid_slug(id: &str) -> bool {
    if id.is_empty() || id.len() > 64 {
        return false;
    }
    let mut start = true;
    let mut prev_dash = false;
    for c in id.chars() {
        if start {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
                return false;
            }
            start = false;
            continue;
        }
        if c == '-' {
            if prev_dash {
                return false;
            }
            prev_dash = true;
            continue;
        }
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
            return false;
        }
        prev_dash = false;
    }
    !prev_dash && !start
}

/// Ensure `candidate` stays inside `root`.
///
/// The path is first checked lexically (absolute, `..` normalized, must stay
/// under `root`) so a rejected path never creates directories outside `root`.
/// Only after that check do we create missing directories and confirm the
/// final location with `canonicalize` (which also catches symlink escapes).
pub fn assert_path_inside(root: &Path, candidate: &Path) -> Result<PathBuf> {
    let root_lex = normalize_lexically(&absolute_path(root)?)?;
    let cand_lex = normalize_lexically(&absolute_path(candidate)?)?;
    if !cand_lex.starts_with(&root_lex) {
        bail!(
            "path escapes mascot store: {} is not inside {}",
            cand_lex.display(),
            root_lex.display()
        );
    }

    fs::create_dir_all(&root_lex)?;
    if !cand_lex.exists() {
        if let Some(parent) = cand_lex.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
    }

    let root_canon = fs::canonicalize(&root_lex)?;
    let cand_canon = if cand_lex.exists() {
        fs::canonicalize(&cand_lex)?
    } else {
        let parent = cand_lex
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid path"))?;
        let parent_canon = fs::canonicalize(parent)?;
        parent_canon.join(
            cand_lex
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("invalid path"))?,
        )
    };
    if !cand_canon.starts_with(&root_canon) {
        bail!(
            "path escapes mascot store: {} is not inside {}",
            cand_canon.display(),
            root_canon.display()
        );
    }
    Ok(cand_canon)
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

/// Resolve `.` and `..` without touching the filesystem or following symlinks.
fn normalize_lexically(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        bail!("path must be absolute");
    }
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => out.push(component),
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                _ => {
                    bail!(
                        "path escapes mascot store: {} contains '..'",
                        path.display()
                    );
                }
            },
            Component::Normal(_) => out.push(component),
        }
    }
    Ok(out)
}

fn unique_slug(root: &Path, desired: &str) -> Result<String> {
    let base = slugify(desired)?;
    if !is_valid_slug(&base) {
        bail!("invalid mascot id '{base}'");
    }
    if base != BUNDLED_MASCOT_ID && !root.join(&base).exists() {
        return Ok(base);
    }
    for i in 2..1000 {
        let candidate = format!("{base}-{i}");
        if !root.join(&candidate).exists() {
            return Ok(candidate);
        }
    }
    bail!("could not allocate a unique mascot id")
}

#[derive(Debug, Serialize, Clone)]
pub struct LocalMascotInfo {
    pub id: String,
    pub name: String,
}

pub fn list_local_mascots(root: &Path) -> Result<Vec<LocalMascotInfo>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let root_canon = fs::canonicalize(root)?;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let canon = match fs::canonicalize(&path) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !canon.starts_with(&root_canon) {
            continue;
        }
        match validate_pack(&path) {
            Ok(report)
                if report.ok
                    && is_valid_slug(&report.id)
                    && path.file_name().and_then(|s| s.to_str()) == Some(report.id.as_str()) =>
            {
                out.push(LocalMascotInfo {
                    id: report.id,
                    name: report.name,
                });
            }
            _ => continue,
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

pub fn import_bytes_as_local_mascot(
    mascots_root: &Path,
    suggested_name: &str,
    bytes: &[u8],
    colors: u32,
) -> Result<LocalMascotInfo> {
    if bytes.len() > MAX_IMPORT_BYTES {
        bail!("image exceeds the {MAX_IMPORT_BYTES} byte limit");
    }
    if bytes.is_empty() {
        bail!("image is empty");
    }
    fs::create_dir_all(mascots_root)?;
    let _root = fs::canonicalize(mascots_root)?;
    let slug = unique_slug(mascots_root, suggested_name)?;
    let dest = mascots_root.join(&slug);
    let dest = assert_path_inside(mascots_root, &dest)?;
    // write to a temp sibling then... we use dest directly after path check
    let tmp_input = mascots_root.join(format!(".{slug}.incoming.png"));
    let tmp_input = assert_path_inside(mascots_root, &tmp_input)?;
    fs::write(&tmp_input, bytes)?;
    let result = import_image_to_pack(ImportImageRequest {
        input: &tmp_input,
        out: &dest,
        colors,
        layout: ImportLayout::TiledMascot,
        force: false,
        name: Some(suggested_name),
        author: Some("local-user"),
        license: Some("NOASSERTION"),
    });
    let _ = fs::remove_file(&tmp_input);
    let report = result?;
    if !report.ok {
        let _ = fs::remove_dir_all(&dest);
        bail!("imported pack failed validation: {:?}", report.errors);
    }
    // Keep the unique directory name as the pack id so collisions never overwrite
    // a sibling pack. The display name stays as given by the operator.
    if report.id != slug {
        let manifest_path = dest.join(MANIFEST_NAME);
        let raw = fs::read_to_string(&manifest_path)?;
        let mut manifest: FamiliarManifest = serde_json::from_str(&raw)?;
        manifest.id = slug.clone();
        manifest.name = suggested_name.to_string();
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest)? + "\n",
        )?;
        let report = validate_pack(&dest)?;
        if !report.ok {
            let _ = fs::remove_dir_all(&dest);
            bail!(
                "imported pack failed validation after id fix: {:?}",
                report.errors
            );
        }
        return Ok(LocalMascotInfo {
            id: slug,
            name: suggested_name.to_string(),
        });
    }
    Ok(LocalMascotInfo {
        id: report.id,
        name: report.name,
    })
}

pub fn load_validated_pack(mascots_root: &Path, id: &str) -> Result<(PathBuf, FamiliarManifest)> {
    if !is_valid_slug(id) || id == BUNDLED_MASCOT_ID {
        bail!("invalid or reserved mascot id");
    }
    let dest = assert_path_inside(mascots_root, &mascots_root.join(id))?;
    if !dest.is_dir() {
        bail!("mascot not found: {id}");
    }
    let report = validate_pack(&dest)?;
    if !report.ok {
        bail!("local pack is invalid: {:?}", report.errors);
    }
    if report.id != id {
        bail!("pack id does not match directory name");
    }
    let raw = fs::read_to_string(dest.join(MANIFEST_NAME))?;
    let manifest: FamiliarManifest = serde_json::from_str(&raw)?;
    Ok((dest, manifest))
}

pub fn read_validated_sheet(mascots_root: &Path, id: &str) -> Result<Vec<u8>> {
    let (dest, manifest) = load_validated_pack(mascots_root, id)?;
    let rel = manifest
        .sprite_sheet
        .as_ref()
        .map(|s| s.asset.as_str())
        .or_else(|| manifest.states.get("idle").map(|s| s.as_str()))
        .unwrap_or(SPRITESHEET_REL);
    if rel.contains("..") || Path::new(rel).is_absolute() {
        bail!("invalid sheet path");
    }
    let sheet = assert_path_inside(&dest, &dest.join(rel))?;
    Ok(fs::read(sheet)?)
}

pub const EDITOR_SHEET_SIZE: u32 = 256;
pub const MIN_EDITOR_FPS: f32 = 1.0;
pub const MAX_EDITOR_FPS: f32 = 12.0;
pub const MAX_PERSONALITY_CHARS: usize = 8_192;
const ALLOWED_LICENSES: &[&str] = &["CC0-1.0", "CC-BY-4.0", "Apache-2.0", "NOASSERTION"];

#[derive(Debug, Clone)]
pub struct EditorSaveRequest<'a> {
    pub existing_id: Option<&'a str>,
    pub name: &'a str,
    pub license: &'a str,
    pub personality: &'a str,
    pub png: &'a [u8],
    pub fps: f32,
    pub animations: BTreeMap<String, AnimationDefinition>,
}

pub fn save_editor_mascot(
    mascots_root: &Path,
    request: EditorSaveRequest<'_>,
) -> Result<LocalMascotInfo> {
    if request.png.len() > MAX_IMPORT_BYTES {
        bail!("image exceeds the {MAX_IMPORT_BYTES} byte limit");
    }
    if request.png.is_empty() {
        bail!("image is empty");
    }
    let (width, height) = familiar_pixel::peek_dimensions(request.png)
        .map_err(|error| anyhow::anyhow!("invalid sheet: {error}"))?;
    if width != EDITOR_SHEET_SIZE || height != EDITOR_SHEET_SIZE {
        bail!("editor sheet must be {EDITOR_SHEET_SIZE}x{EDITOR_SHEET_SIZE}");
    }
    let name = request.name.trim();
    if name.is_empty() || name.len() > 80 {
        bail!("mascot name is required");
    }
    if !ALLOWED_LICENSES.contains(&request.license) {
        bail!("license must be one of: {}", ALLOWED_LICENSES.join(", "));
    }
    if request.personality.chars().count() > MAX_PERSONALITY_CHARS
        || request.personality.contains('\0')
    {
        bail!("personality is too large or contains a NUL byte");
    }
    if !(MIN_EDITOR_FPS..=MAX_EDITOR_FPS).contains(&request.fps) {
        bail!("fps must be between {MIN_EDITOR_FPS} and {MAX_EDITOR_FPS}");
    }
    if request.animations.is_empty() {
        bail!("at least one animation is required");
    }
    let max_frame = SHEET_COLUMNS * SHEET_ROWS - 1;
    for (state, animation) in &request.animations {
        if !REQUIRED_STATES.contains(&state.as_str())
            && !matches!(
                state.as_str(),
                "listening" | "answering" | "sleeping" | "wake" | "dragging"
            )
        {
            bail!("unknown animation state '{state}'");
        }
        if animation.frames.is_empty() || animation.frames.iter().any(|frame| *frame > max_frame) {
            bail!("animation '{state}' has an out-of-range frame");
        }
        if animation.fps < MIN_EDITOR_FPS || animation.fps > MAX_EDITOR_FPS {
            bail!("animation '{state}' fps is out of range");
        }
    }
    for required in REQUIRED_STATES {
        if !request.animations.contains_key(*required) {
            bail!("missing required animation '{required}'");
        }
    }

    fs::create_dir_all(mascots_root)?;
    let slug = if let Some(existing) = request.existing_id {
        if !is_valid_slug(existing) || existing == BUNDLED_MASCOT_ID {
            bail!("invalid or reserved mascot id");
        }
        existing.to_string()
    } else {
        unique_slug(mascots_root, name)?
    };
    let dest = assert_path_inside(mascots_root, &mascots_root.join(&slug))?;
    if request.existing_id.is_some() && !dest.is_dir() {
        bail!("mascot not found: {slug}");
    }
    if request.existing_id.is_none() && dest.exists() {
        bail!("mascot directory already exists: {slug}");
    }
    fs::create_dir_all(dest.join("assets"))?;
    fs::write(dest.join(SPRITESHEET_REL), request.png)?;

    let mut states = BTreeMap::new();
    for state in REQUIRED_STATES {
        states.insert((*state).to_string(), SPRITESHEET_REL.to_string());
    }
    let manifest = FamiliarManifest {
        schema: Some("https://openfamiliar.dev/schema/familiar-v1.json".into()),
        id: slug.clone(),
        name: name.to_string(),
        version: "0.1.0".into(),
        engine: ">=0.1.0".into(),
        author: "local-user".into(),
        license: request.license.to_string(),
        homepage: None,
        personality: PERSONALITY_NAME.into(),
        states,
        sprite_sheet: Some(SpriteSheetDefinition {
            asset: SPRITESHEET_REL.into(),
            frame_width: FRAME_SIZE,
            frame_height: FRAME_SIZE,
            columns: SHEET_COLUMNS,
            rows: SHEET_ROWS,
        }),
        variants: BTreeMap::new(),
        animations: request.animations,
        asset_sources: vec!["Created locally with the OpenFamiliar avatar editor.".into()],
        ai_generated: false,
    };
    fs::write(
        dest.join(MANIFEST_NAME),
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    fs::write(
        dest.join(PERSONALITY_NAME),
        request.personality.trim().to_string() + "\n",
    )?;
    let report = validate_pack(&dest)?;
    if !report.ok {
        bail!("saved pack failed validation: {:?}", report.errors);
    }
    Ok(LocalMascotInfo {
        id: report.id,
        name: report.name,
    })
}

pub fn read_validated_personality(mascots_root: &Path, id: &str) -> Result<String> {
    let (dest, manifest) = load_validated_pack(mascots_root, id)?;
    let rel = manifest.personality;
    if rel.contains("..") || Path::new(&rel).is_absolute() {
        bail!("invalid personality path");
    }
    let path = assert_path_inside(&dest, &dest.join(rel))?;
    let text = fs::read_to_string(path)?;
    if text.chars().count() > MAX_PERSONALITY_CHARS {
        bail!("personality is too large");
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use tempfile::tempdir;

    fn tiny_png() -> Vec<u8> {
        let mut img = RgbaImage::new(16, 16);
        for px in img.pixels_mut() {
            *px = Rgba([40, 80, 120, 255]);
        }
        let mut out = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut out);
        img.write_with_encoder(encoder).unwrap();
        out
    }

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
        let manifest_path = pack.join(MANIFEST_NAME);
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

    #[test]
    fn import_image_writes_fixed_names_and_validates() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("foto.png");
        fs::write(&input, tiny_png()).unwrap();
        let out = dir.path().join("my-buddy");
        let report = import_image_to_pack(ImportImageRequest {
            input: &input,
            out: &out,
            colors: 16,
            layout: ImportLayout::TiledMascot,
            force: false,
            name: Some("My Buddy"),
            author: Some("tester"),
            license: Some("CC0-1.0"),
        })
        .unwrap();
        assert!(report.ok, "{:?}", report.errors);
        assert!(out.join(SPRITESHEET_REL).is_file());
        assert!(out.join(MANIFEST_NAME).is_file());
        assert!(out.join(PERSONALITY_NAME).is_file());
        assert_eq!(report.id, "my-buddy");
        // no other image names
        let assets: Vec<_> = fs::read_dir(out.join("assets"))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(assets, vec!["spritesheet.png"]);
    }

    #[test]
    fn import_refuses_to_overwrite_without_force() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("foto.png");
        fs::write(&input, tiny_png()).unwrap();
        let out = dir.path().join("pack");
        fs::create_dir_all(&out).unwrap();
        let err = import_image_to_pack(ImportImageRequest {
            input: &input,
            out: &out,
            colors: 16,
            layout: ImportLayout::TiledMascot,
            force: false,
            name: None,
            author: None,
            license: None,
        })
        .unwrap_err();
        assert!(err.to_string().contains("--force"), "{err}");
    }

    #[test]
    fn import_rejects_invalid_image() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("foto.png");
        fs::write(&input, b"not a png").unwrap();
        let out = dir.path().join("pack");
        let err = import_image_to_pack(ImportImageRequest {
            input: &input,
            out: &out,
            colors: 16,
            layout: ImportLayout::ContactSheet,
            force: false,
            name: None,
            author: None,
            license: None,
        })
        .unwrap_err();
        assert!(!out.exists());
        assert!(
            err.to_string().to_lowercase().contains("image") || err.to_string().contains("invalid"),
            "{err}"
        );
    }

    #[test]
    fn slug_validation() {
        assert!(is_valid_slug("my-buddy"));
        assert!(is_valid_slug("a1"));
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("My-Buddy"));
        assert!(!is_valid_slug("../etc"));
        assert!(!is_valid_slug("foo--bar"));
        assert!(!is_valid_slug("-foo"));
        assert!(!is_valid_slug("foo-"));
    }

    #[test]
    fn import_bytes_rejects_oversized_payload_before_write() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("mascots");
        let bytes = vec![0u8; MAX_IMPORT_BYTES + 1];
        let err = import_bytes_as_local_mascot(&root, "buddy", &bytes, 16).unwrap_err();
        assert!(err.to_string().contains("byte limit"));
        assert!(!root.join("buddy").exists());
    }

    #[test]
    fn import_bytes_stays_inside_root_and_validates() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("mascots");
        let info = import_bytes_as_local_mascot(&root, "Buddy Pal", &tiny_png(), 16).unwrap();
        assert!(is_valid_slug(&info.id));
        let listed = list_local_mascots(&root).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, info.id);
        let png = read_validated_sheet(&root, &info.id).unwrap();
        assert!(png.starts_with(b"\x89PNG"));
        let escaped = import_bytes_as_local_mascot(&root, "..", &tiny_png(), 16);
        assert!(escaped.is_err());
    }

    #[test]
    fn path_escape_is_rejected() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("mascots");
        fs::create_dir_all(&root).unwrap();
        let outside = dir.path().join("outside");
        let err = assert_path_inside(&root, &outside).unwrap_err();
        assert!(err.to_string().contains("escapes"));
    }

    #[test]
    fn path_escape_does_not_create_directories_outside_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("mascots");
        fs::create_dir_all(&root).unwrap();

        let outside_dir = dir.path().join("outside-created");
        let escaped = root.join("..").join("outside-created").join("pack");
        assert!(!outside_dir.exists());
        let err = assert_path_inside(&root, &escaped).unwrap_err();
        assert!(err.to_string().contains("escapes"), "{err}");
        assert!(
            !outside_dir.exists(),
            "a rejected path must not create directories outside root"
        );

        let deep_root = dir.path().join("deep-outside");
        let deep = root
            .join("..")
            .join("deep-outside")
            .join("a")
            .join("b")
            .join("pack");
        let err = assert_path_inside(&root, &deep).unwrap_err();
        assert!(err.to_string().contains("escapes"), "{err}");
        assert!(
            !deep_root.exists(),
            "a rejected path must not create directories outside root"
        );
    }

    fn blank_sheet_png() -> Vec<u8> {
        let img = RgbaImage::from_pixel(256, 256, Rgba([0, 0, 0, 0]));
        let mut out = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut out);
        img.write_with_encoder(encoder).unwrap();
        out
    }

    fn default_editor_anims() -> BTreeMap<String, AnimationDefinition> {
        let mut animations = BTreeMap::new();
        for state in REQUIRED_STATES {
            animations.insert(
                (*state).to_string(),
                AnimationDefinition {
                    frames: vec![0],
                    fps: 4.0,
                    loop_playback: true,
                },
            );
        }
        animations
    }

    #[test]
    fn editor_save_rejects_wrong_dimensions_and_bad_fps() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("mascots");
        let err = save_editor_mascot(
            &root,
            EditorSaveRequest {
                existing_id: None,
                name: "Painter",
                license: "CC0-1.0",
                personality: "A painted buddy.",
                png: &tiny_png(),
                fps: 4.0,
                animations: default_editor_anims(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("256"));

        let mut animations = default_editor_anims();
        animations.get_mut("idle").unwrap().fps = 99.0;
        let err = save_editor_mascot(
            &root,
            EditorSaveRequest {
                existing_id: None,
                name: "Painter",
                license: "CC0-1.0",
                personality: "A painted buddy.",
                png: &blank_sheet_png(),
                fps: 4.0,
                animations,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("fps"));
    }

    #[test]
    fn editor_save_writes_validated_pack() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("mascots");
        let info = save_editor_mascot(
            &root,
            EditorSaveRequest {
                existing_id: None,
                name: "Painter Pal",
                license: "CC-BY-4.0",
                personality: "A quiet painted familiar.",
                png: &blank_sheet_png(),
                fps: 4.0,
                animations: default_editor_anims(),
            },
        )
        .unwrap();
        assert_eq!(info.id, "painter-pal");
        let listed = list_local_mascots(&root).unwrap();
        assert_eq!(listed.len(), 1);
        let png = read_validated_sheet(&root, &info.id).unwrap();
        assert!(png.starts_with(b"\x89PNG"));
        let personality = fs::read_to_string(root.join(info.id).join(PERSONALITY_NAME)).unwrap();
        assert!(personality.contains("quiet painted"));
    }
}
