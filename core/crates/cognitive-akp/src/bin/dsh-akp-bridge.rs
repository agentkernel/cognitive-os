//! JSONL stdio bridge for DeepSeek Harness plugin events.
//!
//! This binary is a candidate-only translator. It never writes authority
//! SQLite, never holds a Provider key, and never treats a dsh response as
//! Task completion. The daemon remains the only authority writer.

use cognitive_akp::deepseek_harness::{
    DeepSeekHarnessAdapter, MAX_FRAME_BYTES, default_config, handle_jsonl_line,
};
use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(3);
        }
    }
}

fn run() -> Result<(), String> {
    let mut dsh_version = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dsh-version" => {
                dsh_version = Some(
                    args.next()
                        .ok_or_else(|| "missing --dsh-version value".to_owned())?,
                );
            }
            other => return Err(format!("unsupported argument `{other}`")),
        }
    }
    let version = dsh_version
        .unwrap_or_else(|| cognitive_akp::deepseek_harness::PINNED_DSH_REVISION.to_owned());
    if version.trim().is_empty() {
        return Err("dsh version pin is empty".to_owned());
    }
    let mut adapter =
        DeepSeekHarnessAdapter::new(default_config(version)).map_err(|error| error.to_string())?;
    adapter
        .activate("dsh-stdio")
        .map_err(|error| error.to_string())?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("stdin read failed: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_jsonl_line(&mut adapter, &line, MAX_FRAME_BYTES);
        let encoded = serde_json::to_string(&response)
            .map_err(|error| format!("response serialization failed: {error}"))?;
        writeln!(stdout, "{encoded}").map_err(|error| format!("stdout write failed: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("stdout flush failed: {error}"))?;
    }
    Ok(())
}
