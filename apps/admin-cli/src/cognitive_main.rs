//! `cognitive` — CognitiveOS Personal product CLI entry (P1-T06).
//!
//! Hard rules:
//! - non-authority client only (layout/config + daemon HTTP projections)
//! - Provider API keys never enter config, SQLite, env, argv, logs, or evidence
//! - does not claim G0 / B01-B12 / Profile conformance

use admin_cli::personal_cli::{
    COGNITIVE_USAGE, EXIT_USAGE, parse_cognitive_args, run_cognitive_command,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        eprintln!("{COGNITIVE_USAGE}");
        std::process::exit(if args.is_empty() { EXIT_USAGE } else { 0 });
    }
    match parse_cognitive_args(&args) {
        Ok(command) => std::process::exit(run_cognitive_command(command)),
        Err(message) => {
            eprintln!("error: {message}\n\n{COGNITIVE_USAGE}");
            std::process::exit(EXIT_USAGE);
        }
    }
}
