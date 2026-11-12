//! Credential storage for the stable desktop. The only production backend
//! is the OS keychain (Windows Credential Manager via `keyring`). Tests use
//! an in-memory mock. Secrets never go in app-data files.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const KEYCHAIN_SERVICE: &str = "dev.openfamiliar.desktop";
pub const MAX_SECRET_CHARS: usize = 8_192;
pub const ALLOWED_PROVIDER_IDS: &[&str] = &["codex", "anthropic", "opencode", "agy"];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CredentialError {
    #[error("unknown provider")]
    UnknownProvider,
    #[error("secret is empty or too large")]
    InvalidSecret,
    #[error("credential backend error")]
    Backend,
}

pub fn is_allowed_provider(id: &str) -> bool {
    ALLOWED_PROVIDER_IDS.contains(&id)
}

pub fn validate_secret(secret: &str) -> Result<(), CredentialError> {
    let trimmed = secret.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_SECRET_CHARS || trimmed.contains('\0') {
        return Err(CredentialError::InvalidSecret);
    }
    Ok(())
}

pub trait SecretStore: Send + Sync {
    fn set_secret(&self, provider_id: &str, secret: &str) -> Result<(), CredentialError>;
    fn get_secret(&self, provider_id: &str) -> Result<Option<String>, CredentialError>;
    fn delete_secret(&self, provider_id: &str) -> Result<(), CredentialError>;
    fn is_configured(&self, provider_id: &str) -> Result<bool, CredentialError> {
        Ok(self.get_secret(provider_id)?.is_some())
    }
}

/// In-memory store for unit tests and CI. Never used in the production desktop path.
#[derive(Clone, Default)]
pub struct MemorySecretStore {
    inner: Arc<Mutex<HashMap<String, String>>>,
}

impl MemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemorySecretStore {
    fn set_secret(&self, provider_id: &str, secret: &str) -> Result<(), CredentialError> {
        if !is_allowed_provider(provider_id) {
            return Err(CredentialError::UnknownProvider);
        }
        validate_secret(secret)?;
        self.inner
            .lock()
            .map_err(|_| CredentialError::Backend)?
            .insert(provider_id.to_string(), secret.trim().to_string());
        Ok(())
    }

    fn get_secret(&self, provider_id: &str) -> Result<Option<String>, CredentialError> {
        if !is_allowed_provider(provider_id) {
            return Err(CredentialError::UnknownProvider);
        }
        let map = self.inner.lock().map_err(|_| CredentialError::Backend)?;
        Ok(map.get(provider_id).cloned())
    }

    fn delete_secret(&self, provider_id: &str) -> Result<(), CredentialError> {
        if !is_allowed_provider(provider_id) {
            return Err(CredentialError::UnknownProvider);
        }
        self.inner
            .lock()
            .map_err(|_| CredentialError::Backend)?
            .remove(provider_id);
        Ok(())
    }
}

/// OS keychain backend. The service name is unique to this desktop app.
pub struct KeyringSecretStore {
    service: String,
}

impl KeyringSecretStore {
    pub fn new() -> Self {
        Self {
            service: KEYCHAIN_SERVICE.to_string(),
        }
    }

    fn entry(service: &str, provider_id: &str) -> Result<keyring::Entry, CredentialError> {
        if !is_allowed_provider(provider_id) {
            return Err(CredentialError::UnknownProvider);
        }
        keyring::Entry::new(service, provider_id).map_err(|_| CredentialError::Backend)
    }
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for KeyringSecretStore {
    fn set_secret(&self, provider_id: &str, secret: &str) -> Result<(), CredentialError> {
        validate_secret(secret)?;
        let entry = Self::entry(&self.service, provider_id)?;
        entry
            .set_password(secret.trim())
            .map_err(|_| CredentialError::Backend)
    }

    fn get_secret(&self, provider_id: &str) -> Result<Option<String>, CredentialError> {
        let entry = Self::entry(&self.service, provider_id)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(CredentialError::Backend),
        }
    }

    fn delete_secret(&self, provider_id: &str) -> Result<(), CredentialError> {
        let entry = Self::entry(&self.service, provider_id)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(CredentialError::Backend),
        }
    }
}

/// Redact common secret patterns so invoke arguments and keys never appear in logs.
pub fn redact_secrets(input: &str) -> String {
    let patterns = [
        (
            r"(?i)(api[_-]?key|token|password|authorization|secret|x-api-key)\s*[:=]\s*\S+",
            "$1=[REDACTED]",
        ),
        (r"sk-(ant-)?[A-Za-z0-9_-]{8,}", "sk-[REDACTED]"),
        (r"AIza[0-9A-Za-z\-_]{10,}", "AIza[REDACTED]"),
    ];
    let mut out = input.to_string();
    for (pat, rep) in patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            out = re.replace_all(&out, rep).to_string();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_set_get_delete_and_configured() {
        let store = MemorySecretStore::new();
        assert!(!store.is_configured("anthropic").unwrap());
        store
            .set_secret("anthropic", "  test-fixture-not-a-real-key  ")
            .unwrap();
        assert!(store.is_configured("anthropic").unwrap());
        let value = store.get_secret("anthropic").unwrap().unwrap();
        assert_eq!(value, "test-fixture-not-a-real-key");
        store.delete_secret("anthropic").unwrap();
        assert!(!store.is_configured("anthropic").unwrap());
        assert!(store.get_secret("anthropic").unwrap().is_none());
    }

    #[test]
    fn rejects_unknown_provider_and_invalid_secrets() {
        let store = MemorySecretStore::new();
        assert_eq!(
            store.set_secret("openai", "abc").unwrap_err(),
            CredentialError::UnknownProvider
        );
        assert_eq!(
            store.set_secret("anthropic", "   ").unwrap_err(),
            CredentialError::InvalidSecret
        );
        assert_eq!(
            store
                .set_secret("anthropic", &"x".repeat(MAX_SECRET_CHARS + 1))
                .unwrap_err(),
            CredentialError::InvalidSecret
        );
        assert!(!is_allowed_provider("not-a-provider"));
    }

    #[test]
    fn redact_hides_fixture_keys_and_invoke_like_payloads() {
        let secret = "sk-ant-testfixture0000000000";
        let logged = redact_secrets(&format!(
            "set_provider_secret args: provider=anthropic api_key={secret}"
        ));
        assert!(!logged.contains(secret));
        assert!(logged.contains("REDACTED"));
        assert!(!redact_secrets("token=abcdefg hij").contains("abcdefg"));
    }
}
