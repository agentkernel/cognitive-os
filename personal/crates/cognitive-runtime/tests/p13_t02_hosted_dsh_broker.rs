#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T02 hosted DSH stdio broker against **real** child processes (`node`).
//!
//! Negatives: process death (exit 0 / non-zero / timeout kill / spawn failure)
//! is never completion; unknown output is never success; secret-shaped env or
//! argv refuses the spawn; a child that points at a Provider directly is
//! refused and recorded; native MCP / base tool / HMR / home patch are refused;
//! the bounded Context reaches the child over stdin only; the child inherits
//! an allowlisted environment, never the daemon's. All of it is fenced on
//! `DEV-WIN-GNU-01`; Windows sandbox / ACL / supply-chain stays `P13-T13`.

use cognitive_runtime::{
    HOSTED_DSH_CONFIG_FILE_NAME, HOSTED_DSH_REVISION_FILE_NAME, HOSTED_FRAME_PROTOCOL,
    HostedBrokerError, HostedChildLaunchPlan, HostedContextPayload, HostedDshArtifact,
    HostedFrameKind, HostedTerminalKind, isolated_spawn_is_fenced, run_hosted_child,
    validate_launch_plan,
};
use cognitive_store::HOSTED_DSH_ARTIFACT_DIGEST;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

fn node_plan(script: &str) -> HostedChildLaunchPlan {
    let mut plan = HostedChildLaunchPlan::new("node", vec!["-e".to_owned(), script.to_owned()]);
    plan.env = cognitive_runtime::inherited_child_environment();
    plan.timeout = Duration::from_secs(20);
    plan
}

fn payload(context: &str) -> HostedContextPayload {
    HostedContextPayload {
        attempt_id: "dshattempt-test".to_owned(),
        task_ref: "task://personal/p13-t02".to_owned(),
        employee_id: "employee-test".to_owned(),
        project_id: "project-test".to_owned(),
        bounded_context: context.to_owned(),
        daemon_origin: Some("http://127.0.0.1:48181".to_owned()),
        bootstrap_file: None,
    }
}

fn run(plan: &HostedChildLaunchPlan, context: &str) -> cognitive_runtime::HostedChildRun {
    let mut spawned_pid = None;
    let run = run_hosted_child(plan, &payload(context), |pid| spawned_pid = Some(pid))
        .expect("broker run");
    if run.terminal != HostedTerminalKind::SpawnFailed {
        assert!(spawned_pid.is_some(), "on_spawn must observe the pid");
        assert_eq!(run.pid, spawned_pid);
    }
    run
}

/// Reads the request frame from stdin and echoes selected fields back.
const ECHO_CHILD: &str = r#"
let data = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { data += chunk; });
process.stdin.on("end", () => {
  const request = JSON.parse(data.trim().split("\n")[0]);
  process.stdout.write(JSON.stringify({ frame: "observation", text: "protocol:" + request.protocol }) + "\n");
  process.stdout.write(JSON.stringify({ frame: "observation", text: "digest:" + request.context_digest }) + "\n");
  process.stdout.write(JSON.stringify({ frame: "observation", text: "context:" + request.context }) + "\n");
  process.stdout.write(JSON.stringify({ frame: "observation", text: "env:" + Object.keys(process.env).sort().join(",") }) + "\n");
  process.stdout.write(JSON.stringify({ frame: "heartbeat" }) + "\n");
  process.stdout.write(JSON.stringify({ frame: "candidate", operation: "DeliverableDraft", payload: { text: "draft" } }) + "\n");
  process.stdout.write(JSON.stringify({ frame: "response", status: "done" }) + "\n");
  process.exit(0);
});
"#;

#[test]
fn p13_t02_bounded_context_reaches_child_over_stdin_and_env_is_allowlisted() {
    let plan = node_plan(ECHO_CHILD);
    if isolated_spawn_is_fenced() {
        assert_eq!(validate_launch_plan(&plan), Err(HostedBrokerError::Fenced));
        return;
    }
    let context = "summarize README.md in one sentence";
    let run = run(&plan, context);
    assert_eq!(run.terminal, HostedTerminalKind::Exited { code: 0 });
    assert!(!run.completion_claimed());
    assert_eq!(run.response_status.as_deref(), Some("done"));
    assert_eq!(run.candidate_count(), 1);
    assert_eq!(run.observation_count(), 5);
    assert!(run.rejected_frames.is_empty());
    assert_eq!(run.unknown_lines, 0);
    let texts: Vec<&str> = run
        .frames
        .iter()
        .map(|frame| frame.text_redacted.as_str())
        .collect();
    assert!(texts.contains(&format!("protocol:{HOSTED_FRAME_PROTOCOL}").as_str()));
    assert!(texts.contains(&format!("digest:{}", payload(context).context_digest()).as_str()));
    assert_eq!(run.context_digest, payload(context).context_digest());
    assert!(texts.contains(&format!("context:{context}").as_str()));
    let env_line = texts
        .iter()
        .find(|text| text.starts_with("env:"))
        .expect("env observation");
    // cargo sets CARGO_* in the test process; an isolated child must not see them.
    assert!(!env_line.contains("CARGO_MANIFEST_DIR"), "{env_line}");
    assert!(!env_line.contains("CARGO_PKG_NAME"), "{env_line}");
    assert!(
        !env_line.to_ascii_lowercase().contains("secret"),
        "{env_line}"
    );
    assert!(
        !env_line.to_ascii_lowercase().contains("token"),
        "{env_line}"
    );
    let candidate = run
        .frames
        .iter()
        .find(|frame| frame.kind == HostedFrameKind::Candidate)
        .expect("candidate");
    assert_eq!(candidate.operation.as_deref(), Some("DeliverableDraft"));
    assert!(candidate.payload_digest.is_some());
    let ledger = run.ledger_frames();
    assert_eq!(ledger.len(), 7);
    assert!(ledger.windows(2).all(|pair| pair[0].seq < pair[1].seq));
}

#[test]
fn p13_t02_process_death_is_never_completion() {
    if isolated_spawn_is_fenced() {
        return;
    }
    let clean = run(&node_plan("process.exit(0)"), "do the task");
    assert_eq!(clean.terminal, HostedTerminalKind::Exited { code: 0 });
    assert!(!clean.completion_claimed());
    assert!(clean.response_status.is_none());
    assert_eq!(clean.candidate_count(), 0);

    let failed = run(&node_plan("process.exit(7)"), "do the task");
    assert_eq!(failed.terminal, HostedTerminalKind::Exited { code: 7 });
    assert_eq!(failed.terminal.exit_code(), Some(7));
    assert!(!failed.completion_claimed());

    let done_then_die = run(
        &node_plan(
            r#"process.stdout.write(JSON.stringify({frame:"response",status:"done"})+"\n"); process.exit(0);"#,
        ),
        "do the task",
    );
    assert_eq!(done_then_die.response_status.as_deref(), Some("done"));
    assert!(!done_then_die.completion_claimed());
    assert!(!done_then_die.terminal.implies_completion());

    let mut sleeper = node_plan(
        r#"process.stdout.write(JSON.stringify({frame:"heartbeat"})+"\n"); setTimeout(() => {}, 60000);"#,
    );
    sleeper.timeout = Duration::from_millis(600);
    let started = std::time::Instant::now();
    let timed_out = run(&sleeper, "do the task");
    assert_eq!(timed_out.terminal, HostedTerminalKind::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "kill must reap"
    );
    assert!(!timed_out.completion_claimed());
    assert_eq!(timed_out.observation_count(), 1);

    let mut missing = node_plan("process.exit(0)");
    missing.program = Path::new("/definitely/not/a/program/cognitiveos-hosted-child").into();
    let spawn_failed = run(&missing, "do the task");
    assert_eq!(spawn_failed.terminal, HostedTerminalKind::SpawnFailed);
    assert!(spawn_failed.pid.is_none());
    assert!(!spawn_failed.completion_claimed());
    assert!(spawn_failed.stderr_tail_redacted.contains("spawn failed"));
}

#[test]
fn p13_t02_unknown_child_output_is_never_success() {
    if isolated_spawn_is_fenced() {
        return;
    }
    let script = r#"
process.stdout.write("ok\n");
process.stdout.write("success\n");
process.stdout.write(JSON.stringify({status:"success"})+"\n");
process.stdout.write(JSON.stringify({frame:"task_complete"})+"\n");
process.stdout.write(JSON.stringify({frame:"effect",operation:"WorkspaceWrite"})+"\n");
process.stdout.write(JSON.stringify({frame:"candidate",payload:{}})+"\n");
process.stdout.write(JSON.stringify({frame:"response",status:"success"})+"\n");
process.exit(0);
"#;
    let run = run(&node_plan(script), "do the task");
    assert_eq!(run.terminal, HostedTerminalKind::Exited { code: 0 });
    assert_eq!(run.unknown_lines, 3);
    assert_eq!(run.rejected_frames.len(), 3);
    assert!(
        run.rejected_frames
            .iter()
            .any(|frame| frame.reason == "child-cannot-emit-authority-frame")
    );
    assert!(
        run.rejected_frames
            .iter()
            .any(|frame| frame.reason == "candidate-without-operation")
    );
    assert_eq!(run.candidate_count(), 0);
    assert_eq!(run.response_status.as_deref(), Some("unknown"));
    assert!(!run.completion_claimed());
}

#[test]
fn p13_t02_child_direct_provider_is_refused_and_recorded() {
    if isolated_spawn_is_fenced() {
        return;
    }
    // The launch-plan validator refuses Provider hosts, `Authorization` and
    // bearer shapes in argv itself (proven below), so the child assembles
    // them at runtime.
    let script = r#"
const deepseek = "https://api." + "deepseek.com/v1/chat/completions";
const openai = "https://api." + "openai.com/v1/chat";
const header = "Author" + "ization: Bea" + "rer abc.def";
process.stdout.write(JSON.stringify({frame:"provider_request",url:deepseek})+"\n");
process.stdout.write(JSON.stringify({frame:"candidate",operation:"HttpFetch",payload:{url:openai}})+"\n");
process.stdout.write(JSON.stringify({frame:"observation",provider_direct:true,text:"tried"})+"\n");
process.stdout.write(JSON.stringify({frame:"observation",text:header})+"\n");
process.exit(0);
"#;
    let run = run(&node_plan(script), "do the task");
    assert_eq!(run.rejected_frames.len(), 3);
    assert!(
        run.rejected_frames
            .iter()
            .all(|frame| frame.reason == "child-direct-provider")
    );
    assert_eq!(run.frames.len(), 1);
    assert!(run.frames[0].text_redacted.contains("Bearer [redacted]"));
    assert!(!run.frames[0].text_redacted.contains("abc.def"));
    let ledger = run.ledger_frames();
    assert_eq!(ledger.len(), 4);
    assert_eq!(
        ledger
            .iter()
            .filter(|frame| frame.kind == "rejected")
            .count(),
        3
    );

    // Launch plans that would point the child at a Provider never spawn.
    let mut direct = node_plan("process.exit(0)");
    direct.args.push("--direct-base-url".to_owned());
    direct.args.push("https://api.deepseek.com".to_owned());
    assert!(matches!(
        run_hosted_child(&direct, &payload("x"), |_| {}),
        Err(HostedBrokerError::DirectProvider { .. })
    ));
    let mut path_a = node_plan("process.exit(0)");
    path_a.args.push("--provider-path".to_owned());
    path_a.args.push("a".to_owned());
    assert!(matches!(
        run_hosted_child(&path_a, &payload("x"), |_| {}),
        Err(HostedBrokerError::DirectProvider { .. })
    ));
    let mut origin = payload("x");
    origin.daemon_origin = Some("https://api.deepseek.com".to_owned());
    assert!(matches!(
        run_hosted_child(&node_plan("process.exit(0)"), &origin, |_| {}),
        Err(HostedBrokerError::DirectProvider { .. })
    ));
}

#[test]
fn p13_t02_secret_env_argv_and_native_escape_never_spawn() {
    if isolated_spawn_is_fenced() {
        return;
    }
    let marker_dir = TempDir::new().expect("temp");
    let marker = marker_dir.path().join("spawned.marker");
    let script = format!(
        "require('node:fs').writeFileSync({}, 'spawned'); process.exit(0);",
        json!(marker.display().to_string())
    );
    let mut with_env = node_plan(&script);
    with_env
        .env
        .insert("OPENAI_API_KEY".to_owned(), "sk-not-real".to_owned());
    assert_eq!(
        run_hosted_child(&with_env, &payload("x"), |_| {}),
        Err(HostedBrokerError::SecretMaterial { surface: "env" })
    );
    let mut with_argv = node_plan(&script);
    with_argv.args.push("--token".to_owned());
    assert_eq!(
        run_hosted_child(&with_argv, &payload("x"), |_| {}),
        Err(HostedBrokerError::SecretMaterial { surface: "argv" })
    );
    let mut with_value = node_plan(&script);
    with_value
        .env
        .insert("DSH_NOTE".to_owned(), "Bearer abc.def".to_owned());
    assert_eq!(
        run_hosted_child(&with_value, &payload("x"), |_| {}),
        Err(HostedBrokerError::SecretMaterial { surface: "env" })
    );
    let mut escape = node_plan(&script);
    escape.args.push("--mcp".to_owned());
    assert_eq!(
        run_hosted_child(&escape, &payload("x"), |_| {}),
        Err(HostedBrokerError::NativeHarnessEscape)
    );
    let mut digest = node_plan(&script);
    digest.artifact_digest = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_owned();
    assert_eq!(
        run_hosted_child(&digest, &payload("x"), |_| {}),
        Err(HostedBrokerError::ArtifactDigestMismatch)
    );
    let oversize = "x".repeat(cognitive_runtime::HOSTED_CONTEXT_MAX_BYTES + 1);
    assert!(matches!(
        run_hosted_child(&node_plan(&script), &payload(&oversize), |_| {}),
        Err(HostedBrokerError::ContextTooLarge { .. })
    ));
    assert_eq!(
        run_hosted_child(&node_plan(&script), &payload("   "), |_| {}),
        Err(HostedBrokerError::ContextEmpty)
    );
    assert!(!marker.exists(), "no refused plan may reach the OS");
}

#[test]
fn p13_t02_stdout_is_bounded_and_child_still_drains() {
    if isolated_spawn_is_fenced() {
        return;
    }
    let script = r#"
for (let i = 0; i < 4000; i += 1) {
  process.stdout.write(JSON.stringify({frame:"observation",text:"line " + i + " " + "x".repeat(200)})+"\n");
}
process.stdout.write(JSON.stringify({frame:"response",status:"failed"})+"\n");
process.exit(3);
"#;
    let mut plan = node_plan(script);
    plan.max_stdout_bytes = 8 * 1024;
    plan.max_frames = 16;
    let run = run(&plan, "do the task");
    assert_eq!(run.terminal, HostedTerminalKind::Exited { code: 3 });
    assert!(run.stdout_truncated);
    assert!(run.frames.len() <= 16);
    assert!(run.stdout_bytes > 8 * 1024);
    assert!(!run.completion_claimed());
}

fn write_config(config_dir: &Path, dsh_root: &Path, adapter_root: &Path, revision: &str) {
    fs::create_dir_all(config_dir).expect("config dir");
    fs::write(
        config_dir.join(HOSTED_DSH_CONFIG_FILE_NAME),
        json!({
            "schema_version": 1,
            "surface": "personal-dsh-config",
            "dsh_root": dsh_root.display().to_string(),
            "adapter_root": adapter_root.display().to_string(),
            "revision": revision,
            "adapter_id": "deepseek.dsh.akp",
            "candidate_only": true,
        })
        .to_string(),
    )
    .expect("config");
}

#[test]
fn p13_t02_artifact_observation_reports_health_and_never_spawns() {
    let temporary = TempDir::new().expect("temp");
    let config_dir = temporary.path().join("config");
    let dsh_root = temporary.path().join("dsh");
    let adapter_root = temporary.path().join("adapter");
    fs::create_dir_all(&dsh_root).expect("dsh root");
    fs::create_dir_all(adapter_root.join("scripts")).expect("adapter root");

    let absent = HostedDshArtifact::observe(&config_dir);
    assert_eq!(absent.health, "absent");
    assert!(absent.configured_revision.is_none());

    fs::create_dir_all(&config_dir).expect("config dir");
    fs::write(config_dir.join(HOSTED_DSH_CONFIG_FILE_NAME), "{not json").expect("corrupt");
    assert_eq!(HostedDshArtifact::observe(&config_dir).health, "corrupt");

    write_config(
        &config_dir,
        &dsh_root,
        &adapter_root,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
    );
    let mismatch = HostedDshArtifact::observe(&config_dir);
    assert_eq!(mismatch.health, "mismatch");
    assert_eq!(
        mismatch.configured_revision.as_deref(),
        Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
    );
    assert!(matches!(
        HostedDshArtifact::resolve(&config_dir),
        Err(HostedBrokerError::ArtifactDigestMismatch)
    ));

    write_config(
        &config_dir,
        &dsh_root,
        &adapter_root,
        HOSTED_DSH_ARTIFACT_DIGEST,
    );
    assert_eq!(HostedDshArtifact::observe(&config_dir).health, "absent");
    fs::write(
        dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME),
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n",
    )
    .expect("pin");
    assert_eq!(HostedDshArtifact::observe(&config_dir).health, "mismatch");
    fs::write(
        dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME),
        format!("{HOSTED_DSH_ARTIFACT_DIGEST}\n"),
    )
    .expect("pin");
    let script_missing = HostedDshArtifact::observe(&config_dir);
    assert_eq!(script_missing.health, "script-missing");
    assert!(script_missing.child_script_digest.is_none());
    assert!(matches!(
        HostedDshArtifact::resolve(&config_dir),
        Err(HostedBrokerError::ArtifactUnavailable { .. })
    ));

    fs::write(
        adapter_root.join("scripts/hosted-attempt-child.mjs"),
        "process.exit(0);\n",
    )
    .expect("script");
    let pinned = HostedDshArtifact::observe(&config_dir);
    assert_eq!(pinned.health, "pinned");
    assert_eq!(
        pinned.pin_file_revision.as_deref(),
        Some(HOSTED_DSH_ARTIFACT_DIGEST)
    );
    assert_eq!(
        pinned.child_script_digest.as_deref().map(str::len),
        Some(64)
    );
    let artifact = HostedDshArtifact::resolve(&config_dir).expect("resolve");
    assert_eq!(
        artifact.child_script_digest,
        pinned.child_script_digest.unwrap()
    );
    let plan = artifact.launch_plan(Duration::from_secs(5));
    assert_eq!(plan.program, Path::new("node"));
    assert_eq!(plan.cwd.as_deref(), Some(dsh_root.as_path()));
    assert!(plan.args.iter().any(|arg| arg == "--provider-path"));
    assert!(plan.args.iter().any(|arg| arg == "b"));
    assert!(plan.args.iter().all(|arg| arg != "--api-key-file"));
    assert!(plan.env.keys().all(|key| {
        let lowered = key.to_ascii_lowercase();
        !lowered.contains("key") && !lowered.contains("secret") && !lowered.contains("token")
    }));
    if isolated_spawn_is_fenced() {
        assert_eq!(validate_launch_plan(&plan), Err(HostedBrokerError::Fenced));
    } else {
        validate_launch_plan(&plan).expect("product plan validates");
    }
}
