//! Durable non-secret selected-model state for the Personal Provider proxy.
//!
//! This state deliberately lives outside `provider.json`: provider configuration
//! binds a Provider endpoint to an opaque secret reference, while this document
//! records the one model that completed a successful chat capability probe.

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// File name under the Personal config directory.
pub const SELECTED_MODEL_FILE_NAME: &str = "selected-model.json";

const SELECTED_MODEL_SCHEMA_VERSION: u32 = 1;

/// Failures while validating or persisting selected-model state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectedModelError {
    /// The model identifier does not satisfy the local persistence policy.
    Invalid { detail: &'static str },
    /// The document exists but is not a valid fixed-schema document.
    Corrupt { detail: &'static str },
    /// The document could not be read, written, or deleted.
    Io { detail: &'static str },
}

impl fmt::Display for SelectedModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { detail } => write!(formatter, "invalid selected model: {detail}"),
            Self::Corrupt { detail } => write!(formatter, "selected model corrupt: {detail}"),
            Self::Io { detail } => write!(formatter, "selected model I/O failure: {detail}"),
        }
    }
}

impl std::error::Error for SelectedModelError {}

/// A chat-capable Provider model selected by the most recent successful probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedModel {
    model_id: String,
    selected_snapshot_digest: String,
    chat_capable: bool,
}

impl SelectedModel {
    /// Build a validated non-secret selected-model record.
    pub fn new(
        model_id: impl Into<String>,
        selected_snapshot_digest: impl Into<String>,
        chat_capable: bool,
    ) -> Result<Self, SelectedModelError> {
        let model_id = model_id.into();
        let selected_snapshot_digest = selected_snapshot_digest.into();
        validate_model_id(&model_id)?;
        validate_snapshot_digest(&selected_snapshot_digest)?;
        if !chat_capable {
            return Err(SelectedModelError::Invalid {
                detail: "selected model is not chat capable",
            });
        }
        Ok(Self {
            model_id,
            selected_snapshot_digest,
            chat_capable,
        })
    }

    /// Model identifier required by the local proxy.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Digest of the exact successful capability snapshot that selected it.
    pub fn selected_snapshot_digest(&self) -> &str {
        &self.selected_snapshot_digest
    }

    /// Whether the selected model passed the non-streaming chat probe.
    pub fn chat_capable(&self) -> bool {
        self.chat_capable
    }

    fn to_json_document(&self) -> String {
        format!(
            "{{\n  \"schema_version\": {SELECTED_MODEL_SCHEMA_VERSION},\n  \"selected_model\": \"{}\",\n  \"selected_snapshot_digest\": \"{}\",\n  \"chat_capable\": true\n}}\n",
            escape_json_string(&self.model_id),
            escape_json_string(&self.selected_snapshot_digest)
        )
    }

    fn from_json_document(document: &str) -> Result<Self, SelectedModelError> {
        let schema_version = required_u32_field(document, "schema_version")?;
        if schema_version != SELECTED_MODEL_SCHEMA_VERSION {
            return Err(SelectedModelError::Corrupt {
                detail: "unsupported selected model schema_version",
            });
        }
        Self::new(
            required_string_field(document, "selected_model")?,
            required_string_field(document, "selected_snapshot_digest")?,
            required_bool_field(document, "chat_capable")?,
        )
    }
}

fn validate_snapshot_digest(digest: &str) -> Result<(), SelectedModelError> {
    if digest.is_empty()
        || digest.len() > 128
        || digest
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(SelectedModelError::Invalid {
            detail: "selected_snapshot_digest invalid",
        });
    }
    Ok(())
}

/// Load, atomically store, and clear selected-model state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedModelRepository {
    state_path: PathBuf,
}

impl SelectedModelRepository {
    /// Use an explicit state file path (tests and non-XDG callers).
    pub fn from_file_path(state_path: impl Into<PathBuf>) -> Self {
        Self {
            state_path: state_path.into(),
        }
    }

    /// Resolve `config_dir/selected-model.json`.
    pub fn under_config_dir(config_dir: impl AsRef<Path>) -> Self {
        Self::from_file_path(config_dir.as_ref().join(SELECTED_MODEL_FILE_NAME))
    }

    /// Absolute path of the selected-model document.
    pub fn path(&self) -> &Path {
        &self.state_path
    }

    /// Load selected-model state, or `None` when no successful probe selected one.
    pub fn load(&self) -> Result<Option<SelectedModel>, SelectedModelError> {
        let document = match fs::read_to_string(&self.state_path) {
            Ok(document) => document,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => {
                return Err(SelectedModelError::Io {
                    detail: "failed to read selected model file",
                });
            }
        };
        SelectedModel::from_json_document(&document).map(Some)
    }

    /// Atomically persist a chat-capable selected model without secret material.
    pub fn store(&self, selected_model: &SelectedModel) -> Result<(), SelectedModelError> {
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent).map_err(|_| SelectedModelError::Io {
                detail: "failed to create selected model parent directory",
            })?;
        }
        let temporary_path = self.state_path.with_extension("json.tmp");
        let mut file = fs::File::create(&temporary_path).map_err(|_| SelectedModelError::Io {
            detail: "failed to create temporary selected model file",
        })?;
        file.write_all(selected_model.to_json_document().as_bytes())
            .map_err(|_| SelectedModelError::Io {
                detail: "failed to write temporary selected model file",
            })?;
        file.sync_all().map_err(|_| SelectedModelError::Io {
            detail: "failed to sync temporary selected model file",
        })?;
        fs::rename(&temporary_path, &self.state_path).map_err(|_| SelectedModelError::Io {
            detail: "failed to publish selected model file",
        })
    }

    /// Clear stale selected-model state. Missing state is already cleared.
    pub fn clear(&self) -> Result<(), SelectedModelError> {
        match fs::remove_file(&self.state_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(SelectedModelError::Io {
                detail: "failed to delete selected model file",
            }),
        }
    }
}

fn validate_model_id(model_id: &str) -> Result<(), SelectedModelError> {
    if model_id.is_empty() || model_id.len() > 256 {
        return Err(SelectedModelError::Invalid {
            detail: "model_id length out of range",
        });
    }
    if model_id
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(SelectedModelError::Invalid {
            detail: "model_id contains whitespace or control characters",
        });
    }
    Ok(())
}

fn required_u32_field(document: &str, field_name: &str) -> Result<u32, SelectedModelError> {
    let key = format!("\"{field_name}\"");
    let after_key = document
        .split_once(&key)
        .and_then(|(_, remainder)| remainder.split_once(':').map(|(_, value)| value))
        .ok_or(SelectedModelError::Corrupt {
            detail: "required selected model field missing",
        })?;
    let value = after_key
        .trim_start()
        .split([',', '}', '\n'])
        .next()
        .unwrap_or("")
        .trim();
    value
        .parse::<u32>()
        .map_err(|_| SelectedModelError::Corrupt {
            detail: "selected model numeric field invalid",
        })
}

fn required_string_field(document: &str, field_name: &str) -> Result<String, SelectedModelError> {
    let key = format!("\"{field_name}\"");
    let after_key = document
        .split_once(&key)
        .and_then(|(_, remainder)| remainder.split_once(':').map(|(_, value)| value))
        .ok_or(SelectedModelError::Corrupt {
            detail: "required selected model field missing",
        })?
        .trim_start();
    let Some(after_quote) = after_key.strip_prefix('"') else {
        return Err(SelectedModelError::Corrupt {
            detail: "selected model string field invalid",
        });
    };
    let Some((value, _)) = after_quote.split_once('"') else {
        return Err(SelectedModelError::Corrupt {
            detail: "selected model string field invalid",
        });
    };
    if value.contains('\\') {
        return Err(SelectedModelError::Corrupt {
            detail: "selected model escaping is unsupported",
        });
    }
    Ok(value.to_owned())
}

fn required_bool_field(document: &str, field_name: &str) -> Result<bool, SelectedModelError> {
    let key = format!("\"{field_name}\"");
    let value = document
        .split_once(&key)
        .and_then(|(_, remainder)| remainder.split_once(':').map(|(_, value)| value))
        .ok_or(SelectedModelError::Corrupt {
            detail: "required selected model field missing",
        })?
        .trim_start()
        .split([',', '}', '\n'])
        .next()
        .unwrap_or("")
        .trim();
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(SelectedModelError::Corrupt {
            detail: "selected model boolean field invalid",
        }),
    }
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
