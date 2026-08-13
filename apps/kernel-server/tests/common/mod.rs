//! Shared readiness waiting for the Personal daemon integration tests.
//!
//! Every daemon integration test spawns a real `kernel-server` child and then
//! waits for it to publish its bootstrap secret before it can authenticate.
//! Each test file used to carry its own copy of that wait, so the budget was
//! whatever the file happened to declare — from an unbounded `loop` to a fixed
//! `for _ in 0..100 { sleep(20ms) }`, i.e. exactly two seconds.
//!
//! This module is the single implementation of that wait so the budget is one
//! reviewable decision instead of a per-file accident.
#![allow(dead_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Gap between readiness probes.
pub const READY_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Upper bound on a readiness wait: the 100 probes at [`READY_POLL_INTERVAL`]
/// that the copied implementations budgeted.
pub const READY_TIMEOUT: Duration = Duration::from_millis(2_000);

/// The daemon publishes its bootstrap secret here, relative to its runtime root.
pub fn bootstrap_secret_path(runtime_root: &Path) -> PathBuf {
    runtime_root
        .join("cognitiveos")
        .join("local-bootstrap.secret")
}

/// Probe until `probe` yields a value or `timeout` elapses.
///
/// The error is the test's failure message, so it names the subject, the
/// budget, and how much of it was actually consumed.
pub fn poll_until<T>(
    subject: &str,
    timeout: Duration,
    mut probe: impl FnMut() -> Option<T>,
) -> Result<T, String> {
    let started = Instant::now();
    let mut probes = 0_u32;
    loop {
        probes += 1;
        if let Some(observed) = probe() {
            return Ok(observed);
        }
        let waited = started.elapsed();
        if waited >= timeout {
            return Err(format!(
                "{subject} did not become ready within {} ms ({probes} probes over {} ms)",
                timeout.as_millis(),
                waited.as_millis()
            ));
        }
        std::thread::sleep(READY_POLL_INTERVAL.min(timeout - waited));
    }
}

/// Wait for the daemon to publish a non-empty bootstrap secret.
pub fn wait_for_bootstrap_secret(runtime_root: &Path) -> String {
    try_wait_for_bootstrap_secret(runtime_root, READY_TIMEOUT)
        .unwrap_or_else(|diagnostic| panic!("{diagnostic}"))
}

/// [`wait_for_bootstrap_secret`] with an explicit budget, returning the
/// diagnostic instead of panicking so the wait itself can be tested.
pub fn try_wait_for_bootstrap_secret(
    runtime_root: &Path,
    timeout: Duration,
) -> Result<String, String> {
    let path = bootstrap_secret_path(runtime_root);
    let subject = format!("bootstrap secret at {}", path.display());
    poll_until(&subject, timeout, || {
        let contents = std::fs::read_to_string(&path).ok()?;
        let secret = contents.trim();
        // A reader can observe the file between create and write.
        (!secret.is_empty()).then(|| secret.to_owned())
    })
}
