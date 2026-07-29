//! Production adapter for the inspected Personal Linux bootstrap.
//!
//! The shell owns only bounded download and this executable's digest check.
//! This adapter constructs the product-owned keyring and delegates verified
//! staging, canonical unit publication, service compensation and receipt
//! creation to `cognitive-runtime`.

#[cfg(unix)]
use cognitive_runtime::{
    ExpectedPiCompatibility, SystemdUserServiceController, TrustedKeyInput, TrustedKeyStatus,
    TrustedKeyring, install_linux_bundle_single_service,
};
use std::env;
#[cfg(unix)]
use std::net::SocketAddr;
use std::path::PathBuf;

const USAGE: &str = "linux-bundle-installer --bundle-directory <directory> --expected-pi-version <version> --expected-pi-integrity <integrity> --keyring-version <version> --key-id <id> --public-key-base64url <key>";

#[derive(Debug, Default)]
struct InstallerArguments {
    bundle_directory: Option<PathBuf>,
    expected_pi_version: Option<String>,
    expected_pi_integrity: Option<String>,
    keyring_version: Option<String>,
    key_id: Option<String>,
    public_key_base64url: Option<String>,
}

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match install_downloaded_bundle(&arguments) {
        Ok(()) => std::process::exit(0),
        Err(()) => {
            // Typed library errors may contain untrusted path text. The public
            // adapter emits only a stable non-secret failure summary.
            eprintln!("CognitiveOS Linux bundle installation failed");
            std::process::exit(1);
        }
    }
}

#[cfg(unix)]
fn install_downloaded_bundle(arguments: &[String]) -> Result<(), ()> {
    let parsed_arguments = parse_arguments(arguments).map_err(|message| {
        eprintln!("{message}");
    })?;
    let bundle_directory = parsed_arguments.bundle_directory.ok_or(())?;
    let expected_pi = ExpectedPiCompatibility::new(
        parsed_arguments.expected_pi_version.ok_or(())?,
        parsed_arguments.expected_pi_integrity.ok_or(())?,
    );
    let trusted_keyring = TrustedKeyring::new(
        parsed_arguments.keyring_version.ok_or(())?,
        vec![TrustedKeyInput {
            key_id: parsed_arguments.key_id.ok_or(())?,
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: parsed_arguments.public_key_base64url.ok_or(())?,
            status: TrustedKeyStatus::Active,
        }],
    )
    .map_err(|_| ())?;
    let deployment_root = product_deployment_root()?;
    let health_address: SocketAddr = "127.0.0.1:48181".parse().map_err(|_| ())?;
    let mut controller =
        SystemdUserServiceController::new_production(&deployment_root, health_address)
            .map_err(|_| ())?;
    let receipt = install_linux_bundle_single_service(
        &bundle_directory,
        &deployment_root,
        &expected_pi,
        &trusted_keyring,
        &mut controller,
    )
    .map_err(|_| ())?;

    println!(
        "installed-cognitiveos-personal version={} previous={} service=cognitiveos-personal.service port=48181 trusted-key-id={} keyring-version={}",
        receipt.installed_version,
        receipt.previous_active_version.as_deref().unwrap_or("none"),
        receipt.trusted_key_id,
        receipt.trusted_keyring_version,
    );
    Ok(())
}

#[cfg(not(unix))]
fn install_downloaded_bundle(_arguments: &[String]) -> Result<(), ()> {
    Err(())
}

#[cfg(unix)]
fn product_deployment_root() -> Result<PathBuf, ()> {
    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        let xdg_data_home = PathBuf::from(xdg_data_home);
        if xdg_data_home.is_absolute() {
            return Ok(xdg_data_home.join("cognitiveos/deployment"));
        }
        return Err(());
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|home_directory| home_directory.join(".local/share/cognitiveos/deployment"))
        .ok_or(())
}

fn parse_arguments(arguments: &[String]) -> Result<InstallerArguments, &'static str> {
    let mut parsed_arguments = InstallerArguments::default();
    let mut argument_index = 0;
    while argument_index < arguments.len() {
        let flag = &arguments[argument_index];
        let value = arguments
            .get(argument_index + 1)
            .ok_or("Linux bundle installer argument is missing a value")?
            .clone();
        argument_index += 2;
        match flag.as_str() {
            "--bundle-directory" => {
                if parsed_arguments
                    .bundle_directory
                    .replace(PathBuf::from(value))
                    .is_some()
                {
                    return Err("Linux bundle installer received a duplicate argument");
                }
            }
            "--expected-pi-version" => {
                set_once(&mut parsed_arguments.expected_pi_version, value)?;
            }
            "--expected-pi-integrity" => {
                set_once(&mut parsed_arguments.expected_pi_integrity, value)?;
            }
            "--keyring-version" => set_once(&mut parsed_arguments.keyring_version, value)?,
            "--key-id" => set_once(&mut parsed_arguments.key_id, value)?,
            "--public-key-base64url" => {
                set_once(&mut parsed_arguments.public_key_base64url, value)?;
            }
            "--help" => return Err(USAGE),
            _ => return Err("Linux bundle installer received an unsupported argument"),
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
        return Err("Linux bundle installer received a duplicate argument");
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
        let unknown_arguments = vec!["--systemctl".to_owned(), "custom".to_owned()];
        assert!(parse_arguments(&unknown_arguments).is_err());
    }
}
