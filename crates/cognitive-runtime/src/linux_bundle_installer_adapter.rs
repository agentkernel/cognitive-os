//! Product-owned orchestration for the inspected Linux installer adapter.
//!
//! The runner intentionally receives a controller rather than manager
//! configuration. Production constructs its fixed controller separately;
//! isolated tests may use the existing fixture controller without widening the
//! public bootstrap command line.

use crate::{
    ExpectedPiCompatibility, LinuxBundleServiceError, LinuxBundleServiceReceipt,
    LinuxBundleSingleServiceController, TrustedKeyInput, TrustedKeyStatus, TrustedKeyring,
    install_linux_bundle_single_service, verify_linux_bundle_for_release,
};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxBundleInstallerAdapterError {
    InvalidArguments,
    Verification,
    Service,
    DeploymentRoot,
}

#[derive(Debug, Default)]
struct InstallerArguments {
    bundle_directory: Option<PathBuf>,
    expected_release_version: Option<String>,
    expected_pi_version: Option<String>,
    expected_pi_integrity: Option<String>,
    keyring_version: Option<String>,
    key_id: Option<String>,
    public_key_base64url: Option<String>,
}

/// Resolve the product-owned data root from real user XDG conventions.
pub fn product_deployment_root() -> Result<PathBuf, LinuxBundleInstallerAdapterError> {
    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        let xdg_data_home = PathBuf::from(xdg_data_home);
        return xdg_data_home
            .is_absolute()
            .then(|| xdg_data_home.join("cognitiveos/deployment"))
            .ok_or(LinuxBundleInstallerAdapterError::DeploymentRoot);
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home_directory| home_directory.is_absolute())
        .map(|home_directory| home_directory.join(".local/share/cognitiveos/deployment"))
        .ok_or(LinuxBundleInstallerAdapterError::DeploymentRoot)
}

/// Execute the release-bound installer transaction with a preconstructed,
/// product-constrained lifecycle controller.
pub fn install_linux_bundle_with_controller(
    arguments: &[String],
    deployment_root: &Path,
    service_controller: &mut impl LinuxBundleSingleServiceController,
) -> Result<LinuxBundleServiceReceipt, LinuxBundleInstallerAdapterError> {
    let parsed_arguments = parse_arguments(arguments)?;
    let bundle_directory = parsed_arguments
        .bundle_directory
        .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?;
    let expected_release_version = parsed_arguments
        .expected_release_version
        .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?;
    let expected_pi = ExpectedPiCompatibility::new(
        parsed_arguments
            .expected_pi_version
            .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?,
        parsed_arguments
            .expected_pi_integrity
            .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?,
    );
    let trusted_keyring = TrustedKeyring::new(
        parsed_arguments
            .keyring_version
            .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?,
        vec![TrustedKeyInput {
            key_id: parsed_arguments
                .key_id
                .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?,
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: parsed_arguments
                .public_key_base64url
                .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?,
            status: TrustedKeyStatus::Active,
        }],
    )
    .map_err(|_| LinuxBundleInstallerAdapterError::Verification)?;
    verify_linux_bundle_for_release(
        &bundle_directory,
        &expected_release_version,
        &expected_pi,
        &trusted_keyring,
    )
    .map_err(|_| LinuxBundleInstallerAdapterError::Verification)?;
    install_linux_bundle_single_service(
        &bundle_directory,
        deployment_root,
        &expected_pi,
        &trusted_keyring,
        service_controller,
    )
    .map_err(map_service_error)
}

fn map_service_error(_error: LinuxBundleServiceError) -> LinuxBundleInstallerAdapterError {
    LinuxBundleInstallerAdapterError::Service
}

fn parse_arguments(
    arguments: &[String],
) -> Result<InstallerArguments, LinuxBundleInstallerAdapterError> {
    let mut parsed_arguments = InstallerArguments::default();
    let mut argument_index = 0;
    while argument_index < arguments.len() {
        let flag = &arguments[argument_index];
        let value = arguments
            .get(argument_index + 1)
            .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)?
            .clone();
        argument_index += 2;
        match flag.as_str() {
            "--bundle-directory" => set_path_once(&mut parsed_arguments.bundle_directory, value)?,
            "--expected-release-version" => {
                set_once(&mut parsed_arguments.expected_release_version, value)?
            }
            "--expected-pi-version" => set_once(&mut parsed_arguments.expected_pi_version, value)?,
            "--expected-pi-integrity" => {
                set_once(&mut parsed_arguments.expected_pi_integrity, value)?
            }
            "--keyring-version" => set_once(&mut parsed_arguments.keyring_version, value)?,
            "--key-id" => set_once(&mut parsed_arguments.key_id, value)?,
            "--public-key-base64url" => {
                set_once(&mut parsed_arguments.public_key_base64url, value)?
            }
            _ => return Err(LinuxBundleInstallerAdapterError::InvalidArguments),
        }
    }
    let has_all_required_arguments = parsed_arguments.bundle_directory.is_some()
        && parsed_arguments.expected_release_version.is_some()
        && parsed_arguments.expected_pi_version.is_some()
        && parsed_arguments.expected_pi_integrity.is_some()
        && parsed_arguments.keyring_version.is_some()
        && parsed_arguments.key_id.is_some()
        && parsed_arguments.public_key_base64url.is_some();
    has_all_required_arguments
        .then_some(parsed_arguments)
        .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)
}

fn set_once(
    target: &mut Option<String>,
    value: String,
) -> Result<(), LinuxBundleInstallerAdapterError> {
    target
        .replace(value)
        .is_none()
        .then_some(())
        .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)
}

fn set_path_once(
    target: &mut Option<PathBuf>,
    value: String,
) -> Result<(), LinuxBundleInstallerAdapterError> {
    target
        .replace(PathBuf::from(value))
        .is_none()
        .then_some(())
        .ok_or(LinuxBundleInstallerAdapterError::InvalidArguments)
}

#[cfg(test)]
mod tests {
    use super::{LinuxBundleInstallerAdapterError, parse_arguments};

    #[test]
    fn rejects_duplicate_and_unknown_arguments() {
        let duplicate_arguments = vec![
            "--bundle-directory".to_owned(),
            "bundle".to_owned(),
            "--bundle-directory".to_owned(),
            "other".to_owned(),
        ];
        assert!(matches!(
            parse_arguments(&duplicate_arguments),
            Err(LinuxBundleInstallerAdapterError::InvalidArguments)
        ));
        let unknown_arguments = vec!["--systemctl".to_owned(), "custom".to_owned()];
        assert!(matches!(
            parse_arguments(&unknown_arguments),
            Err(LinuxBundleInstallerAdapterError::InvalidArguments)
        ));
    }
}
