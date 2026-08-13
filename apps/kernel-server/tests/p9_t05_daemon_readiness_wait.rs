//! P9-T05 evidence for the shared daemon readiness wait.
//!
//! A GitHub Windows runner took roughly 2.2 s to publish the bootstrap secret
//! and failed the required check, while the 151 unit tests in the same run
//! passed and a re-run of the identical revision passed. The daemon was
//! healthy; the wait was too thin. These cases pin both directions: a healthy
//! but slow start must be waited for, and a daemon that never becomes ready
//! must still fail, bounded and with a diagnostic that says why.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::net::TcpListener;
use std::process::Command;
use std::time::{Duration, Instant};

/// Longer than the 2 s budget the copied waits used and than the ~2.2 s start
/// that was observed, and far below the readiness ceiling.
const SLOW_START: Duration = Duration::from_millis(2_500);

fn runtime_root(label: &str) -> std::path::PathBuf {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let path = std::env::temp_dir().join(format!(
        "cos-p9t05-{}-{}-{}",
        label,
        std::process::id(),
        port
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Publish the secret the way the daemon does — visible only once complete —
/// so the wait cannot observe a half-written file.
fn publish_secret_after(delay: Duration, path: std::path::PathBuf, secret: &'static str) {
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let staging = path.with_extension("staging");
        std::fs::write(&staging, secret).unwrap();
        std::fs::rename(&staging, &path).unwrap();
    });
}

#[test]
fn readiness_wait_tolerates_a_start_slower_than_two_seconds() {
    let root = runtime_root("slow-start");
    publish_secret_after(
        SLOW_START,
        common::bootstrap_secret_path(&root),
        "slow-start-secret",
    );

    let started = Instant::now();
    let secret = common::wait_for_bootstrap_secret(&root);
    let waited = started.elapsed();

    assert_eq!(secret, "slow-start-secret");
    assert!(
        waited >= SLOW_START,
        "the wait returned after {} ms, before the secret was published",
        waited.as_millis()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn readiness_wait_fails_closed_with_a_diagnostic_when_the_secret_never_appears() {
    let root = runtime_root("never-ready");
    let budget = Duration::from_millis(300);

    let started = Instant::now();
    let outcome = common::try_wait_for_bootstrap_secret(&root, budget);
    let waited = started.elapsed();

    let diagnostic =
        outcome.expect_err("a daemon that never publishes a secret must fail the wait");
    assert!(
        diagnostic.contains("local-bootstrap.secret"),
        "{diagnostic}"
    );
    assert!(diagnostic.contains("within 300 ms"), "{diagnostic}");
    assert!(
        waited >= budget,
        "the wait gave up after {} ms, short of its own budget",
        waited.as_millis()
    );
    assert!(
        waited < budget * 20,
        "the wait ran {} ms past a {} ms budget",
        waited.as_millis(),
        budget.as_millis()
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A generous ceiling must not turn a dead daemon into a slow one. `--serve`
/// refuses a non-loopback bind before it does any work, so this child is gone
/// immediately and can never publish anything.
#[test]
fn readiness_wait_reports_an_exited_daemon_instead_of_consuming_the_ceiling() {
    let root = runtime_root("exited-daemon");
    let mut daemon = Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args(["--serve", "--bind", "203.0.113.1:9"])
        .spawn()
        .unwrap();

    let started = Instant::now();
    let outcome =
        common::try_wait_for_bootstrap_secret_from(&mut daemon, &root, common::READY_TIMEOUT);
    let waited = started.elapsed();

    let diagnostic = outcome.expect_err("a daemon that exited must fail the wait");
    assert!(
        diagnostic.contains("the daemon exited with"),
        "{diagnostic}"
    );
    assert!(
        waited < common::READY_TIMEOUT / 4,
        "the wait spent {} ms of a {} ms ceiling on a daemon that was already gone",
        waited.as_millis(),
        common::READY_TIMEOUT.as_millis()
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = std::fs::remove_dir_all(&root);
}
