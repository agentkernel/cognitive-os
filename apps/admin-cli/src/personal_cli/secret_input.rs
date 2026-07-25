//! Capture Provider API key material without logging or echoing bytes.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use cognitive_secret::{SecretError, SecretMaterial, read_secret_material_from_reader};

/// Read secret material from a file path, stdin (`-`), or an interactive TTY.
///
/// Interactive mode never prints the bytes. On hosts where echo-off cannot be
/// configured, the CLI fails closed and asks the operator to pass
/// `--api-key-file` instead of accepting a visible prompt.
pub fn read_api_key_material(api_key_file: Option<&Path>) -> Result<SecretMaterial, String> {
    match api_key_file {
        Some(path) if path.as_os_str() == "-" => {
            let mut stdin = io::stdin().lock();
            read_secret_material_from_reader(&mut stdin).map_err(map_secret_error)
        }
        Some(path) => {
            let mut file = File::open(path).map_err(|error| {
                format!(
                    "unable to open --api-key-file {}: {error}",
                    path.display()
                )
            })?;
            read_secret_material_from_reader(&mut file).map_err(map_secret_error)
        }
        None => read_interactive_hidden_material(),
    }
}

fn map_secret_error(error: SecretError) -> String {
    // SecretError Display is already redacted / non-material.
    format!("failed to capture API key material: {error}")
}

fn read_interactive_hidden_material() -> Result<SecretMaterial, String> {
    #[cfg(unix)]
    {
        return read_unix_hidden_material();
    }
    #[cfg(not(unix))]
    {
        Err(
            "interactive hidden API key input is not available on this host; \
             re-run with --api-key-file <path> or --api-key-file -"
                .to_owned(),
        )
    }
}

#[cfg(unix)]
fn read_unix_hidden_material() -> Result<SecretMaterial, String> {
    use std::io::Write;
    use std::process::Command;

    eprint!("Enter Provider API key (input hidden): ");
    let _ = io::stderr().flush();

    // Best-effort echo-off via stty. Failure fails closed rather than accepting
    // a visible secret prompt.
    let stty_off = Command::new("stty")
        .args(["-echo"])
        .status()
        .map_err(|error| format!("unable to disable terminal echo: {error}"))?;
    if !stty_off.success() {
        return Err(
            "unable to disable terminal echo for hidden input; re-run with --api-key-file"
                .to_owned(),
        );
    }

    let mut line = String::new();
    let read_result = io::stdin().read_line(&mut line);
    let _ = Command::new("stty").args(["echo"]).status();
    eprintln!();
    read_result.map_err(|error| format!("failed to read API key: {error}"))?;

    let mut bytes = line.into_bytes();
    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }
    SecretMaterial::from_bytes(bytes).map_err(map_secret_error)
}
