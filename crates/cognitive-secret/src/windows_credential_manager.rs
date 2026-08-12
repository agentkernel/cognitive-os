//! Windows native Credential Manager adapter via Windows PowerShell (P7-T07).
//!
//! This backend is `SecretStoreClass::Native`. It mirrors the Linux
//! `secret-tool` subprocess architecture so the crate keeps zero external
//! dependencies and the workspace `unsafe_code = "forbid"` policy: the Win32
//! Credential Manager calls (`CredWriteW`/`CredReadW`/`CredDeleteW`) run inside
//! a fixed, audited PowerShell helper script executed from the absolute
//! system PowerShell path.
//!
//! Hard boundaries (same as Linux):
//! - secret material transits only child stdin/stdout as hex, never argv,
//!   environment variables, config files, SQLite, logs, errors, or evidence
//! - generic credentials persist as `CRED_PERSIST_LOCAL_MACHINE`; roaming
//!   persistence classes are never requested
//! - there is no plaintext fallback; every unavailable or failed path stays
//!   fail closed
//! - on non-Windows hosts probe reports `Unavailable` and mutations fail closed

use crate::error::SecretError;
use crate::material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};
use crate::store::{SecretStore, SecretStoreAvailability, SecretStoreClass};
use std::io::{Read, Write};
use std::process::{Command, Stdio};

const SECRET_REF_PREFIX: &str = "ssv1:wincred";

/// Fixed probe target. It is never written, only read, so probe stays
/// non-mutating while still proving the helper pipeline can execute.
const PROBE_TARGET: &str = "ssv1:wincred/application/cognitiveos-secret-store-probe";

/// `CRED_MAX_CREDENTIAL_BLOB_SIZE` for generic credentials (5 * 512 bytes).
const MAX_CREDENTIAL_BLOB_BYTES: usize = 2560;

/// Helper exit codes shared with the embedded PowerShell script.
const EXIT_OK: i32 = 0;
const EXIT_NOT_FOUND: i32 = 3;
const EXIT_INVALID_INPUT: i32 = 10;

/// C# P/Invoke surface compiled by `Add-Type` inside Windows PowerShell.
///
/// The type definition is a fixed constant: no attribute, label, or secret
/// value is ever interpolated into it. Exit-code contract: `0` success,
/// `3` not found (`ERROR_NOT_FOUND` 1168), `10` invalid input, `11` API
/// failure.
const CREDENTIAL_MANAGER_TYPE_DEFINITION: &str = r#"
using System;
using System.Runtime.InteropServices;

public static class CognitiveCred
{
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct CREDENTIAL
    {
        public uint Flags;
        public uint Type;
        public string TargetName;
        public string Comment;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastWritten;
        public uint CredentialBlobSize;
        public IntPtr CredentialBlob;
        public uint Persist;
        public uint AttributeCount;
        public IntPtr Attributes;
        public string TargetAlias;
        public string UserName;
    }

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true, EntryPoint = "CredWriteW")]
    private static extern bool CredWrite(ref CREDENTIAL credential, uint flags);

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true, EntryPoint = "CredReadW")]
    private static extern bool CredRead(string target, uint type, uint flags, out IntPtr credentialPtr);

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true, EntryPoint = "CredDeleteW")]
    private static extern bool CredDelete(string target, uint type, uint flags);

    [DllImport("advapi32.dll", EntryPoint = "CredFree")]
    private static extern void CredFree(IntPtr buffer);

    public static int Write(string target, string userName, byte[] blob)
    {
        if (blob == null || blob.Length == 0 || blob.Length > 2560) { return 10; }
        IntPtr blobPtr = Marshal.AllocHGlobal(blob.Length);
        try
        {
            Marshal.Copy(blob, 0, blobPtr, blob.Length);
            CREDENTIAL credential = new CREDENTIAL();
            credential.Flags = 0;
            credential.Type = 1;
            credential.TargetName = target;
            credential.Comment = null;
            credential.CredentialBlobSize = (uint)blob.Length;
            credential.CredentialBlob = blobPtr;
            credential.Persist = 2;
            credential.AttributeCount = 0;
            credential.Attributes = IntPtr.Zero;
            credential.TargetAlias = null;
            credential.UserName = userName;
            if (!CredWrite(ref credential, 0)) { return 11; }
            return 0;
        }
        finally
        {
            for (int index = 0; index < blob.Length; index++) { Marshal.WriteByte(blobPtr, index, 0); }
            Marshal.FreeHGlobal(blobPtr);
        }
    }

    public static int Read(string target, ref byte[] blob)
    {
        blob = null;
        IntPtr credentialPtr;
        if (!CredRead(target, 1, 0, out credentialPtr))
        {
            int lastError = Marshal.GetLastWin32Error();
            return lastError == 1168 ? 3 : 11;
        }
        try
        {
            CREDENTIAL credential = (CREDENTIAL)Marshal.PtrToStructure(credentialPtr, typeof(CREDENTIAL));
            int size = (int)credential.CredentialBlobSize;
            byte[] copied = new byte[size];
            if (size > 0) { Marshal.Copy(credential.CredentialBlob, copied, 0, size); }
            blob = copied;
            return 0;
        }
        finally { CredFree(credentialPtr); }
    }

    public static int Delete(string target)
    {
        if (!CredDelete(target, 1, 0))
        {
            int lastError = Marshal.GetLastWin32Error();
            return lastError == 1168 ? 3 : 11;
        }
        return 0;
    }
}
"#;

/// Windows Credential Manager adapter driven by a fixed PowerShell helper.
#[derive(Debug, Default)]
pub struct WindowsCredentialManagerStore;

impl WindowsCredentialManagerStore {
    /// Construct a native adapter instance.
    pub fn new() -> Self {
        Self
    }

    /// Absolute Windows PowerShell 5.1 path. PATH lookup is never used, so a
    /// planted `powershell.exe` earlier on PATH cannot intercept secrets.
    fn powershell_path() -> Option<std::path::PathBuf> {
        let system_root = std::env::var_os("SystemRoot")?;
        let path = std::path::Path::new(&system_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        if path.is_file() { Some(path) } else { None }
    }

    fn require_available(&self) -> Result<(), SecretError> {
        match self.probe()? {
            SecretStoreAvailability::Available => Ok(()),
            SecretStoreAvailability::Locked => Err(SecretError::Locked),
            SecretStoreAvailability::PromptUnavailable => Err(SecretError::PromptUnavailable),
            SecretStoreAvailability::Unavailable => Err(SecretError::Unavailable {
                reason: "Windows Credential Manager / system PowerShell is not available",
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

    /// Validates that a ref belongs to this backend and has well-formed
    /// attribute pairs; foreign or malformed refs fail closed as `NotFound`.
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
        SecretAttributes::from_pairs(pairs).map_err(|_| SecretError::NotFound)
    }

    /// Defense in depth before embedding the target into the fixed script:
    /// `SecretRef` validation already restricts the charset, but embedding is
    /// only performed for tokens that cannot escape a single-quoted literal.
    fn assert_embeddable_target(target: &str) -> Result<(), SecretError> {
        let safe = target.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'/' | b'-' | b'_' | b'.')
        });
        if safe && !target.is_empty() {
            Ok(())
        } else {
            Err(SecretError::InvalidAttributes {
                detail: "credential target has unsupported characters",
            })
        }
    }

    fn helper_script(operation_body: &str, target: &str) -> Result<String, SecretError> {
        Self::assert_embeddable_target(target)?;
        let body = operation_body.replace("{TARGET}", target);
        Ok(format!(
            "$ErrorActionPreference = 'Stop'\ntry {{\nAdd-Type -TypeDefinition @'\n{CREDENTIAL_MANAGER_TYPE_DEFINITION}\n'@\n}} catch {{ exit 11 }}\n{body}\n"
        ))
    }

    fn write_script(target: &str, label: &SecretLabel) -> Result<String, SecretError> {
        // The label may contain arbitrary printable characters, so it is
        // hex-encoded before embedding and decoded inside the helper.
        let label_hex = String::from_utf8(encode_hex(label.as_str().as_bytes())).map_err(|_| {
            SecretError::Backend {
                detail: "label hex encoding failed",
            }
        })?;
        let body = r#"try {
  $hex = [Console]::In.ReadToEnd().Trim()
  if ($hex.Length -eq 0 -or ($hex.Length % 2) -ne 0) { exit 10 }
  $blob = New-Object byte[] ($hex.Length / 2)
  for ($i = 0; $i -lt $blob.Length; $i++) { $blob[$i] = [Convert]::ToByte($hex.Substring($i * 2, 2), 16) }
  $labelHex = '{LABEL_HEX}'
  $labelBytes = New-Object byte[] ($labelHex.Length / 2)
  for ($i = 0; $i -lt $labelBytes.Length; $i++) { $labelBytes[$i] = [Convert]::ToByte($labelHex.Substring($i * 2, 2), 16) }
  $label = [System.Text.Encoding]::UTF8.GetString($labelBytes)
  $code = [CognitiveCred]::Write('{TARGET}', $label, $blob)
  for ($i = 0; $i -lt $blob.Length; $i++) { $blob[$i] = 0 }
  exit $code
} catch { exit 10 }"#
            .replace("{LABEL_HEX}", &label_hex);
        Self::helper_script(&body, target)
    }

    fn read_script(target: &str) -> Result<String, SecretError> {
        let body = r#"try {
  $blob = $null
  $code = [CognitiveCred]::Read('{TARGET}', [ref]$blob)
  if ($code -ne 0) { exit $code }
  if ($null -eq $blob) { exit 11 }
  $builder = New-Object System.Text.StringBuilder
  foreach ($b in $blob) { [void]$builder.AppendFormat('{0:x2}', $b) }
  for ($i = 0; $i -lt $blob.Length; $i++) { $blob[$i] = 0 }
  [Console]::Out.Write($builder.ToString())
  exit 0
} catch { exit 11 }"#;
        Self::helper_script(body, target)
    }

    fn delete_script(target: &str) -> Result<String, SecretError> {
        let body = r#"try {
  exit [CognitiveCred]::Delete('{TARGET}')
} catch { exit 11 }"#;
        Self::helper_script(body, target)
    }

    fn helper_command(script: &str) -> Result<Command, SecretError> {
        let Some(powershell) = Self::powershell_path() else {
            return Err(SecretError::Unavailable {
                reason: "system Windows PowerShell is not present at its fixed path",
            });
        };
        let mut command = Command::new(powershell);
        command
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-EncodedCommand")
            .arg(encode_command_base64(script));
        Ok(command)
    }

    fn map_helper_failure(status_code: Option<i32>) -> SecretError {
        match status_code {
            Some(EXIT_NOT_FOUND) => SecretError::NotFound,
            Some(EXIT_INVALID_INPUT) => SecretError::Backend {
                detail: "windows credential helper rejected its input",
            },
            _ => SecretError::Backend {
                detail: "windows credential helper failed",
            },
        }
    }
}

impl SecretStore for WindowsCredentialManagerStore {
    fn class(&self) -> SecretStoreClass {
        SecretStoreClass::Native
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        if !cfg!(target_os = "windows") {
            return Ok(SecretStoreAvailability::Unavailable);
        }
        if Self::powershell_path().is_none() {
            return Ok(SecretStoreAvailability::Unavailable);
        }
        // A read of the fixed absent probe target proves the helper pipeline
        // (PowerShell start, Add-Type compilation, CredRead call) works in
        // this session without creating or mutating any credential.
        let Ok(script) = Self::read_script(PROBE_TARGET) else {
            return Ok(SecretStoreAvailability::Unavailable);
        };
        let Ok(mut command) = Self::helper_command(&script) else {
            return Ok(SecretStoreAvailability::Unavailable);
        };
        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(exit) => match exit.code() {
                Some(EXIT_OK) | Some(EXIT_NOT_FOUND) => Ok(SecretStoreAvailability::Available),
                _ => Ok(SecretStoreAvailability::Unavailable),
            },
            Err(_) => Ok(SecretStoreAvailability::Unavailable),
        }
    }

    fn put(
        &self,
        label: &SecretLabel,
        attributes: &SecretAttributes,
        material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        self.require_available()?;
        if material.len() > MAX_CREDENTIAL_BLOB_BYTES {
            return Err(SecretError::InvalidAttributes {
                detail: "secret material exceeds the Windows generic credential blob limit",
            });
        }
        let secret_ref = Self::encode_secret_ref(attributes)?;
        let script = Self::write_script(secret_ref.as_str(), label)?;
        let mut command = Self::helper_command(&script)?;
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| SecretError::Backend {
                detail: "failed to spawn windows credential helper",
            })?;
        {
            let stdin = child.stdin.as_mut().ok_or(SecretError::Backend {
                detail: "windows credential helper stdin unavailable",
            })?;
            let mut material_hex = encode_hex(material.expose_bytes());
            let write_result = stdin.write_all(&material_hex);
            wipe_bytes(&mut material_hex);
            write_result.map_err(|_| SecretError::Backend {
                detail: "failed to write secret material to windows credential helper stdin",
            })?;
        }
        let status = child.wait().map_err(|_| SecretError::Backend {
            detail: "failed to wait for windows credential helper",
        })?;
        if !status.success() {
            return Err(Self::map_helper_failure(status.code()));
        }
        Ok(secret_ref)
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        self.require_available()?;
        Self::decode_secret_ref(secret_ref)?;
        let script = Self::read_script(secret_ref.as_str())?;
        let mut command = Self::helper_command(&script)?;
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| SecretError::Backend {
                detail: "failed to spawn windows credential helper",
            })?;
        let mut hex_output: Vec<u8> = Vec::new();
        if let Some(stdout) = child.stdout.as_mut() {
            stdout
                .read_to_end(&mut hex_output)
                .map_err(|_| SecretError::Backend {
                    detail: "failed to read windows credential helper output",
                })?;
        }
        let status = child.wait().map_err(|_| SecretError::Backend {
            detail: "failed to wait for windows credential helper",
        })?;
        if !status.success() {
            wipe_bytes(&mut hex_output);
            return Err(Self::map_helper_failure(status.code()));
        }
        let decoded = decode_hex(hex_output.trim_ascii());
        wipe_bytes(&mut hex_output);
        let Some(bytes) = decoded else {
            return Err(SecretError::Backend {
                detail: "windows credential helper returned malformed data",
            });
        };
        if bytes.is_empty() {
            return Err(SecretError::Backend {
                detail: "stored windows credential blob is empty",
            });
        }
        SecretMaterial::from_bytes(bytes)
    }

    fn delete(&self, secret_ref: &SecretRef) -> Result<(), SecretError> {
        self.require_available()?;
        Self::decode_secret_ref(secret_ref)?;
        let script = Self::delete_script(secret_ref.as_str())?;
        let mut command = Self::helper_command(&script)?;
        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| SecretError::Backend {
                detail: "failed to spawn windows credential helper",
            })?;
        if !status.success() {
            return Err(Self::map_helper_failure(status.code()));
        }
        Ok(())
    }
}

/// Best-effort in-place wipe of a transient buffer that carried secret bytes.
/// This mirrors `SecretMaterial::drop` and is not a formal side-channel claim.
fn wipe_bytes(bytes: &mut [u8]) {
    for byte in bytes {
        *byte = 0;
    }
}

const HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";

fn encode_hex(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX_DIGITS[(byte >> 4) as usize]);
        output.push(HEX_DIGITS[(byte & 0x0f) as usize]);
    }
    output
}

fn decode_hex(raw: &[u8]) -> Option<Vec<u8>> {
    if !raw.len().is_multiple_of(2) {
        return None;
    }
    let mut output = Vec::with_capacity(raw.len() / 2);
    let mut index = 0;
    while index < raw.len() {
        let high = hex_value(raw[index])?;
        let low = hex_value(raw[index + 1])?;
        output.push((high << 4) | low);
        index += 2;
    }
    Some(output)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Encode a helper script as the UTF-16LE base64 form that
/// `powershell.exe -EncodedCommand` expects. The script text is fixed apart
/// from charset-validated identifiers, and never contains secret material.
fn encode_command_base64(script: &str) -> String {
    let utf16_bytes: Vec<u8> = script
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();
    base64_encode(&utf16_bytes)
}

const BASE64_TABLE: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let byte_0 = u32::from(chunk[0]);
        let byte_1 = chunk.get(1).copied().map(u32::from);
        let byte_2 = chunk.get(2).copied().map(u32::from);
        let triple =
            (byte_0 << 16) | (byte_1.unwrap_or_default() << 8) | byte_2.unwrap_or_default();
        output.push(BASE64_TABLE[(triple >> 18) as usize & 63] as char);
        output.push(BASE64_TABLE[(triple >> 12) as usize & 63] as char);
        output.push(match byte_1 {
            Some(_) => BASE64_TABLE[(triple >> 6) as usize & 63] as char,
            None => '=',
        });
        output.push(match byte_2 {
            Some(_) => BASE64_TABLE[triple as usize & 63] as char,
            None => '=',
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip_is_exact() {
        let bytes: Vec<u8> = (0u8..=255).collect();
        let encoded = encode_hex(&bytes);
        let decoded = decode_hex(&encoded);
        assert_eq!(decoded.as_deref(), Some(bytes.as_slice()));
    }

    #[test]
    fn malformed_hex_is_rejected() {
        assert!(decode_hex(b"0").is_none());
        assert!(decode_hex(b"zz").is_none());
        assert!(decode_hex(b"0g").is_none());
    }

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn scripts_never_embed_unvalidated_targets() {
        let malicious = "ssv1:wincred/application/x' ; Write-Host 'leak";
        assert!(WindowsCredentialManagerStore::read_script(malicious).is_err());
        assert!(WindowsCredentialManagerStore::delete_script(malicious).is_err());
    }

    #[test]
    fn foreign_and_malformed_refs_decode_as_not_found() {
        let foreign = SecretRef::from_opaque("ssv1:fdss/application/cognitiveos-personal")
            .map(|reference| WindowsCredentialManagerStore::decode_secret_ref(&reference));
        assert!(matches!(foreign, Ok(Err(SecretError::NotFound))));

        let odd_pairs = SecretRef::from_opaque("ssv1:wincred/application")
            .map(|reference| WindowsCredentialManagerStore::decode_secret_ref(&reference));
        assert!(matches!(odd_pairs, Ok(Err(SecretError::NotFound))));
    }
}
