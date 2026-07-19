//! `conformance-runner`: M0 enumerate-only CLI.
//!
//! Usage: `conformance-runner [--repo-root <path>] [--out-dir <path>]`
//!
//! Writes `conformance-report.json` and `sample-profile-manifest.json` to the
//! output directory (default `artifacts/evidence/conformance/`) and prints
//! the human-readable summary. Every vector is reported `not-run`; execution
//! capability is an M1 deliverable.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("conformance-runner error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut repo_root = PathBuf::from(".");
    let mut out_dir: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().ok_or("--repo-root requires a value")?);
            }
            "--out-dir" => {
                out_dir = Some(PathBuf::from(
                    args.next().ok_or("--out-dir requires a value")?,
                ));
            }
            other => return Err(format!("unknown argument `{other}`").into()),
        }
    }
    let out_dir = out_dir.unwrap_or_else(|| {
        repo_root
            .join("artifacts")
            .join("evidence")
            .join("conformance")
    });

    let vectors = cognitive_conformance::enumerate_vectors(&repo_root)?;
    let report = cognitive_conformance::build_report(vectors);
    let encoding_digest = cognitive_conformance::golden_fixture_digest(&repo_root)?;
    let manifest = cognitive_conformance::sample_profile_manifest(&repo_root, &encoding_digest)?;

    std::fs::create_dir_all(&out_dir)?;
    let report_path = out_dir.join("conformance-report.json");
    let manifest_path = out_dir.join("sample-profile-manifest.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)? + "\n")?;
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;

    print!("{}", cognitive_conformance::human_summary(&report));
    println!("Machine report: {}", report_path.display());
    println!(
        "Sample profile manifest (all profiles `planned`): {}",
        manifest_path.display()
    );
    Ok(())
}
