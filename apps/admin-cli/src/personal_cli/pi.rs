//! Non-secret Personal Pi configuration writer.

use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use super::LayoutRoots;
use super::layout::build_layout;

const PI_CONFIG_FILE_NAME: &str = "pi.json";
const PI_CONFIG_SCHEMA_VERSION: u64 = 1;
const PI_CONFIG_SURFACE: &str = "personal-pi-config";

/// Inputs accepted by `cognitive pi configure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiConfigureOptions {
    pub layout_roots: LayoutRoots,
    pub executable_path: PathBuf,
    pub extension_entry_path: PathBuf,
}

/// Write the Personal Pi configuration without consulting Provider state.
///
/// The daemon owns subsequent file observation and version validation. This
/// client operation deliberately does not start Pi, access a SecretStore, or
/// inspect Provider/authority state.
pub fn configure(options: &PiConfigureOptions) -> Result<Value, String> {
    validate_absolute_path(&options.executable_path, "Pi executable")?;
    validate_absolute_path(&options.extension_entry_path, "CognitiveOS Extension entry")?;

    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    layout
        .ensure_directories()
        .map_err(|error| format!("unable to create Personal configuration directory: {error}"))?;

    let configuration_path = layout.config_dir().join(PI_CONFIG_FILE_NAME);
    let document = json!({
        "schema_version": PI_CONFIG_SCHEMA_VERSION,
        "surface": PI_CONFIG_SURFACE,
        "executable_path": options.executable_path,
        "extension_entry_path": options.extension_entry_path,
    });
    let serialized = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("unable to serialize non-secret Pi configuration: {error}"))?;
    atomic_write_configuration(&configuration_path, &serialized)?;

    Ok(json!({
        "status": "ok",
        "surface": "cognitive-pi-configure",
        "action": "configured",
        "config_path": configuration_path,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false,
    }))
}

fn validate_absolute_path(path: &std::path::Path, label: &str) -> Result<(), String> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(format!("{label} path must be absolute"))
}

fn atomic_write_configuration(path: &std::path::Path, contents: &[u8]) -> Result<(), String> {
    let parent_directory = path
        .parent()
        .ok_or_else(|| "Pi configuration has no parent directory".to_owned())?;
    let temporary_path =
        parent_directory.join(format!(".{PI_CONFIG_FILE_NAME}.{}.tmp", std::process::id()));
    fs::write(&temporary_path, contents).map_err(|error| {
        format!(
            "unable to write temporary Pi configuration at {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        format!(
            "unable to atomically publish Pi configuration at {}: {error}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_rejects_relative_paths_before_writing_any_file() {
        let options = PiConfigureOptions {
            layout_roots: LayoutRoots { runtime_root: None },
            executable_path: PathBuf::from("pi"),
            extension_entry_path: PathBuf::from("extension.js"),
        };

        let error = configure(&options).expect_err("relative paths must be rejected");

        assert!(error.contains("absolute"), "{error}");
    }

    #[test]
    fn configuration_writes_only_the_documented_non_secret_pi_fields() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let executable_path = temporary_root.path().join("bin").join("pi");
        let extension_entry_path = temporary_root.path().join("extension").join("index.js");
        let options = PiConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary_root.path().to_path_buf()),
            },
            executable_path: executable_path.clone(),
            extension_entry_path: extension_entry_path.clone(),
        };

        let report = configure(&options).expect("write non-secret Pi configuration");
        let configuration_path = temporary_root.path().join("config/cognitiveos/pi.json");
        let document: Value =
            serde_json::from_slice(&fs::read(&configuration_path).expect("read Pi configuration"))
                .expect("parse Pi configuration");

        assert_eq!(document["schema_version"], PI_CONFIG_SCHEMA_VERSION);
        assert_eq!(document["surface"], PI_CONFIG_SURFACE);
        assert_eq!(
            document["executable_path"],
            executable_path.display().to_string()
        );
        assert_eq!(
            document["extension_entry_path"],
            extension_entry_path.display().to_string()
        );
        assert_eq!(document.as_object().expect("object").len(), 4);
        assert!(!document.to_string().contains("secret"));
        assert!(!document.to_string().contains("provider"));
        assert!(!document.to_string().contains("sqlite"));
        assert_eq!(report["authority_side_effects"], false);
        assert_eq!(report["gate_claim"], "not-claimed");
    }
}
