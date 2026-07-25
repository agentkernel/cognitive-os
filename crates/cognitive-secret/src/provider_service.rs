//! Bind Provider configuration to a [`SecretStore`] without leaking material.
//!
//! Personal daemons and CLIs use this service to put/rotate/delete Provider API
//! keys. Config files only receive opaque [`SecretRef`] values. This module is
//! not an authority writer and never touches SQLite or Task/Effect state.

use crate::error::SecretError;
use crate::material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};
use crate::provider_config::{ProviderConfig, ProviderConfigError, ProviderConfigRepository};
use crate::store::{SecretStore, SecretStoreAvailability};
use std::fmt;

/// Stable Secret Service label for Personal Provider keys.
pub const PROVIDER_SECRET_LABEL: &str = "cognitiveos-personal-provider-api-key";

/// Failures while binding Provider config to secret material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderKeyServiceError {
    /// Secret backend rejected the operation.
    Secret(SecretError),
    /// Non-secret config document rejected the operation.
    Config(ProviderConfigError),
    /// Config exists but the secret ref no longer resolves.
    SecretMissing,
    /// No provider configuration has been stored yet.
    NotConfigured,
}

impl fmt::Display for ProviderKeyServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secret(error) => write!(formatter, "provider key secret failure: {error}"),
            Self::Config(error) => write!(formatter, "provider key config failure: {error}"),
            Self::SecretMissing => write!(
                formatter,
                "provider key secret is missing for the configured secret_ref"
            ),
            Self::NotConfigured => write!(formatter, "provider is not configured"),
        }
    }
}

impl std::error::Error for ProviderKeyServiceError {}

impl From<SecretError> for ProviderKeyServiceError {
    fn from(error: SecretError) -> Self {
        Self::Secret(error)
    }
}

impl From<ProviderConfigError> for ProviderKeyServiceError {
    fn from(error: ProviderConfigError) -> Self {
        Self::Config(error)
    }
}

/// Build the non-secret attributes used to identify a Provider API key item.
pub fn provider_secret_attributes(
    provider_id: &str,
) -> Result<SecretAttributes, ProviderKeyServiceError> {
    Ok(SecretAttributes::from_pairs(vec![
        ("application".into(), "cognitiveos-personal".into()),
        ("provider".into(), provider_id.to_owned()),
        ("purpose".into(), "provider-api-key".into()),
    ])?)
}

/// Build the desktop collection label for a Provider API key item.
pub fn provider_secret_label() -> Result<SecretLabel, ProviderKeyServiceError> {
    Ok(SecretLabel::new(PROVIDER_SECRET_LABEL)?)
}

/// Daemon-facing binding between non-secret Provider config and SecretStore.
pub struct ProviderKeyService<S> {
    secret_store: S,
    config_repository: ProviderConfigRepository,
}

impl<S> fmt::Debug for ProviderKeyService<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderKeyService")
            .field("config_path", &self.config_repository.path())
            .field("secret_store", &"<redacted-backend>")
            .finish()
    }
}

impl<S: SecretStore> ProviderKeyService<S> {
    /// Construct a service over an injected SecretStore and config repository.
    pub fn new(secret_store: S, config_repository: ProviderConfigRepository) -> Self {
        Self {
            secret_store,
            config_repository,
        }
    }

    /// Borrow the underlying SecretStore (for advanced probe diagnostics only).
    pub fn secret_store(&self) -> &S {
        &self.secret_store
    }

    /// Borrow the config repository.
    pub fn config_repository(&self) -> &ProviderConfigRepository {
        &self.config_repository
    }

    /// Non-mutating readiness probe of the secret backend.
    pub fn probe_secret_store(&self) -> Result<SecretStoreAvailability, ProviderKeyServiceError> {
        Ok(self.secret_store.probe()?)
    }

    /// Load the non-secret Provider config if present.
    pub fn load_config(&self) -> Result<Option<ProviderConfig>, ProviderKeyServiceError> {
        match self.config_repository.load() {
            Ok(config) => Ok(Some(config)),
            Err(ProviderConfigError::NotFound) => Ok(None),
            Err(error) => Err(ProviderKeyServiceError::Config(error)),
        }
    }

    /// Put or replace the Provider API key and persist only the opaque ref.
    pub fn configure_provider(
        &self,
        provider_id: &str,
        base_url: &str,
        material: SecretMaterial,
        selected_snapshot_digest: Option<String>,
    ) -> Result<ProviderConfig, ProviderKeyServiceError> {
        let label = provider_secret_label()?;
        let attributes = provider_secret_attributes(provider_id)?;
        let secret_ref = self.secret_store.put(&label, &attributes, material)?;
        let config =
            ProviderConfig::new(provider_id, base_url, secret_ref, selected_snapshot_digest)?;
        self.config_repository.store(&config)?;
        Ok(config)
    }

    /// Rotate secret material for the currently configured provider.
    pub fn rotate_provider_key(
        &self,
        material: SecretMaterial,
    ) -> Result<ProviderConfig, ProviderKeyServiceError> {
        let existing = self
            .load_config()?
            .ok_or(ProviderKeyServiceError::NotConfigured)?;
        let label = provider_secret_label()?;
        let attributes = provider_secret_attributes(existing.provider_id())?;
        let secret_ref = self.secret_store.put(&label, &attributes, material)?;
        let updated = existing.with_secret_ref(secret_ref);
        self.config_repository.store(&updated)?;
        Ok(updated)
    }

    /// Delete the secret material for the configured ref and remove the config.
    pub fn delete_provider_key(&self) -> Result<(), ProviderKeyServiceError> {
        match self.load_config()? {
            Some(config) => {
                match self.secret_store.delete(config.secret_ref()) {
                    Ok(()) => {}
                    Err(SecretError::NotFound) => {}
                    Err(error) => return Err(ProviderKeyServiceError::Secret(error)),
                }
                self.config_repository.delete_file()?;
                Ok(())
            }
            None => Err(ProviderKeyServiceError::NotConfigured),
        }
    }

    /// Resolve secret material for the configured provider after restart.
    pub fn resolve_provider_material(&self) -> Result<SecretMaterial, ProviderKeyServiceError> {
        let config = self
            .load_config()?
            .ok_or(ProviderKeyServiceError::NotConfigured)?;
        match self.secret_store.get(config.secret_ref()) {
            Ok(material) => Ok(material),
            Err(SecretError::NotFound) => Err(ProviderKeyServiceError::SecretMissing),
            Err(error) => Err(ProviderKeyServiceError::Secret(error)),
        }
    }

    /// Return the configured opaque secret ref without resolving material.
    pub fn configured_secret_ref(&self) -> Result<SecretRef, ProviderKeyServiceError> {
        let config = self
            .load_config()?
            .ok_or(ProviderKeyServiceError::NotConfigured)?;
        Ok(config.secret_ref().clone())
    }
}
