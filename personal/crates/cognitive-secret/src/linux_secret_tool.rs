//! Linux native Secret Service adapter via `secret-tool` (P1-T02).
//!
//! This backend is `SecretStoreClass::Native`. It never writes secrets to
//! SQLite, config files, environment variables, argv, logs, or evidence.
//! Secret material is passed to `secret-tool store` only on stdin.
//!
//! Availability requires Linux, `secret-tool` on PATH, and a user session bus
//! signal (`DBUS_SESSION_BUS_ADDRESS` or `$XDG_RUNTIME_DIR/bus`). When those
//! are missing, probe reports Unavailable and mutating calls fail closed.

use crate::error::SecretError;
use crate::material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};
use crate::store::{SecretStore, SecretStoreAvailability, SecretStoreClass};
use std::io::Write;
use std::process::{Command, Stdio};

const SECRET_REF_PREFIX: &str = "ssv1:fdss";

/// Linux FreeDesktop Secret Service adapter driven by the `secret-tool` CLI.
#[derive(Debug, Default)]
pub struct LinuxSecretToolStore;

impl LinuxSecretToolStore {
    /// Construct a native adapter instance.
    pub fn new() -> Self {
        Self
    }

    fn session_bus_present() -> bool {
        if let Ok(address) = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            && !address.trim().is_empty()
        {
            return true;
        }
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let bus_path = std::path::Path::new(&runtime_dir).join("bus");
            if bus_path.exists() {
                return true;
            }
        }
        false
    }

    fn secret_tool_on_path() -> bool {
        Command::new("secret-tool")
            // libsecret's `secret-tool` reports usage with exit status 2 for
            // both `--version` and `--help`. Starting a lookup with complete,
            // non-sensitive probe attributes confirms the executable can be
            // invoked without interpreting lookup success as availability.
            // The actual `store` call remains the authoritative fail-closed
            // Secret Service operation.
            .arg("lookup")
            .arg("application")
            .arg("cognitiveos-secret-store-probe")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut child| child.wait())
            .is_ok()
    }

    fn require_available(&self) -> Result<(), SecretError> {
        match self.probe()? {
            SecretStoreAvailability::Available => Ok(()),
            SecretStoreAvailability::Locked => Err(SecretError::Locked),
            SecretStoreAvailability::PromptUnavailable => Err(SecretError::PromptUnavailable),
            SecretStoreAvailability::Unavailable => Err(SecretError::Unavailable {
                reason: "Linux secret-tool / session bus is not available",
            }),
        }
    }

    fn encode_secret_ref(attributes: &SecretAttributes) -> Result<SecretRef, SecretError> {
        let mut segments = vec![SECRET_REF_PREFIX.to_owned()];
        for (key, value) in attributes.pairs() {
            segments.push(key.clone());
            segments.push(value.clone());
        }
        SecretRef::from_opaque(segments.join("/"))
    }

    fn decode_secret_ref(secret_ref: &SecretRef) -> Result<SecretAttributes, SecretError> {
        let raw = secret_ref.as_str();
        let prefix = format!("{SECRET_REF_PREFIX}/");
        let Some(rest) = raw.strip_prefix(&prefix) else {
            return Err(SecretError::NotFound);
        };
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.is_empty() || !parts.len().is_multiple_of(2) {
            return Err(SecretError::NotFound);
        }
        let mut pairs = Vec::with_capacity(parts.len() / 2);
        let mut index = 0;
        while index < parts.len() {
            pairs.push((parts[index].to_owned(), parts[index + 1].to_owned()));
            index += 2;
        }
        SecretAttributes::from_pairs(pairs)
    }

    fn attribute_argv(attributes: &SecretAttributes) -> Vec<String> {
        let mut argv = Vec::new();
        for (key, value) in attributes.pairs() {
            argv.push(key.clone());
            argv.push(value.clone());
        }
        argv
    }

    fn map_command_failure(status_code: Option<i32>) -> SecretError {
        match status_code {
            Some(1) => SecretError::NotFound,
            _ => SecretError::Backend {
                detail: "secret-tool command failed",
            },
        }
    }
}

impl SecretStore for LinuxSecretToolStore {
    fn class(&self) -> SecretStoreClass {
        SecretStoreClass::Native
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        if !cfg!(target_os = "linux") {
            return Ok(SecretStoreAvailability::Unavailable);
        }
        if !Self::session_bus_present() {
            return Ok(SecretStoreAvailability::Unavailable);
        }
        if !Self::secret_tool_on_path() {
            return Ok(SecretStoreAvailability::Unavailable);
        }
        Ok(SecretStoreAvailability::Available)
    }

    fn put(
        &self,
        label: &SecretLabel,
        attributes: &SecretAttributes,
        material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        self.require_available()?;
        let secret_ref = Self::encode_secret_ref(attributes)?;
        let attribute_argv = Self::attribute_argv(attributes);

        let mut clear_command = Command::new("secret-tool");
        clear_command.arg("clear");
        for argument in &attribute_argv {
            clear_command.arg(argument);
        }
        let _ = clear_command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let mut store_command = Command::new("secret-tool");
        store_command
            .arg("store")
            .arg("--label")
            .arg(label.as_str());
        for argument in &attribute_argv {
            store_command.arg(argument);
        }
        let mut child = store_command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| SecretError::Backend {
                detail: "failed to spawn secret-tool store",
            })?;
        {
            let stdin = child.stdin.as_mut().ok_or(SecretError::Backend {
                detail: "secret-tool store stdin unavailable",
            })?;
            stdin
                .write_all(material.expose_bytes())
                .map_err(|_| SecretError::Backend {
                    detail: "failed to write secret material to secret-tool stdin",
                })?;
        }
        let status = child.wait().map_err(|_| SecretError::Backend {
            detail: "failed to wait for secret-tool store",
        })?;
        if !status.success() {
            return Err(Self::map_command_failure(status.code()));
        }
        Ok(secret_ref)
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        self.require_available()?;
        let attributes = Self::decode_secret_ref(secret_ref)?;
        let attribute_argv = Self::attribute_argv(&attributes);
        let mut lookup_command = Command::new("secret-tool");
        lookup_command.arg("lookup");
        for argument in &attribute_argv {
            lookup_command.arg(argument);
        }
        let output = lookup_command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|_| SecretError::Backend {
                detail: "failed to spawn secret-tool lookup",
            })?;
        if !output.status.success() {
            return Err(Self::map_command_failure(output.status.code()));
        }
        let mut bytes = output.stdout;
        while matches!(bytes.last(), Some(b'\n' | b'\r')) {
            bytes.pop();
        }
        SecretMaterial::from_bytes(bytes)
    }

    fn delete(&self, secret_ref: &SecretRef) -> Result<(), SecretError> {
        self.require_available()?;
        let attributes = Self::decode_secret_ref(secret_ref)?;
        let attribute_argv = Self::attribute_argv(&attributes);
        let mut clear_command = Command::new("secret-tool");
        clear_command.arg("clear");
        for argument in &attribute_argv {
            clear_command.arg(argument);
        }
        let status = clear_command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| SecretError::Backend {
                detail: "failed to spawn secret-tool clear",
            })?;
        if !status.success() {
            return Err(Self::map_command_failure(status.code()));
        }
        Ok(())
    }
}
