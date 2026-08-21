//! Non-secret Personal dsh configuration and fail-closed launch client (P8-T10).
//!
//! Installs DeepSeek Harness as a candidate-only AKP agent path: pin, adapter
//! registration digest, and launch through `cognitive dsh`, not a one-shot
//! harness script. The CLI is not an authority writer. A dsh response is never
//! Task completion.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cognitive_runtime::{
    AuthorityChannel, DSH_ADAPTER_ID, DSH_PACKAGE_REVISION, activate_dsh_lifecycle,
    bind_dsh_package_identity, open_dsh_lifecycle, register_dsh_adapter,
};
use serde_json::{Value, json};

use super::LayoutRoots;
use super::client::PersonalDaemonClient;
use super::layout::build_layout;

const DAEMON_ENDPOINT_FILE_NAME: &str = "daemon-endpoint.json";
const DAEMON_ENDPOINT_SCHEMA_VERSION: u64 = 1;
const DAEMON_ENDPOINT_SURFACE: &str = "personal-daemon-endpoint";
const DSH_CONFIG_FILE_NAME: &str = "dsh.json";
const DSH_CONFIG_SCHEMA_VERSION: u64 = 1;
const DSH_CONFIG_SURFACE: &str = "personal-dsh-config";
const DSH_REVISION_FILE_NAME: &str = ".cognitiveos-dsh-revision";
const PERSONAL_DOCTOR_SCHEMA_VERSION: u64 = 1;
const PERSONAL_DOCTOR_SURFACE: &str = "personal-doctor";
const REQUIRED_DSH_COMPONENTS: [&str; 5] = ["system", "database", "secret", "provider", "daemon"];

/// Inputs accepted by `cognitive dsh configure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshConfigureOptions {
    pub layout_roots: LayoutRoots,
    pub dsh_root: PathBuf,
    pub adapter_root: PathBuf,
    pub revision: String,
}

/// Inputs accepted by `cognitive dsh launch`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshLaunchOptions {
    pub layout_roots: LayoutRoots,
    pub print_mode: bool,
    pub provider_path: DshProviderPath,
    pub task: Option<String>,
}

/// Path A is dsh → DeepSeek Flash direct. Path B is dsh → AKP → daemon → Flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DshProviderPath {
    Direct,
    Adapter,
}

impl DshProviderPath {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "a",
            Self::Adapter => "b",
        }
    }
}

/// Write non-secret dsh pin + adapter registration digest.
pub fn configure(options: &DshConfigureOptions) -> Result<Value, String> {
    validate_absolute_path(&options.dsh_root, "dsh root")?;
    validate_absolute_path(&options.adapter_root, "dsh AKP adapter root")?;
    if options.revision != DSH_PACKAGE_REVISION {
        return Err(format!(
            "dsh revision must be the exact pin {DSH_PACKAGE_REVISION}"
        ));
    }

    let package = bind_dsh_package_identity(&options.revision).map_err(|error| error.to_string())?;
    let registered =
        register_dsh_adapter(&package, false, false, false).map_err(|error| error.to_string())?;
    let opened = open_dsh_lifecycle(&registered).map_err(|error| error.to_string())?;
    let active = activate_dsh_lifecycle(
        &opened,
        &opened.declaration_digest,
        opened.fencing_epoch,
        AuthorityChannel::Management,
    )
    .map_err(|error| error.to_string())?;

    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    layout
        .ensure_directories()
        .map_err(|error| format!("unable to create Personal configuration directory: {error}"))?;

    fs::create_dir_all(&options.dsh_root)
        .map_err(|error| format!("unable to create dsh root: {error}"))?;
    let revision_pin_path = options.dsh_root.join(DSH_REVISION_FILE_NAME);
    atomic_write(&revision_pin_path, format!("{}\n", options.revision).as_bytes())?;

    let configuration_path = layout.config_dir().join(DSH_CONFIG_FILE_NAME);
    let document = json!({
        "schema_version": DSH_CONFIG_SCHEMA_VERSION,
        "surface": DSH_CONFIG_SURFACE,
        "dsh_root": options.dsh_root,
        "adapter_root": options.adapter_root,
        "revision": options.revision,
        "adapter_id": DSH_ADAPTER_ID,
        "declaration_digest": active.declaration_digest,
        "lifecycle_state": "active",
        "candidate_only": true,
        "gate_claim": "not-claimed",
        "authority_writer": false,
    });
    let serialized = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("unable to serialize non-secret dsh configuration: {error}"))?;
    atomic_write(&configuration_path, &serialized)?;

    Ok(json!({
        "status": "ok",
        "surface": "cognitive-dsh-configure",
        "action": "configured",
        "config_path": configuration_path,
        "adapter_id": DSH_ADAPTER_ID,
        "revision": DSH_PACKAGE_REVISION,
        "declaration_digest": active.declaration_digest,
        "candidate_only": true,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false,
    }))
}

/// Launch the configured dsh agent after daemon-owned readiness admission.
pub fn launch(options: &DshLaunchOptions) -> Result<Value, String> {
    if options.provider_path == DshProviderPath::Direct {
        return Err(
            "direct Flash path is measurement-only; run packages/dsh-akp-adapter/scripts/paired-path.mjs"
                .to_owned(),
        );
    }
    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    let endpoint_document = fs::read_to_string(layout.state_dir().join(DAEMON_ENDPOINT_FILE_NAME))
        .map_err(|_| "daemon endpoint is absent; run `cognitive daemon start`".to_owned())?;
    let endpoint = parse_loopback_endpoint(&endpoint_document)?;
    let doctor_document = PersonalDaemonClient::connect(&endpoint, &layout)
        .and_then(|client| client.get_doctor())
        .map_err(|_| "daemon readiness is unavailable; run `cognitive doctor`".to_owned())?;
    let launch_plan =
        prepare_launch_with_doctor_document(options, &endpoint_document, &doctor_document)?;

    let mut child_process = spawn_dsh_helper(&launch_plan)?;
    let action = if options.print_mode {
        let exit_status = child_process
            .wait()
            .map_err(|_| "dsh non-interactive conversation could not be joined".to_owned())?;
        if !exit_status.success() {
            return Err("dsh non-interactive conversation exited unsuccessfully".to_owned());
        }
        "completed"
    } else {
        "spawned"
    };

    Ok(json!({
        "status": "ok",
        "surface": "cognitive-dsh-launch",
        "action": action,
        "process_id": child_process.id(),
        "adapter_id": DSH_ADAPTER_ID,
        "revision": DSH_PACKAGE_REVISION,
        "provider_path": options.provider_path.as_str(),
        "candidate_only": true,
        "dsh_response_is_not_task_completion": true,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false,
        "conversation_claim": "not-claimed",
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DshLaunchPlan {
    node_executable: PathBuf,
    helper_path: PathBuf,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
}

fn prepare_launch_with_doctor_document(
    options: &DshLaunchOptions,
    endpoint_document: &str,
    doctor_document: &str,
) -> Result<DshLaunchPlan, String> {
    let endpoint = parse_loopback_endpoint(endpoint_document)?;
    validate_doctor_readiness(doctor_document)?;

    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    let dsh_document = fs::read_to_string(layout.config_dir().join(DSH_CONFIG_FILE_NAME))
        .map_err(|_| "dsh configuration is absent; run `cognitive dsh configure`".to_owned())?;
    let (dsh_root, adapter_root, revision) = parse_dsh_configuration(&dsh_document)?;
    if revision != DSH_PACKAGE_REVISION {
        return Err(format!(
            "configured dsh revision does not match the required {DSH_PACKAGE_REVISION} pin"
        ));
    }
    verify_revision_pin(&dsh_root, &revision)?;
    validate_existing_file(
        &adapter_root.join("scripts/dsh-real-process.mjs"),
        "dsh real-process helper",
    )?;
    validate_existing_file(&adapter_root.join("plugin.bundle.cjs"), "dsh AKP plugin")?;

    let bootstrap_path = layout.local_bootstrap_secret_path();
    validate_existing_file(&bootstrap_path, "daemon bootstrap secret")?;
    let port = endpoint
        .rsplit(':')
        .next()
        .ok_or_else(|| "daemon endpoint has no port".to_owned())?;

    let mut arguments = vec![
        path_to_argument(&adapter_root.join("scripts/dsh-real-process.mjs"))?,
        "--port".to_owned(),
        port.to_owned(),
        "--bootstrap-file".to_owned(),
        path_to_argument(&bootstrap_path)?,
        "--revision".to_owned(),
        revision,
        "--dsh-root".to_owned(),
        path_to_argument(&dsh_root)?,
        "--adapter-root".to_owned(),
        path_to_argument(&adapter_root)?,
        "--provider-path".to_owned(),
        options.provider_path.as_str().to_owned(),
    ];
    if let Some(task) = &options.task {
        arguments.push("--task".to_owned());
        arguments.push(task.clone());
    }

    Ok(DshLaunchPlan {
        node_executable: PathBuf::from("node"),
        helper_path: adapter_root.join("scripts/dsh-real-process.mjs"),
        arguments,
        environment: execution_environment_for_layout_roots(&options.layout_roots)?,
    })
}

fn parse_dsh_configuration(document: &str) -> Result<(PathBuf, PathBuf, String), String> {
    let value: Value = serde_json::from_str(document)
        .map_err(|_| "dsh configuration document is corrupt".to_owned())?;
    let object = value
        .as_object()
        .ok_or_else(|| "dsh configuration document must be an object".to_owned())?;
    if object.get("schema_version").and_then(Value::as_u64) != Some(DSH_CONFIG_SCHEMA_VERSION)
        || object.get("surface").and_then(Value::as_str) != Some(DSH_CONFIG_SURFACE)
    {
        return Err("dsh configuration document has an unsupported contract".to_owned());
    }
    if object.get("adapter_id").and_then(Value::as_str) != Some(DSH_ADAPTER_ID) {
        return Err("dsh configuration adapter id is not deepseek.dsh.akp".to_owned());
    }
    if object.get("candidate_only").and_then(Value::as_bool) != Some(true) {
        return Err("dsh configuration must remain candidate-only".to_owned());
    }
    let dsh_root = object
        .get("dsh_root")
        .and_then(Value::as_str)
        .ok_or_else(|| "dsh configuration has no dsh_root".to_owned())?;
    let adapter_root = object
        .get("adapter_root")
        .and_then(Value::as_str)
        .ok_or_else(|| "dsh configuration has no adapter_root".to_owned())?;
    let revision = object
        .get("revision")
        .and_then(Value::as_str)
        .ok_or_else(|| "dsh configuration has no revision".to_owned())?;
    Ok((
        PathBuf::from(dsh_root),
        PathBuf::from(adapter_root),
        revision.to_owned(),
    ))
}

fn verify_revision_pin(dsh_root: &Path, expected: &str) -> Result<(), String> {
    let pin_path = dsh_root.join(DSH_REVISION_FILE_NAME);
    let pinned = fs::read_to_string(&pin_path)
        .map_err(|_| "dsh revision pin file is absent; run `cognitive dsh configure`".to_owned())?;
    if pinned.trim() != expected {
        return Err("dsh revision pin file does not match the required pin".to_owned());
    }
    let git_dir = dsh_root.join(".git");
    if git_dir.exists() {
        let output = Command::new("git")
            .args(["-C", &path_to_argument(dsh_root)?, "rev-parse", "HEAD"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|_| "unable to execute git rev-parse for the configured dsh root".to_owned())?;
        if !output.status.success() {
            return Err("configured dsh root is not a git checkout".to_owned());
        }
        let head = String::from_utf8_lossy(&output.stdout);
        if head.trim() != expected {
            return Err("configured dsh git HEAD does not match the required pin".to_owned());
        }
    }
    Ok(())
}

fn parse_loopback_endpoint(document: &str) -> Result<String, String> {
    let endpoint_document: Value = serde_json::from_str(document)
        .map_err(|_| "daemon endpoint document is corrupt".to_owned())?;
    let endpoint_object = endpoint_document
        .as_object()
        .ok_or_else(|| "daemon endpoint document must be an object".to_owned())?;
    if endpoint_object
        .get("schema_version")
        .and_then(Value::as_u64)
        != Some(DAEMON_ENDPOINT_SCHEMA_VERSION)
        || endpoint_object.get("surface").and_then(Value::as_str) != Some(DAEMON_ENDPOINT_SURFACE)
    {
        return Err("daemon endpoint document has an unsupported contract".to_owned());
    }
    let endpoint = endpoint_object
        .get("endpoint")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "daemon endpoint document has no endpoint".to_owned())?;
    let socket_address = endpoint
        .parse::<std::net::SocketAddr>()
        .map_err(|_| "daemon endpoint must be a numeric loopback socket address".to_owned())?;
    if !socket_address.ip().is_loopback() {
        return Err("daemon endpoint must be loopback".to_owned());
    }
    Ok(endpoint.to_owned())
}

fn validate_doctor_readiness(document: &str) -> Result<(), String> {
    let doctor_document: Value = serde_json::from_str(document)
        .map_err(|_| "daemon doctor projection is corrupt".to_owned())?;
    let doctor_object = doctor_document
        .as_object()
        .ok_or_else(|| "daemon doctor projection must be an object".to_owned())?;
    if doctor_object.get("schema_version").and_then(Value::as_u64)
        != Some(PERSONAL_DOCTOR_SCHEMA_VERSION)
        || doctor_object.get("surface").and_then(Value::as_str) != Some(PERSONAL_DOCTOR_SURFACE)
    {
        return Err("daemon doctor projection has an unsupported contract".to_owned());
    }
    if doctor_object.get("overall").and_then(Value::as_str) != Some("ready") {
        return Err("daemon is not ready for a dsh agent launch".to_owned());
    }
    let components = doctor_object
        .get("components")
        .and_then(Value::as_array)
        .ok_or_else(|| "daemon doctor projection has no component checks".to_owned())?;
    let mut component_statuses = BTreeMap::new();
    for component in components {
        let object = component
            .as_object()
            .ok_or_else(|| "daemon doctor component is not an object".to_owned())?;
        let component_name = object
            .get("component")
            .and_then(Value::as_str)
            .ok_or_else(|| "daemon doctor component has no name".to_owned())?;
        let component_status = object
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| "daemon doctor component has no status".to_owned())?;
        component_statuses.insert(component_name.to_owned(), component_status.to_owned());
    }
    for required in REQUIRED_DSH_COMPONENTS {
        if component_statuses.get(required).map(String::as_str) != Some("ready") {
            return Err(format!(
                "daemon doctor component `{required}` is not ready for dsh launch"
            ));
        }
    }
    Ok(())
}

fn spawn_dsh_helper(launch_plan: &DshLaunchPlan) -> Result<std::process::Child, String> {
    Command::new(&launch_plan.node_executable)
        .args(&launch_plan.arguments)
        .env_clear()
        .envs(&launch_plan.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| {
            format!(
                "unable to execute configured dsh helper at {}",
                launch_plan.helper_path.display()
            )
        })
}

fn validate_absolute_path(path: &Path, label: &str) -> Result<(), String> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(format!("{label} path must be absolute"))
}

fn validate_existing_file(path: &Path, label: &str) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{label} is missing or is not a regular file"))
    }
}

fn path_to_argument(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "dsh path is not valid Unicode".to_owned())
}

fn execution_environment_for_layout_roots(
    layout_roots: &LayoutRoots,
) -> Result<BTreeMap<String, String>, String> {
    let mut environment = BTreeMap::new();
    for key in [
        "PATH", "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL", "TZ", "TMPDIR", "TMP", "TEMP",
        "PNPM_HOME", "COREPACK_HOME", "SystemRoot", "WINDIR", "ComSpec", "PATHEXT",
    ] {
        if let Ok(value) = std::env::var(key) {
            environment.insert(key.to_owned(), value);
        }
    }
    if let Some(runtime_root) = &layout_roots.runtime_root {
        environment.insert(
            "XDG_CONFIG_HOME".to_owned(),
            path_to_argument(&runtime_root.join("config"))?,
        );
        environment.insert(
            "XDG_DATA_HOME".to_owned(),
            path_to_argument(&runtime_root.join("data"))?,
        );
        environment.insert(
            "XDG_STATE_HOME".to_owned(),
            path_to_argument(&runtime_root.join("state"))?,
        );
    }
    Ok(environment)
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent_directory = path
        .parent()
        .ok_or_else(|| "dsh configuration has no parent directory".to_owned())?;
    fs::create_dir_all(parent_directory)
        .map_err(|error| format!("unable to create dsh configuration parent: {error}"))?;
    let temporary_path = parent_directory.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("dsh"),
        std::process::id()
    ));
    fs::write(&temporary_path, contents).map_err(|error| {
        format!(
            "unable to write temporary dsh configuration at {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        format!(
            "unable to atomically publish dsh configuration at {}: {error}",
            path.display()
        )
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn ready_doctor_without_pi() -> String {
        json!({
            "schema_version": PERSONAL_DOCTOR_SCHEMA_VERSION,
            "surface": PERSONAL_DOCTOR_SURFACE,
            "overall": "ready",
            "first_conversation_ready": false,
            "components": [
                {"component": "system", "status": "ready"},
                {"component": "database", "status": "ready"},
                {"component": "secret", "status": "ready"},
                {"component": "provider", "status": "ready"},
                {"component": "daemon", "status": "ready"},
                {"component": "pi", "status": "not_configured"}
            ]
        })
        .to_string()
    }

    fn endpoint_document() -> String {
        json!({
            "schema_version": DAEMON_ENDPOINT_SCHEMA_VERSION,
            "surface": DAEMON_ENDPOINT_SURFACE,
            "endpoint": "127.0.0.1:48181"
        })
        .to_string()
    }

    fn write_helper_tree(root: &Path) {
        fs::create_dir_all(root.join("scripts")).expect("scripts");
        fs::write(
            root.join("scripts/dsh-real-process.mjs"),
            "export {}\n",
        )
        .expect("helper");
        fs::write(root.join("plugin.bundle.cjs"), "module.exports = {}\n").expect("plugin");
    }

    #[test]
    fn configuration_rejects_relative_paths_and_wrong_revision() {
        let relative = configure(&DshConfigureOptions {
            layout_roots: LayoutRoots { runtime_root: None },
            dsh_root: PathBuf::from("dsh"),
            adapter_root: PathBuf::from("adapter"),
            revision: DSH_PACKAGE_REVISION.to_owned(),
        });
        assert!(
            relative.as_ref().expect_err("relative path").contains("absolute"),
            "{relative:?}"
        );

        let temporary = TempDir::new().expect("temp");
        let wrong = configure(&DshConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary.path().to_path_buf()),
            },
            dsh_root: temporary.path().join("dsh"),
            adapter_root: temporary.path().join("adapter"),
            revision: "0000000000000000000000000000000000000000".to_owned(),
        });
        assert!(
            wrong.as_ref().expect_err("wrong revision").contains("exact pin"),
            "{wrong:?}"
        );
    }

    #[test]
    fn configuration_registers_candidate_only_adapter_and_writes_pin() {
        let temporary = TempDir::new().expect("temp");
        let dsh_root = temporary.path().join("dsh");
        let adapter_root = temporary.path().join("adapter");
        let report = configure(&DshConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary.path().to_path_buf()),
            },
            dsh_root: dsh_root.clone(),
            adapter_root: adapter_root.clone(),
            revision: DSH_PACKAGE_REVISION.to_owned(),
        })
        .expect("configure");
        assert_eq!(report["adapter_id"], DSH_ADAPTER_ID);
        assert_eq!(report["candidate_only"], true);
        assert_eq!(report["gate_claim"], "not-claimed");
        let config_path = temporary.path().join("config/cognitiveos/dsh.json");
        let document = fs::read_to_string(config_path).expect("dsh.json");
        assert!(document.contains(DSH_ADAPTER_ID));
        assert!(document.contains("candidate_only"));
        let pin = fs::read_to_string(dsh_root.join(DSH_REVISION_FILE_NAME)).expect("pin");
        assert_eq!(pin.trim(), DSH_PACKAGE_REVISION);
    }

    #[test]
    fn launch_preparation_accepts_ready_daemon_without_pi() {
        let temporary = TempDir::new().expect("temp");
        let dsh_root = temporary.path().join("dsh");
        let adapter_root = temporary.path().join("adapter");
        write_helper_tree(&adapter_root);
        configure(&DshConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary.path().to_path_buf()),
            },
            dsh_root,
            adapter_root,
            revision: DSH_PACKAGE_REVISION.to_owned(),
        })
        .expect("configure");
        fs::write(
            temporary.path().join("cognitiveos/local-bootstrap.secret"),
            "test-bootstrap-secret\n",
        )
        .expect("bootstrap");
        let plan = prepare_launch_with_doctor_document(
            &DshLaunchOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(temporary.path().to_path_buf()),
                },
                print_mode: true,
                provider_path: DshProviderPath::Adapter,
                task: Some("Reply with the single word pong and nothing else.".to_owned()),
            },
            &endpoint_document(),
            &ready_doctor_without_pi(),
        )
        .expect("prepare");
        assert!(plan.arguments.contains(&"--provider-path".to_owned()));
        assert!(plan.arguments.contains(&"b".to_owned()));
        assert!(plan.arguments.contains(&DSH_PACKAGE_REVISION.to_owned()));
    }

    #[test]
    fn launch_preparation_rejects_missing_config_and_unready_secret() {
        let temporary = TempDir::new().expect("temp");
        let missing = prepare_launch_with_doctor_document(
            &DshLaunchOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(temporary.path().to_path_buf()),
                },
                print_mode: false,
                provider_path: DshProviderPath::Adapter,
                task: None,
            },
            &endpoint_document(),
            &ready_doctor_without_pi(),
        );
        assert!(
            missing.as_ref().expect_err("missing config").contains("dsh configuration is absent"),
            "{missing:?}"
        );

        let dsh_root = temporary.path().join("dsh");
        let adapter_root = temporary.path().join("adapter");
        write_helper_tree(&adapter_root);
        configure(&DshConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary.path().to_path_buf()),
            },
            dsh_root,
            adapter_root,
            revision: DSH_PACKAGE_REVISION.to_owned(),
        })
        .expect("configure");
        let mut doctor: Value = serde_json::from_str(&ready_doctor_without_pi()).expect("doctor");
        doctor["components"][2]["status"] = json!("not_ready");
        let unready = prepare_launch_with_doctor_document(
            &DshLaunchOptions {
                layout_roots: LayoutRoots {
                    runtime_root: Some(temporary.path().to_path_buf()),
                },
                print_mode: false,
                provider_path: DshProviderPath::Direct,
                task: None,
            },
            &endpoint_document(),
            &doctor.to_string(),
        );
        assert!(
            unready.as_ref().expect_err("unready secret").contains("`secret` is not ready"),
            "{unready:?}"
        );
    }

    #[test]
    fn launch_rejects_direct_flash_path() {
        let error = launch(&DshLaunchOptions {
            layout_roots: LayoutRoots {
                runtime_root: None,
            },
            print_mode: true,
            provider_path: DshProviderPath::Direct,
            task: None,
        })
        .expect_err("direct path must stay measurement-only");
        assert!(error.contains("measurement-only"), "{error}");
    }
}
