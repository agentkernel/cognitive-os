//! Capture secret material from a reader without logging or echoing bytes.
//!
//! CLI hidden-input wiring (terminal echo off) belongs to the product binary
//! (P1-T06). This helper only provides the non-logging material capture used by
//! that CLI and by tests that feed synthetic non-production bytes.

use crate::error::SecretError;
use crate::material::SecretMaterial;
use std::io::Read;

/// Read secret bytes from `reader`, strip a single trailing newline, and wrap
/// them as [`SecretMaterial`].
///
/// The function never writes the bytes to logs, environment variables, or
/// temporary files. Callers must drop the material promptly after `put`.
pub fn read_secret_material_from_reader(
    reader: &mut impl Read,
) -> Result<SecretMaterial, SecretError> {
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|_| SecretError::Backend {
            detail: "failed to read secret material from input",
        })?;
    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }
    SecretMaterial::from_bytes(bytes)
}
