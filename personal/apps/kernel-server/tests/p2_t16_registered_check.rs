#![allow(clippy::expect_used)]

use std::process::Command;

#[test]
fn fixed_c2a_worker_runs_without_shell_or_ambient_environment() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/p2_t16_registered_check/c2a-repair-typescript");
    let output = Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args([
            "--personal-registered-check-worker",
            "c2a.repair.typescript",
        ])
        .current_dir(fixture)
        .env_clear()
        .output()
        .expect("执行固定登记 helper");

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("helper 输出 UTF-8");
    assert!(stdout.contains("\"passed\":true"), "{stdout}");
}

#[test]
fn fixed_worker_rejects_extra_argv_before_execution() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/p2_t16_registered_check/c2a-repair-typescript");
    let output = Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args([
            "--personal-registered-check-worker",
            "c2a.repair.typescript",
            "--injected",
        ])
        .current_dir(fixture)
        .env_clear()
        .output()
        .expect("执行拒绝路径");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn fixed_rust_worker_runs_without_shell_or_ambient_environment() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/p2_t16_registered_check/c2a-repair-rust");
    let output = Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args(["--personal-registered-check-worker", "c2a.repair.rust"])
        .current_dir(fixture)
        .env_clear()
        .output()
        .expect("执行 Rust 固定登记 helper");

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("helper 输出 UTF-8");
    assert!(stdout.contains("\"passed\":true"), "{stdout}");
}
