//! `conformance-runner`: M1 static-contract execution CLI.
//!
//! Usage: `conformance-runner [--repo-root <path>] [--out-dir <path>] [--self-check]`
//!
//! Default mode writes `conformance-report.json` (five-state results; the
//! statically decidable subset executed against deterministic reference
//! gates, behavioral layers honestly `not-run`) plus
//! `sample-profile-manifest.json` (all profiles `planned`, no conformance
//! claim) to the output directory (default
//! `artifacts/evidence/conformance/`) and prints the human summary with the
//! report file digest.
//!
//! `--self-check` additionally executes the deliberately wrong
//! implementation (schema-valid outputs, wrong behavior) and exits non-zero
//! unless the runner fails every corrupted vector
//! (`docs/standards/conformance-evidence.md` section 3; DEVELOPMENT-PLAN M1
//! acceptance 2).

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("conformance-runner error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn file_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut repo_root = PathBuf::from(".");
    let mut out_dir: Option<PathBuf> = None;
    let mut self_check_mode = false;
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
            "--self-check" => {
                self_check_mode = true;
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
    std::fs::create_dir_all(&out_dir)?;

    let vectors = cognitive_conformance::enumerate_vectors(&repo_root)?;

    if self_check_mode {
        let report = cognitive_conformance::self_check(&repo_root, &vectors)?;
        let path = out_dir.join("self-check-report.json");
        let bytes = serde_json::to_string_pretty(&report)? + "\n";
        std::fs::write(&path, &bytes)?;
        println!(
            "Self-check ({} corrupted-gate vectors, {} flipped to fail): {}",
            report.must_flip.len(),
            report.flipped_to_fail.len(),
            report.verdict
        );
        println!(
            "Self-check report: {} ({})",
            path.display(),
            file_sha256(bytes.as_bytes())
        );
        if !cognitive_conformance::self_check_passed(&report) {
            eprintln!(
                "conformance-runner: SELF-CHECK FAILED — the deliberately wrong implementation \
                 was not failed; the runner must not be trusted (conformance-evidence.md \
                 section 3)"
            );
            return Ok(ExitCode::FAILURE);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let outcomes = cognitive_conformance::execute_all(
        &repo_root,
        &vectors,
        cognitive_conformance::ImplementationKind::Reference,
    )?;
    let report = cognitive_conformance::build_report(outcomes);
    let encoding_digest = cognitive_conformance::golden_fixture_digest(&repo_root)?;
    let manifest = cognitive_conformance::sample_profile_manifest(&repo_root, &encoding_digest)?;

    let report_path = out_dir.join("conformance-report.json");
    let manifest_path = out_dir.join("sample-profile-manifest.json");
    let report_bytes = serde_json::to_string_pretty(&report)? + "\n";
    std::fs::write(&report_path, &report_bytes)?;
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;

    print!("{}", cognitive_conformance::human_summary(&report));
    println!(
        "Machine report: {} ({})",
        report_path.display(),
        file_sha256(report_bytes.as_bytes())
    );
    println!(
        "Sample profile manifest (all profiles `planned`): {}",
        manifest_path.display()
    );
    Ok(ExitCode::SUCCESS)
}
