use pi_agent_adapter::{PiLaunchPolicy, observed_response_models, redact_secret};

#[test]
fn deepseek_candidate_launch_disables_every_unmediated_pi_surface() -> Result<(), String> {
    let policy = PiLaunchPolicy::deepseek_candidate("deepseek-chat")?;

    assert!(
        policy
            .args()
            .windows(2)
            .any(|pair| pair == ["--provider", "deepseek"])
    );
    assert!(
        policy
            .args()
            .windows(2)
            .any(|pair| pair == ["--model", "deepseek-chat"])
    );
    for required in [
        "--no-tools",
        "--no-extensions",
        "--no-skills",
        "--no-context-files",
        "--no-session",
        "--no-approve",
        "--mode",
        "--print",
    ] {
        assert!(
            policy.args().iter().any(|arg| arg == required),
            "missing {required}"
        );
    }
    assert!(!policy.authority_committed());
    assert!(!policy.effects_created());
    assert_eq!(policy.classification(), "uncontained_candidate_only");
    Ok(())
}

#[test]
fn empty_or_non_deepseek_model_is_rejected_before_process_launch() {
    assert!(PiLaunchPolicy::deepseek_candidate("").is_err());
    assert!(PiLaunchPolicy::deepseek_candidate("openai/gpt-4o").is_err());
}

#[test]
fn candidate_prompt_is_the_only_caller_supplied_process_argument() -> Result<(), String> {
    let policy = PiLaunchPolicy::deepseek_candidate("deepseek-chat")?;
    let args = policy.command_args("Reply with exactly: candidate-ok")?;

    assert_eq!(
        args.last().map(String::as_str),
        Some("Reply with exactly: candidate-ok")
    );
    assert!(!args.iter().any(|arg| arg == "--api-key"));
    assert!(policy.command_args("").is_err());
    Ok(())
}

#[test]
fn process_output_redacts_the_process_scoped_api_key() {
    let secret = "test-deepseek-key";
    let rendered = redact_secret("provider rejected test-deepseek-key", secret);
    assert_eq!(rendered, "provider rejected [REDACTED]");
}

#[test]
fn observed_model_is_extracted_from_pi_json_events_not_inferred_from_request() {
    let output = concat!(
        "{\"type\":\"message_start\",\"responseModel\":\"deepseek-v4-flash\"}\n",
        "{\"type\":\"message_end\",\"responseModel\":\"deepseek-v4-flash\"}\n"
    );
    assert_eq!(observed_response_models(output), vec!["deepseek-v4-flash"]);
}
