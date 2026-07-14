use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_path: PathBuf,
    pub log_dir: PathBuf,
    pub neutral_work_dir: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self, String> {
        let dirs = ProjectDirs::from("dev", "OpenFamiliar", "OpenFamiliar")
            .ok_or_else(|| "Windows application directories are unavailable".to_string())?;
        let data_dir = dirs.data_local_dir().to_path_buf();
        let cache_dir = dirs.cache_dir().to_path_buf();
        Ok(Self {
            config_path: data_dir.join("preferences.json"),
            log_dir: data_dir.join("logs"),
            neutral_work_dir: cache_dir.join("codex-empty-workdir"),
        })
    }

    pub fn ensure(&self) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::create_dir_all(&self.log_dir).map_err(|error| error.to_string())?;
        fs::create_dir_all(&self.neutral_work_dir).map_err(|error| error.to_string())?;
        Ok(())
    }
}
