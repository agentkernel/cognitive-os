//! `cognitive daemon start|status|stop` process control (client-side).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use cognitive_store::PersonalDataLayout;
use serde_json::{Value, json};

use super::DaemonStartOptions;
use super::StatusOptions;
use super::layout::build_layout;

const ENDPOINT_FILE_NAME: &str = "daemon-endpoint.json";

/// Start `kernel-server --personal` under the resolved layout.
pub fn start(options: &DaemonStartOptions) -> Result<Value, String> {
    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    layout
        .ensure_directories()
        .map_err(|error| format!("unable to create runtime directories: {error}"))?;

    if layout.daemon_lock_path().exists() {
        if process_from_lock_alive(&layout)? {
            let endpoint = load_endpoint(&layout).unwrap_or_else(|_| options.bind_address.clone());
            return Ok(json!({
                "status": "ok",
                "surface": "cognitive-daemon",
                "action": "already_running",
                "endpoint": endpoint,
                "lock_path": layout.daemon_lock_path().display().to_string()
            }));
        }
        return Err(format!(
            "daemon.lock exists at {} but the process is not running; remove the stale lock \
             after confirming no Personal daemon holds it, then retry",
            layout.daemon_lock_path().display()
        ));
    }

    ensure_loopback_bind(&options.bind_address)?;
    let kernel_server = resolve_kernel_server_path(options.kernel_server_path.as_deref())?;
    let runtime_root = resolve_runtime_root_for_spawn(&options.layout_roots, &layout)?;
    write_endpoint(&layout, &options.bind_address)?;

    let mut child =
        spawn_detached_kernel_server(&kernel_server, &options.bind_address, &runtime_root)?;

    // Windows MSVC debug and cold disks can take longer than a tight local loop.
    for _ in 0..250 {
        if layout.daemon_lock_path().exists() && layout.local_bootstrap_secret_path().exists() {
            // Intentionally leak the Child handle so Drop does not close OS process
            // handles while the daemon continues as an independent process.
            let daemon_pid = child.id();
            std::mem::forget(child);
            return Ok(json!({
                "status": "ok",
                "surface": "cognitive-daemon",
                "action": "started",
                "endpoint": options.bind_address,
                "pid": daemon_pid,
                "kernel_server": kernel_server.display().to_string(),
                "lock_path": layout.daemon_lock_path().display().to_string(),
                "profile_claim": "not-claimed",
                "gate_claim": "not-claimed"
            }));
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "kernel-server exited before becoming ready (status {status:?})"
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }

    let _ = child.kill();
    Err(
        "Personal daemon did not publish lock/bootstrap within timeout; check bind address \
         and runtime permissions"
            .to_owned(),
    )
}

/// Report whether the daemon lock and endpoint look live.
pub fn status(options: &StatusOptions) -> Result<Value, String> {
    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    let lock_exists = layout.daemon_lock_path().exists();
    let alive = if lock_exists {
        process_from_lock_alive(&layout)?
    } else {
        false
    };
    let endpoint = load_endpoint(&layout).ok();
    Ok(json!({
        "status": "ok",
        "surface": "cognitive-daemon",
        "action": "status",
        "lock_exists": lock_exists,
        "process_alive": alive,
        "endpoint": endpoint,
        "bootstrap_present": layout.local_bootstrap_secret_path().exists(),
        "profile_claim": "not-claimed",
        "gate_claim": "not-claimed"
    }))
}

/// Stop a running Personal daemon by pid recorded in the lock file.
pub fn stop(options: &StatusOptions) -> Result<Value, String> {
    let layout = build_layout(&options.layout_roots).map_err(|error| error.to_string())?;
    if !layout.daemon_lock_path().exists() {
        return Ok(json!({
            "status": "ok",
            "surface": "cognitive-daemon",
            "action": "already_stopped"
        }));
    }
    let pid = read_lock_pid(&layout)?.ok_or_else(|| {
        format!(
            "daemon.lock at {} has no parseable pid; remove it manually after inspection",
            layout.daemon_lock_path().display()
        )
    })?;
    // SIGTERM/taskkill terminate the OS process without running Rust Drop, so the
    // create-new lock file often remains. Product stop semantics: signal, wait
    // until the recorded pid is dead, then remove the confirmed-stale lock.
    if process_is_alive(pid) {
        terminate_pid(pid)?;
    }
    for _ in 0..150 {
        if !process_is_alive(pid) {
            let lock_was_present = layout.daemon_lock_path().exists();
            if lock_was_present {
                fs::remove_file(layout.daemon_lock_path()).map_err(|error| {
                    format!(
                        "pid {pid} is stopped but unable to remove stale daemon.lock at {}: {error}",
                        layout.daemon_lock_path().display()
                    )
                })?;
            }
            let _ = fs::remove_file(endpoint_path(&layout));
            return Ok(json!({
                "status": "ok",
                "surface": "cognitive-daemon",
                "action": "stopped",
                "pid": pid,
                "stale_lock_removed": lock_was_present,
                "profile_claim": "not-claimed",
                "gate_claim": "not-claimed"
            }));
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "signaled pid {pid} but the process is still alive after timeout; refuse to remove daemon.lock"
    ))
}

/// Load the last published daemon endpoint for a layout.
pub fn load_endpoint(layout: &PersonalDataLayout) -> Result<String, String> {
    let path = endpoint_path(layout);
    let document = fs::read_to_string(&path).map_err(|error| {
        format!(
            "daemon endpoint file missing at {} ({error}); start the daemon with \
             `cognitive daemon start`",
            path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&document)
        .map_err(|error| format!("daemon endpoint file corrupt: {error}"))?;
    value
        .get("endpoint")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "daemon endpoint file missing endpoint field".to_owned())
}

fn write_endpoint(layout: &PersonalDataLayout, endpoint: &str) -> Result<(), String> {
    let path = endpoint_path(layout);
    let document = json!({
        "schema_version": 1,
        "endpoint": endpoint,
        "surface": "personal-daemon-endpoint"
    });
    fs::write(
        &path,
        serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("unable to write endpoint file {}: {error}", path.display()))?;
    Ok(())
}

fn endpoint_path(layout: &PersonalDataLayout) -> PathBuf {
    layout.state_dir().join(ENDPOINT_FILE_NAME)
}

fn ensure_loopback_bind(bind_address: &str) -> Result<(), String> {
    let allowed = bind_address.starts_with("127.")
        || bind_address.starts_with("[::1]")
        || bind_address.starts_with("localhost:");
    if !allowed {
        return Err(
            "daemon bind address must be loopback (127.0.0.0/8, [::1], or localhost)".to_owned(),
        );
    }
    Ok(())
}

fn spawn_detached_kernel_server(
    kernel_server: &Path,
    bind_address: &str,
    runtime_root: &Path,
) -> Result<std::process::Child, String> {
    let runtime_root_text = runtime_root
        .to_str()
        .ok_or_else(|| "runtime root path is not valid UTF-8".to_owned())?;
    let mut command = Command::new(kernel_server);
    command
        .args([
            "--personal",
            "--bind",
            bind_address,
            "--runtime-root",
            runtime_root_text,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Detach so the daemon outlives the short-lived `cognitive daemon start`
    // process under shells and CI job objects (Windows).
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
        command.creation_flags(
            CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS | CREATE_BREAKAWAY_FROM_JOB,
        );
    }

    match command.spawn() {
        Ok(child) => Ok(child),
        Err(first_error) => {
            // Some hosts refuse BREAKAWAY; retry with milder detach flags.
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
                const DETACHED_PROCESS: u32 = 0x0000_0008;
                let mut fallback = Command::new(kernel_server);
                fallback
                    .args([
                        "--personal",
                        "--bind",
                        bind_address,
                        "--runtime-root",
                        runtime_root_text,
                    ])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
                if let Ok(child) = fallback.spawn() {
                    return Ok(child);
                }
            }
            Err(format!(
                "failed to spawn kernel-server from {}: {first_error}",
                kernel_server.display()
            ))
        }
    }
}

fn resolve_kernel_server_path(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(format!(
            "kernel-server path {} does not exist; pass --kernel-server <path>",
            path.display()
        ));
    }
    if let Ok(from_env) = std::env::var("COGNITIVE_KERNEL_SERVER") {
        let path = PathBuf::from(from_env);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "COGNITIVE_KERNEL_SERVER={} is not a file",
            path.display()
        ));
    }
    if let Ok(current) = std::env::current_exe()
        && let Some(dir) = current.parent()
    {
        let candidate = dir.join(if cfg!(windows) {
            "kernel-server.exe"
        } else {
            "kernel-server"
        });
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Ok(PathBuf::from(if cfg!(windows) {
        "kernel-server.exe"
    } else {
        "kernel-server"
    }))
}

fn resolve_runtime_root_for_spawn(
    roots: &super::layout::LayoutRoots,
    layout: &PersonalDataLayout,
) -> Result<PathBuf, String> {
    if let Some(runtime_root) = &roots.runtime_root {
        return Ok(runtime_root.clone());
    }
    layout
        .runtime_dir()
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "unable to derive runtime root parent for kernel-server".to_owned())
}

fn read_lock_pid(layout: &PersonalDataLayout) -> Result<Option<u32>, String> {
    let contents = fs::read_to_string(layout.daemon_lock_path())
        .map_err(|error| format!("unable to read daemon.lock: {error}"))?;
    for token in contents.split_whitespace() {
        if let Some(pid_text) = token.strip_prefix("pid=")
            && let Ok(pid) = pid_text.parse::<u32>()
        {
            return Ok(Some(pid));
        }
    }
    Ok(None)
}

fn process_from_lock_alive(layout: &PersonalDataLayout) -> Result<bool, String> {
    let Some(pid) = read_lock_pid(layout)? else {
        return Ok(false);
    };
    Ok(process_is_alive(pid))
}

fn process_is_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|output| {
                let text = String::from_utf8_lossy(&output.stdout);
                text.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

fn terminate_pid(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|error| format!("taskkill failed: {error}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("taskkill exited with {status:?}"));
    }
    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .args([&pid.to_string()])
            .status()
            .map_err(|error| format!("kill failed: {error}"))?;
        if status.success() {
            return Ok(());
        }
        Err(format!("kill exited with {status:?}"))
    }
}
