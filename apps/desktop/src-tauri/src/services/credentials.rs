use familiar_credentials::{KeyringSecretStore, SecretStore};

pub const SETTINGS_WINDOW_LABEL: &str = "settings";

pub fn production_store() -> KeyringSecretStore {
    KeyringSecretStore::new()
}

pub fn is_settings_window(label: &str) -> bool {
    label == SETTINGS_WINDOW_LABEL
}

pub use familiar_credentials::redact_secrets;
