//! Contract-layer schema re-verification (F-003 closure evidence, Rust side).
//!
//! Proves, without any conformance runner, that:
//! 1. every schema under `specs/schemas/` compiles under draft 2020-12 with
//!    all relative `$ref`s resolvable (REQ-GOBJ-VALID-001 shape discipline);
//! 2. the migrated single-track contracts REJECT the legacy
//!    `common-defs.schema.json#/$defs/{metadata,strongRef}` dual-track shapes
//!    (REQ-GOBJ-HEADER-001, REQ-GOBJ-REF-001, REQ-GOBJ-MIG-001) — the exact
//!    instances pinned by the negative vectors
//!    `conformance/vectors/governed-object-legacy-{metadata,strongref}-001.json`;
//! 3. a migrated positive instance is accepted (the validator is not
//!    vacuously rejecting).
//!
//! This is NOT vector execution: no expected-outcome comparison engine, no
//! result reporting. Vector result states remain `not-run` until the
//! Lane-CFR runner executes them (`docs/standards/conformance-evidence.md`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use jsonschema::{Retrieve, Uri};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn load_schemas() -> HashMap<String, Value> {
    let dir = repo_root().join("specs").join("schemas");
    let mut out = HashMap::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.unwrap_or_else(|e| panic!("dir entry: {e}")).path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let raw = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let doc: Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            // $id policy (D-001/D-006 closure): $id == file name, so every
            // relative `$ref` resolves from the containing schema file and
            // the file name is the retrieval URI.
            assert_eq!(
                doc.get("$id").and_then(Value::as_str),
                Some(name.as_str()),
                "{name}: schema $id must equal its file name"
            );
            out.insert(name, doc);
        }
    }
    assert!(out.len() >= 56, "schema suite shrank: {}", out.len());
    out
}

/// Resolves any URI (relative file name, absolute URL, or synthetic base)
/// to the schema whose file name matches the URI's last path segment.
/// Matches the repository convention that every relative `$ref` resolves
/// from the containing schema file (`conformance/README.md`).
struct FileNameRetriever {
    schemas: HashMap<String, Value>,
}

impl Retrieve for FileNameRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let path = uri.path().as_str();
        let file_name = path.rsplit('/').next().unwrap_or(path);
        self.schemas
            .get(file_name)
            .cloned()
            .ok_or_else(|| format!("schema not found for retrieval URI {uri}").into())
    }
}

fn validator_for(schemas: &HashMap<String, Value>, name: &str) -> jsonschema::Validator {
    let schema = schemas
        .get(name)
        .unwrap_or_else(|| panic!("schema {name} missing"));
    jsonschema::options()
        .with_retriever(FileNameRetriever {
            schemas: schemas.clone(),
        })
        .should_validate_formats(true)
        .build(schema)
        .unwrap_or_else(|e| panic!("schema {name} failed to compile: {e}"))
}

fn vector_object(file: &str) -> Value {
    let path = repo_root().join("conformance").join("vectors").join(file);
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let vector: Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    vector
        .get("input")
        .and_then(|i| i.get("object"))
        .cloned()
        .unwrap_or_else(|| panic!("{file} has no input.object"))
}

#[test]
fn every_schema_compiles_with_resolvable_refs() {
    let schemas = load_schemas();
    for name in schemas.keys() {
        let _ = validator_for(&schemas, name);
    }
}

#[test]
fn legacy_metadata_envelope_is_rejected() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "effect.schema.json");
    let object = vector_object("governed-object-legacy-metadata-001.json");
    assert!(
        !validator.is_valid(&object),
        "legacy common-defs metadata envelope must be rejected by the \
         single-track Effect contract (REQ-GOBJ-HEADER-001, REQ-GOBJ-MIG-001)"
    );
}

#[test]
fn legacy_strongref_shape_is_rejected() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "effect.schema.json");
    let object = vector_object("governed-object-legacy-strongref-001.json");
    assert!(
        !validator.is_valid(&object),
        "legacy common-defs strongRef shape must be rejected where an \
         ObjectReference strong reference is required (REQ-GOBJ-REF-001, REQ-GOBJ-MIG-001)"
    );
}

#[test]
fn migrated_positive_effect_is_accepted() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "effect.schema.json");
    // The legacy-strongref vector object with the reference migrated to the
    // ObjectReference strong shape is exactly a valid single-track Effect.
    let mut object = vector_object("governed-object-legacy-strongref-001.json");
    object["intent_ref"] = serde_json::json!({
        "kind": "strong",
        "id": "01890a5d-ac96-774b-bcce-b302099a805d",
        "object_version": 1,
        "content_digest":
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });
    if let Some(err) = validator.validate(&object).err() {
        panic!("migrated Effect instance must validate, got: {err}");
    }
}

/// Positive AKP request envelope: the D-013 wire schema must accept the
/// members the companion describes (specs/akp/README.md section 3), so the
/// negative vectors are not passing vacuously.
fn positive_request_envelope() -> Value {
    serde_json::json!({
        "message_id": "01890a5d-ac96-774b-bcce-b302099a8070",
        "operation": "shell.submit",
        "protocol_version": "cognitiveos.akp/0.2",
        "schema_digest":
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "sender": "principal://tenant-a/user-alice",
        "audience": "kernel://task-gateway",
        "correlation_id": "conv://tenant-a/session-1/turn-9",
        "causation_id": "01890a5d-ac96-774b-bcce-b302099a806f",
        "deadline": "2026-07-20T00:05:00Z",
        "idempotency_key": "idem-shell-submit-0001",
        "authorization_ref": "cap://tenant-a/lease-77",
        "budget": { "wall_time_ms": 60000 },
        "payload": { "proposal_ref": "proposal://tenant-a/sap-0001" },
        "payload_digest":
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "extensions": [ { "id": "x-trace", "critical": false } ]
    })
}

#[test]
fn akp_request_envelope_accepts_described_members_and_rejects_vector_negatives() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "akp-request-envelope.schema.json");
    if let Some(err) = validator.validate(&positive_request_envelope()).err() {
        panic!("described request envelope must validate, got: {err}");
    }
    // Management members ride the same envelope (AKP section 10.1) but the
    // session ref never travels alone.
    let mut management = positive_request_envelope();
    management["management_session_ref"] = serde_json::json!("session://tenant-a/pms-1");
    assert!(
        !validator.is_valid(&management),
        "management_session_ref without actor_chain_digest/activity_context_ref must be rejected"
    );
    management["actor_chain_digest"] = serde_json::json!(format!("sha256:{}", "d".repeat(64)));
    management["activity_context_ref"] = serde_json::json!("activity://tenant-a/act-1");
    if let Some(err) = validator.validate(&management).err() {
        panic!("management-bound request envelope must validate, got: {err}");
    }
    // The exact instances pinned by the negative vectors are rejected.
    for vector in [
        "akp-envelope-no-schema-pin-001.json",
        "akp-envelope-ambiguous-payload-002.json",
    ] {
        assert!(
            !validator.is_valid(&vector_object(vector)),
            "{vector} object must be rejected (REQ-AKP-ENV-001/002)"
        );
    }
}

#[test]
fn akp_result_envelope_requires_machine_error_and_continuation() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "akp-result-envelope.schema.json");
    let ok = serde_json::json!({
        "in_reply_to": "01890a5d-ac96-774b-bcce-b302099a8070",
        "correlation_id": "conv://tenant-a/session-1/turn-9",
        "protocol_version": "cognitiveos.akp/0.2",
        "status": "ok",
        "result": { "accepted_ref": "task://tenant-a/tsk-0007" },
        "observed_versions": { "task": 4 },
        "cost": { "wall_time_ms": 12 },
        "audit_ref": "audit://tenant-a/rec-1"
    });
    if let Some(err) = validator.validate(&ok).err() {
        panic!("ok result envelope must validate, got: {err}");
    }
    let error_result = serde_json::json!({
        "in_reply_to": "01890a5d-ac96-774b-bcce-b302099a8070",
        "correlation_id": "conv://tenant-a/session-1/turn-9",
        "protocol_version": "cognitiveos.akp/0.2",
        "status": "error",
        "error": {
            "code": "STATE_CONFLICT",
            "category": "state",
            "stage": "authorization",
            "retryable": true
        }
    });
    if let Some(err) = validator.validate(&error_result).err() {
        panic!("error result with registered machine error must validate, got: {err}");
    }
    assert!(
        !validator.is_valid(&vector_object(
            "akp-result-error-without-machine-code-003.json"
        )),
        "error status without the machine error envelope must be rejected (REQ-ERR-001)"
    );
    let mut partial = ok.clone();
    partial["status"] = serde_json::json!("partial");
    assert!(
        !validator.is_valid(&partial),
        "partial without continuation must be rejected (AKP section 5)"
    );
    partial["continuation"] = serde_json::json!({ "high_watermark": 7 });
    if let Some(err) = validator.validate(&partial).err() {
        panic!("partial with continuation must validate, got: {err}");
    }
}

#[test]
fn akp_stream_frame_kinds_carry_their_required_members() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "akp-stream-frame.schema.json");
    let snapshot = serde_json::json!({
        "stream_id": "watch://tenant-a/wsub-1",
        "sequence": 0,
        "kind": "snapshot",
        "snapshot_version": 4,
        "payload": { "view": "initial" },
        "final": false,
        "cost": { "context_bytes": 2048 }
    });
    if let Some(err) = validator.validate(&snapshot).err() {
        panic!("snapshot frame must validate, got: {err}");
    }
    let error_frame = serde_json::json!({
        "stream_id": "watch://tenant-a/wsub-1",
        "sequence": 9,
        "kind": "error",
        "error": {
            "code": "WATCH_CURSOR_STALE",
            "category": "watch",
            "stage": "resume",
            "retryable": true
        },
        "final": true
    });
    if let Some(err) = validator.validate(&error_frame).err() {
        panic!("machine-coded error frame must validate, got: {err}");
    }
    let mut unversioned_snapshot = snapshot.clone();
    unversioned_snapshot
        .as_object_mut()
        .map(|o| o.remove("snapshot_version"));
    assert!(
        !validator.is_valid(&unversioned_snapshot),
        "snapshot frame without snapshot_version must be rejected"
    );
    assert!(
        !validator.is_valid(&vector_object("akp-stream-frame-unsequenced-004.json")),
        "frame without stream identity/sequence must be rejected (REQ-AKP-STR-001)"
    );
}

#[test]
fn shell_control_request_is_cancel_with_target_and_reason() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "shell-control-request.schema.json");
    let cancel = serde_json::json!({
        "schema_version": "cognitiveos.shell-control-request/0.1",
        "control": "cancel",
        "target_ref": "task://tenant-a/tsk-0007",
        "reason": "user requested stop from the shell"
    });
    if let Some(err) = validator.validate(&cancel).err() {
        panic!("cancel control request must validate, got: {err}");
    }
    assert!(
        !validator.is_valid(&vector_object("shell-control-unreasoned-cancel-001.json")),
        "cancel without reason must be rejected (REQ-AKP-CAN-001)"
    );
}

/// Positive R1 approval request (F-011 registration): the described
/// challenge members validate, so the tier negatives below are not passing
/// vacuously.
fn positive_r1_approval_request() -> Value {
    serde_json::json!({
        "schema_version": "cognitiveos.management-approval-request/0.1",
        "request_id": "mar_r1-net-cfg-0001",
        "proposal_ref": "proposal://tenant-a/map_cfg-network-42",
        "proposal_digest": format!("sha256:{}", "a".repeat(64)),
        "risk_class": "R1",
        "confirmation_surface": "chat_structured",
        "human_principal": "principal://tenant-a/user-alice",
        "proposer_principal": "principal://tenant-a/agent-worker-7",
        "proposer_actor_chain_digest": format!("sha256:{}", "b".repeat(64)),
        "channel_identity": "channel://os/approval-bot-1",
        "challenge_digest": format!("sha256:{}", "c".repeat(64)),
        "method": "digest_shortcode_match",
        "single_use": true,
        "aggregation_key": "system.configure/network",
        "requested_at": "2026-07-20T00:00:00Z",
        "expires_at": "2026-07-20T00:05:00Z"
    })
}

#[test]
fn approval_request_tiers_fail_closed_by_risk_class() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "management-approval-request.schema.json");
    if let Some(err) = validator.validate(&positive_r1_approval_request()).err() {
        panic!("R1 chat-structured approval request must validate, got: {err}");
    }
    // Chat is never a completion surface above R1 (whitepaper 12.12 matrix).
    let mut r2_chat = positive_r1_approval_request();
    r2_chat["risk_class"] = serde_json::json!("R2");
    r2_chat["session_ref"] = serde_json::json!("session://tenant-a/pms-1");
    assert!(
        !validator.is_valid(&r2_chat),
        "R2 with chat_structured completion surface must be rejected"
    );
    r2_chat["confirmation_surface"] = serde_json::json!("trusted_surface");
    if let Some(err) = validator.validate(&r2_chat).err() {
        panic!("R2 trusted-surface request must validate, got: {err}");
    }
    // R2/R3 bind a persistent management session.
    let mut r2_sessionless = r2_chat.clone();
    r2_sessionless
        .as_object_mut()
        .map(|o| o.remove("session_ref"));
    assert!(
        !validator.is_valid(&r2_sessionless),
        "R2 without session_ref must be rejected"
    );
    // R3 requires the dual-independent surface.
    let mut r3 = r2_chat.clone();
    r3["risk_class"] = serde_json::json!("R3");
    assert!(
        !validator.is_valid(&r3),
        "R3 on a non-dual surface must be rejected"
    );
    r3["confirmation_surface"] = serde_json::json!("dual_independent");
    if let Some(err) = validator.validate(&r3).err() {
        panic!("R3 dual-independent request must validate, got: {err}");
    }
    // Auto-approval is an R0-only surface; R1+ must confirm.
    let mut r1_auto = positive_r1_approval_request();
    r1_auto["confirmation_surface"] = serde_json::json!("policy_auto");
    assert!(
        !validator.is_valid(&r1_auto),
        "R1 with policy_auto must be rejected"
    );
    // The challenge is single-use by contract (REQ-AKP-MGMT-002).
    let mut reusable = positive_r1_approval_request();
    reusable["single_use"] = serde_json::json!(false);
    assert!(
        !validator.is_valid(&reusable),
        "a reusable approval request must be rejected"
    );
}

#[test]
fn approval_decision_r1_conditional_binds_request_and_single_use() {
    let schemas = load_schemas();
    let validator = validator_for(&schemas, "management-approval-decision.schema.json");
    let base = serde_json::json!({
        "schema_version": "cognitiveos.management-approval-decision/0.1",
        "decision_id": "mad_r1-net-cfg-0001",
        "object_version": 1,
        "proposal_ref": "proposal://tenant-a/map_cfg-network-42",
        "proposal_digest": format!("sha256:{}", "a".repeat(64)),
        "session_ref": "approval://tenant-a/one-shot/mar_r1-net-cfg-0001",
        "decision": "approve",
        "deciding_authority": "authority://platform/management-session",
        "approver_principal": "principal://tenant-a/user-alice",
        "approver_actor_chain_digest": format!("sha256:{}", "d".repeat(64)),
        "policy_version": 3,
        "risk_class": "R1",
        "challenge_digest": format!("sha256:{}", "c".repeat(64)),
        "decided_at": "2026-07-20T00:01:00Z",
        "expires_at": "2026-07-20T00:05:00Z",
        "decision_digest": format!("sha256:{}", "e".repeat(64)),
        "authority_signature": "sig-0123456789abcdef"
    });
    // R1 approve without the request binding / single-use pledge is rejected
    // (hardened conditional, F-011).
    assert!(
        !validator.is_valid(&base),
        "R1 approve without request_ref/single_use must be rejected"
    );
    let mut bound = base.clone();
    bound["request_ref"] = serde_json::json!("approval-request://tenant-a/mar_r1-net-cfg-0001");
    bound["single_use"] = serde_json::json!(true);
    if let Some(err) = validator.validate(&bound).err() {
        panic!("bound single-use R1 approval must validate, got: {err}");
    }
    let mut reusable = bound.clone();
    reusable["single_use"] = serde_json::json!(false);
    assert!(
        !validator.is_valid(&reusable),
        "reusable R1 approval must be rejected"
    );
    // Pre-existing shapes stay valid: an R2 independent approval without the
    // new members is untouched by the hardening (non-breaking proof).
    let mut r2 = base.clone();
    r2["risk_class"] = serde_json::json!("R2");
    r2["independent_from_proposer"] = serde_json::json!(true);
    r2["step_up_method"] = serde_json::json!("fido2_sign");
    if let Some(err) = validator.validate(&r2).err() {
        panic!("existing R2 approval shape must stay valid, got: {err}");
    }
}

#[test]
fn legacy_defs_stay_deprecated_and_unreferenced() {
    // Decision record for F-003 remaining condition (legacy `$defs`): the
    // shapes stay in common-defs.schema.json, marked deprecated, referenced
    // by ZERO schemas (adapter-only per governed-object-contract section 6).
    let schemas = load_schemas();
    let common = &schemas["common-defs.schema.json"];
    for def in ["metadata", "strongRef"] {
        assert_eq!(
            common["$defs"][def]["deprecated"],
            Value::Bool(true),
            "common-defs $defs/{def} must stay marked deprecated"
        );
    }
    for (name, doc) in &schemas {
        if name == "common-defs.schema.json" {
            continue;
        }
        let raw = doc.to_string();
        for banned in [
            "common-defs.schema.json#/$defs/metadata",
            "common-defs.schema.json#/$defs/strongRef",
        ] {
            assert!(
                !raw.contains(banned),
                "{name} references legacy shape {banned} (dual-track ban, F-003)"
            );
        }
    }
}
