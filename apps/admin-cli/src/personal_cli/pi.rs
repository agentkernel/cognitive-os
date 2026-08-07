//! Non-secret Personal Pi configuration and fail-closed launch client.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::LayoutRoots;
use super::client::PersonalDaemonClient;
use super::layout::build_layout;

const DAEMON_ENDPOINT_FILE_NAME: &str = "daemon-endpoint.json";
const DAEMON_ENDPOINT_SCHEMA_VERSION: u64 = 1;
const DAEMON_ENDPOINT_SURFACE: &str = "personal-daemon-endpoint";
const PI_CONFIG_FILE_NAME: &str = "pi.json";
const PI_CONFIG_SCHEMA_VERSION: u64 = 1;
const PI_CONFIG_SURFACE: &str = "personal-pi-config";
const PERSONAL_DOCTOR_SCHEMA_VERSION: u64 = 1;
const PERSONAL_DOCTOR_SURFACE: &str = "personal-doctor";
const PINNED_PI_VERSION: &str = "0.81.1";
const PI_VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const REQUIRED_READINESS_COMPONENTS: [&str; 6] =
    ["system", "database", "secret", "provider", "daemon", "pi"];

/// Inputs accepted by `cognitive pi launch`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiLaunchOptions {
    pub layout_roots: LayoutRoots,
}

/// Inputs accepted by `cognitive pi configure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiConfigureOptions {
    pub layout_roots: LayoutRoots,
    pub executable_path: PathBuf,
    pub extension_entry_path: PathBuf,
    pub candidate_adapter_path: Option<PathBuf>,
    pub candidate_extension_entry_path: Option<PathBuf>,
}

/// Write the Personal Pi configuration without consulting Provider state.
///
/// The daemon owns subsequent file observation and version validation. This
/// client operation deliberately does not start Pi, access a SecretStore, or
/// inspect Provider/authority state.
pub fn configure(options: &PiConfigureOptions) -> Result<Value, String> {
    validate_absolute_path(&options.executable_path, "Pi executable")?;
    validate_absolute_path(&options.extension_entry_path, "CognitiveOS Extension entry")?;
    if let Some(path) = &options.candidate_adapter_path {
        validate_absolute_path(path, "private candidate adapter")?;
    }
    if let Some(path) = &options.candidate_extension_entry_path {
        validate_absolute_path(path, "private candidate extension")?;
    }
    if options.candidate_adapter_path.is_some() != options.candidate_extension_entry_path.is_some()
    {
        return Err(
            "private candidate configuration requires both adapter and extension paths".to_owned(),
        );
    }

    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    layout
        .ensure_directories()
        .map_err(|error| format!("unable to create Personal configuration directory: {error}"))?;

    let configuration_path = layout.config_dir().join(PI_CONFIG_FILE_NAME);
    let mut document = json!({
        "schema_version": PI_CONFIG_SCHEMA_VERSION,
        "surface": PI_CONFIG_SURFACE,
        "executable_path": options.executable_path,
        "extension_entry_path": options.extension_entry_path,
    });
    let document_object = document
        .as_object_mut()
        .ok_or_else(|| "unable to construct non-secret Pi configuration".to_owned())?;
    if let Some(path) = &options.candidate_adapter_path {
        document_object.insert("candidate_adapter_path".to_owned(), json!(path));
    }
    if let Some(path) = &options.candidate_extension_entry_path {
        document_object.insert("candidate_extension_entry_path".to_owned(), json!(path));
    }
    let serialized = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("unable to serialize non-secret Pi configuration: {error}"))?;
    atomic_write_configuration(&configuration_path, &serialized)?;

    Ok(json!({
        "status": "ok",
        "surface": "cognitive-pi-configure",
        "action": "configured",
        "config_path": configuration_path,
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false,
    }))
}

/// Start the configured Pi client only after daemon-owned readiness admission.
///
/// The daemon remains the owner of SecretStore, Provider, selected-model, and
/// authority state checks. This client reads no Provider configuration or
/// secret material and gives Pi only its configured Extension entrypoint.
pub fn launch(options: &PiLaunchOptions) -> Result<Value, String> {
    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    let endpoint_document = fs::read_to_string(layout.state_dir().join(DAEMON_ENDPOINT_FILE_NAME))
        .map_err(|_| "daemon endpoint is absent; run `cognitive daemon start`".to_owned())?;
    let endpoint = parse_loopback_endpoint(&endpoint_document)?;
    let doctor_document = PersonalDaemonClient::connect(&endpoint, &layout)
        .and_then(|client| client.get_doctor())
        .map_err(|_| "daemon readiness is unavailable; run `cognitive doctor`".to_owned())?;
    let launch_plan =
        prepare_launch_with_doctor_document(options, &endpoint_document, &doctor_document)?;

    verify_pinned_pi_version(&launch_plan)?;
    let child_process = spawn_pi_client(&launch_plan)?;

    Ok(json!({
        "status": "ok",
        "surface": "cognitive-pi-launch",
        "action": "spawned",
        "process_id": child_process.id(),
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed",
        "authority_side_effects": false,
        "conversation_claim": "not-claimed",
        "extension_load_claim": "not-claimed",
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PiLaunchPlan {
    executable_path: PathBuf,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
}

fn prepare_launch_with_doctor_document(
    options: &PiLaunchOptions,
    endpoint_document: &str,
    doctor_document: &str,
) -> Result<PiLaunchPlan, String> {
    parse_loopback_endpoint(endpoint_document)?;
    validate_doctor_readiness(doctor_document)?;

    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    let pi_document = fs::read_to_string(layout.config_dir().join(PI_CONFIG_FILE_NAME))
        .map_err(|_| "Pi configuration is absent; run `cognitive pi configure`".to_owned())?;
    let (executable_path, extension_entry_path) = parse_pi_configuration(&pi_document)?;
    validate_existing_file(&executable_path, "Pi executable")?;
    validate_existing_file(&extension_entry_path, "CognitiveOS Extension entry")?;

    Ok(PiLaunchPlan {
        executable_path,
        arguments: vec![
            "--extension".to_owned(),
            path_to_argument(&extension_entry_path)?,
        ],
        environment: minimal_execution_environment(),
    })
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
    if doctor_object.get("overall").and_then(Value::as_str) != Some("ready")
        || doctor_object
            .get("first_conversation_ready")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return Err("daemon is not ready for a first conversation".to_owned());
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
        if component_statuses
            .insert(component_name, component_status)
            .is_some()
        {
            return Err("daemon doctor projection repeats a component".to_owned());
        }
    }
    if REQUIRED_READINESS_COMPONENTS
        .iter()
        .any(|component| component_statuses.get(component) != Some(&"ready"))
    {
        return Err("daemon readiness prerequisites are incomplete".to_owned());
    }
    Ok(())
}

fn parse_pi_configuration(document: &str) -> Result<(PathBuf, PathBuf), String> {
    let configuration: Value =
        serde_json::from_str(document).map_err(|_| "Pi configuration is corrupt".to_owned())?;
    let object = configuration
        .as_object()
        .ok_or_else(|| "Pi configuration must be an object".to_owned())?;
    let supported_fields: BTreeSet<&str> = [
        "schema_version",
        "surface",
        "executable_path",
        "extension_entry_path",
        "candidate_adapter_path",
        "candidate_extension_entry_path",
    ]
    .into_iter()
    .collect();
    if !object
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .is_subset(&supported_fields)
        || object.get("schema_version").and_then(Value::as_u64) != Some(PI_CONFIG_SCHEMA_VERSION)
        || object.get("surface").and_then(Value::as_str) != Some(PI_CONFIG_SURFACE)
    {
        return Err("Pi configuration has an unsupported contract".to_owned());
    }
    let executable_path =
        required_absolute_config_path(object, "executable_path", "Pi executable")?;
    let extension_entry_path = required_absolute_config_path(
        object,
        "extension_entry_path",
        "CognitiveOS Extension entry",
    )?;
    for (field, label) in [
        ("candidate_adapter_path", "private candidate adapter"),
        (
            "candidate_extension_entry_path",
            "private candidate extension",
        ),
    ] {
        if object.contains_key(field) {
            required_absolute_config_path(object, field, label)?;
        }
    }
    if object.contains_key("candidate_adapter_path")
        != object.contains_key("candidate_extension_entry_path")
    {
        return Err(
            "Pi configuration requires both private candidate adapter and extension paths"
                .to_owned(),
        );
    }
    Ok((executable_path, extension_entry_path))
}

fn required_absolute_config_path(
    object: &serde_json::Map<String, Value>,
    field: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let path = object
        .get(field)
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| format!("Pi configuration has no {field}"))?;
    validate_absolute_path(&path, label)?;
    Ok(path)
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
        .ok_or_else(|| "Pi path is not valid Unicode".to_owned())
}

fn minimal_execution_environment() -> BTreeMap<String, String> {
    const EXECUTION_ENVIRONMENT_ALLOWLIST: [&str; 11] = [
        "HOME",
        "LOGNAME",
        "PATH",
        "TMP",
        "TEMP",
        "TMPDIR",
        "USER",
        "XDG_CACHE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_RUNTIME_DIR",
    ];
    EXECUTION_ENVIRONMENT_ALLOWLIST
        .into_iter()
        .filter_map(|variable| {
            std::env::var(variable)
                .ok()
                .map(|value| (variable.to_owned(), value))
        })
        .collect()
}

fn verify_pinned_pi_version(launch_plan: &PiLaunchPlan) -> Result<(), String> {
    let mut version_process = Command::new(&launch_plan.executable_path)
        .arg("--version")
        .env_clear()
        .envs(&launch_plan.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "unable to execute configured Pi version probe".to_owned())?;
    let probe_deadline = Instant::now() + PI_VERSION_PROBE_TIMEOUT;
    let exit_status = loop {
        match version_process.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < probe_deadline => {
                std::thread::sleep(Duration::from_millis(20))
            }
            Ok(None) => {
                let _ = version_process.kill();
                let _ = version_process.wait();
                return Err("configured Pi version probe timed out".to_owned());
            }
            Err(_) => return Err("unable to observe configured Pi version probe".to_owned()),
        }
    };
    let version_output = version_process
        .stdout
        .take()
        .ok_or_else(|| "configured Pi version probe produced no output".to_owned())?;
    let mut version_text = String::new();
    std::io::Read::read_to_string(
        &mut std::io::BufReader::new(version_output),
        &mut version_text,
    )
    .map_err(|_| "unable to read configured Pi version probe output".to_owned())?;
    if !exit_status.success() || version_text.trim() != PINNED_PI_VERSION {
        return Err("configured Pi version does not match the required 0.81.1 pin".to_owned());
    }
    Ok(())
}

fn spawn_pi_client(launch_plan: &PiLaunchPlan) -> Result<std::process::Child, String> {
    Command::new(&launch_plan.executable_path)
        .args(&launch_plan.arguments)
        .env_clear()
        .envs(&launch_plan.environment)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| "unable to launch configured Pi client".to_owned())
}

fn validate_absolute_path(path: &std::path::Path, label: &str) -> Result<(), String> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(format!("{label} path must be absolute"))
}

fn atomic_write_configuration(path: &std::path::Path, contents: &[u8]) -> Result<(), String> {
    let parent_directory = path
        .parent()
        .ok_or_else(|| "Pi configuration has no parent directory".to_owned())?;
    let temporary_path =
        parent_directory.join(format!(".{PI_CONFIG_FILE_NAME}.{}.tmp", std::process::id()));
    fs::write(&temporary_path, contents).map_err(|error| {
        format!(
            "unable to write temporary Pi configuration at {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        format!(
            "unable to atomically publish Pi configuration at {}: {error}",
            path.display()
        )
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    const LOOPBACK_ENDPOINT_DOCUMENT: &str =
        r#"{"schema_version":1,"surface":"personal-daemon-endpoint","endpoint":"127.0.0.1:48181"}"#;

    fn ready_doctor_document() -> String {
        let components = REQUIRED_READINESS_COMPONENTS
            .iter()
            .map(|component| json!({ "component": component, "status": "ready" }))
            .collect::<Vec<_>>();
        json!({
            "schema_version": PERSONAL_DOCTOR_SCHEMA_VERSION,
            "surface": PERSONAL_DOCTOR_SURFACE,
            "overall": "ready",
            "first_conversation_ready": true,
            "components": components,
        })
        .to_string()
    }

    fn launch_options(temporary_root: &tempfile::TempDir) -> PiLaunchOptions {
        PiLaunchOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary_root.path().to_path_buf()),
            },
        }
    }

    fn write_launch_configuration(
        temporary_root: &tempfile::TempDir,
        executable_path: &Path,
        extension_entry_path: &Path,
    ) {
        let configuration_directory = temporary_root.path().join("config/cognitiveos");
        fs::create_dir_all(&configuration_directory).expect("configuration directory");
        fs::write(
            configuration_directory.join(PI_CONFIG_FILE_NAME),
            json!({
                "schema_version": PI_CONFIG_SCHEMA_VERSION,
                "surface": PI_CONFIG_SURFACE,
                "executable_path": executable_path,
                "extension_entry_path": extension_entry_path,
            })
            .to_string(),
        )
        .expect("Pi configuration");
    }

    #[test]
    fn configuration_rejects_relative_paths_before_writing_any_file() {
        let options = PiConfigureOptions {
            layout_roots: LayoutRoots { runtime_root: None },
            executable_path: PathBuf::from("pi"),
            extension_entry_path: PathBuf::from("extension.js"),
            candidate_adapter_path: None,
            candidate_extension_entry_path: None,
        };

        let error = configure(&options).expect_err("relative paths must be rejected");

        assert!(error.contains("absolute"), "{error}");
    }

    #[test]
    fn configuration_writes_only_the_documented_non_secret_pi_fields() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let executable_path = temporary_root.path().join("bin").join("pi");
        let extension_entry_path = temporary_root.path().join("extension").join("index.js");
        let options = PiConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary_root.path().to_path_buf()),
            },
            executable_path: executable_path.clone(),
            extension_entry_path: extension_entry_path.clone(),
            candidate_adapter_path: None,
            candidate_extension_entry_path: None,
        };

        let report = configure(&options).expect("write non-secret Pi configuration");
        let configuration_path = temporary_root.path().join("config/cognitiveos/pi.json");
        let document: Value =
            serde_json::from_slice(&fs::read(&configuration_path).expect("read Pi configuration"))
                .expect("parse Pi configuration");

        assert_eq!(document["schema_version"], PI_CONFIG_SCHEMA_VERSION);
        assert_eq!(document["surface"], PI_CONFIG_SURFACE);
        assert_eq!(
            document["executable_path"],
            executable_path.display().to_string()
        );
        assert_eq!(
            document["extension_entry_path"],
            extension_entry_path.display().to_string()
        );
        assert_eq!(document.as_object().expect("object").len(), 4);
        assert!(!document.to_string().contains("secret"));
        assert!(!document.to_string().contains("provider"));
        assert!(!document.to_string().contains("sqlite"));
        assert_eq!(report["authority_side_effects"], false);
        assert_eq!(report["gate_claim"], "not-claimed");
    }

    #[test]
    fn configuration_persists_complete_private_candidate_paths() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let options = PiConfigureOptions {
            layout_roots: LayoutRoots {
                runtime_root: Some(temporary_root.path().to_path_buf()),
            },
            executable_path: temporary_root.path().join("bin/pi"),
            extension_entry_path: temporary_root.path().join("extension/index.js"),
            candidate_adapter_path: Some(temporary_root.path().join("bin/pi-agent-adapter")),
            candidate_extension_entry_path: Some(
                temporary_root
                    .path()
                    .join("extension/private-candidate.mjs"),
            ),
        };
        configure(&options).expect("candidate configuration must persist");
        let document: Value = serde_json::from_slice(
            &fs::read(temporary_root.path().join("config/cognitiveos/pi.json"))
                .expect("written configuration"),
        )
        .expect("valid configuration JSON");
        assert_eq!(
            document["candidate_adapter_path"],
            json!(options.candidate_adapter_path.clone())
        );
        assert_eq!(
            document["candidate_extension_entry_path"],
            json!(options.candidate_extension_entry_path.clone())
        );

        let incomplete = PiConfigureOptions {
            candidate_extension_entry_path: None,
            ..options
        };
        assert!(configure(&incomplete).is_err());
    }

    #[test]
    fn launch_preparation_rejects_a_non_loopback_daemon_endpoint_before_provider_access() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let options = launch_options(&temporary_root);

        let error = prepare_launch_with_doctor_document(
            &options,
            r#"{"schema_version":1,"surface":"personal-daemon-endpoint","endpoint":"203.0.113.9:48181"}"#,
            r#"{}"#,
        )
        .expect_err("Pi launch must reject a non-loopback daemon endpoint");

        assert!(error.contains("loopback"), "{error}");
    }

    #[test]
    fn launch_preparation_rejects_missing_or_unready_daemon_prerequisites() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let options = launch_options(&temporary_root);

        let corrupt_endpoint = prepare_launch_with_doctor_document(&options, "not json", "{}");
        assert!(
            corrupt_endpoint.is_err(),
            "corrupt endpoint must fail closed"
        );

        let unready_doctor = json!({
            "schema_version": PERSONAL_DOCTOR_SCHEMA_VERSION,
            "surface": PERSONAL_DOCTOR_SURFACE,
            "overall": "blocked",
            "first_conversation_ready": false,
            "components": [],
        })
        .to_string();
        let error = prepare_launch_with_doctor_document(
            &options,
            LOOPBACK_ENDPOINT_DOCUMENT,
            &unready_doctor,
        )
        .expect_err("selected-model, Provider, or SecretStore failure must block launch");

        assert!(error.contains("not ready"), "{error}");
    }

    #[test]
    fn launch_preparation_rejects_invalid_paths_and_missing_pi_files() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let options = launch_options(&temporary_root);
        let configuration_directory = temporary_root.path().join("config/cognitiveos");
        fs::create_dir_all(&configuration_directory).expect("configuration directory");
        fs::write(
            configuration_directory.join(PI_CONFIG_FILE_NAME),
            "not json",
        )
        .expect("corrupt Pi configuration");
        let doctor_document = ready_doctor_document();

        let corrupt_error = prepare_launch_with_doctor_document(
            &options,
            LOOPBACK_ENDPOINT_DOCUMENT,
            &doctor_document,
        )
        .expect_err("corrupt Pi configuration must block launch");
        assert!(corrupt_error.contains("corrupt"), "{corrupt_error}");

        write_launch_configuration(
            &temporary_root,
            Path::new("relative-pi"),
            Path::new("relative-extension"),
        );
        let relative_error = prepare_launch_with_doctor_document(
            &options,
            LOOPBACK_ENDPOINT_DOCUMENT,
            &doctor_document,
        )
        .expect_err("relative Pi configuration must block launch");
        assert!(relative_error.contains("absolute"), "{relative_error}");

        let missing_executable = temporary_root.path().join("missing-pi");
        let missing_extension = temporary_root.path().join("missing-extension");
        write_launch_configuration(&temporary_root, &missing_executable, &missing_extension);
        let missing_error = prepare_launch_with_doctor_document(
            &options,
            LOOPBACK_ENDPOINT_DOCUMENT,
            &doctor_document,
        )
        .expect_err("missing Pi files must block launch");
        assert!(missing_error.contains("missing"), "{missing_error}");
    }

    #[test]
    fn launch_plan_passes_only_extension_and_an_allowlisted_environment() {
        let temporary_root = tempfile::tempdir().expect("temporary root");
        let executable_path = temporary_root.path().join("pi");
        let extension_entry_path = temporary_root.path().join("index.js");
        fs::write(&executable_path, "placeholder").expect("Pi executable");
        fs::write(&extension_entry_path, "placeholder").expect("Extension entry");
        write_launch_configuration(&temporary_root, &executable_path, &extension_entry_path);

        let launch_plan = prepare_launch_with_doctor_document(
            &launch_options(&temporary_root),
            LOOPBACK_ENDPOINT_DOCUMENT,
            &ready_doctor_document(),
        )
        .expect("ready daemon and valid non-secret configuration prepare launch only");

        assert_eq!(
            launch_plan.arguments,
            vec![
                "--extension",
                extension_entry_path.to_str().expect("UTF-8 path")
            ]
        );
        assert!(!launch_plan.environment.contains_key("DEEPSEEK_API_KEY"));
        assert!(
            !launch_plan
                .environment
                .contains_key("PROVIDER_SECRET_MARKER")
        );
        assert!(
            launch_plan
                .environment
                .keys()
                .all(|key| !key.contains("SECRET") && !key.contains("PROVIDER"))
        );
    }

    #[test]
    fn version_probe_rejects_a_non_pinned_executable() {
        let launch_plan = PiLaunchPlan {
            executable_path: std::env::current_exe().expect("test executable"),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
        };

        let error = verify_pinned_pi_version(&launch_plan)
            .expect_err("a non-Pi executable cannot satisfy the exact Pi version pin");

        assert!(error.contains("required 0.81.1"), "{error}");
    }
}
