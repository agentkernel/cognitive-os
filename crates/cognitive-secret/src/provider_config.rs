//! Non-secret Provider configuration document for Personal (P1-T02).
//!
//! The on-disk document may store only provider id, base URL, opaque
//! [`SecretRef`], and an optional model-snapshot digest. Secret bytes never
//! appear in this file, in Debug/Display output, or in error messages.

use crate::error::SecretError;
use crate::material::SecretRef;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// File name under the Personal XDG config directory.
pub const PROVIDER_CONFIG_FILE_NAME: &str = "provider.json";

/// Current on-disk schema version for [`ProviderConfig`].
pub const PROVIDER_CONFIG_SCHEMA_VERSION: u32 = 1;

/// Failures while validating or persisting non-secret Provider configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderConfigError {
    /// Structural or policy validation failed.
    Invalid { detail: &'static str },
    /// The config file is missing.
    NotFound,
    /// The file exists but cannot be parsed as the fixed schema.
    Corrupt { detail: &'static str },
    /// Filesystem I/O failed without embedding secret material.
    Io { detail: &'static str },
}

impl fmt::Display for ProviderConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { detail } => {
                write!(formatter, "invalid provider config: {detail}")
            }
            Self::NotFound => write!(formatter, "provider config not found"),
            Self::Corrupt { detail } => {
                write!(formatter, "provider config corrupt: {detail}")
            }
            Self::Io { detail } => write!(formatter, "provider config I/O failure: {detail}"),
        }
    }
}

impl std::error::Error for ProviderConfigError {}

impl From<SecretError> for ProviderConfigError {
    fn from(error: SecretError) -> Self {
        match error {
            SecretError::InvalidAttributes { detail } => Self::Invalid { detail },
            _ => Self::Invalid {
                detail: "secret reference rejected by SecretRef validation",
            },
        }
    }
}

/// Non-secret Provider configuration persisted under XDG config.
#[derive(Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    provider_id: String,
    base_url: String,
    secret_ref: SecretRef,
    selected_snapshot_digest: Option<String>,
}

impl ProviderConfig {
    /// Build a validated config. `base_url` must be absolute HTTPS.
    pub fn new(
        provider_id: impl Into<String>,
        base_url: impl Into<String>,
        secret_ref: SecretRef,
        selected_snapshot_digest: Option<String>,
    ) -> Result<Self, ProviderConfigError> {
        let provider_id = provider_id.into();
        validate_provider_id(&provider_id)?;
        let base_url = base_url.into();
        validate_https_base_url(&base_url)?;
        if let Some(digest) = &selected_snapshot_digest {
            validate_snapshot_digest(digest)?;
        }
        Ok(Self {
            provider_id,
            base_url,
            secret_ref,
            selected_snapshot_digest,
        })
    }

    /// Provider identifier (for example `deepseek`). Never a secret.
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Absolute HTTPS base URL for the OpenAI-compatible API root.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Opaque handle into the native SecretStore. Not secret material.
    pub fn secret_ref(&self) -> &SecretRef {
        &self.secret_ref
    }

    /// Optional capability snapshot digest from later Provider probe work.
    pub fn selected_snapshot_digest(&self) -> Option<&str> {
        self.selected_snapshot_digest.as_deref()
    }

    /// Replace only the opaque secret reference (used after rotate flows).
    pub fn with_secret_ref(&self, secret_ref: SecretRef) -> Self {
        Self {
            provider_id: self.provider_id.clone(),
            base_url: self.base_url.clone(),
            secret_ref,
            selected_snapshot_digest: self.selected_snapshot_digest.clone(),
        }
    }

    /// Replace only the selected capability-snapshot digest (P1-T03).
    ///
    /// Pass `None` to clear a previously selected digest. Digest text is
    /// validated with the same rules as [`Self::new`].
    pub fn with_selected_snapshot_digest(
        &self,
        selected_snapshot_digest: Option<String>,
    ) -> Result<Self, ProviderConfigError> {
        if let Some(digest) = &selected_snapshot_digest {
            validate_snapshot_digest(digest)?;
        }
        Ok(Self {
            provider_id: self.provider_id.clone(),
            base_url: self.base_url.clone(),
            secret_ref: self.secret_ref.clone(),
            selected_snapshot_digest,
        })
    }

    /// Serialize the fixed JSON schema. Secret bytes are never included.
    pub fn to_json_document(&self) -> String {
        let digest_json = match &self.selected_snapshot_digest {
            Some(digest) => format!("\"{}\"", escape_json_string(digest)),
            None => "null".to_owned(),
        };
        format!(
            "{{\n  \"schema_version\": {version},\n  \"provider_id\": \"{provider_id}\",\n  \"base_url\": \"{base_url}\",\n  \"secret_ref\": \"{secret_ref}\",\n  \"selected_snapshot_digest\": {digest}\n}}\n",
            version = PROVIDER_CONFIG_SCHEMA_VERSION,
            provider_id = escape_json_string(&self.provider_id),
            base_url = escape_json_string(&self.base_url),
            secret_ref = escape_json_string(self.secret_ref.as_str()),
            digest = digest_json,
        )
    }

    /// Parse the fixed JSON schema produced by [`Self::to_json_document`].
    pub fn from_json_document(document: &str) -> Result<Self, ProviderConfigError> {
        let schema_version = required_u32_field(document, "schema_version")?;
        if schema_version != PROVIDER_CONFIG_SCHEMA_VERSION {
            return Err(ProviderConfigError::Corrupt {
                detail: "unsupported provider config schema_version",
            });
        }
        let provider_id = required_string_field(document, "provider_id")?;
        let base_url = required_string_field(document, "base_url")?;
        let secret_ref_raw = required_string_field(document, "secret_ref")?;
        let secret_ref = SecretRef::from_opaque(secret_ref_raw)?;
        let selected_snapshot_digest = optional_string_field(document, "selected_snapshot_digest")?;
        Self::new(provider_id, base_url, secret_ref, selected_snapshot_digest)
    }
}

impl fmt::Debug for ProviderConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderConfig")
            .field("provider_id", &self.provider_id)
            .field("base_url", &self.base_url)
            .field("secret_ref", &self.secret_ref)
            .field("selected_snapshot_digest", &self.selected_snapshot_digest)
            .finish()
    }
}

/// Load and store [`ProviderConfig`] under a Personal config directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderConfigRepository {
    config_path: PathBuf,
}

impl ProviderConfigRepository {
    /// Use an explicit config file path (tests and non-XDG callers).
    pub fn from_file_path(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    /// Resolve `config_dir/provider.json` (intended: Personal XDG config dir).
    pub fn under_config_dir(config_dir: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_dir.as_ref().join(PROVIDER_CONFIG_FILE_NAME),
        }
    }

    /// Absolute path of the provider config document.
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    /// Load config when present. Missing file returns [`ProviderConfigError::NotFound`].
    pub fn load(&self) -> Result<ProviderConfig, ProviderConfigError> {
        let document = fs::read_to_string(&self.config_path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                ProviderConfigError::NotFound
            } else {
                ProviderConfigError::Io {
                    detail: "failed to read provider config file",
                }
            }
        })?;
        ProviderConfig::from_json_document(&document)
    }

    /// Atomically replace the config document. Never writes secret material.
    pub fn store(&self, config: &ProviderConfig) -> Result<(), ProviderConfigError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|_| ProviderConfigError::Io {
                detail: "failed to create provider config parent directory",
            })?;
            #[cfg(unix)]
            restrict_private_directory(parent)?;
        }

        let temporary_path = self.config_path.with_extension("json.tmp");
        {
            let mut file =
                fs::File::create(&temporary_path).map_err(|_| ProviderConfigError::Io {
                    detail: "failed to create temporary provider config file",
                })?;
            file.write_all(config.to_json_document().as_bytes())
                .map_err(|_| ProviderConfigError::Io {
                    detail: "failed to write temporary provider config file",
                })?;
            file.sync_all().map_err(|_| ProviderConfigError::Io {
                detail: "failed to sync temporary provider config file",
            })?;
        }
        #[cfg(unix)]
        restrict_private_file(&temporary_path)?;

        fs::rename(&temporary_path, &self.config_path).map_err(|_| ProviderConfigError::Io {
            detail: "failed to publish provider config file",
        })?;
        #[cfg(unix)]
        restrict_private_file(&self.config_path)?;
        Ok(())
    }

    /// Remove the config document if present. Does not touch SecretStore items.
    pub fn delete_file(&self) -> Result<(), ProviderConfigError> {
        match fs::remove_file(&self.config_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(ProviderConfigError::Io {
                detail: "failed to delete provider config file",
            }),
        }
    }
}

fn validate_provider_id(provider_id: &str) -> Result<(), ProviderConfigError> {
    if provider_id.is_empty() || provider_id.len() > 64 {
        return Err(ProviderConfigError::Invalid {
            detail: "provider_id length out of range",
        });
    }
    if !provider_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ProviderConfigError::Invalid {
            detail: "provider_id has unsupported characters",
        });
    }
    Ok(())
}

fn validate_https_base_url(base_url: &str) -> Result<(), ProviderConfigError> {
    if base_url.len() > 512 {
        return Err(ProviderConfigError::Invalid {
            detail: "base_url exceeds length limit",
        });
    }
    let lowercase = base_url.to_ascii_lowercase();
    if !lowercase.starts_with("https://") {
        return Err(ProviderConfigError::Invalid {
            detail: "base_url must use https://",
        });
    }
    if base_url.chars().any(char::is_whitespace) {
        return Err(ProviderConfigError::Invalid {
            detail: "base_url must not contain whitespace",
        });
    }
    let without_scheme = &base_url["https://".len()..];
    if without_scheme.contains('@') {
        return Err(ProviderConfigError::Invalid {
            detail: "base_url must not embed credentials",
        });
    }
    if without_scheme.is_empty() || without_scheme.starts_with('/') {
        return Err(ProviderConfigError::Invalid {
            detail: "base_url host is required",
        });
    }
    Ok(())
}

fn validate_snapshot_digest(digest: &str) -> Result<(), ProviderConfigError> {
    if digest.is_empty() || digest.len() > 128 {
        return Err(ProviderConfigError::Invalid {
            detail: "selected_snapshot_digest length out of range",
        });
    }
    // Product-local identity digests may use a short algorithm prefix
    // (for example `fnv1a64:`) plus hex. Allow alphanumerics and common
    // separators; never treat this field as secret material.
    if !digest
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.'))
    {
        return Err(ProviderConfigError::Invalid {
            detail: "selected_snapshot_digest has unsupported characters",
        });
    }
    Ok(())
}

fn escape_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(escaped, "\\u{:04x}", u32::from(ch));
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn required_u32_field(document: &str, field_name: &str) -> Result<u32, ProviderConfigError> {
    let marker = format!("\"{field_name}\"");
    let field_offset = document.find(&marker).ok_or(ProviderConfigError::Corrupt {
        detail: "required numeric field missing",
    })?;
    let after_name = &document[field_offset + marker.len()..];
    let after_colon =
        after_name
            .split_once(':')
            .map(|(_, rest)| rest)
            .ok_or(ProviderConfigError::Corrupt {
                detail: "numeric field missing colon",
            })?;
    let token = after_colon
        .trim_start()
        .split(|character: char| character == ',' || character == '}' || character.is_whitespace())
        .next()
        .ok_or(ProviderConfigError::Corrupt {
            detail: "numeric field token missing",
        })?;
    token
        .parse::<u32>()
        .map_err(|_| ProviderConfigError::Corrupt {
            detail: "numeric field is not a u32",
        })
}

fn required_string_field(document: &str, field_name: &str) -> Result<String, ProviderConfigError> {
    match optional_string_field(document, field_name)? {
        Some(value) => Ok(value),
        None => Err(ProviderConfigError::Corrupt {
            detail: "required string field missing or null",
        }),
    }
}

fn optional_string_field(
    document: &str,
    field_name: &str,
) -> Result<Option<String>, ProviderConfigError> {
    let marker = format!("\"{field_name}\"");
    let Some(field_offset) = document.find(&marker) else {
        return Ok(None);
    };
    let after_name = &document[field_offset + marker.len()..];
    let after_colon = after_name
        .split_once(':')
        .map(|(_, rest)| rest.trim_start())
        .ok_or(ProviderConfigError::Corrupt {
            detail: "string field missing colon",
        })?;
    if after_colon.starts_with("null") {
        return Ok(None);
    }
    if !after_colon.starts_with('"') {
        return Err(ProviderConfigError::Corrupt {
            detail: "string field is not a JSON string",
        });
    }
    let mut decoded = String::new();
    let mut characters = after_colon[1..].chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '"' => return Ok(Some(decoded)),
            '\\' => {
                let escaped = characters.next().ok_or(ProviderConfigError::Corrupt {
                    detail: "string field has truncated escape",
                })?;
                match escaped {
                    '"' => decoded.push('"'),
                    '\\' => decoded.push('\\'),
                    'n' => decoded.push('\n'),
                    'r' => decoded.push('\r'),
                    't' => decoded.push('\t'),
                    'u' => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            let digit = characters.next().ok_or(ProviderConfigError::Corrupt {
                                detail: "string field has truncated unicode escape",
                            })?;
                            hex.push(digit);
                        }
                        let code_point = u32::from_str_radix(&hex, 16).map_err(|_| {
                            ProviderConfigError::Corrupt {
                                detail: "string field has invalid unicode escape",
                            }
                        })?;
                        let decoded_char =
                            char::from_u32(code_point).ok_or(ProviderConfigError::Corrupt {
                                detail: "string field has invalid unicode code point",
                            })?;
                        decoded.push(decoded_char);
                    }
                    _ => {
                        return Err(ProviderConfigError::Corrupt {
                            detail: "string field has unsupported escape",
                        });
                    }
                }
            }
            other => decoded.push(other),
        }
    }
    Err(ProviderConfigError::Corrupt {
        detail: "string field is not terminated",
    })
}

#[cfg(unix)]
fn restrict_private_directory(path: &Path) -> Result<(), ProviderConfigError> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, permissions).map_err(|_| ProviderConfigError::Io {
        detail: "failed to set provider config directory mode 0700",
    })
}

#[cfg(unix)]
fn restrict_private_file(path: &Path) -> Result<(), ProviderConfigError> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions).map_err(|_| ProviderConfigError::Io {
        detail: "failed to set provider config file mode 0600",
    })
}
