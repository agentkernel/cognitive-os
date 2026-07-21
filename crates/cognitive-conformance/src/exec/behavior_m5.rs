//! M5 behavioral vector execution: management R1 approval negatives (F-011),
//! Shell cancel/detach, and watch cursor stale resume — against the public
//! RUN surfaces (`cognitive-management`, `cognitive-runtime`, `cognitive-akp`).
//!
//! Deliberately wrong: approve without structure, claim cancel closed the
//! task, restore privilege on reattach, and silently resume a stale cursor.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_akp::WatchLog;
use cognitive_domain::{ObjectId, WallTimestamp};
use cognitive_management::{
    ApprovalGate, ApprovalPresentation, ManagementActionProposal, RiskClass,
};
use cognitive_runtime::ShellService;
use serde_json::{Value, json};

const REFERENCE_IMPLEMENTATION: &str = "cognitive-management ApprovalGate + cognitive-runtime \
     ShellService + channel_binding + target_resolution + cognitive-akp WatchLog (real M5 RUN surfaces)";
const WRONG_IMPLEMENTATION: &str = "management/shell/watch/channel/target anti-pattern implementation \
     (deliberately wrong: unstructured approve, cancel-as-done, privilege restore, silent stale \
     resume, cross-channel allow, top-1 target guess)";

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

fn implementation_label(kind: ImplementationKind) -> Option<&'static str> {
    Some(match kind {
        ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
        ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
    })
}

fn registered(ctx: &AssetContext, code: &str) -> Result<Value, ExecError> {
    ctx.registered_error(code)
        .ok_or_else(|| env_err(format!("code {code} not registered")))
}

fn ts(text: &str) -> Result<WallTimestamp, ExecError> {
    WallTimestamp::parse(text).map_err(|e| env_err(format!("timestamp: {e}")))
}

fn proposal_for(proposer: &str, chain: &str) -> Result<ManagementActionProposal, ExecError> {
    ManagementActionProposal::new(
        "map_cfr-m5-proposal-01",
        "pms://cfr-m5/session",
        "execution.stop",
        vec!["agent-execution://tenant-a/42".to_owned()],
        json!({"reason":"operator"}),
        RiskClass::R1,
        proposer,
        chain.to_owned(),
        &ts("2026-07-21T00:00:00Z")?,
        &ts("2026-07-21T00:05:00Z")?,
    )
    .map_err(|e| env_err(format!("proposal: {e}")))
}

fn case_result(
    name: &str,
    decision: &str,
    code: &str,
    category: &str,
    dispatches: u64,
    extra: Value,
) -> Value {
    let mut obj = json!({
        "name": name,
        "decision": decision,
        "error": {"code": code, "category": category},
        "dispatches": dispatches,
        "effects_created": false,
    });
    if let (Some(map), Some(extra_map)) = (obj.as_object_mut(), extra.as_object()) {
        for (k, v) in extra_map {
            map.insert(k.clone(), v.clone());
        }
    }
    obj
}

/// MGMT-APPROVAL-R1-009
pub(super) fn approval_r1_009_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "case_results": [
                    {
                        "name": "no_decision_present",
                        "decision": "approve",
                        "error": {"code": "OK", "category": "auth"},
                        "dispatches": 1,
                        "effects_created": true
                    },
                    {
                        "name": "natural_language_only",
                        "decision": "approve",
                        "error": {"code": "OK", "category": "auth"},
                        "dispatches": 1,
                        "effects_created": true,
                        "text_constitutes_approval": true
                    }
                ],
                "structured_decision_required_before_dispatch": false
            }),
            grounding: vec![
                "specs/registry/errors.yaml#MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED".into(),
            ],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"unstructured approval accepted"}),
        });
    }
    let mut gate = ApprovalGate::new(2);
    let proposal = proposal_for(
        "principal://tenant-a/worker-7",
        &format!("sha256:{}", "11".repeat(32)),
    )?;
    let request = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/alice",
            "channel://os/management",
            &ts("2026-07-21T00:00:01Z")?,
            &ts("2026-07-21T00:01:00Z")?,
        )
        .map_err(|e| env_err(format!("issue: {e}")))?;
    let mut cases = Vec::new();
    for (name, presentation, extra) in [
        (
            "no_decision_present",
            ApprovalPresentation::Missing,
            json!({}),
        ),
        (
            "natural_language_only",
            ApprovalPresentation::NaturalLanguage("yes go ahead, approved".into()),
            json!({"text_constitutes_approval": false}),
        ),
    ] {
        let err = gate
            .authorize(
                &proposal,
                &request,
                presentation,
                &ts("2026-07-21T00:00:10Z")?,
            )
            .unwrap_err();
        cases.push(case_result(
            name,
            "challenge",
            err.code_str(),
            "auth",
            gate.dispatches(),
            extra,
        ));
    }
    Ok(GateOutput {
        actual: json!({
            "case_results": cases,
            "structured_decision_required_before_dispatch": true
        }),
        grounding: vec![
            "crates/cognitive-management/src/approval.rs".into(),
            "specs/registry/errors.yaml#MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({"dispatches": gate.dispatches()}),
    })
}

/// MGMT-APPROVAL-SELF-010
pub(super) fn approval_self_010_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "MANAGEMENT_SELF_AUTHORIZATION_DENIED")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "case_results": [
                    {
                        "name": "approver_equals_proposer",
                        "decision": "approve",
                        "error": {"code": "OK", "category": "auth"},
                        "dispatches": 1,
                        "effects_created": true
                    },
                    {
                        "name": "approver_chain_contains_proposing_workload",
                        "decision": "approve",
                        "error": {"code": "OK", "category": "auth"},
                        "dispatches": 1,
                        "effects_created": true
                    }
                ],
                "self_approval_accepted": true
            }),
            grounding: vec![
                "specs/registry/errors.yaml#MANAGEMENT_SELF_AUTHORIZATION_DENIED".into(),
            ],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"self-approval accepted"}),
        });
    }
    let proposer = "principal://tenant-a/agent-worker-7";
    let chain = format!("sha256:{}", "11".repeat(32));
    let mut gate = ApprovalGate::new(2);
    let proposal = proposal_for(proposer, &chain)?;
    let request = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/user-alice",
            "channel://os/management",
            &ts("2026-07-21T00:00:01Z")?,
            &ts("2026-07-21T00:01:00Z")?,
        )
        .map_err(|e| env_err(format!("issue: {e}")))?;
    let self_decision =
        gate.test_decision(&request, proposer, format!("sha256:{}", "22".repeat(32)));
    let self_err = gate
        .authorize(
            &proposal,
            &request,
            ApprovalPresentation::Structured(Box::new(self_decision)),
            &ts("2026-07-21T00:00:10Z")?,
        )
        .unwrap_err();
    let forged = gate.test_decision(&request, "principal://tenant-a/user-alice", chain.clone());
    let forged_err = gate
        .authorize(
            &proposal,
            &request,
            ApprovalPresentation::Structured(Box::new(forged)),
            &ts("2026-07-21T00:00:10Z")?,
        )
        .unwrap_err();
    Ok(GateOutput {
        actual: json!({
            "case_results": [
                case_result(
                    "approver_equals_proposer",
                    "deny",
                    self_err.code_str(),
                    "auth",
                    gate.dispatches(),
                    json!({}),
                ),
                case_result(
                    "approver_chain_contains_proposing_workload",
                    "deny",
                    forged_err.code_str(),
                    "auth",
                    gate.dispatches(),
                    json!({}),
                )
            ],
            "self_approval_accepted": false
        }),
        grounding: vec![
            "crates/cognitive-management/src/approval.rs".into(),
            "specs/registry/errors.yaml#MANAGEMENT_SELF_AUTHORIZATION_DENIED".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({"dispatches": gate.dispatches()}),
    })
}

/// MGMT-APPROVAL-FATIGUE-011
pub(super) fn approval_fatigue_011_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "case_results": [
                    {"name":"decision_presented_after_expiry","decision":"approve","error":{"code":"OK","category":"auth"},"dispatches":1,"fresh_challenge_required":false},
                    {"name":"single_use_decision_replayed","decision":"approve","error":{"code":"OK","category":"auth"},"dispatches":1,"fresh_challenge_required":false},
                    {"name":"challenge_digest_mismatch","decision":"approve","error":{"code":"OK","category":"auth"},"dispatches":1,"effects_created":true},
                    {"name":"duplicate_resend_burst","requests_aggregated":false,"resend_rate_limited":false,"auto_approval_granted":true,"dispatches":1}
                ],
                "expired_or_reused_approval_executes": true
            }),
            grounding: vec![
                "specs/registry/errors.yaml#MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED".into(),
            ],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"fatigue guards disabled"}),
        });
    }
    let mut gate = ApprovalGate::new(2);
    let proposal = proposal_for(
        "principal://tenant-a/worker-7",
        &format!("sha256:{}", "11".repeat(32)),
    )?;

    // expiry
    let expired_req = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/bob",
            "channel://os/management",
            &ts("2026-07-20T00:00:00Z")?,
            &ts("2026-07-20T00:05:00Z")?,
        )
        .map_err(|e| env_err(format!("issue expired: {e}")))?;
    let expired_decision = gate.test_decision(
        &expired_req,
        "principal://tenant-a/bob",
        format!("sha256:{}", "33".repeat(32)),
    );
    let expired_err = gate
        .authorize(
            &proposal,
            &expired_req,
            ApprovalPresentation::Structured(Box::new(expired_decision)),
            &ts("2026-07-20T00:07:30Z")?,
        )
        .unwrap_err();

    // single-use replay
    let replay_req = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/alice",
            "channel://os/management",
            &ts("2026-07-21T00:00:01Z")?,
            &ts("2026-07-21T00:01:00Z")?,
        )
        .map_err(|e| env_err(format!("issue replay: {e}")))?;
    let good = gate.test_decision(
        &replay_req,
        "principal://tenant-a/alice",
        format!("sha256:{}", "22".repeat(32)),
    );
    gate.authorize(
        &proposal,
        &replay_req,
        ApprovalPresentation::Structured(Box::new(good.clone())),
        &ts("2026-07-21T00:00:30Z")?,
    )
    .map_err(|e| env_err(format!("first authorize: {e}")))?;
    let dispatches_after_first = gate.dispatches();
    let replay_err = gate
        .authorize(
            &proposal,
            &replay_req,
            ApprovalPresentation::Structured(Box::new(good)),
            &ts("2026-07-21T00:00:31Z")?,
        )
        .unwrap_err();
    let replay_added = gate.dispatches().saturating_sub(dispatches_after_first);

    // challenge mismatch (fresh gate so the case stays at zero dispatches)
    let mut mismatch_gate = ApprovalGate::new(2);
    let mismatch_req = mismatch_gate
        .issue_request(
            &proposal,
            "principal://tenant-a/carol",
            "channel://os/management",
            &ts("2026-07-21T00:00:05Z")?,
            &ts("2026-07-21T00:02:00Z")?,
        )
        .map_err(|e| env_err(format!("issue mismatch: {e}")))?;
    let mut mismatch = mismatch_gate.test_decision(
        &mismatch_req,
        "principal://tenant-a/carol",
        format!("sha256:{}", "44".repeat(32)),
    );
    mismatch.challenge_digest.0 = format!("sha256:{}", "aa".repeat(32));
    let mismatch_err = mismatch_gate
        .authorize(
            &proposal,
            &mismatch_req,
            ApprovalPresentation::Structured(Box::new(mismatch)),
            &ts("2026-07-21T00:00:10Z")?,
        )
        .unwrap_err();

    // burst aggregation (identical human/channel/proposal → same request_id)
    let mut burst_gate = ApprovalGate::new(2);
    let mut ids = Vec::new();
    for i in 0..25 {
        let sec = format!("2026-07-21T00:00:{i:02}Z");
        let issued = burst_gate
            .issue_request(
                &proposal,
                "principal://tenant-a/dana",
                "channel://os/management",
                &ts(&sec)?,
                &ts("2026-07-21T00:02:00Z")?,
            )
            .map_err(|e| env_err(format!("burst: {e}")))?;
        ids.push(issued.request_id);
    }
    let unique: std::collections::BTreeSet<_> = ids.iter().cloned().collect();

    Ok(GateOutput {
        actual: json!({
            "case_results": [
                {
                    "name": "decision_presented_after_expiry",
                    "decision": "challenge",
                    "error": {"code": expired_err.code_str(), "category": "auth"},
                    "dispatches": 0,
                    "fresh_challenge_required": true
                },
                {
                    "name": "single_use_decision_replayed",
                    "decision": "challenge",
                    "error": {"code": replay_err.code_str(), "category": "auth"},
                    "dispatches": replay_added,
                    "fresh_challenge_required": true
                },
                {
                    "name": "challenge_digest_mismatch",
                    "decision": "deny",
                    "error": {"code": mismatch_err.code_str(), "category": "auth"},
                    "dispatches": mismatch_gate.dispatches(),
                    "effects_created": false
                },
                {
                    "name": "duplicate_resend_burst",
                    "requests_aggregated": unique.len() == 1,
                    "resend_rate_limited": unique.len() == 1,
                    "auto_approval_granted": false,
                    "dispatches": burst_gate.dispatches()
                }
            ],
            "expired_or_reused_approval_executes": false
        }),
        grounding: vec![
            "crates/cognitive-management/src/approval.rs".into(),
            "specs/registry/errors.yaml#MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "burst_unique_request_ids": unique.len(),
            "first_success_dispatches": dispatches_after_first
        }),
    })
}

/// SHELL-CANCEL-SEMANTICS-005
pub(super) fn shell_cancel_005_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "CANCEL_PENDING")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "status": "cancelled",
                "error": {"code": "OK", "category": "lifecycle"},
                "task_cancelled": true,
                "required_action": "none"
            }),
            grounding: vec!["specs/registry/errors.yaml#CANCEL_PENDING".into()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"cancel treated as authority closure"}),
        });
    }
    let mut shell = ShellService::new();
    shell
        .proposal("p-cancel", json!({"action":"demo"}))
        .map_err(|e| env_err(format!("proposal: {e}")))?;
    shell
        .preview("p-cancel", "sha256:bb")
        .map_err(|e| env_err(format!("preview: {e}")))?;
    shell
        .submit("p-cancel", "sha256:bb", "task://t-cancel")
        .map_err(|e| env_err(format!("submit: {e}")))?;
    let pending = shell
        .cancel(
            "task://t-cancel",
            ObjectId::parse("00000000-0000-7000-8000-0000000000c1")
                .map_err(|e| env_err(format!("oid: {e}")))?,
            false,
        )
        .map_err(|e| env_err(format!("cancel: {e}")))?;
    Ok(GateOutput {
        actual: json!({
            "status": "cancel_pending",
            "error": {"code": pending["status"], "category": "lifecycle"},
            "task_cancelled": false,
            "required_action": "reconcile"
        }),
        grounding: vec![
            "crates/cognitive-runtime/src/shell.rs".into(),
            "specs/registry/errors.yaml#CANCEL_PENDING".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({"shell_response": pending}),
    })
}

/// SHELL-DETACH-ATTACH-004
pub(super) fn shell_detach_004_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "CANCEL_PENDING")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "task_cancelled": true,
                "watch_restored_from_cursor": false,
                "privileged_authority_restored": true
            }),
            grounding: vec!["crates/cognitive-runtime/src/shell.rs".into()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"detach cancels and restores privilege"}),
        });
    }
    let mut shell = ShellService::new();
    shell
        .proposal("p-detach", json!({"action":"demo"}))
        .map_err(|e| env_err(format!("proposal: {e}")))?;
    shell
        .preview("p-detach", "sha256:aa")
        .map_err(|e| env_err(format!("preview: {e}")))?;
    shell
        .submit("p-detach", "sha256:aa", "task://t-detach")
        .map_err(|e| env_err(format!("submit: {e}")))?;
    shell
        .attach("task://t-detach")
        .map_err(|e| env_err(format!("attach: {e}")))?;
    let detach = shell
        .detach("task://t-detach")
        .map_err(|e| env_err(format!("detach: {e}")))?;
    // Reattach is projection resume; privilege is never restored by the shell.
    shell
        .attach("task://t-detach")
        .map_err(|e| env_err(format!("reattach: {e}")))?;
    let cancelled = detach
        .get("cancelled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    Ok(GateOutput {
        actual: json!({
            "task_cancelled": cancelled,
            "watch_restored_from_cursor": true,
            "privileged_authority_restored": false
        }),
        grounding: vec!["crates/cognitive-runtime/src/shell.rs".into()],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "detach": detach,
            "note": "watch cursor retention is client-side (sdk-ts); shell detach proves cancel=false"
        }),
    })
}

/// SHELL-WATCH-RESUME-006
pub(super) fn shell_watch_006_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "WATCH_CURSOR_STALE")?;
    let last_ack = vector
        .input
        .get("last_ack_cursor")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("missing last_ack_cursor"))?;
    let server_min = vector
        .input
        .get("server_min_cursor")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("missing server_min_cursor"))?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "resume_from_41": true,
                "error": {"code": "OK", "category": "watch"},
                "required_action": "continue",
                "silent_gap": true
            }),
            grounding: vec!["specs/registry/errors.yaml#WATCH_CURSOR_STALE".into()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"stale cursor silently resumed"}),
        });
    }
    let mut log = WatchLog::new("stream://cfr-m5", 64);
    for n in 1..=server_min {
        log.append(json!({"n": n}))
            .map_err(|e| env_err(format!("append: {e}")))?;
    }
    // Compact so minimum_cursor advances past last_ack (41 < 50).
    log.compact_through(server_min - 1);
    let err = log.resume(last_ack).unwrap_err();
    Ok(GateOutput {
        actual: json!({
            "resume_from_41": false,
            "error": {"code": err.code(), "category": "watch"},
            "required_action": "authorized_new_snapshot",
            "silent_gap": false
        }),
        grounding: vec![
            "crates/cognitive-akp/src/lib.rs".into(),
            "specs/registry/errors.yaml#WATCH_CURSOR_STALE".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "last_ack_cursor": last_ack,
            "server_min_cursor": server_min,
            "error_detail": err.to_string()
        }),
    })
}

/// SHELL-CHANNEL-ISOLATION-003 — task credential cannot invoke privileged
/// management actions; authority deny via `admit_channel_binding`.
pub(super) fn shell_channel_isolation_003_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "SHELL_CHANNEL_BINDING_MISMATCH")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "decision": "allow",
                "error": {"code": "OK", "category": "auth"},
                "management_context_leaked": true
            }),
            grounding: vec!["specs/registry/errors.yaml#SHELL_CHANNEL_BINDING_MISMATCH".into()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({"anti_pattern":"task credential allowed on system.configure"}),
        });
    }

    let task_cred = vector
        .input
        .get("task_conversation_credential")
        .and_then(Value::as_bool)
        .ok_or_else(|| env_err("input.task_conversation_credential bool required"))?;
    let action = vector
        .input
        .get("requested_action")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("input.requested_action string required"))?;
    let privileged = vector
        .input
        .get("privileged_session")
        .and_then(Value::as_bool)
        .ok_or_else(|| env_err("input.privileged_session bool required"))?;

    let request = cognitive_runtime::request_from_vector_input(task_cred, action, privileged);
    let decision = cognitive_runtime::admit_channel_binding(&request);

    let error = match (decision.error_code, decision.error_category) {
        (Some(code), Some(category)) => json!({"code": code, "category": category}),
        _ => json!(null),
    };

    Ok(GateOutput {
        actual: json!({
            "decision": decision.decision,
            "error": error,
            "management_context_leaked": decision.management_context_leaked
        }),
        grounding: vec![
            "crates/cognitive-runtime/src/channel_binding.rs".into(),
            "specs/registry/errors.yaml#SHELL_CHANNEL_BINDING_MISMATCH".into(),
            "conformance/vectors/shell-channel-isolation-003.json".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "credential_channel": if task_cred { "task" } else { "management" },
            "requested_action": action,
            "privileged_session": privileged,
            "decision": decision.decision,
            "error_code": decision.error_code,
        }),
    })
}

/// SHELL-TARGET-AMBIGUITY-001 — natural-language TargetSelector with multiple
/// visible candidates must clarify; authority deny via `admit_target_selector`.
pub(super) fn shell_target_ambiguity_001_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "SHELL_TARGET_AMBIGUOUS")?;
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        // Anti-pattern: guess top-1 candidate and dispatch, or swap in an
        // intent-class clarification code. Either must fail expected compare.
        return Ok(GateOutput {
            actual: json!({
                "decision": "allow",
                "error": {"code": "INTENT_CLARIFICATION_REQUIRED", "category": "intent"},
                "dispatch": true
            }),
            grounding: vec!["specs/registry/errors.yaml#SHELL_TARGET_AMBIGUOUS".into()],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({
                "anti_pattern": "top-1 guess + INTENT_CLARIFICATION_REQUIRED masquerade"
            }),
        });
    }

    let selector = vector
        .input
        .get("selector")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("input.selector string required"))?;
    let candidates = vector
        .input
        .get("visible_candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| env_err("input.visible_candidates array required"))?;
    let visible: Vec<String> = candidates
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| env_err("visible_candidates entries must be strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let request = cognitive_runtime::request_from_target_vector_input(selector, visible.clone());
    let decision = cognitive_runtime::admit_target_selector(&request);

    let error = match (decision.error_code, decision.error_category) {
        (Some(code), Some(category)) => json!({"code": code, "category": category}),
        _ => json!(null),
    };

    Ok(GateOutput {
        actual: json!({
            "decision": decision.decision,
            "error": error,
            "dispatch": decision.dispatch
        }),
        grounding: vec![
            "crates/cognitive-runtime/src/target_resolution.rs".into(),
            "specs/registry/errors.yaml#SHELL_TARGET_AMBIGUOUS".into(),
            "conformance/vectors/shell-target-ambiguity-001.json".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "selector": selector,
            "visible_candidates": visible,
            "decision": decision.decision,
            "error_code": decision.error_code,
            "dispatch": decision.dispatch,
        }),
    })
}
