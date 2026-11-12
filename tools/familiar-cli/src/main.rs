//! `familiar` CLI — pack tooling for OpenFamiliar.

use anyhow::Result;
use clap::{Parser, Subcommand};
use familiar_cli::{
    build_pack, default_import_colors, import_image_to_pack, inspect_archive, pack_init,
    validate_pack, ImportImageRequest, ImportLayout,
};
use std::path::PathBuf;

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
    /// Import a local image into a pack directory with a fixed spritesheet path
    ImportImage {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = default_import_colors())]
        colors: u32,
        /// sheet (256x256 contact), frame (64x64), or tiled (64x64 repeated 4x4)
        #[arg(long, default_value = "tiled")]
        layout: String,
        /// Overwrite an existing --out path
        #[arg(long)]
        force: bool,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        license: Option<String>,
    },
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
            PackCommands::ImportImage {
                input,
                out,
                colors,
                layout,
                force,
                name,
                author,
                license,
            } => {
                let layout = ImportLayout::parse(&layout)?;
                let report = import_image_to_pack(ImportImageRequest {
                    input: &input,
                    out: &out,
                    colors,
                    layout,
                    force,
                    name: name.as_deref(),
                    author: author.as_deref(),
                    license: license.as_deref(),
                })?;
                println!("{}", serde_json::to_string_pretty(&report)?);
                if !report.ok {
                    std::process::exit(1);
                }
            }
        },
    }
    Ok(())
}
