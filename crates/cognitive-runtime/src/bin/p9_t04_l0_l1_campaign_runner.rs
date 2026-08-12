//! Redacted L0/L1 campaign runner entry point for P9-T04.
//!
//! L0 admits or refuses measurement; L1 delegates to the existing deterministic
//! module benchmark. The process holds no credential, contacts no Provider, and
//! opens no authority store, so its output stays a hypothesis-level observation.

use cognitive_runtime::campaign_runner::{
    CampaignRunRequest, build_campaign_runner_report, parse_campaign_run_request,
};
use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Variables the child benchmark needs to locate its toolchain, home and
/// temporary fixture root. Every other inherited variable is dropped.
const FORWARDED_CHILD_ENVIRONMENT: [&str; 7] = [
    "PATH",
    "HOME",
    "LD_LIBRARY_PATH",
    "TMPDIR",
    "TEMP",
    "TMP",
    "SYSTEMROOT",
];

fn main() -> Result<(), Box<dyn Error>> {
    let l0_started_at = Instant::now();
    let request =
        parse_campaign_run_request(env::args().skip(1), env::vars().map(|(name, _)| name))?;
    let l0_elapsed_nanos = l0_started_at.elapsed().as_nanos();

    let l1_started_at = Instant::now();
    let module_observation = run_module_benchmark(&request)?;
    let l1_elapsed_nanos = l1_started_at.elapsed().as_nanos();

    let report = build_campaign_runner_report(
        request,
        l0_elapsed_nanos,
        l1_elapsed_nanos,
        module_observation,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn run_module_benchmark(request: &CampaignRunRequest) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut command = Command::new(sibling_executable("p7_t04_module_benchmark")?);
    command
        .args(["--source-revision", &request.source_revision])
        .env_clear()
        .env(
            "COGNITIVEOS_BENCHMARK_SAMPLES",
            request.sample_count.to_string(),
        );
    for name in FORWARDED_CHILD_ENVIRONMENT {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(io::Error::other("L1 module benchmark did not complete").into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn sibling_executable(name: &str) -> Result<PathBuf, io::Error> {
    let executable_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    Ok(env::current_exe()?
        .parent()
        .ok_or_else(|| io::Error::other("campaign executable has no parent directory"))?
        .join(executable_name))
}
