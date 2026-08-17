#![allow(clippy::expect_used)]

use pi_agent_adapter::{
    DAEMON_CANDIDATE_FRAME_LIMIT, DaemonCandidateRequest, DaemonCandidateResponse,
    extract_daemon_candidate_response_from_pi_events, parse_daemon_candidate_request,
    parse_daemon_candidate_response,
};

fn valid_request_json() -> Vec<u8> {
    serde_json::to_vec(&DaemonCandidateRequest {
        protocol: "cognitiveos.private-candidate/1".to_owned(),
        task_ref: "task://personal/example".to_owned(),
        contract_epoch: 1,
        rendered_context: "bounded context".to_owned(),
    })
    .expect("serialize request")
}

fn valid_response_json() -> Vec<u8> {
    serde_json::to_vec(&DaemonCandidateResponse {
        tool_ref: "tool://personal/example".to_owned(),
        action: "observe".to_owned(),
        target: "workspace://personal/example".to_owned(),
        parameters: None,
        parameters_digest: "sha256:parameters".to_owned(),
        expected_state_version: 1,
        operation_descriptor_id: "0190f5c0-0000-7000-8000-000000000001".to_owned(),
    })
    .expect("serialize response")
}

#[test]
fn daemon_candidate_request_accepts_only_bounded_candidate_input() {
    let request = parse_daemon_candidate_request(&valid_request_json()).expect("valid request");

    assert_eq!(request.protocol, "cognitiveos.private-candidate/1");
    assert_eq!(request.task_ref, "task://personal/example");
}

#[test]
fn daemon_candidate_request_rejects_authority_fields() {
    let mut request = String::from_utf8(valid_request_json()).expect("request is UTF-8");
    request.pop();
    request.push_str(",\"wia\":\"authority-must-not-cross-boundary\"}");

    let error = parse_daemon_candidate_request(request.as_bytes()).expect_err("reject WIA");

    assert!(error.contains("invalid"));
}

#[test]
fn daemon_candidate_request_rejects_oversized_context_before_processing() {
    let mut request = valid_request_json();
    request.resize(DAEMON_CANDIDATE_FRAME_LIMIT + 1, b'x');

    let error = parse_daemon_candidate_request(&request).expect_err("reject oversized request");

    assert_eq!(error, "daemon candidate request exceeds transport limit");
}

#[test]
fn daemon_candidate_response_accepts_exact_candidate_shape() {
    let response = parse_daemon_candidate_response(&valid_response_json()).expect("valid response");

    assert_eq!(response.action, "observe");
    assert_eq!(response.expected_state_version, 1);
}

#[test]
fn daemon_candidate_response_rejects_extra_authority_output() {
    let mut response = String::from_utf8(valid_response_json()).expect("response is UTF-8");
    response.pop();
    response.push_str(",\"effect\":{\"state\":\"admitted\"}}");

    let error = parse_daemon_candidate_response(response.as_bytes()).expect_err("reject effect");

    assert!(error.contains("invalid"));
}

#[test]
fn daemon_candidate_response_rejects_non_positive_state_version() {
    let response = String::from_utf8(valid_response_json())
        .expect("response is UTF-8")
        .replace(
            "\"expected_state_version\":1",
            "\"expected_state_version\":0",
        );

    let error = parse_daemon_candidate_response(response.as_bytes()).expect_err("reject version");

    assert_eq!(
        error,
        "daemon candidate response expected_state_version is invalid"
    );
}

fn finalized_pi_event(candidate_response: &str) -> String {
    serde_json::json!({
        "type": "message_end",
        "message": {
            "role": "assistant",
            "content": [{ "type": "text", "text": candidate_response }]
        }
    })
    .to_string()
}

#[test]
fn pi_print_events_accept_one_finalized_candidate_response() {
    let response = String::from_utf8(valid_response_json()).expect("response is UTF-8");
    let events = format!(
        "{{\"type\":\"agent_start\"}}\n{}\n{{\"type\":\"agent_end\"}}\n",
        finalized_pi_event(&response)
    );

    let candidate =
        extract_daemon_candidate_response_from_pi_events(&events).expect("one candidate");

    assert_eq!(candidate.tool_ref, "tool://personal/example");
}

#[test]
fn pi_print_events_reject_tool_execution_before_candidate_extraction() {
    let response = String::from_utf8(valid_response_json()).expect("response is UTF-8");
    let events = format!(
        "{{\"type\":\"tool_execution_start\",\"toolName\":\"bash\"}}\n{}\n",
        finalized_pi_event(&response)
    );

    let error = extract_daemon_candidate_response_from_pi_events(&events).expect_err("reject tool");

    assert_eq!(
        error,
        "Pi candidate event stream attempted a tool operation"
    );
}

#[test]
fn pi_print_events_accept_one_daemon_governed_workspace_search() {
    let events = concat!(
        r#"{"type":"tool_execution_start","toolCallId":"1","toolName":"WorkspaceSearch","args":{"query":"TODO","target":"workspace://personal/example"}}"#,
        "\n",
        r#"{"type":"tool_execution_end","toolCallId":"1","toolName":"WorkspaceSearch","isError":false,"result":{"content":[{"type":"text","text":"queued"}]}}"#,
        "\n"
    );

    let candidate =
        extract_daemon_candidate_response_from_pi_events(events).expect("workspace search");

    assert_eq!(candidate.tool_ref, "native.workspace.search");
    assert_eq!(candidate.action, "search");
    assert_eq!(candidate.target, "workspace://personal/example");
    assert_eq!(
        candidate.operation_descriptor_id,
        "00000000-0000-7000-8000-000000002002"
    );
    assert_eq!(candidate.expected_state_version, 1);
    let parameters = candidate.parameters.expect("search parameters");
    assert_eq!(parameters["family"], "WorkspaceSearch");
    assert_eq!(parameters["query"], "TODO");
    assert!(
        candidate.parameters_digest.starts_with("sha256:"),
        "{}",
        candidate.parameters_digest
    );
}

#[test]
fn pi_print_events_reject_bash_even_when_a_workspace_tool_is_present() {
    let events = concat!(
        r#"{"type":"tool_execution_start","toolName":"bash","args":{"command":"ls"}}"#,
        "\n",
        r#"{"type":"tool_execution_start","toolName":"WorkspaceSearch","args":{"query":"TODO","target":"workspace://personal/example"}}"#,
        "\n"
    );

    let error = extract_daemon_candidate_response_from_pi_events(events).expect_err("reject bash");
    assert_eq!(
        error,
        "Pi candidate event stream attempted a tool operation"
    );
}

#[test]
fn pi_print_events_reject_mixed_workspace_tool_and_json_candidate() {
    let response = String::from_utf8(valid_response_json()).expect("response is UTF-8");
    let events = format!(
        "{}\n{}\n",
        r#"{"type":"tool_execution_start","toolName":"WorkspaceSearch","args":{"query":"TODO","target":"workspace://personal/example"}}"#,
        finalized_pi_event(&response)
    );

    let error =
        extract_daemon_candidate_response_from_pi_events(&events).expect_err("reject mixed");
    assert!(error.contains("mixed"), "{error}");
}

#[test]
fn pi_print_events_reject_two_workspace_tool_calls() {
    let event = r#"{"type":"tool_execution_start","toolName":"WorkspaceSearch","args":{"query":"TODO","target":"workspace://personal/example"}}"#;
    let events = format!("{event}\n{event}\n");

    let error =
        extract_daemon_candidate_response_from_pi_events(&events).expect_err("reject duplicate");
    assert!(error.contains("multiple Workspace*"), "{error}");
}

#[test]
fn pi_print_events_reject_multiple_finalized_candidates() {
    let response = String::from_utf8(valid_response_json()).expect("response is UTF-8");
    let event = finalized_pi_event(&response);
    let events = format!("{event}\n{event}\n");

    let error =
        extract_daemon_candidate_response_from_pi_events(&events).expect_err("reject ambiguity");

    assert_eq!(
        error,
        "Pi candidate event stream has multiple final responses"
    );
}

#[test]
fn pi_print_events_reject_markdown_or_non_candidate_prose() {
    let events = finalized_pi_event("```json\n{}\n```");

    let error = extract_daemon_candidate_response_from_pi_events(&format!("{events}\n"))
        .expect_err("reject prose");

    assert!(error.contains("Pi candidate final response is invalid"));
}
