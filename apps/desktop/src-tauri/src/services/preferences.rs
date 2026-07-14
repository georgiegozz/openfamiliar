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
        fs::write(&self.path, encoded).map_err(|error| error.to_string())?;
        *self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = validated.clone();
        Ok(validated)
    }
}
