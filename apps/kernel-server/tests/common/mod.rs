//! Shared readiness waiting for the Personal daemon integration tests.
//!
//! Every daemon integration test spawns a real `kernel-server` child and then
//! waits for it to publish its bootstrap secret before it can authenticate.
//! Each test file used to carry its own copy of that wait, so the budget was
//! whatever the file happened to declare — from an unbounded `loop` to a fixed
//! `for _ in 0..100 { sleep(20ms) }`, i.e. exactly two seconds. A Windows
//! runner that took ~2.2 s to start a healthy daemon therefore failed the
//! required check, and a stuck daemon under an unbounded `loop` failed nothing
//! at all.
//!
//! This module is the single implementation of that wait, so the budget is one
//! reviewable decision instead of a per-file accident.
#![allow(dead_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

/// Gap between readiness probes. Short enough that a fast start is not
/// noticeably delayed.
pub const READY_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Upper bound on a readiness wait.
///
/// Roughly thirty times the slowest start actually observed on a hosted
/// runner, so a healthy daemon is never failed for being slow, and still short
/// enough that a daemon which hangs while alive is reported by this wait rather
/// than by the CI job timeout. A daemon that dies instead of hanging does not
/// consume this budget at all — see [`try_wait_for_bootstrap_secret_from`].
pub const READY_TIMEOUT: Duration = Duration::from_secs(60);

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

/// Wait for `daemon` to publish a non-empty bootstrap secret.
pub fn wait_for_bootstrap_secret_from(daemon: &mut Child, runtime_root: &Path) -> String {
    try_wait_for_bootstrap_secret_from(daemon, runtime_root, READY_TIMEOUT)
        .unwrap_or_else(|diagnostic| panic!("{diagnostic}"))
}

/// [`wait_for_bootstrap_secret_from`] with an explicit budget, returning the
/// diagnostic instead of panicking so the wait itself can be tested.
///
/// A daemon that has exited will never publish anything, so it is reported as
/// soon as it is observed instead of after the whole budget: waiting out the
/// ceiling would report a timeout when the real fact is an exit status.
pub fn try_wait_for_bootstrap_secret_from(
    daemon: &mut Child,
    runtime_root: &Path,
    timeout: Duration,
) -> Result<String, String> {
    let path = bootstrap_secret_path(runtime_root);
    let subject = format!("bootstrap secret at {}", path.display());
    let resolved = poll_until(&subject, timeout, || {
        // Read before checking the process: a daemon that published and then
        // exited did become ready.
        if let Some(secret) = read_bootstrap_secret(&path) {
            return Some(Ok(secret));
        }
        match daemon.try_wait() {
            Ok(Some(status)) => Some(Err(format!(
                "{subject} will never be published: the daemon exited with {status}"
            ))),
            _ => None,
        }
    });
    match resolved {
        Ok(outcome) => outcome,
        Err(expired) => Err(expired),
    }
}

/// Wait for a bootstrap secret without a handle on the daemon process.
pub fn wait_for_bootstrap_secret(runtime_root: &Path) -> String {
    try_wait_for_bootstrap_secret(runtime_root, READY_TIMEOUT)
        .unwrap_or_else(|diagnostic| panic!("{diagnostic}"))
}

/// [`wait_for_bootstrap_secret`] with an explicit budget.
pub fn try_wait_for_bootstrap_secret(
    runtime_root: &Path,
    timeout: Duration,
) -> Result<String, String> {
    let path = bootstrap_secret_path(runtime_root);
    let subject = format!("bootstrap secret at {}", path.display());
    poll_until(&subject, timeout, || read_bootstrap_secret(&path))
}

/// Connect to a daemon that may still be binding its listener.
pub fn connect_when_ready(port: u16) -> TcpStream {
    try_connect_when_ready(port, READY_TIMEOUT).unwrap_or_else(|diagnostic| panic!("{diagnostic}"))
}

/// [`connect_when_ready`] with an explicit budget.
pub fn try_connect_when_ready(port: u16, timeout: Duration) -> Result<TcpStream, String> {
    let subject = format!("a daemon accepting connections on 127.0.0.1:{port}");
    poll_until(&subject, timeout, || {
        TcpStream::connect(("127.0.0.1", port)).ok()
    })
}

fn read_bootstrap_secret(path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(path).ok()?;
    let secret = contents.trim();
    // A reader can observe the file between create and write.
    (!secret.is_empty()).then(|| secret.to_owned())
}
