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
mod dsh;
mod init;
mod layout;
mod pi;
mod provider;
mod secret_input;
mod url;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

pub use dsh::{DshConfigureOptions, DshLaunchOptions, DshProviderPath, DshStatusOptions};
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
    Dsh(DshCommand),
    Resource(ResourceCommand),
    Task(TaskCommand),
    Provider(provider::ProviderCommand),
    AgentBinding(provider::BindingCommand),
    Usage(StatusOptions),
    Budget(provider::BudgetCommand),
    Alerts(provider::AlertCommand),
    Audit(StatusOptions),
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

/// `cognitive dsh` subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DshCommand {
    Configure(DshConfigureOptions),
    Launch(DshLaunchOptions),
    Web(DshLaunchOptions),
    Apply(DshStatusOptions),
    Status(DshStatusOptions),
}

/// Private resource projection and common Resource Manager commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCommand {
    Get(ResourceOptions),
    Watch(ResourceOptions),
    List(ResourceListOptions),
    Inspect(ResourceInspectOptions),
    Mutate(ResourceMutateOptions),
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
pub struct ResourceListOptions {
    pub status: StatusOptions,
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInspectOptions {
    pub status: StatusOptions,
    pub family: String,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMutateOperation {
    Bind,
    Unbind,
    Enable,
    Disable,
    Revoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceMutateOptions {
    pub status: StatusOptions,
    pub family: String,
    pub id: String,
    pub expected_version: i64,
    pub idempotency_key: String,
    pub operation: ResourceMutateOperation,
    pub payload: String,
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
        "dsh" => {
            let Some((subcommand, dsh_rest)) = rest.split_first() else {
                return Err("dsh requires subcommand configure|launch|web|apply|status".to_owned());
            };
            let flags = parse_flags(dsh_rest)?;
            match subcommand.as_str() {
                "configure" => Ok(CognitiveCommand::Dsh(DshCommand::Configure(
                    parse_dsh_configure_options(&flags)?,
                ))),
                "launch" => Ok(CognitiveCommand::Dsh(DshCommand::Launch(
                    parse_dsh_launch_options(&flags)?,
                ))),
                "web" => Ok(CognitiveCommand::Dsh(DshCommand::Web(
                    parse_dsh_web_options(&flags)?,
                ))),
                "status" => Ok(CognitiveCommand::Dsh(DshCommand::Status(
                    parse_dsh_status_options(&flags)?,
                ))),
                "apply" => Ok(CognitiveCommand::Dsh(DshCommand::Apply(
                    parse_dsh_status_options(&flags)?,
                ))),
                other => Err(format!(
                    "unknown dsh subcommand `{other}` (expected configure|launch|web|apply|status)"
                )),
            }
        }
        "resource" => parse_resource_command(rest),
        "task" => parse_task_command(rest),
        "provider" => provider::parse_provider_args(rest),
        "agent" => provider::parse_agent_args(rest),
        "usage" => provider::parse_usage_args(rest),
        "budget" => provider::parse_budget_args(rest),
        "alerts" => provider::parse_alerts_args(rest),
        "audit" => provider::parse_audit_args(rest),
        "backup" => parse_backup_command(rest),
        "restore" => parse_restore_command(rest),
        other => Err(format!(
            "unknown verb `{other}` (expected init|status|doctor|daemon|pi|dsh|resource|task|provider|agent|usage|budget|alerts|audit|backup|restore)"
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
        CognitiveCommand::Dsh(DshCommand::Configure(options)) => match dsh::configure(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Dsh(DshCommand::Launch(options)) => match dsh::launch(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Dsh(DshCommand::Web(options)) => match dsh::launch(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Dsh(DshCommand::Status(options)) => match dsh::status(&options) {
            Ok(report) => {
                println!("{}", pretty_json(&report));
                EXIT_SUCCESS
            }
            Err(error) => print_operational_error(&error),
        },
        CognitiveCommand::Dsh(DshCommand::Apply(options)) => match dsh::apply(&options) {
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
        CognitiveCommand::Resource(ResourceCommand::List(options)) => fetch_resource_list(&options),
        CognitiveCommand::Resource(ResourceCommand::Inspect(options)) => {
            fetch_resource_inspect(&options)
        }
        CognitiveCommand::Resource(ResourceCommand::Mutate(options)) => {
            fetch_resource_mutate(&options)
        }
        CognitiveCommand::Task(TaskCommand::Watch(options)) => fetch_task_watch(&options),
        CognitiveCommand::Task(TaskCommand::Evidence(options)) => fetch_task_evidence(&options),
        CognitiveCommand::Provider(command) => provider::run_provider(command),
        CognitiveCommand::AgentBinding(command) => provider::run_binding(command),
        CognitiveCommand::Usage(options) => provider::run_usage(&options),
        CognitiveCommand::Budget(command) => provider::run_budget(command),
        CognitiveCommand::Alerts(command) => provider::run_alerts(command),
        CognitiveCommand::Audit(options) => provider::run_audit(&options),
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
        return Err(
            "resource requires subcommand get|watch|list|inspect|bind|unbind|enable|disable|revoke"
                .to_owned(),
        );
    };
    let flags = parse_flags(remaining_arguments)?;
    match subcommand.as_str() {
        "get" | "watch" => {
            reject_unexpected_flags(
                &flags,
                &["runtime-root", "endpoint", "family", "resume-from"],
            )?;
            let family = required_resource_family(&flags)?;
            let resume_from = parse_optional_cursor(&flags)?;
            let status = parse_status_options(&flags)?;
            let options = ResourceOptions {
                status,
                family,
                resume_from,
            };
            if subcommand == "get" && options.resume_from.is_some() {
                return Err("resource get does not accept --resume-from".to_owned());
            }
            if subcommand == "get" {
                Ok(CognitiveCommand::Resource(ResourceCommand::Get(options)))
            } else {
                Ok(CognitiveCommand::Resource(ResourceCommand::Watch(options)))
            }
        }
        "list" => {
            reject_unexpected_flags(&flags, &["runtime-root", "endpoint", "family"])?;
            Ok(CognitiveCommand::Resource(ResourceCommand::List(
                ResourceListOptions {
                    status: parse_status_options(&flags)?,
                    family: required_resource_family(&flags)?,
                },
            )))
        }
        "inspect" => {
            reject_unexpected_flags(&flags, &["runtime-root", "endpoint", "family", "id"])?;
            Ok(CognitiveCommand::Resource(ResourceCommand::Inspect(
                ResourceInspectOptions {
                    status: parse_status_options(&flags)?,
                    family: required_resource_family(&flags)?,
                    id: required_resource_flag(&flags, "id")?,
                },
            )))
        }
        "bind" | "unbind" | "enable" | "disable" | "revoke" => {
            reject_unexpected_flags(
                &flags,
                &[
                    "runtime-root",
                    "endpoint",
                    "family",
                    "id",
                    "expected-version",
                    "idempotency-key",
                    "payload",
                ],
            )?;
            let operation = match subcommand.as_str() {
                "bind" => ResourceMutateOperation::Bind,
                "unbind" => ResourceMutateOperation::Unbind,
                "enable" => ResourceMutateOperation::Enable,
                "disable" => ResourceMutateOperation::Disable,
                _ => ResourceMutateOperation::Revoke,
            };
            let expected_version = flags
                .get("expected-version")
                .ok_or_else(|| {
                    "resource mutation requires --expected-version <integer>".to_owned()
                })?
                .parse::<i64>()
                .map_err(|_| "--expected-version must be an integer".to_owned())?;
            Ok(CognitiveCommand::Resource(ResourceCommand::Mutate(
                ResourceMutateOptions {
                    status: parse_status_options(&flags)?,
                    family: required_resource_family(&flags)?,
                    id: required_resource_flag(&flags, "id")?,
                    expected_version,
                    idempotency_key: required_resource_flag(&flags, "idempotency-key")?,
                    operation,
                    payload: flags.get("payload").cloned().unwrap_or_default(),
                },
            )))
        }
        _ => Err(
            "resource requires subcommand get|watch|list|inspect|bind|unbind|enable|disable|revoke"
                .to_owned(),
        ),
    }
}

fn required_resource_family(flags: &BTreeMap<String, String>) -> Result<String, String> {
    let family = flags
        .get("family")
        .cloned()
        .ok_or_else(|| "resource command requires --family <family>".to_owned())?;
    if !["memory", "skill", "tool", "context", "task", "runtime"].contains(&family.as_str()) {
        return Err("resource --family must be memory|skill|tool|context|task|runtime".to_owned());
    }
    Ok(family)
}

fn required_resource_flag(flags: &BTreeMap<String, String>, name: &str) -> Result<String, String> {
    flags
        .get(name)
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("resource command requires --{name}"))
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

fn fetch_resource_list(options: &ResourceListOptions) -> i32 {
    match connect_resource_client(&options.status) {
        Ok(client) => print_resource_client_result(client.list_resources(&options.family)),
        Err(code) => code,
    }
}

fn fetch_resource_inspect(options: &ResourceInspectOptions) -> i32 {
    match connect_resource_client(&options.status) {
        Ok(client) => {
            print_resource_client_result(client.inspect_resource(&options.family, &options.id))
        }
        Err(code) => code,
    }
}

fn fetch_resource_mutate(options: &ResourceMutateOptions) -> i32 {
    let operation = match options.operation {
        ResourceMutateOperation::Bind => "bind",
        ResourceMutateOperation::Unbind => "unbind",
        ResourceMutateOperation::Enable => "enable",
        ResourceMutateOperation::Disable => "disable",
        ResourceMutateOperation::Revoke => "revoke",
    };
    match connect_resource_client(&options.status) {
        Ok(client) => print_resource_client_result(client.mutate_resource(
            operation,
            &options.family,
            &options.id,
            options.expected_version,
            &options.idempotency_key,
            &options.payload,
        )),
        Err(code) => code,
    }
}

pub(crate) fn connect_resource_client(
    status: &StatusOptions,
) -> Result<client::PersonalDaemonClient, i32> {
    let layout = match layout::build_layout(&status.layout_roots) {
        Ok(layout) => layout,
        Err(error) => return Err(print_operational_error(&error.to_string())),
    };
    let endpoint = match resolve_endpoint(status, &layout) {
        Ok(endpoint) => endpoint,
        Err(error) => return Err(print_operational_error(&error)),
    };
    match client::PersonalDaemonClient::connect(&endpoint, &layout) {
        Ok(client) => Ok(client),
        Err(error) => Err(print_operational_error(&error.to_string())),
    }
}

fn print_resource_client_result(result: Result<String, client::PersonalDaemonClientError>) -> i32 {
    match result {
        Ok(body) => {
            println!("{body}");
            EXIT_SUCCESS
        }
        Err(error) => print_operational_error(&error.to_string()),
    }
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

pub(crate) fn parse_status_options(
    flags: &BTreeMap<String, String>,
) -> Result<StatusOptions, String> {
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

fn parse_dsh_configure_options(
    flags: &BTreeMap<String, String>,
) -> Result<DshConfigureOptions, String> {
    reject_unexpected_flags(
        flags,
        &["runtime-root", "dsh-root", "adapter-root", "revision"],
    )?;
    Ok(DshConfigureOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        dsh_root: required_dsh_path_flag(flags, "dsh-root")?,
        adapter_root: required_dsh_path_flag(flags, "adapter-root")?,
        revision: flags
            .get("revision")
            .cloned()
            .ok_or_else(|| "dsh configuration requires --revision <git-object>".to_owned())?,
    })
}

fn parse_dsh_status_options(flags: &BTreeMap<String, String>) -> Result<DshStatusOptions, String> {
    reject_unexpected_flags(flags, &["runtime-root"])?;
    Ok(DshStatusOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
    })
}

fn parse_dsh_launch_options(flags: &BTreeMap<String, String>) -> Result<DshLaunchOptions, String> {
    reject_unexpected_flags(flags, &["runtime-root", "print", "path", "task"])?;
    let provider_path = match flags.get("path").map(String::as_str) {
        None | Some("b") => DshProviderPath::Adapter,
        Some("a") => DshProviderPath::Direct,
        Some(other) => return Err(format!("dsh launch --path must be a or b, not `{other}`")),
    };
    Ok(DshLaunchOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        print_mode: flag_bool(flags, "print")?,
        provider_path,
        task: flags.get("task").cloned(),
        web_mode: false,
        listen_host: dsh::DEFAULT_WEB_HOST.to_owned(),
        listen_port: dsh::DEFAULT_WEB_PORT,
    })
}

fn parse_dsh_web_options(flags: &BTreeMap<String, String>) -> Result<DshLaunchOptions, String> {
    reject_unexpected_flags(flags, &["runtime-root", "path", "host", "port", "no-open"])?;
    if flag_bool(flags, "print")? {
        return Err("dsh web does not accept --print; use `cognitive dsh launch --print` for headless Path B".to_owned());
    }
    if flags.contains_key("task") {
        return Err(
            "dsh web does not accept --task; the native panel is a long-running process".to_owned(),
        );
    }
    if flag_bool(flags, "open")? {
        return Err(
            "dsh web refuses --open; native dsh web has no TLS/auth — bind loopback and open http://127.0.0.1:<port> yourself"
                .to_owned(),
        );
    }
    let provider_path = match flags.get("path").map(String::as_str) {
        None | Some("b") => DshProviderPath::Adapter,
        Some("a") => DshProviderPath::Direct,
        Some(other) => return Err(format!("dsh web --path must be a or b, not `{other}`")),
    };
    let listen_host = flags
        .get("host")
        .cloned()
        .unwrap_or_else(|| dsh::DEFAULT_WEB_HOST.to_owned());
    let listen_port = match flags.get("port") {
        None => dsh::DEFAULT_WEB_PORT,
        Some(value) => value
            .parse::<u16>()
            .map_err(|_| "dsh web --port must be an integer 1..=65535".to_owned())
            .and_then(|port| {
                if port == 0 {
                    Err("dsh web --port must be an integer 1..=65535".to_owned())
                } else {
                    Ok(port)
                }
            })?,
    };
    if listen_host == "0.0.0.0" || listen_host == "::" || listen_host == "[::]" {
        return Err(
            "dsh web --host 0.0.0.0/:: is refused; native dsh web has no TLS/auth and must bind loopback only"
                .to_owned(),
        );
    }
    Ok(DshLaunchOptions {
        layout_roots: LayoutRoots::from_flags(flags)?,
        print_mode: false,
        provider_path,
        task: None,
        web_mode: true,
        listen_host,
        listen_port,
    })
}

fn required_dsh_path_flag(flags: &BTreeMap<String, String>, name: &str) -> Result<PathBuf, String> {
    flags
        .get(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("dsh configuration requires --{name} <absolute-path>"))
}

fn required_path_flag(flags: &BTreeMap<String, String>, name: &str) -> Result<PathBuf, String> {
    flags
        .get(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("Pi configuration requires --{name} <absolute-path>"))
}

pub(crate) fn reject_unexpected_flags(
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

pub(crate) fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
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
        if flag == "--no-open" {
            if flags
                .insert("no-open".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --no-open given twice".to_owned());
            }
            continue;
        }
        if flag == "--open" {
            if flags.insert("open".to_owned(), "true".to_owned()).is_some() {
                return Err("flag --open given twice".to_owned());
            }
            continue;
        }
        if flag == "--allow-private-network" {
            if flags
                .insert("allow-private-network".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --allow-private-network given twice".to_owned());
            }
            continue;
        }
        if flag == "--allow-insecure-http" {
            if flags
                .insert("allow-insecure-http".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --allow-insecure-http given twice".to_owned());
            }
            continue;
        }
        if flag == "--reconfirm" {
            if flags
                .insert("reconfirm".to_owned(), "true".to_owned())
                .is_some()
            {
                return Err("flag --reconfirm given twice".to_owned());
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

pub(crate) fn flag_bool(flags: &BTreeMap<String, String>, name: &str) -> Result<bool, String> {
    match flags.get(name).map(String::as_str) {
        None => Ok(false),
        Some("true") | Some("1") | Some("yes") => Ok(true),
        Some(other) => Err(format!("flag --{name} expects true/false, got `{other}`")),
    }
}

pub(crate) fn print_operational_error(message: &str) -> i32 {
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
  cognitive dsh configure [--runtime-root <dir>] --dsh-root <absolute-path>
                          --adapter-root <absolute-path> --revision <git-object>
  cognitive dsh launch [--runtime-root <dir>] [--print] [--path a|b]
                       [--task <prompt>]
  cognitive dsh web [--runtime-root <dir>] [--path b] [--host 127.0.0.1]
                    [--port 3080] [--no-open]
  cognitive dsh apply [--runtime-root <dir>]
  cognitive dsh status [--runtime-root <dir>]
  cognitive resource get|watch [--runtime-root <dir>] [--endpoint <host:port>]
                       --family <memory|skill|tool|context|task|runtime>
                       [--resume-from <cursor>]
  cognitive resource list [--runtime-root <dir>] [--endpoint <host:port>]
                       --family <memory|skill|tool|context|task|runtime>
  cognitive resource inspect [--runtime-root <dir>] [--endpoint <host:port>]
                       --family <memory|skill|tool|context|task|runtime> --id <id>
  cognitive resource bind|unbind|enable|disable|revoke
                       [--runtime-root <dir>] [--endpoint <host:port>]
                       --family <family> --id <id> --expected-version <n>
                       --idempotency-key <key> [--payload <json-object>]
  cognitive task watch [--runtime-root <dir>] [--endpoint <host:port>]
                       [--resume-from <cursor>]
  cognitive task evidence [--runtime-root <dir>] [--endpoint <host:port>]
                          --task-ref <task-uri>
  cognitive provider account create --name <id> --provider-kind <openai_official|anthropic_official|openai_compatible>
                       [--endpoint-url <url>] [--api-key-file <path|->]
                       [--allow-private-network] [--allow-insecure-http]
  cognitive provider account list|show|update|delete [--id <acct>] [--endpoint-url <url>] [--reconfirm]
  cognitive provider key set|rotate|remove --id <acct> [--api-key-file <path|->]
  cognitive provider models refresh|list|add|set-price --account-id <acct> [--model-id <id>]
  cognitive agent binding set|show|list|remove [--agent pi|dsh] [--account-id <acct>] [--model-id <id>]
  cognitive usage query
  cognitive budget set|list|remove [--scope-kind account|agent] [--scope-id <id>]
  cognitive alerts list|acknowledge [--alert-id <id>]
  cognitive audit query
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
  - dsh configuration writes only non-secret pin/paths and a candidate-only adapter digest
  - dsh launch requires daemon-owned system/database/secret/daemon ready (Pi and the
    Pi provider.json component may stay blocked). Path B uses the Cos Provider
    control plane + SecretStore, loads the pinned AKP plugin, and never treats a
    dsh response as Task completion
  - dsh web starts the native dsh control panel (`dsh --profile web --no-open`) on
    loopback only (default http://127.0.0.1:3080). This is not Personal `/ui/`.
    Missing apps/web/dist fails closed. Path B still uses the daemon Provider proxy
    and overlays the Cos-assigned dsh model plus that account catalog.
    A panel session is never Task completion.
  - dsh apply POSTs /personal/dsh/runtime op=apply (Cos dsh binding → selected-model)
    and restarts only the Cos-installed `cognitive dsh web` pair on loopback 3080
    so conversation and Models show the Cos-assigned model
  - dsh status reads GET /personal/dsh/runtime (sessions, fencing, optional pid liveness)
  - dsh --path a is dsh→Flash direct; --path b is dsh→AKP→daemon→Flash (default)
    (web refuses --host 0.0.0.0 and --path a)
  - resource list/inspect/bind|unbind|enable|disable|revoke call the management Resource Manager; get/watch remain the private projection
  - provider/agent/usage/budget/alerts/audit call the management Provider Control Plane; keys use --api-key-file only
  - never advances Task/Effect/Verification authority state
  - daemon start appends kernel-server stdout/stderr to state/cognitiveos/daemon.log (mode 0600)
  - admin-cli management verbs remain available as the emergency path
  - --allow-ephemeral-secret-backend is for hermetic tests only

Exit codes: 0 success, 1 operational error, 2 usage error.";

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
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
    fn dsh_configuration_accepts_only_non_secret_path_flags() {
        let command = parse_cognitive_args(&[
            "dsh".to_owned(),
            "configure".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--dsh-root".to_owned(),
            "/opt/dsh".to_owned(),
            "--adapter-root".to_owned(),
            "/opt/cognitiveos/packages/dsh-akp-adapter".to_owned(),
            "--revision".to_owned(),
            cognitive_runtime::DSH_PACKAGE_REVISION.to_owned(),
        ])
        .expect("parse dsh configuration");
        assert_eq!(
            command,
            CognitiveCommand::Dsh(DshCommand::Configure(DshConfigureOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
                dsh_root: PathBuf::from("/opt/dsh"),
                adapter_root: PathBuf::from("/opt/cognitiveos/packages/dsh-akp-adapter"),
                revision: cognitive_runtime::DSH_PACKAGE_REVISION.to_owned(),
            }))
        );

        let rejected = parse_cognitive_args(&[
            "dsh".to_owned(),
            "configure".to_owned(),
            "--dsh-root".to_owned(),
            "/opt/dsh".to_owned(),
            "--adapter-root".to_owned(),
            "/opt/adapter".to_owned(),
            "--revision".to_owned(),
            cognitive_runtime::DSH_PACKAGE_REVISION.to_owned(),
            "--api-key-file".to_owned(),
            "/tmp/key".to_owned(),
        ])
        .expect_err("dsh configuration must reject Provider secret flags");
        assert!(rejected.contains("not accepted"), "{rejected}");
    }

    #[test]
    fn dsh_status_parses_runtime_root_without_secret_flags() {
        let command = parse_cognitive_args(&[
            "dsh".to_owned(),
            "status".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
        ])
        .expect("parse dsh status");
        assert_eq!(
            command,
            CognitiveCommand::Dsh(DshCommand::Status(DshStatusOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
            }))
        );

        let rejected = parse_cognitive_args(&[
            "dsh".to_owned(),
            "status".to_owned(),
            "--api-key-file".to_owned(),
            "/tmp/key".to_owned(),
        ])
        .expect_err("dsh status must reject Provider secret flags");
        assert!(rejected.contains("not accepted"), "{rejected}");
    }

    #[test]
    fn dsh_apply_parses_runtime_root_without_secret_flags() {
        let command = parse_cognitive_args(&[
            "dsh".to_owned(),
            "apply".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
        ])
        .expect("parse dsh apply");
        assert_eq!(
            command,
            CognitiveCommand::Dsh(DshCommand::Apply(DshStatusOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(PathBuf::from("/tmp/cognitiveos")),
                },
            }))
        );
        let rejected = parse_cognitive_args(&[
            "dsh".to_owned(),
            "apply".to_owned(),
            "--api-key-file".to_owned(),
            "/tmp/key".to_owned(),
        ])
        .expect_err("dsh apply must reject Provider secret flags");
        assert!(rejected.contains("not accepted"), "{rejected}");
    }

    #[test]
    fn dsh_web_parses_loopback_and_rejects_wildcard_host() {
        let command = parse_cognitive_args(&[
            "dsh".to_owned(),
            "web".to_owned(),
            "--runtime-root".to_owned(),
            "/tmp/cognitiveos".to_owned(),
            "--host".to_owned(),
            "127.0.0.1".to_owned(),
            "--port".to_owned(),
            "3080".to_owned(),
            "--no-open".to_owned(),
        ])
        .expect("parse dsh web");
        match command {
            CognitiveCommand::Dsh(DshCommand::Web(options)) => {
                assert!(options.web_mode);
                assert_eq!(options.listen_host, "127.0.0.1");
                assert_eq!(options.listen_port, 3080);
                assert!(!options.print_mode);
                assert_eq!(options.provider_path, DshProviderPath::Adapter);
            }
            other => panic!("expected dsh web, got {other:?}"),
        }

        let wildcard = parse_cognitive_args(&[
            "dsh".to_owned(),
            "web".to_owned(),
            "--host".to_owned(),
            "0.0.0.0".to_owned(),
        ])
        .expect_err("wildcard host");
        assert!(
            wildcard.contains("0.0.0.0") || wildcard.contains("loopback"),
            "{wildcard}"
        );

        let open = parse_cognitive_args(&["dsh".to_owned(), "web".to_owned(), "--open".to_owned()])
            .expect_err("open refused");
        assert!(open.contains("--open"), "{open}");
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

    #[test]
    fn resource_manager_verbs_parse_common_envelope_flags() {
        let list = parse_cognitive_args(&[
            "resource".to_owned(),
            "list".to_owned(),
            "--family".to_owned(),
            "tool".to_owned(),
        ])
        .expect("parse resource list");
        assert_eq!(
            list,
            CognitiveCommand::Resource(ResourceCommand::List(ResourceListOptions {
                status: StatusOptions {
                    layout_roots: LayoutRoots { runtime_root: None },
                    endpoint_override: None,
                },
                family: "tool".to_owned(),
            }))
        );

        let disable = parse_cognitive_args(&[
            "resource".to_owned(),
            "disable".to_owned(),
            "--family".to_owned(),
            "tool".to_owned(),
            "--id".to_owned(),
            "native.workspace.read".to_owned(),
            "--expected-version".to_owned(),
            "1".to_owned(),
            "--idempotency-key".to_owned(),
            "p8-t12-disable-1".to_owned(),
        ])
        .expect("parse resource disable");
        assert_eq!(
            disable,
            CognitiveCommand::Resource(ResourceCommand::Mutate(ResourceMutateOptions {
                status: StatusOptions {
                    layout_roots: LayoutRoots { runtime_root: None },
                    endpoint_override: None,
                },
                family: "tool".to_owned(),
                id: "native.workspace.read".to_owned(),
                expected_version: 1,
                idempotency_key: "p8-t12-disable-1".to_owned(),
                operation: ResourceMutateOperation::Disable,
                payload: String::new(),
            }))
        );

        let unknown = parse_cognitive_args(&[
            "resource".to_owned(),
            "create".to_owned(),
            "--family".to_owned(),
            "tool".to_owned(),
        ])
        .expect_err("generic create is not a CLI verb");
        assert!(unknown.contains("get|watch|list|inspect"), "{unknown}");
    }

    #[test]
    fn provider_control_plane_verbs_parse_and_refuse_api_key_flag() {
        let create = parse_cognitive_args(&[
            "provider".to_owned(),
            "account".to_owned(),
            "create".to_owned(),
            "--name".to_owned(),
            "openai-work".to_owned(),
            "--provider-kind".to_owned(),
            "openai_official".to_owned(),
        ])
        .expect("parse provider account create");
        match create {
            CognitiveCommand::Provider(provider::ProviderCommand::Account(
                provider::AccountCommand::Create(options),
            )) => {
                assert_eq!(options.name, "openai-work");
                assert_eq!(options.provider_kind, "openai_official");
            }
            other => panic!("unexpected command {other:?}"),
        }

        let refused = parse_cognitive_args(&[
            "provider".to_owned(),
            "key".to_owned(),
            "set".to_owned(),
            "--id".to_owned(),
            "acct-1".to_owned(),
            "--api-key".to_owned(),
            "sk-live-should-never-parse".to_owned(),
        ])
        .expect_err("CLI must refuse --api-key");
        assert!(refused.contains("--api-key"), "{refused}");

        let binding = parse_cognitive_args(&[
            "agent".to_owned(),
            "binding".to_owned(),
            "set".to_owned(),
            "--agent".to_owned(),
            "dsh".to_owned(),
            "--account-id".to_owned(),
            "acct-1".to_owned(),
            "--model-id".to_owned(),
            "deepseek-chat".to_owned(),
        ])
        .expect("parse agent binding set");
        match binding {
            CognitiveCommand::AgentBinding(provider::BindingCommand::Set(options)) => {
                assert_eq!(options.agent, "dsh");
            }
            other => panic!("unexpected command {other:?}"),
        }
    }
}
