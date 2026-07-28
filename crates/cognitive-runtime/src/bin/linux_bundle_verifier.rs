//! Narrow executable adapter for the inspected Linux bootstrap.
//!
//! This program performs no network, service-management, deployment, or
//! authority operation. It constructs the product-selected verification inputs
//! supplied by the inspected bootstrap and delegates all bundle semantics to
//! `cognitive_runtime::verify_linux_bundle`.

use cognitive_runtime::{
    ExpectedPiCompatibility, TrustedKeyInput, TrustedKeyStatus, TrustedKeyring, verify_linux_bundle,
};
use std::env;
use std::path::PathBuf;

const USAGE: &str = "linux-bundle-verifier --bundle-directory <directory> --expected-pi-version <version> --expected-pi-integrity <integrity> --keyring-version <version> --key-id <id> --public-key-base64url <key>";

#[derive(Debug, Default)]
struct VerifierArguments {
    bundle_directory: Option<PathBuf>,
    expected_pi_version: Option<String>,
    expected_pi_integrity: Option<String>,
    keyring_version: Option<String>,
    key_id: Option<String>,
    public_key_base64url: Option<String>,
}

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match verify_downloaded_bundle(&arguments) {
        Ok(()) => std::process::exit(0),
        Err(()) => {
            // The verifier error is deliberately not rendered because it can
            // contain untrusted filenames. The library remains the detailed,
            // typed semantic boundary for callers that need it.
            eprintln!("Linux bundle verification failed");
            std::process::exit(1);
        }
    }
}

fn verify_downloaded_bundle(arguments: &[String]) -> Result<(), ()> {
    let parsed_arguments = parse_arguments(arguments).map_err(|message| {
        eprintln!("{message}");
    })?;

    let bundle_directory = parsed_arguments.bundle_directory.ok_or(())?;
    let expected_pi_version = parsed_arguments.expected_pi_version.ok_or(())?;
    let expected_pi_integrity = parsed_arguments.expected_pi_integrity.ok_or(())?;
    let keyring_version = parsed_arguments.keyring_version.ok_or(())?;
    let key_id = parsed_arguments.key_id.ok_or(())?;
    let public_key_base64url = parsed_arguments.public_key_base64url.ok_or(())?;

    let expected_pi = ExpectedPiCompatibility::new(expected_pi_version, expected_pi_integrity);
    let trusted_keyring = TrustedKeyring::new(
        keyring_version,
        vec![TrustedKeyInput {
            key_id,
            algorithm: "Ed25519".to_owned(),
            public_key_base64url,
            status: TrustedKeyStatus::Active,
        }],
    )
    .map_err(|_| ())?;

    let verified_bundle =
        verify_linux_bundle(&bundle_directory, &expected_pi, &trusted_keyring).map_err(|_| ())?;

    println!(
        "verified-linux-bundle version={} trusted-key-id={} keyring-version={}",
        verified_bundle.manifest().version,
        verified_bundle.trusted_key_id(),
        trusted_keyring.version()
    );
    Ok(())
}

fn parse_arguments(arguments: &[String]) -> Result<VerifierArguments, &'static str> {
    let mut parsed_arguments = VerifierArguments::default();
    let mut argument_index = 0;

    while argument_index < arguments.len() {
        let flag = &arguments[argument_index];
        let value = arguments
            .get(argument_index + 1)
            .ok_or("Linux bundle verifier argument is missing a value")?
            .clone();
        argument_index += 2;

        match flag.as_str() {
            "--bundle-directory" => {
                if parsed_arguments
                    .bundle_directory
                    .replace(PathBuf::from(value))
                    .is_some()
                {
                    return Err("Linux bundle verifier received a duplicate argument");
                }
            }
            "--expected-pi-version" => {
                set_once(&mut parsed_arguments.expected_pi_version, value)?;
            }
            "--expected-pi-integrity" => {
                set_once(&mut parsed_arguments.expected_pi_integrity, value)?;
            }
            "--keyring-version" => {
                set_once(&mut parsed_arguments.keyring_version, value)?;
            }
            "--key-id" => {
                set_once(&mut parsed_arguments.key_id, value)?;
            }
            "--public-key-base64url" => {
                set_once(&mut parsed_arguments.public_key_base64url, value)?;
            }
            "--help" => return Err(USAGE),
            _ => return Err("Linux bundle verifier received an unsupported argument"),
        }
    }

    if parsed_arguments.bundle_directory.is_none()
        || parsed_arguments.expected_pi_version.is_none()
        || parsed_arguments.expected_pi_integrity.is_none()
        || parsed_arguments.keyring_version.is_none()
        || parsed_arguments.key_id.is_none()
        || parsed_arguments.public_key_base64url.is_none()
    {
        return Err(USAGE);
    }
    Ok(parsed_arguments)
}

fn set_once(target: &mut Option<String>, value: String) -> Result<(), &'static str> {
    if target.replace(value).is_some() {
        return Err("Linux bundle verifier received a duplicate argument");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_arguments;

    #[test]
    fn rejects_duplicate_or_unknown_arguments() {
        let duplicate_arguments = vec![
            "--bundle-directory".to_owned(),
            "bundle".to_owned(),
            "--bundle-directory".to_owned(),
            "other".to_owned(),
        ];
        assert!(parse_arguments(&duplicate_arguments).is_err());

        let unknown_arguments = vec!["--network".to_owned(), "enabled".to_owned()];
        assert!(parse_arguments(&unknown_arguments).is_err());
    }
}
