//! P2-T26 authenticated bounded O2/O3/O4/O5/O13 observation plane.
//!
//! This is a read plane, not a second authority API. Samples are daemon-authored
//! redacted receipts: authorization decisions, Context-cache/compaction classes,
//! scheduler/fence/budget counters, Intent/Effect history, and durable audit
//! cursor/replay. Context bodies, capability material, receipts, and raw
//! parameters are never stored or returned. An empty collector window returns
//! `observed_zero` with a named negative control — never a silent default-zero
//! count.

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use cognitive_domain::LifecycleDomain;
use cognitive_kernel::ports::{
    AuthorityStore, CommittedEvent, IntentChainStore, IntentRow, ProtocolStore, StoredObject,
    TaskBinding,
};
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

const OBSERVATION_FILE_NAME: &str = "personal-observation-plane.json";
const OBSERVATION_SCHEMA: &str = "cognitiveos.personal.observation-plane/0.1";
const MAX_SAMPLES: usize = 256;
const MAX_RETURNED_SAMPLES: usize = 64;
const MAX_TASK_REF_CHARS: usize = 160;
const MAX_AUDIT_EVENTS_SCANNED: usize = 4096;
const AUDIT_EVENT_BATCH_SIZE: usize = 256;
const GENESIS_DIGEST: &str = "sha256:genesis";
const FORBIDDEN_KEYS: [&str; 8] = [
    "prompt",
    "body",
    "query_text",
    "receipt",
    "parameters",
    "headers",
    "capability",
    "context",
];

static BOUND_DATA_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ObservationSample {
    family: String,
    task_ref: String,
    class: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    purpose: String,
    #[serde(default)]
    epoch: i64,
    #[serde(default)]
    input_digest: String,
    #[serde(default)]
    reason_code: String,
    #[serde(default)]
    count: u64,
    #[serde(default)]
    stable_prefix_segment_count: usize,
    #[serde(default)]
    delta_segment_count: usize,
    #[serde(default)]
    compaction_input_tokens: u64,
    #[serde(default)]
    compaction_output_tokens: u64,
    #[serde(default)]
    compaction_input_bytes: u64,
    #[serde(default)]
    compaction_output_bytes: u64,
    #[serde(default)]
    loss_manifest_digest: String,
    #[serde(default)]
    sampling_window: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ObservationFile {
    schema: String,
    samples: Vec<ObservationSample>,
}

#[derive(Debug)]
pub(crate) struct ObservationResponse {
    pub status: u16,
    pub body: String,
}

pub(crate) fn bind_observation_store(data_dir: PathBuf) {
    let mut bound = BOUND_DATA_DIR
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *bound = Some(data_dir);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_authorization_decision(
    data_dir: &Path,
    task_ref: &str,
    scope: &str,
    purpose: &str,
    epoch: i64,
    action: &str,
    decision_class: &str,
    reason_code: &str,
) {
    let input_digest = digest_bytes(
        format!("{action}\n{purpose}\n{scope}\n{epoch}").as_bytes(),
        "cognitiveos.personal.observation-input/0.1",
    );
    append_sample(
        data_dir,
        ObservationSample {
            family: "o2".to_owned(),
            task_ref: task_ref.to_owned(),
            class: decision_class.to_owned(),
            scope: scope.to_owned(),
            purpose: purpose.to_owned(),
            epoch,
            input_digest,
            reason_code: reason_code.to_owned(),
            count: 1,
            ..ObservationSample::blank()
        },
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_cache_sample(
    data_dir: &Path,
    task_ref: &str,
    class: &str,
    epoch: i64,
    stable_prefix_segment_count: usize,
    delta_segment_count: usize,
    loss_manifest_digest: &str,
) {
    append_sample(
        data_dir,
        ObservationSample {
            family: "o3".to_owned(),
            task_ref: task_ref.to_owned(),
            class: class.to_owned(),
            epoch,
            count: 1,
            stable_prefix_segment_count,
            delta_segment_count,
            loss_manifest_digest: loss_manifest_digest.to_owned(),
            ..ObservationSample::blank()
        },
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_compaction_sample(
    data_dir: &Path,
    task_ref: &str,
    epoch: i64,
    input_tokens: u64,
    output_tokens: u64,
    input_bytes: u64,
    output_bytes: u64,
    loss_manifest_digest: &str,
) {
    append_sample(
        data_dir,
        ObservationSample {
            family: "o3".to_owned(),
            task_ref: task_ref.to_owned(),
            class: "compaction".to_owned(),
            epoch,
            count: 1,
            compaction_input_tokens: input_tokens,
            compaction_output_tokens: output_tokens,
            compaction_input_bytes: input_bytes,
            compaction_output_bytes: output_bytes,
            loss_manifest_digest: loss_manifest_digest.to_owned(),
            ..ObservationSample::blank()
        },
    );
}

pub(crate) fn record_scheduler_sample(
    data_dir: &Path,
    task_ref: &str,
    class: &str,
    epoch: i64,
    count: u64,
    sampling_window: &str,
) {
    append_sample(
        data_dir,
        ObservationSample {
            family: "o4".to_owned(),
            task_ref: task_ref.to_owned(),
            class: class.to_owned(),
            epoch,
            count,
            sampling_window: sampling_window.to_owned(),
            ..ObservationSample::blank()
        },
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_authorization_on_bound_store(
    task_ref: &str,
    scope: &str,
    purpose: &str,
    epoch: i64,
    action: &str,
    decision_class: &str,
    reason_code: &str,
) {
    if let Some(data_dir) = bound_data_dir() {
        record_authorization_decision(
            &data_dir,
            task_ref,
            scope,
            purpose,
            epoch,
            action,
            decision_class,
            reason_code,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_cache_on_bound_store(
    task_ref: &str,
    class: &str,
    epoch: i64,
    stable_prefix_segment_count: usize,
    delta_segment_count: usize,
    loss_manifest_digest: &str,
) {
    if let Some(data_dir) = bound_data_dir() {
        record_cache_sample(
            &data_dir,
            task_ref,
            class,
            epoch,
            stable_prefix_segment_count,
            delta_segment_count,
            loss_manifest_digest,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_compaction_on_bound_store(
    task_ref: &str,
    epoch: i64,
    input_tokens: u64,
    output_tokens: u64,
    input_bytes: u64,
    output_bytes: u64,
    loss_manifest_digest: &str,
) {
    if let Some(data_dir) = bound_data_dir() {
        record_compaction_sample(
            &data_dir,
            task_ref,
            epoch,
            input_tokens,
            output_tokens,
            input_bytes,
            output_bytes,
            loss_manifest_digest,
        );
    }
}

pub(crate) fn record_scheduler_on_bound_store(
    task_ref: &str,
    class: &str,
    epoch: i64,
    count: u64,
    sampling_window: &str,
) {
    if let Some(data_dir) = bound_data_dir() {
        record_scheduler_sample(&data_dir, task_ref, class, epoch, count, sampling_window);
    }
}

pub(crate) fn loss_manifest_digest(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    digest_bytes(
        parts.join("\n").as_bytes(),
        "cognitiveos.personal.observation-loss/0.1",
    )
}

pub(crate) fn management_channel_forbidden() -> ObservationResponse {
    error(
        403,
        "RESOURCE_OBSERVATION_CHANNEL_FORBIDDEN",
        "observation is a task-channel read plane, not a management authority API",
    )
}

pub(crate) fn write_forbidden() -> ObservationResponse {
    error(
        403,
        "RESOURCE_OBSERVATION_WRITE_FORBIDDEN",
        "observation collectors are daemon-authored; callers cannot write samples",
    )
}

pub(crate) fn handle(
    request_line: &str,
    layout: &cognitive_store::PersonalDataLayout,
) -> ObservationResponse {
    let tokens: Vec<&str> = request_line.split_whitespace().take(2).collect();
    let method = tokens.first().copied().unwrap_or_default();
    let path_and_query = tokens.get(1).copied().unwrap_or_default();
    let (path, query) = match path_and_query.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (path_and_query, None),
    };
    if path != "/task/observation" && path != "/task/resource/v1/observation" {
        return error(
            404,
            "TASK_OBSERVATION_NOT_FOUND",
            "observation route was not matched",
        );
    }
    if method != "GET" {
        return write_forbidden();
    }
    let Some(query) = query else {
        return error(
            400,
            "TASK_OBSERVATION_QUERY_REQUIRED",
            "family and task_ref query parameters are required",
        );
    };
    let mut family = None;
    let mut task_ref = None;
    let mut cursor = None;
    let mut expect_digest = None;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if FORBIDDEN_KEYS.contains(&name) || name == "prompt" {
            return error(
                400,
                "TASK_OBSERVATION_QUERY_FORBIDDEN",
                "observation queries accept only family, task_ref, and o13 cursor/expect_digest",
            );
        }
        if name != "family" && name != "task_ref" && name != "cursor" && name != "expect_digest" {
            return error(
                400,
                "TASK_OBSERVATION_QUERY_FORBIDDEN",
                "observation queries accept only family, task_ref, and o13 cursor/expect_digest",
            );
        }
        let decoded = match percent_decode(value) {
            Ok(decoded) => decoded,
            Err(response) => return response,
        };
        if decoded.trim().is_empty() {
            return error(
                400,
                "TASK_OBSERVATION_QUERY_REQUIRED",
                "family and task_ref must be non-empty",
            );
        }
        match name {
            "family" => {
                if family.replace(decoded).is_some() {
                    return error(
                        400,
                        "TASK_OBSERVATION_QUERY_REQUIRED",
                        "exactly one family is required",
                    );
                }
            }
            "task_ref" => {
                if task_ref.replace(decoded).is_some() {
                    return error(
                        400,
                        "TASK_OBSERVATION_QUERY_REQUIRED",
                        "exactly one task_ref is required",
                    );
                }
            }
            "cursor" => {
                if cursor.replace(decoded).is_some() {
                    return error(
                        400,
                        "TASK_OBSERVATION_QUERY_REQUIRED",
                        "exactly one cursor is required",
                    );
                }
            }
            _ => {
                if expect_digest.replace(decoded).is_some() {
                    return error(
                        400,
                        "TASK_OBSERVATION_QUERY_REQUIRED",
                        "exactly one expect_digest is required",
                    );
                }
            }
        }
    }
    let (Some(family), Some(task_ref)) = (family, task_ref) else {
        return error(
            400,
            "TASK_OBSERVATION_QUERY_REQUIRED",
            "family and task_ref query parameters are required",
        );
    };
    let family = match canonical_family(&family) {
        Some(family) => family,
        None => {
            return error(
                400,
                "TASK_OBSERVATION_FAMILY_INVALID",
                "family must be o2, o3, o4, o5, or o13",
            );
        }
    };
    if family != "o13" && (cursor.is_some() || expect_digest.is_some()) {
        return error(
            400,
            "TASK_OBSERVATION_QUERY_FORBIDDEN",
            "cursor and expect_digest are valid only for family=o13",
        );
    }
    if task_ref.len() > MAX_TASK_REF_CHARS || cognitive_domain::UriRef::parse(&task_ref).is_err() {
        return error(
            400,
            "TASK_OBSERVATION_INVALID_TASK_REF",
            "task_ref must be a canonical URI",
        );
    }
    let body = match family {
        "o5" => match project_o5(layout, &task_ref) {
            Ok(body) => body,
            Err(response) => return response,
        },
        "o13" => match project_o13(
            layout,
            &task_ref,
            cursor.as_deref(),
            expect_digest.as_deref(),
        ) {
            Ok(body) => body,
            Err(response) => return response,
        },
        _ => project(layout.data_dir(), family, &task_ref),
    };
    ObservationResponse {
        status: 200,
        body: body.to_string(),
    }
}

impl ObservationSample {
    fn blank() -> Self {
        Self {
            family: String::new(),
            task_ref: String::new(),
            class: String::new(),
            scope: String::new(),
            purpose: String::new(),
            epoch: 0,
            input_digest: String::new(),
            reason_code: String::new(),
            count: 0,
            stable_prefix_segment_count: 0,
            delta_segment_count: 0,
            compaction_input_tokens: 0,
            compaction_output_tokens: 0,
            compaction_input_bytes: 0,
            compaction_output_bytes: 0,
            loss_manifest_digest: String::new(),
            sampling_window: String::new(),
        }
    }
}

fn canonical_family(family: &str) -> Option<&'static str> {
    match family {
        "o2" | "authorization" => Some("o2"),
        "o3" | "cache" => Some("o3"),
        "o4" | "scheduler" => Some("o4"),
        "o5" | "effects" => Some("o5"),
        "o13" | "audit" => Some("o13"),
        _ => None,
    }
}

fn project(data_dir: &Path, family: &str, task_ref: &str) -> Value {
    let file = load_file(data_dir).unwrap_or_else(|| ObservationFile {
        schema: OBSERVATION_SCHEMA.to_owned(),
        samples: Vec::new(),
    });
    let matched: Vec<&ObservationSample> = file
        .samples
        .iter()
        .filter(|sample| sample.family == family && sample.task_ref == task_ref)
        .collect();
    let truncated = matched.len() > MAX_RETURNED_SAMPLES;
    let returned = matched
        .iter()
        .rev()
        .take(MAX_RETURNED_SAMPLES)
        .rev()
        .copied()
        .collect::<Vec<_>>();
    let denominator = matched.len() as u64;
    let observed_zero = denominator == 0;
    let mut body = json!({
        "schema_version": 1,
        "kind": "observation.plane",
        "family": family,
        "task_ref": task_ref,
        "denominator": denominator,
        "observed_zero": observed_zero,
        "samples_truncated": truncated,
        "authority_side_effects": false,
    });
    match family {
        "o2" => project_o2(&mut body, &returned, observed_zero),
        "o3" => project_o3(&mut body, &returned, observed_zero),
        _ => project_o4(&mut body, &returned, observed_zero),
    }
    body
}

fn project_o2(body: &mut Value, matched: &[&ObservationSample], observed_zero: bool) {
    let mut grant_count = 0u64;
    let mut deny_count = 0u64;
    for sample in matched {
        match sample.class.as_str() {
            "grant" | "allow" => grant_count += sample.count.max(1),
            "deny" => deny_count += sample.count.max(1),
            _ => {}
        }
    }
    body["grant_count"] = json!(grant_count);
    body["deny_count"] = json!(deny_count);
    body["negative_control"] = json!(if observed_zero {
        "no_authorization_sample"
    } else if deny_count > 0 {
        "deny_recorded"
    } else {
        "grant_recorded"
    });
    body["samples"] = json!(
        matched
            .iter()
            .map(|sample| json!({
                "decision_class": sample.class,
                "scope": sample.scope,
                "purpose": sample.purpose,
                "epoch": sample.epoch,
                "input_digest": sample.input_digest,
                "reason_code": sample.reason_code,
            }))
            .collect::<Vec<_>>()
    );
}

fn project_o3(body: &mut Value, matched: &[&ObservationSample], observed_zero: bool) {
    let cache: Vec<&&ObservationSample> = matched
        .iter()
        .filter(|sample| sample.class != "compaction")
        .collect();
    let compaction: Vec<&&ObservationSample> = matched
        .iter()
        .filter(|sample| sample.class == "compaction")
        .collect();
    let mut class_counts = Map::new();
    for sample in &cache {
        let entry = class_counts
            .entry(sample.class.clone())
            .or_insert(json!(0u64));
        if let Some(current) = entry.as_u64() {
            *entry = json!(current.saturating_add(sample.count.max(1)));
        }
    }
    body["cache"] = json!({
        "denominator": cache.len() as u64,
        "observed_zero": cache.is_empty(),
        "negative_control": if cache.is_empty() { "no_cache_sample" } else { "cache_class_recorded" },
        "class_counts": class_counts,
        "samples": cache.iter().map(|sample| json!({
            "class": sample.class,
            "epoch": sample.epoch,
            "stable_prefix_segment_count": sample.stable_prefix_segment_count,
            "delta_segment_count": sample.delta_segment_count,
            "loss_manifest_digest": sample.loss_manifest_digest,
        })).collect::<Vec<_>>(),
    });
    body["compaction"] = json!({
        "denominator": compaction.len() as u64,
        "observed_zero": compaction.is_empty(),
        "negative_control": if compaction.is_empty() { "compaction_not_invoked" } else { "compaction_recorded" },
        "samples": compaction.iter().map(|sample| json!({
            "epoch": sample.epoch,
            "input_tokens": sample.compaction_input_tokens,
            "output_tokens": sample.compaction_output_tokens,
            "input_bytes": sample.compaction_input_bytes,
            "output_bytes": sample.compaction_output_bytes,
            "loss_manifest_digest": sample.loss_manifest_digest,
        })).collect::<Vec<_>>(),
    });
    body["negative_control"] = json!(if observed_zero {
        "no_cache_or_compaction_sample"
    } else {
        "o3_collector_recorded"
    });
}

fn project_o4(body: &mut Value, matched: &[&ObservationSample], observed_zero: bool) {
    let required = [
        "queue_wait",
        "lease_acquired",
        "runnable_count",
        "budget_stop",
        "stale_fence_denial",
        "fairness",
    ];
    let mut counters = Map::new();
    for class in required {
        let samples: Vec<&&ObservationSample> = matched
            .iter()
            .filter(|sample| sample.class == class)
            .collect();
        let total = samples
            .iter()
            .fold(0u64, |acc, sample| acc.saturating_add(sample.count));
        counters.insert(
            class.to_owned(),
            json!({
                "denominator": samples.len() as u64,
                "observed_zero": samples.is_empty(),
                "negative_control": if samples.is_empty() {
                    format!("no_{class}_sample")
                } else {
                    format!("{class}_recorded")
                },
                "count": total,
            }),
        );
    }
    body["counters"] = Value::Object(counters);
    body["negative_control"] = json!(if observed_zero {
        "no_scheduler_sample"
    } else {
        "scheduler_probe_recorded"
    });
    body["samples"] = json!(
        matched
            .iter()
            .map(|sample| json!({
                "class": sample.class,
                "epoch": sample.epoch,
                "count": sample.count,
                "sampling_window": sample.sampling_window,
            }))
            .collect::<Vec<_>>()
    );
}

fn project_o5(layout: &PersonalDataLayout, task_ref: &str) -> Result<Value, ObservationResponse> {
    let Some(store) = open_existing_authority_store(layout)? else {
        return Ok(empty_o5(task_ref, 0));
    };
    let contract_epoch = store.current_contract_epoch(task_ref).map_err(|_| {
        error(
            503,
            "TASK_OBSERVATION_READ_FAILED",
            "current TaskContract epoch could not be read",
        )
    })?;
    if contract_epoch == 0 {
        return Ok(empty_o5(task_ref, 0));
    }
    let task_binding = TaskBinding {
        task_ref: task_ref.to_owned(),
        contract_epoch,
    };
    let intents = store
        .list_intents_for_task_binding(&task_binding)
        .map_err(|_| {
            error(
                503,
                "TASK_OBSERVATION_READ_FAILED",
                "durable Intent bindings could not be read",
            )
        })?;
    let truncated = intents.len() > MAX_RETURNED_SAMPLES;
    let mut effects = Vec::new();
    for intent in intents.iter().take(MAX_RETURNED_SAMPLES) {
        let effect = store
            .load_object(LifecycleDomain::Effect, &intent.effect_object_id)
            .map_err(|_| {
                error(
                    503,
                    "TASK_OBSERVATION_READ_FAILED",
                    "durable Effect lifecycle could not be read",
                )
            })?;
        effects.push(project_effect_sample(intent, effect.as_ref()));
    }
    let denominator = effects.len() as u64;
    let observed_zero = denominator == 0;
    Ok(json!({
        "schema_version": 1,
        "kind": "observation.plane",
        "family": "o5",
        "task_ref": task_ref,
        "contract_epoch": contract_epoch,
        "denominator": denominator,
        "observed_zero": observed_zero,
        "samples_truncated": truncated,
        "authority_side_effects": false,
        "negative_control": if observed_zero {
            "no_effect_sample"
        } else {
            "effect_history_recorded"
        },
        "effects": effects,
    }))
}

fn empty_o5(task_ref: &str, contract_epoch: i64) -> Value {
    json!({
        "schema_version": 1,
        "kind": "observation.plane",
        "family": "o5",
        "task_ref": task_ref,
        "contract_epoch": contract_epoch,
        "denominator": 0,
        "observed_zero": true,
        "samples_truncated": false,
        "authority_side_effects": false,
        "negative_control": "no_effect_sample",
        "effects": [],
    })
}

fn project_effect_sample(intent: &IntentRow, effect: Option<&StoredObject>) -> Value {
    let stage = effect
        .map(|row| row.state.as_str().to_owned())
        .unwrap_or_else(|| "MISSING".to_owned());
    let body = effect.map(|row| &row.body);
    let observed = body
        .and_then(|value| value.get("observed_outcome"))
        .and_then(Value::as_str);
    let mutation_count = match stage.as_str() {
        "NOT_EXECUTED" | "DENIED" | "PROPOSED" | "AUTHORIZED" => Some(0u8),
        "EXECUTED" | "RECONCILED" | "VERIFIED" | "VERIFY_FAILED" => Some(1u8),
        _ => None,
    };
    let opaque_ref = |value: Option<&Value>| {
        let reference = value.and_then(Value::as_str)?.trim();
        if reference.is_empty() {
            return None;
        }
        let lowered = reference.to_ascii_lowercase();
        if lowered.contains("receipt") || lowered.contains("parameter") {
            return None;
        }
        Some(reference.to_owned())
    };
    json!({
        "effect_ref": format!("effect://{}", intent.effect_object_id),
        "original_key_digest": digest_bytes(intent.idempotency_key.as_bytes(), "cognitiveos.personal.observation-effect-key/0.1"),
        "stage": stage,
        "outcome_class": effect_outcome_class(&stage, observed),
        "reconcile_class": effect_reconcile_class(std::slice::from_ref(&stage)),
        "mutation_count": mutation_count,
        "fixed_post_state_ref": opaque_ref(body.and_then(|value| {
            value
                .get("fixed_post_state_ref")
                .or_else(|| value.pointer("/verification/fixed_post_state_ref"))
        })),
        "report_ref": opaque_ref(body.and_then(|value| {
            value
                .get("reconciliation_report_ref")
                .or_else(|| value.pointer("/verification/report_ref"))
        })),
    })
}

fn effect_outcome_class(stage: &str, observed: Option<&str>) -> &'static str {
    match stage {
        "OUTCOME_UNKNOWN" | "EXECUTING" | "MISSING" => "indeterminate",
        "NOT_EXECUTED" | "DENIED" | "PROPOSED" | "AUTHORIZED" => "not_executed",
        "VERIFY_FAILED" => "failed",
        "EXECUTED" | "RECONCILED" | "VERIFIED" => match observed {
            Some("failed") => "failed",
            Some("unknown") => "indeterminate",
            _ => "executed",
        },
        _ => "indeterminate",
    }
}

fn effect_reconcile_class(effect_states: &[String]) -> &'static str {
    if effect_states.is_empty() {
        return "not_applicable";
    }
    let is_durable = |state: &str| matches!(state, "RECONCILED" | "VERIFIED" | "VERIFY_FAILED");
    if effect_states.iter().all(|state| is_durable(state)) {
        return "closed";
    }
    if effect_states.iter().any(|state| is_durable(state)) {
        return "pending_reconciliation";
    }
    "must_reconcile"
}

fn project_o13(
    layout: &PersonalDataLayout,
    task_ref: &str,
    cursor: Option<&str>,
    expect_digest: Option<&str>,
) -> Result<Value, ObservationResponse> {
    let cursor_sequence = parse_cursor(cursor)?;
    if let Some(expected) = expect_digest
        && !expected.starts_with("sha256:")
    {
        return Err(error(
            400,
            "TASK_OBSERVATION_DIGEST_INVALID",
            "expect_digest must be a sha256 digest",
        ));
    }
    let Some(store) = open_existing_authority_store(layout)? else {
        if cursor_sequence > 0 {
            return Err(error(
                409,
                "TASK_OBSERVATION_CURSOR_STALE",
                "requested audit cursor is beyond the retained high watermark",
            ));
        }
        if let Some(expected) = expect_digest
            && expected != GENESIS_DIGEST
        {
            return Err(error(
                409,
                "TASK_OBSERVATION_DIGEST_BREAK",
                "replay chain digest does not match expect_digest",
            ));
        }
        return Ok(empty_o13(task_ref, 0, GENESIS_DIGEST));
    };
    let (high_watermark, watermark_truncated) = scan_high_watermark(&store)?;
    if cursor_sequence > high_watermark {
        return Err(error(
            409,
            "TASK_OBSERVATION_CURSOR_STALE",
            "requested audit cursor is beyond the retained high watermark",
        ));
    }
    if cursor_sequence > 0 {
        let at_cursor = store
            .read_events(cursor_sequence.saturating_sub(1), 1)
            .map_err(|_| {
                error(
                    503,
                    "TASK_OBSERVATION_READ_FAILED",
                    "durable audit cursor could not be read",
                )
            })?;
        if at_cursor
            .first()
            .is_none_or(|event| event.sequence != cursor_sequence)
        {
            return Err(error(
                409,
                "TASK_OBSERVATION_EVENT_MISSING",
                "requested audit cursor does not identify a retained event",
            ));
        }
    }
    let (window, gap_detected, truncated) =
        read_audit_window(&store, cursor_sequence, MAX_RETURNED_SAMPLES)?;
    if gap_detected {
        return Err(error(
            409,
            "TASK_OBSERVATION_EVENT_GAP",
            "durable audit sequence has a gap in the requested window",
        ));
    }
    let chain_head = chain_head_digest(cursor_sequence, &window);
    if let Some(expected) = expect_digest
        && expected != chain_head
    {
        return Err(error(
            409,
            "TASK_OBSERVATION_DIGEST_BREAK",
            "replay chain digest does not match expect_digest",
        ));
    }
    let related_ids = task_related_object_ids(&store, task_ref)?;
    let task_events: Vec<&CommittedEvent> = window
        .iter()
        .filter(|event| related_ids.contains(event.object_id.as_str()))
        .collect();
    let bounded = bound_audit_events(&task_events, MAX_RETURNED_SAMPLES);
    let observed_zero = bounded.events.is_empty();
    Ok(json!({
        "schema_version": 1,
        "kind": "observation.plane",
        "family": "o13",
        "task_ref": task_ref,
        "denominator": bounded.events.len() as u64,
        "observed_zero": observed_zero,
        "samples_truncated": truncated || watermark_truncated || bounded.truncated,
        "authority_side_effects": false,
        "negative_control": if observed_zero {
            "no_audit_sample"
        } else {
            "audit_replay_recorded"
        },
        "cursor": cursor_sequence,
        "high_watermark": high_watermark,
        "chain_head_digest": chain_head,
        "gap_detected": false,
        "events": bounded.events.iter().map(|event| json!({
            "sequence": event.sequence,
            "event_ref": format!("event://{}", event.event_id),
            "event_type": event.event_type,
            "domain": event.domain.as_str(),
            "event_digest": event_digest(event),
        })).collect::<Vec<_>>(),
    }))
}

fn empty_o13(task_ref: &str, cursor: i64, chain_head: &str) -> Value {
    json!({
        "schema_version": 1,
        "kind": "observation.plane",
        "family": "o13",
        "task_ref": task_ref,
        "denominator": 0,
        "observed_zero": true,
        "samples_truncated": false,
        "authority_side_effects": false,
        "negative_control": "no_audit_sample",
        "cursor": cursor,
        "high_watermark": 0,
        "chain_head_digest": chain_head,
        "gap_detected": false,
        "events": [],
    })
}

struct BoundedAudit<'a> {
    events: Vec<&'a CommittedEvent>,
    truncated: bool,
}

fn bound_audit_events<'a>(events: &[&'a CommittedEvent], max: usize) -> BoundedAudit<'a> {
    let truncated = events.len() > max;
    BoundedAudit {
        events: events.iter().copied().take(max).collect(),
        truncated,
    }
}

fn parse_cursor(cursor: Option<&str>) -> Result<i64, ObservationResponse> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let parsed = cursor.parse::<i64>().map_err(|_| {
        error(
            400,
            "TASK_OBSERVATION_CURSOR_INVALID",
            "cursor must be a non-negative integer sequence",
        )
    })?;
    if parsed < 0 {
        return Err(error(
            400,
            "TASK_OBSERVATION_CURSOR_INVALID",
            "cursor must be a non-negative integer sequence",
        ));
    }
    Ok(parsed)
}

fn open_existing_authority_store(
    layout: &PersonalDataLayout,
) -> Result<Option<SqliteAuthorityStore>, ObservationResponse> {
    let path = layout.authority_database_path();
    if !path.exists() {
        return Ok(None);
    }
    SqliteAuthorityStore::open(&path).map(Some).map_err(|_| {
        error(
            503,
            "TASK_AUTHORITY_STORE_UNAVAILABLE",
            "durable authority store is unavailable",
        )
    })
}

fn scan_high_watermark(store: &SqliteAuthorityStore) -> Result<(i64, bool), ObservationResponse> {
    let mut after_sequence = 0i64;
    let mut scanned = 0usize;
    loop {
        let remaining = MAX_AUDIT_EVENTS_SCANNED.saturating_sub(scanned);
        if remaining == 0 {
            return Ok((after_sequence, true));
        }
        let batch_limit = remaining.min(AUDIT_EVENT_BATCH_SIZE);
        let events = store
            .read_events(after_sequence, batch_limit)
            .map_err(|_| {
                error(
                    503,
                    "TASK_OBSERVATION_READ_FAILED",
                    "durable audit high watermark could not be read",
                )
            })?;
        if events.is_empty() {
            return Ok((after_sequence, false));
        }
        scanned += events.len();
        after_sequence = events
            .last()
            .map(|event| event.sequence)
            .unwrap_or(after_sequence);
        if events.len() < batch_limit {
            return Ok((after_sequence, false));
        }
    }
}

fn read_audit_window(
    store: &SqliteAuthorityStore,
    after_sequence: i64,
    limit: usize,
) -> Result<(Vec<CommittedEvent>, bool, bool), ObservationResponse> {
    let fetch_limit = limit.saturating_add(1);
    let events = store
        .read_events(after_sequence, fetch_limit)
        .map_err(|_| {
            error(
                503,
                "TASK_OBSERVATION_READ_FAILED",
                "durable audit cursor could not be read",
            )
        })?;
    let truncated = events.len() > limit;
    let window: Vec<CommittedEvent> = events.into_iter().take(limit).collect();
    let mut previous: Option<i64> = if after_sequence > 0 {
        Some(after_sequence)
    } else {
        None
    };
    for event in &window {
        if let Some(prev) = previous
            && event.sequence != prev + 1
        {
            return Ok((window, true, truncated));
        }
        previous = Some(event.sequence);
    }
    Ok((window, false, truncated))
}

fn chain_head_digest(cursor: i64, events: &[CommittedEvent]) -> String {
    let mut previous = if cursor == 0 {
        GENESIS_DIGEST.to_owned()
    } else {
        digest_bytes(
            format!("cursor:{cursor}").as_bytes(),
            "cognitiveos.personal.observation-audit-cursor/0.1",
        )
    };
    for event in events {
        previous = digest_bytes(
            format!("{previous}\n{}", event_digest(event)).as_bytes(),
            "cognitiveos.personal.observation-audit-chain/0.1",
        );
    }
    previous
}

fn event_digest(event: &CommittedEvent) -> String {
    digest_bytes(
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            event.sequence,
            event.event_id,
            event.domain.as_str(),
            event.object_id,
            event.event_type,
            event.canonical_json
        )
        .as_bytes(),
        "cognitiveos.personal.observation-audit-event/0.1",
    )
}

fn task_related_object_ids(
    store: &SqliteAuthorityStore,
    task_ref: &str,
) -> Result<HashSet<String>, ObservationResponse> {
    let mut ids = HashSet::new();
    let contract_epoch = store.current_contract_epoch(task_ref).map_err(|_| {
        error(
            503,
            "TASK_OBSERVATION_READ_FAILED",
            "current TaskContract epoch could not be read",
        )
    })?;
    if contract_epoch == 0 {
        return Ok(ids);
    }
    if let Some(contract) = store
        .load_task_contract(task_ref, contract_epoch)
        .map_err(|_| {
            error(
                503,
                "TASK_OBSERVATION_READ_FAILED",
                "current TaskContract could not be read",
            )
        })?
    {
        ids.insert(contract.contract_id.to_string());
    }
    let intents = store
        .list_intents_for_task_binding(&TaskBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch,
        })
        .map_err(|_| {
            error(
                503,
                "TASK_OBSERVATION_READ_FAILED",
                "durable Intent bindings could not be read",
            )
        })?;
    for intent in intents {
        ids.insert(intent.intent_id.to_string());
        ids.insert(intent.effect_object_id.to_string());
    }
    Ok(ids)
}

fn append_sample(data_dir: &Path, sample: ObservationSample) {
    if sample.task_ref.is_empty() || sample.task_ref.len() > MAX_TASK_REF_CHARS {
        return;
    }
    let mut file = load_file(data_dir).unwrap_or_else(|| ObservationFile {
        schema: OBSERVATION_SCHEMA.to_owned(),
        samples: Vec::new(),
    });
    file.samples.push(sample);
    if file.samples.len() > MAX_SAMPLES {
        let overflow = file.samples.len() - MAX_SAMPLES;
        file.samples.drain(0..overflow);
    }
    let _ = persist_file(data_dir, &file);
}

fn load_file(data_dir: &Path) -> Option<ObservationFile> {
    let bytes = fs::read(data_dir.join(OBSERVATION_FILE_NAME)).ok()?;
    let file: ObservationFile = serde_json::from_slice(&bytes).ok()?;
    (file.schema == OBSERVATION_SCHEMA).then_some(file)
}

fn persist_file(data_dir: &Path, file: &ObservationFile) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    let encoded = serde_json::to_vec_pretty(file)?;
    let path = data_dir.join(OBSERVATION_FILE_NAME);
    let staging = data_dir.join(format!("{OBSERVATION_FILE_NAME}.staging"));
    {
        let mut handle = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&staging)?;
        handle.write_all(&encoded)?;
        handle.sync_all()?;
    }
    fs::rename(staging, path)
}

fn bound_data_dir() -> Option<PathBuf> {
    BOUND_DATA_DIR
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

fn percent_decode(value: &str) -> Result<String, ObservationResponse> {
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        match raw[index] {
            b'%' if index + 2 < raw.len() => {
                let hex = std::str::from_utf8(&raw[index + 1..index + 3]).map_err(|_| {
                    error(
                        400,
                        "TASK_OBSERVATION_QUERY_REQUIRED",
                        "query percent-encoding is invalid",
                    )
                })?;
                let decoded = u8::from_str_radix(hex, 16).map_err(|_| {
                    error(
                        400,
                        "TASK_OBSERVATION_QUERY_REQUIRED",
                        "query percent-encoding is invalid",
                    )
                })?;
                bytes.push(decoded);
                index += 1;
                index += 2;
            }
            b'+' => {
                bytes.push(b' ');
                index += 1;
            }
            byte => {
                bytes.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(bytes).map_err(|_| {
        error(
            400,
            "TASK_OBSERVATION_QUERY_REQUIRED",
            "query percent-encoding is invalid",
        )
    })
}

fn digest_bytes(bytes: &[u8], domain: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update(b"\n");
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn error(status: u16, code: &str, message: &str) -> ObservationResponse {
    ObservationResponse {
        status,
        body: json!({
            "error": { "code": code, "message": message }
        })
        .to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use cognitive_store::PersonalDataLayout;

    fn layout() -> PersonalDataLayout {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t26-obs-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("temp layout");
        PersonalDataLayout::from_xdg_roots(&root, &root, &root, &root, &root)
    }

    #[test]
    fn missing_samples_are_controlled_zeros_not_silent_counts() {
        let layout = layout();
        let response = handle(
            "GET /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let body: Value = serde_json::from_str(&response.body).expect("json");
        assert_eq!(body["observed_zero"], true);
        assert_eq!(body["denominator"], 0);
        assert_eq!(body["negative_control"], "no_authorization_sample");
        assert_eq!(body["grant_count"], 0);
        assert_eq!(body["deny_count"], 0);
    }

    #[test]
    fn prompt_restatement_and_unknown_family_fail_closed() {
        let layout = layout();
        let restated = handle(
            "GET /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&prompt=widen",
            &layout,
        );
        assert_eq!(restated.status, 400, "{}", restated.body);
        assert!(restated.body.contains("TASK_OBSERVATION_QUERY_FORBIDDEN"));

        let family = handle(
            "GET /task/observation?family=secret&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(family.status, 400, "{}", family.body);
        assert!(family.body.contains("TASK_OBSERVATION_FAMILY_INVALID"));
    }

    #[test]
    fn authorization_deny_is_an_active_negative_control() {
        let layout = layout();
        record_authorization_decision(
            layout.data_dir(),
            "task://personal/p2-t26",
            "workspace",
            "read_body",
            3,
            "read_body",
            "deny",
            "CONTEXT_AUTH_DENIED",
        );
        let response = handle(
            "GET /task/resource/v1/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let body: Value = serde_json::from_str(&response.body).expect("json");
        assert_eq!(body["family"], "o2");
        assert_eq!(body["deny_count"], 1);
        assert_eq!(body["grant_count"], 0);
        assert_eq!(body["denominator"], 1);
        assert_eq!(body["negative_control"], "deny_recorded");
        assert_eq!(body["observed_zero"], false);
        assert_eq!(body["samples"][0]["reason_code"], "CONTEXT_AUTH_DENIED");
        assert!(
            body["samples"][0]["input_digest"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
        assert!(body["samples"][0].get("capability").is_none());
        assert!(!response.body.contains("prompt"));
    }

    #[test]
    fn cache_revalidation_and_compaction_zeros_are_controlled() {
        let layout = layout();
        record_cache_sample(
            layout.data_dir(),
            "task://personal/p2-t26",
            "miss",
            4,
            1,
            2,
            "",
        );
        record_cache_sample(
            layout.data_dir(),
            "task://personal/p2-t26",
            "revalidated",
            4,
            1,
            0,
            "",
        );
        let other = handle(
            "GET /task/observation?family=o3&task_ref=task%3A%2F%2Fpersonal%2Fother",
            &layout,
        );
        let other_body: Value = serde_json::from_str(&other.body).expect("json");
        assert_eq!(other.status, 200, "{}", other.body);
        assert_eq!(other_body["observed_zero"], true);
        assert_eq!(other_body["cache"]["negative_control"], "no_cache_sample");
        let response = handle(
            "GET /task/observation?family=o3&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let body: Value = serde_json::from_str(&response.body).expect("json");
        assert_eq!(body["cache"]["class_counts"]["miss"], 1);
        assert_eq!(body["cache"]["class_counts"]["revalidated"], 1);
        assert_eq!(body["compaction"]["observed_zero"], true);
        assert_eq!(
            body["compaction"]["negative_control"],
            "compaction_not_invoked"
        );
        record_compaction_sample(
            layout.data_dir(),
            "task://personal/p2-t26",
            4,
            12,
            8,
            120,
            80,
            "sha256:loss",
        );
        let compacted = handle(
            "GET /task/observation?family=o3&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        let compacted_body: Value = serde_json::from_str(&compacted.body).expect("json");
        assert_eq!(compacted_body["compaction"]["observed_zero"], false);
        assert_eq!(
            compacted_body["compaction"]["samples"][0]["input_tokens"],
            12
        );
        assert_eq!(
            compacted_body["compaction"]["samples"][0]["loss_manifest_digest"],
            "sha256:loss"
        );
    }

    #[test]
    fn scheduler_zero_runnable_records_a_probe_and_names_missing_counters() {
        let layout = layout();
        record_scheduler_sample(
            layout.data_dir(),
            "task://personal/p2-t26",
            "runnable_count",
            2,
            0,
            "tick",
        );
        let response = handle(
            "GET /task/observation?family=o4&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let body: Value = serde_json::from_str(&response.body).expect("json");
        assert_eq!(body["counters"]["runnable_count"]["count"], 0);
        assert_eq!(body["counters"]["runnable_count"]["denominator"], 1);
        assert_eq!(body["counters"]["budget_stop"]["observed_zero"], true);
        assert_eq!(
            body["counters"]["budget_stop"]["negative_control"],
            "no_budget_stop_sample"
        );
        assert_eq!(body["denominator"], 1);
        assert_eq!(body["negative_control"], "scheduler_probe_recorded");
        assert_eq!(body["observed_zero"], false);
    }

    #[test]
    fn management_and_writes_are_channel_forbidden() {
        let forbidden = management_channel_forbidden();
        assert_eq!(forbidden.status, 403);
        assert!(
            forbidden
                .body
                .contains("RESOURCE_OBSERVATION_CHANNEL_FORBIDDEN")
        );
        let layout = layout();
        let posted = handle(
            "POST /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(posted.status, 403, "{}", posted.body);
        assert!(posted.body.contains("RESOURCE_OBSERVATION_WRITE_FORBIDDEN"));
    }

    #[test]
    fn effect_and_audit_empty_windows_are_controlled_zeros() {
        let layout = layout();
        let effects = handle(
            "GET /task/observation?family=o5&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(effects.status, 200, "{}", effects.body);
        let effects_body: Value = serde_json::from_str(&effects.body).expect("json");
        assert_eq!(effects_body["family"], "o5");
        assert_eq!(effects_body["observed_zero"], true);
        assert_eq!(effects_body["denominator"], 0);
        assert_eq!(effects_body["negative_control"], "no_effect_sample");
        assert!(effects_body["effects"].as_array().unwrap().is_empty());
        assert!(!effects.body.contains("receipt"));
        assert!(!effects.body.contains("parameters"));

        let audit = handle(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(audit.status, 200, "{}", audit.body);
        let audit_body: Value = serde_json::from_str(&audit.body).expect("json");
        assert_eq!(audit_body["family"], "o13");
        assert_eq!(audit_body["observed_zero"], true);
        assert_eq!(audit_body["negative_control"], "no_audit_sample");
        assert_eq!(audit_body["cursor"], 0);
        assert_eq!(audit_body["high_watermark"], 0);
        assert_eq!(audit_body["chain_head_digest"], "sha256:genesis");
        assert_eq!(audit_body["gap_detected"], false);
    }

    #[test]
    fn audit_cursor_digest_and_family_negatives_fail_closed() {
        let layout = layout();
        let stale = handle(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&cursor=9",
            &layout,
        );
        assert_eq!(stale.status, 409, "{}", stale.body);
        assert!(stale.body.contains("TASK_OBSERVATION_CURSOR_STALE"));

        let digest = handle(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&expect_digest=sha256%3Adeadbeef",
            &layout,
        );
        assert_eq!(digest.status, 409, "{}", digest.body);
        assert!(digest.body.contains("TASK_OBSERVATION_DIGEST_BREAK"));

        let on_o2 = handle(
            "GET /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&cursor=0",
            &layout,
        );
        assert_eq!(on_o2.status, 400, "{}", on_o2.body);
        assert!(on_o2.body.contains("TASK_OBSERVATION_QUERY_FORBIDDEN"));

        let invalid_cursor = handle(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&cursor=nope",
            &layout,
        );
        assert_eq!(invalid_cursor.status, 400, "{}", invalid_cursor.body);
        assert!(
            invalid_cursor
                .body
                .contains("TASK_OBSERVATION_CURSOR_INVALID")
        );

        let empty: Vec<&CommittedEvent> = Vec::new();
        let truncated = bound_audit_events(&empty, 1);
        assert!(truncated.events.is_empty());
        assert!(!truncated.truncated);
    }

    #[test]
    fn audit_replay_is_stable_across_store_reopen() {
        let layout = layout();
        let first = handle(
            "GET /task/observation?family=audit&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        let second = handle(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26",
            &layout,
        );
        assert_eq!(first.status, 200, "{}", first.body);
        assert_eq!(second.status, 200, "{}", second.body);
        let first_body: Value = serde_json::from_str(&first.body).expect("json");
        let second_body: Value = serde_json::from_str(&second.body).expect("json");
        assert_eq!(
            first_body["chain_head_digest"],
            second_body["chain_head_digest"]
        );
        assert_eq!(first_body["high_watermark"], second_body["high_watermark"]);
    }
}
