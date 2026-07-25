//! Personal product CLI surface (`cognitive` bin, P1-T06).
//!
//! This module is a **client**: it prepares XDG layout / non-secret Provider
//! config, launches the Personal daemon process, and consumes authenticated
//! readiness projections over HTTP. It never opens SQLite authority tables to
//! advance Task/Effect/Verification state, never logs secret material, and
//! never claims G0 / B01-B12 / Profile conformance.

mod client;
mod daemon;
mod init;
mod layout;
mod secret_input;
mod url;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

pub use init::run_init;
pub use layout::{LayoutRoots, resolve_layout_roots};

/// Top-level `cognitive` verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CognitiveCommand {
    Init(InitOptions),
    Status(StatusOptions),
    Doctor(StatusOptions),
    Daemon(DaemonCommand),
}

/// Options for `cognitive init`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    pub layout_roots: LayoutRoots,
    pub provider_id: Option<String>,
    pub base_url: Option<String>,
    pub model_id: Option<String>,
    pub api_key_file: Option<PathBuf>,
    /// When true, allow the ephemeral SecretStore test double (tests only).
    pub allow_ephemeral_secret_backend: bool,
    /// When true, rotate an already-configured provider key.
    pub rotate_key: bool,
}

/// Options for `cognitive status` / `cognitive doctor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusOptions {
    pub layout_roots: LayoutRoots,
    pub endpoint_override: Option<String>,
}

/// `cognitive daemon` subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonCommand {
    Start(DaemonStartOptions),
    Status(StatusOptions),
    Stop(StatusOptions),
}

/// Options for `cognitive daemon start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStartOptions {
    pub layout_roots: LayoutRoots,
    pub bind_address: String,
    pub kernel_server_path: Option<PathBuf>,
}

/// Exit codes mirror admin-cli: 0 success, 1 operational denial, 2 usage.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_OPERATIONAL: i32 = 1;
pub const EXIT_USAGE: i32 = 2;

/// Parse argv (without program name) into a [`CognitiveCommand`].
pub fn parse_cognitive_args(args: &[String]) -> Result<CognitiveCommand, String> {
    let Some((verb, rest)) = args.split_first() else {
        return Err("missing verb".to_owned());
    };
    match verb.as_str() {
        "init" => {
            let flags = parse_flags(rest)?;
            Ok(CognitiveCommand::Init(parse_init_options(&flags)?))
        }
        "status" => {
            let flags = parse_flags(rest)?;
            Ok(CognitiveCommand::Status(parse_status_options(&flags)?))
        }
        "doctor" => {
            let flags = parse_flags(rest)?;
            Ok(CognitiveCommand::Doctor(parse_status_options(&flags)?))
        }
        "daemon" => {
            let Some((sub, daemon_rest)) = rest.split_first() else {
                return Err("daemon requires subcommand start|status|stop".to_owned());
            };
            let daemon_flags = parse_flags(daemon_rest)?;
            match sub.as_str() {
                "start" => Ok(CognitiveCommand::Daemon(DaemonCommand::Start(
                    parse_daemon_start_options(&daemon_flags)?,
                ))),
                "status" => Ok(CognitiveCommand::Daemon(DaemonCommand::Status(
                    parse_status_options(&daemon_flags)?,
                ))),
                "stop" => Ok(CognitiveCommand::Daemon(DaemonCommand::Stop(
                    parse_status_options(&daemon_flags)?,
                ))),
                other => Err(format!(
                    "unknown daemon subcommand `{other}` (expected start|status|stop)"
                )),
            }
        }
        other => Err(format!(
            "unknown verb `{other}` (expected init|status|doctor|daemon)"
        )),
    }
}

/// Dispatch a parsed command and return process exit code.
pub fn run_cognitive_command(command: CognitiveCommand) -> i32 {
    match command {
        CognitiveCommand::Init(options) => match run_init(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Status(options) => match fetch_projection(&options, ProjectionKind::Status) {
            Ok(body) => {
                println!("{body}");
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Doctor(options) => match fetch_projection(&options, ProjectionKind::Doctor) {
            Ok(body) => {
                println!("{body}");
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Daemon(DaemonCommand::Start(options)) => match daemon::start(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Daemon(DaemonCommand::Status(options)) => match daemon::status(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Daemon(DaemonCommand::Stop(options)) => match daemon::stop(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
    }
}

enum ProjectionKind {
    Status,
    Doctor,
}

fn fetch_projection(options: &StatusOptions, kind: ProjectionKind) -> Result<String, String> {
    let layout = layout::build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    let endpoint = match &options.endpoint_override {
        Some(value) => value.clone(),
        None => daemon::load_endpoint(&layout)?,
    };
    let client = client::PersonalDaemonClient::connect(&endpoint, &layout)
        .map_err(|error| error.to_string())?;
    match kind {
        ProjectionKind::Status => client.get_status().map_err(|error| error.to_string()),
        ProjectionKind::Doctor => client.get_doctor().map_err(|error| error.to_string()),
    }
}

fn parse_init_options(flags: &BTreeMap<String, String>) -> Result<InitOptions, String> {
    Ok(InitOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        provider_id: flags.get("provider").cloned(),
        base_url: flags.get("base-url").cloned(),
        model_id: flags.get("model-id").cloned(),
        api_key_file: flags.get("api-key-file").map(PathBuf::from),
        allow_ephemeral_secret_backend: flag_bool(flags, "allow-ephemeral-secret-backend")?,
        rotate_key: flag_bool(flags, "rotate-key")?,
    })
}

fn parse_status_options(flags: &BTreeMap<String, String>) -> Result<StatusOptions, String> {
    Ok(StatusOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        endpoint_override: flags.get("endpoint").cloned(),
    })
}

fn parse_daemon_start_options(flags: &BTreeMap<String, String>) -> Result<DaemonStartOptions, String> {
    Ok(DaemonStartOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        bind_address: flags.get("bind").cloned().unwrap_or_else(|| "127.0.0.1:7420".to_owned()),
        kernel_server_path: flags.get("kernel-server").map(PathBuf::from),
    })
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut flags = BTreeMap::new();
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        if flag == "--allow-ephemeral-secret-backend" {
            if flags.insert("allow-ephemeral-secret-backend".to_owned(), "true".to_owned()).is_some() {
                return Err("flag --allow-ephemeral-secret-backend given twice".to_owned());
            }
            continue;
        }
        if flag == "--rotate-key" {
            if flags.insert("rotate-key".to_owned(), "true".to_owned()).is_some() {
                return Err("flag --rotate-key given twice".to_owned());
            }
            continue;
        }
        let Some(name) = flag.strip_prefix("--") else {
            return Err(format!("unexpected argument `{flag}`"));
        };
        let Some(value) = iter.next() else {
            return Err(format!("flag --{name} requires a value"));
        };
        if flags.insert(name.to_owned(), value.clone()).is_some() {
            return Err(format!("flag --{name} given twice"));
        }
    }
    Ok(flags)
}

fn flag_bool(flags: &BTreeMap<String, String>, name: &str) -> Result<bool, String> {
    match flags.get(name).map(String::as_str) {
        None => Ok(false),
        Some("true") | Some("1") | Some("yes") => Ok(true),
        Some(other) => Err(format!("flag --{name} expects true/false, got `{other}`")),
    }
}

fn print_operational_error(message: &str) -> i32 {
    let payload = serde_json::json!({
        "status": "error",
        "surface": "cognitive-cli",
        "message": message,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed"
    });
    eprintln!("{}", pretty_json(&payload));
    EXIT_OPERATIONAL
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

/// Product CLI usage text.
pub const COGNITIVE_USAGE: &str = "cognitive — CognitiveOS Personal product CLI (P1-T06)

USAGE:
  cognitive init   [--runtime-root <dir>] [--provider <id>] [--base-url <https-url>]
                   [--model-id <id>] [--api-key-file <path|->] [--rotate-key]
                   [--allow-ephemeral-secret-backend]
  cognitive status [--runtime-root <dir>] [--endpoint <host:port>]
  cognitive doctor [--runtime-root <dir>] [--endpoint <host:port>]
  cognitive daemon start  [--runtime-root <dir>] [--bind 127.0.0.1:7420]
                          [--kernel-server <path>]
  cognitive daemon status [--runtime-root <dir>]
  cognitive daemon stop   [--runtime-root <dir>]

Hard rules:
  - never writes Provider API keys to config, SQLite, env, argv, logs, or evidence
  - never advances Task/Effect/Verification authority state
  - admin-cli management verbs remain available as the emergency path
  - --allow-ephemeral-secret-backend is for hermetic tests only

Exit codes: 0 success, 1 operational error, 2 usage error.";
