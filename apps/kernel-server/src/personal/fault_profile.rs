//! P2-T24 default-off authorized Effect fault profiles.
//!
//! Management callers may persist a task-scoped profile. Task-channel callers
//! are denied before this module runs. Missing files mean every fault is off.
//! This surface does not return receipts, raw parameters, or idempotency keys.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use cognitive_domain::UriRef;
use cognitive_kernel::ports::ProtocolStore;
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore};
use serde::{Deserialize, Serialize};
use serde_json::json;

const FAULT_PROFILE_FILE_NAME: &str = "personal-fault-profiles.json";
const FAULT_PROFILE_SCHEMA: &str = "cognitiveos.personal.fault-profile/0.1";
const MAX_PROFILES: usize = 32;
const MAX_CAMPAIGN_ID_CHARS: usize = 32;
const MAX_CASE_REF_CHARS: usize = 64;

/// Variant names keep the plan's before-stage suffix, matching P2-T17.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthorizedFaultPoint {
    DispatchBefore,
    MutationAfterReceiptBefore,
    ReceiptAfterEffectCloseBefore,
    VerificationBefore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FaultProfileRecord {
    pub task_ref: String,
    pub campaign_id: String,
    pub case_ref: String,
    pub faults_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_point: Option<AuthorizedFaultPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FaultProfileFile {
    schema: String,
    profiles: Vec<FaultProfileRecord>,
}

pub(crate) struct FaultProfileResponse {
    pub status: u16,
    pub body: String,
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
) -> FaultProfileResponse {
    if method_path.starts_with("GET /management/resource/v1/fault-profile") {
        return get_profile(method_path, layout, store);
    }
    if method_path.starts_with("POST /management/resource/v1/fault-profile") {
        return put_profile(body, layout, store);
    }
    error(
        404,
        "RESOURCE_FAULT_PROFILE_ROUTE_NOT_FOUND",
        "no fault-profile route matched",
    )
}

pub(crate) fn task_channel_forbidden() -> FaultProfileResponse {
    error(
        403,
        "RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN",
        "ordinary task callers cannot enable or inspect fault profiles",
    )
}

fn get_profile(
    method_path: &str,
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
) -> FaultProfileResponse {
    let task_ref = match required_task_ref_query(method_path) {
        Ok(task_ref) => task_ref,
        Err(response) => return response,
    };
    if let Err(response) = require_existing_task(store, &task_ref) {
        return response;
    }
    let profile = match load_profile(layout, &task_ref) {
        Ok(Some(profile)) => profile,
        Ok(None) => FaultProfileRecord {
            task_ref: task_ref.clone(),
            campaign_id: String::new(),
            case_ref: String::new(),
            faults_enabled: false,
            fault_point: None,
        },
        Err(response) => return response,
    };
    ok(json!({
        "schema_version": 1,
        "task_ref": profile.task_ref,
        "campaign_id": profile.campaign_id,
        "case_ref": profile.case_ref,
        "faults_enabled": profile.faults_enabled,
        "fault_point": profile.fault_point,
        "authority_side_effects": false,
    }))
}

fn put_profile(
    body: &[u8],
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
) -> FaultProfileResponse {
    let document = match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(document) => document,
        Err(_) => {
            return error(
                400,
                "RESOURCE_FAULT_PROFILE_PAYLOAD_INVALID",
                "fault profile payload is invalid",
            );
        }
    };
    let Some(task_ref) = document.get("task_ref").and_then(|value| value.as_str()) else {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_TASK_REF_REQUIRED",
            "task_ref is required",
        );
    };
    if UriRef::parse(task_ref).is_err() {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_TASK_REF_INVALID",
            "task_ref must be a canonical URI",
        );
    }
    let Some(campaign_id) = document
        .get("campaign_id")
        .and_then(|value| value.as_str())
        .map(str::trim)
    else {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_UNAUTHORIZED",
            "campaign_id is required",
        );
    };
    if !valid_campaign_id(campaign_id) {
        return error(
            403,
            "RESOURCE_FAULT_PROFILE_UNAUTHORIZED",
            "campaign is not authorized to enable fault profiles",
        );
    }
    let Some(case_ref) = document
        .get("case_ref")
        .and_then(|value| value.as_str())
        .map(str::trim)
    else {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_CASE_REF_REQUIRED",
            "case_ref is required",
        );
    };
    if !valid_case_ref(case_ref) {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_CASE_REF_INVALID",
            "case_ref is not a bounded campaign case identifier",
        );
    }
    let faults_enabled = document
        .get("faults_enabled")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let fault_point = match document.get("fault_point") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match serde_json::from_value::<AuthorizedFaultPoint>(value.clone()) {
            Ok(point) => Some(point),
            Err(_) => {
                return error(
                    400,
                    "RESOURCE_FAULT_PROFILE_POINT_INVALID",
                    "fault_point must be one of the four fixed authorized points",
                );
            }
        },
    };
    if faults_enabled && fault_point.is_none() {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_POINT_REQUIRED",
            "enabling faults requires one fixed authorized fault_point",
        );
    }
    if !faults_enabled && fault_point.is_some() {
        return error(
            400,
            "RESOURCE_FAULT_PROFILE_POINT_FORBIDDEN",
            "default-off profiles must not name a fault_point",
        );
    }
    if let Err(response) = require_existing_task(store, task_ref) {
        return response;
    }
    let record = FaultProfileRecord {
        task_ref: task_ref.to_owned(),
        campaign_id: campaign_id.to_owned(),
        case_ref: case_ref.to_owned(),
        faults_enabled,
        fault_point,
    };
    if let Err(response) = persist_profile(layout, record.clone()) {
        return response;
    }
    FaultProfileResponse {
        status: 200,
        body: json!({
            "schema_version": 1,
            "task_ref": record.task_ref,
            "campaign_id": record.campaign_id,
            "case_ref": record.case_ref,
            "faults_enabled": record.faults_enabled,
            "fault_point": record.fault_point,
            "authority_side_effects": true,
        })
        .to_string(),
    }
}

fn require_existing_task(
    store: &SqliteAuthorityStore,
    task_ref: &str,
) -> Result<(), FaultProfileResponse> {
    match store.current_contract_epoch(task_ref) {
        Ok(epoch) if epoch > 0 => Ok(()),
        Ok(_) => Err(error(
            404,
            "RESOURCE_FAULT_PROFILE_TASK_NOT_FOUND",
            "no durable TaskContract exists for task_ref",
        )),
        Err(_) => Err(error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "durable authority store is unavailable",
        )),
    }
}

fn required_task_ref_query(method_path: &str) -> Result<String, FaultProfileResponse> {
    let query = method_path
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.split_once('?').map(|(_, query)| query))
        .ok_or_else(|| {
            error(
                400,
                "RESOURCE_FAULT_PROFILE_TASK_REF_REQUIRED",
                "task_ref query parameter is required",
            )
        })?;
    let mut task_ref = None;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if name != "task_ref" {
            return Err(error(
                400,
                "RESOURCE_FAULT_PROFILE_QUERY_FORBIDDEN",
                "fault profile queries accept only task_ref",
            ));
        }
        if task_ref.replace(percent_decode(value)?).is_some() {
            return Err(error(
                400,
                "RESOURCE_FAULT_PROFILE_TASK_REF_REQUIRED",
                "exactly one task_ref query parameter is required",
            ));
        }
    }
    let Some(task_ref) = task_ref.filter(|value| !value.trim().is_empty()) else {
        return Err(error(
            400,
            "RESOURCE_FAULT_PROFILE_TASK_REF_REQUIRED",
            "exactly one task_ref query parameter is required",
        ));
    };
    if UriRef::parse(&task_ref).is_err() {
        return Err(error(
            400,
            "RESOURCE_FAULT_PROFILE_TASK_REF_INVALID",
            "task_ref must be a canonical URI",
        ));
    }
    Ok(task_ref)
}

fn percent_decode(value: &str) -> Result<String, FaultProfileResponse> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(error(
                    400,
                    "RESOURCE_FAULT_PROFILE_TASK_REF_INVALID",
                    "task_ref contains invalid percent encoding",
                ));
            }
            let high = hex_value(bytes[index + 1]);
            let low = hex_value(bytes[index + 2]);
            let (Some(high), Some(low)) = (high, low) else {
                return Err(error(
                    400,
                    "RESOURCE_FAULT_PROFILE_TASK_REF_INVALID",
                    "task_ref contains invalid percent encoding",
                ));
            };
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| {
        error(
            400,
            "RESOURCE_FAULT_PROFILE_TASK_REF_INVALID",
            "task_ref must be UTF-8",
        )
    })
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn campaign_is_authorized(campaign_id: &str) -> bool {
    valid_campaign_id(campaign_id)
}

pub(crate) fn case_ref_is_authorized(case_ref: &str) -> bool {
    valid_case_ref(case_ref)
}

/// Production consult: missing, default-off, or unauthorized file content never
/// injects. Enabled profiles must name one of the four fixed points.
pub(crate) fn load_enabled_authorized_profile(
    data_dir: &Path,
    task_ref: &str,
) -> Option<FaultProfileRecord> {
    let path = data_dir.join(FAULT_PROFILE_FILE_NAME);
    if !path.exists() {
        return None;
    }
    let file = read_file(&path).ok()?;
    let profile = file
        .profiles
        .into_iter()
        .find(|profile| profile.task_ref == task_ref)?;
    if !profile.faults_enabled {
        return None;
    }
    if !valid_campaign_id(&profile.campaign_id) || !valid_case_ref(&profile.case_ref) {
        return None;
    }
    if profile.fault_point.is_none() {
        return None;
    }
    Some(profile)
}

pub(crate) fn authorized_injection_point(
    data_dir: &Path,
    task_ref: &str,
) -> Option<AuthorizedFaultPoint> {
    load_enabled_authorized_profile(data_dir, task_ref).and_then(|profile| profile.fault_point)
}

#[cfg(test)]
pub(crate) fn write_profile_record(
    data_dir: &Path,
    record: FaultProfileRecord,
) -> Result<(), String> {
    fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;
    persist_profile_at(data_dir, record)
        .map_err(|_| "fault profile could not be persisted for the D02 consult test".to_owned())
}

fn persist_profile_at(
    data_dir: &Path,
    record: FaultProfileRecord,
) -> Result<(), FaultProfileResponse> {
    let path = data_dir.join(FAULT_PROFILE_FILE_NAME);
    let mut file = if path.exists() {
        read_file(&path)?
    } else {
        FaultProfileFile {
            schema: FAULT_PROFILE_SCHEMA.to_owned(),
            profiles: Vec::new(),
        }
    };
    if file.schema != FAULT_PROFILE_SCHEMA {
        return Err(error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "fault profile schema is not current",
        ));
    }
    file.profiles
        .retain(|profile| profile.task_ref != record.task_ref);
    if file.profiles.len() >= MAX_PROFILES {
        return Err(error(
            409,
            "RESOURCE_FAULT_PROFILE_CAPACITY",
            "bounded fault profile capacity would be exceeded",
        ));
    }
    file.profiles.push(record);
    write_file_atomically(&path, &file)
}

fn valid_campaign_id(campaign_id: &str) -> bool {
    if campaign_id.is_empty() || campaign_id.len() > MAX_CAMPAIGN_ID_CHARS {
        return false;
    }
    let allowed = campaign_id
        .bytes()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-');
    allowed && (campaign_id.starts_with("PERSONAL-PERF-EVAL-") || campaign_id == "P2-T24")
}

fn valid_case_ref(case_ref: &str) -> bool {
    !case_ref.is_empty()
        && case_ref.len() <= MAX_CASE_REF_CHARS
        && case_ref
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn profile_path(layout: &PersonalDataLayout) -> std::path::PathBuf {
    layout.data_dir().join(FAULT_PROFILE_FILE_NAME)
}

fn load_profile(
    layout: &PersonalDataLayout,
    task_ref: &str,
) -> Result<Option<FaultProfileRecord>, FaultProfileResponse> {
    let path = profile_path(layout);
    if !path.exists() {
        return Ok(None);
    }
    let file = read_file(&path)?;
    Ok(file
        .profiles
        .into_iter()
        .find(|profile| profile.task_ref == task_ref))
}

fn persist_profile(
    layout: &PersonalDataLayout,
    record: FaultProfileRecord,
) -> Result<(), FaultProfileResponse> {
    layout.ensure_directories().map_err(|_| {
        error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "fault profile directory is unavailable",
        )
    })?;
    persist_profile_at(layout.data_dir(), record)
}

fn read_file(path: &Path) -> Result<FaultProfileFile, FaultProfileResponse> {
    let bytes = fs::read(path).map_err(|_| {
        error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "fault profile file cannot be read",
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "fault profile file is corrupt",
        )
    })
}

fn write_file_atomically(path: &Path, file: &FaultProfileFile) -> Result<(), FaultProfileResponse> {
    let bytes = serde_json::to_vec(file).map_err(|_| {
        error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "fault profile cannot be serialized",
        )
    })?;
    let temporary_path = path.with_extension("json.tmp");
    let mut handle = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|_| {
            error(
                503,
                "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
                "fault profile cannot be created",
            )
        })?;
    handle
        .write_all(&bytes)
        .and_then(|()| handle.sync_all())
        .map_err(|_| {
            error(
                503,
                "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
                "fault profile cannot be persisted",
            )
        })?;
    fs::rename(&temporary_path, path).map_err(|_| {
        error(
            503,
            "RESOURCE_FAULT_PROFILE_STORE_UNAVAILABLE",
            "fault profile cannot be committed",
        )
    })
}

fn ok(value: serde_json::Value) -> FaultProfileResponse {
    FaultProfileResponse {
        status: 200,
        body: value.to_string(),
    }
}

fn error(status: u16, code: &str, message: &str) -> FaultProfileResponse {
    FaultProfileResponse {
        status,
        body: json!({"status":"error", "code": code, "message": message}).to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_campaign_cannot_enable_faults() {
        assert!(!valid_campaign_id("owner-local"));
        assert!(!valid_campaign_id(""));
        assert!(valid_campaign_id("P2-T24"));
        assert!(valid_campaign_id("PERSONAL-PERF-EVAL-004"));
    }

    #[test]
    fn default_off_profile_serializes_without_receipt_or_parameters() {
        let record = FaultProfileRecord {
            task_ref: "task://personal/example".to_owned(),
            campaign_id: "P2-T24".to_owned(),
            case_ref: "BR-04-D01".to_owned(),
            faults_enabled: false,
            fault_point: None,
        };
        let serialized = serde_json::to_string(&record).expect("serialize");
        for forbidden in ["receipt", "parameters", "idempotency"] {
            assert!(!serialized.to_ascii_lowercase().contains(forbidden));
        }
        assert!(serialized.contains("\"faults_enabled\":false"));
    }

    #[test]
    fn missing_and_default_off_profiles_never_inject() {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t24-consult-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("temp data dir");
        let task_ref = "task://personal/p2-t24/consult";
        assert_eq!(authorized_injection_point(&root, task_ref), None);
        write_profile_record(
            &root,
            FaultProfileRecord {
                task_ref: task_ref.to_owned(),
                campaign_id: "P2-T24".to_owned(),
                case_ref: "BR-04-D02".to_owned(),
                faults_enabled: false,
                fault_point: None,
            },
        )
        .expect("write default-off");
        assert_eq!(authorized_injection_point(&root, task_ref), None);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn unauthorized_campaign_file_never_injects() {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t24-unauth-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("temp data dir");
        let task_ref = "task://personal/p2-t24/unauth";
        let path = root.join(FAULT_PROFILE_FILE_NAME);
        fs::write(
            &path,
            r#"{"schema":"cognitiveos.personal.fault-profile/0.1","profiles":[{"task_ref":"task://personal/p2-t24/unauth","campaign_id":"owner-local","case_ref":"spoof","faults_enabled":true,"fault_point":"dispatch_before"}]}"#,
        )
        .expect("write spoofed profile");
        assert_eq!(authorized_injection_point(&root, task_ref), None);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn authorized_enabled_profile_exposes_the_pinned_point() {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t24-on-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("temp data dir");
        let task_ref = "task://personal/p2-t24/enabled";
        write_profile_record(
            &root,
            FaultProfileRecord {
                task_ref: task_ref.to_owned(),
                campaign_id: "P2-T24".to_owned(),
                case_ref: "BR-04-D02".to_owned(),
                faults_enabled: true,
                fault_point: Some(AuthorizedFaultPoint::MutationAfterReceiptBefore),
            },
        )
        .expect("write enabled profile");
        assert_eq!(
            authorized_injection_point(&root, task_ref),
            Some(AuthorizedFaultPoint::MutationAfterReceiptBefore)
        );
        fs::remove_dir_all(&root).ok();
    }
}
