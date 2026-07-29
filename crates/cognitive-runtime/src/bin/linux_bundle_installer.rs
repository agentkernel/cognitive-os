//! Production process wrapper for the inspected Personal Linux bootstrap.
//!
//! Trust parsing and transaction orchestration live in the runtime library so
//! they can be exercised with its fixture-only controller. This wrapper keeps
//! production manager selection fixed to the product-owned constructor.

#[cfg(unix)]
use cognitive_runtime::{
    SystemdUserServiceController, install_linux_bundle_with_controller, product_deployment_root,
};
use std::env;
#[cfg(unix)]
use std::net::SocketAddr;

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match install_downloaded_bundle(&arguments) {
        Ok(()) => std::process::exit(0),
        Err(()) => {
            // Library errors can carry untrusted path text. The public process
            // boundary emits only this stable non-secret failure summary.
            eprintln!("CognitiveOS Linux bundle installation failed");
            std::process::exit(1);
        }
    }
}

#[cfg(unix)]
fn install_downloaded_bundle(arguments: &[String]) -> Result<(), ()> {
    let deployment_root = product_deployment_root().map_err(|_| ())?;
    let health_address: SocketAddr = "127.0.0.1:48181".parse().map_err(|_| ())?;
    let mut controller =
        SystemdUserServiceController::new_production(&deployment_root, health_address)
            .map_err(|_| ())?;
    let receipt =
        install_linux_bundle_with_controller(arguments, &deployment_root, &mut controller)
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
