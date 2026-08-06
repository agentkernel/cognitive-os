use pi_agent_adapter::{
    DAEMON_CANDIDATE_FRAME_LIMIT, DaemonCandidateRequest, DaemonCandidateResponse,
    parse_daemon_candidate_request, parse_daemon_candidate_response,
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
        parameters_digest: "sha256:parameters".to_owned(),
        expected_state_version: 0,
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
    assert_eq!(response.expected_state_version, 0);
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
fn daemon_candidate_response_rejects_negative_state_version() {
    let response = String::from_utf8(valid_response_json())
        .expect("response is UTF-8")
        .replace(
            "\"expected_state_version\":0",
            "\"expected_state_version\":-1",
        );

    let error = parse_daemon_candidate_response(response.as_bytes()).expect_err("reject version");

    assert_eq!(
        error,
        "daemon candidate response expected_state_version is invalid"
    );
}
