use crate::domain::AppPreferences;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct PreferencesStore {
    path: PathBuf,
    current: Mutex<AppPreferences>,
}

impl PreferencesStore {
    pub fn load(path: PathBuf) -> Self {
        let preferences = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<AppPreferences>(&content).ok())
            .unwrap_or_default()
            .validate();
        Self {
            path,
            current: Mutex::new(preferences),
        }
    }

    pub fn get(&self) -> AppPreferences {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn replace(&self, preferences: AppPreferences) -> Result<AppPreferences, String> {
        let validated = preferences.validate();
        let encoded = serde_json::to_vec_pretty(&validated).map_err(|error| error.to_string())?;
        let mut current = self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&self.path, encoded).map_err(|error| error.to_string())?;
        *current = validated.clone();
        Ok(validated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn replace_creates_parent_and_roundtrips() {
        let root = tempdir().expect("temp directory");
        let path = root.path().join("nested").join("preferences.json");
        let store = PreferencesStore::load(path.clone());
        let preferences = AppPreferences {
            scale: 3,
            language: "en-US".to_string(),
            timeout_seconds: 210,
            ..AppPreferences::default()
        };

        let saved = store.replace(preferences).expect("save preferences");
        assert!(path.is_file());
        assert_eq!(saved.scale, 3);
        assert_eq!(PreferencesStore::load(path).get().timeout_seconds, 210);
    }

    #[test]
    fn malformed_file_falls_back_to_valid_defaults() {
        let root = tempdir().expect("temp directory");
        let path = root.path().join("preferences.json");
        fs::write(&path, b"not-json").expect("write malformed fixture");

        assert_eq!(PreferencesStore::load(path).get().scale, 2);
    }

    #[test]
    fn failed_write_does_not_mutate_current_preferences() {
        let root = tempdir().expect("temp directory");
        let blocking_file = root.path().join("not-a-directory");
        fs::write(&blocking_file, b"fixture").expect("write blocking file");
        let store = PreferencesStore::load(blocking_file.join("preferences.json"));
        let mut changed = store.get();
        changed.scale = 3;

        assert!(store.replace(changed).is_err());
        assert_eq!(store.get().scale, 2);
    }
}
