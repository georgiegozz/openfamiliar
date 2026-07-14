//! Workspace context: explicit roots, ignore rules, sensitive blocklist, preview.

use ignore::WalkBuilder;
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const DEFAULT_BLOCKLIST: &[&str] = &[
    ".env",
    "*.pem",
    "*.key",
    "*.pfx",
    "*.p12",
    "id_rsa*",
    "credentials*",
    "secrets*",
    "terraform.tfstate*",
    ".azure",
    ".aws",
    ".ssh",
];

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("path escapes workspace: {0}")]
    PathEscape(String),
    #[error("path blocked as sensitive: {0}")]
    Sensitive(String),
    #[error("workspace not authorized")]
    NotAuthorized,
    #[error("binary or unsupported file")]
    UnsupportedFile,
}

pub type Result<T> = std::result::Result<T, ContextError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRoot {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub bytes: usize,
    pub estimated_tokens: usize,
    pub content: Option<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreview {
    pub workspace_id: String,
    pub files: Vec<ContextFile>,
    pub total_bytes: usize,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub max_files: usize,
    pub max_file_bytes: usize,
    pub max_total_bytes: usize,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_files: 20,
            max_file_bytes: 32 * 1024,
            max_total_bytes: 200 * 1024,
        }
    }
}

pub struct WorkspaceContext {
    roots: Vec<WorkspaceRoot>,
    budget: ContextBudget,
}

impl WorkspaceContext {
    pub fn new(budget: ContextBudget) -> Self {
        Self {
            roots: Vec::new(),
            budget,
        }
    }

    pub fn authorize_root(
        &mut self,
        id: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<()> {
        let path = path.into().clean();
        if !path.exists() {
            return Err(ContextError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "workspace path not found",
            )));
        }
        let meta = fs::metadata(&path)?;
        if !meta.is_dir() {
            return Err(ContextError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "workspace must be a directory",
            )));
        }
        let id = id.into();
        self.roots.retain(|r| r.id != id);
        self.roots.push(WorkspaceRoot { id, path });
        Ok(())
    }

    pub fn roots(&self) -> &[WorkspaceRoot] {
        &self.roots
    }

    pub fn resolve_under(&self, workspace_id: &str, relative: &Path) -> Result<PathBuf> {
        let root = self
            .roots
            .iter()
            .find(|r| r.id == workspace_id)
            .ok_or(ContextError::NotAuthorized)?;
        let joined = root.path.join(relative).clean();
        if !joined.starts_with(&root.path) {
            return Err(ContextError::PathEscape(joined.display().to_string()));
        }
        // symlink resolve
        let canonical = fs::canonicalize(&joined).unwrap_or(joined.clone());
        let root_canon = fs::canonicalize(&root.path).unwrap_or(root.path.clone());
        if !canonical.starts_with(&root_canon) {
            return Err(ContextError::PathEscape(canonical.display().to_string()));
        }
        if is_sensitive(&canonical) {
            return Err(ContextError::Sensitive(canonical.display().to_string()));
        }
        Ok(canonical)
    }

    pub fn summarize_tree(&self, workspace_id: &str, max_entries: usize) -> Result<Vec<String>> {
        let root = self
            .roots
            .iter()
            .find(|r| r.id == workspace_id)
            .ok_or(ContextError::NotAuthorized)?;
        let mut out = Vec::new();
        let walker = WalkBuilder::new(&root.path)
            .hidden(false)
            .git_ignore(true)
            .add_custom_ignore_filename(".openfamiliarignore")
            .build();
        for entry in walker.flatten() {
            let path = entry.path();
            if path == root.path {
                continue;
            }
            if is_sensitive(path) {
                continue;
            }
            if let Ok(rel) = path.strip_prefix(&root.path) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
            if out.len() >= max_entries {
                break;
            }
        }
        Ok(out)
    }

    pub fn read_files(
        &self,
        workspace_id: &str,
        relative_paths: &[PathBuf],
        include_content: bool,
    ) -> Result<ContextPreview> {
        let mut files = Vec::new();
        let mut total_bytes = 0usize;
        for rel in relative_paths.iter().take(self.budget.max_files) {
            let path = self.resolve_under(workspace_id, rel)?;
            let data = fs::read(&path)?;
            if looks_binary(&data) {
                continue;
            }
            let mut truncated = false;
            let slice = if data.len() > self.budget.max_file_bytes {
                truncated = true;
                &data[..self.budget.max_file_bytes]
            } else {
                &data[..]
            };
            if total_bytes + slice.len() > self.budget.max_total_bytes {
                break;
            }
            total_bytes += slice.len();
            let content = if include_content {
                Some(String::from_utf8_lossy(slice).to_string())
            } else {
                None
            };
            let estimated_tokens = estimate_tokens(slice.len());
            files.push(ContextFile {
                path: rel.to_string_lossy().replace('\\', "/"),
                bytes: slice.len(),
                estimated_tokens,
                content,
                truncated,
            });
        }
        let estimated_tokens = files.iter().map(|f| f.estimated_tokens).sum();
        Ok(ContextPreview {
            workspace_id: workspace_id.to_string(),
            files,
            total_bytes,
            estimated_tokens,
        })
    }

    pub fn git_status_summary(&self, workspace_id: &str) -> Result<String> {
        let root = self
            .roots
            .iter()
            .find(|r| r.id == workspace_id)
            .ok_or(ContextError::NotAuthorized)?;
        let output = std::process::Command::new("git")
            .args(["status", "--short"])
            .current_dir(&root.path)
            .output();
        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Ok(o) => Ok(format!(
                "git status failed: {}",
                String::from_utf8_lossy(&o.stderr)
            )),
            Err(e) => Ok(format!("git unavailable: {e}")),
        }
    }
}

pub fn estimate_tokens(bytes: usize) -> usize {
    // rough 4 chars/token heuristic
    (bytes / 4).max(1)
}

fn looks_binary(data: &[u8]) -> bool {
    data.iter().take(8000).any(|&b| b == 0)
}

fn is_sensitive(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let lower = name.to_lowercase();
    for pat in DEFAULT_BLOCKLIST {
        if pat_match(pat, &lower) || pat_match(pat, &name) {
            return true;
        }
        // directory segment match
        for comp in path.components() {
            let c = comp.as_os_str().to_string_lossy();
            if pat_match(pat, &c) {
                return true;
            }
        }
    }
    false
}

fn pat_match(pattern: &str, value: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix)
            || value.to_lowercase().starts_with(&prefix.to_lowercase());
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return value.ends_with(suffix) || value.to_lowercase().ends_with(&suffix.to_lowercase());
    }
    value.eq_ignore_ascii_case(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn blocks_env_and_escape() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("ok.txt"), "hello").unwrap();
        fs::write(dir.path().join(".env"), "SECRET=1").unwrap();
        let mut ctx = WorkspaceContext::new(ContextBudget::default());
        ctx.authorize_root("w1", dir.path()).unwrap();
        assert!(ctx.resolve_under("w1", Path::new("ok.txt")).is_ok());
        assert!(matches!(
            ctx.resolve_under("w1", Path::new(".env")),
            Err(ContextError::Sensitive(_))
        ));
        assert!(matches!(
            ctx.resolve_under("w1", Path::new("../outside")),
            Err(ContextError::PathEscape(_))
        ));
    }

    #[test]
    fn preview_reads_selected_only() {
        let dir = tempdir().unwrap();
        let mut f = fs::File::create(dir.path().join("a.rs")).unwrap();
        writeln!(f, "fn main() {{}}").unwrap();
        let mut ctx = WorkspaceContext::new(ContextBudget::default());
        ctx.authorize_root("demo", dir.path()).unwrap();
        let preview = ctx
            .read_files("demo", &[PathBuf::from("a.rs")], true)
            .unwrap();
        assert_eq!(preview.files.len(), 1);
        assert!(preview.files[0].content.as_ref().unwrap().contains("main"));
    }

    #[test]
    fn authorize_nonexistent_path_fails() {
        let mut ctx = WorkspaceContext::new(ContextBudget::default());
        let result = ctx.authorize_root("bad", PathBuf::from("C:/nonexistent_path_xyz_12345"));
        assert!(result.is_err());
    }

    #[test]
    fn authorize_file_not_dir_fails() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("not_a_dir.txt");
        fs::write(&file_path, "content").unwrap();
        let mut ctx = WorkspaceContext::new(ContextBudget::default());
        let result = ctx.authorize_root("file", &file_path);
        assert!(result.is_err());
    }

    #[test]
    fn estimate_tokens_heuristic() {
        // 4 chars per token, minimum 1
        assert_eq!(estimate_tokens(0), 1);
        assert_eq!(estimate_tokens(4), 1);
        assert_eq!(estimate_tokens(8), 2);
        assert_eq!(estimate_tokens(100), 25);
        assert_eq!(estimate_tokens(1000), 250);
    }

    #[test]
    fn binary_file_skipped_in_preview() {
        let dir = tempdir().unwrap();
        // Create a file with null bytes (binary)
        let binary_path = dir.path().join("image.bin");
        fs::write(&binary_path, b"PNG\x00\x00\x00binary content").unwrap();
        // Create a normal text file
        let text_path = dir.path().join("readme.txt");
        fs::write(&text_path, "hello").unwrap();
        let mut ctx = WorkspaceContext::new(ContextBudget::default());
        ctx.authorize_root("ws", dir.path()).unwrap();
        let preview = ctx
            .read_files(
                "ws",
                &[PathBuf::from("image.bin"), PathBuf::from("readme.txt")],
                true,
            )
            .unwrap();
        // Binary file should be skipped, only text file included
        assert_eq!(preview.files.len(), 1);
        assert_eq!(preview.files[0].path, "readme.txt");
    }
}
