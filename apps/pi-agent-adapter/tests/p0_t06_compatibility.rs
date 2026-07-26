use pi_agent_adapter::{PiCompatibilityPin, parse_rpc_jsonl_records};

#[test]
fn pinned_pi_metadata_matches_the_reviewed_package_release() {
    let compatibility_pin = PiCompatibilityPin::expected();

    assert_eq!(compatibility_pin.package_version(), "0.81.1");
    assert_eq!(
        compatibility_pin.npm_integrity(),
        "sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A=="
    );
    assert_eq!(
        compatibility_pin.source_commit(),
        "20be4b18d4c57487f8993d2762bace129f0cf7c6"
    );
    assert_eq!(
        compatibility_pin.repository_url(),
        "https://github.com/earendil-works/pi.git"
    );
    assert_eq!(
        compatibility_pin.repository_directory(),
        "packages/coding-agent"
    );
    assert_eq!(compatibility_pin.node_engine(), ">=22.19.0");
}

#[test]
fn pi_version_mismatch_is_rejected_before_candidate_process_use() -> Result<(), String> {
    let compatibility_pin = PiCompatibilityPin::expected();

    assert!(
        compatibility_pin
            .validate_reported_version("pi 0.81.1\n")
            .is_ok()
    );

    let error = match compatibility_pin.validate_reported_version("pi 0.82.0\n") {
        Ok(()) => return Err("a drifting Pi release must fail closed".to_owned()),
        Err(error) => error,
    };
    assert!(error.contains("expected 0.81.1"));
    assert!(error.contains("reported 0.82.0"));
    Ok(())
}

#[test]
fn rpc_fixture_preserves_json_strings_containing_unicode_line_separators() -> Result<(), String> {
    let fixture = concat!(
        "{\"id\":\"p0-t06-prompt\",\"type\":\"prompt\",\"message\":\"first\\u2028second\"}\n",
        "{\"id\":\"p0-t06-prompt\",\"type\":\"response\",\"command\":\"prompt\",\"success\":true}\n",
        "{\"type\":\"message_start\",\"messageId\":\"candidate-1\"}\n"
    );

    let records = parse_rpc_jsonl_records(fixture)?;

    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["message"], "first\u{2028}second");
    assert_eq!(records[1]["type"], "response");
    assert_eq!(records[2]["type"], "message_start");
    Ok(())
}

#[test]
fn rpc_fixture_accepts_crlf_records_but_rejects_non_jsonl_delimiters() -> Result<(), String> {
    let crlf_fixture = concat!(
        "{\"type\":\"project_trust\"}\r\n",
        "{\"type\":\"session_start\",\"reason\":\"startup\"}\r\n"
    );

    assert_eq!(parse_rpc_jsonl_records(crlf_fixture)?.len(), 2);

    let error =
        match parse_rpc_jsonl_records("{\"type\":\"project_trust\"}\r{\"type\":\"session_start\"}")
        {
            Ok(_) => return Err("bare carriage returns must not delimit RPC records".to_owned()),
            Err(error) => error,
        };
    assert!(error.contains("LF"));
    Ok(())
}

#[test]
fn rpc_fixture_rejects_non_object_or_malformed_records() {
    assert!(parse_rpc_jsonl_records("[\"not-an-rpc-record\"]\n").is_err());
    assert!(parse_rpc_jsonl_records("{\"type\":\"prompt\"\n").is_err());
    assert!(parse_rpc_jsonl_records("\n").is_err());
}

#[test]
fn extension_poc_denies_project_trust_and_intercepts_mutating_tools() {
    let extension_source = include_str!("../fixtures/p0_t06_extension.ts");

    assert!(extension_source.contains("pi.on(\"project_trust\""));
    assert!(extension_source.contains("{ trusted: \"no\" }"));
    assert!(extension_source.contains("pi.on(\"tool_call\""));
    for blocked_tool in ["write", "edit", "bash"] {
        assert!(extension_source.contains(&format!("\"{blocked_tool}\"")));
    }
}

#[test]
fn extension_poc_handles_session_start_without_secret_or_database_access() {
    let extension_source = include_str!("../fixtures/p0_t06_extension.ts");

    assert!(extension_source.contains("pi.on(\"session_start\""));
    for forbidden_access in ["process.env", "fs.", "node:fs", "sqlite", "database"] {
        assert!(
            !extension_source.contains(forbidden_access),
            "extension PoC must not access {forbidden_access}"
        );
    }
}
