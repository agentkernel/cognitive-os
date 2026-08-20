//! Personal product CLI surface (`cognitive` bin, P1-T06).
//!
//! This module is a **client**: it prepares XDG layout / non-secret Provider
//! config, launches the Personal daemon process, and consumes authenticated
//! readiness projections over HTTP. It never opens SQLite authority tables to
//! advance Task/Effect/Verification state, never logs secret material, and
//! never claims G0 / B01-B12 / Profile conformance.

mod backup;
mod client;
mod daemon;
mod init;
mod layout;
mod pi;
mod secret_input;
mod url;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

pub use init::run_init;
pub use layout::{LayoutRoots, resolve_layout_roots};
pub use pi::{PiConfigureOptions, PiLaunchOptions};

/// Top-level `cognitive` verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CognitiveCommand {
    Init(InitOptions),
    Status(StatusOptions),
    Doctor(StatusOptions),
    Daemon(DaemonCommand),
    Pi(PiCommand),
    Resource(ResourceCommand),
    Task(TaskCommand),
    Backup(backup::BackupOptions),
    Restore(backup::RestoreOptions),
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
    /// Bind `provider.json` to the existing production SecretStore item for
    /// `--provider` without reading or writing key material.
    pub reuse_existing_secret_binding: bool,
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

/// `cognitive pi` subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiCommand {
    Configure(PiConfigureOptions),
    Launch(PiLaunchOptions),
}

/// Read-only private resource projection commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCommand {
    Get(ResourceOptions),
    Watch(ResourceOptions),
}

/// Read-only Task observation commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskCommand {
    Watch(TaskWatchOptions),
    Evidence(TaskEvidenceOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOptions {
    pub status: StatusOptions,
    pub family: String,
    pub resume_from: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskWatchOptions {
    pub status: StatusOptions,
    pub resume_from: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEvidenceOptions {
    pub status: StatusOptions,
    pub task_ref: String,
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
        "pi" => {
            let Some((subcommand, pi_rest)) = rest.split_first() else {
                return Err("pi requires subcommand configure|launch".to_owned());
            };
            let flags = parse_flags(pi_rest)?;
            match subcommand.as_str() {
                "configure" => Ok(CognitiveCommand::Pi(PiCommand::Configure(
                    parse_pi_configure_options(&flags)?,
                ))),
                "launch" => Ok(CognitiveCommand::Pi(PiCommand::Launch(
                    parse_pi_launch_options(&flags)?,
                ))),
                other => Err(format!(
                    "unknown pi subcommand `{other}` (expected configure|launch)"
                )),
            }
        }
        "resource" => parse_resource_command(rest),
        "task" => parse_task_command(rest),
        "backup" => parse_backup_command(rest),
        "restore" => parse_restore_command(rest),
        other => Err(format!(
            "unknown verb `{other}` (expected init|status|doctor|daemon|pi|resource|task|backup|restore)"
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
        CognitiveCommand::Status(options) => {
            match fetch_projection(&options, ProjectionKind::Status) {
                Ok(body) => {
                    println!("{body}");
                    EXIT_SUCCESS
                }
                Err(error) => print_operational_error(&error),
            }
        }
        CognitiveCommand::Pi(PiCommand::Configure(options)) => match pi::configure(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Pi(PiCommand::Launch(options)) => match pi::launch(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Doctor(options) => {
            match fetch_projection(&options, ProjectionKind::Doctor) {
                Ok(body) => {
                    println!("{body}");
                    EXIT_SUCCESS
                }
                Err(error) => print_operational_error(&error),
            }
        }
        CognitiveCommand::Resource(ResourceCommand::Get(options)) => {
            fetch_resource_projection(&options, false)
        }
        CognitiveCommand::Resource(ResourceCommand::Watch(options)) => {
            fetch_resource_projection(&options, true)
        }
        CognitiveCommand::Task(TaskCommand::Watch(options)) => fetch_task_watch(&options),
        CognitiveCommand::Task(TaskCommand::Evidence(options)) => fetch_task_evidence(&options),
        CognitiveCommand::Backup(options) => match backup::run_backup(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Restore(options) => match backup::run_restore(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
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
        CognitiveCommand::Daemon(DaemonCommand::Status(options)) => {
            match daemon::status(&options) {
                Ok(report) => {
                    println!("{}", pretty_json(&report));
                    EXIT_SUCCESS
                }
                Err(error) => print_operational_error(&error),
            }
        }
        CognitiveCommand::Daemon(DaemonCommand::Stop(options)) => match daemon::stop(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
    }
}

fn parse_resource_command(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((subcommand, remaining_arguments)) = arguments.split_first() else {
        return Err("resource requires subcommand get|watch".to_owned());
    };
    let flags = parse_flags(remaining_arguments)?;
    reject_unexpected_flags(
        &flags,
        &["runtime-root", "endpoint", "family", "resume-from"],
    )?;
    let family = flags
        .get("family")
        .cloned()
        .ok_or_else(|| "resource command requires --family <family>".to_owned())?;
    if !["memory", "skill", "tool", "context", "task", "runtime"].contains(&family.as_str()) {
        return Err("resource --family must be memory|skill|tool|context|task|runtime".to_owned());
    }
    let resume_from = parse_optional_cursor(&flags)?;
    let status = parse_status_options(&flags)?;
    let options = ResourceOptions {
        status,
        family,
        resume_from,
    };
    match subcommand.as_str() {
        "get" if options.resume_from.is_none() => {
            Ok(CognitiveCommand::Resource(ResourceCommand::Get(options)))
        }
        "get" => Err("resource get does not accept --resume-from".to_owned()),
        "watch" => Ok(CognitiveCommand::Resource(ResourceCommand::Watch(options))),
        _ => Err("resource requires subcommand get|watch".to_owned()),
    }
}

fn parse_backup_command(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let flags = parse_flags(arguments)?;
    reject_unexpected_flags(&flags, &["runtime-root", "output", "endpoint"])?;
    Ok(CognitiveCommand::Backup(backup::BackupOptions {
        layout_roots: LayoutRoots::from_flags(&flags)?,
        endpoint_override: flags.get("endpoint").cloned(),
        output: flags.get("output").map(PathBuf::from),
    }))
}

fn parse_restore_command(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let flags = parse_flags(arguments)?;
    reject_unexpected_flags(
        &flags,
        &[
            "runtime-root",
            "endpoint",
            "archive",
            "archive-id",
            "preflight",
        ],
    )?;
    let archive = flags.get("archive").map(PathBuf::from);
    let archive_id = flags
        .get("archive-id")
        .filter(|value| !value.trim().is_empty())
        .cloned();
    if archive.is_none() && archive_id.is_none() {
        return Err("restore requires --archive <dir> or --archive-id <id>".to_owned());
    }
    Ok(CognitiveCommand::Restore(backup::RestoreOptions {
        layout_roots: LayoutRoots::from_flags(&flags)?,
        endpoint_override: flags.get("endpoint").cloned(),
        archive,
        archive_id,
        preflight_only: flag_bool(&flags, "preflight")?,
    }))
}

fn parse_task_command(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((subcommand, remaining_arguments)) = arguments.split_first() else {
        return Err("task requires subcommand watch|evidence".to_owned());
    };
    let flags = parse_flags(remaining_arguments)?;
    match subcommand.as_str() {
        "watch" => {
            reject_unexpected_flags(&flags, &["runtime-root", "endpoint", "resume-from"])?;
            Ok(CognitiveCommand::Task(TaskCommand::Watch(
                TaskWatchOptions {
                    status: parse_status_options(&flags)?,
                    resume_from: parse_optional_cursor(&flags)?,
                },
            )))
        }
        "evidence" => {
            reject_unexpected_flags(&flags, &["runtime-root", "endpoint", "task-ref"])?;
            let task_ref = flags
                .get("task-ref")
                .filter(|value| !value.trim().is_empty())
                .cloned()
                .ok_or_else(|| "task evidence requires --task-ref <task-uri>".to_owned())?;
            Ok(CognitiveCommand::Task(TaskCommand::Evidence(
                TaskEvidenceOptions {
                    status: parse_status_options(&flags)?,
                    task_ref,
                },
            )))
        }
        _ => Err("task requires subcommand watch|evidence".to_owned()),
    }
}

fn parse_optional_cursor(flags: &BTreeMap<String, String>) -> Result<Option<u64>, String> {
    flags
        .get("resume-from")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "--resume-from must be an unsigned integer".to_owned())
        })
        .transpose()
}

fn fetch_resource_projection(options: &ResourceOptions, watch: bool) -> i32 {
    let layout = match layout::build_layout(&options.status.layout_roots) {
        Ok(layout) => layout,
        Err(error) => return print_operational_error(&error.to_string()),
    };
    let endpoint = match resolve_endpoint(&options.status, &layout) {
        Ok(endpoint) => endpoint,
        Err(error) => return print_operational_error(&error),
    };
    let client = match client::PersonalDaemonClient::connect(&endpoint, &layout) {
        Ok(client) => client,
        Err(error) => return print_operational_error(&error.to_string()),
    };
    let result = if watch {
        client.watch_resource(&options.family, options.resume_from)
    } else {
        client.get_resource_projection(&options.family)
    };
    match result {
        Ok(body) => {
            println!("{body}");
            EXIT_SUCCESS
        }
        Err(error) => print_operational_error(&error.to_string()),
    }
}

fn fetch_task_watch(options: &TaskWatchOptions) -> i32 {
    let layout = match layout::build_layout(&options.status.layout_roots) {
        Ok(layout) => layout,
        Err(error) => return print_operational_error(&error.to_string()),
    };
    let endpoint = match resolve_endpoint(&options.status, &layout) {
        Ok(endpoint) => endpoint,
        Err(error) => return print_operational_error(&error),
    };
    let client = match client::PersonalDaemonClient::connect(&endpoint, &layout) {
        Ok(client) => client,
        Err(error) => return print_operational_error(&error.to_string()),
    };
    match client.watch_task(options.resume_from) {
        Ok(body) => {
            println!("{body}");
            EXIT_SUCCESS
        }
        Err(error) => print_operational_error(&error.to_string()),
    }
}

fn fetch_task_evidence(options: &TaskEvidenceOptions) -> i32 {
    let layout = match layout::build_layout(&options.status.layout_roots) {
        Ok(layout) => layout,
        Err(error) => return print_operational_error(&error.to_string()),
    };
    let endpoint = match resolve_endpoint(&options.status, &layout) {
        Ok(endpoint) => endpoint,
        Err(error) => return print_operational_error(&error),
    };
    let client = match client::PersonalDaemonClient::connect(&endpoint, &layout) {
        Ok(client) => client,
        Err(error) => return print_operational_error(&error.to_string()),
    };
    match client.get_task_evidence(&options.task_ref) {
        Ok(body) => {
            println!("{body}");
            EXIT_SUCCESS
        }
        Err(error) => print_operational_error(&error.to_string()),
    }
}

fn resolve_endpoint(
    options: &StatusOptions,
    layout: &cognitive_store::PersonalDataLayout,
) -> Result<String, String> {
    options
        .endpoint_override
        .clone()
        .map_or_else(|| daemon::load_endpoint(layout), Ok)
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
        reuse_existing_secret_binding: flag_bool(flags, "reuse-existing-secret-binding")?,
    })
}

fn parse_status_options(flags: &BTreeMap<String, String>) -> Result<StatusOptions, String> {
    Ok(StatusOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        endpoint_override: flags.get("endpoint").cloned(),
    })
}

fn parse_daemon_start_options(
    flags: &BTreeMap<String, String>,
) -> Result<DaemonStartOptions, String> {
    Ok(DaemonStartOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        bind_address: flags
            .get("bind")
            .cloned()
            .unwrap_or_else(|| "127.0.0.1:48181".to_owned()),
        kernel_server_path: flags.get("kernel-server").map(PathBuf::from),
    })
}

fn parse_pi_configure_options(
    flags: &BTreeMap<String, String>,
) -> Result<PiConfigureOptions, String> {
    reject_unexpected_flags(
        flags,
        &[
            "runtime-root",
            "executable",
            "extension-entry",
            "candidate-adapter",
            "candidate-extension",
        ],
    )?;
    let executable_path = required_path_flag(flags, "executable")?;
    let extension_entry_path = required_path_flag(flags, "extension-entry")?;
    Ok(PiConfigureOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        executable_path,
        extension_entry_path,
        candidate_adapter_path: flags.get("candidate-adapter").map(PathBuf::from),
        candidate_extension_entry_path: flags.get("candidate-extension").map(PathBuf::from),
    })
}

fn parse_pi_launch_options(flags: &BTreeMap<String, String>) -> Result<PiLaunchOptions, String> {
    reject_unexpected_flags(
        flags,
        &["runtime-root", "print", "task-ref", "append-system-prompt"],
    )?;
    Ok(PiLaunchOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        print_mode: flag_bool(flags, "print")?,
        task_ref: flags.get("task-ref").cloned(),
        append_system_prompt: flags.get("append-system-prompt").map(PathBuf::from),
    })
}

fn required_path_flag(flags: &BTreeMap<String, String>, name: &str) -> Result<PathBuf, String> {
    flags
        .get(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("Pi configuration requires --{name} <absolute-path>"))
}

fn reject_unexpected_flags(
    flags: &BTreeMap<String, String>,
    allowed_flags: &[&str],
) -> Result<(), String> {
    if let Some(unexpected) = flags
        .keys()
        .find(|name| !allowed_flags.contains(&name.as_str()))
    {
        return Err(format!(
            "flag --{unexpected} is not accepted for this command"
        ));
    }
    Ok(())
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut flags = BTreeMap::new();
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        if flag == "--allow-ephemeral-secret-backend" {
            if flags
                .insert(
                    "allow-ephemeral-secret-backend".to_owned(),
                    "true".to_owned(),
                )
                .is_some()
            {
                return Err("flag --allow-ephemeral-secret-backend given twice".to_owned());
            }
            continue;
        }
        if flag == "--rotate-key" {
            if flags
                .insert("rotate-key".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --rotate-key given twice".to_owned());
            }
            continue;
        }
        if flag == "--reuse-existing-secret-binding" {
            if flags
                .insert(
                    "reuse-existing-secret-binding".to_owned(),
                    "true".to_owned(),
                )
                .is_some()
            {
                return Err("flag --reuse-existing-secret-binding given twice".to_owned());
            }
            continue;
        }
        if flag == "--preflight" {
            if flags
                .insert("preflight".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --preflight given twice".to_owned());
            }
            continue;
        }
        if flag == "--print" {
            if flags
                .insert("print".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --print given twice".to_owned());
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
                   [--reuse-existing-secret-binding]
                   [--allow-ephemeral-secret-backend]
  cognitive status [--runtime-root <dir>] [--endpoint <host:port>]
  cognitive doctor [--runtime-root <dir>] [--endpoint <host:port>]
  cognitive daemon start  [--runtime-root <dir>] [--bind 127.0.0.1:48181]
                          [--kernel-server <path>]
  cognitive daemon status [--runtime-root <dir>]
  cognitive daemon stop   [--runtime-root <dir>]
  cognitive pi configure [--runtime-root <dir>] --executable <absolute-path>
                         --extension-entry <absolute-path>
  cognitive pi launch [--runtime-root <dir>] [--print]
                       [--append-system-prompt <absolute-path>]
  cognitive task watch [--runtime-root <dir>] [--endpoint <host:port>]
                       [--resume-from <cursor>]
  cognitive task evidence [--runtime-root <dir>] [--endpoint <host:port>]
                          --task-ref <task-uri>
  cognitive backup  [--runtime-root <dir>] [--endpoint <host:port>] [--output <dir>]
  cognitive restore [--runtime-root <dir>] [--endpoint <host:port>]
                    (--archive <dir> | --archive-id <id>) [--preflight]

Hard rules:
  - never writes Provider API keys to config, SQLite, env, argv, logs, or evidence
  - backup/restore never copy secret, bearer, provider-config, or authority SQLite
  - Pi configuration writes only non-secret executable and Extension paths
  - Pi launch requires daemon-owned ready state, loads only its configured Extension,
    and disables Pi-native tools that bypass daemon authority
  - --append-system-prompt forwards an existing absolute UTF-8 file to Pi; it is not a
    Provider credential and the file bytes are not printed
  - never advances Task/Effect/Verification authority state
  - daemon start appends kernel-server stdout/stderr to state/cognitiveos/daemon.log (mode 0600)
  - admin-cli management verbs remain available as the emergency path
  - --allow-ephemeral-secret-backend is for hermetic tests only

Exit codes: 0 success, 1 operational error, 2 usage error.";

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn init_parses_reuse_existing_secret_binding_without_key_file() {
        let command = parse_cognitive_args(&[
            "init".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--provider".to_owned(),
            "deepseek".to_owned(),
            "--base-url".to_owned(),
            "https://api.deepseek.com/v1".to_owned(),
            "--reuse-existing-secret-binding".to_owned(),
        ])
        .expect("parse reuse-existing-secret-binding");
        assert_eq!(
            command,
            CognitiveCommand::Init(InitOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                provider_id: Some("deepseek".to_owned()),
                base_url: Some("https://api.deepseek.com/v1".to_owned()),
                model_id: None,
                api_key_file: None,
                allow_ephemeral_secret_backend: false,
                rotate_key: false,
                reuse_existing_secret_binding: true,
            })
        );
    }

    #[test]
    fn daemon_start_defaults_to_the_canonical_personal_loopback_port() {
        let arguments = vec!["daemon".to_owned(), "start".to_owned()];
        let command = parse_cognitive_args(&arguments).expect("parse daemon start");

        assert_eq!(
            command,
            CognitiveCommand::Daemon(DaemonCommand::Start(DaemonStartOptions {
                layout_roots: LayoutRoots { runtime_root: None },
                bind_address: "127.0.0.1:48181".to_owned(),
                kernel_server_path: None,
            }))
        );
    }

    #[test]
    fn pi_configuration_accepts_only_non_secret_path_flags() {
        let arguments = vec![
            "pi".to_owned(),
            "configure".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--executable".to_owned(),
            "/opt/pi/bin/pi".to_owned(),
            "--extension-entry".to_owned(),
            "/opt/cognitiveos/pi-cognitiveos/index.js".to_owned(),
            "--candidate-adapter".to_owned(),
            "/opt/cognitiveos/bin/pi-agent-adapter".to_owned(),
            "--candidate-extension".to_owned(),
            "/opt/cognitiveos/pi-cognitiveos/private-candidate.mjs".to_owned(),
        ];

        let command = parse_cognitive_args(&arguments).expect("parse Pi configuration");

        assert_eq!(
            command,
            CognitiveCommand::Pi(PiCommand::Configure(PiConfigureOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                executable_path: PathBuf::from("/opt/pi/bin/pi"),
                extension_entry_path: PathBuf::from("/opt/cognitiveos/pi-cognitiveos/index.js"),
                candidate_adapter_path: Some(PathBuf::from(
                    "/opt/cognitiveos/bin/pi-agent-adapter"
                )),
                candidate_extension_entry_path: Some(PathBuf::from(
                    "/opt/cognitiveos/pi-cognitiveos/private-candidate.mjs",
                )),
            }))
        );

        let rejected = parse_cognitive_args(&[
            "pi".to_owned(),
            "configure".to_owned(),
            "--executable".to_owned(),
            "/opt/pi/bin/pi".to_owned(),
            "--extension-entry".to_owned(),
            "/opt/cognitiveos/pi-cognitiveos/index.js".to_owned(),
            "--api-key-file".to_owned(),
            "/tmp/key".to_owned(),
        ])
        .expect_err("Pi configuration must reject Provider secret flags");

        assert!(rejected.contains("not accepted"), "{rejected}");
    }

    #[test]
    fn pi_launch_accepts_only_the_hermetic_runtime_root_flag() {
        let command = parse_cognitive_args(&[
            "pi".to_owned(),
            "launch".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
        ])
        .expect("parse constrained Pi launch command");
        assert_eq!(
            command,
            CognitiveCommand::Pi(PiCommand::Launch(PiLaunchOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                print_mode: false,
                task_ref: None,
                append_system_prompt: None,
            }))
        );

        let rejected = parse_cognitive_args(&[
            "pi".to_owned(),
            "launch".to_owned(),
            "--api-key-file".to_owned(),
            "/tmp/key".to_owned(),
        ])
        .expect_err("Pi launch must reject Provider secret flags");

        assert!(rejected.contains("not accepted"), "{rejected}");
    }

    #[test]
    fn pi_launch_accepts_noninteractive_print_mode_without_secret_flags() {
        let command = parse_cognitive_args(&[
            "pi".to_owned(),
            "launch".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--print".to_owned(),
        ])
        .expect("parse noninteractive Pi launch command");

        assert_eq!(
            command,
            CognitiveCommand::Pi(PiCommand::Launch(PiLaunchOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                print_mode: true,
                task_ref: None,
                append_system_prompt: None,
            }))
        );
    }

    #[test]
    fn pi_launch_accepts_append_system_prompt_without_secret_flags() {
        let command = parse_cognitive_args(&[
            "pi".to_owned(),
            "launch".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--print".to_owned(),
            "--append-system-prompt".to_owned(),
            "/tmp/frozen-system-task-prompt.txt".to_owned(),
        ])
        .expect("parse append-system-prompt Pi launch command");

        assert_eq!(
            command,
            CognitiveCommand::Pi(PiCommand::Launch(PiLaunchOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                print_mode: true,
                task_ref: None,
                append_system_prompt: Some(PathBuf::from("/tmp/frozen-system-task-prompt.txt")),
            }))
        );
    }

    #[test]
    fn task_evidence_requires_one_task_reference_and_accepts_endpoint_options() {
        let command = parse_cognitive_args(&[
            "task".to_owned(),
            "evidence".to_owned(),
            "--task-ref".to_owned(),
            "task://personal/example".to_owned(),
            "--endpoint".to_owned(),
            "127.0.0.1:48181".to_owned(),
        ])
        .expect("parse task evidence command");

        assert_eq!(
            command,
            CognitiveCommand::Task(TaskCommand::Evidence(TaskEvidenceOptions {
                status: StatusOptions {
                    layout_roots: LayoutRoots { runtime_root: None },
                    endpoint_override: Some("127.0.0.1:48181".to_owned()),
                },
                task_ref: "task://personal/example".to_owned(),
            }))
        );

        let missing_reference = parse_cognitive_args(&["task".to_owned(), "evidence".to_owned()])
            .expect_err("task evidence must require a Task reference");
        assert!(missing_reference.contains("--task-ref"));

        let rejected_secret_flag = parse_cognitive_args(&[
            "task".to_owned(),
            "evidence".to_owned(),
            "--task-ref".to_owned(),
            "task://personal/example".to_owned(),
            "--api-key-file".to_owned(),
            "secret.txt".to_owned(),
        ])
        .expect_err("task evidence must reject secret-bearing flags");
        assert!(rejected_secret_flag.contains("not accepted"));
    }

    #[test]
    fn backup_and_restore_parse_without_secret_flags() {
        let backup = parse_cognitive_args(&[
            "backup".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--output".to_owned(),
            "/tmp/backup-out".to_owned(),
        ])
        .expect("parse backup");
        assert_eq!(
            backup,
            CognitiveCommand::Backup(backup::BackupOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                endpoint_override: None,
                output: Some(PathBuf::from("/tmp/backup-out")),
            })
        );

        let restore = parse_cognitive_args(&[
            "restore".to_owned(),
            "--archive".to_owned(),
            "/tmp/backup-out".to_owned(),
            "--preflight".to_owned(),
        ])
        .expect("parse restore");
        assert_eq!(
            restore,
            CognitiveCommand::Restore(backup::RestoreOptions {
                layout_roots: LayoutRoots { runtime_root: None },
                endpoint_override: None,
                archive: Some(PathBuf::from("/tmp/backup-out")),
                archive_id: None,
                preflight_only: true,
            })
        );

        let rejected = parse_cognitive_args(&[
            "backup".to_owned(),
            "--api-key-file".to_owned(),
            "secret.txt".to_owned(),
        ])
        .expect_err("backup must reject secret flags");
        assert!(rejected.contains("not accepted"), "{rejected}");
    }
}
