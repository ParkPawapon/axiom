use crate::ports::secure_storage::SecureStorage;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct KeychainStorage;

impl KeychainStorage {
    pub fn new() -> Self {
        Self
    }
}

impl SecureStorage for KeychainStorage {
    fn store_secret(&self, namespace: &str, key: &str, secret: &str) -> AppResult<()> {
        validate_secret_locator(namespace, key)?;
        let entry = keyring_entry(namespace, key)?;

        entry.set_password(secret).map_err(|error| {
            AppError::Infrastructure(format!("secure storage write failed: {error}"))
        })
    }

    fn get_secret(&self, namespace: &str, key: &str) -> AppResult<Option<String>> {
        validate_secret_locator(namespace, key)?;
        let entry = keyring_entry(namespace, key)?;

        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(AppError::Infrastructure(format!(
                "secure storage read failed: {error}"
            ))),
        }
    }

    fn delete_secret(&self, namespace: &str, key: &str) -> AppResult<()> {
        validate_secret_locator(namespace, key)?;
        let entry = keyring_entry(namespace, key)?;

        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(AppError::Infrastructure(format!(
                "secure storage delete failed: {error}"
            ))),
        }
    }
}

fn keyring_entry(namespace: &str, key: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(&format!("AxiomPHP.{namespace}"), key)
        .map_err(|error| AppError::Infrastructure(format!("secure storage entry failed: {error}")))
}

fn validate_secret_locator(namespace: &str, key: &str) -> AppResult<()> {
    for (label, value) in [("namespace", namespace), ("key", key)] {
        if value.trim().is_empty()
            || value.len() > 160
            || value
                .chars()
                .any(|character| character.is_control() || matches!(character, '/' | '\\'))
        {
            return Err(AppError::Validation(format!(
                "secure storage {label} is invalid"
            )));
        }
    }

    Ok(())
}
