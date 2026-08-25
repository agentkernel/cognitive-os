//! Management-channel Provider Control Plane CLI (P8-T13).
//!
//! The CLI is a daemon client only: it never opens SQLite or Secret Store.
//! API keys are read from `--api-key-file` (or hidden stdin) and sent once
//! over loopback HTTP. Output is redacted for key-shaped material.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};

use super::client::{PersonalDaemonClient, PersonalDaemonClientError};
use super::secret_input::read_api_key_material;
use super::{
    CognitiveCommand, EXIT_SUCCESS, StatusOptions, connect_resource_client, flag_bool, parse_flags,
    parse_status_options, print_operational_error, reject_unexpected_flags,
};

const CONTROL_PLANE_FLAGS: &[&str] = &[
    "runtime-root",
    "endpoint",
    "id",
    "name",
    "provider-kind",
    "endpoint-url",
    "api-key-file",
    "op",
    "account-id",
    "model-id",
    "agent",
    "budget-id",
    "scope-kind",
    "scope-id",
    "token-limit",
    "amount-micros-limit",
    "alert-id",
    "pricing-version",
    "price-input-per-million",
    "price-output-per-million",
    "price-cache-read-per-million",
    "price-cache-write-per-million",
    "allow-private-network",
    "allow-insecure-http",
    "reconfirm",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderCommand {
    Account(AccountCommand),
    Key(KeyCommand),
    Models(ModelsCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountCommand {
    Create(AccountCreateOptions),
    List(StatusOptions),
    Show(InspectOptions),
    Update(AccountUpdateOptions),
    Delete(InspectOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCommand {
    Set(KeyMaterialOptions),
    Rotate(KeyMaterialOptions),
    Remove(InspectOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelsCommand {
    Refresh(InspectOptions),
    List(InspectOptions),
    Add(ModelAddOptions),
    SetPrice(ModelPriceOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingCommand {
    Set(BindingSetOptions),
    Show(BindingShowOptions),
    List(StatusOptions),
    Remove(BindingShowOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectOptions {
    pub status: StatusOptions,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountCreateOptions {
    pub status: StatusOptions,
    pub name: String,
    pub provider_kind: String,
    pub endpoint: Option<String>,
    pub api_key_file: Option<PathBuf>,
    pub allow_private_network: bool,
    pub allow_insecure_http: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountUpdateOptions {
    pub status: StatusOptions,
    pub id: String,
    pub endpoint: Option<String>,
    pub allow_private_network: bool,
    pub allow_insecure_http: bool,
    pub reconfirm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMaterialOptions {
    pub status: StatusOptions,
    pub id: String,
    pub api_key_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAddOptions {
    pub status: StatusOptions,
    pub account_id: String,
    pub model_id: String,
    pub pricing_version: Option<String>,
    pub price_input_per_million: Option<String>,
    pub price_output_per_million: Option<String>,
    pub price_cache_read_per_million: Option<String>,
    pub price_cache_write_per_million: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPriceOptions {
    pub status: StatusOptions,
    pub account_id: String,
    pub model_id: String,
    pub pricing_version: String,
    pub price_input_per_million: Option<String>,
    pub price_output_per_million: Option<String>,
    pub price_cache_read_per_million: Option<String>,
    pub price_cache_write_per_million: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingSetOptions {
    pub status: StatusOptions,
    pub agent: String,
    pub account_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingShowOptions {
    pub status: StatusOptions,
    pub agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetSetOptions {
    pub status: StatusOptions,
    pub budget_id: Option<String>,
    pub scope_kind: String,
    pub scope_id: String,
    pub token_limit: Option<String>,
    pub amount_micros_limit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetCommand {
    Set(BudgetSetOptions),
    List(StatusOptions),
    Remove(InspectOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertCommand {
    List(StatusOptions),
    Acknowledge(InspectOptions),
}

pub fn parse_provider_args(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((group, rest)) = arguments.split_first() else {
        return Err("provider requires account|key|models".to_owned());
    };
    match group.as_str() {
        "account" => parse_account(rest),
        "key" => parse_key(rest),
        "models" => parse_models(rest),
        other => Err(format!(
            "unknown provider group `{other}` (expected account|key|models)"
        )),
    }
}

pub fn parse_agent_args(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((group, rest)) = arguments.split_first() else {
        return Err("agent requires binding".to_owned());
    };
    if group != "binding" {
        return Err("agent requires subcommand binding".to_owned());
    }
    let Some((op, flags_args)) = rest.split_first() else {
        return Err("agent binding requires set|show|list|remove".to_owned());
    };
    let flags = parse_control_flags(flags_args)?;
    let status = parse_status_options(&flags)?;
    match op.as_str() {
        "list" => Ok(CognitiveCommand::AgentBinding(BindingCommand::List(status))),
        "set" => Ok(CognitiveCommand::AgentBinding(BindingCommand::Set(
            BindingSetOptions {
                status,
                agent: required(&flags, "agent")?,
                account_id: required(&flags, "account-id")?,
                model_id: required(&flags, "model-id")?,
            },
        ))),
        "show" => Ok(CognitiveCommand::AgentBinding(BindingCommand::Show(
            BindingShowOptions {
                status,
                agent: required(&flags, "agent")?,
            },
        ))),
        "remove" => Ok(CognitiveCommand::AgentBinding(BindingCommand::Remove(
            BindingShowOptions {
                status,
                agent: required(&flags, "agent")?,
            },
        ))),
        other => Err(format!(
            "unknown agent binding subcommand `{other}` (expected set|show|list|remove)"
        )),
    }
}

pub fn parse_usage_args(arguments: &[String]) -> Result<CognitiveCommand, String> {
    parse_single_query(arguments, "usage", "query", CognitiveCommand::Usage)
}

pub fn parse_audit_args(arguments: &[String]) -> Result<CognitiveCommand, String> {
    parse_single_query(arguments, "audit", "query", CognitiveCommand::Audit)
}

pub fn parse_budget_args(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((op, rest)) = arguments.split_first() else {
        return Err("budget requires set|list|remove".to_owned());
    };
    let flags = parse_control_flags(rest)?;
    let status = parse_status_options(&flags)?;
    match op.as_str() {
        "list" => Ok(CognitiveCommand::Budget(BudgetCommand::List(status))),
        "remove" => Ok(CognitiveCommand::Budget(BudgetCommand::Remove(
            InspectOptions {
                status,
                id: required(&flags, "budget-id")?,
            },
        ))),
        "set" => Ok(CognitiveCommand::Budget(BudgetCommand::Set(
            BudgetSetOptions {
                status,
                budget_id: flags.get("budget-id").cloned(),
                scope_kind: required(&flags, "scope-kind")?,
                scope_id: required(&flags, "scope-id")?,
                token_limit: flags.get("token-limit").cloned(),
                amount_micros_limit: flags.get("amount-micros-limit").cloned(),
            },
        ))),
        other => Err(format!(
            "unknown budget subcommand `{other}` (expected set|list|remove)"
        )),
    }
}

pub fn parse_alerts_args(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((op, rest)) = arguments.split_first() else {
        return Err("alerts requires list|acknowledge".to_owned());
    };
    let flags = parse_control_flags(rest)?;
    let status = parse_status_options(&flags)?;
    match op.as_str() {
        "list" => Ok(CognitiveCommand::Alerts(AlertCommand::List(status))),
        "acknowledge" => Ok(CognitiveCommand::Alerts(AlertCommand::Acknowledge(
            InspectOptions {
                status,
                id: required(&flags, "alert-id")?,
            },
        ))),
        other => Err(format!(
            "unknown alerts subcommand `{other}` (expected list|acknowledge)"
        )),
    }
}

pub fn run_provider(command: ProviderCommand) -> i32 {
    match command {
        ProviderCommand::Account(AccountCommand::List(status)) => with_client(&status, |client| {
            client.get_authorized_public("/management/providers/accounts")
        }),
        ProviderCommand::Account(AccountCommand::Show(options)) => {
            with_client(&options.status, |client| {
                client.get_authorized_public(&format!(
                    "/management/providers/accounts/inspect?id={}",
                    url_query(&options.id)
                ))
            })
        }
        ProviderCommand::Account(AccountCommand::Create(options)) => {
            match account_create_body(&options) {
                Ok(body) => with_client(&options.status, |client| {
                    client.post_authorized_public("/management/providers/accounts", &body)
                }),
                Err(error) => print_operational_error(&error),
            }
        }
        ProviderCommand::Account(AccountCommand::Update(options)) => {
            let mut body = json!({"id": options.id, "reconfirm": options.reconfirm});
            if let Some(endpoint) = &options.endpoint {
                body["endpoint"] = json!(endpoint);
            }
            if options.allow_private_network {
                body["allow_private_network"] = json!(true);
            }
            if options.allow_insecure_http {
                body["allow_insecure_http"] = json!(true);
            }
            with_client(&options.status, |client| {
                client.post_authorized_public(
                    "/management/providers/accounts/update",
                    &body.to_string(),
                )
            })
        }
        ProviderCommand::Account(AccountCommand::Delete(options)) => {
            let body = json!({"id": options.id}).to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/providers/accounts/delete", &body)
            })
        }
        ProviderCommand::Key(KeyCommand::Remove(options)) => {
            let body = json!({"id": options.id, "op": "remove"}).to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/providers/accounts/key", &body)
            })
        }
        ProviderCommand::Key(KeyCommand::Set(options)) => key_op(&options, "set"),
        ProviderCommand::Key(KeyCommand::Rotate(options)) => key_op(&options, "rotate"),
        ProviderCommand::Models(ModelsCommand::List(options)) => {
            with_client(&options.status, |client| {
                client.get_authorized_public(&format!(
                    "/management/providers/models?account_id={}",
                    url_query(&options.id)
                ))
            })
        }
        ProviderCommand::Models(ModelsCommand::Refresh(options)) => {
            let body = json!({"id": options.id}).to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/providers/models/refresh", &body)
            })
        }
        ProviderCommand::Models(ModelsCommand::Add(options)) => {
            let body = model_price_json(
                &options.account_id,
                &options.model_id,
                options.pricing_version.as_deref(),
                options.price_input_per_million.as_deref(),
                options.price_output_per_million.as_deref(),
                options.price_cache_read_per_million.as_deref(),
                options.price_cache_write_per_million.as_deref(),
            )
            .to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/providers/models/add", &body)
            })
        }
        ProviderCommand::Models(ModelsCommand::SetPrice(options)) => {
            let body = model_price_json(
                &options.account_id,
                &options.model_id,
                Some(&options.pricing_version),
                options.price_input_per_million.as_deref(),
                options.price_output_per_million.as_deref(),
                options.price_cache_read_per_million.as_deref(),
                options.price_cache_write_per_million.as_deref(),
            )
            .to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/providers/models/set-price", &body)
            })
        }
    }
}

pub fn run_binding(command: BindingCommand) -> i32 {
    match command {
        BindingCommand::List(status) => with_client(&status, |client| {
            client.get_authorized_public("/management/agent-bindings")
        }),
        BindingCommand::Show(options) => with_client(&options.status, |client| {
            client.get_authorized_public("/management/agent-bindings")
        }),
        BindingCommand::Set(options) => {
            let body = json!({
                "agent": options.agent,
                "account_id": options.account_id,
                "model_id": options.model_id
            })
            .to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/agent-bindings", &body)
            })
        }
        BindingCommand::Remove(options) => {
            let body = json!({"agent": options.agent}).to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/agent-bindings/remove", &body)
            })
        }
    }
}

pub fn run_usage(status: &StatusOptions) -> i32 {
    with_client(status, |client| {
        client.get_authorized_public("/management/usage")
    })
}

pub fn run_audit(status: &StatusOptions) -> i32 {
    with_client(status, |client| {
        client.get_authorized_public("/management/audit")
    })
}

pub fn run_budget(command: BudgetCommand) -> i32 {
    match command {
        BudgetCommand::List(status) => with_client(&status, |client| {
            client.get_authorized_public("/management/budgets")
        }),
        BudgetCommand::Remove(options) => {
            let body = json!({"budget_id": options.id}).to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/budgets/remove", &body)
            })
        }
        BudgetCommand::Set(options) => {
            let mut body = json!({
                "scope_kind": options.scope_kind,
                "scope_id": options.scope_id,
            });
            if let Some(budget_id) = &options.budget_id {
                body["budget_id"] = json!(budget_id);
            }
            if let Some(token_limit) = options
                .token_limit
                .as_deref()
                .and_then(|value| value.parse::<i64>().ok())
            {
                body["token_limit"] = json!(token_limit);
            }
            if let Some(amount) = options
                .amount_micros_limit
                .as_deref()
                .and_then(|value| value.parse::<i64>().ok())
            {
                body["amount_micros_limit"] = json!(amount);
            }
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/budgets", &body.to_string())
            })
        }
    }
}

pub fn run_alerts(command: AlertCommand) -> i32 {
    match command {
        AlertCommand::List(status) => with_client(&status, |client| {
            client.get_authorized_public("/management/alerts")
        }),
        AlertCommand::Acknowledge(options) => {
            let body = json!({"alert_id": options.id}).to_string();
            with_client(&options.status, |client| {
                client.post_authorized_public("/management/alerts/acknowledge", &body)
            })
        }
    }
}

fn parse_account(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((op, rest)) = arguments.split_first() else {
        return Err("provider account requires create|list|show|update|delete".to_owned());
    };
    let flags = parse_control_flags(rest)?;
    let status = parse_status_options(&flags)?;
    match op.as_str() {
        "list" => Ok(CognitiveCommand::Provider(ProviderCommand::Account(
            AccountCommand::List(status),
        ))),
        "show" => Ok(CognitiveCommand::Provider(ProviderCommand::Account(
            AccountCommand::Show(InspectOptions {
                status,
                id: required(&flags, "id")?,
            }),
        ))),
        "delete" => Ok(CognitiveCommand::Provider(ProviderCommand::Account(
            AccountCommand::Delete(InspectOptions {
                status,
                id: required(&flags, "id")?,
            }),
        ))),
        "create" => Ok(CognitiveCommand::Provider(ProviderCommand::Account(
            AccountCommand::Create(AccountCreateOptions {
                status,
                name: required(&flags, "name")?,
                provider_kind: required(&flags, "provider-kind")?,
                endpoint: flags.get("endpoint-url").cloned(),
                api_key_file: flags.get("api-key-file").map(PathBuf::from),
                allow_private_network: flag_bool(&flags, "allow-private-network")?,
                allow_insecure_http: flag_bool(&flags, "allow-insecure-http")?,
            }),
        ))),
        "update" => Ok(CognitiveCommand::Provider(ProviderCommand::Account(
            AccountCommand::Update(AccountUpdateOptions {
                status,
                id: required(&flags, "id")?,
                endpoint: flags.get("endpoint-url").cloned(),
                allow_private_network: flag_bool(&flags, "allow-private-network")?,
                allow_insecure_http: flag_bool(&flags, "allow-insecure-http")?,
                reconfirm: flag_bool(&flags, "reconfirm")?,
            }),
        ))),
        other => Err(format!(
            "unknown provider account subcommand `{other}` (expected create|list|show|update|delete)"
        )),
    }
}

fn parse_key(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((op, rest)) = arguments.split_first() else {
        return Err("provider key requires set|rotate|remove".to_owned());
    };
    let flags = parse_control_flags(rest)?;
    let status = parse_status_options(&flags)?;
    let id = required(&flags, "id")?;
    match op.as_str() {
        "remove" => Ok(CognitiveCommand::Provider(ProviderCommand::Key(
            KeyCommand::Remove(InspectOptions { status, id }),
        ))),
        "set" => Ok(CognitiveCommand::Provider(ProviderCommand::Key(
            KeyCommand::Set(KeyMaterialOptions {
                status,
                id,
                api_key_file: flags.get("api-key-file").map(PathBuf::from),
            }),
        ))),
        "rotate" => Ok(CognitiveCommand::Provider(ProviderCommand::Key(
            KeyCommand::Rotate(KeyMaterialOptions {
                status,
                id,
                api_key_file: flags.get("api-key-file").map(PathBuf::from),
            }),
        ))),
        other => Err(format!(
            "unknown provider key subcommand `{other}` (expected set|rotate|remove)"
        )),
    }
}

fn parse_models(arguments: &[String]) -> Result<CognitiveCommand, String> {
    let Some((op, rest)) = arguments.split_first() else {
        return Err("provider models requires refresh|list|add|set-price".to_owned());
    };
    let flags = parse_control_flags(rest)?;
    let status = parse_status_options(&flags)?;
    match op.as_str() {
        "list" => Ok(CognitiveCommand::Provider(ProviderCommand::Models(
            ModelsCommand::List(InspectOptions {
                status,
                id: required(&flags, "account-id")?,
            }),
        ))),
        "refresh" => Ok(CognitiveCommand::Provider(ProviderCommand::Models(
            ModelsCommand::Refresh(InspectOptions {
                status,
                id: required(&flags, "account-id")?,
            }),
        ))),
        "add" => Ok(CognitiveCommand::Provider(ProviderCommand::Models(
            ModelsCommand::Add(ModelAddOptions {
                status,
                account_id: required(&flags, "account-id")?,
                model_id: required(&flags, "model-id")?,
                pricing_version: flags.get("pricing-version").cloned(),
                price_input_per_million: flags.get("price-input-per-million").cloned(),
                price_output_per_million: flags.get("price-output-per-million").cloned(),
                price_cache_read_per_million: flags.get("price-cache-read-per-million").cloned(),
                price_cache_write_per_million: flags.get("price-cache-write-per-million").cloned(),
            }),
        ))),
        "set-price" => Ok(CognitiveCommand::Provider(ProviderCommand::Models(
            ModelsCommand::SetPrice(ModelPriceOptions {
                status,
                account_id: required(&flags, "account-id")?,
                model_id: required(&flags, "model-id")?,
                pricing_version: flags
                    .get("pricing-version")
                    .cloned()
                    .unwrap_or_else(|| "manual".to_owned()),
                price_input_per_million: flags.get("price-input-per-million").cloned(),
                price_output_per_million: flags.get("price-output-per-million").cloned(),
                price_cache_read_per_million: flags.get("price-cache-read-per-million").cloned(),
                price_cache_write_per_million: flags.get("price-cache-write-per-million").cloned(),
            }),
        ))),
        other => Err(format!(
            "unknown provider models subcommand `{other}` (expected refresh|list|add|set-price)"
        )),
    }
}

fn parse_single_query(
    arguments: &[String],
    verb: &str,
    expected: &str,
    wrap: fn(StatusOptions) -> CognitiveCommand,
) -> Result<CognitiveCommand, String> {
    let Some((op, rest)) = arguments.split_first() else {
        return Err(format!("{verb} requires {expected}"));
    };
    if op != expected {
        return Err(format!("{verb} requires {expected}"));
    }
    let flags = parse_control_flags(rest)?;
    Ok(wrap(parse_status_options(&flags)?))
}

fn parse_control_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let flags = parse_flags(args)?;
    reject_unexpected_flags(&flags, CONTROL_PLANE_FLAGS)?;
    if flags.contains_key("api-key") {
        return Err("--api-key is not accepted; use --api-key-file".to_owned());
    }
    Ok(flags)
}

fn required(flags: &BTreeMap<String, String>, name: &str) -> Result<String, String> {
    flags
        .get(name)
        .cloned()
        .ok_or_else(|| format!("missing --{name}"))
}

fn with_client(
    status: &StatusOptions,
    call: impl FnOnce(&PersonalDaemonClient) -> Result<String, PersonalDaemonClientError>,
) -> i32 {
    match connect_resource_client(status) {
        Ok(client) => print_redacted(call(&client)),
        Err(code) => code,
    }
}

fn print_redacted(result: Result<String, PersonalDaemonClientError>) -> i32 {
    match result {
        Ok(body) => {
            println!("{}", redact_cli_body(&body));
            EXIT_SUCCESS
        }
        Err(error) => print_operational_error(&redact_cli_body(&error.to_string())),
    }
}

fn redact_cli_body(body: &str) -> String {
    let mut redacted = body.to_owned();
    for needle in ["sk-", "Bearer ", "bearer ", "x-api-key"] {
        if let Some(index) = redacted
            .to_ascii_lowercase()
            .find(&needle.to_ascii_lowercase())
        {
            let _ = index;
            redacted = redacted.replace(needle, "[redacted]");
        }
    }
    redacted
}

fn account_create_body(options: &AccountCreateOptions) -> Result<String, String> {
    let mut body = json!({
        "display_name": options.name,
        "provider_kind": options.provider_kind,
        "allow_private_network": options.allow_private_network,
        "allow_insecure_http": options.allow_insecure_http,
    });
    if let Some(endpoint) = &options.endpoint {
        body["endpoint"] = json!(endpoint);
    }
    if options.api_key_file.is_some() {
        let material = read_api_key_material(options.api_key_file.as_deref())?;
        let key = String::from_utf8(material.expose_bytes().to_vec())
            .map_err(|_| "api key material is not UTF-8".to_owned())?;
        body["api_key"] = json!(key);
    }
    Ok(body.to_string())
}

fn key_op(options: &KeyMaterialOptions, op: &str) -> i32 {
    let material = match read_api_key_material(options.api_key_file.as_deref()) {
        Ok(material) => material,
        Err(error) => return print_operational_error(&error),
    };
    let key = match String::from_utf8(material.expose_bytes().to_vec()) {
        Ok(key) => key,
        Err(_) => return print_operational_error("api key material is not UTF-8"),
    };
    let body = json!({"id": options.id, "op": op, "api_key": key}).to_string();
    with_client(&options.status, |client| {
        client.post_authorized_public("/management/providers/accounts/key", &body)
    })
}

fn model_price_json(
    account_id: &str,
    model_id: &str,
    pricing_version: Option<&str>,
    input: Option<&str>,
    output: Option<&str>,
    cache_read: Option<&str>,
    cache_write: Option<&str>,
) -> Value {
    let mut body = json!({"account_id": account_id, "model_id": model_id});
    if let Some(version) = pricing_version {
        body["pricing_version"] = json!(version);
    }
    if let Some(value) = input {
        body["price_input_per_million"] = json!(value);
    }
    if let Some(value) = output {
        body["price_output_per_million"] = json!(value);
    }
    if let Some(value) = cache_read {
        body["price_cache_read_per_million"] = json!(value);
    }
    if let Some(value) = cache_write {
        body["price_cache_write_per_million"] = json!(value);
    }
    body
}

fn url_query(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
}
