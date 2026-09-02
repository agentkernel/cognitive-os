#![allow(clippy::expect_used)]

//! P13-T03 hidden-assistant turn: the adapter returns exactly one untrusted
//! final assistant text and refuses every tool attempt, Workspace* included.

use cognitive_runtime::{
    ASSISTANT_INFERENCE_PROTOCOL, AssistantInferenceResponse, parse_assistant_inference_response,
    parse_assistant_object_chain,
};
use pi_agent_adapter::extract_assistant_text_from_pi_events;

fn finalized(text: &str) -> String {
    serde_json::json!({
        "type": "message_end",
        "message": {
            "role": "assistant",
            "stopReason": "stop",
            "content": [{ "type": "text", "text": text }]
        }
    })
    .to_string()
}

const CHAIN: &str = r#"{"reply":"A candidate charter.","objects":[{"object_kind":"charter","fields":{"title":{"value":"Weekly report","provenance":{"kind":"owner-stated"}}}}]}"#;

#[test]
fn assistant_events_yield_exactly_one_final_text() {
    let events = format!(
        "{{\"type\":\"agent_start\"}}\n{{\"type\":\"message_start\"}}\n{}\n{{\"type\":\"agent_end\"}}\n",
        finalized(CHAIN)
    );
    let text = extract_assistant_text_from_pi_events(&events).expect("one final text");
    assert_eq!(text, CHAIN);
    let chain = parse_assistant_object_chain(&text, &[]).expect("daemon-side parse");
    assert_eq!(chain.object_kinds, ["charter"]);
}

#[test]
fn assistant_events_refuse_any_tool_attempt_including_workspace_families() {
    for tool in [
        "bash",
        "edit",
        "write",
        "read",
        "WorkspaceRead",
        "WorkspaceSearch",
        "WorkspaceWrite",
    ] {
        let events = format!(
            "{{\"type\":\"tool_execution_start\",\"toolCallId\":\"1\",\"toolName\":\"{tool}\",\"args\":{{\"target\":\"workspace://personal/x\"}}}}\n{}\n",
            finalized(CHAIN)
        );
        let error = extract_assistant_text_from_pi_events(&events).expect_err(tool);
        assert_eq!(
            error, "Pi assistant event stream attempted a tool operation",
            "{tool}"
        );
    }
    let end_only = format!(
        "{}\n{{\"type\":\"tool_execution_end\",\"toolName\":\"bash\"}}\n",
        finalized(CHAIN)
    );
    assert!(extract_assistant_text_from_pi_events(&end_only).is_err());
}

#[test]
fn assistant_events_refuse_multiple_missing_or_errored_finals() {
    let doubled = format!("{}\n{}\n", finalized(CHAIN), finalized(CHAIN));
    assert_eq!(
        extract_assistant_text_from_pi_events(&doubled).expect_err("two finals"),
        "Pi assistant event stream has multiple final responses"
    );
    assert_eq!(
        extract_assistant_text_from_pi_events(
            "{\"type\":\"agent_start\"}\n{\"type\":\"agent_end\"}\n"
        )
        .expect_err("no final"),
        "Pi assistant event stream has no final assistant response"
    );
    let errored = serde_json::json!({
        "type": "message_end",
        "message": {
            "role": "assistant",
            "stopReason": "error",
            "errorMessage": "daemon private completion failed",
            "content": []
        }
    })
    .to_string();
    assert!(
        extract_assistant_text_from_pi_events(&format!("{errored}\n"))
            .expect_err("provider error is not a candidate")
            .contains("Provider error")
    );
    let two_blocks = serde_json::json!({
        "type": "message_end",
        "message": {
            "role": "assistant",
            "content": [{ "type": "text", "text": "a" }, { "type": "text", "text": "b" }]
        }
    })
    .to_string();
    assert!(extract_assistant_text_from_pi_events(&format!("{two_blocks}\n")).is_err());
    let user_only = serde_json::json!({
        "type": "message_end",
        "message": { "role": "user", "content": [{ "type": "text", "text": CHAIN }] }
    })
    .to_string();
    assert!(extract_assistant_text_from_pi_events(&format!("{user_only}\n")).is_err());
}

#[test]
fn adapter_response_frame_is_protocol_bound_and_untrusted_until_parsed() {
    let frame = serde_json::to_vec(&AssistantInferenceResponse {
        protocol: ASSISTANT_INFERENCE_PROTOCOL.to_owned(),
        assistant_text: "I think a weekly report would be nice.".to_owned(),
        response_model: Some("deepseek-chat".to_owned()),
    })
    .expect("frame");
    let response = parse_assistant_inference_response(&frame).expect("frame parses");
    assert!(
        parse_assistant_object_chain(&response.assistant_text, &[]).is_err(),
        "prose is not a candidate object chain"
    );
}
